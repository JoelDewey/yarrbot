use crate::client::configuration::YarrbotMatrixClientSettings;
use crate::client::YarrbotMatrixClient;
use crate::send_handler::SendMessageActor;
use crate::{RoomMessageActor, StrippedStateMemberActor};
use actix::{Actor, Addr};
use anyhow::{Context, Result};
use matrix_sdk::{Client, ClientConfig, SyncSettings};
use tracing::debug;
use yarrbot_db::DbPool;

/// Initialize the [Client] belonging to the Matrix SDK.
/// 
/// # Notes
/// 
/// May return an error if logging into or syncing with the homeserver for the first time fails.
pub async fn initialize_matrix_sdk_client() -> Result<Client> {
    let YarrbotMatrixClientSettings {
        url,
        username,
        password,
        storage_dir,
    } = YarrbotMatrixClientSettings::default();
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

    Ok(client)
}

/// Initialize the Matrix components of Yarrbot.
pub fn initialize_matrix_actors(
    client: Client,
    pool: DbPool,
) -> Result<(
    Addr<RoomMessageActor>,
    Addr<SendMessageActor>,
    Addr<StrippedStateMemberActor>,
)> {
    let send_addr = SendMessageActor::new(client.clone()).start();
    Ok((
        RoomMessageActor::new(client.clone(), pool.clone(), send_addr.clone()).start(),
        send_addr,
        StrippedStateMemberActor::new(client, pool).start(),
    ))
}

/// Initialize Yarrbot's wrapper around the Matrix SDK [Client].
pub async fn initialize_yarrbot_matrix_client(
    client: Client,
    pool: DbPool,
    send_addr: Addr<SendMessageActor>,
) -> Result<YarrbotMatrixClient> {
    YarrbotMatrixClient::new(client.clone(), pool.clone(), send_addr.clone())
        .await
        .context("Failed to create a YarrbotMatrixClient.")
}
