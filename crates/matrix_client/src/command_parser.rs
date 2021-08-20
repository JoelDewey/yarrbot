//! Handles events from the Matrix server; mostly just for parsing commands.

use crate::commands::*;
use crate::message::MessageData;
use anyhow::Result;
use matrix_sdk::{
    async_trait,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    room::Room,
    Client, EventHandler,
};
use std::collections::VecDeque;
use yarrbot_db::DbPool;

const YARRBOT_COMMAND: &str = "!yarrbot";

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
                        is_direct_message: match room.members().await {
                            Ok(v) => v.len() == 2,
                            Err(_) => false,
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
}
