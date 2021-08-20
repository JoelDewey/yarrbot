//! Supporting functions for listing all webhooks.

use super::get_user;
use crate::command_parser::CommandMetadata;
use crate::message::MessageData;
use anyhow::Result;
use itertools::Itertools;
use std::collections::VecDeque;
use tokio::task::spawn_blocking;
use yarrbot_common::short_id::ShortId;
use yarrbot_db::actions::webhook_actions::WebhookActions;
use yarrbot_db::enums::UserRole;
use yarrbot_db::models::{User, Webhook};
use yarrbot_db::DbPool;

/// Handle the list command. Specifying `!yarrbot webhook list all` will list all users
/// webhooks if the requesting user is a System Administrator.
pub async fn handle_list(
    metadata: CommandMetadata,
    pool: &DbPool,
    mut data: VecDeque<&str>,
) -> MessageData {
    let user = match get_user(pool, metadata.user).await {
        Ok(Some(u)) => u,
        Ok(None) => {
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
    let specifier = String::from(data.pop_front().unwrap_or(""));
    let webhooks = match get_webhooks(pool, &user, specifier).await {
        Ok(v) => v,
        Err(e) => {
            error!("Encountered an error while retrieving webhooks: {:?}", e);
            return MessageData::from(
                "Couldn't retrieve the list of webhooks, please try again later.",
            );
        }
    };
    let id_list = webhooks.iter().map(|w| w.id.to_short_id()).join(" | ");

    MessageData::from(format!("Webhooks: {}", id_list).as_str())
}

/// Get all webhooks for a user or all webhooks in the system if the user is a System Administrator.
async fn get_webhooks(pool: &DbPool, user: &User, specifier: String) -> Result<Vec<Webhook>> {
    let conn = pool.get()?;
    let is_system_admin = matches!(user.user_role, UserRole::SystemAdministrator);
    let user_id = user.id;
    Ok(spawn_blocking(move || {
        if specifier == "all" && is_system_admin {
            Webhook::get_all(&conn)
        } else {
            Webhook::get_all_by_user_id(&conn, &user_id)
        }
    })
    .await??)
}
