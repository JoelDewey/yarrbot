use actix::{Actor, ActorContext, AsyncContext, Context, Handler, WrapFuture};
use matrix_sdk::{Client, SyncSettings};
use tracing::{debug, info};
use yarrbot_common::ShutdownNotice;

/// Manages the sync loop performed against the homeserver, which allows Yarrbot to respond to new messages and events.
pub struct MatrixSyncActor {
    client: Client,
}

impl Actor for MatrixSyncActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Started MatrixSyncActor.");
        info!("Beginning Matrix sync loop.");
        let fut = MatrixSyncActor::start_sync_loop(self.client.clone());
        let fut_actor = fut.into_actor(self);
        ctx.spawn(fut_actor);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Stopped MatrixSyncActor.");
        info!("Stopped Matrix sync loop.")
    }
}

impl Handler<ShutdownNotice> for MatrixSyncActor {
    type Result = ();

    fn handle(&mut self, _msg: ShutdownNotice, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl MatrixSyncActor {
    pub fn new(client: Client) -> Self {
        MatrixSyncActor { client }
    }

    /// Performs an initial sync with the homeserver, then starts the sync loop.
    ///
    /// # Note
    ///
    /// As this calls [Client::sync], the future will never complete. The actor will cancel the future when it is
    /// shutdown.
    async fn start_sync_loop(client: Client) {
        let token = match client.sync_token().await {
            Some(t) => t,
            None => {
                client
                    .sync_once(SyncSettings::default())
                    .await
                    .expect("Unable to sync with the Matrix homeserver.");
                client
                    .sync_token()
                    .await
                    .expect("Yarrbot just synced with the homeserver, but no token was found.")
            }
        };
        let settings = SyncSettings::default().token(token);
        debug!("Beginning sync loop.");
        client.sync(settings).await;
    }
}
