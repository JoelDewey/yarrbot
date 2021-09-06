extern crate dotenv;

use lazy_static::{initialize, lazy_static};
use std::sync::Once;
use yarrbot_db::{build_pool, DbPool};

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
