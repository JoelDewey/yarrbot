use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use std::str::FromStr;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing_actix_web::TracingLogger;
use yarrbot_common::environment::get_env_var;
use yarrbot_common::environment::variables::WEB_PORT;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::YarrbotMatrixClient;
use yarrbot_webhook_api::{webhook_config, YarrbotRootSpan};

pub fn initialize_web_server(
    pool: DbPool,
    matrix_client: YarrbotMatrixClient,
    shutdown_complete_tx: Sender<()>,
) -> Result<JoinHandle<()>> {
    let http_server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<YarrbotRootSpan>::new())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(matrix_client.clone()))
            .service(web::scope("/api/v1").configure(webhook_config::<YarrbotMatrixClient>))
    })
    .bind(format!("127.0.0.1:{}", get_port()?))?
    .run();
    Ok(tokio::spawn(async move {
        http_server.await.expect("Actix server failed.");
        // Actix-Web has its own facility for listening to shutdown signals.
        // Drop the channel sender once that happens (the future completes).
        drop(shutdown_complete_tx);
    }))
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
