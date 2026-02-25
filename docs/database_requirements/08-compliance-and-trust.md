# 08 — Compliance & Trust

## Overview

This shard documents the regulatory compliance and trust identity entities for the 4iiz call tracking platform. The compliance domain exists because US telecom law and carrier policy impose affirmative registration and disclosure obligations on any business that sends SMS messages or makes voice calls at scale. The three primary regulatory tracks are: voice call attestation under STIR/SHAKEN (FCC mandate, effective June 2021), A2P 10DLC registration for local-number SMS (carrier-mandated via The Campaign Registry), and toll-free number message verification (carrier-mandated via Syniverse or equivalent aggregators). A fourth operational concern — CNAM (Caller ID Name) management — is not a registration program per se but is closely associated with compliance because incorrect or missing caller ID is a carrier trust signal and a consumer protection issue.

The root identity anchor for all of these programs is the legal business entity. Before any registration can be submitted, the business must establish its legal identity (name, EIN, address) in BusinessInfo. AuthorizedContacts identify the humans accountable for those registrations. From this root, A2PCampaign, TollFreeRegistration, and VoiceRegistration branch outward as independent carrier registration workflows, each with its own approval lifecycle. ComplianceRequirement and ComplianceApplication provide a cross-cutting checklist and audit surface that spans all programs. ComplianceAddress supports multi-location businesses whose registrations may reference branch offices rather than a single headquarters. The Trust Center section of the UI surfaces all of these entities and is the primary interface for compliance operations within the platform.

---

## Entities

### BusinessInfo

**UI References:** Trust Center > Business Info page

**Relationships:**
- Belongs to one Account (many-to-one): each account has at most one BusinessInfo, enforced by UQ on account_id
- Has many AuthorizedContacts (one-to-many)
- Referenced by VoiceRegistration as the authoritative EIN and address source
- Referenced by A2PCampaign and TollFreeRegistration as the registered brand identity

**Notes:** EIN is encrypted at rest because it is a sensitive federal tax identifier subject to PII protection obligations. This entity must be fully populated before any compliance registration (A2P, toll-free, or voice) can be submitted — the UI enforces this as a prerequisite gate. The data here feeds directly into carrier registration APIs; field-level accuracy is critical because carrier rejections due to mismatched legal names or invalid EINs require manual remediation and incur additional processing delays. The country field defaults to "US" but accommodates Canadian registrants (CRTC compliance) and future international expansion.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN, UQ | Owning account; one BusinessInfo per account |
| legal_business_name | short_text | NN | Full legal name of the business as registered with state/federal authorities |
| dba | short_text | | "Doing Business As" trade name, if different from the legal business name |
| ein | encrypted_text | NN | Employer Identification Number (federal tax ID); encrypted at rest |
| industry | short_text | | Business vertical or industry classification (e.g., "Healthcare", "Automotive") |
| address_line1 | short_text | NN | Primary street address of the registered business location |
| address_line2 | short_text | | Suite, floor, unit, or secondary address component |
| city | short_text | NN | City of the registered business address |
| state | short_text | NN | State or province abbreviation (e.g., "CA", "ON") |
| zip | short_text | NN | Postal or ZIP code |
| country | short_text | NN | ISO 3166-1 alpha-2 country code; default "US" |
| phone | e164 | NN | Primary business phone number in E.164 format |
| email | email | NN | Primary business contact email address |
| website | url | | Business website URL |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### AuthorizedContact

**UI References:** Trust Center > Business Info page (contacts section)

**Relationships:**
- Belongs to one Account (many-to-one)
- Belongs to one BusinessInfo (many-to-one): an authorized contact is scoped to a specific business registration
- At most one contact per BusinessInfo may have is_primary = true (enforced at application layer)

**Notes:** Authorized contacts are the humans that carriers and regulatory bodies communicate with during registration review, rejection, and re-verification cycles. The is_primary flag designates which contact should receive automated compliance notifications from the platform (approval confirmations, rejection notices, renewal reminders). Multiple contacts are permitted to support organizations with separate legal, compliance, and technical contacts. Job title is stored to satisfy carrier intake forms that require contact seniority information.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| business_info_id | uuid | FK(BusinessInfo), NN | Business registration this contact is authorized to act on behalf of |
| name | short_text | NN | Full name of the authorized individual |
| title | short_text | | Job title or role (e.g., "Chief Compliance Officer", "VP of Marketing") |
| phone | e164 | NN | Direct phone number for the authorized contact in E.164 format |
| email | email | NN | Direct email address for the authorized contact |
| is_primary | boolean | NN | True if this is the primary compliance contact for the account; default false |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### A2PCampaign

**UI References:** Trust Center > Local Text Messaging page

**Relationships:**
- Belongs to one Account (many-to-one)
- Implicitly references BusinessInfo for brand identity (via account_id)
- Referenced by TrackingNumber and TextNumber (many-to-one): numbers are assigned to an approved campaign before they can send SMS
- Carrier approval is coordinated through The Campaign Registry (TCR) as an external system

**Notes:** A2P 10DLC (10-Digit Long Code) registration is required by all major US carriers (AT&T, T-Mobile, Verizon) for local phone numbers sending SMS at any commercial volume. Without an approved campaign, SMS messages from local numbers will be filtered or blocked at the carrier level. The status lifecycle is strictly ordered: Draft (being authored) → Pending (submitted to carrier) → Approved or Rejected. A Rejected campaign may be corrected and resubmitted, re-entering the Pending state. A Suspended campaign (e.g., due to abuse reports) must also be resubmitted. The assigned_numbers count is maintained by the application layer as tracking numbers are linked or unlinked from this campaign. The dlc_campaign_id is the external identifier assigned by The Campaign Registry upon submission and must be stored for carrier reconciliation and support escalations.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| campaign_name | short_text | NN | Internal name for identifying this campaign within the platform |
| brand_name | short_text | | Registered brand name as it will appear in TCR filings |
| use_case | enum(Marketing, Notifications, Customer Care, Delivery Notifications, Account Notifications, Two-Factor Authentication, Emergency, Charity) | NN | TCR-defined use case category that determines carrier scrutiny and throughput limits |
| description | long_text | NN | Full campaign description submitted to the carrier for review; must accurately describe messaging intent |
| sample_messages | long_text | NN | Representative example messages that will be sent under this campaign; carriers review these for content compliance |
| opt_in_description | text | | Description of how message recipients opt in to receive messages from this campaign |
| opt_out_description | text | | Description of how message recipients opt out (e.g., reply STOP instructions) |
| assigned_numbers | integer | default 0 | Count of phone numbers currently associated with this campaign |
| max_numbers | integer | | Carrier-imposed ceiling on the number of numbers that may use this campaign |
| monthly_cost | money | | Monthly recurring carrier fee for maintaining this campaign registration |
| carrier | short_text | | Primary registering carrier or aggregator (e.g., "Telnyx", "Twilio", "Bandwidth") |
| status | enum(Draft, Pending, Approved, Rejected, Suspended) | NN | Current registration status; default Draft |
| rejection_reason | text | | Carrier-provided explanation when status is Rejected or Suspended |
| dlc_campaign_id | short_text | | External campaign ID assigned by The Campaign Registry upon submission |
| submitted_at | timestamp_tz | | Timestamp when the campaign was first submitted to the carrier |
| approved_at | timestamp_tz | | Timestamp when the carrier granted approval |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### TollFreeRegistration

**UI References:** Trust Center > Toll-Free Text Messaging page

**Relationships:**
- Belongs to one Account (many-to-one)
- toll_free_numbers references E.164 numbers that are also tracked in the TrackingNumber and TextNumber entities; stored as a JSON array for carrier submission, not enforced as foreign keys at the DB layer
- Verified externally through the toll-free verification aggregator (e.g., Syniverse)

**Notes:** Toll-free number message verification is distinct from A2P 10DLC and uses a different registry and review process. Carriers began enforcing unverified toll-free filtering in 2023. A single TollFreeRegistration can cover multiple toll-free numbers owned by the same account, unlike A2P where campaigns are per-use-case. The Not Registered status indicates the account has toll-free numbers but has not yet initiated verification. Monthly volume helps the carrier allocate throughput capacity and is a required field on the submission form. Rejected registrations may be appealed or resubmitted with corrected information.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| business_name | short_text | NN | Legal business name as submitted to the toll-free verification registry |
| contact_name | short_text | NN | Full name of the primary contact for this registration |
| contact_phone | e164 | NN | Phone number of the registration contact in E.164 format |
| contact_email | email | NN | Email address of the registration contact |
| use_case | enum(Marketing, Notifications, Customer Care, Two-Factor Authentication, Emergency, Charity) | NN | Messaging use case category for the toll-free numbers being registered |
| use_case_description | long_text | | Detailed narrative description of the messaging use case provided to the verification aggregator |
| monthly_volume | enum(Under 1000, 1000-10000, 10000-100000, Over 100000) | NN | Estimated monthly SMS volume tier across all registered toll-free numbers |
| toll_free_numbers | json | | JSON array of E.164 toll-free numbers included in this registration submission |
| status | enum(Not Registered, Draft, Pending, Approved, Rejected) | NN | Current verification status; default Not Registered |
| rejection_reason | text | | Aggregator or carrier explanation when status is Rejected |
| submitted_at | timestamp_tz | | Timestamp when the registration was submitted to the verification aggregator |
| approved_at | timestamp_tz | | Timestamp when verification was granted |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### VoiceRegistration

**UI References:** Trust Center > Voice Registration page

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many VoiceRegistrationHistory records (one-to-many): every status change and verification event is journaled
- Implicitly references BusinessInfo for EIN and address; the fields are duplicated here at submission time to preserve a snapshot of what was filed

**Notes:** STIR/SHAKEN (Secure Telephone Identity Revisited / Signature-based Handling of Asserted information using toKENs) is an FCC-mandated framework for authenticating caller ID on voice calls. Attestation level A (Full) means the carrier can confirm the calling party is authorized to use the caller ID number; level B (Partial) means partial confidence; level C (Gateway) means the call entered the network at a gateway with no assurance. Higher attestation levels reduce the probability of a call being labeled "Spam Likely" on the recipient's device. EIN is stored encrypted because it is submitted as part of the voice provider registration and is a sensitive identifier. The next_verification_due date drives renewal reminder notifications and is calculated as one year from last_verified_at.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| business_name | short_text | NN | Legal business name as filed with the voice service provider |
| ein | encrypted_text | NN | Employer Identification Number submitted as part of the voice registration; encrypted at rest |
| address_line1 | short_text | NN | Registered business street address submitted with the application |
| address_line2 | short_text | | Suite, floor, or secondary address component |
| city | short_text | NN | City of the registered business address |
| state | short_text | NN | State or province abbreviation |
| zip | short_text | NN | Postal or ZIP code |
| status | enum(Not Registered, Pending, Approved, Rejected) | NN | Current STIR/SHAKEN registration status; default Not Registered |
| attestation_level | enum(A, B, C) | | STIR/SHAKEN attestation level granted: A = Full, B = Partial, C = Gateway |
| last_verified_at | timestamp_tz | | Timestamp of the most recent successful verification or re-verification |
| next_verification_due | date | | Calendar date by which annual re-verification must be completed |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### VoiceRegistrationHistory

**UI References:** Trust Center > Voice Registration page (history timeline)

**Relationships:**
- Belongs to one VoiceRegistration (many-to-one): the history log is scoped to a single registration record
- Records are append-only; no updates or deletes are permitted on this table

**Notes:** This entity implements the audit trail pattern for voice registration status transitions. Every submission, verification outcome, appeal, and annual re-verification event is recorded as a new row. The old_status and new_status fields capture the transition, while notes captures the carrier or regulatory body's response text. Rows are ordered by event_date ascending to construct the timeline view shown on the Voice Registration UI page. Because this is a compliance audit record, rows must be treated as immutable once written — the created_at timestamp serves as the authoritative record of when the platform became aware of the event.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| registration_id | uuid | FK(VoiceRegistration), NN | Parent voice registration this event belongs to |
| event_date | date | NN | Calendar date on which the event occurred (may differ from created_at if imported from carrier logs) |
| event_type | enum(Initial Registration, Annual Re-verification, Status Change, Update Submitted, Rejection Appeal) | NN | Classification of the compliance event |
| old_status | short_text | | Registration status before this event; null for Initial Registration events |
| new_status | short_text | | Registration status after this event |
| notes | text | | Carrier or regulatory body response text, internal notes, or appeal rationale |
| created_at | timestamp_tz | NN | Timestamp when this history record was written to the platform |

---

### CallerIdCnam

**UI References:** Trust Center > Caller ID / CNAM page

**Relationships:**
- Belongs to one Account (many-to-one)
- Optionally references one TrackingNumber (many-to-one): CNAM records for tracking numbers carry the FK; CNAM records for standalone numbers (receiving numbers, porting targets) do not
- number field corresponds to E.164 numbers that may also appear in TrackingNumber, ReceivingNumber, or TargetNumber entities

**Notes:** CNAM (Caller Name) is the 15-character string displayed as the caller's name on the recipient's phone. The 15-character limit is an NANP industry standard enforced by the CNAM database providers (Neustar, Transaction Systems). Updates to CNAM propagate through carrier databases asynchronously and can take 24-72 hours to appear on all networks. The requested_cnam field holds a pending update while propagation is in progress; once propagation is confirmed, current_cnam is updated to match and requested_cnam is cleared. Status Not Configured indicates the number has no CNAM record on file; this is the default state for newly acquired numbers.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| number | e164 | NN | The phone number whose CNAM record this row manages |
| tracking_number_id | uuid | FK(TrackingNumber) | Link to the corresponding TrackingNumber entity, if applicable |
| current_cnam | short_text | MAX(15) | The CNAM string currently active in the carrier database for this number |
| requested_cnam | short_text | MAX(15) | A pending CNAM update that has been submitted but not yet propagated across all carrier databases |
| status | enum(Active, Pending Update, Not Configured) | NN | Current CNAM management status; default Not Configured |
| last_updated_at | timestamp_tz | | Timestamp of the most recent confirmed CNAM propagation |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### ComplianceRequirement

**UI References:** Trust Center > Compliance page (requirements checklist)

**Relationships:**
- Belongs to one Account (many-to-one)
- Status may be automatically updated by the platform when a linked A2PCampaign, TollFreeRegistration, or VoiceRegistration reaches Approved status

**Notes:** This entity acts as a per-account compliance checklist, tracking the account's standing against each named regulatory requirement. Some requirements are auto-populated at account creation based on the account's country and the types of numbers it owns (e.g., an account with local numbers in the US automatically gets an A2P 10DLC requirement entry). The status field reflects the account's current posture: Not Started means no action has been taken, In Progress means a related application or registration is active but not approved, Completed means the requirement has been satisfied. Not Applicable covers requirements that do not apply to the account's profile (e.g., a voice-only account would mark A2P 10DLC as Not Applicable). The documentation_url links to the platform's compliance guidance article or the regulatory body's official documentation for that requirement.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| country | short_text | NN | ISO 3166-1 alpha-2 country code this requirement applies to (e.g., "US", "CA") |
| requirement_name | short_text | NN | Short canonical name of the requirement (e.g., "STIR/SHAKEN", "A2P 10DLC", "CRTC Compliance", "GDPR") |
| requirement_description | text | | Full prose description of what this requirement entails and why it applies to the account |
| status | enum(Completed, In Progress, Not Started, Not Applicable) | NN | Account's current compliance posture against this requirement; default Not Started |
| documentation_url | url | | Link to platform help article or official regulatory documentation for this requirement |
| due_date | date | | Deadline by which this requirement must be satisfied, if applicable |
| completed_at | timestamp_tz | | Timestamp when this requirement was marked Completed |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### ComplianceApplication

**UI References:** Trust Center > Compliance page (applications section)

**Relationships:**
- Belongs to one Account (many-to-one)
- Logically related to ComplianceRequirement: an application's approval may trigger a requirement's status to change to Completed, but the FK is not enforced at the database layer to preserve flexibility

**Notes:** ComplianceApplication tracks discrete filings submitted to external regulatory bodies or carrier registration systems that do not fit neatly into the specialized A2PCampaign, TollFreeRegistration, or VoiceRegistration entities. Examples include STIR/SHAKEN certificate requests, brand-level 10DLC registrations with The Campaign Registry (which precede campaign registration), and Ofcom registrations for UK operations. The external_reference_id stores the reference number assigned by the receiving body and is essential for support escalations and status lookups. Certificates carry an expires_at timestamp that drives renewal reminder logic. The Expired status is set automatically by the platform when expires_at passes without renewal.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| application_name | short_text | NN | Human-readable name identifying this filing within the platform |
| application_type | short_text | NN | Classification of the application (e.g., "STIR/SHAKEN Certificate", "10DLC Brand Registration", "Ofcom Registration") |
| country | short_text | NN | ISO 3166-1 alpha-2 country code of the regulatory jurisdiction |
| status | enum(Draft, Submitted, Under Review, Approved, Rejected, Expired) | NN | Current application status; default Draft |
| submitted_at | timestamp_tz | | Timestamp when the application was formally submitted to the external body |
| reviewed_at | timestamp_tz | | Timestamp when the external body completed its review (approval or rejection) |
| expires_at | timestamp_tz | | Expiration timestamp for time-limited approvals such as STIR/SHAKEN certificates |
| rejection_reason | text | | Explanation provided by the regulatory body or carrier when status is Rejected |
| external_reference_id | short_text | | Reference number assigned by the carrier, TCR, or regulatory body for this filing |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### ComplianceAddress

**UI References:** Trust Center > Business Info page (addresses section), Trust Center > Compliance page (address selection in registration forms)

**Relationships:**
- Belongs to one Account (many-to-one)
- Referenced by VoiceRegistration, A2PCampaign, and TollFreeRegistration as the filing address; the reference is by convention at the application layer, not enforced as a DB-level FK on the registration entities, because registrations capture an address snapshot at submission time
- Multiple addresses per account are permitted to support organizations with headquarters and branch offices

**Notes:** Verified addresses carry regulatory weight: carriers and regulatory bodies may require proof of a physical business presence at the submitted address. The verification_method field captures how the address was confirmed (e.g., "Utility Bill", "Bank Statement", "Google Business Profile", "Manual Review"). Once verified, is_verified = true and verified_at is set. Address verification is not renewable on a schedule, but if the address changes, the record should be updated and re-verified rather than mutated in place. The label field is the user-assigned identifier for distinguishing multiple locations (e.g., "Headquarters", "Branch Office - Chicago").

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| label | short_text | NN | User-assigned display name for this address (e.g., "Headquarters", "Branch Office - NYC") |
| address_line1 | short_text | NN | Primary street address |
| address_line2 | short_text | | Suite, floor, unit, or secondary address component |
| city | short_text | NN | City |
| state | short_text | NN | State or province abbreviation |
| zip | short_text | NN | Postal or ZIP code |
| country | short_text | NN | ISO 3166-1 alpha-2 country code; default "US" |
| is_verified | boolean | NN | True if this address has been confirmed through an accepted verification method; default false |
| verification_method | short_text | | Method used to verify the address (e.g., "Utility Bill", "Bank Statement", "Google Business Profile") |
| verified_at | timestamp_tz | | Timestamp when verification was confirmed |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

## Regulatory Landscape & Data Sensitivity

### Why This Data Exists: FCC and Carrier Mandates

The entities in this shard exist because of binding regulatory obligations imposed on telephone service users, not merely platform design choices. The FCC's STIR/SHAKEN mandate (47 CFR Part 64, Subpart CC) requires originating carriers to digitally sign calls with attestation tokens; businesses that wish to achieve Full (A) attestation must register with their originating carrier and maintain current business identity records. The FCC's A2P 10DLC requirement is carrier-enforced rather than directly FCC-mandated, but it derives from the FCC's directive that carriers implement robocall mitigation programs, which in practice means all major US carriers require campaign registration through The Campaign Registry before allowing commercial SMS at any meaningful volume. Toll-free verification is similarly carrier-enforced through aggregators. Non-compliance in any of these tracks results in direct service degradation: calls labeled as spam, SMS messages silently filtered, or number suspension. The compliance data exists because the platform is the system of record for these registrations and must track their status, history, and renewal obligations on behalf of every tenant account.

### Fields Requiring Encryption at Rest

The following fields contain sensitive identifiers that warrant encryption at rest independent of transport-layer protection:

- **BusinessInfo.ein** — The Employer Identification Number is a federal tax identifier that uniquely identifies a US legal entity. Exposure of an EIN enables identity fraud, fraudulent tax filings, and unauthorized credit applications in the business's name. It must be encrypted at rest using a platform-managed key, with access logged.
- **VoiceRegistration.ein** — This is a snapshot copy of the EIN as submitted in the STIR/SHAKEN filing. It requires the same encryption treatment as BusinessInfo.ein because it is independently stored and may outlive the parent BusinessInfo record if that record is updated.
- Any carrier API credentials or signing certificates associated with STIR/SHAKEN (not modeled as first-class entities in this shard but stored in ComplianceApplication or as platform configuration) should be treated as secrets and stored in a secrets manager rather than in the primary database.

Fields that do not require encryption but do require access control include contact email addresses, phone numbers, and business addresses, all of which are subject to GDPR, CCPA, and equivalent privacy regulations if the account operates in the relevant jurisdictions.

### Audit Trail Requirements: The VoiceRegistrationHistory Pattern

VoiceRegistrationHistory demonstrates the audit trail pattern that compliance data requires. Because STIR/SHAKEN attestation is an ongoing obligation with annual re-verification, the platform must be able to produce a complete chronological record of every status transition, submission, and verification event for a given registration. This serves three operational purposes: (1) internal troubleshooting when a carrier disputes the registration timeline, (2) regulatory audit response if the FCC or a carrier requests proof of continuous compliance, and (3) customer-facing transparency in the UI timeline view.

The append-only constraint on VoiceRegistrationHistory rows is intentional: compliance audit logs must not be mutated or deleted. This pattern should be extended to A2PCampaign and TollFreeRegistration if those entities accumulate complex lifecycles with multiple resubmission cycles, although the current model captures their status history through the status, rejection_reason, submitted_at, and approved_at fields on the entity itself rather than through a separate history table. As the platform matures, dedicated history tables for A2PCampaign and TollFreeRegistration may be warranted.

### BusinessInfo as the Root of All Registrations

BusinessInfo is the prerequisite entity for all compliance registrations in this domain. This design reflects the actual carrier registration process: before submitting a campaign, a brand, or a STIR/SHAKEN application, the submitting party must establish a verified business identity. In the platform, this is enforced as a UI gate (compliance registration forms are locked until BusinessInfo is complete) and should also be enforced at the API layer (registration creation endpoints should return an error if the account's BusinessInfo is incomplete).

The relationships flow outward from BusinessInfo in a tree pattern: BusinessInfo anchors AuthorizedContacts (the humans accountable for the registrations), which in turn are referenced in carrier submissions. VoiceRegistration duplicates the EIN and address fields from BusinessInfo rather than foreign-keying them, because each registration is a point-in-time snapshot of what was filed — if the business later corrects its address in BusinessInfo, the historical filing must preserve the original values. A2PCampaign and TollFreeRegistration similarly carry business_name as a snapshot field for the same reason. This snapshot-at-submission pattern is a compliance requirement, not a denormalization deficiency.

ComplianceAddress extends BusinessInfo's address by supporting multiple verified physical locations, which is necessary for multi-location businesses that may register different numbers under different office addresses.

### Status Lifecycle Patterns

A2PCampaign, TollFreeRegistration, and VoiceRegistration all share a family of status lifecycle patterns, though each has a distinct set of states reflecting their respective registration processes:

**A2PCampaign** follows: Draft → Pending → Approved or Rejected. From Rejected, a corrected campaign re-enters Pending. Approved campaigns can be Suspended by a carrier following abuse reports; Suspended campaigns must be resubmitted (re-enter Pending). This lifecycle means Approved is not a terminal state.

**TollFreeRegistration** follows: Not Registered → Draft → Pending → Approved or Rejected. Not Registered is the initial state for accounts that have toll-free numbers but have not yet begun verification. This distinguishes passive non-compliance from active but incomplete filings.

**VoiceRegistration** follows: Not Registered → Pending → Approved or Rejected. Approved registrations remain in Approved status across annual re-verifications unless the re-verification fails, at which point the status may return to Pending or Rejected. The VoiceRegistrationHistory table is the mechanism for tracking these annual cycles without mutating the primary status record until a determination is made.

**ComplianceApplication** has the broadest lifecycle: Draft → Submitted → Under Review → Approved, Rejected, or Expired. Expired is unique to this entity and covers certificates or time-limited approvals that lapse without renewal. The Rejected state allows for resubmission, which creates a new ComplianceApplication record rather than mutating the rejected one, preserving the rejection record for audit purposes.

The common pattern across all of these is that status transitions are driven by asynchronous external events (carrier decisions, regulatory body reviews) rather than by synchronous user actions. The platform must therefore support webhook-driven or polling-driven status updates from carrier APIs, and the UI must communicate the asynchronous nature of these workflows clearly to users (e.g., "Pending carrier review — typically 1-7 business days").
