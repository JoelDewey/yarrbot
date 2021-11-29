pub mod client;
mod commands;
pub mod message;
pub mod send_handler;
mod sync_handler;

use crate::client::RoomMessageActor;
use crate::client::StrippedStateMemberActor;
use crate::message::Message;
use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::ruma::identifiers::UserId;
use std::convert::TryFrom;

use crate::send_handler::{SendMessageActor, SendToMatrix};
pub use crate::sync_handler::MatrixSyncActor;

/// Check if a given [user_id] is valid.
pub fn is_user_id(user_id: &str) -> bool {
    UserId::try_from(user_id).is_ok()
}

/// Represents the base functionality that Yarrbot wants out of a client that connects to a Matrix
/// homeserver.
#[async_trait]
pub trait MatrixClient {
    /// Send a message contained within a [MessageData] to a given [MatrixRoom].
    async fn send_message(&self, message: Message) -> Result<()>;
}
