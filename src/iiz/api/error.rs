//! Unified API error type with JSON responses.
//!
//! Every handler returns `Result<..., ApiError>`. The `IntoResponse` impl
//! maps each variant to the correct HTTP status code and a JSON body:
//! `{ "error": "<type>", "message": "<detail>" }`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// JSON body returned for all error responses.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

/// Unified error type for all iiz API handlers.
#[derive(Debug)]
pub enum ApiError {
    /// 404 — resource not found.
    NotFound(String),
    /// 400 — bad request (validation, malformed input).
    BadRequest(String),
    /// 401 — missing or invalid credentials.
    Unauthorized(String),
    /// 403 — authenticated but not authorized.
    Forbidden(String),
    /// 500 — catch-all internal error.
    Internal(String),
    /// 500 — database-specific error (logged separately).
    Database(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
            ApiError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg),
        };
        let body = ErrorBody {
            error: error_type.to_string(),
            message,
        };
        (status, Json(body)).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ApiError::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

// -- Conversions from common error types --

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => {
                ApiError::NotFound("Resource not found".to_string())
            }
            _ => ApiError::Database(err.to_string()),
        }
    }
}
