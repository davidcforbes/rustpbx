//! Diesel model structs for the Trust Center tables:
//! `business_info`, `authorized_contacts`, `a2p_campaigns`, `toll_free_registrations`,
//! `voice_registrations`, `voice_registration_history`, `compliance_requirements`,
//! `compliance_applications`, and `compliance_addresses`.
//!
//! Each table has three structs (Read + Insert + Update) except `voice_registration_history`
//! which is an event log with two structs (Read + Insert only).

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{AttestationLevel, ComplianceStatus};
use crate::iiz::schema::iiz::{
    a2p_campaigns, authorized_contacts, business_info, compliance_addresses,
    compliance_applications, compliance_requirements, toll_free_registrations,
    voice_registration_history, voice_registrations,
};

// ---------------------------------------------------------------------------
// business_info
// ---------------------------------------------------------------------------

/// Read model for the `iiz.business_info` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = business_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BusinessInfo {
    pub id: Uuid,
    pub account_id: Uuid,
    pub legal_business_name: Option<String>,
    pub dba: Option<String>,
    pub ein: Option<String>,
    pub industry: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new business info record.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = business_info)]
pub struct NewBusinessInfo {
    pub account_id: Uuid,
    pub legal_business_name: Option<String>,
    pub dba: Option<String>,
    pub ein: Option<String>,
    pub industry: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

/// Update model for partial business info updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = business_info)]
pub struct UpdateBusinessInfo {
    pub legal_business_name: Option<Option<String>>,
    pub dba: Option<Option<String>>,
    pub ein: Option<Option<String>>,
    pub industry: Option<Option<String>>,
    pub address_line1: Option<Option<String>>,
    pub address_line2: Option<Option<String>>,
    pub city: Option<Option<String>>,
    pub state: Option<Option<String>>,
    pub zip: Option<Option<String>>,
    pub country: Option<String>,
    pub phone: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub website: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// authorized_contacts
// ---------------------------------------------------------------------------

/// Read model for the `iiz.authorized_contacts` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = authorized_contacts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuthorizedContact {
    pub id: Uuid,
    pub account_id: Uuid,
    pub business_info_id: Uuid,
    pub name: String,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new authorized contact.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = authorized_contacts)]
pub struct NewAuthorizedContact {
    pub account_id: Uuid,
    pub business_info_id: Uuid,
    pub name: String,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

/// Update model for partial authorized contact updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = authorized_contacts)]
pub struct UpdateAuthorizedContact {
    pub business_info_id: Option<Uuid>,
    pub name: Option<String>,
    pub title: Option<Option<String>>,
    pub phone: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub is_primary: Option<bool>,
}

// ---------------------------------------------------------------------------
// a2p_campaigns
// ---------------------------------------------------------------------------

/// Read model for the `iiz.a2p_campaigns` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = a2p_campaigns)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct A2pCampaign {
    pub id: Uuid,
    pub account_id: Uuid,
    pub campaign_name: String,
    pub brand_name: Option<String>,
    pub use_case: Option<String>,
    pub description: Option<String>,
    pub sample_messages: Option<String>,
    pub opt_in_description: Option<String>,
    pub opt_out_description: Option<String>,
    pub assigned_numbers: i32,
    pub max_numbers: Option<i32>,
    pub monthly_cost: Option<BigDecimal>,
    pub carrier: Option<String>,
    pub status: ComplianceStatus,
    pub rejection_reason: Option<String>,
    pub dlc_campaign_id: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new A2P campaign.
/// `id`, `assigned_numbers`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `assigned_numbers` is system-maintained and excluded.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = a2p_campaigns)]
pub struct NewA2pCampaign {
    pub account_id: Uuid,
    pub campaign_name: String,
    pub brand_name: Option<String>,
    pub use_case: Option<String>,
    pub description: Option<String>,
    pub sample_messages: Option<String>,
    pub opt_in_description: Option<String>,
    pub opt_out_description: Option<String>,
    pub max_numbers: Option<i32>,
    pub monthly_cost: Option<BigDecimal>,
    pub carrier: Option<String>,
    pub status: ComplianceStatus,
    pub rejection_reason: Option<String>,
    pub dlc_campaign_id: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
}

/// Update model for partial A2P campaign updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `assigned_numbers` is system-maintained and excluded.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = a2p_campaigns)]
pub struct UpdateA2pCampaign {
    pub campaign_name: Option<String>,
    pub brand_name: Option<Option<String>>,
    pub use_case: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub sample_messages: Option<Option<String>>,
    pub opt_in_description: Option<Option<String>>,
    pub opt_out_description: Option<Option<String>>,
    pub max_numbers: Option<Option<i32>>,
    pub monthly_cost: Option<Option<BigDecimal>>,
    pub carrier: Option<Option<String>>,
    pub status: Option<ComplianceStatus>,
    pub rejection_reason: Option<Option<String>>,
    pub dlc_campaign_id: Option<Option<String>>,
    pub submitted_at: Option<Option<DateTime<Utc>>>,
    pub approved_at: Option<Option<DateTime<Utc>>>,
}

// ---------------------------------------------------------------------------
// toll_free_registrations
// ---------------------------------------------------------------------------

/// Read model for the `iiz.toll_free_registrations` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = toll_free_registrations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TollFreeRegistration {
    pub id: Uuid,
    pub account_id: Uuid,
    pub business_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub use_case: Option<String>,
    pub use_case_description: Option<String>,
    pub monthly_volume: Option<String>,
    pub toll_free_numbers: Option<serde_json::Value>,
    pub status: ComplianceStatus,
    pub rejection_reason: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new toll-free registration.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = toll_free_registrations)]
pub struct NewTollFreeRegistration {
    pub account_id: Uuid,
    pub business_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub use_case: Option<String>,
    pub use_case_description: Option<String>,
    pub monthly_volume: Option<String>,
    pub toll_free_numbers: Option<serde_json::Value>,
    pub status: ComplianceStatus,
    pub rejection_reason: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
}

/// Update model for partial toll-free registration updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = toll_free_registrations)]
pub struct UpdateTollFreeRegistration {
    pub business_name: Option<Option<String>>,
    pub contact_name: Option<Option<String>>,
    pub contact_phone: Option<Option<String>>,
    pub contact_email: Option<Option<String>>,
    pub use_case: Option<Option<String>>,
    pub use_case_description: Option<Option<String>>,
    pub monthly_volume: Option<Option<String>>,
    pub toll_free_numbers: Option<Option<serde_json::Value>>,
    pub status: Option<ComplianceStatus>,
    pub rejection_reason: Option<Option<String>>,
    pub submitted_at: Option<Option<DateTime<Utc>>>,
    pub approved_at: Option<Option<DateTime<Utc>>>,
}

// ---------------------------------------------------------------------------
// voice_registrations
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voice_registrations` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voice_registrations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoiceRegistration {
    pub id: Uuid,
    pub account_id: Uuid,
    pub business_name: Option<String>,
    pub ein: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub status: ComplianceStatus,
    pub attestation_level: Option<AttestationLevel>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub next_verification_due: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new voice registration.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voice_registrations)]
pub struct NewVoiceRegistration {
    pub account_id: Uuid,
    pub business_name: Option<String>,
    pub ein: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub status: ComplianceStatus,
    pub attestation_level: Option<AttestationLevel>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub next_verification_due: Option<NaiveDate>,
}

/// Update model for partial voice registration updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = voice_registrations)]
pub struct UpdateVoiceRegistration {
    pub business_name: Option<Option<String>>,
    pub ein: Option<Option<String>>,
    pub address_line1: Option<Option<String>>,
    pub address_line2: Option<Option<String>>,
    pub city: Option<Option<String>>,
    pub state: Option<Option<String>>,
    pub zip: Option<Option<String>>,
    pub status: Option<ComplianceStatus>,
    pub attestation_level: Option<Option<AttestationLevel>>,
    pub last_verified_at: Option<Option<DateTime<Utc>>>,
    pub next_verification_due: Option<Option<NaiveDate>>,
}

// ---------------------------------------------------------------------------
// voice_registration_history (READ + INSERT only — event log)
// ---------------------------------------------------------------------------

/// Read model for the `iiz.voice_registration_history` table.
/// This is an event log table with no `updated_at` column.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = voice_registration_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoiceRegistrationHistoryEntry {
    pub id: Uuid,
    pub account_id: Uuid,
    pub registration_id: Uuid,
    pub event_date: NaiveDate,
    pub event_type: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for a new voice registration history event.
/// `id`, `created_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = voice_registration_history)]
pub struct NewVoiceRegistrationHistoryEntry {
    pub account_id: Uuid,
    pub registration_id: Uuid,
    pub event_date: NaiveDate,
    pub event_type: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// compliance_requirements
// ---------------------------------------------------------------------------

/// Read model for the `iiz.compliance_requirements` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = compliance_requirements)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ComplianceRequirement {
    pub id: Uuid,
    pub account_id: Uuid,
    pub country: String,
    pub requirement_name: String,
    pub requirement_description: Option<String>,
    pub status: ComplianceStatus,
    pub documentation_url: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new compliance requirement.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = compliance_requirements)]
pub struct NewComplianceRequirement {
    pub account_id: Uuid,
    pub country: String,
    pub requirement_name: String,
    pub requirement_description: Option<String>,
    pub status: ComplianceStatus,
    pub documentation_url: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Update model for partial compliance requirement updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = compliance_requirements)]
pub struct UpdateComplianceRequirement {
    pub country: Option<String>,
    pub requirement_name: Option<String>,
    pub requirement_description: Option<Option<String>>,
    pub status: Option<ComplianceStatus>,
    pub documentation_url: Option<Option<String>>,
    pub due_date: Option<Option<NaiveDate>>,
    pub completed_at: Option<Option<DateTime<Utc>>>,
}

// ---------------------------------------------------------------------------
// compliance_applications
// ---------------------------------------------------------------------------

/// Read model for the `iiz.compliance_applications` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = compliance_applications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ComplianceApplication {
    pub id: Uuid,
    pub account_id: Uuid,
    pub application_name: String,
    pub application_type: Option<String>,
    pub country: String,
    pub status: ComplianceStatus,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub external_reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new compliance application.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = compliance_applications)]
pub struct NewComplianceApplication {
    pub account_id: Uuid,
    pub application_name: String,
    pub application_type: Option<String>,
    pub country: String,
    pub status: ComplianceStatus,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub external_reference_id: Option<String>,
}

/// Update model for partial compliance application updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = compliance_applications)]
pub struct UpdateComplianceApplication {
    pub application_name: Option<String>,
    pub application_type: Option<Option<String>>,
    pub country: Option<String>,
    pub status: Option<ComplianceStatus>,
    pub submitted_at: Option<Option<DateTime<Utc>>>,
    pub reviewed_at: Option<Option<DateTime<Utc>>>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub rejection_reason: Option<Option<String>>,
    pub external_reference_id: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// compliance_addresses
// ---------------------------------------------------------------------------

/// Read model for the `iiz.compliance_addresses` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = compliance_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ComplianceAddress {
    pub id: Uuid,
    pub account_id: Uuid,
    pub label: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub is_verified: bool,
    pub verification_method: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new compliance address.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = compliance_addresses)]
pub struct NewComplianceAddress {
    pub account_id: Uuid,
    pub label: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub is_verified: bool,
    pub verification_method: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Update model for partial compliance address updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = compliance_addresses)]
pub struct UpdateComplianceAddress {
    pub label: Option<Option<String>>,
    pub address_line1: Option<String>,
    pub address_line2: Option<Option<String>>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub is_verified: Option<bool>,
    pub verification_method: Option<Option<String>>,
    pub verified_at: Option<Option<DateTime<Utc>>>,
}
