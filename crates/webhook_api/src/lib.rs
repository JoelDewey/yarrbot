//! Configuration and handling of webhook pushes from Sonarr/Radarr.

use crate::facades::{
    handle_radarr_webhook, handle_sonarr_webhook, send_matrix_messages, RADARR_NAME, SONARR_NAME,
};
use crate::models::ArrWebhook;
use actix_web::{web, HttpResponse};
use anyhow::{bail, Context, Result};
use extractors::webhook_extractor::WebhookInfo;
use futures_util::StreamExt;
use std::str;
use tracing::{error, error_span, info_span};
use tracing_actix_web::RootSpan;
use tracing_futures::Instrument;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::message::MessageData;
use yarrbot_matrix_client::MatrixClient;
pub use yarrbot_root_span::YarrbotRootSpan;

mod extractors;
mod facades;
mod models;
mod yarrbot_api_error;
mod yarrbot_root_span;

const MAX_SIZE: usize = 262_144; // Limit max payload size to 256k.

/// Configure the webhook API endpoints.
pub fn webhook_config<T: MatrixClient + Send + Sync + 'static + Clone>(
    cfg: &mut web::ServiceConfig,
) {
    cfg.service(
        web::scope("/webhook").service(
            web::resource("/{webhook_id}")
                .route(web::post().to(index::<T>))
                .route(web::put().to(index::<T>)),
        ),
    );
}

fn deserialize_body(body: web::BytesMut) -> Result<ArrWebhook> {
    serde_json::from_slice::<ArrWebhook>(&body).with_context(|| {
        const ERR_MESSAGE: &str = "Encountered an error while parsing webhook request body.";
        let str_body = str::from_utf8(&body).unwrap_or("Could not convert body to string.");
        error!(request_body = str_body, "{}", ERR_MESSAGE);
        ERR_MESSAGE
    })
}

async fn handle_webhook(body: ArrWebhook, root_span: &RootSpan) -> Result<MessageData> {
    match body {
        ArrWebhook::Sonarr(w) => {
            root_span.record("webhook_arr_type", &SONARR_NAME);
            handle_sonarr_webhook(w).await
        }
        ArrWebhook::Radarr(w) => {
            root_span.record("webhook_arr_type", &RADARR_NAME);
            handle_radarr_webhook(w).await
        }
    }
}

async fn index<T: MatrixClient>(
    root_span: RootSpan,
    webhook_info: WebhookInfo,
    pool: web::Data<DbPool>,
    matrix_client: web::Data<T>,
    mut payload: web::Payload,
) -> HttpResponse {
    root_span.record("webhook_short_id", &webhook_info.short_id.as_str());

    let deserialization_result = async move {
        // Essentially copied from: https://actix.rs/docs/request/
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > MAX_SIZE {
                bail!("Request body exceeded max allowed size.");
            }
            body.extend_from_slice(&chunk);
        }

        deserialize_body(body)
    }
    .instrument(error_span!("Deserializing Request Body"))
    .await;

    if let Ok(body) = deserialization_result {
        let webhook = &webhook_info.webhook;
        let message = handle_webhook(body, &root_span)
            .instrument(info_span!("Converting Webhook to Matrix Message"))
            .await;
        match message {
            Ok(m) => {
                let send_result =
                    send_matrix_messages(pool.get_ref(), &webhook.id, matrix_client.get_ref(), m)
                        .instrument(info_span!("Sending Matrix Messages"))
                        .await;
                if let Err(e) = send_result {
                    error!(
                        "{:?}",
                        e.context("Encountered error while sending webhook Matrix messages.")
                    );
                    return HttpResponse::InternalServerError().finish();
                }
            }
            Err(e) => {
                error!(
                    "{:?}",
                    e.context("Encountered error during webhook to Matrix message conversion.")
                );
                return HttpResponse::InternalServerError().finish();
            }
        }
    } else {
        return HttpResponse::BadRequest().finish();
    }

    HttpResponse::Ok().finish()
}
