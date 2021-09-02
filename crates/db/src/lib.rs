//! Provides access to database operations to the rest of Yarrbot.

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
use diesel::r2d2::PooledConnection;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

embed_migrations!();

pub mod actions;
mod diesel_types;
pub mod enums;
pub mod models;
mod pool_helper;
mod schema;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbPoolConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub use pool_helper::build_pool;

/// Given some [DbPoolConnection], run the migrations embedded in Yarrbot on the
/// database.
///
/// # Remarks
///
/// This function takes ownership of the given [DbPoolConnection].
pub fn migrate(connection: DbPoolConnection) -> Result<(), anyhow::Error> {
    embedded_migrations::run(&connection)?;
    Ok(())
}
