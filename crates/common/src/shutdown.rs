//! Shared shutdown management code.
//! Based off of: https://github.com/tokio-rs/mini-redis/blob/master/src/shutdown.rs

use tokio::sync::watch;

/// Simple interface for manging shutdown requests.
#[derive(Debug, Clone)]
pub struct ShutdownManager {
    shutdown: bool,
    notify: watch::Receiver<bool>,
}

impl ShutdownManager {
    pub fn new(notify: watch::Receiver<bool>) -> Self {
        ShutdownManager {
            shutdown: false,
            notify,
        }
    }

    /// Returns `true` if shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Watch a channel for the shutdown request, completing if shutdown has been requested.
    pub async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.changed().await;

        self.shutdown = true;
    }
}
