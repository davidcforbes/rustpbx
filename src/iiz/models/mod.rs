//! Diesel model structs for the iiz schema.
//!
//! Each struct derives Queryable, Insertable, and Serde traits for use with
//! the Diesel ORM and API serialization.

pub mod accounts;
pub mod activities;
pub mod ai_tools;
pub mod automations;
pub mod communication;
pub mod contacts;
pub mod engagement;
pub mod enums;
pub mod flows;
pub mod numbers;
pub mod reports;
pub mod tags;
pub mod trust_center;

// Re-export enum types for convenience
pub use enums::*;

// Re-export model structs for convenience
pub use accounts::{
    Account, AccountVariable, NewAccount, NewAccountVariable, NewUser, UpdateAccount,
    UpdateAccountVariable, UpdateUser, User,
};
pub use contacts::{
    BlockedNumber, ContactList, ContactListMember, DncEntry, DntEntry, NewBlockedNumber,
    NewContactList, NewContactListMember, NewDncEntry, NewDntEntry, UpdateBlockedNumber,
    UpdateContactList, UpdateContactListMember, UpdateDncEntry, UpdateDntEntry,
};
pub use numbers::{
    CallSetting, CallerIdCnam, NewCallSetting, NewCallerIdCnam, NewNumberPool,
    NewNumberPoolMember, NewPortRequest, NewReceivingNumber, NewTargetNumber, NewTextNumber,
    NewTrackingNumber, NewTrackingSource, NumberPool, NumberPoolMember, PortRequest,
    ReceivingNumber, TargetNumber, TextNumber, TrackingNumber, TrackingSource, UpdateCallSetting,
    UpdateCallerIdCnam, UpdateNumberPool, UpdateNumberPoolMember, UpdatePortRequest,
    UpdateReceivingNumber, UpdateTargetNumber, UpdateTextNumber, UpdateTrackingNumber,
    UpdateTrackingSource,
};
pub use tags::{NewTag, Tag, UpdateTag};
pub use automations::{
    Lambda, LambdaEnvVar, NewLambda, NewLambdaEnvVar, NewTrigger, NewTriggerAction,
    NewTriggerCondition, NewWebhook, NewWebhookSubscription, NewWorkflow, NewWorkflowEdge,
    NewWorkflowNode, Trigger, TriggerAction, TriggerCondition, UpdateLambda, UpdateLambdaEnvVar,
    UpdateTrigger, UpdateTriggerAction, UpdateTriggerCondition, UpdateWebhook,
    UpdateWebhookSubscription, UpdateWorkflow, UpdateWorkflowEdge, UpdateWorkflowNode, Webhook,
    WebhookSubscription, Workflow, WorkflowEdge, WorkflowNode,
};
pub use engagement::{
    BulkMessage, ChatWidget, FormReactorEntry, KeywordSpottingConfig, KeywordSpottingKeyword,
    KeywordSpottingNumber, LeadReactorAction, LeadReactorConfig, NewBulkMessage, NewChatWidget,
    NewFormReactorEntry, NewKeywordSpottingConfig, NewKeywordSpottingKeyword,
    NewKeywordSpottingNumber, NewLeadReactorAction, NewLeadReactorConfig, NewReminder,
    NewSmartDialerConfig, Reminder, SmartDialerConfig, UpdateBulkMessage, UpdateChatWidget,
    UpdateFormReactorEntry, UpdateKeywordSpottingConfig, UpdateKeywordSpottingKeyword,
    UpdateKeywordSpottingNumber, UpdateLeadReactorAction, UpdateLeadReactorConfig, UpdateReminder,
    UpdateSmartDialerConfig,
};
pub use flows::{
    AgentScript, GeoRouter, GeoRouterRule, NewAgentScript, NewGeoRouter, NewGeoRouterRule,
    NewQueue, NewQueueAgent, NewRoutingTable, NewRoutingTableRoute, NewSchedule,
    NewScheduleHoliday, NewScoringConfig, NewSmartRouter, NewSmartRouterRule, NewVoiceMenu,
    NewVoiceMenuOption, NewVoicemailBox, NewVoicemailMessage, Queue, QueueAgent, RoutingTable,
    RoutingTableRoute, Schedule, ScheduleHoliday, ScoringConfig, SmartRouter, SmartRouterRule,
    UpdateAgentScript, UpdateGeoRouter, UpdateGeoRouterRule, UpdateQueue, UpdateQueueAgent,
    UpdateRoutingTable, UpdateRoutingTableRoute, UpdateSchedule, UpdateScheduleHoliday,
    UpdateScoringConfig, UpdateSmartRouter, UpdateSmartRouterRule, UpdateVoiceMenu,
    UpdateVoiceMenuOption, UpdateVoicemailBox, UpdateVoicemailMessage, VoiceMenu, VoiceMenuOption,
    VoicemailBox, VoicemailMessage,
};
pub use ai_tools::{
    AskAiConfig, ChatAiAgent, ChatAiConfig, DialogflowConfig, KnowledgeBank,
    KnowledgeBankDocument, KnowledgeBankEmbedding, NewAskAiConfig, NewChatAiAgent,
    NewChatAiConfig, NewDialogflowConfig, NewKnowledgeBank, NewKnowledgeBankDocument,
    NewKnowledgeBankEmbedding, NewSummaryConfig, NewVoiceAiAgent, SummaryConfig,
    UpdateAskAiConfig, UpdateChatAiAgent, UpdateChatAiConfig, UpdateDialogflowConfig,
    UpdateKnowledgeBank, UpdateKnowledgeBankDocument, UpdateKnowledgeBankEmbedding,
    UpdateSummaryConfig, UpdateVoiceAiAgent, VoiceAiAgent,
};
pub use activities::{
    ActiveCall, AgentStateLogEntry, ApiLogEntry, CallAiSummary, CallAnnotation, CallDailySummary,
    CallFlowEvent, CallKeywordHit, CallRecord, CallTag, CallTranscriptionSegment,
    CallVisitorSession, MonitoringEvent, NewActiveCall, NewAgentStateLogEntry, NewApiLogEntry,
    NewCallAiSummary, NewCallAnnotation, NewCallDailySummary, NewCallFlowEvent, NewCallKeywordHit,
    NewCallRecord, NewCallTag, NewCallTranscriptionSegment, NewCallVisitorSession,
    NewMonitoringEvent, UpdateActiveCall, UpdateCallAnnotation, UpdateCallDailySummary,
};
pub use reports::{
    Appointment, CustomReport, NewAppointment, NewCustomReport, NewNotification,
    NewNotificationRule, Notification, NotificationRule, UpdateAppointment, UpdateCustomReport,
    UpdateNotification, UpdateNotificationRule,
};
pub use communication::{
    ChatRecord, ExportRecord, FaxRecord, FormRecord, NewChatRecord, NewExportRecord, NewFaxRecord,
    NewFormRecord, NewTextMessage, NewTextRecord, NewVideoRecord, TextMessage, TextRecord,
    UpdateChatRecord, UpdateExportRecord, VideoRecord,
};
pub use trust_center::{
    A2pCampaign, AuthorizedContact, BusinessInfo, ComplianceAddress, ComplianceApplication,
    ComplianceRequirement, NewA2pCampaign, NewAuthorizedContact, NewBusinessInfo,
    NewComplianceAddress, NewComplianceApplication, NewComplianceRequirement,
    NewTollFreeRegistration, NewVoiceRegistration, NewVoiceRegistrationHistoryEntry,
    TollFreeRegistration, UpdateA2pCampaign, UpdateAuthorizedContact, UpdateBusinessInfo,
    UpdateComplianceAddress, UpdateComplianceApplication, UpdateComplianceRequirement,
    UpdateTollFreeRegistration, UpdateVoiceRegistration, VoiceRegistration,
    VoiceRegistrationHistoryEntry,
};

// Common type aliases used across models
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;
