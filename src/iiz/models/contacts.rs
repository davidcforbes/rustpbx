//! Diesel model structs for the `contact_lists`, `contact_list_members`,
//! `blocked_numbers`, `dnc_entries`, and `dnt_entries` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::schema::iiz::{
    blocked_numbers, contact_list_members, contact_lists, dnc_entries, dnt_entries,
};

// ---------------------------------------------------------------------------
// contact_lists
// ---------------------------------------------------------------------------

/// Read model for the `iiz.contact_lists` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = contact_lists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContactList {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new contact list.
/// `id`, `member_count`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = contact_lists)]
pub struct NewContactList {
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Update model for partial contact list updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = contact_lists)]
pub struct UpdateContactList {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// contact_list_members
// ---------------------------------------------------------------------------

/// Read model for the `iiz.contact_list_members` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = contact_list_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContactListMember {
    pub id: Uuid,
    pub account_id: Uuid,
    pub list_id: Uuid,
    pub phone: String,
    pub contact_name: Option<String>,
    pub added_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new contact list member.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `added_at` is user-facing and included here.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = contact_list_members)]
pub struct NewContactListMember {
    pub account_id: Uuid,
    pub list_id: Uuid,
    pub phone: String,
    pub contact_name: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Update model for partial contact list member updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = contact_list_members)]
pub struct UpdateContactListMember {
    pub phone: Option<String>,
    pub contact_name: Option<Option<String>>,
    pub added_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// blocked_numbers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.blocked_numbers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = blocked_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockedNumber {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub cnam: Option<String>,
    pub calls_blocked: i32,
    pub last_blocked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new blocked number.
/// `id`, `calls_blocked`, `last_blocked_at`, `created_at`, `updated_at`, and `deleted_at`
/// are set by database defaults. `calls_blocked` and `last_blocked_at` are system-updated.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = blocked_numbers)]
pub struct NewBlockedNumber {
    pub account_id: Uuid,
    pub number: String,
    pub cnam: Option<String>,
}

/// Update model for partial blocked number updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `calls_blocked` and `last_blocked_at` are system-updated and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = blocked_numbers)]
pub struct UpdateBlockedNumber {
    pub number: Option<String>,
    pub cnam: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// dnc_entries
// ---------------------------------------------------------------------------

/// Read model for the `iiz.dnc_entries` table (Do-Not-Call).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dnc_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DncEntry {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub added_by_id: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new DNC entry.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dnc_entries)]
pub struct NewDncEntry {
    pub account_id: Uuid,
    pub number: String,
    pub added_by_id: Option<Uuid>,
    pub reason: Option<String>,
}

/// Update model for partial DNC entry updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dnc_entries)]
pub struct UpdateDncEntry {
    pub number: Option<String>,
    pub added_by_id: Option<Option<Uuid>>,
    pub reason: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// dnt_entries
// ---------------------------------------------------------------------------

/// Read model for the `iiz.dnt_entries` table (Do-Not-Text).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dnt_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DntEntry {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub e164: String,
    pub rejected_count: i32,
    pub last_rejected_at: Option<DateTime<Utc>>,
    pub added_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new DNT entry.
/// `id`, `rejected_count`, `last_rejected_at`, `created_at`, `updated_at`, and `deleted_at`
/// are set by database defaults. `rejected_count` and `last_rejected_at` are system-updated.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dnt_entries)]
pub struct NewDntEntry {
    pub account_id: Uuid,
    pub number: String,
    pub e164: String,
    pub added_by_id: Option<Uuid>,
}

/// Update model for partial DNT entry updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `rejected_count` and `last_rejected_at` are system-updated and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dnt_entries)]
pub struct UpdateDntEntry {
    pub number: Option<String>,
    pub e164: Option<String>,
    pub added_by_id: Option<Option<Uuid>>,
}
