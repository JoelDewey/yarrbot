use crate::message::MessageData;
use tracing::info;

const PONG: &str = "pong";

/// Returns "pong".
pub fn get_message() -> MessageData {
    info!("Received ping command.");
    MessageData::from(PONG)
}
