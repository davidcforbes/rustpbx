//! Shared response types matching the backend API envelope.

use serde::{Deserialize, Serialize};

/// Pagination metadata returned with list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
}

/// Paginated list response from any collection endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    #[serde(flatten)]
    pub pagination: PaginationMeta,
    pub items: Vec<T>,
}

/// Error body returned by the API on failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

/// Generic API response — either data or error.
///
/// Used internally by the api_* helpers to parse error bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok(T),
    Err(ErrorBody),
}

impl<T> ApiResponse<T> {
    /// Create an error response (used as fallback when JSON parsing fails).
    pub fn error(_status: u16, msg: &str) -> Self {
        ApiResponse::Err(ErrorBody {
            error: "unknown".to_string(),
            message: msg.to_string(),
        })
    }

    /// Extract the error message (or a default).
    pub fn message(&self) -> &str {
        match self {
            ApiResponse::Ok(_) => "ok",
            ApiResponse::Err(e) => &e.message,
        }
    }
}
