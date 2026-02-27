//! Diesel model structs for the `tags` table.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::schema::iiz::tags;

// ---------------------------------------------------------------------------
// tags
// ---------------------------------------------------------------------------

/// Read model for the `iiz.tags` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new tag.
/// `id`, `usage_count`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `usage_count` is system-maintained and defaults to 0.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = tags)]
pub struct NewTag {
    pub account_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

/// Update model for partial tag updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `usage_count` is system-maintained and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = tags)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<Option<String>>,
    pub description: Option<Option<String>>,
}
