mod configuration;
mod initialization;
mod on_room_message;
mod on_stripped_state_member;
mod room_message_actor;
mod stripped_state_member_actor;

use crate::message::Message;
use crate::{MatrixClient, SendMessageActor, SendToMatrix};
use actix::Addr;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use itertools::Itertools;
use matrix_sdk::{ruma::identifiers::RoomId, Client};
use std::convert::TryFrom;
use tokio::task::spawn_blocking;
use tracing::{error, error_span, info};
use tracing_futures::Instrument;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;

pub use initialization::{
    initialize_matrix_actors, initialize_matrix_sdk_client, initialize_yarrbot_matrix_client,
};
pub use on_room_message::OnRoomMessage;
pub use room_message_actor::RoomMessageActor;
pub use stripped_state_member_actor::StrippedStateMemberActor;

/// A Matrix client for Yarrbot.
#[derive(Clone)]
pub struct YarrbotMatrixClient {
    client: Client,
    pool: DbPool,
    message_addr: Addr<SendMessageActor>,
}

impl YarrbotMatrixClient {
    async fn init(&self) -> Result<()> {
        let conn = self.pool.get()?;
        info!("Retrieving list of MatrixRooms from the database.");
        let matrix_rooms = spawn_blocking(move || MatrixRoom::get_many(&conn, None)).await??;
        let join_room_tasks = matrix_rooms
            .iter()
            .map(|r| r.room_id.as_str())
            .unique()
            .map(|room_id| join_room(&self.client, room_id));
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
        message_addr: Addr<SendMessageActor>,
    ) -> Result<Self> {
        let yarrbot_matrix_client = YarrbotMatrixClient {
            client,
            pool,
            message_addr,
        };
        yarrbot_matrix_client.init().await?;
        Ok(yarrbot_matrix_client)
    }
}

#[async_trait]
impl MatrixClient for YarrbotMatrixClient {
    async fn send_message(&self, message: Message) -> Result<()> {
        self.message_addr
            .try_send(SendToMatrix::new(
                message.destination.as_str(),
                message.message_data,
            ))
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
