use anyhow::{Context, Result};
use std::env;
use std::fs;

/// Retrieve some environment variable value by its name. Also checks if the environment variable
/// value is in some file, the path to which is retrieved from an environment variable by the
/// given name concatenated with `_FILE`.
///
/// Returns [Result::Ok()] if a value is successfully retrieved from either environment variable;
/// returns [Result::Err()] otherwise.
pub fn get_env_var(name: &str) -> Result<String> {
    let result = match env::var(name) {
        Ok(s) => Ok(s),
        Err(e) => Ok(get_from_file(name).context(format!(
            "Could not find a value for {} nor for {}_FILE. Original Error: {:?}",
            name, name, e
        ))?),
    };
    if let Ok(s) = result {
        Ok(s.trim().to_string())
    } else {
        result
    }
}

fn get_from_file(name: &str) -> Result<String> {
    let path = env::var(format!("{}_FILE", name))?;
    Ok(fs::read_to_string(path)?)
}

pub mod variables {
    pub use crate::environment_variables::*;
}
