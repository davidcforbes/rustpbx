//! Pagination helpers for list endpoints.
//!
//! Query parameters are extracted into `ListParams`, validated/clamped, and
//! responses are wrapped in `ListResponse<T>` with a `PaginationMeta` sidecar.

use serde::{Deserialize, Serialize};

/// Query params for paginated list endpoints.
///
/// Defaults: page=1, per_page=25. Max per_page is clamped to 100.
#[derive(Debug, Deserialize)]
pub struct ListParams {
    /// 1-based page number.
    #[serde(default = "default_page")]
    pub page: i64,
    /// Items per page (clamped to 1..=100).
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    /// Optional sort expression (e.g. "created_at:desc").
    #[serde(default)]
    pub sort: Option<String>,
    /// Optional full-text search query.
    #[serde(default)]
    pub q: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    25
}

impl ListParams {
    /// Normalize page/per_page to valid ranges and return `(offset, limit)`.
    pub fn normalize(&self) -> (i64, i64) {
        let per_page = self.per_page.clamp(1, 100);
        let page = self.page.max(1);
        let offset = (page - 1) * per_page;
        (offset, per_page)
    }
}

/// Pagination metadata included in list responses.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
}

impl PaginationMeta {
    pub fn new(page: i64, per_page: i64, total_items: i64) -> Self {
        let per_page = per_page.max(1);
        let total_pages = (total_items + per_page - 1) / per_page;
        let page = page.max(1);
        Self {
            page,
            per_page,
            total_items,
            total_pages,
            has_prev: page > 1,
            has_next: page < total_pages,
        }
    }
}

/// Standard paginated list response envelope.
///
/// Serializes as a flat JSON object with pagination fields alongside `items`.
#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    #[serde(flatten)]
    pub pagination: PaginationMeta,
    pub items: Vec<T>,
}
