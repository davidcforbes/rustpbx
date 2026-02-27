//! Shared response types matching the backend API envelope.

use serde::{Deserialize, Serialize};

/// Pagination metadata returned with list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
}

/// Paginated list response from any collection endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    #[serde(flatten)]
    pub pagination: PaginationMeta,
    pub items: Vec<T>,
}

/// Error body returned by the API on failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

/// Generic API response — either data or error.
///
/// Used internally by the api_* helpers to parse error bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok(T),
    Err(ErrorBody),
}

// -------------------------------------------------------------------------
// Domain response types for Contacts section
// -------------------------------------------------------------------------

/// A contact list returned by GET /contacts/lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A blocked number returned by GET /contacts/blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedNumberItem {
    pub id: String,
    pub number: String,
    pub cnam: Option<String>,
    pub calls_blocked: i32,
    pub last_blocked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A Do-Not-Call entry returned by GET /contacts/dnc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DncEntryItem {
    pub id: String,
    pub number: String,
    pub added_by_id: Option<String>,
    pub reason: Option<String>,
    pub created_at: String,
}

/// A Do-Not-Text entry returned by GET /contacts/dnt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DntEntryItem {
    pub id: String,
    pub number: String,
    pub e164: String,
    pub rejected_count: i32,
    pub last_rejected_at: Option<String>,
    pub added_by_id: Option<String>,
    pub created_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Numbers section
// -------------------------------------------------------------------------

/// A tracking number returned by GET /numbers/tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingNumberItem {
    pub id: String,
    pub number: String,
    pub source_id: Option<String>,
    pub routing_description: Option<String>,
    pub routing_type: Option<String>,
    pub routing_target_type: Option<String>,
    pub routing_target_id: Option<String>,
    pub text_enabled: bool,
    pub receiving_number_id: Option<String>,
    pub number_type: String,
    pub number_class: String,
    pub pool_id: Option<String>,
    pub billing_date: Option<i32>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A tracking source returned by GET /numbers/sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSourceItem {
    pub id: String,
    pub name: String,
    pub source_type: Option<String>,
    pub position: i32,
    pub last_touch: bool,
    pub number_count: i32,
    pub call_count: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A receiving number returned by GET /numbers/receiving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingNumberItem {
    pub id: String,
    pub number: String,
    pub description: Option<String>,
    pub tracking_count: i32,
    pub total_calls: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A target number returned by GET /numbers/targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetNumberItem {
    pub id: String,
    pub number: String,
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub priority: i32,
    pub concurrency_cap: Option<i32>,
    pub weight: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A text-enabled number returned by GET /numbers/text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextNumberItem {
    pub id: String,
    pub number: String,
    pub name: Option<String>,
    pub is_assigned: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A call settings profile returned by GET /numbers/call-settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSettingItem {
    pub id: String,
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
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Flows section
// -------------------------------------------------------------------------

/// A voice menu returned by GET /flows/voice-menus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMenuItem {
    pub id: String,
    pub name: String,
    pub greeting_type: String,
    pub greeting_audio_url: Option<String>,
    pub greeting_text: Option<String>,
    pub speech_recognition: bool,
    pub speech_language: Option<String>,
    pub timeout_secs: i32,
    pub max_retries: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A queue returned by GET /flows/queues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub strategy: String,
    pub schedule_id: Option<String>,
    pub repeat_callers: bool,
    pub caller_id_display: Option<String>,
    pub max_wait_secs: i32,
    pub no_answer_destination_type: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A smart router returned by GET /flows/smart-routers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRouterItem {
    pub id: String,
    pub name: String,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A schedule returned by GET /flows/schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: String,
    pub name: String,
    pub timezone: String,
    pub monday_open: Option<String>,
    pub monday_close: Option<String>,
    pub tuesday_open: Option<String>,
    pub tuesday_close: Option<String>,
    pub wednesday_open: Option<String>,
    pub wednesday_close: Option<String>,
    pub thursday_open: Option<String>,
    pub thursday_close: Option<String>,
    pub friday_open: Option<String>,
    pub friday_close: Option<String>,
    pub saturday_open: Option<String>,
    pub saturday_close: Option<String>,
    pub sunday_open: Option<String>,
    pub sunday_close: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Flows section — Automation & Engagement
// -------------------------------------------------------------------------

/// A trigger returned by GET /flows/triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerItem {
    pub id: String,
    pub name: String,
    pub trigger_event: String,
    pub run_on: Option<String>,
    pub runs_7d: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A webhook returned by GET /flows/webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookItem {
    pub id: String,
    pub name: String,
    pub trigger_event: Option<String>,
    pub callback_url: String,
    pub method: String,
    pub body_type: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A bulk message returned by GET /flows/bulk-messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkMessageItem {
    pub id: String,
    pub label: Option<String>,
    pub sender_phone: Option<String>,
    pub message_body: String,
    pub recipient_count: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub failed_count: i32,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A form reactor entry returned by GET /flows/form-reactor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormReactorItem {
    pub id: String,
    pub name: String,
    pub form_fields: Option<String>,
    pub call_count: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A chat widget returned by GET /flows/chat-widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatWidgetItem {
    pub id: String,
    pub name: String,
    pub website_url: Option<String>,
    pub routing_type: Option<String>,
    pub agent_count: i32,
    pub custom_fields_count: i32,
    pub status: String,
    pub chat_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for AI Tools section
// -------------------------------------------------------------------------

/// A knowledge bank returned by GET /ai-tools/knowledge-banks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBankItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub document_count: i32,
    pub total_size_bytes: i64,
    pub status: String,
    pub last_import_at: Option<String>,
    pub used_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Activities section
// -------------------------------------------------------------------------

/// A text record returned by GET /activities/texts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRecordItem {
    pub id: String,
    pub contact_phone: Option<String>,
    pub tracking_number_id: Option<String>,
    pub direction: String,
    pub preview: Option<String>,
    pub status: String,
    pub sent_at: String,
    pub created_at: String,
}

/// A chat record returned by GET /activities/chats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRecordItem {
    pub id: String,
    pub visitor_name: Option<String>,
    pub visitor_detail: Option<String>,
    pub channel: Option<String>,
    pub message_count: i32,
    pub agent_id: Option<String>,
    pub widget_id: Option<String>,
    pub status: String,
    pub duration_secs: i32,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
}

/// A form record returned by GET /activities/forms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormRecordItem {
    pub id: String,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub form_name: Option<String>,
    pub source: Option<String>,
    pub tracking_number: Option<String>,
    pub status: String,
    pub submitted_at: String,
    pub created_at: String,
}

/// A fax record returned by GET /activities/fax
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaxRecordItem {
    pub id: String,
    pub from_number: Option<String>,
    pub to_number: Option<String>,
    pub direction: String,
    pub pages: i32,
    pub status: String,
    pub document_url: Option<String>,
    pub sent_at: String,
    pub created_at: String,
}

/// A video record returned by GET /activities/video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoRecordItem {
    pub id: String,
    pub participant_name: Option<String>,
    pub participant_email: Option<String>,
    pub host_agent_id: Option<String>,
    pub platform: Option<String>,
    pub has_recording: bool,
    pub recording_url: Option<String>,
    pub duration_secs: i32,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
}

/// An export record returned by GET /activities/exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRecordItem {
    pub id: String,
    pub name: Option<String>,
    pub export_type: Option<String>,
    pub format: String,
    pub date_range: Option<String>,
    pub record_count: i32,
    pub status: String,
    pub download_url: Option<String>,
    pub requested_by_id: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

/// An API log entry returned by GET /activities/api-logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogEntryItem {
    pub id: String,
    pub source: Option<String>,
    pub method: String,
    pub endpoint: String,
    pub response_code: Option<i32>,
    pub duration_ms: Option<i32>,
    pub activity_description: Option<String>,
    pub error_message: Option<String>,
    pub timestamp: String,
    pub created_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Trust Center section
// -------------------------------------------------------------------------

/// An A2P campaign returned by GET /trust-center/a2p-campaigns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2pCampaignItem {
    pub id: String,
    pub campaign_name: String,
    pub brand_name: Option<String>,
    pub use_case: Option<String>,
    pub assigned_numbers: i32,
    pub max_numbers: Option<i32>,
    pub monthly_cost: Option<f64>,
    pub carrier: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Trust Center section
// -------------------------------------------------------------------------

/// A compliance requirement returned by GET /trust-center/requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementItem {
    pub id: String,
    pub name: String,
    pub requirement_type: Option<String>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A compliance application returned by GET /trust-center/applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceApplicationItem {
    pub id: String,
    pub name: String,
    pub application_type: Option<String>,
    pub country: Option<String>,
    pub status: String,
    pub submitted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A compliance address returned by GET /trust-center/addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAddressItem {
    pub id: String,
    pub label: Option<String>,
    pub street_line1: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    pub is_verified: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A caller ID CNAM entry returned by GET /numbers/caller-id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerIdCnamItem {
    pub id: String,
    pub number: String,
    pub display_name: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A toll-free registration returned by GET /trust-center/toll-free
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TollFreeRegistrationItem {
    pub id: String,
    pub number: Option<String>,
    pub business_name: Option<String>,
    pub use_case: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A voice registration returned by GET /trust-center/voice-registrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceRegistrationItem {
    pub id: String,
    pub business_name: Option<String>,
    pub status: String,
    pub attestation_level: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Reports section
// -------------------------------------------------------------------------

/// A custom report returned by GET /reports/custom-reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReportItem {
    pub id: String,
    pub name: String,
    pub report_type: Option<String>,
    pub date_range_type: String,
    pub schedule: Option<String>,
    pub is_shared: bool,
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A notification rule returned by GET /reports/notification-rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRuleItem {
    pub id: String,
    pub name: String,
    pub metric: String,
    pub condition_operator: String,
    pub threshold_value: f64,
    pub time_window_minutes: i32,
    pub notification_method: String,
    pub cooldown_minutes: i32,
    pub is_active: bool,
    pub trigger_count: i32,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// An appointment returned by GET /reports/appointments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentItem {
    pub id: String,
    pub scheduled_at: String,
    pub caller_name: Option<String>,
    pub caller_phone: Option<String>,
    pub appointment_type: String,
    pub status: String,
    pub revenue: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A tag returned by GET /tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagItem {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub usage_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Flows section — additional
// -------------------------------------------------------------------------

/// A geo router returned by GET /flows/geo-routers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoRouterItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// An agent script returned by GET /flows/agent-scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentScriptItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A routing table returned by GET /flows/routing-tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTableItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A voicemail box returned by GET /flows/voicemails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicemailBoxItem {
    pub id: String,
    pub name: String,
    pub greeting_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A keyword spotting config returned by GET /flows/keyword-spotting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordSpottingItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A lambda returned by GET /flows/lambdas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub runtime: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A workflow returned by GET /flows/workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A lead reactor config returned by GET /flows/lead-reactor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadReactorItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A smart dialer config returned by GET /flows/smart-dialers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDialerItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A dialogflow config returned by GET /flows/dialogflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogflowItem {
    pub id: String,
    pub name: String,
    pub project_id: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A reminder returned by GET /flows/reminders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for AI Tools section — additional
// -------------------------------------------------------------------------

/// An AskAI config returned by GET /ai-tools/ask-ai
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskAiConfigItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model_provider: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A Voice AI agent returned by GET /ai-tools/voice-ai
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAiAgentItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub welcome_message: Option<String>,
    pub voice_name: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A Chat AI agent returned by GET /ai-tools/chat-ai
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAiAgentItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Numbers section — additional
// -------------------------------------------------------------------------

/// A number pool returned by GET /numbers/pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberPoolItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub pool_type: Option<String>,
    pub member_count: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// -------------------------------------------------------------------------
// Domain response types for Activities section — Call Records
// -------------------------------------------------------------------------

/// A call record returned by GET /activities/calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecordItem {
    pub id: String,
    pub caller_name: Option<String>,
    pub caller_number: Option<String>,
    pub caller_city: Option<String>,
    pub caller_state: Option<String>,
    pub caller_country: Option<String>,
    pub tracking_number_id: Option<String>,
    pub tracking_source_id: Option<String>,
    pub receiving_number_id: Option<String>,
    pub direction: String,
    pub status: String,
    pub duration_secs: i32,
    pub talk_time_secs: Option<i32>,
    pub ring_time_secs: Option<i32>,
    pub has_recording: bool,
    pub has_voicemail: bool,
    pub agent_id: Option<String>,
    pub queue_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
}

// -------------------------------------------------------------------------
// Dashboard report types (generic for all report pages)
// -------------------------------------------------------------------------

/// A KPI card on a dashboard report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportKpi {
    pub label: String,
    pub value: String,
    pub subtitle: Option<String>,
    pub trend: Option<String>,
    pub color: Option<String>,
}

/// A data point in a report chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub label: String,
    pub values: Vec<f64>,
}

/// A legend entry for a report chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartLegend {
    pub label: String,
    pub color: String,
}

/// Chart data for a dashboard report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportChart {
    pub title: String,
    pub chart_type: String,
    pub legend: Vec<ChartLegend>,
    pub points: Vec<ChartPoint>,
}

/// A table row in a dashboard report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<String>,
}

/// Complete dashboard data returned by GET /reports/dashboard/:type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub report_type: String,
    pub kpis: Vec<ReportKpi>,
    pub chart: Option<ReportChart>,
    pub table_headers: Vec<String>,
    pub table_rows: Vec<TableRow>,
    pub table_footer: Option<Vec<String>>,
    pub column_alignments: Option<Vec<String>>,
}

impl<T> ApiResponse<T> {
    /// Create an error response (used as fallback when JSON parsing fails).
    pub fn error(_status: u16, msg: &str) -> Self {
        ApiResponse::Err(ErrorBody {
            error: "unknown".to_string(),
            message: msg.to_string(),
        })
    }

    /// Extract the error message (or a default).
    pub fn message(&self) -> &str {
        match self {
            ApiResponse::Ok(_) => "ok",
            ApiResponse::Err(e) => &e.message,
        }
    }
}
