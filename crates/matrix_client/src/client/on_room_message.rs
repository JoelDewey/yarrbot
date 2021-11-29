use actix::Message;
use matrix_sdk::{
    room::Room,
    ruma::events::{room::message::MessageEventContent, SyncMessageEvent},
};

/// Actor messages sent on delivery of Matrix messages visible to Yarrbot.
pub struct OnRoomMessage {
    pub room: Room,
    pub event: SyncMessageEvent<MessageEventContent>,
}

impl OnRoomMessage {
    /// Create a new [OnRoomMessage] from a given Matrix [Room] and [SyncMessageEvent<MessageEventContent>]
    pub fn new(room: Room, event: SyncMessageEvent<MessageEventContent>) -> Self {
        OnRoomMessage { room, event }
    }
}

impl Message for OnRoomMessage {
    type Result = ();
}
