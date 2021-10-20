//! Services for reading webhook data from Sonarr/Radarr and sending it out
//! via Matrix.

mod radarr_facade;
mod sonarr_facade;

use crate::models::common::ArrHealthCheckResult;
use actix_web::web::block;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
pub use radarr_facade::handle_radarr_webhook;
pub use sonarr_facade::handle_sonarr_webhook;
use std::option::Option::Some;
use std::sync::Arc;
use tracing::{error, info, info_span, warn};
use uuid::Uuid;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::message::{
    Message, MessageData, MessageDataBuilder, SectionHeadingLevel,
};
use yarrbot_matrix_client::MatrixClient;

pub use radarr_facade::RADARR_NAME;
pub use sonarr_facade::SONARR_NAME;
use tracing_futures::Instrument;

pub async fn send_matrix_messages<T: MatrixClient>(
    pool: &DbPool,
    webhook_id: &Uuid,
    client: &T,
    message_data: MessageData,
) {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!(error = ?e, "Failed to retrieve database connection from the pool.");
            return;
        }
    };
    let id = *webhook_id;
    let rooms = match block(move || MatrixRoom::get_by_webhook_id(&conn, &id)).await {
        Ok(Ok(r)) => r,
        Err(e) => {
            error!(
                error = ?e,
                "Failed to retrieve rooms due to a blocking error."
            );
            Vec::new()
        }
        Ok(Err(e)) => {
            error!(
                error = ?e,
                "Failed to retrieve rooms due to a database error."
            );
            Vec::new()
        }
    };
    info!("Sending a webhook message to {} room(s).", rooms.len());
    let arc = Arc::new(message_data);
    let tasks = rooms
        .iter()
        .map(|r| Message::new(r.room_id.as_str(), arc.clone()))
        .map(|m| client.send_message(m));
    let mut stream = tasks.collect::<FuturesUnordered<_>>();
    while let Some(item) = stream
        .next()
        .instrument(info_span!("Sending Matrix Message"))
        .await
    {
        if item.is_err() {
            error!(
                error = ?item.unwrap_err(),
                "Encountered error while posting to matrix room."
            );
        }
    }

    info!("Finished sending webhook messages.");
}

fn add_heading(
    builder: &mut MessageDataBuilder,
    key: &str,
    value: &str,
    server_name: &Option<String>,
) {
    if let Some(sn) = server_name {
        builder.add_heading(
            &SectionHeadingLevel::One,
            &format!("{} - {}: {}", sn, key, value),
        );
    } else {
        builder.add_heading(&SectionHeadingLevel::One, &format!("{}: {}", key, value));
    }
}

fn add_quality(builder: &mut MessageDataBuilder, quality: &Option<String>) {
    builder.add_key_value(
        "Quality",
        quality
            .as_ref()
            .unwrap_or(&String::from("Not Specified"))
            .as_str(),
    );
}

/// Respond to health checks from Sonarr/Radarr.
fn on_health_check(
    arr_type: &str,
    level: Option<ArrHealthCheckResult>,
    message: Option<String>,
    health_type: Option<String>,
    wiki_url: Option<String>,
    server_name: &Option<String>,
) -> MessageData {
    info!("Received Health Check webhook from {}.", arr_type);

    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, arr_type, "Health Check", server_name);
    if level.is_some() {
        let l = match level.as_ref().unwrap() {
            ArrHealthCheckResult::Ok => "Ok",
            ArrHealthCheckResult::Notice => "Notice",
            ArrHealthCheckResult::Warning => "Warning",
            ArrHealthCheckResult::Error => "Error",
            ArrHealthCheckResult::Unknown => {
                warn!("Did not recognize the health check level; \"Unknown\" will be used.");
                "Unknown"
            }
        };
        builder.add_key_value("Level", l);
    } else {
        builder.add_key_value("Level", "Unknown");
    }

    builder.add_key_value(
        "Message",
        message
            .as_ref()
            .unwrap_or(&String::from("No Message Given")),
    );
    builder.add_key_value(
        "Type",
        health_type
            .as_ref()
            .unwrap_or(&String::from("No Message Given")),
    );
    builder.add_key_value(
        "Wiki URL",
        wiki_url
            .as_ref()
            .unwrap_or(&String::from("No Message Given")),
    );

    builder.to_message_data()
}
