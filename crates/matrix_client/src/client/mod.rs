mod on_room_message;
mod on_stripped_state_member;

use crate::message::Message;
use crate::MatrixClient;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use itertools::Itertools;
use matrix_sdk::{ruma::identifiers::RoomId, Client};
use std::convert::TryFrom;
use tokio::sync::mpsc::Sender;
use tokio::task::spawn_blocking;
use tracing::{error, error_span, info};
use tracing_futures::Instrument;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;

pub use on_room_message::on_room_message;
pub use on_stripped_state_member::on_stripped_state_member;

/// A Matrix client for Yarrbot.
#[derive(Clone)]
pub struct YarrbotMatrixClient {
    client: Client,
    pool: DbPool,
    message_channel: Sender<Message>,
}

impl YarrbotMatrixClient {
    async fn init(client: &Client, pool: &DbPool) -> Result<()> {
        let conn = pool.get()?;
        info!("Retrieving list of MatrixRooms from the database.");
        let matrix_rooms = spawn_blocking(move || MatrixRoom::get_many(&conn, None)).await??;
        let join_room_tasks = matrix_rooms
            .iter()
            .map(|r| r.room_id.as_str())
            .unique()
            .map(|room_id| join_room(client, room_id));
        let mut stream = join_room_tasks.collect::<FuturesUnordered<_>>();
        while let Some(item) = stream
            .next()
            .instrument(error_span!("Joining Saved Matrix Rooms"))
            .await
        {
            if item.is_err() {
                let err = item.unwrap_err();
                error!(error = ?err, "Unable to join room.");
            }
        }

        Ok(())
    }

    /// Create a new [YarrbotMatrixClient] and connect it to a Matrix homeserver.
    /// The client will attempt to join all [MatrixRoom]s that Yarrbot is configured for.
    pub(crate) async fn new(
        client: Client,
        pool: DbPool,
        message_channel: Sender<Message>,
    ) -> Result<Self> {
        YarrbotMatrixClient::init(&client, &pool).await?;
        Ok(YarrbotMatrixClient {
            client,
            pool,
            message_channel,
        })
    }
}

#[async_trait]
impl MatrixClient for YarrbotMatrixClient {
    async fn send_message(&self, message: Message) -> Result<()> {
        self.message_channel
            .send(message)
            .await
            .context("Failed to send message over mpsc channel.")
    }
}

async fn join_room(client: &Client, id: &str) -> Result<()> {
    let room_id = RoomId::try_from(id)?;
    match client.join_room_by_id(&room_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e).with_context(|| format!("Room ID: {}", id)),
    }
}
