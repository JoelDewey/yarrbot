//! The list of environment variables used throughout the application.

// Database environment variables
pub const DB_URL: &str = "YARRBOT_DATABASE_URL";
pub const DB_POOL: &str = "YARRBOT_DATABASE_POOL_SIZE";

// Matrix environment variables
pub const MATRIX_USER: &str = "YARRBOT_MATRIX_USERNAME";
pub const MATRIX_PASS: &str = "YARRBOT_MATRIX_PASSWORD";
pub const MATRIX_HOMESERVER_URL: &str = "YARRBOT_MATRIX_HOMESERVER_URL";
pub const BOT_STORAGE_DIR: &str = "YARRBOT_STORAGE_DIR";
pub const FIRST_MATRIX_USER: &str = "YARRBOT_INITIALIZATION_USER";

// Web API environment variables
pub const WEB_PORT: &str = "YARRBOT_WEB_PORT";

// Miscellaneous
pub const LOG_FILTER: &str = "YARRBOT_LOG_FILTER";
