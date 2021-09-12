//! Helper utilities for returning API errors to clients.

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter};

/// Represents an error to send back to clients (e.g. Sonarr or Radarr).
#[derive(Debug, Serialize)]
pub struct YarrbotApiError {
    pub status: u16,
    pub message: String,
}

impl ResponseError for YarrbotApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap()
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(self)
    }
}

impl YarrbotApiError {
    /// Create a new instance of [YarrbotApiError] with a given [StatusCode].
    /// Before creating a new [YarrbotApiError], check for dedicated methods
    /// to return the appropriate status code.
    pub fn new(message: &str, status: StatusCode) -> Self {
        YarrbotApiError {
            message: String::from(message),
            status: status.as_u16(),
        }
    }

    #[allow(dead_code)]
    pub fn bad_request(message: &str) -> Self {
        Self::new(message, StatusCode::BAD_REQUEST)
    }

    pub fn not_found() -> Self {
        Self::new("Not Found", StatusCode::NOT_FOUND)
    }

    pub fn unauthorized() -> Self {
        Self::new("Unauthorized", StatusCode::UNAUTHORIZED)
    }

    pub fn internal_server_error() -> Self {
        Self::new("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl Display for YarrbotApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| String::from("{ message: \"Fatal Error\" }"))
        )
    }
}
