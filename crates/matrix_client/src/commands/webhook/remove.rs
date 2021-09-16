//! Supporting functions for removing webhooks.

use super::get_user;
use crate::command_parser::CommandMetadata;
use crate::message::MessageData;
use anyhow::Result;
use std::collections::VecDeque;
use tokio::task::spawn_blocking;
use tracing::{error, info, warn};
use uuid::Uuid;
use yarrbot_common::short_id::ShortId;
use yarrbot_db::actions::webhook_actions::WebhookActions;
use yarrbot_db::enums::UserRole;
use yarrbot_db::models::Webhook;
use yarrbot_db::DbPool;

/// Remove a webhook from the database.
#[tracing::instrument(skip(pool, data), fields(webhook_id))]
pub async fn handle_remove(
    metadata: CommandMetadata,
    pool: &DbPool,
    mut data: VecDeque<&str>,
) -> MessageData {
    info!("Received webhook remove command.");
    let user = match get_user(pool, &metadata.user).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            warn!("User attempted to remove a webhook but is not authorized to do so.");
            return MessageData::from("You are not allowed to modify webhooks.");
        }
        Err(e) => {
            error!(
                error = ?e,
                "Encountered an error while retrieving user information from the database."
            );
            return MessageData::from("Encountered an error while retrieving user information.");
        }
    };

    let webhook_id = match data.pop_front() {
        Some(w) => {
            tracing::Span::current().record("webhook_id", &w);
            w
        }
        None => {
            info!("Webhook does not exist.");
            return MessageData::from("No webhook specified.");
        }
    };
    let webhook = match get_webhook(pool, webhook_id).await {
        Ok(Some(w)) => w,
        Ok(None) => return MessageData::from("That webhook doesn't exist."),
        Err(e) => {
            error!(
                error = ?e,
                "Encountered an error while retrieving Webhook data from the database."
            );
            return MessageData::from("Error encountered while looking up the webhook.");
        }
    };
    if webhook.user_id != user.id && !matches!(user.user_role, UserRole::SystemAdministrator) {
        return MessageData::from("You are not allowed to modify this webhook.");
    }

    match delete_webhook(pool, webhook).await {
        Ok(_) => {
            info!("Deleted webhook.");
            MessageData::from("Webhook removed.")
        }
        Err(e) => {
            error!("Encountered error while deleting webhook: {:?}", e);
            MessageData::from("Failed to delete webhook. Please try again.")
        }
    }
}

/// Get a webhook record from the database.
async fn get_webhook(pool: &DbPool, webhook_id: &str) -> Result<Option<Webhook>> {
    let webhook_uuid = match Uuid::from_short_id(webhook_id) {
        Ok(u) => u,
        Err(e) => {
            warn!(
                error = ?e,
                "Encountered error decoding UUID from short ID."
            );
            return Ok(None);
        }
    };
    let conn = pool.get()?;
    Ok(spawn_blocking(move || Webhook::try_get(&conn, &webhook_uuid)).await??)
}

/// Delete the webhook from the database.
async fn delete_webhook(pool: &DbPool, webhook: Webhook) -> Result<()> {
    let conn2 = pool.get()?;
    Ok(spawn_blocking(move || webhook.delete(&conn2)).await??)
}
