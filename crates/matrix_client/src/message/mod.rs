use std::sync::Arc;

mod message_data;
mod message_data_builder;

pub use message_data::MessageData;
pub use message_data_builder::{MatrixMessageDataPart, MessageDataBuilder, SectionHeadingLevel};

/// Represents all data needed to send a message to a given [MatrixRoom].
#[derive(Debug)]
pub struct Message {
    /// The struct containing the plain and rich text message data.
    pub message_data: Arc<MessageData>,

    /// The fully qualified Matrix ID for a Room.
    pub destination: String,
}

impl Message {
    pub fn new(destination: &str, data: Arc<MessageData>) -> Self {
        Message {
            destination: String::from(destination),
            message_data: data,
        }
    }
}
