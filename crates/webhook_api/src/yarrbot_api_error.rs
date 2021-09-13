//! Helper utilities for returning API errors to clients.

use actix_web::http::StatusCode;
use actix_web::{error::ResponseError, HttpResponse};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum YarrbotApiError {
    #[error("{message:?}")]
    UserError {
        status: YarrbotStatusCode,
        message: String,
        source: Option<anyhow::Error>,
    },
    #[error("Internal Server Error")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug)]
pub enum YarrbotStatusCode {
    BadRequest,
    NotFound,
    Unauthorized,
}

#[derive(Debug, Serialize)]
struct YarrbotUserErrorMessage<'a> {
    pub status: u16,
    pub message: &'a str,
}

impl ResponseError for YarrbotApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            YarrbotApiError::UserError { status, .. } => match status {
                YarrbotStatusCode::BadRequest => StatusCode::BAD_REQUEST,
                YarrbotStatusCode::NotFound => StatusCode::NOT_FOUND,
                YarrbotStatusCode::Unauthorized => StatusCode::UNAUTHORIZED,
            },
            YarrbotApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            YarrbotApiError::UserError {
                message, source, ..
            } => {
                if source.is_some() {
                    error!("{:?}", source.as_ref().unwrap());
                }
                let code = self.status_code();
                let code_u16 = code.as_u16();
                HttpResponse::build(code).json(YarrbotUserErrorMessage {
                    status: code_u16,
                    message,
                })
            }
            YarrbotApiError::InternalError(source) => {
                error!("{:?}", source);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(
                    YarrbotUserErrorMessage {
                        status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        message: "Internal Server Error",
                    },
                )
            }
        }
    }
}

impl YarrbotApiError {
    /// Create a new instance of [YarrbotApiError] with a given [StatusCode].
    /// Before creating a new [YarrbotApiError], check for dedicated methods
    /// to return the appropriate status code.
    fn new(message: &str, status: YarrbotStatusCode, inner: Option<anyhow::Error>) -> Self {
        YarrbotApiError::UserError {
            message: String::from(message),
            status,
            source: inner,
        }
    }

    #[allow(dead_code)]
    pub fn bad_request(message: &str, inner: Option<anyhow::Error>) -> Self {
        Self::new(message, YarrbotStatusCode::BadRequest, inner)
    }

    pub fn not_found(inner: Option<anyhow::Error>) -> Self {
        Self::new("Not Found", YarrbotStatusCode::NotFound, inner)
    }

    pub fn unauthorized(inner: Option<anyhow::Error>) -> Self {
        Self::new("Unauthorized", YarrbotStatusCode::Unauthorized, inner)
    }

    pub fn internal_server_error(inner: anyhow::Error) -> Self {
        YarrbotApiError::InternalError(inner)
    }
}
