extern crate dotenv;

use anyhow::Context;
use dotenv::dotenv;
use yarrbot_db::{initialize_pool, migrate};

fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let pool = initialize_pool()?;
    let connection = pool
        .get()
        .context("Could not retrieve a connection from the connection pool.")?;
    migrate(connection)?;
    Ok(())
}
