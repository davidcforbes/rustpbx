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

// -------------------------------------------------------------------------
// Domain response types for Contacts section
// -------------------------------------------------------------------------

/// A contact list returned by GET /contacts/lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A blocked number returned by GET /contacts/blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedNumberItem {
    pub id: String,
    pub number: String,
    pub cnam: Option<String>,
    pub calls_blocked: i32,
    pub last_blocked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A Do-Not-Call entry returned by GET /contacts/dnc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DncEntryItem {
    pub id: String,
    pub number: String,
    pub added_by_id: Option<String>,
    pub reason: Option<String>,
    pub created_at: String,
}

/// A Do-Not-Text entry returned by GET /contacts/dnt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DntEntryItem {
    pub id: String,
    pub number: String,
    pub e164: String,
    pub rejected_count: i32,
    pub last_rejected_at: Option<String>,
    pub added_by_id: Option<String>,
    pub created_at: String,
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
