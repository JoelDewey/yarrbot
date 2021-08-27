use crate::models::sonarr::SonarrWebhook;
use actix_web::HttpResponse;
use anyhow::Result;
use yarrbot_matrix_client::YarrbotMatrixClient;

pub async fn handle_sonarr_webhook(
    webhook: SonarrWebhook,
    matrix_client: &YarrbotMatrixClient,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}
