mod on_room_message;
mod on_stripped_state_member;

use crate::message::MessageData;
use crate::MatrixClient;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use itertools::Itertools;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::MemberEventContent;
use matrix_sdk::ruma::events::room::message::MessageEventContent;
use matrix_sdk::ruma::events::{StrippedStateEvent, SyncMessageEvent};
use matrix_sdk::{
    ruma::events::AnyMessageEventContent, ruma::identifiers::RoomId, Client, ClientConfig,
    SyncSettings,
};
use on_room_message::on_room_message;
use on_stripped_state_member::on_stripped_state_member;
use std::convert::TryFrom;
use std::path::PathBuf;
use tokio::task::spawn_blocking;
use tracing::{debug, error, error_span, info};
use tracing_futures::Instrument;
use url::Url;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;

/// Settings to configure a [YarrbotMatrixClient].
pub struct YarrbotMatrixClientSettings {
    pub url: Url,
    pub username: String,
    pub password: String,
    pub storage_dir: PathBuf,
}

/// A Matrix client for Yarrbot.
#[derive(Clone)]
pub struct YarrbotMatrixClient {
    client: Client,
    pool: DbPool,
}

impl YarrbotMatrixClient {
    async fn init(client: &Client, pool: &DbPool) -> Result<()> {
        let conn = pool.get()?;
        info!("Retrieving list of MatrixRooms from the database.");
        let matrix_rooms = spawn_blocking(move || MatrixRoom::get_many(&conn, None)).await??;
        let join_room_tasks = matrix_rooms
            .iter()
            .map(|r| &r.room_id[..])
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
    pub async fn new(matrix_settings: YarrbotMatrixClientSettings, pool: DbPool) -> Result<Self> {
        let YarrbotMatrixClientSettings {
            url,
            username,
            password,
            storage_dir,
        } = matrix_settings;
        debug!("Logging into the homeserver.");
        let client_config = ClientConfig::new().store_path(storage_dir);
        let client: Client = Client::new_with_config(url, client_config)?;
        client
            .login(
                username.as_str(),
                password.as_str(),
                Some("yarrbot"),
                Some("yarrbot"),
            )
            .await?;

        debug!("Performing initial sync with homeserver.");
        client.sync_once(SyncSettings::default()).await?;
        YarrbotMatrixClient::init(&client, &pool).await?;
        debug!("Setting CommandParser event handler.");
        client
            .register_event_handler({
                let pool = pool.clone();
                move |ev: SyncMessageEvent<MessageEventContent>, room: Room, client: Client| {
                    let pool = pool.clone();
                    async move {
                        on_room_message(&client, &pool, &room, &ev).await;
                    }
                }
            })
            .await
            .register_event_handler({
                let pool = pool.clone();
                move |ev: StrippedStateEvent<MemberEventContent>, room: Room, client: Client| {
                    let pool = pool.clone();
                    async move {
                        on_stripped_state_member(&client, &pool, room, &ev).await;
                    }
                }
            })
            .await;

        debug!("Matrix Client is ready.");
        Ok(YarrbotMatrixClient { client, pool })
    }

    /// Start the sync loop with the homeserver.
    ///
    /// Note: This method will never return.
    pub async fn start_sync_loop(&self) -> Result<()> {
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
        self.client.sync(settings).await;

        Ok(())
    }
}

#[async_trait]
impl MatrixClient for YarrbotMatrixClient {
    async fn send_message(&self, message: &MessageData, room: &MatrixRoom) -> Result<()> {
        info!(room.matrix_id = %room.room_id, "Sending Matrix message to room.");
        debug!(message = ?message, "Sending Matrix message with the given contents.");
        let room_id = RoomId::try_from(&room.room_id[..])?;
        let content = AnyMessageEventContent::RoomMessage(message.into());

        self.client.room_send(&room_id, content, None).await?;

        Ok(())
    }
}

async fn join_room(client: &Client, id: &str) -> Result<()> {
    let room_id = RoomId::try_from(id)?;
    match client.join_room_by_id(&room_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e).with_context(|| format!("Room ID: {}", id)),
    }
}
