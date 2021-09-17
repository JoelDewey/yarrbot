//! Private helper functions to aid in building the [DbPool].

use crate::DbPool;
use anyhow::{Context, Result};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use tracing::info;
use yarrbot_common::environment::{
    get_env_var,
    variables::{DB_POOL, DB_URL},
};

const DB_POOL_DEFAULT: u32 = 20;

/// Initializes a [DbPool] that manages [DbPoolConnection]s to the sqlite database.
/// The YARRBOT_DATABASE_URL and YARRBOT_DATABASE_POOL_SIZE environment variables
/// adjust the various properties of the database to connect to.
pub fn build_pool() -> Result<DbPool> {
    let database_url = get_database_url().context(format!("{} must be set.", DB_URL))?;
    let pool_size = get_pool_size();

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .context(format!(
            "Failed to start the connection pool. Is {} correct?",
            DB_URL
        ))?;
    Ok(pool)
}

fn get_database_url() -> Result<String> {
    get_env_var(DB_URL)
}

fn get_pool_size() -> u32 {
    let size = match get_env_var(DB_POOL) {
        Ok(size) => size.parse().unwrap_or(DB_POOL_DEFAULT),
        Err(_) => DB_POOL_DEFAULT,
    };
    info!("Using connection pool size of {}", size);

    size
}
