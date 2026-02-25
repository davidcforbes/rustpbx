# 02 — People & Accounts

> **Shard:** 02 of 09
> **Status:** Pre-analysis / Data element reconciliation
> **Date:** 2026-02-25
> **Parent index:** [00-data-dictionary-index.md](00-data-dictionary-index.md)

---

## Overview

This shard defines the entities that represent the human and organizational layer of the 4iiz platform. Every entity in the system is ultimately scoped to an **Account**, making it the multi-tenant root of the entire data model. **Users** are the people who interact with the platform — both administrators who configure it and agents who handle calls. **AgentStateLog** provides the fine-grained availability timeline needed to calculate agent performance metrics. **Notification** delivers real-time alerts inside the console UI.

The contact management entities — **ContactList**, **ContactListMember**, **BlockedNumber**, **DncEntry**, and **DntEntry** — govern which external phone numbers may receive calls and messages from (or send them to) the system. These are distinct from call records or routing logic; they represent policy decisions about reachability and compliance.

Together, the entities in this shard underpin authentication, authorization, multi-tenancy, real-time agent monitoring, and compliance gating for all outbound communication.

---

## Entities

### Account

**UI References:** Global (account selector in header), all pages filtered by account context

**Relationships:**
- One Account has zero or one parent Account (self-referential; null for top-level accounts)
- One Account (Agency) owns zero or many child Accounts (sub-accounts)
- One Account owns zero or many Users
- One Account owns zero or many Notifications
- One Account owns zero or many ContactLists
- One Account owns zero or many BlockedNumbers
- One Account owns zero or many DncEntries
- One Account owns zero or many DntEntries
- One Account is the tenant root for all other entities in the system

**Notes:** The Account entity is the multi-tenant isolation boundary. All data queries must be scoped by account_id. Agency accounts may read and manage all of their direct sub-accounts but not accounts belonging to other agencies. The hierarchy is intentionally limited to two levels (agency → sub-account) to keep permission resolution simple and auditable. A Standard account with no parent_account_id is a standalone tenant. The slug is used for subdomain routing or white-label URL segments and must be globally unique across all accounts, not just within a parent.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| name | short_text | NN, MAX(255) | Human-readable display name (e.g., "Diener Law") |
| account_type | enum(Agency, Standard) | NN | Determines whether this account can own sub-accounts |
| parent_account_id | uuid | FK(Account) | References the owning agency account; null for top-level accounts |
| slug | short_text | UQ, MAX(63) | URL-safe lowercase identifier used in routing and white-label URLs |
| timezone | short_text | NN, MAX(64) | IANA timezone name; default "America/New_York"; affects schedule evaluation and report display |
| status | enum(Active, Suspended, Closed) | NN | Lifecycle state; Suspended blocks logins and call processing; Closed is soft-delete |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### User

**UI References:** Global (user menu, session header), Reports > Agent Performance, Coaching page, Queue agent lists, CallDetailPanel (agent assignment tab)

**Relationships:**
- Many Users belong to one Account
- One User generates zero or many AgentStateLogs
- One User receives zero or many Notifications
- One User is referenced as added_by on DncEntry and DntEntry
- One User is referenced as the assigned agent on CallRecord (see shard 01)
- One User may be assigned to zero or many Queues via QueueAgent (see shard 04)

**Notes:** The User entity covers both platform administrators and call center agents; the role field distinguishes them. Agents appear in queue assignments, coaching views, and call records. The initials field is derived from display_name at creation time (first letter of first name + first letter of last name) but can be overridden manually. The initials + avatar_color combination renders as a small badge throughout the UI wherever an agent name would appear. Soft deletion is preferred over hard deletion: set is_active = false to deactivate an agent while preserving all historical references. Global uniqueness on email is required to support cross-account SSO and password reset flows.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account; all reads and writes are scoped to this value |
| username | short_text | NN, UQ(account_id), MAX(64) | Login username; unique within an account |
| email | email | NN, UQ, MAX(254) | Email address; globally unique across all accounts for password reset and SSO |
| password_hash | encrypted_text | NN | Bcrypt or Argon2 hash of the user's password; never stored or returned in plaintext |
| display_name | short_text | NN, MAX(128) | Full name shown in UI (e.g., "Jordan Davis") |
| initials | short_text | MAX(3) | Two- or three-letter initials derived from display_name or set manually (e.g., "JD") |
| avatar_color | hex_color | MAX(7) | CSS hex color for the initials badge background (e.g., "#00bcd4") |
| role | enum(Admin, Agent, Supervisor) | NN | Authorization role; Admin has full account access; Supervisor can view all agents; Agent is scoped to own activity |
| phone | e164 | | Agent's direct phone number for escalation or callback |
| is_active | boolean | NN | When false, the user cannot log in; historical records are preserved |
| last_login_at | timestamp_tz | | Timestamp of the most recent successful login; null if never logged in |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### AgentStateLog

**UI References:** Reports > Agent Performance (availability %, ACW time, idle time), Reports > Real Time (live agent status grid), Coaching page (timeline view)

**Relationships:**
- Many AgentStateLogs belong to one User (agent)
- Each AgentStateLog is implicitly scoped to an account via the agent's account_id

**Notes:** This is a high-volume, append-only table. Each row represents a single state transition event — there is no update to existing rows except to backfill duration_secs. When an agent transitions to a new state, the pipeline calculates duration_secs for the previous open row (changed_at of new row minus changed_at of previous row) and writes it back. Reports aggregate these rows to compute availability percentage (time in Available / total logged-in time), ACW percentage (time in After Call Work / total logged-in time), and idle time (time in Available with no call). Retention policy is likely to mirror call record retention for the account. Do not use this table for real-time presence — use the Presence entity in shard 09 for that.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| agent_id | uuid | FK(User), NN | The agent whose state changed |
| status | enum(Available, On Call, After Call Work, Offline, Break) | NN | The new state the agent transitioned into |
| changed_at | timestamp_tz | NN | Exact time the state transition occurred |
| duration_secs | duration_sec | | How long the agent remained in this state; null until the next state change is recorded |
| reason | short_text | MAX(128) | Optional free-text or system-generated reason for the transition (e.g., "Break - Lunch", "Call ended") |
| created_at | timestamp_tz | NN | Row insertion time; may differ slightly from changed_at if written asynchronously |

---

### Notification

**UI References:** FilterBar (bell icon with unread count badge), Notification dropdown panel

**Relationships:**
- Many Notifications belong to one User (recipient)
- Many Notifications belong to one Account
- Each Notification optionally references one entity of any type via entity_type + entity_id (polymorphic reference)

**Notes:** Notifications are generated by the platform's event pipeline, not directly by user action. The event_type field is a freeform string key (e.g., "missed_call", "voicemail", "form_submission") that the UI uses to select the appropriate icon and copy template. The entity_type + entity_id pair provides a polymorphic link so the UI can navigate to the source record when the notification is clicked — for example, entity_type = "CallRecord" and entity_id = some call UUID would link to the call detail panel. The unread count badge in the FilterBar is driven by a count of rows where is_read = false for the current user. Marking all notifications read is a bulk update. Mock data shows approximately 4,950 total notifications, suggesting these accumulate rapidly and may require a retention or archival policy.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| user_id | uuid | FK(User), NN | The user who should receive this notification |
| account_id | uuid | FK(Account), NN | Owning account; used to scope queries without joining through User |
| event_type | short_text | NN, MAX(64) | Machine-readable event key that generated this notification (e.g., "missed_call", "voicemail", "form_submission") |
| title | short_text | NN, MAX(255) | Short human-readable headline displayed in the notification dropdown |
| body | text | | Optional longer description or context for the notification |
| entity_type | short_text | MAX(64) | Type name of the related entity for click-through navigation (e.g., "CallRecord", "FormRecord") |
| entity_id | uuid | | ID of the related entity; combined with entity_type to resolve the navigation target |
| is_read | boolean | NN | False until the user dismisses or opens the notification; drives the unread count badge |
| created_at | timestamp_tz | NN | Time the notification was generated |

---

### ContactList

**UI References:** Contacts > Contact Lists page, Bulk Messages (recipient selection dropdown), Reminders (target list selection)

**Relationships:**
- Many ContactLists belong to one Account
- One ContactList has zero or many ContactListMembers

**Notes:** ContactLists are named collections of phone numbers used to target bulk SMS campaigns, reminder sequences, or filter call logs. The member_count field is a denormalized cache of the count of ContactListMembers for the list; it must be kept consistent by the application when members are added or removed (increment/decrement on write). Deleting a ContactList should cascade to all ContactListMember rows. ContactLists are account-scoped and cannot be shared across accounts.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, UQ(account_id), MAX(255) | Display name for the list (e.g., "Q1 Lead Follow-Up") |
| description | text | | Optional free-text description of the list's purpose |
| member_count | counter | NN | Denormalized count of ContactListMembers; must equal the actual row count in ContactListMember for this list |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### ContactListMember

**UI References:** Contacts > Contact Lists page (member management drawer)

**Relationships:**
- Many ContactListMembers belong to one ContactList
- ContactListMember rows are deleted when their parent ContactList is deleted (cascade)

**Notes:** This is the junction table that associates individual phone numbers with a ContactList. A given phone number may appear in multiple distinct lists, but may appear in any single list only once (enforced by UQ(list_id, phone)). The contact_name field is optional metadata for display purposes and is not a reference to a normalized contact entity. Bulk import of members should validate E.164 format before insertion. The phone field should be stored in normalized E.164 format to ensure deduplication works correctly.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| list_id | uuid | FK(ContactList), NN | The parent ContactList this member belongs to |
| phone | e164 | NN, UQ(list_id), MAX(20) | Phone number in E.164 format; unique within the list |
| contact_name | short_text | MAX(128) | Optional human-readable name for display in the member list |
| added_at | timestamp_tz | NN | Time this phone number was added to the list |

---

### BlockedNumber

**UI References:** Contacts > Blocked Numbers page

**Relationships:**
- Many BlockedNumbers belong to one Account

**Notes:** When an inbound call arrives, the call processing pipeline checks the caller's number against BlockedNumber for the receiving account. If matched, the call is rejected or silently routed to voicemail depending on account configuration. The calls_blocked counter is incremented atomically by the call pipeline each time a blocked call is detected — it is not written by the UI. The last_blocked_at field is updated at the same time. The cnam field stores the result of a CNAM lookup performed at block time, used for display in the Blocked Numbers table. The number field must be E.164 and is unique per account — the same number can be blocked by different accounts independently.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account; block lists are not shared across accounts |
| number | e164 | NN, UQ(account_id), MAX(20) | The blocked caller number in E.164 format |
| cnam | short_text | MAX(64) | CNAM (Caller Name) result from carrier lookup; stored for display purposes |
| calls_blocked | counter | NN | Running total of calls rejected due to this block rule; incremented by the call pipeline |
| last_blocked_at | timestamp_tz | | Timestamp of the most recent call rejected by this rule; null if no calls have been blocked yet |
| created_at | timestamp_tz | NN | Time the block rule was created |
| updated_at | timestamp_tz | NN | Last modification time (e.g., when calls_blocked or last_blocked_at was updated) |

---

### DncEntry (Do Not Call)

**UI References:** Contacts > Do Not Call page

**Relationships:**
- Many DncEntries belong to one Account
- One DncEntry is optionally attributed to one User (added_by_id)

**Notes:** Compliance-critical entity. Before any outbound call is initiated — whether manual, automated, or dialed by SmartDialer — the destination number must be checked against the DncEntry table for the account. If a match is found, the call must be blocked and an appropriate log entry made. Bulk import (CSV upload) is a required capability and must validate E.164 format. The added_by_id provides an audit trail of who placed a number on the list, which may be required for regulatory compliance. Hard deletion of DncEntry rows should be avoided; if removal is needed, an audit log entry should be created. The number field is unique per account to prevent duplicate entries.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account; DNC lists are per-account |
| number | e164 | NN, UQ(account_id), MAX(20) | The phone number in E.164 format that must not be called |
| added_by_id | uuid | FK(User) | The User who added this entry; null if added by bulk import or system process |
| reason | short_text | MAX(255) | Optional free-text reason for the DNC entry (e.g., "Customer request", "TCPA complaint") |
| created_at | timestamp_tz | NN | Time the entry was created |

---

### DntEntry (Do Not Text)

**UI References:** Contacts > Do Not Text page

**Relationships:**
- Many DntEntries belong to one Account
- One DntEntry is optionally attributed to one User (added_by_id)

**Notes:** Analogous to DncEntry but governs outbound SMS. Before any outbound text message is sent — whether individual, bulk, or automated — the destination number must be checked against DntEntry for the account. The rejected_count counter is incremented by the SMS pipeline each time a message is blocked. The e164 field holds the canonically normalized form of the number and is used for lookups; the number field holds the display format as entered by the user or received from an opt-out reply (e.g., "8005551212" or "+1 (800) 555-1212"). Both fields are stored to preserve the original input for audit purposes while ensuring reliable lookup. Adding a number via SMS opt-out keyword (e.g., "STOP") should automatically create a DntEntry. Bulk import via CSV is required.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account; DNT lists are per-account |
| number | short_text | NN, MAX(32) | Phone number in display/input format as originally received or entered |
| e164 | e164 | NN, UQ(account_id), MAX(20) | Normalized E.164 form of the number; used for lookup deduplication |
| rejected_count | counter | NN | Running total of outbound texts blocked due to this DNT rule; incremented by the SMS pipeline |
| last_rejected_at | timestamp_tz | | Timestamp of the most recent outbound text blocked by this rule |
| added_by_id | uuid | FK(User) | The User who added this entry; null if added via SMS opt-out or bulk import |
| created_at | timestamp_tz | NN | Time the entry was created |

---

## Multi-Tenancy Considerations

### Data Isolation Model

Every entity in the 4iiz platform — with the sole exception of Account itself — carries an account_id foreign key. All application-layer queries must include `WHERE account_id = ?` as a mandatory predicate. There is no cross-account data sharing at the row level. This design requires that the API layer and service layer enforce account scoping on every read and write operation; it must not be left to query authors to remember to add the filter.

The Notification entity carries both user_id and account_id to allow efficient querying by either dimension without requiring a join through User. Similarly, AgentStateLog is implicitly scoped through agent_id but may warrant a denormalized account_id column for time-series query performance at scale.

### Agency Hierarchy

The Account entity supports a two-level hierarchy: an Agency account may own zero or many Standard sub-accounts. Agency users (role = Admin on an Agency account) can view and manage any sub-account that has parent_account_id pointing to their account. They cannot manage accounts owned by other agencies.

The hierarchy is intentionally flat (maximum depth of 2) to keep permission checks simple: to determine whether a user may access a resource, the check is either (1) user.account_id = resource.account_id (direct ownership) or (2) user.account.parent_account_id IS NULL AND user.account.account_type = 'Agency' AND resource.account.parent_account_id = user.account_id (agency access to sub-account). There is no recursive traversal required.

Sub-accounts do not inherit configuration from their parent agency account. Each sub-account maintains its own independent set of tracking numbers, routing rules, agents, and compliance lists. The agency relationship is purely for management access, billing consolidation, and reporting roll-up.

### Account Status and Access Control

The Account.status field gates access at the session authentication layer:

- **Active:** All features available; calls and messages processed normally.
- **Suspended:** Console logins are rejected; inbound calls may still be received (configurable) but outbound calls and SMS are blocked; billing disputes or usage violations are the typical trigger.
- **Closed:** Soft-deleted state; no logins, no call processing; data is retained for the contractual retention period before physical deletion.

User.is_active governs individual user access independently of account status. A user can be deactivated (is_active = false) without affecting other users on the same account.

### Compliance List Scoping

BlockedNumber, DncEntry, and DntEntry are all scoped per account. There is no system-wide shared DNC list in the data model; if a platform-level DNC registry is required (e.g., integration with the national DNC registry), it should be implemented as a separate lookup service rather than rows in these tables. This keeps the per-account compliance tables lean and ensures that a number blocked or DNC-listed by one customer does not affect any other customer's account.

### Agency-Level Reporting

When an Agency user views aggregate reports, the report query must union or group across all sub-account IDs that have parent_account_id = the agency's account ID. The CallDailySummary and related analytics entities (shard 07) should include account_id so that agency-level roll-up queries can use indexed account_id lookups rather than joining through a subquery.
