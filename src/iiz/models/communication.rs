//! Diesel model structs for the `text_messages`, `text_records`, `chat_records`,
//! `fax_records`, `video_records`, `form_records`, and `export_records` tables.
//!
//! - `text_messages`, `text_records`, `fax_records`, `video_records`, `form_records`:
//!   Read + Insert only (append-only / immutable records).
//! - `chat_records`, `export_records`: Read + Insert + Update (mutable status fields).

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{CallDirection, ChannelType, ExportFormat};
use crate::iiz::schema::iiz::{
    chat_records, export_records, fax_records, form_records, text_messages, text_records,
    video_records,
};

// ---------------------------------------------------------------------------
// text_messages (composite PK: id, sent_at — partitioned, append-only)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.text_messages` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = text_messages)]
#[diesel(primary_key(id, sent_at))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TextMessage {
    pub id: Uuid,
    pub account_id: Uuid,
    pub contact_phone: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub call_id: Option<Uuid>,
    pub direction: CallDirection,
    pub body: String,
    pub status: String,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new text message.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = text_messages)]
pub struct NewTextMessage {
    pub account_id: Uuid,
    pub contact_phone: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub call_id: Option<Uuid>,
    pub direction: CallDirection,
    pub body: String,
    pub status: String,
    pub sent_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// text_records
// ---------------------------------------------------------------------------

/// Read model for the `iiz.text_records` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = text_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TextRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub contact_phone: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub direction: CallDirection,
    pub preview: Option<String>,
    pub status: String,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new text record.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = text_records)]
pub struct NewTextRecord {
    pub account_id: Uuid,
    pub contact_phone: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub direction: CallDirection,
    pub preview: Option<String>,
    pub status: String,
    pub sent_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// chat_records (mutable: agent assignment, status, message_count)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.chat_records` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = chat_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub visitor_name: Option<String>,
    pub visitor_detail: Option<String>,
    pub channel: Option<ChannelType>,
    pub message_count: i32,
    pub agent_id: Option<Uuid>,
    pub widget_id: Option<Uuid>,
    pub status: String,
    pub duration_secs: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new chat record.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
/// `message_count` and `duration_secs` are system-maintained.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = chat_records)]
pub struct NewChatRecord {
    pub account_id: Uuid,
    pub visitor_name: Option<String>,
    pub visitor_detail: Option<String>,
    pub channel: Option<ChannelType>,
    pub message_count: i32,
    pub agent_id: Option<Uuid>,
    pub widget_id: Option<Uuid>,
    pub status: String,
    pub duration_secs: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

/// Update model for partial chat record updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `message_count` and `duration_secs` are system-maintained and excluded.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = chat_records)]
pub struct UpdateChatRecord {
    pub visitor_name: Option<Option<String>>,
    pub visitor_detail: Option<Option<String>>,
    pub channel: Option<Option<ChannelType>>,
    pub agent_id: Option<Option<Uuid>>,
    pub widget_id: Option<Option<Uuid>>,
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<Option<DateTime<Utc>>>,
}

// ---------------------------------------------------------------------------
// fax_records
// ---------------------------------------------------------------------------

/// Read model for the `iiz.fax_records` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = fax_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FaxRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub from_number: Option<String>,
    pub to_number: Option<String>,
    pub direction: CallDirection,
    pub pages: i32,
    pub status: String,
    pub document_url: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new fax record.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = fax_records)]
pub struct NewFaxRecord {
    pub account_id: Uuid,
    pub from_number: Option<String>,
    pub to_number: Option<String>,
    pub direction: CallDirection,
    pub pages: i32,
    pub status: String,
    pub document_url: Option<String>,
    pub sent_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// video_records
// ---------------------------------------------------------------------------

/// Read model for the `iiz.video_records` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = video_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VideoRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub participant_name: Option<String>,
    pub participant_email: Option<String>,
    pub host_agent_id: Option<Uuid>,
    pub platform: Option<String>,
    pub has_recording: bool,
    pub recording_url: Option<String>,
    pub duration_secs: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new video record.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = video_records)]
pub struct NewVideoRecord {
    pub account_id: Uuid,
    pub participant_name: Option<String>,
    pub participant_email: Option<String>,
    pub host_agent_id: Option<Uuid>,
    pub platform: Option<String>,
    pub has_recording: bool,
    pub recording_url: Option<String>,
    pub duration_secs: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// form_records
// ---------------------------------------------------------------------------

/// Read model for the `iiz.form_records` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = form_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FormRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub form_name: Option<String>,
    pub source: Option<String>,
    pub tracking_number: Option<String>,
    pub form_data: Option<serde_json::Value>,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new form record.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = form_records)]
pub struct NewFormRecord {
    pub account_id: Uuid,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub form_name: Option<String>,
    pub source: Option<String>,
    pub tracking_number: Option<String>,
    pub form_data: Option<serde_json::Value>,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// export_records (mutable: status, download_url after completion)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.export_records` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = export_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExportRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: Option<String>,
    pub export_type: Option<String>,
    pub format: ExportFormat,
    pub date_range: Option<String>,
    pub record_count: i32,
    pub status: String,
    pub download_url: Option<String>,
    pub requested_by_id: Option<Uuid>,
    pub filters_applied: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new export record.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
/// `record_count` is system-maintained.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = export_records)]
pub struct NewExportRecord {
    pub account_id: Uuid,
    pub name: Option<String>,
    pub export_type: Option<String>,
    pub format: ExportFormat,
    pub date_range: Option<String>,
    pub record_count: i32,
    pub status: String,
    pub download_url: Option<String>,
    pub requested_by_id: Option<Uuid>,
    pub filters_applied: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Update model for partial export record updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `record_count` is system-maintained and excluded.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = export_records)]
pub struct UpdateExportRecord {
    pub name: Option<Option<String>>,
    pub export_type: Option<Option<String>>,
    pub format: Option<ExportFormat>,
    pub date_range: Option<Option<String>>,
    pub status: Option<String>,
    pub download_url: Option<Option<String>>,
    pub requested_by_id: Option<Option<Uuid>>,
    pub filters_applied: Option<Option<serde_json::Value>>,
    pub completed_at: Option<Option<DateTime<Utc>>>,
}
