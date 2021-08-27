//! Handles events from the Matrix server; mostly just for parsing commands.

use crate::commands::*;
use crate::message::MessageData;
use anyhow::Result;
use matrix_sdk::{
    async_trait,
    events::{
        room::member::MemberEventContent,
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, StrippedStateEvent, SyncMessageEvent,
    },
    room::Room,
    Client, EventHandler,
};
use rand::distributions::Uniform;
use rand::prelude::*;
use std::collections::VecDeque;
use tokio::task::spawn_blocking;
use tokio::time::{sleep, Duration};
use yarrbot_db::DbPool;
use yarrbot_db::{actions::user_actions::UserActions, models::User};

const YARRBOT_COMMAND: &str = "!yarrbot";

async fn user_exists(pool: &DbPool, username: &str) -> Result<bool> {
    let conn = pool.get()?;
    let username2 = String::from(username);
    let u = spawn_blocking(move || User::try_get_by_username(&conn, username2.as_str())).await??;
    match u {
        Some(u) => {
            debug!("User \"{}\" exists with ID {}.", username, u.id.to_string());
            Ok(true)
        }
        None => Ok(false),
    }
}

pub struct CommandMetadata {
    pub user: String,
    pub is_direct_message: bool,
}

pub struct CommandParser {
    client: Client,
    pool: DbPool,
}

impl CommandParser {
    pub fn new(client: Client, pool: DbPool) -> Self {
        Self { client, pool }
    }

    async fn execute_command(
        &self,
        command: &str,
        metadata: CommandMetadata,
        data: VecDeque<&str>,
    ) -> Result<MessageData> {
        let result = match command {
            "ping" => ping_command::get_message(),
            "webhook" => {
                webhook_command::handle_webhook_command(metadata, &self.client, &self.pool, data)
                    .await?
            }
            _ => MessageData::from("Unrecognized command."),
        };
        Ok(result)
    }
}

#[async_trait]
impl EventHandler for CommandParser {
    async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        // Don't respond to messages posted by our bot.
        if event.sender == self.client.user_id().await.unwrap() {
            return;
        }

        // Based off of: https://github.com/matrix-org/matrix-rust-sdk/blob/0.3.0/matrix_sdk/examples/command_bot.rs
        if let Room::Joined(room) = room {
            let message_body = if let SyncMessageEvent {
                content:
                    MessageEventContent {
                        msgtype:
                            MessageType::Text(TextMessageEventContent {
                                body: message_body, ..
                            }),
                        ..
                    },
                ..
            } = event
            {
                message_body
            } else {
                return;
            };
            let mut split = message_body.split_whitespace();
            let first = split.next().unwrap_or("").to_lowercase();
            if first == YARRBOT_COMMAND {
                let key = split.next().unwrap_or("").to_lowercase();
                let message_data = if !key.is_empty() {
                    let metadata = CommandMetadata {
                        user: event.sender.to_string(),
                        // Note: room.is_direct() doesn't return true when expected.
                        // This works around the issue.
                        is_direct_message: match room.members().await {
                            Ok(v) => v.len() == 2,
                            Err(e) => {
                                error!("Failed to retrieve the number of members in the room with ID {}: {:?}", room.room_id().as_str(), e);
                                return;
                            }
                        },
                    };
                    let data: VecDeque<&str> = split.collect();
                    self.execute_command(key.as_str(), metadata, data)
                        .await
                        .unwrap_or_else(|e| e.into())
                } else {
                    MessageData::from("Unrecognized command.")
                };
                let send_result = room
                    .send(
                        AnyMessageEventContent::RoomMessage(message_data.into()),
                        None,
                    )
                    .await;
                if let Err(e) = send_result {
                    error!("Encountered error while responding to command: {:?}", e);
                }
            }
        }
    }

    async fn on_stripped_state_member(
        &self,
        room: Room,
        room_member: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        // Based off of https://github.com/matrix-org/matrix-rust-sdk/blob/0.3.0/matrix_sdk/examples/autojoin.rs

        // Don't respond to invites if they're not meant for us.
        if room_member.state_key != self.client.user_id().await.unwrap() {
            return;
        }

        if let Room::Invited(room) = room {
            // Synapse has a bug where the bot can receive an invite, but the server isn't ready
            // for the bot to join the room.
            // https://github.com/matrix-org/synapse/issues/4345

            // Don't let users that aren't admins invite the bot to rooms.
            let room_name = room
                .display_name()
                .await
                .unwrap_or_else(|_| String::from("(Unknown Name)"));
            let room_id = room.room_id().as_str();
            let username = room_member.sender.as_str();
            match user_exists(&self.pool, username).await {
                Ok(exists) => {
                    if !exists {
                        match room.reject_invitation().await {
                            Ok(_) => {
                                warn!("\"{}\" attempted to invite the bot to room \"{}\" ({}) but is not authorized to do so.", username, &room_name, room_id);
                            }
                            Err(e) => {
                                error!("Failed to reject room invitation from \"{}\" ({}) for room \"{}\": {:?}", username, &room_name, room_id, e);
                            }
                        };
                        return;
                    }
                }
                Err(e) => {
                    error!(
                        "Error encountered while checking if inviting user exists: {:?}",
                        e
                    );
                    return;
                }
            };

            let mut last_error: Option<matrix_sdk::Error> = None;
            let mut rng: SmallRng = SmallRng::from_entropy();
            let dist = Uniform::new_inclusive(0, 1000);
            for i in 0..5 {
                match room.accept_invitation().await {
                    Ok(_) => {
                        info!(
                            "Joined room \"{}\" ({}) after invitation from \"{}\".",
                            &room_name, room_id, username
                        );
                        return;
                    }
                    Err(e) => {
                        let base: u64 = u64::pow(2, i) * 100;
                        let jitter: u64 = dist.sample(&mut rng);
                        let delay = base + jitter;
                        debug!(
                            "Encountered error while attempting to join room \"{}\" ({}), delaying for {} ms: {:?}",
                            room_id, &room_name, delay, e
                        );
                        sleep(Duration::from_millis(delay)).await;
                        last_error = Some(e);
                    }
                }
            }

            error!(
                "Failed to join room \"{}\" ({}) after five attempts; last error encountered: {:?}",
                room_id,
                &room_name,
                last_error.unwrap()
            );
        }
    }
}
