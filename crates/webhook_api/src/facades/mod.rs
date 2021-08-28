use actix_web::web::block;
use anyhow::Result;

mod sonarr_matrix_facade;

use futures::future::join_all;
pub use sonarr_matrix_facade::handle_sonarr_webhook;
use uuid::Uuid;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::message::{MessageData, MessageDataBuilder, SectionHeadingLevel};
use yarrbot_matrix_client::YarrbotMatrixClient;

async fn send_matrix_messages(
    pool: &DbPool,
    webhook_id: &Uuid,
    client: &YarrbotMatrixClient,
    message: &MessageData,
) -> Result<()> {
    let conn = pool.get()?;
    let id = *webhook_id;
    let rooms = block(move || MatrixRoom::get_by_webhook_id(&conn, &id)).await??;
    let tasks = rooms
        .iter()
        .map(|r| client.send_message(message.clone(), r));
    for t in join_all(tasks).await.iter().filter(|r| r.is_err()) {
        error!(
            "Encountered error while posting to matrix room: {:?}",
            t.as_ref().unwrap_err()
        );
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
