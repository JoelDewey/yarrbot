mod client;
mod commands;
pub mod message;
pub mod send_handler;
mod sync_handler;

use crate::client::on_room_message;
use crate::client::on_stripped_state_member;
use crate::message::Message;
use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::MemberEventContent;
use matrix_sdk::ruma::events::room::message::MessageEventContent;
use matrix_sdk::ruma::events::{StrippedStateEvent, SyncMessageEvent};
use matrix_sdk::ruma::identifiers::UserId;
use matrix_sdk::{Client, ClientConfig, SyncSettings};
use std::convert::TryFrom;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tracing::debug;
use url::Url;
use yarrbot_common::ShutdownManager;
use yarrbot_db::DbPool;

pub use crate::client::YarrbotMatrixClient;
use crate::send_handler::YarrbotMessageSendHandler;
pub use crate::sync_handler::YarrbotMatrixSyncHandler;

/// Settings to configure a [YarrbotMatrixClient].
pub struct YarrbotMatrixClientSettings {
    pub url: Url,
    pub username: String,
    pub password: String,
    pub storage_dir: PathBuf,
}

/// Check if a given [user_id] is valid.
pub fn is_user_id(user_id: &str) -> bool {
    UserId::try_from(user_id).is_ok()
}

/// Represents the base functionality that Yarrbot wants out of a client that connects to a Matrix
/// homeserver.
#[async_trait]
pub trait MatrixClient {
    /// Send a message contained within a [MessageData] to a given [MatrixRoom].
    async fn send_message(&self, message: Message) -> Result<()>;
}

pub async fn initialize_matrix(
    matrix_settings: YarrbotMatrixClientSettings,
    pool: DbPool,
    shutdown_rx: Receiver<bool>,
) -> Result<(
    YarrbotMatrixClient,
    YarrbotMessageSendHandler,
    YarrbotMatrixSyncHandler,
)> {
    let (tx, rx) = mpsc::channel::<Message>(64);
    let client = initialize_matrix_sdk_client(matrix_settings, &pool, tx.clone()).await?;
    let yarrbot_client = YarrbotMatrixClient::new(client.clone(), pool.clone(), tx).await?;
    let sender = YarrbotMessageSendHandler::new(
        client.clone(),
        rx,
        ShutdownManager::new(shutdown_rx.clone()),
    );
    let sync =
        YarrbotMatrixSyncHandler::new(client.clone(), ShutdownManager::new(shutdown_rx.clone()));

    Ok((yarrbot_client, sender, sync))
}

async fn initialize_matrix_sdk_client(
    matrix_settings: YarrbotMatrixClientSettings,
    pool: &DbPool,
    send_handler_tx: Sender<Message>,
) -> Result<Client> {
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
    debug!("Setting CommandParser event sender.");
    client
        .register_event_handler({
            let pool = pool.clone();
            let tx = send_handler_tx.clone();
            move |ev: SyncMessageEvent<MessageEventContent>, room: Room, client: Client| {
                let pool = pool.clone();
                let tx = tx.clone();
                async move {
                    on_room_message(&client, &pool, &room, &ev, tx).await;
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

    Ok(client)
}
