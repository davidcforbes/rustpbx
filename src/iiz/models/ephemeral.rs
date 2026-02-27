//! Diesel model structs for ephemeral/UNLOGGED tables in the iiz schema.
//!
//! These tables store transient state (presence, SIP registrations, rate limits)
//! and live on UNLOGGED tables for write performance. Each table follows the
//! Read + Insert + Update (3-struct) pattern.
//!
//! Note: `active_calls` and `monitoring_events` are modeled in `activities.rs`.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{AgentStatus, SipTransport};
use crate::iiz::schema::iiz::{frequency_limits, locations, presence};

// ---------------------------------------------------------------------------
// presence (NON-STANDARD PK: identity TEXT, not UUID)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.presence` table.
/// Primary key is `identity` (Text), not the standard UUID `id`.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = presence)]
#[diesel(primary_key(identity))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Presence {
    pub identity: String,
    pub account_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status: AgentStatus,
    pub note: Option<String>,
    pub activity: Option<String>,
    pub current_call_id: Option<Uuid>,
    pub last_updated: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new presence record.
/// `identity` is user-provided (it is the primary key). `deleted_at` excluded.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = presence)]
pub struct NewPresence {
    pub identity: String,
    pub account_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status: AgentStatus,
    pub note: Option<String>,
    pub activity: Option<String>,
    pub current_call_id: Option<Uuid>,
    pub last_updated: DateTime<Utc>,
}

/// Update model for partial presence updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `identity` (PK) is excluded.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = presence)]
pub struct UpdatePresence {
    pub account_id: Option<Option<Uuid>>,
    pub user_id: Option<Option<Uuid>>,
    pub status: Option<AgentStatus>,
    pub note: Option<Option<String>>,
    pub activity: Option<Option<String>>,
    pub current_call_id: Option<Option<Uuid>>,
    pub last_updated: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// locations (nullable account_id)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.locations` table.
/// `account_id` is nullable (unlike most other tables).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = locations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Location {
    pub id: Uuid,
    pub account_id: Option<Uuid>,
    pub aor: String,
    pub username: Option<String>,
    pub realm: Option<String>,
    pub destination: String,
    pub expires: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub supports_webrtc: bool,
    pub source_ip: Option<String>,
    pub source_port: Option<i32>,
    pub transport: Option<SipTransport>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new location (SIP registration).
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = locations)]
pub struct NewLocation {
    pub account_id: Option<Uuid>,
    pub aor: String,
    pub username: Option<String>,
    pub realm: Option<String>,
    pub destination: String,
    pub expires: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub supports_webrtc: bool,
    pub source_ip: Option<String>,
    pub source_port: Option<i32>,
    pub transport: Option<SipTransport>,
}

/// Update model for partial location updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = locations)]
pub struct UpdateLocation {
    pub account_id: Option<Option<Uuid>>,
    pub aor: Option<String>,
    pub username: Option<Option<String>>,
    pub realm: Option<Option<String>>,
    pub destination: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub user_agent: Option<Option<String>>,
    pub supports_webrtc: Option<bool>,
    pub source_ip: Option<Option<String>>,
    pub source_port: Option<Option<i32>>,
    pub transport: Option<Option<SipTransport>>,
}

// ---------------------------------------------------------------------------
// frequency_limits (standard CRUD)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.frequency_limits` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = frequency_limits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FrequencyLimit {
    pub id: Uuid,
    pub account_id: Uuid,
    pub policy_id: String,
    pub scope: String,
    pub limit_type: String,
    pub max_count: i32,
    pub current_count: i32,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new frequency limit.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `current_count` defaults to 0 but is included for explicit initialization.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = frequency_limits)]
pub struct NewFrequencyLimit {
    pub account_id: Uuid,
    pub policy_id: String,
    pub scope: String,
    pub limit_type: String,
    pub max_count: i32,
    pub current_count: i32,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
}

/// Update model for partial frequency limit updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `current_count` is system-maintained and excluded from API-driven updates.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = frequency_limits)]
pub struct UpdateFrequencyLimit {
    pub policy_id: Option<String>,
    pub scope: Option<String>,
    pub limit_type: Option<String>,
    pub max_count: Option<i32>,
    pub window_start: Option<Option<DateTime<Utc>>>,
    pub window_end: Option<Option<DateTime<Utc>>>,
}
