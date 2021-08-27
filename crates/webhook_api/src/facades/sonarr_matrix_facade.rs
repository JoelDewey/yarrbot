use crate::models::sonarr::SonarrWebhook;
use actix_web::web::block;
use actix_web::HttpResponse;
use anyhow::Result;
use futures::future::join_all;
use yarrbot_db::actions::matrix_room_actions::MatrixRoomActions;
use yarrbot_db::models::{MatrixRoom, Webhook};
use yarrbot_db::DbPool;
use yarrbot_matrix_client::message::MessageData;
use yarrbot_matrix_client::YarrbotMatrixClient;

pub async fn handle_sonarr_webhook(
    webhook: &Webhook,
    data: &SonarrWebhook,
    pool: &DbPool,
    matrix_client: &YarrbotMatrixClient,
) -> Result<HttpResponse> {
    let message: Option<MessageData> = match data {
        SonarrWebhook::Test { .. } => Some(MessageData::from("Received test push from Sonarr.")),
        _ => None, // TODO: Remove this once all of the variants have been accounted for.
    };

    if message.is_some() {
        let conn = pool.get()?;
        let webhook_id = webhook.id;
        let rooms = block(move || MatrixRoom::get_by_webhook_id(&conn, &webhook_id)).await??;
        let m = message.unwrap();
        let tasks = rooms
            .iter()
            .map(|r| matrix_client.send_message(m.clone(), r));
        for t in join_all(tasks).await.iter().filter(|r| r.is_err()) {
            error!(
                "Encountered error while posting to matrix room: {:?}",
                t.as_ref().unwrap_err()
            );
        }
    }

    Ok(HttpResponse::Ok().finish())
}
