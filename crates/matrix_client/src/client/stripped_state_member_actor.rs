use crate::client::on_stripped_state_member::OnStrippedStateMember;
use actix::{Actor, Addr, AsyncContext, Context, Handler, WrapFuture};
use anyhow::Result;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::MemberEventContent;
use matrix_sdk::ruma::events::StrippedStateEvent;
use matrix_sdk::Client;
use rand::distributions::Uniform;
use rand::prelude::*;
use rand::SeedableRng;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use yarrbot_db::actions::user_actions::UserActions;
use yarrbot_db::models::User;
use yarrbot_db::DbPool;

/// Responds to stripped state events (e.g. Yarrbot is invited to a room).
pub struct StrippedStateMemberActor {
    client: Client,
    pool: DbPool,
}

impl StrippedStateMemberActor {
    pub fn new(client: Client, pool: DbPool) -> Self {
        StrippedStateMemberActor { client, pool }
    }

    /// Configures the Matrix SDK [Client] to listen for stripped state events.
    async fn register(client: Client, addr: Addr<StrippedStateMemberActor>) {
        client
            .register_event_handler({
                let addr = addr.clone();
                move |ev: StrippedStateEvent<MemberEventContent>, room: Room| {
                    let addr = addr.clone();
                    async move {
                        if let Err(e) = addr.send(OnStrippedStateMember::new(room, ev)).await {
                            error!(error = ?e, "Encountered error while sending stripped state event to actor.")
                        }
                    }
                }
            })
            .await;
    }
}

impl Actor for StrippedStateMemberActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Started StrippedStateMemberActor.");
        let client_fut = Self::register(self.client.clone(), ctx.address());
        let actor = client_fut.into_actor(self);
        ctx.wait(actor);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Stopped StrippedStateMemberActor.");
    }
}

impl Handler<OnStrippedStateMember> for StrippedStateMemberActor {
    type Result = ();

    fn handle(&mut self, msg: OnStrippedStateMember, ctx: &mut Self::Context) -> Self::Result {
        let fut =
            on_stripped_state_member(self.client.clone(), self.pool.clone(), msg.room, msg.event);
        let actor = fut.into_actor(self);

        ctx.spawn(actor);
    }
}

/// Respond to stripped state events; mostly just responding to invites.
#[tracing::instrument(skip(client, pool, room, room_member), fields(room.room_id = %room.room_id(), room.name, username = %room_member.sender))]
async fn on_stripped_state_member(
    client: Client,
    pool: DbPool,
    room: Room,
    room_member: StrippedStateEvent<MemberEventContent>,
) {
    let room_name = room.name().unwrap_or_else(|| String::from("(No Name)"));
    tracing::Span::current().record("room.name", &room_name.as_str());
    // Based off of https://github.com/matrix-org/matrix-rust-sdk/blob/0.3.0/matrix_sdk/examples/autojoin.rs

    // Don't respond to invites if they're not meant for us.
    if room_member.state_key != client.user_id().await.unwrap() {
        debug!("Received invite that's not meant for the bot.");
        return;
    }

    if let Room::Invited(room) = room {
        // Don't let users that aren't admins invite the bot to rooms.
        let username = room_member.sender.as_str();
        match user_exists(&pool, username).await {
            Ok(exists) => {
                if !exists {
                    match room.reject_invitation().await {
                        Ok(_) => {
                            warn!("User attempted to invite the bot to the room but is not authorized to do so.");
                        }
                        Err(e) => {
                            error!(error = ?e, "Failed to reject room invitation.");
                        }
                    };
                    return;
                }
            }
            Err(e) => {
                error!(
                    error = ?e,
                    "Error encountered while checking if inviting user exists."
                );
                return;
            }
        };

        // Synapse has a bug where the bot can receive an invite, but the server isn't ready
        // for the bot to join the room.
        // https://github.com/matrix-org/synapse/issues/4345
        let mut last_error: Option<matrix_sdk::Error> = None;
        let mut rng: SmallRng = SmallRng::from_entropy();
        let dist = Uniform::new_inclusive(0, 1000);
        for i in 0..5 {
            match room.accept_invitation().await {
                Ok(_) => {
                    info!("Joined room after invitation.");
                    return;
                }
                Err(e) => {
                    let base: u64 = u64::pow(2, i) * 100;
                    let jitter: u64 = dist.sample(&mut rng);
                    let delay = base + jitter;
                    debug!(
                        error = ?e,
                        base = %base,
                        jitter = %jitter,
                        delay = %delay,
                        "Encountered error while attempting to join room, delaying before the next attempt."
                    );
                    sleep(Duration::from_millis(delay)).await;
                    last_error = Some(e);
                }
            }
        }

        error!(last_error = ?last_error.unwrap(), "Failed to join room after five attempts.");
    }
}

async fn user_exists(pool: &DbPool, username: &str) -> Result<bool> {
    let conn = pool.get()?;
    let username2 = String::from(username);
    let u = spawn_blocking(move || User::try_get_by_username(&conn, username2.as_str())).await??;
    match u {
        Some(u) => {
            debug!(username = %username, id = %u.id.to_string(), "User exists.");
            Ok(true)
        }
        None => Ok(false),
    }
}
