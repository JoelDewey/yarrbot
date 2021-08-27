use crate::models::sonarr::SonarrWebhook;
use actix_web::HttpResponse;
use anyhow::Result;
use yarrbot_db::models::Webhook;
use yarrbot_matrix_client::YarrbotMatrixClient;

pub async fn handle_sonarr_webhook(
    webhook: &Webhook,
    data: &SonarrWebhook,
    matrix_client: &YarrbotMatrixClient,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}
