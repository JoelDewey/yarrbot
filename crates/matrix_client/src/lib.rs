mod client;
mod command_parser;
mod commands;
pub mod message;

#[macro_use]
extern crate log;

use crate::message::MessageData;
use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::identifiers::UserId;
use std::convert::TryFrom;
use yarrbot_db::models::MatrixRoom;

pub use crate::client::{YarrbotMatrixClient, YarrbotMatrixClientSettings};

/// Check if a given [user_id] is valid.
pub fn is_user_id(user_id: &str) -> bool {
    UserId::try_from(user_id).is_ok()
}

/// Represents the base functionality that Yarrbot wants out of a client that connects to a Matrix
/// homeserver.
#[async_trait]
pub trait MatrixClient {
    /// Send a message contained within a [MessageData] to a given [MatrixRoom].
    async fn send_message(&self, message: &MessageData, room: &MatrixRoom) -> Result<()>;
}
