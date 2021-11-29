use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use url::Url;
use yarrbot_common::environment::{
    get_env_var,
    variables::{BOT_STORAGE_DIR, MATRIX_HOMESERVER_URL, MATRIX_PASS, MATRIX_USER},
};

/// Settings to configure a [YarrbotMatrixClient].
pub(crate) struct YarrbotMatrixClientSettings {
    pub url: Url,
    pub username: String,
    pub password: String,
    pub storage_dir: PathBuf,
}

impl Default for YarrbotMatrixClientSettings {
    /// Create a [YarrbotMatrixClientSettings] by retrieving the values from the environment variables available to
    /// Yarrbot.
    ///
    /// # Note
    ///
    /// This method will panic if the homeserver URL, username, password, or storage directory are not available.
    fn default() -> Self {
        YarrbotMatrixClientSettings {
            url: get_homeserver_url().expect("Could not retrieve Matrix homeserver URL."),
            username: get_username().expect("Could not retrieve Matrix username."),
            password: get_password().expect("Could not retrieve Matrix user password."),
            storage_dir: get_storage_dir()
                .expect("Could not retrieve storage directory for Matrix data."),
        }
    }
}

fn get_homeserver_url() -> Result<Url> {
    let raw = get_env_var(MATRIX_HOMESERVER_URL)?;
    info!(homeserver_url = %raw, "Found homeserver URL.");
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
