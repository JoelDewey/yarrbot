//! Supporting functions for adding a new webhook.

use super::get_user;
use crate::commands::CommandMetadata;
use crate::message::{MessageData, MessageDataBuilder};
use anyhow::{anyhow, bail, Result};
use matrix_sdk::ruma::identifiers::RoomIdOrAliasId;
use matrix_sdk::{room::Room, ruma::identifiers::ServerName, Client};
use std::collections::VecDeque;
use std::convert::TryFrom;
use tokio::task::spawn_blocking;
use tracing::{error, info, warn};
use yarrbot_common::crypto::{generate_password, hash};
use yarrbot_common::short_id::ShortId;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::actions::webhook_actions::WebhookActions;
use yarrbot_db::models::{MatrixRoom, NewMatrixRoom, NewWebhook, User, Webhook};
use yarrbot_db::DbPool;

/// Add a new webhook.
#[tracing::instrument(skip(client, pool, data), fields(raw_room, webhook_user, has_password))]
pub async fn handle_add(
    metadata: CommandMetadata,
    client: &Client,
    pool: &DbPool,
    mut data: VecDeque<&str>,
) -> MessageData {
    info!("Received webhook add command.");
    let span = tracing::Span::current();
    let user = match get_user(pool, &metadata.user).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            warn!(
                username = %metadata.user.as_str(),
                "User attempted to add a webhook but is not authorized to do so."
            );
            return MessageData::from("You are not allowed to modify webhooks.");
        }
        Err(e) => {
            error!(
                error = ?e,
                "Encountered error while retrieving user from the database."
            );
            return MessageData::from(
                "Yarrbot encountered an error communicating with the database.",
            );
        }
    };
    if data.len() < 3 {
        warn!("Not enough parameters provided to add command.");
        return MessageData::from(
            "Adding a new webhook requires a room alias/ID, a username, and optionally a password.",
        );
    }

    // Join room.
    let raw_room = data.pop_front().unwrap();
    span.record("raw_room", &raw_room);
    let room_alias = match RoomIdOrAliasId::try_from(raw_room) {
        Ok(r) => r,
        Err(e) => {
            warn!(error = ?e, "Unable to parse the Room ID or Alias ID.");
            return MessageData::from(
                format!("Could not parse room or alias \"{}\".", raw_room).as_str(),
            );
        }
    };
    let room = match join_room(client, &room_alias).await {
        Ok(r) => r,
        Err(e) => {
            error!(error = ?e, "Encountered an error while joining the room.");
            return MessageData::from(
                "Encountered issue while attempting to join room. You may need to invite yarrbot to the room first."
            );
        }
    };

    // Create webhook.
    let room_id = room.room_id().as_str();
    let username = data.pop_front().unwrap();
    span.record("webhook_user", &username);
    let password = match data.pop_front() {
        Some(p) => {
            span.record("has_password", &true);
            String::from(p)
        }
        None => {
            span.record("has_password", &false);
            generate_password(None).unwrap()
        }
    };
    let webhook = match create_webhook(pool, username, &password, &user).await {
        Ok(w) => w,
        Err(e) => {
            error!(error = ?e, "Failed to create webhook in the database.");
            return MessageData::from("Failed to create new webhook.");
        }
    };
    if let Err(e) = create_matrixroom(pool, room_id, &webhook).await {
        let uuid = &webhook.id;
        error!(error = ?e, webhook_uuid = %uuid, "Failed to create Matrix Room for webhook.");
        return MessageData::from(
            "There was an issue completing the webhook; you may need to remove it and then recreate it."
        );
    }

    let webhook_id = webhook.id.to_short_id();
    info!(webhook_id = %webhook_id, "Webhook created.");
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
    username: &str,
    password: &str,
    user: &User,
) -> Result<Webhook> {
    let hashed = hash(String::from(password)).await?;
    let new_webhook = NewWebhook::new(username, hashed.to_vec(), user);
    let conn = pool.get()?;
    Ok(spawn_blocking(move || Webhook::create_webhook(&conn, new_webhook)).await??)
}

/// Create a Matrix Room record in the database.
async fn create_matrixroom(pool: &DbPool, room_id: &str, webhook: &Webhook) -> Result<MatrixRoom> {
    let conn = pool.get()?;
    let new_matrix_room = NewMatrixRoom::new(room_id, webhook);
    Ok(spawn_blocking(move || MatrixRoom::create_room(&conn, new_matrix_room)).await??)
}
