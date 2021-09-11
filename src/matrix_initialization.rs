use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use url::Url;
use yarrbot_common::environment::{
    get_env_var,
    variables::{BOT_STORAGE_DIR, MATRIX_HOMESERVER_URL, MATRIX_PASS, MATRIX_USER},
};
use yarrbot_db::DbPool;
use yarrbot_matrix_client::{YarrbotMatrixClient, YarrbotMatrixClientSettings};

fn get_homeserver_url() -> Result<Url> {
    let raw = get_env_var(MATRIX_HOMESERVER_URL)?;
    info!("Received homeserver URL: {}", &raw);
    Url::parse(&raw).with_context(|| "Parsing of homeserver URL failed.")
}

fn get_username() -> Result<String> {
    get_env_var(MATRIX_USER)
        .with_context(|| "Could not retrieve the Matrix username from the environment.")
}

fn get_password() -> Result<String> {
    get_env_var(MATRIX_PASS)
        .with_context(|| "Could not retrieve the Matrix password from the environment.")
}

fn get_storage_dir() -> Result<PathBuf> {
    let mut path = match get_env_var(BOT_STORAGE_DIR) {
        Ok(s) => PathBuf::from(s),
        Err(_) => {
            info!("No storage directory specified, using the current directory instead.");
            env::current_dir()?
        }
    };
    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => {
            fs::create_dir_all(&path)?;
            fs::metadata(&path)?
        }
    };
    if metadata.is_file() {
        bail!("Storage directory path is a file: {}", path.display());
    }
    if metadata.permissions().readonly() {
        bail!("Storage directory path is readonly: {}", path.display());
    }
    path.push("matrix");
    match fs::metadata(&path) {
        Ok(_) => (),
        Err(_) => {
            fs::create_dir(&path)?;
        }
    }
    Ok(path)
}

pub async fn initialize_matrix_client(pool: DbPool) -> Result<YarrbotMatrixClient> {
    let settings = YarrbotMatrixClientSettings {
        username: get_username()?,
        password: get_password()?,
        url: get_homeserver_url()?,
        storage_dir: get_storage_dir()?,
    };
    Ok(YarrbotMatrixClient::new(settings, pool).await?)
}
