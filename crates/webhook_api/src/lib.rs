#[macro_use]
extern crate log;

use crate::models::radarr::RadarrWebhook;
use crate::models::sonarr::SonarrWebhook;
use crate::yarrbot_api_error::YarrbotApiError;
use actix_web::{web, Error, HttpResponse};
use extractors::webhook_extractor::WebhookInfo;
use futures_util::StreamExt;
use yarrbot_db::enums::ArrType;

mod extractors;
mod models;
mod yarrbot_api_error;

async fn handle_sonarr(body: &web::BytesMut) -> Result<HttpResponse, Error> {
    let parsed = serde_json::from_slice::<SonarrWebhook>(body);
    match parsed {
        // Temporary; won't return the webhook body once the Matrix parts are fleshed out.
        Ok(w) => Ok(HttpResponse::Ok().json(w)),
        Err(_) => Err(YarrbotApiError::bad_request("Unable to parse request body.").into()),
    }
}

async fn handle_radarr(body: &web::BytesMut) -> Result<HttpResponse, Error> {
    let parsed = serde_json::from_slice::<RadarrWebhook>(body);
    match parsed {
        // Temporary; won't return the webhook body once the Matrix parts are fleshed out.
        Ok(w) => Ok(HttpResponse::Ok().json(w)),
        Err(_) => Err(YarrbotApiError::bad_request("Unable to parse request body.").into()),
    }
}

const MAX_SIZE: usize = 262_144; // Limit max payload size to 256k.

async fn index(
    webhook_info: WebhookInfo,
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
    match webhook.arr_type {
        ArrType::Sonarr => handle_sonarr(&body).await,
        ArrType::Radarr => handle_radarr(&body).await,
    }
}

pub fn webhook_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/webhook").service(
            web::resource("/{webhook_id}")
                .route(web::post().to(index))
                .route(web::put().to(index)),
        ),
    );
}
