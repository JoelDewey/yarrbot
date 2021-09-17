//! Handles events from the Matrix server; mostly just for parsing commands.

use crate::commands::*;
use crate::message::MessageData;
use anyhow::Result;
use matrix_sdk::{
    room::Room,
    ruma::events::{
        room::member::MemberEventContent,
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, StrippedStateEvent, SyncMessageEvent,
    },
    Client,
};
use rand::distributions::Uniform;
use rand::prelude::*;
use std::collections::VecDeque;
use tokio::task::spawn_blocking;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use yarrbot_db::DbPool;
use yarrbot_db::{actions::user_actions::UserActions, models::User};

const YARRBOT_COMMAND: &str = "!yarrbot";

/// Metadata to use when executing the command.
#[derive(Debug)]
pub struct CommandMetadata {
    pub user: String,
    pub is_direct_message: bool,
}

/// Parses commands and reacts to events from the Matrix homeserver.
pub struct CommandParser {
    client: Client,
    pool: DbPool,
}

impl CommandParser {
    /// Create a new [CommandParser]. It requires a Matrix [Client] and a
    /// [DbPool] of connections to the database.
    pub fn new(client: Client, pool: DbPool) -> Self {
        Self { client, pool }
    }

    #[tracing::instrument(skip(self, room, event), fields(event.sender = %event.sender, room.room_id = %room.room_id(), room.name))]
    pub async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        let room_name = room.name().unwrap_or_else(|| String::from("(No Name)"));
        tracing::Span::current().record("room.name", &room_name.as_str());
        // Don't respond to messages posted by our bot.
        if event.sender == self.client.user_id().await.unwrap() {
            debug!("Ignoring message posted by the bot itself.");
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
                debug!("Matrix message body is not correct.");
                return;
            };
            let mut split = message_body.split_whitespace();
            let first = split.next().unwrap_or("").to_lowercase();
            if first == YARRBOT_COMMAND {
                info!("Received !yarrbot command.");
                let key = split.next().unwrap_or("").to_lowercase();
                let message_data = if !key.is_empty() {
                    let metadata = CommandMetadata {
                        user: event.sender.to_string(),
                        // Note: room.is_direct() doesn't return true when expected.
                        // This works around the issue.
                        is_direct_message: match room.members().await {
                            Ok(v) => v.len() == 2,
                            Err(e) => {
                                error!(error = ?e, "Failed to retrieve the number of members in the room.");
                                return;
                            }
                        },
                    };
                    let data: VecDeque<&str> = split.collect();
                    self.execute_command(key.as_str(), metadata, data)
                        .await
                        .unwrap_or_else(|e| e.into())
                } else {
                    debug!(key = &key.as_str(), "Command unrecognized.");
                    MessageData::from("Unrecognized command.")
                };

                info!("Sending response to command.");
                let send_result = room
                    .send(
                        AnyMessageEventContent::RoomMessage(message_data.into()),
                        None,
                    )
                    .await;
                if let Err(e) = send_result {
                    error!(error = ?e, "Encountered error while responding to command.");
                }
            } else {
                debug!("Received first token \"{}\", ignoring.", &first);
            }
        }
    }

    #[tracing::instrument(skip(self, room, room_member), fields(room.room_id = %room.room_id(), room.name, username = %room_member.sender))]
    pub async fn on_stripped_state_member(
        &self,
        room: Room,
        room_member: &StrippedStateEvent<MemberEventContent>,
    ) {
        let room_name = room.name().unwrap_or_else(|| String::from("(No Name)"));
        tracing::Span::current().record("room.name", &room_name.as_str());
        // Based off of https://github.com/matrix-org/matrix-rust-sdk/blob/0.3.0/matrix_sdk/examples/autojoin.rs

        // Don't respond to invites if they're not meant for us.
        if room_member.state_key != self.client.user_id().await.unwrap() {
            debug!("Received invite that's not meant for the bot.");
            return;
        }

        if let Room::Invited(room) = room {
            // Don't let users that aren't admins invite the bot to rooms.
            let username = room_member.sender.as_str();
            match user_exists(&self.pool, username).await {
                Ok(exists) => {
                    if !exists {
                        match room.reject_invitation().await {
                            Ok(_) => {
                                warn!("User attempted to invite the bot to the room but is not authorized to do so.");
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to reject room invitation.");
                            }
                        };
                        return;
                    }
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "Error encountered while checking if inviting user exists."
                    );
                    return;
                }
            };

            // Synapse has a bug where the bot can receive an invite, but the server isn't ready
            // for the bot to join the room.
            // https://github.com/matrix-org/synapse/issues/4345
            let mut last_error: Option<matrix_sdk::Error> = None;
            let mut rng: SmallRng = SmallRng::from_entropy();
            let dist = Uniform::new_inclusive(0, 1000);
            for i in 0..5 {
                match room.accept_invitation().await {
                    Ok(_) => {
                        info!("Joined room after invitation.");
                        return;
                    }
                    Err(e) => {
                        let base: u64 = u64::pow(2, i) * 100;
                        let jitter: u64 = dist.sample(&mut rng);
                        let delay = base + jitter;
                        debug!(
                            error = ?e,
                            base = %base,
                            jitter = %jitter,
                            delay = %delay,
                            "Encountered error while attempting to join room, delaying before the next attempt."
                        );
                        sleep(Duration::from_millis(delay)).await;
                        last_error = Some(e);
                    }
                }
            }

            error!(last_error = ?last_error.unwrap(), "Failed to join room after five attempts.");
        }
    }

    #[tracing::instrument(skip(self))]
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
            "sourcecode" => sourcecode_command::get_message(),
            _ => {
                info!("Received unrecognized command.");
                MessageData::from("Unrecognized command.")
            }
        };
        Ok(result)
    }
}

async fn user_exists(pool: &DbPool, username: &str) -> Result<bool> {
    let conn = pool.get()?;
    let username2 = String::from(username);
    let u = spawn_blocking(move || User::try_get_by_username(&conn, username2.as_str())).await??;
    match u {
        Some(u) => {
            debug!(username = %username, id = %u.id.to_string(), "User exists.");
            Ok(true)
        }
        None => Ok(false),
    }
}
