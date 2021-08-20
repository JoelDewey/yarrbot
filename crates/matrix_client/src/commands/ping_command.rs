use crate::message::MessageData;

const PONG: &str = "pong";

/// Returns "pong".
pub fn get_message() -> MessageData {
    MessageData::from(PONG)
}
