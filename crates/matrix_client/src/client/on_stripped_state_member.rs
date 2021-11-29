use actix::Message;
use matrix_sdk::{
    room::Room,
    ruma::events::{room::member::MemberEventContent, StrippedStateEvent},
};

/// Message to send to an actor handling Matrix [StrippedStateEvent]s.
pub struct OnStrippedStateMember {
    pub room: Room,
    pub event: StrippedStateEvent<MemberEventContent>,
}

impl Message for OnStrippedStateMember {
    type Result = ();
}

impl OnStrippedStateMember {
    pub fn new(room: Room, event: StrippedStateEvent<MemberEventContent>) -> Self {
        OnStrippedStateMember { room, event }
    }
}
