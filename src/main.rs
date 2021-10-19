mod first_time_initialization;
mod matrix_initialization;
mod web_initialization;

extern crate dotenv;

use crate::matrix_initialization::{
    initialize_matrix_components, start_send_handler, start_sync_handler,
};
use crate::web_initialization::initialize_web_server;
use anyhow::{bail, Context, Result};
use dotenv::dotenv;
use std::str::FromStr;
use std::time::Duration;
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tokio::sync::mpsc;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use yarrbot_common::crypto::initialize_cryptography;
use yarrbot_common::environment::{get_env_var, variables::LOG_FILTER};
use yarrbot_db::{build_pool, migrate};

const DEFAULT_TRACE_FILTER: &str = "warn,yarrbot=info";
const SHUTDOWN_WAIT_TIME_SEC: u64 = 10;

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

    info!("Starting up the connection to the Matrix server...");
    let (shutdown_tx, shutdown_rx) = watch::channel(true);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);
    let (matrix_client, message_handler, sync_handler) =
        initialize_matrix_components(pool.clone(), shutdown_rx).await?;
    let send_handle = start_send_handler(message_handler, shutdown_complete_tx.clone());
    let sync_handle = start_sync_handler(sync_handler, shutdown_complete_tx.clone());

    info!("Staring up web server...");
    let http_handle = initialize_web_server(pool, matrix_client, shutdown_complete_tx.clone())
        .expect("Failed to start web server.");
    let handles = vec![sync_handle, send_handle, http_handle];

    info!("Yarrbot started!");
    wait_for_signal()
        .await
        .expect("Failed to wait for closing signals.");

    if let Err(e) = shutdown_tx.send(false) {
        error!(error = ?e, "Failed to send shutdown signal, aborting workers.");
        handles.iter().map(|h| h.abort()).count();
        bail!("Force exiting due to failures.");
    }

    // Drop the original shutdown_complete sender channel and then wait for the receiver channel
    // future to complete, which indicates that all of the workers have shut down.
    info!(
        "Shutting down Yarrbot, waiting up to {} seconds.",
        SHUTDOWN_WAIT_TIME_SEC
    );
    drop(shutdown_complete_tx);
    tokio::select! {
        _ = shutdown_complete_rx.recv() => {},
        _ = tokio::time::sleep(Duration::from_secs(SHUTDOWN_WAIT_TIME_SEC)) => {
            warn!("Yarrbot failed to shutdown within a timely manner. Data may have been lost.");
        }
    }

    info!("Yarrbot shut down.");
    Ok(())
}

async fn wait_for_signal() -> Result<()> {
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        res = ctrl_c() => {
            debug!("Received SIGINT.");
            if let Err(e) = res {
                error!(error = ?e, "Encountered error while listening for SIGINT.");
            }
        },
        _ = sigterm.recv() => {
            debug!("Received SIGTERM.");
        }
    }

    Ok(())
}
