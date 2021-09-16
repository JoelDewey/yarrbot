//! Traits and utilities for sending messages to a Matrix server.

use anyhow::Error;
use matrix_sdk::ruma::events::room::message::MessageEventContent;
use std::fmt::Debug;

/// Formatted message data to send via Matrix.
#[derive(Debug)]
pub struct MessageData {
    /// The plain text version of the message to send to clients that don't support HTML messages.
    pub plain: String,
    /// The HTML (rich text) version of the message to send.
    pub html: String,
}

impl MessageData {
    pub fn new(plain: &str, html: &str) -> MessageData {
        MessageData {
            plain: String::from(plain),
            html: String::from(html),
        }
    }
}

impl From<MessageData> for MessageEventContent {
    fn from(message_data: MessageData) -> Self {
        MessageEventContent::notice_html(message_data.plain, message_data.html)
    }
}

impl From<&MessageData> for MessageEventContent {
    fn from(message_data: &MessageData) -> Self {
        MessageEventContent::notice_html(message_data.plain.as_str(), message_data.html.as_str())
    }
}

impl From<Error> for MessageData {
    fn from(e: Error) -> Self {
        MessageData::from(format!("Error encountered: {:?}", e).as_str())
    }
}

impl From<&str> for MessageData {
    fn from(m: &str) -> Self {
        Self::new(m, m)
    }
}
