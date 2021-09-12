//! Services for reading webhook data from Sonarr/Radarr and sending it out
//! via Matrix.

use actix_web::web::block;
use anyhow::Result;

mod radarr_facade;
mod sonarr_facade;

use crate::models::common::ArrHealthCheckResult;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
pub use radarr_facade::handle_radarr_webhook;
pub use sonarr_facade::handle_sonarr_webhook;
use std::option::Option::Some;
use uuid::Uuid;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::enums::ArrType;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::message::{MessageData, MessageDataBuilder, SectionHeadingLevel};
use yarrbot_matrix_client::MatrixClient;

pub async fn send_matrix_messages<T: MatrixClient>(
    pool: &DbPool,
    webhook_id: &Uuid,
    client: &T,
    message: MessageData,
) -> Result<()> {
    let conn = pool.get()?;
    let id = *webhook_id;
    let rooms = block(move || MatrixRoom::get_by_webhook_id(&conn, &id)).await??;
    let tasks = rooms.iter().map(|r| client.send_message(&message, r));
    let mut stream = tasks.collect::<FuturesUnordered<_>>();
    while let Some(item) = stream.next().await {
        if item.is_err() {
            error!(
                "Encountered error while posting to matrix room: {:?}",
                item.unwrap_err()
            );
        }
    }

    Ok(())
}

fn add_heading(builder: &mut MessageDataBuilder, key: &str, value: &str) {
    builder.add_heading(&SectionHeadingLevel::One, &format!("{}: {}", key, value));
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
    arr_type: &ArrType,
    level: &Option<ArrHealthCheckResult>,
    message: &Option<String>,
    health_type: &Option<String>,
    wiki_url: &Option<String>,
) -> MessageData {
    let arr = match arr_type {
        ArrType::Sonarr => "Sonarr",
        ArrType::Radarr => "Radarr",
    };
    info!("Received Health Check webhook from {}.", arr);

    let mut builder = MessageDataBuilder::new();
    builder.add_heading(&SectionHeadingLevel::One, &format!("{} Health Check", arr));
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
