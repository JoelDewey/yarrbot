use anyhow::Result;
use matrix_sdk::{
    room::Room,
    ruma::events::{room::member::MemberEventContent, StrippedStateEvent},
    Client,
};
use rand::distributions::Uniform;
use rand::prelude::*;
use tokio::task::spawn_blocking;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use yarrbot_db::DbPool;
use yarrbot_db::{actions::user_actions::UserActions, models::User};

#[tracing::instrument(skip(client, pool, room, room_member), fields(room.room_id = %room.room_id(), room.name, username = %room_member.sender))]
pub async fn on_stripped_state_member(
    client: &Client,
    pool: &DbPool,
    room: Room,
    room_member: &StrippedStateEvent<MemberEventContent>,
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
        match user_exists(pool, username).await {
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
