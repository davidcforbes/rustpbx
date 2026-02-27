//! Diesel model structs for the `queues`, `queue_agents`, `routing_tables`,
//! `routing_table_routes`, `schedules`, `schedule_holidays`, `voice_menus`,
//! `voice_menu_options`, `voicemail_boxes`, `voicemail_messages`, `geo_routers`,
//! `geo_router_rules`, `smart_routers`, `smart_router_rules`, `agent_scripts`,
//! and `scoring_configs` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{GreetingType, QueueStrategy};
use crate::iiz::schema::iiz::{
    agent_scripts, geo_router_rules, geo_routers, queue_agents, queues, routing_table_routes,
    routing_tables, schedule_holidays, schedules, scoring_configs, smart_router_rules,
    smart_routers, voice_menu_options, voice_menus, voicemail_boxes, voicemail_messages,
};

// ---------------------------------------------------------------------------
// queues
// ---------------------------------------------------------------------------

/// Read model for the `iiz.queues` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = queues)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Queue {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub strategy: QueueStrategy,
    pub schedule_id: Option<Uuid>,
    pub repeat_callers: bool,
    pub caller_id_display: Option<String>,
    pub max_wait_secs: i32,
    pub no_answer_destination_type: Option<String>,
    pub no_answer_destination_id: Option<Uuid>,
    pub no_answer_destination_number: Option<String>,
    pub moh_audio_url: Option<String>,
    pub wrap_up_secs: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new queue.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = queues)]
pub struct NewQueue {
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub strategy: QueueStrategy,
    pub schedule_id: Option<Uuid>,
    pub repeat_callers: bool,
    pub caller_id_display: Option<String>,
    pub max_wait_secs: i32,
    pub no_answer_destination_type: Option<String>,
    pub no_answer_destination_id: Option<Uuid>,
    pub no_answer_destination_number: Option<String>,
    pub moh_audio_url: Option<String>,
    pub wrap_up_secs: i32,
    pub is_active: bool,
}

/// Update model for partial queue updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = queues)]
pub struct UpdateQueue {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub strategy: Option<QueueStrategy>,
    pub schedule_id: Option<Option<Uuid>>,
    pub repeat_callers: Option<bool>,
    pub caller_id_display: Option<Option<String>>,
    pub max_wait_secs: Option<i32>,
    pub no_answer_destination_type: Option<Option<String>>,
    pub no_answer_destination_id: Option<Option<Uuid>>,
    pub no_answer_destination_number: Option<Option<String>>,
    pub moh_audio_url: Option<Option<String>>,
    pub wrap_up_secs: Option<i32>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// queue_agents
// ---------------------------------------------------------------------------

/// Read model for the `iiz.queue_agents` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = queue_agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QueueAgent {
    pub id: Uuid,
    pub account_id: Uuid,
    pub queue_id: Uuid,
    pub agent_id: Uuid,
    pub priority: i32,
    pub is_active: bool,
    pub added_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new queue agent assignment.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `added_at` is user-facing and included here.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = queue_agents)]
pub struct NewQueueAgent {
    pub account_id: Uuid,
    pub queue_id: Uuid,
    pub agent_id: Uuid,
    pub priority: i32,
    pub is_active: bool,
    pub added_at: DateTime<Utc>,
}

/// Update model for partial queue agent updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = queue_agents)]
pub struct UpdateQueueAgent {
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
    pub added_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// routing_tables
// ---------------------------------------------------------------------------

/// Read model for the `iiz.routing_tables` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = routing_tables)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RoutingTable {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new routing table.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = routing_tables)]
pub struct NewRoutingTable {
    pub account_id: Uuid,
    pub name: String,
    pub is_active: bool,
}

/// Update model for partial routing table updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = routing_tables)]
pub struct UpdateRoutingTable {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// routing_table_routes
// ---------------------------------------------------------------------------

/// Read model for the `iiz.routing_table_routes` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = routing_table_routes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RoutingTableRoute {
    pub id: Uuid,
    pub account_id: Uuid,
    pub table_id: Uuid,
    pub priority: i32,
    pub match_pattern: Option<String>,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new routing table route.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = routing_table_routes)]
pub struct NewRoutingTableRoute {
    pub account_id: Uuid,
    pub table_id: Uuid,
    pub priority: i32,
    pub match_pattern: Option<String>,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
    pub weight: i32,
}

/// Update model for partial routing table route updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = routing_table_routes)]
pub struct UpdateRoutingTableRoute {
    pub priority: Option<i32>,
    pub match_pattern: Option<Option<String>>,
    pub destination_type: Option<Option<String>>,
    pub destination_id: Option<Option<Uuid>>,
    pub destination_number: Option<Option<String>>,
    pub weight: Option<i32>,
}

// ---------------------------------------------------------------------------
// schedules
// ---------------------------------------------------------------------------

/// Read model for the `iiz.schedules` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = schedules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Schedule {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub timezone: String,
    pub monday_open: Option<NaiveTime>,
    pub monday_close: Option<NaiveTime>,
    pub tuesday_open: Option<NaiveTime>,
    pub tuesday_close: Option<NaiveTime>,
    pub wednesday_open: Option<NaiveTime>,
    pub wednesday_close: Option<NaiveTime>,
    pub thursday_open: Option<NaiveTime>,
    pub thursday_close: Option<NaiveTime>,
    pub friday_open: Option<NaiveTime>,
    pub friday_close: Option<NaiveTime>,
    pub saturday_open: Option<NaiveTime>,
    pub saturday_close: Option<NaiveTime>,
    pub sunday_open: Option<NaiveTime>,
    pub sunday_close: Option<NaiveTime>,
    pub closed_destination_type: Option<String>,
    pub closed_destination_id: Option<Uuid>,
    pub closed_destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new schedule.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = schedules)]
pub struct NewSchedule {
    pub account_id: Uuid,
    pub name: String,
    pub timezone: String,
    pub monday_open: Option<NaiveTime>,
    pub monday_close: Option<NaiveTime>,
    pub tuesday_open: Option<NaiveTime>,
    pub tuesday_close: Option<NaiveTime>,
    pub wednesday_open: Option<NaiveTime>,
    pub wednesday_close: Option<NaiveTime>,
    pub thursday_open: Option<NaiveTime>,
    pub thursday_close: Option<NaiveTime>,
    pub friday_open: Option<NaiveTime>,
    pub friday_close: Option<NaiveTime>,
    pub saturday_open: Option<NaiveTime>,
    pub saturday_close: Option<NaiveTime>,
    pub sunday_open: Option<NaiveTime>,
    pub sunday_close: Option<NaiveTime>,
    pub closed_destination_type: Option<String>,
    pub closed_destination_id: Option<Uuid>,
    pub closed_destination_number: Option<String>,
}

/// Update model for partial schedule updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = schedules)]
pub struct UpdateSchedule {
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub monday_open: Option<Option<NaiveTime>>,
    pub monday_close: Option<Option<NaiveTime>>,
    pub tuesday_open: Option<Option<NaiveTime>>,
    pub tuesday_close: Option<Option<NaiveTime>>,
    pub wednesday_open: Option<Option<NaiveTime>>,
    pub wednesday_close: Option<Option<NaiveTime>>,
    pub thursday_open: Option<Option<NaiveTime>>,
    pub thursday_close: Option<Option<NaiveTime>>,
    pub friday_open: Option<Option<NaiveTime>>,
    pub friday_close: Option<Option<NaiveTime>>,
    pub saturday_open: Option<Option<NaiveTime>>,
    pub saturday_close: Option<Option<NaiveTime>>,
    pub sunday_open: Option<Option<NaiveTime>>,
    pub sunday_close: Option<Option<NaiveTime>>,
    pub closed_destination_type: Option<Option<String>>,
    pub closed_destination_id: Option<Option<Uuid>>,
    pub closed_destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// schedule_holidays
// ---------------------------------------------------------------------------

/// Read model for the `iiz.schedule_holidays` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = schedule_holidays)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScheduleHoliday {
    pub id: Uuid,
    pub account_id: Uuid,
    pub schedule_id: Uuid,
    pub date: NaiveDate,
    pub name: String,
    pub is_closed: bool,
    pub custom_open: Option<NaiveTime>,
    pub custom_close: Option<NaiveTime>,
    pub override_destination_type: Option<String>,
    pub override_destination_id: Option<Uuid>,
    pub override_destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new schedule holiday.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = schedule_holidays)]
pub struct NewScheduleHoliday {
    pub account_id: Uuid,
    pub schedule_id: Uuid,
    pub date: NaiveDate,
    pub name: String,
    pub is_closed: bool,
    pub custom_open: Option<NaiveTime>,
    pub custom_close: Option<NaiveTime>,
    pub override_destination_type: Option<String>,
    pub override_destination_id: Option<Uuid>,
    pub override_destination_number: Option<String>,
}

/// Update model for partial schedule holiday updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = schedule_holidays)]
pub struct UpdateScheduleHoliday {
    pub date: Option<NaiveDate>,
    pub name: Option<String>,
    pub is_closed: Option<bool>,
    pub custom_open: Option<Option<NaiveTime>>,
    pub custom_close: Option<Option<NaiveTime>>,
    pub override_destination_type: Option<Option<String>>,
    pub override_destination_id: Option<Option<Uuid>>,
    pub override_destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// voice_menus
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voice_menus` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voice_menus)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoiceMenu {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub greeting_type: GreetingType,
    pub greeting_audio_url: Option<String>,
    pub greeting_text: Option<String>,
    pub speech_recognition: bool,
    pub speech_language: Option<String>,
    pub timeout_secs: i32,
    pub max_retries: i32,
    pub no_input_destination_type: Option<String>,
    pub no_input_destination_id: Option<Uuid>,
    pub no_input_destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new voice menu (IVR).
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voice_menus)]
pub struct NewVoiceMenu {
    pub account_id: Uuid,
    pub name: String,
    pub greeting_type: GreetingType,
    pub greeting_audio_url: Option<String>,
    pub greeting_text: Option<String>,
    pub speech_recognition: bool,
    pub speech_language: Option<String>,
    pub timeout_secs: i32,
    pub max_retries: i32,
    pub no_input_destination_type: Option<String>,
    pub no_input_destination_id: Option<Uuid>,
    pub no_input_destination_number: Option<String>,
}

/// Update model for partial voice menu updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = voice_menus)]
pub struct UpdateVoiceMenu {
    pub name: Option<String>,
    pub greeting_type: Option<GreetingType>,
    pub greeting_audio_url: Option<Option<String>>,
    pub greeting_text: Option<Option<String>>,
    pub speech_recognition: Option<bool>,
    pub speech_language: Option<Option<String>>,
    pub timeout_secs: Option<i32>,
    pub max_retries: Option<i32>,
    pub no_input_destination_type: Option<Option<String>>,
    pub no_input_destination_id: Option<Option<Uuid>>,
    pub no_input_destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// voice_menu_options
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voice_menu_options` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voice_menu_options)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoiceMenuOption {
    pub id: Uuid,
    pub account_id: Uuid,
    pub menu_id: Uuid,
    pub dtmf_digit: String,
    pub description: Option<String>,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new voice menu option.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voice_menu_options)]
pub struct NewVoiceMenuOption {
    pub account_id: Uuid,
    pub menu_id: Uuid,
    pub dtmf_digit: String,
    pub description: Option<String>,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
}

/// Update model for partial voice menu option updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = voice_menu_options)]
pub struct UpdateVoiceMenuOption {
    pub dtmf_digit: Option<String>,
    pub description: Option<Option<String>>,
    pub destination_type: Option<Option<String>>,
    pub destination_id: Option<Option<Uuid>>,
    pub destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// voicemail_boxes
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voicemail_boxes` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voicemail_boxes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoicemailBox {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub max_message_length_secs: i32,
    pub greeting_type: GreetingType,
    pub greeting_audio_url: Option<String>,
    pub transcription_enabled: bool,
    pub email_notification_enabled: bool,
    pub notification_email: Option<String>,
    pub max_messages: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new voicemail box.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voicemail_boxes)]
pub struct NewVoicemailBox {
    pub account_id: Uuid,
    pub name: String,
    pub max_message_length_secs: i32,
    pub greeting_type: GreetingType,
    pub greeting_audio_url: Option<String>,
    pub transcription_enabled: bool,
    pub email_notification_enabled: bool,
    pub notification_email: Option<String>,
    pub max_messages: i32,
}

/// Update model for partial voicemail box updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = voicemail_boxes)]
pub struct UpdateVoicemailBox {
    pub name: Option<String>,
    pub max_message_length_secs: Option<i32>,
    pub greeting_type: Option<GreetingType>,
    pub greeting_audio_url: Option<Option<String>>,
    pub transcription_enabled: Option<bool>,
    pub email_notification_enabled: Option<bool>,
    pub notification_email: Option<Option<String>>,
    pub max_messages: Option<i32>,
}

// ---------------------------------------------------------------------------
// voicemail_messages
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voicemail_messages` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voicemail_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoicemailMessage {
    pub id: Uuid,
    pub account_id: Uuid,
    pub mailbox_id: Uuid,
    pub call_id: Option<Uuid>,
    pub caller_number: Option<String>,
    pub caller_name: Option<String>,
    pub duration_secs: i32,
    pub audio_url: Option<String>,
    pub transcription: Option<String>,
    pub is_read: bool,
    pub recorded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new voicemail message.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// Most fields are system-populated during call recording; only `account_id` and
/// `mailbox_id` are required at insert time.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voicemail_messages)]
pub struct NewVoicemailMessage {
    pub account_id: Uuid,
    pub mailbox_id: Uuid,
    pub call_id: Option<Uuid>,
    pub caller_number: Option<String>,
    pub caller_name: Option<String>,
    pub duration_secs: i32,
    pub audio_url: Option<String>,
    pub transcription: Option<String>,
    pub is_read: bool,
    pub recorded_at: DateTime<Utc>,
}

/// Update model for partial voicemail message updates.
/// Only `is_read` is user-editable (mark as read/unread).
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = voicemail_messages)]
pub struct UpdateVoicemailMessage {
    pub is_read: Option<bool>,
}

// ---------------------------------------------------------------------------
// geo_routers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.geo_routers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = geo_routers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GeoRouter {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub default_destination_type: Option<String>,
    pub default_destination_id: Option<Uuid>,
    pub default_destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new geo router.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = geo_routers)]
pub struct NewGeoRouter {
    pub account_id: Uuid,
    pub name: String,
    pub default_destination_type: Option<String>,
    pub default_destination_id: Option<Uuid>,
    pub default_destination_number: Option<String>,
}

/// Update model for partial geo router updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = geo_routers)]
pub struct UpdateGeoRouter {
    pub name: Option<String>,
    pub default_destination_type: Option<Option<String>>,
    pub default_destination_id: Option<Option<Uuid>>,
    pub default_destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// geo_router_rules
// ---------------------------------------------------------------------------

/// Read model for the `iiz.geo_router_rules` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = geo_router_rules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GeoRouterRule {
    pub id: Uuid,
    pub account_id: Uuid,
    pub router_id: Uuid,
    pub region: String,
    pub region_type: String,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new geo router rule.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = geo_router_rules)]
pub struct NewGeoRouterRule {
    pub account_id: Uuid,
    pub router_id: Uuid,
    pub region: String,
    pub region_type: String,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
}

/// Update model for partial geo router rule updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = geo_router_rules)]
pub struct UpdateGeoRouterRule {
    pub region: Option<String>,
    pub region_type: Option<String>,
    pub destination_type: Option<Option<String>>,
    pub destination_id: Option<Option<Uuid>>,
    pub destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// smart_routers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.smart_routers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = smart_routers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SmartRouter {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub fallback_destination_type: Option<String>,
    pub fallback_destination_id: Option<Uuid>,
    pub fallback_destination_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new smart router.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = smart_routers)]
pub struct NewSmartRouter {
    pub account_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub fallback_destination_type: Option<String>,
    pub fallback_destination_id: Option<Uuid>,
    pub fallback_destination_number: Option<String>,
    pub is_active: bool,
}

/// Update model for partial smart router updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = smart_routers)]
pub struct UpdateSmartRouter {
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub fallback_destination_type: Option<Option<String>>,
    pub fallback_destination_id: Option<Option<Uuid>>,
    pub fallback_destination_number: Option<Option<String>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// smart_router_rules
// ---------------------------------------------------------------------------

/// Read model for the `iiz.smart_router_rules` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = smart_router_rules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SmartRouterRule {
    pub id: Uuid,
    pub account_id: Uuid,
    pub router_id: Uuid,
    pub sort_order: i32,
    pub condition_field: String,
    pub condition_operator: String,
    pub condition_value: String,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new smart router rule.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = smart_router_rules)]
pub struct NewSmartRouterRule {
    pub account_id: Uuid,
    pub router_id: Uuid,
    pub sort_order: i32,
    pub condition_field: String,
    pub condition_operator: String,
    pub condition_value: String,
    pub destination_type: Option<String>,
    pub destination_id: Option<Uuid>,
    pub destination_number: Option<String>,
}

/// Update model for partial smart router rule updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = smart_router_rules)]
pub struct UpdateSmartRouterRule {
    pub sort_order: Option<i32>,
    pub condition_field: Option<String>,
    pub condition_operator: Option<String>,
    pub condition_value: Option<String>,
    pub destination_type: Option<Option<String>>,
    pub destination_id: Option<Option<Uuid>>,
    pub destination_number: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// agent_scripts
// ---------------------------------------------------------------------------

/// Read model for the `iiz.agent_scripts` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = agent_scripts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentScript {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new agent script.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = agent_scripts)]
pub struct NewAgentScript {
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
}

/// Update model for partial agent script updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = agent_scripts)]
pub struct UpdateAgentScript {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub content: Option<String>,
}

// ---------------------------------------------------------------------------
// scoring_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.scoring_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = scoring_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScoringConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub answer_rate_weight: i32,
    pub talk_time_weight: i32,
    pub conversion_weight: i32,
    pub min_talk_time_secs: i32,
    pub target_answer_rate: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new scoring config.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// All weight and threshold fields are user-configurable.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = scoring_configs)]
pub struct NewScoringConfig {
    pub account_id: Uuid,
    pub answer_rate_weight: i32,
    pub talk_time_weight: i32,
    pub conversion_weight: i32,
    pub min_talk_time_secs: i32,
    pub target_answer_rate: i32,
}

/// Update model for partial scoring config updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = scoring_configs)]
pub struct UpdateScoringConfig {
    pub answer_rate_weight: Option<i32>,
    pub talk_time_weight: Option<i32>,
    pub conversion_weight: Option<i32>,
    pub min_talk_time_secs: Option<i32>,
    pub target_answer_rate: Option<i32>,
}
