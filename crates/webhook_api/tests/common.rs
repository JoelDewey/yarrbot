extern crate dotenv;

use async_trait::async_trait;
use lazy_static::{initialize, lazy_static};
use std::sync::{Arc, Once};
use tokio::sync::RwLock;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::{build_pool, DbPool};
use yarrbot_matrix_client::message::MessageData;
use yarrbot_matrix_client::MatrixClient;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry};
use yarrbot_common::environment::variables::LOG_FILTER;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

static INIT: Once = Once::new();

lazy_static! {
    pub static ref POOL: DbPool = build_pool().unwrap();
}

// testuser:myP@ssw0rd123
pub const DEFAULT_B64: &str = "dGVzdHVzZXI6bXlQQHNzdzByZDEyMw==";

pub fn setup() {
    INIT.call_once(|| {
        dotenv::from_filename("integrationtest.env").ok();
        initialize(&POOL);

        // Copy and paste of what is in main.rs of the bin crate.
        // Extracting this to a function in common.rs causes tracing to stop working.
        LogTracer::init().expect("Could not initialize the LogTracer.");
        let filter = EnvFilter::try_from_env(LOG_FILTER)
            .unwrap_or_else(|_| EnvFilter::default())
            .add_directive(LevelFilter::WARN.into());
        let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
        let subscriber = Registry::default()
            .with(filter)
            .with(tracing_subscriber::fmt::Layer::default().with_writer(non_blocking_writer));
        tracing::subscriber::set_global_default(subscriber).expect("Could not set global subscriber for tracing.");
    });
}

/// Fake implementation of [MatrixClient] that captures the [MessageData]+[MatrixRoom] information provided to it.
#[derive(Clone)]
pub struct SpyMatrixClient {
    messages: Arc<RwLock<Vec<(MessageData, MatrixRoom)>>>,
}

impl SpyMatrixClient {
    pub fn new() -> Self {
        SpyMatrixClient {
            messages: Arc::new(RwLock::new(Vec::<(MessageData, MatrixRoom)>::new())),
        }
    }
}

impl Default for SpyMatrixClient {
    fn default() -> Self {
        SpyMatrixClient::new()
    }
}

#[async_trait]
impl MatrixClient for SpyMatrixClient {
    async fn send_message(&self, message: &MessageData, room: &MatrixRoom) -> anyhow::Result<()> {
        let mut messages = self.messages.write().await;
        messages.push((
            MessageData {
                plain: message.plain.clone(),
                html: message.html.clone(),
            },
            room.clone(),
        ));

        Ok(())
    }
}
