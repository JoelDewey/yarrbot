#[macro_use]
extern crate log;

use actix_web::{web, Error, HttpResponse};
use extractors::webhook_extractor::WebhookInfo;

mod extractors;
mod yarrbot_api_error;

async fn index(_webhook_info: WebhookInfo, _body: web::Payload) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
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
