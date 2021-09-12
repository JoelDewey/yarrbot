mod first_time_initialization;
mod matrix_initialization;

extern crate dotenv;

use crate::matrix_initialization::initialize_matrix_client;
use anyhow::{Context, Result};
use dotenv::dotenv;
use tracing::info;
use tracing_subscriber;
use std::str::FromStr;
use tokio::runtime::Handle;
use yarrbot_common::crypto::initialize_cryptography;
use yarrbot_common::environment::{
    get_env_var,
    variables::{LOG_FILTER, WEB_PORT},
};
use yarrbot_db::{build_pool, migrate};
use yarrbot_matrix_client::YarrbotMatrixClient;
use yarrbot_webhook_api::webhook_config;
use tracing_subscriber::EnvFilter;
use tracing::level_filters::LevelFilter;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    use actix_web::{web, App, HttpServer};

    dotenv().ok();

    // Set up logging framework, reading filter configuration from the environment variable
    // or defaulting to warning logs and above globally if the filter isn't specified.
    let filter = EnvFilter::try_from_env(LOG_FILTER)
        .unwrap_or_else(|_| EnvFilter::default())
        .add_directive(LevelFilter::WARN.into());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("Initializing Yarrbot...");

    initialize_cryptography()?;

    info!("Initializing database connection pool...");
    let pool = build_pool()?;
    let connection = pool
        .get()
        .context("Could not retrieve a connection from the connection pool.")?;
    info!("Migrating the database...");
    migrate(connection)?;

    info!("Running any first-time setup functions...");
    first_time_initialization::first_time_initialization(&pool)?;

    info!("Starting up the connection to the Matrix server...");
    let matrix_client = initialize_matrix_client(pool.clone()).await?;
    let matrix_client2 = matrix_client.clone();
    let handle = Handle::current();

    std::thread::spawn(move || {
        handle.spawn(async move {
            matrix_client2.start_sync_loop().await.unwrap();
        });
    });

    info!("Staring up web server...");
    let http_server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(matrix_client.clone()))
            .service(web::scope("/api/v1").configure(webhook_config::<YarrbotMatrixClient>))
    })
    .bind(format!("127.0.0.1:{}", get_port()?))?
    .run();

    info!("Yarrbot started!");
    http_server.await?;

    info!("Shutting Yarrbot down.");
    Ok(())
}

fn get_port() -> Result<String> {
    let value = match get_env_var(WEB_PORT) {
        Ok(v) => v,
        Err(_) => String::from("8080"),
    };
    match u16::from_str(&value) {
        Ok(_) => Ok(value),
        Err(e) => Err(e).context(format!("Failed to parse \"{}\" as a valid port.", value)),
    }
}
