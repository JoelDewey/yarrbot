use crate::shutdown::shutdown_notice::ShutdownNotice;
use crate::shutdown::subscribe_to_shutdown::SubscribeToShutdown;
use actix::{
    Actor, ActorContext, ActorFutureExt, AsyncContext, Context, Handler, Recipient, WrapFuture,
};
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tracing::{debug, error, info};

/// Relays notices to shutdown the Yarrbot by listening for SIGTERM or SIGINT then sending messages to
/// any registered [subscribers].
pub struct ShutdownActor {
    subscribers: Vec<Recipient<ShutdownNotice>>,
}

impl Actor for ShutdownActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("ShutdownActor started.");
        let fut = ShutdownActor::wait_for_signal();
        let fut_actor = fut.into_actor(self);
        let update_self = fut_actor.map(|_result, actor, ctx| {
            for subscriber in &actor.subscribers {
                subscriber
                    .do_send(ShutdownNotice::new())
                    .expect("Failed to send shutdown notice to a subscriber.");
            }

            ctx.stop();
        });
        let pinned = Box::pin(update_self);
        ctx.spawn(pinned);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("ShutdownActor stopped.");
        info!("Shutdown notices have been sent.");
    }
}

impl Handler<SubscribeToShutdown> for ShutdownActor {
    type Result = ();

    fn handle(&mut self, msg: SubscribeToShutdown, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribers.push(msg.0);
    }
}

impl ShutdownActor {
    /// Creates a new [ShutdownActor] with no subscribers.
    pub fn new() -> Self {
        ShutdownActor {
            subscribers: vec![],
        }
    }

    /// Waits for a SIGINT or a SIGTERM before completing.
    async fn wait_for_signal() {
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Could not set up SIGTERM stream.");
        tokio::select! {
            res = ctrl_c() => {
                debug!("Received SIGINT.");
                if let Err(e) = res {
                    error!(error = ?e, "Encountered error while listening for SIGINT.");
                }
            },
            _ = sigterm.recv() => {
                debug!("Received SIGTERM.");
            }
        }

        info!("Shutdown signal received.");
    }
}

impl Default for ShutdownActor {
    /// Calls [ShutdownActor::new()].
    fn default() -> Self {
        ShutdownActor::new()
    }
}
