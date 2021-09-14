extern crate dotenv;

use async_trait::async_trait;
use lazy_static::{initialize, lazy_static};
use std::sync::{Arc, Once};
use tokio::sync::RwLock;
use yarrbot_db::models::MatrixRoom;
use yarrbot_db::{build_pool, DbPool};
use yarrbot_matrix_client::message::MessageData;
use yarrbot_matrix_client::MatrixClient;

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
