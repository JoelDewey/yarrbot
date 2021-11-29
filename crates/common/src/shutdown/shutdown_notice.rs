use actix::Message;

/// Sent when the application is shutting down.
pub struct ShutdownNotice;

impl Message for ShutdownNotice {
    type Result = ();
}

impl ShutdownNotice {
    pub fn new() -> Self {
        ShutdownNotice {}
    }
}

impl Default for ShutdownNotice {
    fn default() -> Self {
        ShutdownNotice::new()
    }
}
