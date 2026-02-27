//! Diesel model structs for the `bulk_messages`, `lead_reactor_configs`,
//! `lead_reactor_actions`, `smart_dialer_configs`, `form_reactor_entries`,
//! `reminders`, `keyword_spotting_configs`, `keyword_spotting_keywords`,
//! `keyword_spotting_numbers`, and `chat_widgets` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, NaiveTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::schema::iiz::{
    bulk_messages, chat_widgets, form_reactor_entries, keyword_spotting_configs,
    keyword_spotting_keywords, keyword_spotting_numbers, lead_reactor_actions,
    lead_reactor_configs, reminders, smart_dialer_configs,
};

// ---------------------------------------------------------------------------
// bulk_messages
// ---------------------------------------------------------------------------

/// Read model for the `iiz.bulk_messages` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = bulk_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BulkMessage {
    pub id: Uuid,
    pub account_id: Uuid,
    pub label: Option<String>,
    pub sender_number_id: Option<Uuid>,
    pub sender_phone: Option<String>,
    pub message_body: String,
    pub msg_type: String,
    pub contact_list_id: Option<Uuid>,
    pub recipient_count: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub failed_count: i32,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new bulk message.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `sent_count`, `delivered_count`, `failed_count` are system-maintained counters.
/// `started_at` and `completed_at` are system-maintained timestamps.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = bulk_messages)]
pub struct NewBulkMessage {
    pub account_id: Uuid,
    pub label: Option<String>,
    pub sender_number_id: Option<Uuid>,
    pub sender_phone: Option<String>,
    pub message_body: String,
    pub msg_type: String,
    pub contact_list_id: Option<Uuid>,
    pub recipient_count: i32,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// Update model for partial bulk message updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `sent_count`, `delivered_count`, `failed_count` are system-maintained counters.
/// `started_at` and `completed_at` are system-maintained timestamps.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = bulk_messages)]
pub struct UpdateBulkMessage {
    pub label: Option<Option<String>>,
    pub sender_number_id: Option<Option<Uuid>>,
    pub sender_phone: Option<Option<String>>,
    pub message_body: Option<String>,
    pub msg_type: Option<String>,
    pub contact_list_id: Option<Option<Uuid>>,
    pub recipient_count: Option<i32>,
    pub status: Option<String>,
    pub scheduled_at: Option<Option<DateTime<Utc>>>,
}

// ---------------------------------------------------------------------------
// lead_reactor_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.lead_reactor_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = lead_reactor_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LeadReactorConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub trigger_event: String,
    pub delay_minutes: i32,
    pub is_active: bool,
    pub working_hours_only: bool,
    pub max_retries: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new lead reactor config.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = lead_reactor_configs)]
pub struct NewLeadReactorConfig {
    pub account_id: Uuid,
    pub name: String,
    pub trigger_event: String,
    pub delay_minutes: i32,
    pub is_active: bool,
    pub working_hours_only: bool,
    pub max_retries: i32,
}

/// Update model for partial lead reactor config updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = lead_reactor_configs)]
pub struct UpdateLeadReactorConfig {
    pub name: Option<String>,
    pub trigger_event: Option<String>,
    pub delay_minutes: Option<i32>,
    pub is_active: Option<bool>,
    pub working_hours_only: Option<bool>,
    pub max_retries: Option<i32>,
}

// ---------------------------------------------------------------------------
// lead_reactor_actions
// ---------------------------------------------------------------------------

/// Read model for the `iiz.lead_reactor_actions` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = lead_reactor_actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LeadReactorAction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub config_id: Uuid,
    pub sort_order: i32,
    pub action_type: String,
    pub template_content: Option<String>,
    pub action_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new lead reactor action.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = lead_reactor_actions)]
pub struct NewLeadReactorAction {
    pub account_id: Uuid,
    pub config_id: Uuid,
    pub sort_order: i32,
    pub action_type: String,
    pub template_content: Option<String>,
    pub action_config: Option<serde_json::Value>,
}

/// Update model for partial lead reactor action updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = lead_reactor_actions)]
pub struct UpdateLeadReactorAction {
    pub sort_order: Option<i32>,
    pub action_type: Option<String>,
    pub template_content: Option<Option<String>>,
    pub action_config: Option<Option<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// smart_dialer_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.smart_dialer_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = smart_dialer_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SmartDialerConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub mode: String,
    pub max_concurrent: i32,
    pub ring_timeout_secs: i32,
    pub retry_attempts: i32,
    pub retry_interval_minutes: i32,
    pub outbound_number: Option<String>,
    pub outbound_cnam: Option<String>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub timezone: Option<String>,
    pub active_days: i32,
    pub contact_list_id: Option<Uuid>,
    pub agent_script_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new smart dialer config.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = smart_dialer_configs)]
pub struct NewSmartDialerConfig {
    pub account_id: Uuid,
    pub name: String,
    pub mode: String,
    pub max_concurrent: i32,
    pub ring_timeout_secs: i32,
    pub retry_attempts: i32,
    pub retry_interval_minutes: i32,
    pub outbound_number: Option<String>,
    pub outbound_cnam: Option<String>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub timezone: Option<String>,
    pub active_days: i32,
    pub contact_list_id: Option<Uuid>,
    pub agent_script_id: Option<Uuid>,
    pub is_active: bool,
}

/// Update model for partial smart dialer config updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = smart_dialer_configs)]
pub struct UpdateSmartDialerConfig {
    pub name: Option<String>,
    pub mode: Option<String>,
    pub max_concurrent: Option<i32>,
    pub ring_timeout_secs: Option<i32>,
    pub retry_attempts: Option<i32>,
    pub retry_interval_minutes: Option<i32>,
    pub outbound_number: Option<Option<String>>,
    pub outbound_cnam: Option<Option<String>>,
    pub start_time: Option<Option<NaiveTime>>,
    pub end_time: Option<Option<NaiveTime>>,
    pub timezone: Option<Option<String>>,
    pub active_days: Option<i32>,
    pub contact_list_id: Option<Option<Uuid>>,
    pub agent_script_id: Option<Option<Uuid>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// form_reactor_entries
// ---------------------------------------------------------------------------

/// Read model for the `iiz.form_reactor_entries` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = form_reactor_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FormReactorEntry {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub form_fields: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub call_count: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new form reactor entry.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `call_count` is a system-maintained counter.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = form_reactor_entries)]
pub struct NewFormReactorEntry {
    pub account_id: Uuid,
    pub name: String,
    pub form_fields: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub status: String,
}

/// Update model for partial form reactor entry updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `call_count` is a system-maintained counter.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = form_reactor_entries)]
pub struct UpdateFormReactorEntry {
    pub name: Option<String>,
    pub form_fields: Option<Option<String>>,
    pub tracking_number_id: Option<Option<Uuid>>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// reminders
// ---------------------------------------------------------------------------

/// Read model for the `iiz.reminders` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = reminders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Reminder {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub remind_at: Option<DateTime<Utc>>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
    pub contact_source: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_list_id: Option<Uuid>,
    pub delivery_method: String,
    pub recipient: Option<String>,
    pub message: Option<String>,
    pub status: String,
    pub call_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new reminder.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = reminders)]
pub struct NewReminder {
    pub account_id: Uuid,
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub remind_at: Option<DateTime<Utc>>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
    pub contact_source: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_list_id: Option<Uuid>,
    pub delivery_method: String,
    pub recipient: Option<String>,
    pub message: Option<String>,
    pub status: String,
    pub call_id: Option<Uuid>,
}

/// Update model for partial reminder updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = reminders)]
pub struct UpdateReminder {
    pub name: Option<Option<String>>,
    pub timezone: Option<Option<String>>,
    pub remind_at: Option<Option<DateTime<Utc>>>,
    pub is_recurring: Option<bool>,
    pub recurrence_rule: Option<Option<String>>,
    pub contact_source: Option<Option<String>>,
    pub contact_phone: Option<Option<String>>,
    pub contact_list_id: Option<Option<Uuid>>,
    pub delivery_method: Option<String>,
    pub recipient: Option<Option<String>>,
    pub message: Option<Option<String>>,
    pub status: Option<String>,
    pub call_id: Option<Option<Uuid>>,
}

// ---------------------------------------------------------------------------
// keyword_spotting_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.keyword_spotting_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = keyword_spotting_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KeywordSpottingConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub sensitivity: String,
    pub apply_to_all_numbers: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new keyword spotting config.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = keyword_spotting_configs)]
pub struct NewKeywordSpottingConfig {
    pub account_id: Uuid,
    pub name: String,
    pub sensitivity: String,
    pub apply_to_all_numbers: bool,
    pub is_active: bool,
}

/// Update model for partial keyword spotting config updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = keyword_spotting_configs)]
pub struct UpdateKeywordSpottingConfig {
    pub name: Option<String>,
    pub sensitivity: Option<String>,
    pub apply_to_all_numbers: Option<bool>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// keyword_spotting_keywords
// ---------------------------------------------------------------------------

/// Read model for the `iiz.keyword_spotting_keywords` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = keyword_spotting_keywords)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KeywordSpottingKeyword {
    pub id: Uuid,
    pub account_id: Uuid,
    pub config_id: Uuid,
    pub keyword: String,
    pub category: String,
    pub score_weight: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new keyword spotting keyword.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = keyword_spotting_keywords)]
pub struct NewKeywordSpottingKeyword {
    pub account_id: Uuid,
    pub config_id: Uuid,
    pub keyword: String,
    pub category: String,
    pub score_weight: f32,
}

/// Update model for partial keyword spotting keyword updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = keyword_spotting_keywords)]
pub struct UpdateKeywordSpottingKeyword {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub score_weight: Option<f32>,
}

// ---------------------------------------------------------------------------
// keyword_spotting_numbers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.keyword_spotting_numbers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = keyword_spotting_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KeywordSpottingNumber {
    pub id: Uuid,
    pub account_id: Uuid,
    pub config_id: Uuid,
    pub tracking_number_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new keyword spotting number assignment.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = keyword_spotting_numbers)]
pub struct NewKeywordSpottingNumber {
    pub account_id: Uuid,
    pub config_id: Uuid,
    pub tracking_number_id: Uuid,
}

/// Update model for partial keyword spotting number updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = keyword_spotting_numbers)]
pub struct UpdateKeywordSpottingNumber {
    pub tracking_number_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// chat_widgets
// ---------------------------------------------------------------------------

/// Read model for the `iiz.chat_widgets` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = chat_widgets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatWidget {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub website_url: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub routing_type: Option<String>,
    pub queue_id: Option<Uuid>,
    pub agent_count: i32,
    pub custom_fields_count: i32,
    pub status: String,
    pub config_json: Option<serde_json::Value>,
    pub chat_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new chat widget.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `agent_count`, `custom_fields_count`, and `chat_count` are system-maintained counters.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = chat_widgets)]
pub struct NewChatWidget {
    pub account_id: Uuid,
    pub name: String,
    pub website_url: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub routing_type: Option<String>,
    pub queue_id: Option<Uuid>,
    pub status: String,
    pub config_json: Option<serde_json::Value>,
}

/// Update model for partial chat widget updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `agent_count`, `custom_fields_count`, and `chat_count` are system-maintained counters.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = chat_widgets)]
pub struct UpdateChatWidget {
    pub name: Option<String>,
    pub website_url: Option<Option<String>>,
    pub tracking_number_id: Option<Option<Uuid>>,
    pub routing_type: Option<Option<String>>,
    pub queue_id: Option<Option<Uuid>>,
    pub status: Option<String>,
    pub config_json: Option<Option<serde_json::Value>>,
}
