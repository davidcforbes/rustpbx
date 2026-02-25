# 09 — System & Operations

This shard documents the system-internal entities that support platform operations, integration monitoring, real-time call state, and policy enforcement. These entities are not primarily user-facing in the way that contacts or campaigns are, but they are foundational to the platform's reliability, observability, and live responsiveness.

The domain divides cleanly into two tiers. **Persistent operational data** — ApiLogEntry, MonitoringEvent, AccountVariable, and FrequencyLimit — is written to the relational database and retained according to defined policies. These records support debugging, auditing, configuration, and compliance. **Ephemeral real-time state** — Presence, Location, and ActiveCall — reflects the current moment and has no value once it is stale. These entities are candidates for in-memory or cache-tier storage and are replaced or deleted rather than appended to. The distinction drives different storage, indexing, and lifecycle strategies and is discussed in detail in the Ephemeral vs. Persistent Storage section at the end of this document.

---

### ApiLogEntry

**UI References:** Flows > API Logs page

**Relationships:**
- Many-to-one with Account (each log entry belongs to one account)

**Notes:** Append-only log table. A retention policy (recommended 90 days) should govern automated deletion of old rows. Request and response bodies may be truncated at a configurable byte limit (e.g., 64 KB) before storage to prevent unbounded growth. All authorization headers and bearer tokens must be stripped or redacted before the request_headers field is written. This table is write-heavy and read-rarely; indexing should favor account_id + timestamp range scans for the UI log view. No updates or deletes are performed by application logic.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Unique identifier for this log entry |
| account_id | uuid | FK(Account), NN | Account that owns the integration producing this log entry |
| timestamp | timestamp_tz | NN | Wall-clock time when the request was received or initiated |
| source | short_text | NN, MAX(100) | Origin system label (e.g., "Zapier", "Custom Webhook", "REST API", "Salesforce") |
| method | enum(GET, POST, PUT, DELETE, PATCH) | NN | HTTP method used in the request |
| endpoint | text | NN | Request URL or normalized path (e.g., "/api/v1/calls") |
| request_headers | json | | HTTP request headers as a key-value object; auth tokens stripped before storage |
| request_body | json | | Request payload; truncated if it exceeds the configured size limit |
| response_code | integer | NN | HTTP response status code returned (e.g., 200, 400, 500) |
| response_body | json | | Response payload; truncated if it exceeds the configured size limit |
| response_size_bytes | integer | | Full byte size of the response body before any truncation |
| duration_ms | duration_ms | NN | Total request processing time in milliseconds from receipt to response sent |
| activity_description | short_text | MAX(255) | Human-readable summary of what the request accomplished (e.g., "Webhook received: new call event") |
| error_message | text | | Detailed error description if response_code >= 400; null on success |
| created_at | timestamp_tz | NN | Row creation timestamp; typically equal to timestamp but set at insert time |

---

### AccountVariable

**UI References:** Flows > Variables page (stub)

**Relationships:**
- Many-to-one with Account (each variable belongs to one account)
- Referenced by call flows, scripts, and notification templates via {{variable_name}} substitution

**Notes:** Variable names must be unique within an account and should be restricted to alphanumeric characters and underscores (no spaces or special characters) to ensure safe template substitution. Variables where is_secret = true must have their value stored as encrypted_text and the plaintext must never be returned in API responses or rendered in the UI; the UI should display "••••••" in place of the value. Secret variables can be updated (replaced) but not read back. Template engines reference these variables at render time, so value changes take effect immediately on the next invocation.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Unique identifier for this variable |
| account_id | uuid | FK(Account), NN | Account that owns this variable |
| name | short_text | NN, UQ(account_id), MAX(100) | Variable name used in template substitution (e.g., "support_email"); alphanumeric and underscore only |
| value | text | NN | Current value of the variable; stored as encrypted_text if is_secret = true |
| description | text | | Human-readable explanation of what this variable is used for and where it appears |
| is_secret | boolean | NN | If true, the value is encrypted at rest and masked in all API and UI responses; default false |
| created_at | timestamp_tz | NN | Timestamp when this variable was first created |
| updated_at | timestamp_tz | NN | Timestamp of the most recent value or metadata change |

---

### FrequencyLimit

**UI References:** No direct UI — system-internal policy enforcement

**Relationships:**
- Many-to-one with Account (limits are scoped to accounts or sub-resources within accounts)

**Notes:** This table is a direct match to the existing `rustpbx_frequency_limits` table. Rows are uniquely identified by (policy_id, scope, window_start) — no two rows should represent the same policy scope for the same time window. Current_count is incremented atomically (optimistic locking or database-level atomic increment) to prevent race conditions under concurrent request load. The window_end timestamp defines when the counter expires; rows with window_end in the past are candidates for garbage collection by a background job. Policy identifiers (policy_id) are defined in application configuration, not the database, and reference well-known names like "api_rate_limit" or "sms_daily_limit".

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Unique identifier for this frequency limit counter row |
| account_id | uuid | FK(Account), NN | Account to which this limit applies |
| policy_id | short_text | NN, MAX(100) | Policy name from application configuration (e.g., "api_rate_limit", "sms_daily_limit") |
| scope | short_text | NN, MAX(255) | Scope key within the policy (e.g., account UUID, user UUID, or client IP address) |
| limit_type | enum(Sliding Window, Fixed Window, Token Bucket) | NN | Algorithm used to enforce this limit |
| max_count | integer | NN | Maximum allowed event count within the window before requests are denied |
| current_count | counter | NN | Atomically incremented count of events that have occurred in this window; default 0 |
| window_start | timestamp_tz | NN | Beginning of the current enforcement window |
| window_end | timestamp_tz | NN | End of the current enforcement window; rows past this time are expired |
| last_incremented_at | timestamp_tz | | Timestamp of the most recent atomic increment to current_count |

---

### MonitoringEvent

**UI References:** Coaching page, Reports > Agent Performance

**Relationships:**
- Many-to-one with Account
- Many-to-one with CallRecord via call_id (the call being supervised)
- Many-to-one with User via monitor_user_id (the supervisor)
- Many-to-one with User via monitored_agent_id (the agent being observed)

**Notes:** This table is a direct match to the existing `rustpbx_monitoring_events` table, extended with additional fields for the 4iiz UI. Each row represents a discrete monitoring action event, not a continuous session — a supervisor starting and stopping a barge session generates at minimum a Start and a Stop event. Mode changes (e.g., switching from Listen to Whisper mid-call) generate a Mode Change event so the full supervision history for a call can be reconstructed. Duration_secs is computed as the interval between started_at and ended_at and may be null for events that have not yet ended.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Unique identifier for this monitoring event |
| account_id | uuid | FK(Account), NN | Account under which the monitoring action occurred |
| session_id | short_text | NN, MAX(255) | SIP Call-ID or session identifier of the call being monitored |
| call_id | uuid | FK(CallRecord) | Reference to the CallRecord for this call, if available at event time |
| monitor_user_id | uuid | FK(User), NN | User (supervisor) who initiated or performed the monitoring action |
| monitored_agent_id | uuid | FK(User) | User (agent) whose call was being supervised; null if agent cannot be identified |
| event_type | enum(Start, Stop, Mode Change) | NN | Type of monitoring action this event represents |
| monitor_mode | enum(Listen, Whisper, Barge) | NN | Monitoring mode active at the time of this event |
| started_at | timestamp_tz | NN | When the monitoring session or mode segment began |
| ended_at | timestamp_tz | | When the monitoring session or mode segment ended; null if still active |
| duration_secs | duration_sec | | Elapsed time between started_at and ended_at; null until ended_at is set |
| created_at | timestamp_tz | NN | Row creation timestamp |

---

### Presence

**UI References:** Agent status indicators, Queue agent availability, Coaching page

**Relationships:**
- Many-to-one with Account (optional — presence may exist for unregistered SIP identities)
- Many-to-one with User via user_id (when the identity maps to a known platform user)
- One-to-one with ActiveCall via current_call_id (when status is On Call)

**Notes:** This table is a direct match to the existing `presence_states` table. This is a mutable real-time state table, not an append-only log. Each identity (SIP AOR or user identifier) has exactly one row that is updated in place whenever state changes. Historical presence state transitions are tracked separately in AgentStateLog (documented in shard 02). The status field drives agent availability for queue routing: only agents with status = Available are eligible to receive queued calls. The Coaching page uses this table to show which agents are currently active and reachable for monitoring.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| identity | short_text | PK, NN, MAX(255) | SIP Address of Record (AOR) or platform user identifier; primary key and lookup key |
| account_id | uuid | FK(Account) | Account this presence record belongs to; null for anonymous or system SIP endpoints |
| user_id | uuid | FK(User) | Platform User this presence record maps to; null if the identity is not a known user |
| status | enum(Available, On Call, After Call Work, Offline, Break, DND) | NN | Current availability state of the agent or endpoint |
| note | short_text | MAX(255) | Optional status message set by the user (e.g., "In meeting until 3pm") |
| activity | short_text | MAX(255) | Current activity description as reported by the endpoint or set by workflow logic |
| current_call_id | uuid | | Active call identifier if status = On Call; null otherwise |
| last_updated | timestamp_tz | NN | Timestamp of the most recent state change; used to detect stale presence records |

---

### Location

**UI References:** No direct UI — system-internal for call routing

**Relationships:**
- Many-to-one with Account (implied via realm/username)
- Many-to-one with User (implied via username + realm lookup)

**Notes:** This table is a direct match to the existing `rustpbx_locations` table, extended with additional fields for transport and source address tracking. A single AOR (Address of Record) may have multiple active Location rows representing forked registrations across multiple devices (e.g., a desk phone and a softphone registered simultaneously). Registrations are time-limited; the expires field defines when the binding is no longer valid. The SIP registrar refreshes Location rows on re-REGISTER and deletes them on REGISTER with Expires: 0. A background garbage collection job should purge rows where expires is in the past.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Unique identifier for this registration binding |
| aor | short_text | NN, MAX(255) | Address of Record — the stable SIP URI representing the user or endpoint (e.g., sip:1001@example.com) |
| username | short_text | NN, MAX(100) | SIP username extracted from the AOR |
| realm | short_text | NN, MAX(255) | SIP domain or realm under which this registration is valid |
| destination | text | NN | Contact URI where the registering device can be reached (e.g., sip:1001@192.168.1.10:5060) |
| expires | timestamp_tz | NN | Absolute timestamp after which this registration binding is no longer valid |
| user_agent | short_text | MAX(255) | Value of the SIP User-Agent header from the REGISTER request |
| supports_webrtc | boolean | NN | True if the registering endpoint supports WebRTC (WSS transport); default false |
| source_ip | short_text | MAX(45) | IP address of the device that sent the REGISTER request (IPv4 or IPv6) |
| source_port | integer | | UDP/TCP port of the device that sent the REGISTER request |
| transport | enum(UDP, TCP, TLS, WSS) | NN | Transport protocol used by the registering device; default UDP |
| created_at | timestamp_tz | NN | Timestamp when this registration binding was first created |
| updated_at | timestamp_tz | NN | Timestamp of the most recent re-REGISTER refresh |

---

### ActiveCall

**UI References:** Reports > Real Time page, Coaching page, Queue live metrics

**Relationships:**
- Many-to-one with Account
- Many-to-one with User via agent_id (assigned agent)
- Many-to-one with Queue via queue_id (if routed through a queue)
- Many-to-one with TrackingSource via source_id
- Many-to-one with TrackingNumber via tracking_number_id
- One-to-one with MonitoringEvent via current monitoring session (when is_monitored = true)
- Transitions to CallRecord on call completion (same id used for correlation)

**Notes:** This entity represents the real-time in-progress state of a call and is ephemeral by design. It should reside in an in-memory store (Redis or equivalent) rather than the primary relational database to support the low-latency polling required by the Real Time dashboard and Coaching page. When a call ends, the relevant data is written to the permanent CallRecord entity and the ActiveCall entry is deleted. Duration_secs and wait_time_secs are computed fields derived from started_at or answered_at relative to the current time; they are not stored as static values. Queue-level real-time metrics (calls waiting, average wait time, service level percentage, abandoned count, agents online) are derived by aggregating over ActiveCall rows and Presence records and are not stored as separate entities.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Unique identifier for this active call; reused as CallRecord.id on completion for correlation |
| account_id | uuid | FK(Account), NN | Account under which this call is occurring |
| call_id | short_text | NN, UQ, MAX(255) | SIP Call-ID header value; unique across all active calls in the system |
| caller_name | short_text | MAX(100) | Display name of the calling party from SIP From header or CRM lookup |
| caller_number | e164 | NN | E.164 formatted phone number of the calling party |
| callee_number | e164 | NN | E.164 formatted phone number of the called party |
| agent_id | uuid | FK(User) | Agent currently handling the call; null if not yet assigned or in IVR |
| queue_id | uuid | FK(Queue) | Queue the call entered, if any; null for direct-routed calls |
| source_id | uuid | FK(TrackingSource) | Attribution source for this call |
| tracking_number_id | uuid | FK(TrackingNumber) | Tracking number that was dialed |
| direction | enum(Inbound, Outbound, Internal) | NN | Whether the call originated from outside, was placed by an agent, or is internal |
| status | enum(Ringing, Active, On Hold, Transferring, Wrapping) | NN | Current leg status of the call |
| started_at | timestamp_tz | NN | Timestamp when the call was first initiated (SIP INVITE received or sent) |
| answered_at | timestamp_tz | | Timestamp when the call was answered by an agent; null if still ringing or in IVR |
| wait_time_secs | duration_sec | | Elapsed time from started_at to answered_at; null until the call is answered |
| duration_secs | duration_sec | | Elapsed time from answered_at (or started_at if unanswered) to the current moment; computed, not stored |
| is_monitored | boolean | NN | True if a supervisor is currently listening to this call; default false |
| monitor_mode | enum(Listen, Whisper, Barge) | | Active monitoring mode if is_monitored = true; null otherwise |

---

## Ephemeral vs. Persistent Storage

### Classification by Lifecycle

The entities in this shard fall into two distinct storage tiers based on their temporal nature.

**Persistent operational data** — entities that accumulate over time and support auditing, debugging, configuration, and historical analysis — must be stored in the primary relational database with full ACID guarantees. This group includes:

- **ApiLogEntry**: Append-only integration audit log. Rows are written once and never modified. Retained for a configurable period (recommended 90 days) before automated deletion.
- **AccountVariable**: Long-lived configuration state. Variables are created, updated, and occasionally deleted by users. The full history of value changes is not required, only the current value.
- **FrequencyLimit**: Enforcement counters tied to time windows. Rows within an active window are mutated (counter incremented); rows in expired windows are candidates for deletion.
- **MonitoringEvent**: Append-only event log. Each supervision action generates one or more rows that are retained as part of the account's call quality and compliance record.

**Ephemeral real-time state** — entities that reflect the current moment and lose all value once stale — are candidates for in-memory or cache-tier storage. This group includes:

- **Presence**: A single mutable row per identity reflecting the agent's current status. State changes overwrite the existing row; the table is never appended to (that is the role of AgentStateLog in shard 02).
- **Location**: SIP registration bindings that expire on a timer and are refreshed or deleted by the SIP registrar. The authoritative state is whatever the endpoint last registered.
- **ActiveCall**: The full real-time state of an in-progress call. Exists only for the duration of the call. On completion, data is promoted to a permanent CallRecord and the ActiveCall entry is removed.

### Storage Tier Recommendations

For **Presence** and **ActiveCall**, an in-memory store such as Redis is the preferred storage tier. These entities are queried at very high frequency by the Real Time dashboard, Coaching page, and queue routing engine, all of which require sub-millisecond reads. Redis hash structures map naturally to both entities. Pub/sub or keyspace notifications can drive live UI updates without polling.

For **Location**, the SIP registrar maintains the authoritative state. Redis with TTL-based key expiration is a natural fit: each registration binding is stored with a TTL equal to the registration expiry interval, and Redis garbage-collects expired entries automatically. If a relational store is used instead, a background job must periodically delete rows where expires is in the past.

For **FrequencyLimit**, Redis atomic increment (INCR) with key TTL is the canonical implementation for sliding-window and fixed-window rate limiting. However, if frequency limits must be durable across process restarts (e.g., daily SMS caps that must survive a service restart), a relational table with atomic counter semantics (using database-level locking or optimistic concurrency) is appropriate. The existing `rustpbx_frequency_limits` table uses this relational approach.

For **ApiLogEntry** and **MonitoringEvent**, a relational database is required. These are append-only audit logs. Partitioning by account_id and timestamp range is advisable at scale to maintain query performance as the tables grow.

For **AccountVariable**, the relational database is the correct tier. Values are small, read frequency is moderate (at call flow invocation time), and durability and encryption-at-rest support for secret values are essential requirements.

### ActiveCall to CallRecord Lifecycle

The ActiveCall entity acts as a staging area for in-progress call data. Its lifecycle is:

1. An ActiveCall row is created when a new SIP session is initiated (on receipt of an INVITE or on origination of an outbound call). The id is assigned at this point and will be reused as the CallRecord id for correlation.
2. The row is updated in real time as the call progresses: status transitions (Ringing → Active → On Hold), agent assignment, answered_at timestamp, and monitoring flags.
3. When the call ends (BYE or CANCEL received, or timeout), a CallRecord row is written to the relational database using the same id, capturing the final disposition, duration, recordings, and attribution data.
4. The ActiveCall entry is deleted from the in-memory store.

This two-phase commit pattern ensures that no call data is lost on normal termination. For abnormal terminations (process crash, network partition), a reconciliation job should scan for CallRecord rows that are missing for any session_id that appears in the SIP CDR log.

### Retention Policies for Log-Type Entities

**ApiLogEntry** should be retained for 90 days by default, configurable per account. Deletion should be performed by a scheduled background job that batches deletes by account_id and timestamp range to avoid lock contention. Accounts on compliance plans may require longer retention (e.g., 1 year).

**MonitoringEvent** rows should be retained for the same period as the associated CallRecord, typically 1 year. If the CallRecord is deleted (e.g., due to data subject deletion requests under privacy regulations), the associated MonitoringEvent rows should also be deleted.

**FrequencyLimit** rows with window_end in the past serve no enforcement function and should be deleted by a garbage collection job. A daily sweep is sufficient; rows that expired more than 24 hours ago can be safely removed.

### Garbage Collection for Location and FrequencyLimit

**Location** registrations that have passed their expires timestamp are invalid and must not be used for call routing. The SIP registrar should ignore expired bindings during location lookup even if the garbage collection job has not yet run. The GC job should run frequently (every 5 minutes) to keep the table size bounded, especially in environments with large numbers of frequently re-registering devices.

**FrequencyLimit** windows that have expired (window_end < now) are stale counters. The GC job can delete them safely because any new request in a new window will create a fresh row. GC frequency for FrequencyLimit can be lower than for Location (hourly or daily) since stale rows do not affect correctness — they are simply dead weight in the table.
