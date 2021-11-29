use crate::shutdown::shutdown_notice::ShutdownNotice;
use actix::{Message, Recipient};

/// Sending this message ensures that the [Recipient] receives notice that Yarrbot is shutting down.
pub struct SubscribeToShutdown(pub Recipient<ShutdownNotice>);

impl Message for SubscribeToShutdown {
    type Result = ();
}
