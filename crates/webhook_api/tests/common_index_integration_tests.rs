//! TODO: These tests need a YarrbotMatrixClient but I need to figure out how to get Synapse to run locally.

use actix_web::http::header::ContentType;
use actix_web::http::Method;
use actix_web::http::StatusCode;
use actix_web::{test, web, App};
use yarrbot_webhook_api::webhook_config;

mod common;

// This short ID leads to a Radarr record that might be useful later.
// const DEFAULT_RADARR_WEBHOOK_SHORTID: &str = "Rkr7-T7zRRqJkqR8uV5yow";
// DEFAULT_WEBHOOK_SHORTID leads to a Sonarr record.
const DEFAULT_WEBHOOK_SHORTID: &str = "CJH9-jYSQa6t8cInfbkOog";
const TEST_BODY: &str = "{
    \"eventType\": \"Test\",
    \"series\": {
        \"id\": 1,
        \"title\": \"Test Title\",
        \"path\": \"C:\\\\testpath\",
        \"tvdbId\": 1234,
        \"type\": \"Standard\"
    },
    \"episodes\": [
        {
            \"id\": 123,
            \"episodeNumber\": 1,
            \"seasonNumber\": 1,
            \"title\": \"Test title\",
            \"qualityVersion\": 0
        }
    ]
}";

#[actix_rt::test]
async fn index_post_returns_200_given_valid_info() {
    // Arrange
    common::setup();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(common::POOL.clone()))
            .service(web::scope("/api/v1").configure(webhook_config)),
    )
    .await;
    let req = test::TestRequest::default()
        .insert_header((
            "authorization",
            format!("Basic {}", common::DEFAULT_B64).as_str(),
        ))
        .insert_header(ContentType::json())
        .method(Method::POST)
        .uri(format!("/api/v1/webhook/{}", DEFAULT_WEBHOOK_SHORTID).as_str())
        .set_payload(TEST_BODY)
        .to_request();

    // Act
    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn index_returns_401_unauthorized_given_invalid_credentials() {
    // Arrange
    common::setup();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(common::POOL.clone()))
            .service(web::scope("/api/v1").configure(webhook_config)),
    )
    .await;
    // NotARealUser:badP@55
    let input_b64 = "Tm90QVJlYWxVc2VyOmJhZFBANTU=";
    let req = test::TestRequest::default()
        .insert_header(("authorization", format!("Basic {}", input_b64).as_str()))
        .insert_header(ContentType::json())
        .method(Method::POST)
        .uri(format!("/api/v1/webhook/{}", DEFAULT_WEBHOOK_SHORTID).as_str())
        .set_payload(TEST_BODY)
        .to_request();

    // Act
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(StatusCode::UNAUTHORIZED, resp.status());
}

#[actix_rt::test]
async fn index_returns_404_not_found_given_short_id_not_uuid() {
    // Arrange
    common::setup();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(common::POOL.clone()))
            .service(web::scope("/api/v1").configure(webhook_config)),
    )
    .await;
    let req = test::TestRequest::default()
        .insert_header((
            "authorization",
            format!("Basic {}", common::DEFAULT_B64).as_str(),
        ))
        .insert_header(ContentType::json())
        .method(Method::POST)
        .uri(format!("/api/v1/webhook/{}", "incorrect").as_str())
        .set_payload(TEST_BODY)
        .to_request();

    // Act
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
}

#[actix_rt::test]
async fn index_returns_404_not_found_given_valid_short_id_but_not_in_db() {
    // Arrange
    common::setup();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(common::POOL.clone()))
            .service(web::scope("/api/v1").configure(webhook_config)),
    )
    .await;
    let req = test::TestRequest::default()
        .insert_header((
            "authorization",
            format!("Basic {}", common::DEFAULT_B64).as_str(),
        ))
        .insert_header(ContentType::json())
        .method(Method::POST)
        // Can be transformed into a valid UUID, but the UUID doesn't match anything.
        .uri(format!("/api/v1/webhook/{}", "UCLrVIqqQWObDdMY4Y1x8g").as_str())
        .set_payload(TEST_BODY)
        .to_request();

    // Act
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
}

#[actix_rt::test]
async fn index_post_returns_400_given_invalid_request_body() {
    // Arrange
    common::setup();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(common::POOL.clone()))
            .service(web::scope("/api/v1").configure(webhook_config)),
    )
    .await;
    let req = test::TestRequest::default()
        .insert_header((
            "authorization",
            format!("Basic {}", common::DEFAULT_B64).as_str(),
        ))
        .insert_header(ContentType::json())
        .method(Method::POST)
        .uri(format!("/api/v1/webhook/{}", DEFAULT_WEBHOOK_SHORTID).as_str())
        // Not a webhook request body.
        .set_payload("{}")
        .to_request();

    // Act
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(StatusCode::BAD_REQUEST, resp.status());
}
