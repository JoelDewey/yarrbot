use anyhow::Result;
use matrix_sdk::{Client, SyncSettings};
use tokio::sync::mpsc::Sender;
use tracing::debug;
use yarrbot_common::ShutdownManager;

/// Polls the homeserver for events for Yarrbot to respond to.
pub struct YarrbotMatrixSyncHandler {
    client: Client,
    shutdown_manager: ShutdownManager,
}

impl YarrbotMatrixSyncHandler {
    pub fn new(client: Client, shutdown_manager: ShutdownManager) -> Self {
        YarrbotMatrixSyncHandler {
            client,
            shutdown_manager,
        }
    }

    /// Start the sync loop with the homeserver.
    ///
    /// Note: Will only return if a shutdown is requested.
    pub async fn start_sync_loop(&mut self, _shutdown_complete: Sender<()>) -> Result<()> {
        let token = match self.client.sync_token().await {
            Some(t) => t,
            None => {
                self.client.sync_once(SyncSettings::default()).await?;
                self.client
                    .sync_token()
                    .await
                    .expect("Yarrbot just synced with the homeserver, but no token was found.")
            }
        };
        let settings = SyncSettings::default().token(token);
        debug!("Beginning sync loop.");
        tokio::select! {
            _ = self.client.sync(settings) => {},
            // Forcibly interrupting the sync loop is okay, we can just fetch any missed state the next time we start
            // up.
            _ = self.shutdown_manager.recv() => {}
        }

        Ok(())
    }
}
