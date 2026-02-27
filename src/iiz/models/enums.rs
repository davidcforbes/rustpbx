//! Diesel enum type mappings for all PostgreSQL enum types in the iiz schema.
//!
//! Each enum maps to a corresponding `CREATE TYPE iiz.<name>` in PostgreSQL
//! via `diesel-derive-enum`. The `ExistingTypePath` attribute links each enum
//! to its SqlType struct in `crate::iiz::schema::iiz::sql_types`.

// ---------------------------------------------------------------------------
// Account & User enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::AccountStatus"]
pub enum AccountStatus {
    #[db_rename = "active"]
    #[serde(rename = "active")]
    Active,
    #[db_rename = "suspended"]
    #[serde(rename = "suspended")]
    Suspended,
    #[db_rename = "closed"]
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::AccountType"]
pub enum AccountType {
    #[db_rename = "agency"]
    #[serde(rename = "agency")]
    Agency,
    #[db_rename = "standard"]
    #[serde(rename = "standard")]
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::UserRole"]
pub enum UserRole {
    #[db_rename = "admin"]
    #[serde(rename = "admin")]
    Admin,
    #[db_rename = "agent"]
    #[serde(rename = "agent")]
    Agent,
    #[db_rename = "supervisor"]
    #[serde(rename = "supervisor")]
    Supervisor,
}

// ---------------------------------------------------------------------------
// Agent & Call Center enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::AgentStatus"]
pub enum AgentStatus {
    #[db_rename = "available"]
    #[serde(rename = "available")]
    Available,
    #[db_rename = "on_call"]
    #[serde(rename = "on_call")]
    OnCall,
    #[db_rename = "after_call_work"]
    #[serde(rename = "after_call_work")]
    AfterCallWork,
    #[db_rename = "offline"]
    #[serde(rename = "offline")]
    Offline,
    #[db_rename = "break"]
    #[serde(rename = "break")]
    Break,
    #[db_rename = "dnd"]
    #[serde(rename = "dnd")]
    Dnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::ActiveCallStatus"]
pub enum ActiveCallStatus {
    #[db_rename = "ringing"]
    #[serde(rename = "ringing")]
    Ringing,
    #[db_rename = "active"]
    #[serde(rename = "active")]
    Active,
    #[db_rename = "on_hold"]
    #[serde(rename = "on_hold")]
    OnHold,
    #[db_rename = "transferring"]
    #[serde(rename = "transferring")]
    Transferring,
    #[db_rename = "wrapping"]
    #[serde(rename = "wrapping")]
    Wrapping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::QueueStrategy"]
pub enum QueueStrategy {
    #[db_rename = "ring_all"]
    #[serde(rename = "ring_all")]
    RingAll,
    #[db_rename = "round_robin"]
    #[serde(rename = "round_robin")]
    RoundRobin,
    #[db_rename = "longest_idle"]
    #[serde(rename = "longest_idle")]
    LongestIdle,
    #[db_rename = "weighted"]
    #[serde(rename = "weighted")]
    Weighted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::MonitorMode"]
pub enum MonitorMode {
    #[db_rename = "listen"]
    #[serde(rename = "listen")]
    Listen,
    #[db_rename = "whisper"]
    #[serde(rename = "whisper")]
    Whisper,
    #[db_rename = "barge"]
    #[serde(rename = "barge")]
    Barge,
}

// ---------------------------------------------------------------------------
// Call & Communication enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::CallDirection"]
pub enum CallDirection {
    #[db_rename = "inbound"]
    #[serde(rename = "inbound")]
    Inbound,
    #[db_rename = "outbound"]
    #[serde(rename = "outbound")]
    Outbound,
    #[db_rename = "internal"]
    #[serde(rename = "internal")]
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::CallStatus"]
pub enum CallStatus {
    #[db_rename = "answered"]
    #[serde(rename = "answered")]
    Answered,
    #[db_rename = "missed"]
    #[serde(rename = "missed")]
    Missed,
    #[db_rename = "voicemail"]
    #[serde(rename = "voicemail")]
    Voicemail,
    #[db_rename = "in_progress"]
    #[serde(rename = "in_progress")]
    InProgress,
    #[db_rename = "failed"]
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::ChannelType"]
pub enum ChannelType {
    #[db_rename = "web_chat"]
    #[serde(rename = "web_chat")]
    WebChat,
    #[db_rename = "sms"]
    #[serde(rename = "sms")]
    Sms,
    #[db_rename = "whatsapp"]
    #[serde(rename = "whatsapp")]
    Whatsapp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::SpeakerType"]
pub enum SpeakerType {
    #[db_rename = "agent"]
    #[serde(rename = "agent")]
    Agent,
    #[db_rename = "caller"]
    #[serde(rename = "caller")]
    Caller,
    #[db_rename = "system"]
    #[serde(rename = "system")]
    System,
}

// ---------------------------------------------------------------------------
// Number & SIP enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::NumberClass"]
pub enum NumberClass {
    #[db_rename = "local"]
    #[serde(rename = "local")]
    Local,
    #[db_rename = "toll_free"]
    #[serde(rename = "toll_free")]
    TollFree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::NumberType"]
pub enum NumberType {
    #[db_rename = "offsite_static"]
    #[serde(rename = "offsite_static")]
    OffsiteStatic,
    #[db_rename = "onsite_dynamic"]
    #[serde(rename = "onsite_dynamic")]
    OnsiteDynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::SipTransport"]
pub enum SipTransport {
    #[db_rename = "udp"]
    #[serde(rename = "udp")]
    Udp,
    #[db_rename = "tcp"]
    #[serde(rename = "tcp")]
    Tcp,
    #[db_rename = "tls"]
    #[serde(rename = "tls")]
    Tls,
    #[db_rename = "wss"]
    #[serde(rename = "wss")]
    Wss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::AttestationLevel"]
pub enum AttestationLevel {
    #[db_rename = "a"]
    #[serde(rename = "a")]
    A,
    #[db_rename = "b"]
    #[serde(rename = "b")]
    B,
    #[db_rename = "c"]
    #[serde(rename = "c")]
    C,
}

// ---------------------------------------------------------------------------
// Compliance & Reporting enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::ComplianceStatus"]
pub enum ComplianceStatus {
    #[db_rename = "draft"]
    #[serde(rename = "draft")]
    Draft,
    #[db_rename = "submitted"]
    #[serde(rename = "submitted")]
    Submitted,
    #[db_rename = "pending"]
    #[serde(rename = "pending")]
    Pending,
    #[db_rename = "in_progress"]
    #[serde(rename = "in_progress")]
    InProgress,
    #[db_rename = "approved"]
    #[serde(rename = "approved")]
    Approved,
    #[db_rename = "rejected"]
    #[serde(rename = "rejected")]
    Rejected,
    #[db_rename = "suspended"]
    #[serde(rename = "suspended")]
    Suspended,
    #[db_rename = "expired"]
    #[serde(rename = "expired")]
    Expired,
    #[db_rename = "completed"]
    #[serde(rename = "completed")]
    Completed,
    #[db_rename = "not_started"]
    #[serde(rename = "not_started")]
    NotStarted,
    #[db_rename = "not_applicable"]
    #[serde(rename = "not_applicable")]
    NotApplicable,
    #[db_rename = "not_registered"]
    #[serde(rename = "not_registered")]
    NotRegistered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::ExportFormat"]
pub enum ExportFormat {
    #[db_rename = "csv"]
    #[serde(rename = "csv")]
    Csv,
    #[db_rename = "pdf"]
    #[serde(rename = "pdf")]
    Pdf,
    #[db_rename = "excel"]
    #[serde(rename = "excel")]
    Excel,
}

// ---------------------------------------------------------------------------
// Media & Workflow enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::GreetingType"]
pub enum GreetingType {
    #[db_rename = "audio"]
    #[serde(rename = "audio")]
    Audio,
    #[db_rename = "tts"]
    #[serde(rename = "tts")]
    Tts,
    #[db_rename = "default"]
    #[serde(rename = "default")]
    Default,
    #[db_rename = "none"]
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::SummaryType"]
pub enum SummaryType {
    #[db_rename = "classic"]
    #[serde(rename = "classic")]
    Classic,
    #[db_rename = "customer_success"]
    #[serde(rename = "customer_success")]
    CustomerSuccess,
    #[db_rename = "key_insights"]
    #[serde(rename = "key_insights")]
    KeyInsights,
    #[db_rename = "action_items"]
    #[serde(rename = "action_items")]
    ActionItems,
    #[db_rename = "sentiment_analysis"]
    #[serde(rename = "sentiment_analysis")]
    SentimentAnalysis,
    #[db_rename = "lead_qualification"]
    #[serde(rename = "lead_qualification")]
    LeadQualification,
    #[db_rename = "compliance_review"]
    #[serde(rename = "compliance_review")]
    ComplianceReview,
    #[db_rename = "topic_classification"]
    #[serde(rename = "topic_classification")]
    TopicClassification,
    #[db_rename = "custom"]
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::iiz::schema::iiz::sql_types::WorkflowNodeType"]
pub enum WorkflowNodeType {
    #[db_rename = "event"]
    #[serde(rename = "event")]
    Event,
    #[db_rename = "condition"]
    #[serde(rename = "condition")]
    Condition,
    #[db_rename = "action"]
    #[serde(rename = "action")]
    Action,
}
