pub mod message;

#[macro_use]
extern crate log;

use crate::message::MatrixMessage;
use anyhow::{Context, Result};
use itertools::Itertools;
use matrix_sdk::{
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    identifiers::RoomId,
    Client, SyncSettings,
};
use std::convert::TryFrom;
use std::env;
use url::Url;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;

const MATRIX_USER_ENV: &str = "YARRBOT_MATRIX_USERNAME";
const MATRIX_PASS_ENV: &str = "YARRBOT_MATRIX_PASSWORD";
const MATRIX_HOMESERVER_URL: &str = "YARRBOT_MATRIX_HOMESERVER_URL";

/// Settings to configure a [YarrbotMatrixClient].
pub struct YarrbotMatrixClientSettings {
    pub url: Url,
    pub username: String,
    pub password: String,
}

/// A Matrix client for Yarrbot.
#[derive(Clone)]
pub struct YarrbotMatrixClient {
    client: Client,
    pool: DbPool,
}

fn get_homeserver_url() -> Result<Url> {
    let raw = env::var(MATRIX_HOMESERVER_URL)?;
    Url::parse(&raw).with_context(|| "Parsing of homeserver URL failed.")
}

fn get_username() -> Result<String> {
    env::var(MATRIX_USER_ENV)
        .with_context(|| "Could not retrieve the Matrix username from the environment.")
}

fn get_password() -> Result<String> {
    env::var(MATRIX_PASS_ENV)
        .with_context(|| "Could not retrieve the Matrix password from the environment.")
}

pub async fn initialize_matrix_client(pool: DbPool) -> Result<YarrbotMatrixClient> {
    let settings = YarrbotMatrixClientSettings {
        username: get_username()?,
        password: get_password()?,
        url: get_homeserver_url()?,
    };
    Ok(YarrbotMatrixClient::new(settings, pool).await?)
}

impl YarrbotMatrixClient {
    async fn init(client: &Client, pool: &DbPool) -> Result<()> {
        let conn = pool.get()?;
        let matrix_rooms = MatrixRoom::get_many(&conn, None)?;
        let db_rooms = matrix_rooms.iter().map(|r| &r.room_id[..]).unique();
        for db_room in db_rooms {
            let room_id = match RoomId::try_from(db_room) {
                Ok(r) => r,
                Err(_) => {
                    error!("Unable to parse Matrix room Id {}.", db_room,);
                    continue;
                }
            };
            let join_result = client.join_room_by_id(&room_id).await;
            if join_result.is_err() {
                error!(
                    "Unable to join room {}: {:?}",
                    db_room,
                    join_result.unwrap_err()
                );
            } else {
                debug!("Joined room: {}", db_room);
            }
        }

        Ok(())
    }

    /// Create a new [YarrbotMatrixClient] and connect it to a Matrix homeserver.
    /// The client will attempt to join all [MatrixRoom]s that Yarrbot is configured for.
    ///
    /// # Remarks
    ///
    /// If a [MatrixRoom]'s [MatrixRoom.room_id] cannot be parsed, the record is deleted.
    ///
    pub async fn new(matrix_settings: YarrbotMatrixClientSettings, pool: DbPool) -> Result<Self> {
        let YarrbotMatrixClientSettings {
            url,
            username,
            password,
        } = matrix_settings;
        let client: Client = Client::new(url)?;
        client
            .login(username.as_str(), password.as_str(), None, Some("yarrbot"))
            .await?;

        client.sync_once(SyncSettings::default()).await?;
        YarrbotMatrixClient::init(&client, &pool).await?;

        Ok(YarrbotMatrixClient { client, pool })
    }

    /// Send a given [MatrixMessage] to a given [MatrixRoom]. Can optionally sync with the server after sending
    /// the message.
    pub async fn send_message(
        &self,
        message: impl MatrixMessage,
        room: &MatrixRoom,
        do_sync: bool,
    ) -> Result<()> {
        let room_id = RoomId::try_from(&room.room_id[..])?;
        let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
            message.get_plain()?,
            message.get_html()?,
        ));

        self.client.room_send(&room_id, content, None).await?;
        if do_sync {
            self.client.sync_once(SyncSettings::default()).await?;
        }

        Ok(())
    }
}
