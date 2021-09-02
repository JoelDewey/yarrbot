mod command_parser;
mod commands;
pub mod message;

#[macro_use]
extern crate log;

use crate::message::MessageData;
use anyhow::{bail, Context, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use itertools::Itertools;
use matrix_sdk::{
    events::AnyMessageEventContent,
    identifiers::{RoomId, UserId},
    Client, ClientConfig, SyncSettings,
};
use std::convert::TryFrom;
use std::env;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use tokio::task::spawn_blocking;
use url::Url;
use yarrbot_common::environment::{
    get_env_var,
    variables::{BOT_STORAGE_DIR, MATRIX_HOMESERVER_URL, MATRIX_PASS, MATRIX_USER},
};
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;

/// Check if a given [user_id] is valid.
pub fn is_user_id(user_id: &str) -> bool {
    UserId::try_from(user_id).is_ok()
}

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

fn get_homeserver_url() -> Result<Url> {
    let raw = get_env_var(MATRIX_HOMESERVER_URL)?;
    info!("Received homeserver URL: {}", &raw);
    Url::parse(&raw).with_context(|| "Parsing of homeserver URL failed.")
}

fn get_username() -> Result<String> {
    get_env_var(MATRIX_USER)
        .with_context(|| "Could not retrieve the Matrix username from the environment.")
}

fn get_password() -> Result<String> {
    get_env_var(MATRIX_PASS)
        .with_context(|| "Could not retrieve the Matrix password from the environment.")
}

fn get_storage_dir() -> Result<PathBuf> {
    let mut path = match get_env_var(BOT_STORAGE_DIR) {
        Ok(s) => PathBuf::from(s),
        Err(_) => {
            info!("No storage directory specified, using the current directory instead.");
            env::current_dir()?
        }
    };
    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => {
            fs::create_dir_all(&path)?;
            fs::metadata(&path)?
        }
    };
    if metadata.is_file() {
        bail!("Storage directory path is a file: {}", path.display());
    }
    if metadata.permissions().readonly() {
        bail!("Storage directory path is readonly: {}", path.display());
    }
    path.push("matrix");
    match fs::metadata(&path) {
        Ok(_) => (),
        Err(_) => {
            fs::create_dir(&path)?;
        }
    }
    Ok(path)
}

pub async fn initialize_matrix_client(pool: DbPool) -> Result<YarrbotMatrixClient> {
    let settings = YarrbotMatrixClientSettings {
        username: get_username()?,
        password: get_password()?,
        url: get_homeserver_url()?,
        storage_dir: get_storage_dir()?,
    };
    Ok(YarrbotMatrixClient::new(settings, pool).await?)
}

async fn join_room(client: &Client, id: &str) -> Result<()> {
    let room_id = RoomId::try_from(id)?;
    match client.join_room_by_id(&room_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e).with_context(|| format!("Room ID: {}", id)),
    }
}

impl YarrbotMatrixClient {
    async fn init(client: &Client, pool: &DbPool) -> Result<()> {
        let conn = pool.get()?;
        info!("Retrieving list of MatrixRooms from the database.");
        let matrix_rooms = spawn_blocking(move || MatrixRoom::get_many(&conn, None)).await??;
        let db_rooms = matrix_rooms
            .iter()
            .map(|r| &r.room_id[..])
            .unique()
            .map(|room_id| join_room(client, room_id));
        let mut futures_unordered = FuturesUnordered::from_iter(db_rooms);
        while let Some(item) = futures_unordered.next().await {
            match item {
                Ok(_) => (),
                Err(e) => error!("Unable to join room: {:?}", e),
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
            .set_event_handler(Box::new(command_parser::CommandParser::new(
                client.clone(),
                pool.clone(),
            )))
            .await;

        debug!("Matrix Client is ready.");
        Ok(YarrbotMatrixClient { client, pool })
    }

    /// Send a given [MatrixMessage] to a given [MatrixRoom]. Can optionally sync with the server after sending
    /// the message.
    pub async fn send_message(&self, message: MessageData, room: &MatrixRoom) -> Result<()> {
        let room_id = RoomId::try_from(&room.room_id[..])?;
        let content = AnyMessageEventContent::RoomMessage(message.into());

        self.client.room_send(&room_id, content, None).await?;

        Ok(())
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
