//! Supporting functions for adding a new webhook.

use super::get_user;
use crate::command_parser::CommandMetadata;
use crate::message::{MessageData, MessageDataBuilder};
use anyhow::{anyhow, bail, Result};
use matrix_sdk::identifiers::RoomIdOrAliasId;
use matrix_sdk::{identifiers::ServerName, room::Room, Client};
use std::collections::VecDeque;
use std::convert::TryFrom;
use tokio::task::spawn_blocking;
use yarrbot_common::crypto::{generate_password, hash};
use yarrbot_common::short_id::ShortId;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::actions::webhook_actions::WebhookActions;
use yarrbot_db::enums::ArrType;
use yarrbot_db::models::{MatrixRoom, NewMatrixRoom, NewWebhook, User, Webhook};
use yarrbot_db::DbPool;

/// Add a new webhook.
pub async fn handle_add(
    metadata: CommandMetadata,
    client: &Client,
    pool: &DbPool,
    mut data: VecDeque<&str>,
) -> MessageData {
    let user = match get_user(pool, &metadata.user).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            warn!(
                "{} attempted to add a webhook but is not authorized to do so.",
                &metadata.user
            );
            return MessageData::from("You are not allowed to modify webhooks.");
        }
        Err(e) => {
            error!(
                "Encountered error while retrieving user from the database: {:?}",
                e
            );
            return MessageData::from(
                "Yarrbot encountered an error communicating with the database.",
            );
        }
    };
    if data.len() < 3 {
        return MessageData::from(
            "Adding a new webhook requires collection manager type, room alias, a username, and optionally a password."
        );
    }

    // Get collection manager type.
    let arr_type = match data.pop_front().unwrap().to_lowercase().as_str() {
        "sonarr" => ArrType::Sonarr,
        "radarr" => ArrType::Radarr,
        t => {
            return MessageData::from(
                format!("Unknown collection manager type \"{}\".", t).as_str(),
            );
        }
    };

    // Join room.
    let raw_room = data.pop_front().unwrap();
    let room_alias = match RoomIdOrAliasId::try_from(raw_room) {
        Ok(r) => r,
        Err(e) => {
            debug!("Unable to parse the Room ID or Alias ID: {:?}", e);
            return MessageData::from(
                format!("Could not parse room or alias \"{}\".", raw_room).as_str(),
            );
        }
    };
    let room = match join_room(client, &room_alias).await {
        Ok(r) => r,
        Err(e) => {
            error!("Encountered an error while joining the room: {:?}", e);
            return MessageData::from(
                "Encountered issue while attempting to join room. You may need to invite yarrbot to the room first."
            );
        }
    };

    // Create webhook.
    let room_id = room.room_id().as_str();
    let username = data.pop_front().unwrap();
    let password = match data.pop_front() {
        Some(p) => String::from(p),
        None => generate_password(None).unwrap(),
    };
    let webhook = match create_webhook(pool, arr_type, username, &password, &user).await {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create webhook in the database: {:?}", e);
            return MessageData::from("Failed to create new webhook.");
        }
    };
    match create_matrixroom(pool, room_id, &webhook).await {
        Ok(m) => m,
        Err(e) => {
            error!(
                "Failed to create matrix room record in the database: {:?}",
                e
            );
            return MessageData::from(
                "There was an issue completing the webhook; you may need to remove it and then recreate it."
            );
        }
    };

    let webhook_id = webhook.id.to_short_id();
    info!("Webhook created: {} ({})", &webhook_id, &webhook.id);
    let mut builder = MessageDataBuilder::new();
    builder.add_line(&format!("Set up a new webhook for {}.", raw_room));
    builder.add_key_value_with_code("ID", &webhook_id);
    builder.add_key_value_with_code("Username", username);
    builder.add_key_value_with_code("Password", &password);
    builder.to_message_data()
}

/// Join a Matrix room by [RoomIdOrAliasId] through the bot's homeserver.
async fn join_room(client: &Client, room_alias_id: &RoomIdOrAliasId) -> Result<Room> {
    let user = match client.user_id().await {
        Some(u) => u,
        None => bail!("Couldn't retrieve the current user for its server name; was the user's session destroyed?")
    };
    let server_name = user.server_name();
    let server_name_array: [Box<ServerName>; 1] = [server_name.into()];
    let room_id = client
        .join_room_by_id_or_alias(room_alias_id, &server_name_array)
        .await?
        .room_id;
    client.get_room(&room_id).ok_or(anyhow!(format!(
        "Couldn't retrieve the room for room alias \"{}\".",
        room_alias_id.as_str()
    )))
}

/// Create a webhook in the database.
async fn create_webhook(
    pool: &DbPool,
    arr_type: ArrType,
    username: &str,
    password: &str,
    user: &User,
) -> Result<Webhook> {
    let hashed = hash(password)?;
    let new_webhook = NewWebhook::new(arr_type, username, hashed.to_vec(), user);
    let conn = pool.get()?;
    Ok(spawn_blocking(move || Webhook::create_webhook(&conn, new_webhook)).await??)
}

/// Create a Matrix Room record in the database.
async fn create_matrixroom(pool: &DbPool, room_id: &str, webhook: &Webhook) -> Result<MatrixRoom> {
    let conn = pool.get()?;
    let new_matrix_room = NewMatrixRoom::new(room_id, webhook);
    Ok(spawn_blocking(move || MatrixRoom::create_room(&conn, new_matrix_room)).await??)
}
