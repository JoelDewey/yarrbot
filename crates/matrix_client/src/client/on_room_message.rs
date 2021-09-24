use crate::commands::*;
use crate::message::MessageData;
use anyhow::Result;
use matrix_sdk::{
    room::Room,
    ruma::events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    Client,
};
use std::collections::VecDeque;
use tracing::{debug, error, info};
use yarrbot_db::DbPool;

const YARRBOT_COMMAND: &str = "!yarrbot";

#[tracing::instrument(skip(client, pool, room, event), fields(event.sender = %event.sender, room.room_id = %room.room_id(), room.name))]
pub async fn on_room_message(
    client: &Client,
    pool: &DbPool,
    room: &Room,
    event: &SyncMessageEvent<MessageEventContent>,
) {
    let room_name = room.name().unwrap_or_else(|| String::from("(No Name)"));
    tracing::Span::current().record("room.name", &room_name.as_str());
    // Don't respond to messages posted by our bot.
    if event.sender == client.user_id().await.unwrap() {
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
                execute_command(client, pool, key.as_str(), metadata, data)
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

async fn execute_command(
    client: &Client,
    pool: &DbPool,
    command: &str,
    metadata: CommandMetadata,
    data: VecDeque<&str>,
) -> Result<MessageData> {
    let result = match command {
        "ping" => ping_command::get_message(),
        "webhook" => webhook_command::handle_webhook_command(metadata, client, pool, data).await?,
        "sourcecode" => sourcecode_command::get_message(),
        _ => {
            info!("Received unrecognized command.");
            MessageData::from("Unrecognized command.")
        }
    };
    Ok(result)
}
