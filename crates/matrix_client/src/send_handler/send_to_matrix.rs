use crate::message::MessageData;
use actix::prelude::*;
use std::sync::Arc;

/// Wrapper for Matrix message data.
pub struct SendToMatrix {
    /// The struct containing the plain and rich text message data.
    pub message_data: Arc<MessageData>,

    /// The fully qualified Matrix ID for a Room.
    pub destination: String,
}

impl Message for SendToMatrix {
    type Result = ();
}

impl SendToMatrix {
    pub fn new(destination: &str, data: Arc<MessageData>) -> Self {
        SendToMatrix {
            destination: String::from(destination),
            message_data: data,
        }
    }
}
