//! Diesel model structs for report & notification tables in the iiz schema.
//!
//! Tables: custom_reports, notification_rules, appointments, notifications.
//! All four tables support full CRUD (Read + Insert + Update).
//!
//! NOTE: `call_daily_summary` and `monitoring_events` are modeled in `activities.rs`.

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::schema::iiz::{appointments, custom_reports, notification_rules, notifications};

// ---------------------------------------------------------------------------
// custom_reports (PK: id) — full CRUD
// ---------------------------------------------------------------------------

/// Read model for the `iiz.custom_reports` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = custom_reports)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CustomReport {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub report_type: Option<String>,
    pub columns_: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub date_range_type: String,
    pub custom_start_date: Option<NaiveDate>,
    pub custom_end_date: Option<NaiveDate>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
    pub schedule: Option<String>,
    pub schedule_recipients: Option<serde_json::Value>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_by_id: Option<Uuid>,
    pub is_shared: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new custom report.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `last_run_at` is system-maintained (set when report is executed).
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = custom_reports)]
pub struct NewCustomReport {
    pub account_id: Uuid,
    pub name: String,
    pub report_type: Option<String>,
    pub columns_: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub date_range_type: String,
    pub custom_start_date: Option<NaiveDate>,
    pub custom_end_date: Option<NaiveDate>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
    pub schedule: Option<String>,
    pub schedule_recipients: Option<serde_json::Value>,
    pub created_by_id: Option<Uuid>,
    pub is_shared: bool,
}

/// Update model for partial custom report updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// Nullable columns use `Option<Option<T>>` (outer = skip vs set, inner = null vs value).
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = custom_reports)]
pub struct UpdateCustomReport {
    pub name: Option<String>,
    pub report_type: Option<Option<String>>,
    pub columns_: Option<Option<serde_json::Value>>,
    pub filters: Option<Option<serde_json::Value>>,
    pub date_range_type: Option<String>,
    pub custom_start_date: Option<Option<NaiveDate>>,
    pub custom_end_date: Option<Option<NaiveDate>>,
    pub sort_column: Option<Option<String>>,
    pub sort_direction: Option<Option<String>>,
    pub schedule: Option<Option<String>>,
    pub schedule_recipients: Option<Option<serde_json::Value>>,
    pub created_by_id: Option<Option<Uuid>>,
    pub is_shared: Option<bool>,
}

// ---------------------------------------------------------------------------
// notification_rules (PK: id) — full CRUD
// ---------------------------------------------------------------------------

/// Read model for the `iiz.notification_rules` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = notification_rules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationRule {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub metric: String,
    pub condition_operator: String,
    pub threshold_value: BigDecimal,
    pub time_window_minutes: i32,
    pub notification_method: String,
    pub recipients: Option<serde_json::Value>,
    pub cooldown_minutes: i32,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new notification rule.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `trigger_count` and `last_triggered_at` are system-maintained (excluded from insert).
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = notification_rules)]
pub struct NewNotificationRule {
    pub account_id: Uuid,
    pub name: String,
    pub metric: String,
    pub condition_operator: String,
    pub threshold_value: BigDecimal,
    pub time_window_minutes: i32,
    pub notification_method: String,
    pub recipients: Option<serde_json::Value>,
    pub cooldown_minutes: i32,
    pub is_active: bool,
}

/// Update model for partial notification rule updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `trigger_count` and `last_triggered_at` are system-maintained (excluded from update).
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = notification_rules)]
pub struct UpdateNotificationRule {
    pub name: Option<String>,
    pub metric: Option<String>,
    pub condition_operator: Option<String>,
    pub threshold_value: Option<BigDecimal>,
    pub time_window_minutes: Option<i32>,
    pub notification_method: Option<String>,
    pub recipients: Option<Option<serde_json::Value>>,
    pub cooldown_minutes: Option<i32>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// appointments (PK: id) — full CRUD
// ---------------------------------------------------------------------------

/// Read model for the `iiz.appointments` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = appointments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Appointment {
    pub id: Uuid,
    pub account_id: Uuid,
    pub call_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub caller_name: Option<String>,
    pub caller_phone: Option<String>,
    pub source_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub appointment_type: String,
    pub status: String,
    pub revenue: Option<BigDecimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new appointment.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = appointments)]
pub struct NewAppointment {
    pub account_id: Uuid,
    pub call_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub caller_name: Option<String>,
    pub caller_phone: Option<String>,
    pub source_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub appointment_type: String,
    pub status: String,
    pub revenue: Option<BigDecimal>,
    pub notes: Option<String>,
}

/// Update model for partial appointment updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// Nullable columns use `Option<Option<T>>` (outer = skip vs set, inner = null vs value).
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = appointments)]
pub struct UpdateAppointment {
    pub call_id: Option<Option<Uuid>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub caller_name: Option<Option<String>>,
    pub caller_phone: Option<Option<String>>,
    pub source_id: Option<Option<Uuid>>,
    pub agent_id: Option<Option<Uuid>>,
    pub appointment_type: Option<String>,
    pub status: Option<String>,
    pub revenue: Option<Option<BigDecimal>>,
    pub notes: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// notifications (PK: id) — CRUD (only is_read is user-modifiable)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.notifications` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Notification {
    pub id: Uuid,
    pub account_id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub title: String,
    pub body: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new notification.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = notifications)]
pub struct NewNotification {
    pub account_id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub title: String,
    pub body: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub is_read: bool,
}

/// Update model for partial notification updates.
/// Only `is_read` is user-modifiable; all other fields are system-set at creation time.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = notifications)]
pub struct UpdateNotification {
    pub is_read: Option<bool>,
}
