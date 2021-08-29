mod initialization;

extern crate dotenv;
#[macro_use]
extern crate log;

use anyhow::Context;
use dotenv::dotenv;
use env_logger::{Builder, Env};
use tokio::runtime::Handle;
use yarrbot_db::{initialize_pool, migrate};
use yarrbot_matrix_client::initialize_matrix_client;
use yarrbot_webhook_api::webhook_config;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    use actix_web::{web, App, HttpServer};

    dotenv().ok();

    // Set up logging framework, reading filter configuration from the environment variable
    // or defaulting to warning logs and above globally if the filter isn't specified.
    let log_env = Env::new().filter_or("YARRBOT_LOG_FILTER", "warn");
    Builder::from_env(log_env).init();

    info!("Initializing Yarrbot...");

    info!("Initializing database connection pool...");
    let pool = initialize_pool()?;
    let connection = pool
        .get()
        .context("Could not retrieve a connection from the connection pool.")?;
    info!("Migrating the database...");
    migrate(connection)?;
    
    info!("Running any first-time setup functions...");
    initialization::first_time_initialization(&pool)?;

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
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(matrix_client.clone()))
            .service(web::scope("/api/v1").configure(webhook_config))
    })
    .bind("127.0.0.1:8080")?
    .run();

    info!("Yarrbot started!");
    http_server.await?;

    info!("Shutting Yarrbot down.");
    Ok(())
}
