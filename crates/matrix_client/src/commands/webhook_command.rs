//! Entrypoint for `!yarrbot webhook ...` commands.

use crate::command_parser::CommandMetadata;
use crate::commands::webhook::{handle_add, handle_list, handle_remove};
use crate::message::MessageData;
use anyhow::{bail, ensure, Result};
use matrix_sdk::Client;
use std::collections::VecDeque;
use yarrbot_db::DbPool;

/// Handles choosing which webhook subcommand to execute.
pub async fn handle_webhook_command(
    metadata: CommandMetadata,
    client: &Client,
    pool: &DbPool,
    mut data: VecDeque<&str>,
) -> Result<MessageData> {
    ensure!(!data.is_empty(), "Not enough arguments");
    ensure!(
        metadata.is_direct_message,
        "Yarrbot will only respond to webhook commands in a private room."
    );
    match data.pop_front().unwrap().to_lowercase().as_str() {
        "add" => Ok(handle_add(metadata, client, pool, data).await),
        "remove" => Ok(handle_remove(metadata, pool, data).await),
        "list" => Ok(handle_list(metadata, pool, data).await),
        c => bail!(format!("Unknown webhook command \"{}\".", c)),
    }
}
