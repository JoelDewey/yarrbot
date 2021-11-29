use crate::send_handler::send_to_matrix::SendToMatrix;
use actix::prelude::*;
use matrix_sdk::ruma::events::room::message::MessageEventContent;
use matrix_sdk::{ruma::events::AnyMessageEventContent, ruma::identifiers::RoomId, Client};
use std::convert::TryFrom;
use tracing::{debug, error, info};

/// Listens for requests to send messages to a Matrix room.
pub struct SendMessageActor {
    client: Client,
}

impl Actor for SendMessageActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("Started SendMessageActor.");
        info!("Yarrbot is listening for incoming Matrix messages.");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Stopped SendMessageActor.");
        info!("Yarrbot is no longer listening for incoming Matrix messages.");
    }
}

impl Handler<SendToMatrix> for SendMessageActor {
    type Result = ();

    fn handle(&mut self, msg: SendToMatrix, ctx: &mut Self::Context) -> Self::Result {
        // Client's just a wrapper around an Arc<InnerClientThingy>.
        let fut = send(self.client.clone(), msg);
        let actor_future = fut.into_actor(self);

        ctx.spawn(actor_future);
    }
}

impl SendMessageActor {
    pub fn new(client: Client) -> Self {
        SendMessageActor { client }
    }
}

/// Sends a given message to a Matrix room.
async fn send(client: Client, msg: SendToMatrix) {
    let destination = msg.destination;
    let message_data = msg.message_data;
    info!(
        room.matrix_id = %destination,
        "Sending Matrix message to room."
    );
    debug!(message = ?message_data, "Sending Matrix message with the given contents.");
    if let Ok(room_id) = RoomId::try_from(destination.as_str()) {
        if let Some(room) = client.get_joined_room(&room_id) {
            let event_content: MessageEventContent = MessageEventContent::notice_html(
                message_data.plain.as_str(),
                message_data.html.as_str(),
            );
            let content = AnyMessageEventContent::RoomMessage(event_content);

            if let Err(e) = room.send(content, None).await {
                error!(error = ?e, room_id = %destination, "Failed to send Matrix message.")
            }
        } else {
            error!(
                room_id = %destination,
                "Failed to send Matrix message because Yarrbot isn't a member of the desired room."
            );
        }
    } else {
        error!(room_id = %destination, "Failed to parse Room ID.");
    }
}
