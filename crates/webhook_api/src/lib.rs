//! Configuration and handling of webhook pushes from Sonarr/Radarr.

#[macro_use]
extern crate log;

use crate::facades::{handle_radarr_webhook, handle_sonarr_webhook, send_matrix_messages};
use crate::models::radarr::RadarrWebhook;
use crate::models::sonarr::SonarrWebhook;
use crate::yarrbot_api_error::YarrbotApiError;
use actix_web::{web, Error, HttpResponse};
use anyhow::{Result, Context};
use extractors::webhook_extractor::WebhookInfo;
use futures_util::StreamExt;
use log::Level::Debug;
use std::str;
use yarrbot_db::enums::ArrType;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::MatrixClient;
use serde::Deserialize;

mod extractors;
mod facades;
mod models;
mod yarrbot_api_error;

const MAX_SIZE: usize = 262_144; // Limit max payload size to 256k.

fn parse_body<'de, T>(body: &'de web::BytesMut) -> Result<T> where T: Deserialize<'de> {
    serde_json::from_slice::<T>(body).with_context(|| {
        if log_enabled!(Debug) {
            let str_body = str::from_utf8(&body).unwrap_or("Could not convert body to string.");
            debug!("Request body: {}", str_body)
        }

        "Encountered an error while parsing webhook request body."
    })
}

async fn index<T: MatrixClient>(
    webhook_info: WebhookInfo,
    pool: web::Data<DbPool>,
    matrix_client: web::Data<T>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    // Essentially copied from: https://actix.rs/docs/request/
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(YarrbotApiError::bad_request(
                format!("Body exceeded limit of {} kilobytes.", MAX_SIZE).as_str(),
            )
            .into());
        }

        body.extend_from_slice(&chunk);
    }

    let webhook = webhook_info.webhook;
    let message_result = match webhook.arr_type {
        ArrType::Sonarr => {
            debug!("Starting processing of Sonarr webhook.");
            match parse_body::<SonarrWebhook>(&body) {
                Ok(w) => handle_sonarr_webhook(&w).await,
                Err(e) => {
                    debug!("Encountered error while parsing webhook: {:?}", e);
                    return Ok(HttpResponse::BadRequest().finish());
                }
            }
        }
        ArrType::Radarr => {
            match parse_body::<RadarrWebhook>(&body) {
                Ok(w) => handle_radarr_webhook(&w).await,
                Err(e) => {
                    debug!("Encountered error while parsing webhook: {:?}", e);
                    return Ok(HttpResponse::BadRequest().finish());
                }
            }
        }
    };
    let message = match message_result {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to transform the webhook into a message to send to Matrix: {:?}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    match send_matrix_messages(pool.get_ref(), &webhook.id, matrix_client.get_ref(), message).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            error!("Encountered error while sending webhook Matrix messages: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

/// Configure the webhook API endpoints.
pub fn webhook_config<T: MatrixClient + Send + Sync + 'static + Clone>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/webhook").service(
            web::resource("/{webhook_id}")
                .route(web::post().to(index::<T>))
                .route(web::put().to(index::<T>)),
        ),
    );
}
