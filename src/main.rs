mod first_time_initialization;

extern crate dotenv;

use actix::Actor;
use actix_web::{rt::Arbiter, web, App, HttpServer};
use anyhow::{Context, Result};
use dotenv::dotenv;
use std::str::FromStr;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use yarrbot_common::crypto::initialize_cryptography;
use yarrbot_common::environment::{
    get_env_var,
    variables::{LOG_FILTER, WEB_PORT},
};
use yarrbot_common::{ShutdownActor, SubscribeToShutdown};
use yarrbot_db::{build_pool, migrate};
use yarrbot_matrix_client::{
    client::{
        initialize_matrix_actors, initialize_matrix_sdk_client, initialize_yarrbot_matrix_client,
        YarrbotMatrixClient,
    },
    MatrixSyncActor,
};
use yarrbot_webhook_api::{webhook_config, YarrbotRootSpan};

const DEFAULT_TRACE_FILTER: &str = "warn,yarrbot=info";

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    // Set up logging framework, reading filter configuration from the environment variable
    // or defaulting to warning logs and above globally if the filter isn't specified.
    LogTracer::init().expect("Could not initialize the LogTracer.");
    let filter = get_env_var(LOG_FILTER)
        .and_then(|f| EnvFilter::from_str(&f).context("Failed to parse tracer filter string."))
        .unwrap_or_else(|_| {
            EnvFilter::from_str(DEFAULT_TRACE_FILTER).expect("Default trace filter is invalid.")
        });
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let subscriber = Registry::default()
        .with(filter)
        .with(tracing_subscriber::fmt::Layer::default().with_writer(non_blocking_writer));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set global subscriber for tracing.");

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

    // Set up the shutdown actor, which listens for signals telling Yarrbot to stop.
    let shutdown_addr = ShutdownActor::default().start();

    info!("Starting up the connection to the Matrix server...");
    let matrix_client = initialize_matrix_sdk_client().await?;
    let (_room_message_addr, send_addr, _stripped_state_addr) =
        initialize_matrix_actors(matrix_client.clone(), pool.clone())?;
    let yarrbot_matrix_client =
        initialize_yarrbot_matrix_client(matrix_client.clone(), pool.clone(), send_addr.clone())
            .await?;

    // The Matrix SDK sync loop locks up the system event loop, so we move it to its own arbiter (thus its own thread).
    let sync_arbiter = Arbiter::new();
    let sync_shutdown_addr = shutdown_addr.clone();
    let sync_fut = async move {
        let sync_addr = MatrixSyncActor::new(matrix_client).start();
        sync_shutdown_addr.do_send(SubscribeToShutdown(sync_addr.recipient()));
    };
    sync_arbiter.spawn(sync_fut);

    info!("Yarrbot started.");
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<YarrbotRootSpan>::new())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(yarrbot_matrix_client.clone()))
            .service(web::scope("/api/v1").configure(webhook_config::<YarrbotMatrixClient>))
    })
    .bind(format!("127.0.0.1:{}", get_port()?))?
    .run()
    .await?;

    info!("Yarrbot shut down.");
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
