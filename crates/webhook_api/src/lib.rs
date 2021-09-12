//! Configuration and handling of webhook pushes from Sonarr/Radarr.

use crate::facades::{send_matrix_messages, handle_webhook};
use actix_web::{web, Error, HttpResponse};
use anyhow::{Context, Result, bail};
use extractors::webhook_extractor::WebhookInfo;
use futures_util::StreamExt;
use tracing::{error, info_span, error_span};
use std::str;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::MatrixClient;
use tracing_actix_web::RootSpan;
use std::convert::AsRef;
use crate::models::ArrWebhook;
use tracing_futures::Instrument;

mod extractors;
mod facades;
mod models;
mod yarrbot_api_error;
mod yarrbot_root_span;

const MAX_SIZE: usize = 262_144; // Limit max payload size to 256k.

pub use yarrbot_root_span::YarrbotRootSpan;

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
        error!(request_body = str_body, ERR_MESSAGE);
        ERR_MESSAGE
    })
}

async fn index<T: MatrixClient>(
    root_span: RootSpan,
    webhook_info: WebhookInfo,
    pool: web::Data<DbPool>,
    matrix_client: web::Data<T>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    root_span.record("webhook_arr_type", &webhook_info.webhook.arr_type.as_ref());
    root_span.record("webhook_short_id", &webhook_info.short_id.as_str());

    let deserialization_result = async move {
        // Essentially copied from: https://actix.rs/docs/request/
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > MAX_SIZE {
                const ERR_MESSAGE: &str = "Request body exceeded max allowed size.";
                error!(max_request_body_size = MAX_SIZE, ERR_MESSAGE);
                bail!(ERR_MESSAGE);
            }
            body.extend_from_slice(&chunk);
        }

        deserialize_body(body)
    }.instrument(error_span!("Deserializing Request Body")).await;

    if let Ok(body) = deserialization_result {
        let webhook = &webhook_info.webhook;
        let message = handle_webhook(body, webhook)
            .instrument(info_span!("Converting Webhook to Matrix Message"))
            .await;
        match message {
            Ok(m) => {
                let send_result = send_matrix_messages(
                    pool.get_ref(),
                    &webhook.id,
                    matrix_client.get_ref(), m)
                    .instrument(info_span!("Sending Matrix Messages"))
                    .await;
                if let Err(e) = send_result {
                    error!("Encountered error while sending webhook Matrix messages: {:?}", e);
                    return Ok(HttpResponse::InternalServerError().finish());
                }
            },
            Err(e) => {
                error!("Encountered error during webhook to Matrix message conversion: {:?}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        }
    } else {
        return Ok(HttpResponse::BadRequest().finish());
    }

    return Ok(HttpResponse::Ok().finish());
}
