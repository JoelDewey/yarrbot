//! Private helper functions to aid in building the [DbPool].

use crate::{DbPool};
use diesel::{r2d2::{ConnectionManager, Pool}, PgConnection};
use anyhow::Context;
use std::env;
use std::env::VarError;

const DB_URL_ENV: &str = "YARRBOT_DATABASE_URL";
const DB_POOL_ENV: &str = "YARRBOT_DATABASE_POOL_SIZE";
const DB_POOL_DEFAULT: u32 = 20;

pub fn build_pool() -> Result<DbPool, anyhow::Error> {
    let database_url = get_database_url()
        .context(format!("{} must be set.", DB_URL_ENV))?;
    let pool_size = get_pool_size();

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .context(format!("Failed to start the connection pool. Is {} correct?", DB_URL_ENV))?;
    Ok(pool)
}

fn get_database_url() -> Result<String, VarError> {
    env::var(DB_URL_ENV)
}

fn get_pool_size() -> u32 {
    match env::var(DB_POOL_ENV) {
        Ok(size) => size.parse().unwrap_or(DB_POOL_DEFAULT),
        _ => DB_POOL_DEFAULT
    }
}