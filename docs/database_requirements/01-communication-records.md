# 01 — Communication Records

## Overview

This shard documents the core communication record entities for the 4iiz call tracking platform. These entities represent every discrete communication event that flows through the system: phone calls, SMS/MMS messages, web form submissions, live chats, faxes, video sessions, and data exports. The CallRecord is the highest-volume and most relationship-rich entity in the entire schema; all other entities in this shard either extend it, exist alongside it as peer activity types, or support its operational lifecycle (transcription, AI analysis, keyword detection, visitor attribution). Together, these entities back the Activities section of the UI, all ten tabs of the CallDetailPanel, and the majority of the Reports section. Designing these tables correctly for high-throughput ingest, per-account partitioning, and read-heavy reporting queries is the primary performance challenge for this domain.

---

## Entities

### CallRecord

**UI References:** Activities > Calls page, CallDetailPanel (all 10 tabs), all Reports pages

**Relationships:**
- Account → CallRecord (many-to-one): each call belongs to exactly one account
- TrackingNumber → CallRecord (many-to-one): the tracking number dialed, via source_number_id
- TrackingSource → CallRecord (many-to-one): marketing attribution source, via source_id
- User → CallRecord (many-to-one): agent assigned to the call, via agent_id
- Queue → CallRecord (many-to-one): queue that handled the call, via queue_id
- Trigger → CallRecord (many-to-one): automation that fired during the call, via automation_id
- CallRecord → Tag (many-to-many): via CallTag junction table
- CallRecord → CallFlowEvent (one-to-many): ordered sequence of IVR/routing events
- CallRecord → CallVisitorSession (one-to-one): web visitor session captured via DNI
- CallRecord → CallTranscriptionSegment (one-to-many): speaker-labeled transcript segments
- CallRecord → CallAISummary (one-to-many): AI-generated summary documents of various types
- CallRecord → TextMessage (one-to-many): SMS messages linked to the same contact and call
- CallRecord → CallKeywordHit (one-to-many): keyword spotting detections from the audio
- CallRecord → VoicemailMessage (one-to-one, optional): voicemail left if call not answered
- CallRecord → Appointment (one-to-many, optional): appointments booked during or after the call

**Notes:** This is the highest-volume entity in the system, with expected volumes up to approximately 3.7 million records per account. The Auto Load feature on the Calls page implies real-time streaming or polling for newly completed records. The `is_first_time_caller` flag requires a historical lookup against all prior `caller_phone` values within the same account — this should be computed at ingest time and stored, not derived at query time. The `rustpbx_call_records` table in the RustPBX backend provides basic CDR fields (call_id, direction, status, from_number, to_number, duration_secs, recording_url, transcript_text, quality columns) that overlap with this entity; migration or integration must account for field mapping and schema extension for marketing attribution, scoring, tags, visitor tracking, and AI summaries. All monetary and scoring operations should treat `score` as a bounded integer (1–5) enforced at the application layer.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| call_id | short_text | NN, UQ | SIP Call-ID from the signaling layer, or system-generated identifier for non-SIP calls |
| caller_name | short_text | MAX(128) | CNAM lookup result or matched contact display name |
| caller_phone | e164 | NN | Originating phone number in E.164 format |
| callee_phone | e164 | NN | Dialed destination number in E.164 format |
| location | short_text | MAX(128) | Caller geographic location derived from CNAM or geo-IP lookup (e.g., "Portland, OR") |
| direction | enum(Inbound, Outbound, Internal) | NN | Call origination direction relative to the account |
| status | enum(Answered, Missed, Voicemail, In Progress, Failed) | NN | Final or current disposition of the call |
| source_id | uuid | FK(TrackingSource) | Marketing attribution source that generated this call |
| source_number_id | uuid | FK(TrackingNumber) | Tracking number that was dialed, linking to DNI pool assignment |
| agent_id | uuid | FK(User) | Agent who handled or is handling the call |
| queue_id | uuid | FK(Queue) | Call queue that routed this call to an agent |
| automation_id | uuid | FK(Trigger) | Automation trigger or workflow that fired in response to this call |
| started_at | timestamp_tz | NN | Moment the call was initiated (SIP INVITE or equivalent) |
| answered_at | timestamp_tz | | Moment the call was answered; null if missed, voicemail, or failed |
| ended_at | timestamp_tz | | Moment the call ended; null if currently in progress |
| duration_secs | duration_sec | | Total elapsed call duration from start to end |
| ring_duration_secs | duration_sec | | Elapsed time from call initiation until answer or abandonment |
| hold_duration_secs | duration_sec | | Cumulative time the caller spent on hold during the call |
| recording_url | file_ref | | Object store path or URL to the recorded audio file |
| has_audio | boolean | NN | True if a recording exists and is playable; default false |
| score | integer | | Agent or system quality score from 1 (lowest) to 5 (highest) |
| converted | boolean | | True if this call resulted in a conversion event; default false |
| outcome | enum(Converted, In Progress, Lost, No Response) | | Final business outcome classification for the call |
| reporting_tag | short_text | MAX(64) | Free-form manual classification label used for reporting grouping |
| is_first_time_caller | boolean | | True if caller_phone has never called this account before; computed at ingest |
| appointment_set | boolean | | True if an appointment was booked as a result of this call; default false |
| category | enum(New, Existing, VIP, Lead) | | CRM-style contact category of the caller |
| notes | long_text | | Free-form agent or supervisor notes attached to the call |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Record last-modification timestamp |

---

### CallFlowEvent

**UI References:** CallDetailPanel > Flow tab

**Relationships:**
- CallRecord → CallFlowEvent (one-to-many): a single call produces an ordered sequence of flow events

**Notes:** Events must be stored and displayed in strict chronological order. The `segment_index` or `occurred_at` should be used for ordering in the UI. The `detail` field captures contextual payload specific to the event type — for example, DTMF digit pressed, transfer destination number, or queue name entered.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| call_id | uuid | FK(CallRecord), NN | Parent call this event belongs to |
| event_type | enum(Call Received, IVR Start, DTMF Input, Queue Enter, Agent Ring, Agent Answer, Hold Start, Hold End, Transfer Initiated, Transfer Connected, Call Wrap-up, Tags Applied, Call Complete) | NN | The type of event that occurred in the call flow |
| occurred_at | timestamp_tz | NN | Wall-clock timestamp when this event occurred |
| detail | short_text | MAX(256) | Optional contextual detail for the event (e.g., digit pressed, transfer target, queue name) |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### CallVisitorSession

**UI References:** CallDetailPanel > Visitor Detail tab

**Relationships:**
- CallRecord → CallVisitorSession (one-to-one): each call has at most one associated visitor session captured via Dynamic Number Insertion

**Notes:** This entity is only populated when the call was initiated from a web session tracked by the DNI JavaScript snippet. The UTM fields mirror standard Google Analytics campaign parameters. `landing_page` is the first page the visitor loaded in their session, which may differ from the page they were on when they called. `referrer` is the HTTP Referer header captured at session start.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| call_id | uuid | FK(CallRecord), NN, UQ | Parent call; enforces the one-to-one relationship |
| ip_address | short_text | MAX(45) | Visitor IP address (supports IPv4 and IPv6) |
| device | short_text | MAX(128) | Device type and model parsed from User-Agent (e.g., "iPhone 15 Pro") |
| browser | short_text | MAX(128) | Browser name and version parsed from User-Agent (e.g., "Chrome 121") |
| os | short_text | MAX(128) | Operating system parsed from User-Agent (e.g., "iOS 17.2") |
| referrer | url | | Full HTTP Referer URL captured at the start of the web session |
| landing_page | url | | First page URL the visitor loaded during their web session |
| keywords | text | | Search query keywords that led the visitor to the site |
| campaign | short_text | MAX(256) | UTM campaign name (utm_campaign parameter) |
| utm_source | short_text | MAX(256) | UTM source parameter (e.g., "google", "facebook") |
| utm_medium | short_text | MAX(256) | UTM medium parameter (e.g., "cpc", "email") |
| utm_content | short_text | MAX(256) | UTM content parameter for A/B test variant identification |
| utm_term | short_text | MAX(256) | UTM term parameter, typically the paid keyword |
| visit_duration_secs | duration_sec | | Total elapsed time of the web session before the call was made |
| pages_viewed | integer | | Number of distinct pages the visitor loaded during the session |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### CallTranscriptionSegment

**UI References:** CallDetailPanel > main area (transcript view with speaker labels)

**Relationships:**
- CallRecord → CallTranscriptionSegment (one-to-many): a call produces one or more speaker-separated transcript segments

**Notes:** Segments are ordered by `segment_index` for display. The `timestamp_offset_secs` field enables the audio player scrubber to sync with transcript segments — clicking a segment should seek the audio player to that offset. The `confidence` score is the raw ASR engine output and may be used to visually indicate low-confidence segments in the UI. Speaker diarization determines the `speaker` value; "System" is reserved for synthesized TTS prompts (IVR greetings, hold messages, etc.).

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| call_id | uuid | FK(CallRecord), NN | Parent call this segment belongs to |
| segment_index | integer | NN | Zero-based ordering index of this segment within the call transcript |
| timestamp_offset_secs | duration_sec | NN | Seconds elapsed from call start when this segment begins |
| speaker | enum(Agent, Caller, System) | NN | Which party spoke this segment |
| content | long_text | NN | Transcribed text content of the segment |
| confidence | float | | ASR engine confidence score for this segment, range 0.0 to 1.0 |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### CallAISummary

**UI References:** CallDetailPanel > Voice Analysis tab (multiple summary types rendered as selectable cards)

**Relationships:**
- CallRecord → CallAISummary (one-to-many): a call may have multiple AI summaries, one per summary_type

**Notes:** The combination of (call_id, summary_type) must be unique — only one summary of each type may exist per call. When regenerated, the existing row should be updated rather than inserting a duplicate. The `model` field records the specific AI model version used to produce the summary, which is important for audit, reproducibility, and future model migration comparisons. Custom summaries may be user-prompted and are not constrained by the type enum in the same way as system-generated types.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| call_id | uuid | FK(CallRecord), NN | Parent call this summary belongs to |
| summary_type | enum(Classic, Customer Success, Key Insights, Action Items, Sentiment Analysis, Lead Qualification, Compliance Review, Topic Classification, Custom) | NN | The category or template used to generate this summary |
| content | long_text | NN | AI-generated summary text |
| model | short_text | MAX(128) | Identifier of the AI model and version that produced this summary (e.g., "gpt-4o-2024-11-20") |
| generated_at | timestamp_tz | NN | Timestamp when the AI model produced this summary |
| created_at | timestamp_tz | NN | Record insertion timestamp |

UQ(call_id, summary_type) — enforced to ensure one summary per type per call.

---

### CallKeywordHit

**UI References:** CallDetailPanel > Voice Analysis tab, Reports > Keyword Spotting

**Relationships:**
- CallRecord → CallKeywordHit (one-to-many): a call may contain multiple keyword detections
- KeywordSpottingKeyword → CallKeywordHit (many-to-one): each hit references a configured keyword definition

**Notes:** This is a junction-style child table that records each instance of a tracked keyword being detected in the call audio. The same keyword may be hit multiple times in a single call at different offsets, so there is no unique constraint on (call_id, keyword_id). The `timestamp_offset_secs` enables playback seek to the moment of detection.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| call_id | uuid | FK(CallRecord), NN | Parent call in which the keyword was detected |
| keyword_id | uuid | FK(KeywordSpottingKeyword), NN | The configured keyword definition that was matched |
| timestamp_offset_secs | duration_sec | | Seconds from call start at which the keyword was detected |
| speaker | enum(Agent, Caller) | NN | Which party uttered the keyword |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### TextRecord

**UI References:** Activities > Texts page

**Relationships:**
- Account → TextRecord (many-to-one): each text record belongs to one account
- TrackingNumber → TextRecord (many-to-one): via tracking_number_id

**Notes:** This entity represents a conversation-level summary row as displayed in the Texts activity list — it is not a full message thread. Individual messages within a thread are stored in TextMessage. The `preview` field stores the first approximately 160 characters of the most recent message for display in list views. Expected volume is approximately 12,847 records based on mock data.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| contact_name | short_text | MAX(128) | Display name of the contact involved in the conversation |
| contact_phone | e164 | NN | Phone number of the external contact in the conversation |
| tracking_number_id | uuid | FK(TrackingNumber) | Tracking number this conversation is associated with |
| direction | enum(Inbound, Outbound) | NN | Direction of the most recent or initiating message |
| preview | text | MAX(160) | Truncated preview of the most recent message body for list display |
| status | enum(Delivered, Failed, Pending) | NN | Delivery status of the most recent message |
| sent_at | timestamp_tz | NN | Timestamp of the most recent message in the conversation |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### TextMessage

**UI References:** CallDetailPanel > Text Message tab, Activities > Texts

**Relationships:**
- Account → TextMessage (many-to-one): each message belongs to one account
- TrackingNumber → TextMessage (many-to-one): via tracking_number_id
- CallRecord → TextMessage (many-to-one, optional): links a message to an associated call

**Notes:** This entity stores individual SMS messages, enabling the full conversation thread view in the CallDetailPanel Text Message tab. The `call_id` link is optional and used when a text exchange is directly associated with a call (e.g., a post-call follow-up SMS sent automatically). Messages within a thread are ordered by `sent_at`.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| contact_phone | e164 | NN | Phone number of the external contact (sender or recipient) |
| tracking_number_id | uuid | FK(TrackingNumber), NN | Platform tracking number used to send or receive this message |
| call_id | uuid | FK(CallRecord) | Optional: associated call record, if this message is linked to a call |
| direction | enum(Inbound, Outbound) | NN | Whether this message was received from or sent to the contact |
| body | long_text | NN | Full text content of the SMS/MMS message |
| status | enum(Delivered, Failed, Pending) | NN | Carrier delivery status of the message |
| sent_at | timestamp_tz | NN | Timestamp when the message was sent or received |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### FormRecord

**UI References:** Activities > Forms page

**Relationships:**
- Account → FormRecord (many-to-one): each form submission belongs to one account

**Notes:** The `form_data` JSON column stores the complete set of form field name/value pairs as submitted, preserving all fields regardless of schema changes to specific form configurations over time. The `tracking_number` field stores the human-readable tracking number string (not a FK) because form submissions may reference numbers that are not yet reconciled to the tracking number catalog. Expected volume is approximately 1,823 records based on mock data.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| contact_name | short_text | MAX(128) | Name of the person who submitted the form |
| contact_phone | e164 | | Phone number provided in the form submission |
| contact_email | email | | Email address provided in the form submission |
| form_name | short_text | NN, MAX(256) | Name or identifier of the form that was submitted |
| source | short_text | MAX(256) | Marketing source attribution string for this submission |
| tracking_number | short_text | MAX(32) | Tracking number string associated with the web session at time of submission |
| form_data | json | | Complete key/value snapshot of all fields submitted in the form |
| status | enum(New, Contacted, Qualified, Closed) | NN | Lead lifecycle status of this form submission; default New |
| submitted_at | timestamp_tz | NN | Timestamp when the form was submitted by the visitor |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### ChatRecord

**UI References:** Activities > Chats page

**Relationships:**
- Account → ChatRecord (many-to-one): each chat session belongs to one account
- User → ChatRecord (many-to-one): agent assigned to the chat session, via agent_id
- ChatWidget → ChatRecord (many-to-one): the widget configuration that initiated the session, via widget_id

**Notes:** Represents a full live chat or web chat session as a single summary row. Individual messages within the session are not modeled in this shard. The `channel` field distinguishes between web-based chat and messaging channel integrations. Expected volume is approximately 5,412 records based on mock data.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| visitor_name | short_text | MAX(128) | Display name of the visitor or contact who initiated the chat |
| visitor_detail | short_text | MAX(256) | Secondary visitor identifier such as location, email, or session ID |
| channel | enum(Web Chat, SMS, WhatsApp) | NN | Communication channel over which the chat occurred |
| message_count | integer | NN | Total number of messages exchanged in the session; default 0 |
| agent_id | uuid | FK(User) | Agent who handled or is handling the chat session |
| widget_id | uuid | FK(ChatWidget) | Chat widget configuration that was used to initiate this session |
| status | enum(Active, Closed, Missed) | NN | Current or final status of the chat session |
| duration_secs | duration_sec | | Elapsed duration of the chat session from start to end |
| started_at | timestamp_tz | NN | Timestamp when the chat session was initiated |
| ended_at | timestamp_tz | | Timestamp when the chat session was closed; null if still active |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### FaxRecord

**UI References:** Activities > Faxes page

**Relationships:**
- Account → FaxRecord (many-to-one): each fax record belongs to one account

**Notes:** The `document_url` references the stored fax document in the object store. For inbound faxes this is the received image; for outbound it is the original transmitted document. Expected volume is approximately 892 records based on mock data, making this the lowest-volume activity type.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| contact_name | short_text | MAX(128) | Name of the sender or recipient contact |
| from_number | e164 | NN | Originating fax number in E.164 format |
| to_number | e164 | NN | Destination fax number in E.164 format |
| direction | enum(Inbound, Outbound) | NN | Whether this fax was received or sent by the account |
| pages | integer | NN | Total number of pages in the fax document |
| status | enum(Received, Sent, Failed, Sending) | NN | Current or final transmission status |
| document_url | file_ref | | Object store path or URL to the fax document file |
| sent_at | timestamp_tz | NN | Timestamp when the fax was sent or received |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### VideoRecord

**UI References:** Activities > Videos page

**Relationships:**
- Account → VideoRecord (many-to-one): each video record belongs to one account
- User → VideoRecord (many-to-one): the host agent, via host_agent_id

**Notes:** Represents a summary of a single video conference session. The `platform` field captures the third-party conferencing platform used (e.g., "Zoom", "Google Meet"). Recording storage via `recording_url` is only populated when `has_recording` is true. Expected volume is approximately 2,156 records based on mock data.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| participant_name | short_text | MAX(128) | Display name of the primary external participant |
| participant_email | email | | Email address of the primary external participant |
| host_agent_id | uuid | FK(User) | Agent who hosted or initiated the video session |
| platform | short_text | NN, MAX(64) | Name of the video conferencing platform used (e.g., "Zoom") |
| has_recording | boolean | NN | True if a video recording exists; default false |
| recording_url | file_ref | | Object store path or URL to the video recording file |
| duration_secs | duration_sec | | Elapsed duration of the video session from start to end |
| started_at | timestamp_tz | NN | Timestamp when the video session began |
| ended_at | timestamp_tz | | Timestamp when the video session ended; null if still active |
| created_at | timestamp_tz | NN | Record creation timestamp |

---

### ExportRecord

**UI References:** Activities > Export Log page

**Relationships:**
- Account → ExportRecord (many-to-one): each export job belongs to one account
- User → ExportRecord (many-to-one): the user who requested the export, via requested_by_id

**Notes:** This entity tracks the lifecycle of asynchronous data export jobs. The `filters_applied` JSON field stores a snapshot of all filter criteria that were active when the export was requested, enabling audit of exactly what data was included. The `download_url` is only populated when `status` is Complete. Expected volume is approximately 47 records based on mock data; this is a low-volume audit/operational entity.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account for multi-tenant isolation |
| name | short_text | NN, MAX(256) | Human-readable label for this export job |
| export_type | short_text | NN, MAX(64) | Category of data exported (e.g., "Calls", "Texts", "Forms") |
| format | enum(CSV, PDF, Excel) | NN | File format of the exported data |
| date_range | short_text | MAX(128) | Human-readable description of the date range included (e.g., "Jan 1 – Jan 31, 2025") |
| record_count | integer | NN | Total number of records included in the export |
| status | enum(Complete, Processing, Failed) | NN | Current lifecycle status of the export job |
| download_url | file_ref | | Object store path or pre-signed URL to the completed export file |
| requested_by_id | uuid | FK(User), NN | User who initiated the export request |
| filters_applied | json | | Snapshot of all filter parameters active when the export was requested |
| created_at | timestamp_tz | NN | Timestamp when the export job was created |
| completed_at | timestamp_tz | | Timestamp when the export job finished (success or failure) |

---

## Volume & Performance Considerations

### CallRecord — Primary Hot Path

CallRecord is by far the most read and write-intensive entity. With up to 3.7 million records per account and multi-tenant deployments, all query plans must assume account-scoped access; `account_id` should be part of every composite index on this table and must appear in every WHERE clause. Partitioning by `account_id` at the storage layer is strongly recommended.

The most common query patterns on CallRecord are:

- Paginated list view filtered by date range, status, direction, agent, source, and tag — requires composite indexes on (account_id, started_at DESC) as the primary sort axis, with additional indexes supporting each filterable dimension
- Aggregate reporting queries (call counts by day, conversion rate by source, average handle time by agent) — these are read-heavy analytical queries that benefit from columnar storage or pre-aggregated summary tables in the reporting layer
- Auto Load polling — a lightweight query returning rows where started_at > {last_seen_at} for a given account_id, requiring the (account_id, started_at) index for efficiency
- Single-call detail fetch — point lookup by id or call_id, both of which should be indexed

The `is_first_time_caller` flag should be computed at ingest time using an EXISTS subquery over historical caller_phone values for the account, then stored as a boolean. Computing it at read time across 3.7M rows would be prohibitive.

The `updated_at` column enables efficient cache invalidation and incremental sync for downstream reporting pipelines.

### CallFlowEvent

Volume is proportional to CallRecord at approximately 8–15 events per call, yielding up to 30–55 million rows per large account. Queries are exclusively child lookups by `call_id` after the parent record is resolved. A covering index on (call_id, occurred_at) satisfies all query patterns. This table benefits from co-location or clustering with its parent CallRecord rows.

### CallTranscriptionSegment

Transcription segments are append-only after generation and are queried exclusively by `call_id`. Volume depends on call duration: at an average of 3–5 segments per minute for a 5-minute call, expect 15–25 segments per call. At 3.7M calls, this yields 55–90 million rows per large account. The (call_id, segment_index) index is the critical path. The `content` field is long text and should not be fully indexed — full-text search over transcripts (if supported) requires a separate search index (e.g., Elasticsearch or a dedicated FTS table) rather than B-tree indexing.

### CallAISummary

A bounded child table with at most 9 rows per call (one per summary_type). The unique constraint on (call_id, summary_type) is both a data integrity requirement and an effective index for the detail fetch. Total volume at 3.7M calls and 2–3 average summary types per call yields approximately 7–11 million rows per large account. No hot-path concerns beyond the parent-child join.

### CallKeywordHit

Volume is sparse and highly variable — accounts with aggressive keyword tracking may produce 10–50 hits per call, while others produce none. The (call_id) index is the primary lookup path. The (keyword_id, account_id, started_at) index pattern (joined via CallRecord) supports the Keyword Spotting report aggregation.

### CallVisitorSession and TextRecord / TextMessage

These entities are medium-volume per-account. CallVisitorSession is one-to-one with CallRecord so inherits its scale. TextRecord and TextMessage are expected at roughly 10x lower volume than CallRecord. Standard account-scoped date-range indexes are sufficient.

### FormRecord, ChatRecord, FaxRecord, VideoRecord

These are low-to-medium volume activity types. FormRecord (~1,823 mock), FaxRecord (~892 mock), VideoRecord (~2,156 mock), and ChatRecord (~5,412 mock) will not impose significant storage or query pressure at launch. Standard (account_id, created_at DESC) composite indexes are sufficient for all list view queries.

### ExportRecord

Operational/audit table with negligible volume (~47 records shown in mock). Standard indexing on (account_id, created_at) is sufficient. The `filters_applied` JSON column does not require indexing.

### Cross-Entity Reporting

Reports pages aggregate across CallRecord, CallFlowEvent, CallKeywordHit, CallVisitorSession, and CallTranscriptionSegment simultaneously. These analytical queries should be served from a read replica or a pre-aggregated OLAP layer rather than the primary transactional store. Summary tables keyed by (account_id, date, source_id, agent_id) should be maintained incrementally at call completion time to power the Reports section without full table scans on CallRecord at query time.
