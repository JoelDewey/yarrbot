//! Supporting functions for listing all webhooks.

use super::get_user;
use crate::command_parser::CommandMetadata;
use crate::message::{MatrixMessageDataPart, MessageData, MessageDataBuilder};
use anyhow::Result;
use std::collections::VecDeque;
use tokio::task::spawn_blocking;
use yarrbot_common::short_id::ShortId;
use yarrbot_db::actions::webhook_actions::WebhookActions;
use yarrbot_db::enums::UserRole;
use yarrbot_db::models::{User, Webhook};
use yarrbot_db::DbPool;
use tracing::{debug, warn, error};

/// Handle the list command. Specifying `!yarrbot webhook list all` will list all users
/// webhooks if the requesting user is a System Administrator.
pub async fn handle_list(
    metadata: CommandMetadata,
    pool: &DbPool,
    mut data: VecDeque<&str>,
) -> MessageData {
    debug!("Listing webhooks.");
    let user = match get_user(pool, &metadata.user).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            warn!(
                "{} attempted to list webhooks but is not authorized to do so.",
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

    debug!("Listing webhooks.");
    let items: Vec<String> = webhooks.iter().map(|w| w.id.to_short_id()).collect();
    let mut builder = MessageDataBuilder::new();
    if items.is_empty() {
        builder.add_line("No webhooks to list.");
    } else {
        builder.add_line("Webhooks:");
        builder.add_matrix_message_part(WebhookList { items });
    }

    builder.to_message_data()
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

struct WebhookList {
    items: Vec<String>,
}

impl MatrixMessageDataPart for WebhookList {
    fn to_plain(&self, break_character: &str) -> String {
        let items_len = self.items.len();
        let mut plain_parts = String::from(' ');
        for (i, item) in self.items.iter().enumerate() {
            plain_parts.push_str(item);
            if i < (items_len - 1) {
                plain_parts.push_str(", ");
            }
        }

        plain_parts.push(' ');
        plain_parts.push_str(break_character);
        plain_parts
    }

    fn to_html(&self, break_character: &str) -> String {
        let mut html_parts = String::from("<ul>");
        for item in &self.items {
            html_parts.push_str("<li><code>");
            html_parts.push_str(item);
            html_parts.push_str("</code></li>");
        }

        html_parts.push_str("</ul>");

        html_parts.push(' ');
        html_parts.push_str(break_character);
        html_parts
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::webhook::list::WebhookList;
    use crate::message::MessageDataBuilder;

    #[test]
    pub fn add_unordered_list_returns_list() {
        // Arrange
        let expected_plain = "1, 2, 3 \n **1**: 2 \n";
        let expected_html = "<ul><li><code>1</code></li><li><code>2</code></li><li><code>3</code></li></ul> <br><strong>1</strong>: 2 <br>";
        let items: Vec<String> = (1..4).map(|i| i.to_string()).collect();
        let mut builder = MessageDataBuilder::new();
        builder.add_matrix_message_part(WebhookList { items });
        builder.add_key_value("1", "2");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }
}
