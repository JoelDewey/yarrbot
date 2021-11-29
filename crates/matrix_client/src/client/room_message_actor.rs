use crate::client::on_room_message::OnRoomMessage;
use crate::commands::*;
use crate::message::MessageData;
use crate::{SendMessageActor, SendToMatrix};
use actix::prelude::*;
use anyhow::Result;
use matrix_sdk::{
    room::Room,
    ruma::events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        SyncMessageEvent,
    },
    Client,
};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{debug, error, info};
use yarrbot_db::DbPool;

const YARRBOT_COMMAND: &str = "!yarrbot";

/// Actor that responds to Matrix messages sent in rooms that Yarrbot is a member of.
pub struct RoomMessageActor {
    client: Client,
    pool: DbPool,
    send_addr: Addr<SendMessageActor>,
}

impl RoomMessageActor {
    pub fn new(client: Client, pool: DbPool, send_addr: Addr<SendMessageActor>) -> Self {
        RoomMessageActor {
            client,
            pool,
            send_addr,
        }
    }

    /// Register the actor with the Matrix SDK [Client] so that stripped state events are passed to the actor.
    async fn register(client: Client, addr: Addr<RoomMessageActor>) {
        client
            .register_event_handler({
                let addr = addr.clone();
                move |ev: SyncMessageEvent<MessageEventContent>, room: Room| {
                    let addr = addr.clone();
                    async move {
                        if let Err(e) = addr.send(OnRoomMessage::new(room, ev)).await {
                            error!(error = ?e, "Failed to send message to RoomMessageActor.");
                        }
                    }
                }
            })
            .await;
    }
}

impl Actor for RoomMessageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Started RoomMessageActor.");
        let client_fut = Self::register(self.client.clone(), ctx.address());
        let actor = client_fut.into_actor(self);
        ctx.wait(actor);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Stopped RoomMessageActor.");
    }
}

impl Handler<OnRoomMessage> for RoomMessageActor {
    type Result = ();

    fn handle(&mut self, msg: OnRoomMessage, ctx: &mut Self::Context) -> Self::Result {
        let fut = on_room_message(
            self.client.clone(),
            self.pool.clone(),
            msg.room,
            msg.event,
            self.send_addr.clone(),
        );
        let actor = fut.into_actor(self);

        ctx.spawn(actor);
    }
}

/// Handle a message sent to a [Room] that the bot is a member of.
#[tracing::instrument(skip(client, pool, room, event, send_addr), fields(event.sender = %event.sender, room.room_id = %room.room_id(), room.name))]
async fn on_room_message(
    client: Client,
    pool: DbPool,
    room: Room,
    event: SyncMessageEvent<MessageEventContent>,
    send_addr: Addr<SendMessageActor>,
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
                execute_command(&client, &pool, key.as_str(), metadata, data)
                    .await
                    .unwrap_or_else(|e| e.into())
            } else {
                debug!(key = &key.as_str(), "Command unrecognized.");
                MessageData::from("Unrecognized command.")
            };

            info!("Sending response to command.");
            let send_result = send_addr.try_send(SendToMatrix::new(
                room.room_id().as_str(),
                Arc::new(message_data),
            ));
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
        "help" => help_command::get_message(),
        _ => {
            info!("Received unrecognized command.");
            MessageData::from("Unrecognized command.")
        }
    };
    Ok(result)
}
