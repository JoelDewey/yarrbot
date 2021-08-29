//! Private helper functions to aid in building the [DbPool].

use crate::DbPool;
use anyhow::{Context, Result};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use yarrbot_common::environment;

const DB_URL_ENV: &str = "YARRBOT_DATABASE_URL";
const DB_POOL_ENV: &str = "YARRBOT_DATABASE_POOL_SIZE";
const DB_POOL_DEFAULT: u32 = 20;

pub fn build_pool() -> Result<DbPool> {
    let database_url = get_database_url().context(format!("{} must be set.", DB_URL_ENV))?;
    let pool_size = get_pool_size();

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .context(format!(
            "Failed to start the connection pool. Is {} correct?",
            DB_URL_ENV
        ))?;
    Ok(pool)
}

fn get_database_url() -> Result<String> {
    environment::get_env_var(DB_URL_ENV)
}

fn get_pool_size() -> u32 {
    match environment::get_env_var(DB_POOL_ENV) {
        Ok(size) => size.parse().unwrap_or(DB_POOL_DEFAULT),
        Err(_) => {
            info!(
                "No value found for {}, using the default value {}.",
                DB_POOL_ENV, DB_POOL_DEFAULT
            );
            DB_POOL_DEFAULT
        }
    }
}
