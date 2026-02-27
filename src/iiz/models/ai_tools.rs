//! Diesel model structs for the `ask_ai_configs`, `summary_configs`,
//! `knowledge_banks`, `knowledge_bank_documents`, `knowledge_bank_embeddings`,
//! `voice_ai_agents`, `chat_ai_agents`, `chat_ai_configs`, and
//! `dialogflow_configs` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::schema::iiz::{
    ask_ai_configs, chat_ai_agents, chat_ai_configs, dialogflow_configs, knowledge_bank_documents,
    knowledge_bank_embeddings, knowledge_banks, summary_configs, voice_ai_agents,
};

// ---------------------------------------------------------------------------
// ask_ai_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.ask_ai_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = ask_ai_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AskAiConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub preset: String,
    pub custom_prompt: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub delay: Option<String>,
    pub output_action: Option<String>,
    pub workflow_ids: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.ask_ai_configs` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = ask_ai_configs)]
pub struct NewAskAiConfig {
    pub account_id: Uuid,
    pub name: String,
    pub preset: String,
    pub custom_prompt: Option<String>,
    pub tracking_number_id: Option<Uuid>,
    pub delay: Option<String>,
    pub output_action: Option<String>,
    pub workflow_ids: Option<serde_json::Value>,
    pub is_active: bool,
}

/// Update model for the `iiz.ask_ai_configs` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = ask_ai_configs)]
pub struct UpdateAskAiConfig {
    pub name: Option<String>,
    pub preset: Option<String>,
    pub custom_prompt: Option<Option<String>>,
    pub tracking_number_id: Option<Option<Uuid>>,
    pub delay: Option<Option<String>>,
    pub output_action: Option<Option<String>>,
    pub workflow_ids: Option<Option<serde_json::Value>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// summary_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.summary_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = summary_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SummaryConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub phone_enabled: bool,
    pub video_enabled: bool,
    pub chat_enabled: bool,
    pub enabled_summary_types: Option<serde_json::Value>,
    pub transcribe_all: bool,
    pub transcription_language: Option<String>,
    pub pii_redaction_enabled: bool,
    pub pii_redaction_rules: Option<String>,
    pub default_model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.summary_configs` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = summary_configs)]
pub struct NewSummaryConfig {
    pub account_id: Uuid,
    pub phone_enabled: bool,
    pub video_enabled: bool,
    pub chat_enabled: bool,
    pub enabled_summary_types: Option<serde_json::Value>,
    pub transcribe_all: bool,
    pub transcription_language: Option<String>,
    pub pii_redaction_enabled: bool,
    pub pii_redaction_rules: Option<String>,
    pub default_model: Option<String>,
}

/// Update model for the `iiz.summary_configs` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = summary_configs)]
pub struct UpdateSummaryConfig {
    pub phone_enabled: Option<bool>,
    pub video_enabled: Option<bool>,
    pub chat_enabled: Option<bool>,
    pub enabled_summary_types: Option<Option<serde_json::Value>>,
    pub transcribe_all: Option<bool>,
    pub transcription_language: Option<Option<String>>,
    pub pii_redaction_enabled: Option<bool>,
    pub pii_redaction_rules: Option<Option<String>>,
    pub default_model: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// knowledge_banks
// ---------------------------------------------------------------------------

/// Read model for the `iiz.knowledge_banks` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = knowledge_banks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgeBank {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub category: String,
    pub document_count: i32,
    pub total_size_bytes: i64,
    pub status: String,
    pub last_import_at: Option<DateTime<Utc>>,
    pub used_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.knowledge_banks` table.
/// System-maintained: `document_count`, `total_size_bytes` (excluded).
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = knowledge_banks)]
pub struct NewKnowledgeBank {
    pub account_id: Uuid,
    pub name: String,
    pub category: String,
    pub status: String,
    pub last_import_at: Option<DateTime<Utc>>,
    pub used_by: Option<String>,
}

/// Update model for the `iiz.knowledge_banks` table.
/// System-maintained: `document_count`, `total_size_bytes` (excluded).
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = knowledge_banks)]
pub struct UpdateKnowledgeBank {
    pub name: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub last_import_at: Option<Option<DateTime<Utc>>>,
    pub used_by: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// knowledge_bank_documents
// ---------------------------------------------------------------------------

/// Read model for the `iiz.knowledge_bank_documents` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = knowledge_bank_documents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgeBankDocument {
    pub id: Uuid,
    pub account_id: Uuid,
    pub bank_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub source_url: Option<String>,
    pub file_ref: Option<String>,
    pub content_hash: Option<String>,
    pub file_size_bytes: i64,
    pub page_count: Option<i32>,
    pub chunk_count: i32,
    pub embedding_status: String,
    pub embedding_model: Option<String>,
    pub error_message: Option<String>,
    pub indexed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.knowledge_bank_documents` table.
/// System-maintained: `chunk_count` (excluded).
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = knowledge_bank_documents)]
pub struct NewKnowledgeBankDocument {
    pub account_id: Uuid,
    pub bank_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub source_url: Option<String>,
    pub file_ref: Option<String>,
    pub content_hash: Option<String>,
    pub file_size_bytes: i64,
    pub page_count: Option<i32>,
    pub embedding_status: String,
    pub embedding_model: Option<String>,
    pub error_message: Option<String>,
    pub indexed_at: Option<DateTime<Utc>>,
}

/// Update model for the `iiz.knowledge_bank_documents` table.
/// System-maintained: `chunk_count` (excluded).
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = knowledge_bank_documents)]
pub struct UpdateKnowledgeBankDocument {
    pub filename: Option<String>,
    pub file_type: Option<String>,
    pub source_url: Option<Option<String>>,
    pub file_ref: Option<Option<String>>,
    pub content_hash: Option<Option<String>>,
    pub file_size_bytes: Option<i64>,
    pub page_count: Option<Option<i32>>,
    pub embedding_status: Option<String>,
    pub embedding_model: Option<Option<String>>,
    pub error_message: Option<Option<String>>,
    pub indexed_at: Option<Option<DateTime<Utc>>>,
}

// ---------------------------------------------------------------------------
// knowledge_bank_embeddings
// ---------------------------------------------------------------------------

/// Read model for the `iiz.knowledge_bank_embeddings` table.
/// Note: this table has no `updated_at` column.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = knowledge_bank_embeddings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgeBankEmbedding {
    pub id: Uuid,
    pub account_id: Uuid,
    pub document_id: Uuid,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub embedding: Option<Vector>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.knowledge_bank_embeddings` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = knowledge_bank_embeddings)]
pub struct NewKnowledgeBankEmbedding {
    pub account_id: Uuid,
    pub document_id: Uuid,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub embedding: Option<Vector>,
}

/// Update model for the `iiz.knowledge_bank_embeddings` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = knowledge_bank_embeddings)]
pub struct UpdateKnowledgeBankEmbedding {
    pub chunk_text: Option<String>,
    pub embedding: Option<Option<Vector>>,
}

// ---------------------------------------------------------------------------
// voice_ai_agents
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voice_ai_agents` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voice_ai_agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoiceAiAgent {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub welcome_message: Option<String>,
    pub instructions: Option<String>,
    pub voice: Option<String>,
    pub language: Option<String>,
    pub knowledge_bank_id: Option<Uuid>,
    pub max_turns: i32,
    pub handoff_threshold: Option<String>,
    pub handoff_destination_type: Option<String>,
    pub handoff_destination_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.voice_ai_agents` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voice_ai_agents)]
pub struct NewVoiceAiAgent {
    pub account_id: Uuid,
    pub name: String,
    pub welcome_message: Option<String>,
    pub instructions: Option<String>,
    pub voice: Option<String>,
    pub language: Option<String>,
    pub knowledge_bank_id: Option<Uuid>,
    pub max_turns: i32,
    pub handoff_threshold: Option<String>,
    pub handoff_destination_type: Option<String>,
    pub handoff_destination_id: Option<Uuid>,
    pub is_active: bool,
}

/// Update model for the `iiz.voice_ai_agents` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = voice_ai_agents)]
pub struct UpdateVoiceAiAgent {
    pub name: Option<String>,
    pub welcome_message: Option<Option<String>>,
    pub instructions: Option<Option<String>>,
    pub voice: Option<Option<String>>,
    pub language: Option<Option<String>>,
    pub knowledge_bank_id: Option<Option<Uuid>>,
    pub max_turns: Option<i32>,
    pub handoff_threshold: Option<Option<String>>,
    pub handoff_destination_type: Option<Option<String>>,
    pub handoff_destination_id: Option<Option<Uuid>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// chat_ai_agents
// ---------------------------------------------------------------------------

/// Read model for the `iiz.chat_ai_agents` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = chat_ai_agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatAiAgent {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub instructions: Option<String>,
    pub knowledge_bank_id: Option<Uuid>,
    pub welcome_message: Option<String>,
    pub max_turns: i32,
    pub handoff_threshold: Option<String>,
    pub handoff_queue_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.chat_ai_agents` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = chat_ai_agents)]
pub struct NewChatAiAgent {
    pub account_id: Uuid,
    pub name: String,
    pub instructions: Option<String>,
    pub knowledge_bank_id: Option<Uuid>,
    pub welcome_message: Option<String>,
    pub max_turns: i32,
    pub handoff_threshold: Option<String>,
    pub handoff_queue_id: Option<Uuid>,
    pub is_active: bool,
}

/// Update model for the `iiz.chat_ai_agents` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = chat_ai_agents)]
pub struct UpdateChatAiAgent {
    pub name: Option<String>,
    pub instructions: Option<Option<String>>,
    pub knowledge_bank_id: Option<Option<Uuid>>,
    pub welcome_message: Option<Option<String>>,
    pub max_turns: Option<i32>,
    pub handoff_threshold: Option<Option<String>>,
    pub handoff_queue_id: Option<Option<Uuid>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// chat_ai_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.chat_ai_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = chat_ai_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatAiConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub knowledge_bank_id: Option<Uuid>,
    pub instructions: Option<String>,
    pub max_turns: i32,
    pub handoff_threshold: Option<String>,
    pub crm_integration_enabled: bool,
    pub crm_type: Option<String>,
    pub crm_config: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.chat_ai_configs` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = chat_ai_configs)]
pub struct NewChatAiConfig {
    pub account_id: Uuid,
    pub name: String,
    pub knowledge_bank_id: Option<Uuid>,
    pub instructions: Option<String>,
    pub max_turns: i32,
    pub handoff_threshold: Option<String>,
    pub crm_integration_enabled: bool,
    pub crm_type: Option<String>,
    pub crm_config: Option<serde_json::Value>,
    pub is_active: bool,
}

/// Update model for the `iiz.chat_ai_configs` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = chat_ai_configs)]
pub struct UpdateChatAiConfig {
    pub name: Option<String>,
    pub knowledge_bank_id: Option<Option<Uuid>>,
    pub instructions: Option<Option<String>>,
    pub max_turns: Option<i32>,
    pub handoff_threshold: Option<Option<String>>,
    pub crm_integration_enabled: Option<bool>,
    pub crm_type: Option<Option<String>>,
    pub crm_config: Option<Option<serde_json::Value>>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// dialogflow_configs
// ---------------------------------------------------------------------------

/// Read model for the `iiz.dialogflow_configs` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dialogflow_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DialogflowConfig {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub project_id: Option<String>,
    pub service_account_json: Option<String>,
    pub language: Option<String>,
    pub default_intent: Option<String>,
    pub fallback_message: Option<String>,
    pub connection_status: String,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for the `iiz.dialogflow_configs` table.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dialogflow_configs)]
pub struct NewDialogflowConfig {
    pub account_id: Uuid,
    pub name: String,
    pub project_id: Option<String>,
    pub service_account_json: Option<String>,
    pub language: Option<String>,
    pub default_intent: Option<String>,
    pub fallback_message: Option<String>,
    pub connection_status: String,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Update model for the `iiz.dialogflow_configs` table.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dialogflow_configs)]
pub struct UpdateDialogflowConfig {
    pub name: Option<String>,
    pub project_id: Option<Option<String>>,
    pub service_account_json: Option<Option<String>>,
    pub language: Option<Option<String>>,
    pub default_intent: Option<Option<String>>,
    pub fallback_message: Option<Option<String>>,
    pub connection_status: Option<String>,
    pub last_tested_at: Option<Option<DateTime<Utc>>>,
    pub is_active: Option<bool>,
}
