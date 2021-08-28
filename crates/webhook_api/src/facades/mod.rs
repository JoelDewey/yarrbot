use anyhow::Result;
use actix_web::web::block;

mod sonarr_matrix_facade;

pub use sonarr_matrix_facade::handle_sonarr_webhook;
use yarrbot_db::DbPool;
use uuid::Uuid;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_matrix_client::YarrbotMatrixClient;
use yarrbot_matrix_client::message::MessageData;
use futures::future::join_all;

async fn send_matrix_messages(pool: &DbPool, webhook_id: &Uuid, client: &YarrbotMatrixClient, message: &MessageData) -> Result<()> {
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
