//! Diesel model structs for the `tracking_numbers`, `tracking_sources`,
//! `receiving_numbers`, `target_numbers`, `text_numbers`, `number_pools`,
//! `number_pool_members`, `caller_id_cnam`, `port_requests`, and `call_settings` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{ComplianceStatus, NumberClass, NumberType};
use crate::iiz::schema::iiz::{
    call_settings, caller_id_cnam, number_pool_members, number_pools, port_requests,
    receiving_numbers, target_numbers, text_numbers, tracking_numbers, tracking_sources,
};

// ---------------------------------------------------------------------------
// tracking_numbers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.tracking_numbers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = tracking_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TrackingNumber {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub source_id: Option<Uuid>,
    pub routing_description: Option<String>,
    pub routing_type: Option<String>,
    pub routing_target_type: Option<String>,
    pub routing_target_id: Option<Uuid>,
    pub text_enabled: bool,
    pub receiving_number_id: Option<Uuid>,
    pub number_type: NumberType,
    pub number_class: NumberClass,
    pub pool_id: Option<Uuid>,
    pub billing_date: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new tracking number.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = tracking_numbers)]
pub struct NewTrackingNumber {
    pub account_id: Uuid,
    pub number: String,
    pub source_id: Option<Uuid>,
    pub routing_description: Option<String>,
    pub routing_type: Option<String>,
    pub routing_target_type: Option<String>,
    pub routing_target_id: Option<Uuid>,
    pub text_enabled: bool,
    pub receiving_number_id: Option<Uuid>,
    pub number_type: NumberType,
    pub number_class: NumberClass,
    pub pool_id: Option<Uuid>,
    pub billing_date: Option<i32>,
    pub is_active: bool,
}

/// Update model for partial tracking number updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = tracking_numbers)]
pub struct UpdateTrackingNumber {
    pub number: Option<String>,
    pub source_id: Option<Option<Uuid>>,
    pub routing_description: Option<Option<String>>,
    pub routing_type: Option<Option<String>>,
    pub routing_target_type: Option<Option<String>>,
    pub routing_target_id: Option<Option<Uuid>>,
    pub text_enabled: Option<bool>,
    pub receiving_number_id: Option<Option<Uuid>>,
    pub number_type: Option<NumberType>,
    pub number_class: Option<NumberClass>,
    pub pool_id: Option<Option<Uuid>>,
    pub billing_date: Option<Option<i32>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// tracking_sources
// ---------------------------------------------------------------------------

/// Read model for the `iiz.tracking_sources` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = tracking_sources)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TrackingSource {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub source_type: Option<String>,
    pub position: i32,
    pub last_touch: bool,
    pub number_count: i32,
    pub call_count: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new tracking source.
/// `id`, `number_count`, `call_count`, `created_at`, `updated_at`, and `deleted_at`
/// are set by database defaults. `number_count` and `call_count` are system-maintained.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = tracking_sources)]
pub struct NewTrackingSource {
    pub account_id: Uuid,
    pub name: String,
    pub source_type: Option<String>,
    pub position: i32,
    pub last_touch: bool,
    pub status: String,
}

/// Update model for partial tracking source updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `number_count` and `call_count` are system-maintained and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = tracking_sources)]
pub struct UpdateTrackingSource {
    pub name: Option<String>,
    pub source_type: Option<Option<String>>,
    pub position: Option<i32>,
    pub last_touch: Option<bool>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// receiving_numbers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.receiving_numbers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = receiving_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReceivingNumber {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub description: Option<String>,
    pub tracking_count: i32,
    pub total_calls: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new receiving number.
/// `id`, `tracking_count`, `total_calls`, `created_at`, `updated_at`, and `deleted_at`
/// are set by database defaults. `tracking_count` and `total_calls` are system-maintained.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = receiving_numbers)]
pub struct NewReceivingNumber {
    pub account_id: Uuid,
    pub number: String,
    pub description: Option<String>,
}

/// Update model for partial receiving number updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `tracking_count` and `total_calls` are system-maintained and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = receiving_numbers)]
pub struct UpdateReceivingNumber {
    pub number: Option<String>,
    pub description: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// target_numbers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.target_numbers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = target_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TargetNumber {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub priority: i32,
    pub concurrency_cap: Option<i32>,
    pub weight: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new target number.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = target_numbers)]
pub struct NewTargetNumber {
    pub account_id: Uuid,
    pub number: String,
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub priority: i32,
    pub concurrency_cap: Option<i32>,
    pub weight: i32,
    pub status: String,
}

/// Update model for partial target number updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = target_numbers)]
pub struct UpdateTargetNumber {
    pub number: Option<String>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub target_type: Option<String>,
    pub priority: Option<i32>,
    pub concurrency_cap: Option<Option<i32>>,
    pub weight: Option<i32>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// text_numbers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.text_numbers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = text_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TextNumber {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub name: Option<String>,
    pub is_assigned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new text number.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = text_numbers)]
pub struct NewTextNumber {
    pub account_id: Uuid,
    pub number: String,
    pub name: Option<String>,
    pub is_assigned: bool,
}

/// Update model for partial text number updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = text_numbers)]
pub struct UpdateTextNumber {
    pub number: Option<String>,
    pub name: Option<Option<String>>,
    pub is_assigned: Option<bool>,
}

// ---------------------------------------------------------------------------
// number_pools
// ---------------------------------------------------------------------------

/// Read model for the `iiz.number_pools` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = number_pools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NumberPool {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source_id: Option<Uuid>,
    pub auto_manage: bool,
    pub target_accuracy: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new number pool.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = number_pools)]
pub struct NewNumberPool {
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source_id: Option<Uuid>,
    pub auto_manage: bool,
    pub target_accuracy: i32,
}

/// Update model for partial number pool updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = number_pools)]
pub struct UpdateNumberPool {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub source_id: Option<Option<Uuid>>,
    pub auto_manage: Option<bool>,
    pub target_accuracy: Option<i32>,
}

// ---------------------------------------------------------------------------
// number_pool_members
// ---------------------------------------------------------------------------

/// Read model for the `iiz.number_pool_members` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = number_pool_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NumberPoolMember {
    pub id: Uuid,
    pub account_id: Uuid,
    pub pool_id: Uuid,
    pub tracking_number_id: Uuid,
    pub status: String,
    pub call_count: i32,
    pub added_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new number pool member.
/// `id`, `call_count`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `call_count` is system-maintained. `added_at` is user-facing and included here.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = number_pool_members)]
pub struct NewNumberPoolMember {
    pub account_id: Uuid,
    pub pool_id: Uuid,
    pub tracking_number_id: Uuid,
    pub status: String,
    pub added_at: DateTime<Utc>,
}

/// Update model for partial number pool member updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `call_count` is system-maintained and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = number_pool_members)]
pub struct UpdateNumberPoolMember {
    pub status: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// caller_id_cnam
// ---------------------------------------------------------------------------

/// Read model for the `iiz.caller_id_cnam` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = caller_id_cnam)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallerIdCnam {
    pub id: Uuid,
    pub account_id: Uuid,
    pub number: String,
    pub tracking_number_id: Option<Uuid>,
    pub current_cnam: Option<String>,
    pub requested_cnam: Option<String>,
    pub status: String,
    pub last_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new caller ID CNAM entry.
/// `id`, `current_cnam`, `last_updated_at`, `created_at`, `updated_at`, and `deleted_at`
/// are set by database defaults. `current_cnam` and `last_updated_at` are system-updated
/// from CNAM lookup results.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = caller_id_cnam)]
pub struct NewCallerIdCnam {
    pub account_id: Uuid,
    pub number: String,
    pub tracking_number_id: Option<Uuid>,
    pub requested_cnam: Option<String>,
    pub status: String,
}

/// Update model for partial caller ID CNAM updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `current_cnam` and `last_updated_at` are system-updated and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = caller_id_cnam)]
pub struct UpdateCallerIdCnam {
    pub number: Option<String>,
    pub tracking_number_id: Option<Option<Uuid>>,
    pub requested_cnam: Option<Option<String>>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// port_requests
// ---------------------------------------------------------------------------

/// Read model for the `iiz.port_requests` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = port_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PortRequest {
    pub id: Uuid,
    pub account_id: Uuid,
    pub numbers_to_port: serde_json::Value,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub billing_address_line1: Option<String>,
    pub billing_address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub authorized_signature: Option<String>,
    pub status: ComplianceStatus,
    pub submitted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new port request.
/// `id`, `submitted_at`, `completed_at`, `rejection_reason`, `created_at`, `updated_at`,
/// and `deleted_at` are set by database defaults. `submitted_at`, `completed_at`, and
/// `rejection_reason` are system-managed workflow fields.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = port_requests)]
pub struct NewPortRequest {
    pub account_id: Uuid,
    pub numbers_to_port: serde_json::Value,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub billing_address_line1: Option<String>,
    pub billing_address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub authorized_signature: Option<String>,
    pub status: ComplianceStatus,
}

/// Update model for partial port request updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `submitted_at`, `completed_at`, and `rejection_reason` are system-managed workflow
/// fields and omitted here.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = port_requests)]
pub struct UpdatePortRequest {
    pub numbers_to_port: Option<serde_json::Value>,
    pub first_name: Option<Option<String>>,
    pub last_name: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub phone: Option<Option<String>>,
    pub billing_address_line1: Option<Option<String>>,
    pub billing_address_line2: Option<Option<String>>,
    pub city: Option<Option<String>>,
    pub state: Option<Option<String>>,
    pub zip: Option<Option<String>>,
    pub authorized_signature: Option<Option<String>>,
    pub status: Option<ComplianceStatus>,
}

// ---------------------------------------------------------------------------
// call_settings
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_settings` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallSetting {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub greeting_enabled: bool,
    pub whisper_enabled: bool,
    pub inbound_recording: bool,
    pub outbound_recording: bool,
    pub transcription_enabled: bool,
    pub caller_id_enabled: bool,
    pub enhanced_caller_id: bool,
    pub caller_id_override: bool,
    pub spam_filter_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new call settings profile.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_settings)]
pub struct NewCallSetting {
    pub account_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub greeting_enabled: bool,
    pub whisper_enabled: bool,
    pub inbound_recording: bool,
    pub outbound_recording: bool,
    pub transcription_enabled: bool,
    pub caller_id_enabled: bool,
    pub enhanced_caller_id: bool,
    pub caller_id_override: bool,
    pub spam_filter_enabled: bool,
}

/// Update model for partial call settings updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = call_settings)]
pub struct UpdateCallSetting {
    pub name: Option<String>,
    pub is_default: Option<bool>,
    pub greeting_enabled: Option<bool>,
    pub whisper_enabled: Option<bool>,
    pub inbound_recording: Option<bool>,
    pub outbound_recording: Option<bool>,
    pub transcription_enabled: Option<bool>,
    pub caller_id_enabled: Option<bool>,
    pub enhanced_caller_id: Option<bool>,
    pub caller_id_override: Option<bool>,
    pub spam_filter_enabled: Option<bool>,
}
