//! Diesel model structs for activity/event tables in the iiz schema.
//!
//! Most activity tables are append-only event logs (Read + Insert only).
//! Mutable tables (call_annotations, call_daily_summary, active_calls) have
//! all three struct variants (Read + Insert + Update).
//!
//! Several tables use composite primary keys for partitioning support.

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{
    ActiveCallStatus, AgentStatus, CallDirection, CallStatus, MonitorMode, SpeakerType, SummaryType,
};
use crate::iiz::schema::iiz::{
    active_calls, agent_state_log, api_log_entries, call_ai_summaries, call_annotations,
    call_daily_summary, call_flow_events, call_keyword_hits, call_records, call_tags,
    call_transcription_segments, call_visitor_sessions, monitoring_events,
};

// ---------------------------------------------------------------------------
// call_records (Composite PK: id, started_at) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_records` table (partitioned by started_at).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_records)]
#[diesel(primary_key(id, started_at))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: String,
    pub caller_phone: Option<String>,
    pub callee_phone: Option<String>,
    pub direction: CallDirection,
    pub status: CallStatus,
    pub source_id: Option<Uuid>,
    pub source_number_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_secs: i32,
    pub ring_duration_secs: i32,
    pub hold_duration_secs: i32,
    pub recording_url: Option<String>,
    pub has_audio: bool,
    pub is_first_time_caller: bool,
    pub location: Option<String>,
    pub automation_id: Option<Uuid>,
    pub source_name: Option<String>,
    pub agent_name: Option<String>,
    pub queue_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new call record.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_records)]
pub struct NewCallRecord {
    pub call_id: String,
    pub account_id: Uuid,
    pub caller_phone: Option<String>,
    pub callee_phone: Option<String>,
    pub direction: CallDirection,
    pub status: CallStatus,
    pub source_id: Option<Uuid>,
    pub source_number_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_secs: i32,
    pub ring_duration_secs: i32,
    pub hold_duration_secs: i32,
    pub recording_url: Option<String>,
    pub has_audio: bool,
    pub is_first_time_caller: bool,
    pub location: Option<String>,
    pub automation_id: Option<Uuid>,
    pub source_name: Option<String>,
    pub agent_name: Option<String>,
    pub queue_name: Option<String>,
}

// ---------------------------------------------------------------------------
// call_flow_events (Composite PK: id, occurred_at) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_flow_events` table (partitioned by occurred_at).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_flow_events)]
#[diesel(primary_key(id, occurred_at))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallFlowEvent {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub detail: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new call flow event.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_flow_events)]
pub struct NewCallFlowEvent {
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub detail: Option<String>,
}

// ---------------------------------------------------------------------------
// call_transcription_segments (Composite PK: id, created_at) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_transcription_segments` table (partitioned by created_at).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_transcription_segments)]
#[diesel(primary_key(id, created_at))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallTranscriptionSegment {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub segment_index: i32,
    pub timestamp_offset_secs: Option<f32>,
    pub speaker: SpeakerType,
    pub content: String,
    pub confidence: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new transcription segment.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_transcription_segments)]
pub struct NewCallTranscriptionSegment {
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub segment_index: i32,
    pub timestamp_offset_secs: Option<f32>,
    pub speaker: SpeakerType,
    pub content: String,
    pub confidence: Option<f32>,
}

// ---------------------------------------------------------------------------
// call_ai_summaries (Composite PK: id, generated_at) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_ai_summaries` table (partitioned by generated_at).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_ai_summaries)]
#[diesel(primary_key(id, generated_at))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallAiSummary {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub summary_type: SummaryType,
    pub content: String,
    pub model: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new AI summary.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_ai_summaries)]
pub struct NewCallAiSummary {
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub summary_type: SummaryType,
    pub content: String,
    pub model: Option<String>,
    pub generated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// call_annotations (PK: call_id) — READ + INSERT + UPDATE (mutable)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_annotations` table.
/// This is the mutable companion to `call_records` (1:1 relationship).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_annotations)]
#[diesel(primary_key(call_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallAnnotation {
    pub call_id: Uuid,
    pub account_id: Uuid,
    pub score: Option<i32>,
    pub converted: Option<bool>,
    pub outcome: Option<String>,
    pub reporting_tag: Option<String>,
    pub category: Option<String>,
    pub appointment_set: Option<bool>,
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new call annotation.
/// `updated_at` and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_annotations)]
pub struct NewCallAnnotation {
    pub call_id: Uuid,
    pub account_id: Uuid,
    pub score: Option<i32>,
    pub converted: Option<bool>,
    pub outcome: Option<String>,
    pub reporting_tag: Option<String>,
    pub category: Option<String>,
    pub appointment_set: Option<bool>,
    pub notes: Option<String>,
    pub updated_by_id: Option<Uuid>,
}

/// Update model for partial call annotation updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = call_annotations)]
pub struct UpdateCallAnnotation {
    pub score: Option<Option<i32>>,
    pub converted: Option<Option<bool>>,
    pub outcome: Option<Option<String>>,
    pub reporting_tag: Option<Option<String>>,
    pub category: Option<Option<String>>,
    pub appointment_set: Option<Option<bool>>,
    pub notes: Option<Option<String>>,
    pub updated_by_id: Option<Option<Uuid>>,
}

// ---------------------------------------------------------------------------
// call_tags (PK: id) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_tags` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallTag {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub tag_id: Uuid,
    pub applied_at: DateTime<Utc>,
    pub applied_by_type: String,
    pub applied_by_id: Option<Uuid>,
    pub trigger_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new call tag.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_tags)]
pub struct NewCallTag {
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub tag_id: Uuid,
    pub applied_at: DateTime<Utc>,
    pub applied_by_type: String,
    pub applied_by_id: Option<Uuid>,
    pub trigger_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// call_keyword_hits (PK: id) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_keyword_hits` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_keyword_hits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallKeywordHit {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub keyword_id: Option<Uuid>,
    pub timestamp_offset_secs: Option<f32>,
    pub speaker: Option<SpeakerType>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new keyword hit.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_keyword_hits)]
pub struct NewCallKeywordHit {
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub keyword_id: Option<Uuid>,
    pub timestamp_offset_secs: Option<f32>,
    pub speaker: Option<SpeakerType>,
}

// ---------------------------------------------------------------------------
// call_visitor_sessions (PK: id) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_visitor_sessions` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_visitor_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallVisitorSession {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub ip_address: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub referrer: Option<String>,
    pub landing_page: Option<String>,
    pub keywords: Option<String>,
    pub campaign: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub visit_duration_secs: Option<i32>,
    pub pages_viewed: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new visitor session.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_visitor_sessions)]
pub struct NewCallVisitorSession {
    pub account_id: Uuid,
    pub call_id: Uuid,
    pub ip_address: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub referrer: Option<String>,
    pub landing_page: Option<String>,
    pub keywords: Option<String>,
    pub campaign: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub visit_duration_secs: Option<i32>,
    pub pages_viewed: Option<i32>,
}

// ---------------------------------------------------------------------------
// call_daily_summary (PK: id) — READ + INSERT + UPDATE (recomputed aggregation)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.call_daily_summary` table.
/// This is a recomputed aggregation table, so it supports updates.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = call_daily_summary)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallDailySummary {
    pub id: Uuid,
    pub account_id: Uuid,
    pub summary_date: NaiveDate,
    pub source_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub total_calls: i32,
    pub answered_calls: i32,
    pub missed_calls: i32,
    pub voicemail_calls: i32,
    pub total_duration_secs: i32,
    pub total_ring_duration_secs: i32,
    pub total_hold_duration_secs: i32,
    pub avg_duration_secs: Option<BigDecimal>,
    pub avg_ring_duration_secs: Option<BigDecimal>,
    pub unique_callers: i32,
    pub first_time_callers: i32,
    pub repeat_callers: i32,
    pub converted_calls: i32,
    pub appointments_set: i32,
    pub computed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new daily summary row.
/// `id`, `created_at`, `updated_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = call_daily_summary)]
pub struct NewCallDailySummary {
    pub account_id: Uuid,
    pub summary_date: NaiveDate,
    pub source_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub total_calls: i32,
    pub answered_calls: i32,
    pub missed_calls: i32,
    pub voicemail_calls: i32,
    pub total_duration_secs: i32,
    pub total_ring_duration_secs: i32,
    pub total_hold_duration_secs: i32,
    pub avg_duration_secs: Option<BigDecimal>,
    pub avg_ring_duration_secs: Option<BigDecimal>,
    pub unique_callers: i32,
    pub first_time_callers: i32,
    pub repeat_callers: i32,
    pub converted_calls: i32,
    pub appointments_set: i32,
    pub computed_at: DateTime<Utc>,
}

/// Update model for recomputing a daily summary.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = call_daily_summary)]
pub struct UpdateCallDailySummary {
    pub total_calls: Option<i32>,
    pub answered_calls: Option<i32>,
    pub missed_calls: Option<i32>,
    pub voicemail_calls: Option<i32>,
    pub total_duration_secs: Option<i32>,
    pub total_ring_duration_secs: Option<i32>,
    pub total_hold_duration_secs: Option<i32>,
    pub avg_duration_secs: Option<Option<BigDecimal>>,
    pub avg_ring_duration_secs: Option<Option<BigDecimal>>,
    pub unique_callers: Option<i32>,
    pub first_time_callers: Option<i32>,
    pub repeat_callers: Option<i32>,
    pub converted_calls: Option<i32>,
    pub appointments_set: Option<i32>,
    pub computed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// active_calls (PK: id) — READ + INSERT + UPDATE (UNLOGGED, ephemeral)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.active_calls` table (UNLOGGED).
/// This is an ephemeral table for tracking live calls in progress.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = active_calls)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ActiveCall {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: String,
    pub caller_name: Option<String>,
    pub caller_number: Option<String>,
    pub callee_number: Option<String>,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub tracking_number_id: Option<Uuid>,
    pub direction: CallDirection,
    pub status: ActiveCallStatus,
    pub started_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub is_monitored: bool,
    pub monitor_mode: Option<MonitorMode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new active call.
/// `id`, `created_at`, `updated_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = active_calls)]
pub struct NewActiveCall {
    pub account_id: Uuid,
    pub call_id: String,
    pub caller_name: Option<String>,
    pub caller_number: Option<String>,
    pub callee_number: Option<String>,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub tracking_number_id: Option<Uuid>,
    pub direction: CallDirection,
    pub status: ActiveCallStatus,
    pub started_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub is_monitored: bool,
    pub monitor_mode: Option<MonitorMode>,
}

/// Update model for changing active call state.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = active_calls)]
pub struct UpdateActiveCall {
    pub caller_name: Option<Option<String>>,
    pub caller_number: Option<Option<String>>,
    pub callee_number: Option<Option<String>>,
    pub agent_id: Option<Option<Uuid>>,
    pub queue_id: Option<Option<Uuid>>,
    pub source_id: Option<Option<Uuid>>,
    pub tracking_number_id: Option<Option<Uuid>>,
    pub direction: Option<CallDirection>,
    pub status: Option<ActiveCallStatus>,
    pub answered_at: Option<Option<DateTime<Utc>>>,
    pub is_monitored: Option<bool>,
    pub monitor_mode: Option<Option<MonitorMode>>,
}

// ---------------------------------------------------------------------------
// monitoring_events (PK: id) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.monitoring_events` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = monitoring_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitoringEvent {
    pub id: Uuid,
    pub account_id: Uuid,
    pub session_id: Option<String>,
    pub call_id: Option<Uuid>,
    pub monitor_user_id: Uuid,
    pub monitored_agent_id: Option<Uuid>,
    pub event_type: String,
    pub monitor_mode: MonitorMode,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new monitoring event.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = monitoring_events)]
pub struct NewMonitoringEvent {
    pub account_id: Uuid,
    pub session_id: Option<String>,
    pub call_id: Option<Uuid>,
    pub monitor_user_id: Uuid,
    pub monitored_agent_id: Option<Uuid>,
    pub event_type: String,
    pub monitor_mode: MonitorMode,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<i32>,
}

// ---------------------------------------------------------------------------
// agent_state_log (Composite PK: id, changed_at) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.agent_state_log` table (partitioned by changed_at).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = agent_state_log)]
#[diesel(primary_key(id, changed_at))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentStateLogEntry {
    pub id: Uuid,
    pub account_id: Uuid,
    pub agent_id: Uuid,
    pub status: AgentStatus,
    pub changed_at: DateTime<Utc>,
    pub duration_secs: Option<i32>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new agent state log entry.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = agent_state_log)]
pub struct NewAgentStateLogEntry {
    pub account_id: Uuid,
    pub agent_id: Uuid,
    pub status: AgentStatus,
    pub changed_at: DateTime<Utc>,
    pub duration_secs: Option<i32>,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// api_log_entries (Composite PK: id, timestamp) — READ + INSERT only
// ---------------------------------------------------------------------------

/// Read model for the `iiz.api_log_entries` table (partitioned by timestamp).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = api_log_entries)]
#[diesel(primary_key(id, timestamp))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApiLogEntry {
    pub id: Uuid,
    pub account_id: Uuid,
    pub source: Option<String>,
    pub method: String,
    pub endpoint: String,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<serde_json::Value>,
    pub response_code: Option<i32>,
    pub response_body: Option<serde_json::Value>,
    pub response_size_bytes: Option<i32>,
    pub duration_ms: Option<i32>,
    pub activity_description: Option<String>,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new API log entry.
/// `id`, `created_at`, `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = api_log_entries)]
pub struct NewApiLogEntry {
    pub account_id: Uuid,
    pub source: Option<String>,
    pub method: String,
    pub endpoint: String,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<serde_json::Value>,
    pub response_code: Option<i32>,
    pub response_body: Option<serde_json::Value>,
    pub response_size_bytes: Option<i32>,
    pub duration_ms: Option<i32>,
    pub activity_description: Option<String>,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}
