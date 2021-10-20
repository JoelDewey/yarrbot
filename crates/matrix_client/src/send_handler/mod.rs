use crate::message::Message;
use anyhow::Result;
use matrix_sdk::ruma::events::room::message::MessageEventContent;
use matrix_sdk::{ruma::events::AnyMessageEventContent, ruma::identifiers::RoomId, Client};
use std::convert::TryFrom;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info};
use yarrbot_common::ShutdownManager;

/// Listens for requests to send messages to a Matrix room.
pub struct YarrbotMessageSendHandler {
    client: Client,
    message_channel: Receiver<Message>,
    shutdown_manager: ShutdownManager,
}

impl YarrbotMessageSendHandler {
    pub fn new(
        client: Client,
        channel: Receiver<Message>,
        shutdown_manager: ShutdownManager,
    ) -> Self {
        YarrbotMessageSendHandler {
            client,
            message_channel: channel,
            shutdown_manager,
        }
    }

    /// Listens to a mpsc channel for Matrix [Message] structs. The [MessageData] within is sent to the given
    /// destination.
    ///
    /// # Remarks
    ///
    /// This method returns after the corresponding [YarrbotMatrixClient] has been dropped or a shutdown is requested.
    pub async fn handle_messages(&mut self, _shutdown_complete: Sender<()>) -> Result<()> {
        while !self.shutdown_manager.is_shutdown() {
            tokio::select! {
                message = self.message_channel.recv() => {
                    if let Some(mess) = message {
                        self.send(mess).await.unwrap_or_else(|e| error!(error = ?e, "Failed to send Matrix message after receiving it from mpsc channel."));
                    }
                },
                _ = self.shutdown_manager.recv() => {/* Do nothing, just fall through and exit. */}
            }
        }

        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        info!(room.matrix_id = %message.destination, "Sending Matrix message to room.");
        debug!(message = ?message.message_data, "Sending Matrix message with the given contents.");
        let room_id = RoomId::try_from(message.destination.as_str())?;
        let message_data = message.message_data;
        let event_content: MessageEventContent = MessageEventContent::notice_html(
            message_data.plain.as_str(),
            message_data.html.as_str(),
        );
        let content = AnyMessageEventContent::RoomMessage(event_content);

        self.client.room_send(&room_id, content, None).await?;

        Ok(())
    }
}
