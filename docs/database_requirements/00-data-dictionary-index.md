# 4iiz Data Element Dictionary — Master Index

> **Status:** Pre-analysis / Data element reconciliation
> **Date:** 2026-02-25
> **Purpose:** Catalog all data entities, attributes, and relationships required by the 4iiz functional requirements, independent of storage technology or physical schema.

---

## Document Map

| # | Shard | Entities | File |
|---|-------|----------|------|
| 01 | Communication Records | CallRecord, TextRecord, FormRecord, ChatRecord, FaxRecord, VideoRecord, ExportRecord + child tables | [01-communication-records.md](01-communication-records.md) |
| 02 | People & Accounts | Account, User, AgentStateLog, Notification, ContactList, BlockedNumber, DncEntry, DntEntry | [02-people-and-accounts.md](02-people-and-accounts.md) |
| 03 | Telephony & Numbers | TrackingNumber, ReceivingNumber, TextNumber, TargetNumber, TrackingSource, NumberPool, PortRequest, CallSettings | [03-telephony-numbers.md](03-telephony-numbers.md) |
| 04 | Routing & Call Flow | VoiceMenu, Queue, SmartRouter, GeoRouter, Schedule, RoutingTable, AgentScript, VoicemailBox + child tables | [04-routing-and-callflow.md](04-routing-and-callflow.md) |
| 05 | Automation & Engagement | Workflow, Trigger, Lambda, Webhook, BulkMessage, LeadReactor, SmartDialer, FormReactor, Reminder, KeywordSpotting, ChatWidget | [05-automation-and-engagement.md](05-automation-and-engagement.md) |
| 06 | AI Tools | AskAIConfig, SummaryConfig, KnowledgeBank, VoiceAIAgent, ChatAIAgent, DialogflowConfig, ChatAIConfig | [06-ai-tools.md](06-ai-tools.md) |
| 07 | Analytics & Reporting | ScoringConfig, Tag, CustomReport, NotificationRule, Appointment, CallDailySummary | [07-analytics-and-reporting.md](07-analytics-and-reporting.md) |
| 08 | Compliance & Trust | BusinessInfo, AuthorizedContact, A2PCampaign, TollFreeRegistration, VoiceRegistration, CallerIdCnam, ComplianceRequirement, ComplianceApplication, ComplianceAddress | [08-compliance-and-trust.md](08-compliance-and-trust.md) |
| 09 | System & Operations | ApiLogEntry, AccountVariable, FrequencyLimit, MonitoringEvent, Presence, Location, ActiveCall | [09-system-and-operations.md](09-system-and-operations.md) |

---

## Semantic Type Glossary

All attribute types in the shard documents use the following semantic types. These are logical types — physical mapping (e.g., `uuid` → `TEXT` in SQLite, `UUID` in PostgreSQL) is deferred to schema design.

| Semantic Type | Description | Example Values |
|---------------|-------------|----------------|
| `uuid` | Universally unique identifier | `550e8400-e29b-41d4-a716-446655440000` |
| `text` | Variable-length character string | `"Acme Corp"` |
| `short_text` | Constrained text (≤255 chars) | `"Main Office Queue"` |
| `long_text` | Unconstrained text (articles, scripts, code) | Transcript body, Lambda source code |
| `e164` | Phone number in E.164 format | `+17072833106` |
| `email` | RFC 5322 email address | `admin@example.com` |
| `url` | Absolute URL / URI | `https://hooks.example.com/callback` |
| `file_ref` | Reference to object store path or URL | `recordings/abc123.wav` |
| `boolean` | True / false | `true` |
| `integer` | Signed integer (no fractional part) | `42`, `300` |
| `counter` | Non-negative integer, monotonically incremented | `1547` (calls blocked) |
| `float` | IEEE 754 double-precision | `0.85`, `3.14` |
| `timestamp_tz` | Date + time with timezone | `2026-02-25T14:30:00Z` |
| `date` | Calendar date without time | `2026-02-25` |
| `time` | Time of day without date | `09:00`, `17:30` |
| `duration_sec` | Duration in whole seconds | `185` (3:05) |
| `duration_ms` | Duration in milliseconds | `2340` |
| `json` | Structured JSON document | `{"key": "value"}` |
| `encrypted_text` | Text requiring encryption at rest | EIN, API keys, credentials |
| `enum(...)` | Constrained value from named set | `enum(Active, Inactive)` |
| `bitmask` | Bit field representing multiple flags | `0b1111100` (Mon-Fri) |
| `percentage` | Integer 0–100 representing a percentage | `95` |
| `money` | Decimal currency amount (2 decimal places) | `29.99` |
| `hex_color` | CSS hex color code | `#00bcd4` |
| `cron` | Cron expression for scheduling | `0 8 * * 1-5` |
| `rrule` | iCalendar recurrence rule | `FREQ=WEEKLY;BYDAY=MO,WE,FR` |
| `regex` | Regular expression pattern | `^\+1707.*` |

---

## Enumeration Registry

Enumerations shared across multiple entities are defined here once. Entity-specific enums are defined inline in their shard document.

### CallStatus
`Answered` · `Missed` · `Voicemail` · `In Progress` · `Failed`

### Direction
`Inbound` · `Outbound` · `Internal`

### DeliveryStatus
`Delivered` · `Failed` · `Pending` · `Sending`

### ActiveStatus
`Active` · `Inactive`

### EnabledStatus
`Active` · `Paused` · `Draft`

### RecordStatus
`Complete` · `Processing` · `Failed`

### UserRole
`Admin` · `Agent` · `Supervisor`

### AgentStatus
`Available` · `On Call` · `After Call Work` · `Offline` · `Break`

### ContactCategory
`New` · `Existing` · `VIP` · `Lead`

### CallOutcome
`Converted` · `In Progress` · `Lost` · `No Response`

### DistributionStrategy
`Ring All` · `Round Robin` · `Longest Idle` · `Weighted`

### DayOfWeek
`Mon` · `Tue` · `Wed` · `Thu` · `Fri` · `Sat` · `Sun`

### ApprovalStatus
`Pending` · `Approved` · `Rejected` · `Suspended`

### ComplianceStatus
`Completed` · `In Progress` · `Not Started`

### SpeakerRole
`Agent` · `Caller` · `System`

### NumberType
`Local` · `Toll-Free`

### TrackingNumberType
`Offsite Static` · `Onsite Dynamic`

---

## Entity Cross-Reference Matrix

Maps each entity to the UI pages and shared components that read or write it.

### Activities Section

| Entity | Calls | Texts | Forms | Chats | Faxes | Videos | Export Log | CallDetailPanel |
|--------|:-----:|:-----:|:-----:|:-----:|:-----:|:------:|:----------:|:---------------:|
| CallRecord | R/W | | | | | | | R/W |
| TextRecord | | R/W | | | | | | |
| TextMessage | | | | | | | | R/W |
| FormRecord | | | R/W | | | | | |
| ChatRecord | | | | R/W | | | | |
| FaxRecord | | | | | R/W | | | |
| VideoRecord | | | | | | R/W | | |
| ExportRecord | | | | | | | R | |
| CallFlowEvent | | | | | | | | R |
| CallVisitorSession | | | | | | | | R |
| CallTranscriptionSegment | | | | | | | | R |
| CallAISummary | | | | | | | | R |
| Tag | R/W | | | | | | | R/W |

### Contacts Section

| Entity | Contact Lists | Blocked Numbers | Do Not Call | Do Not Text |
|--------|:------------:|:---------------:|:-----------:|:-----------:|
| ContactList | R/W | | | |
| ContactListMember | R/W | | | |
| BlockedNumber | | R/W | | |
| DncEntry | | | R/W | |
| DntEntry | | | | R/W |

### Numbers Section

| Entity | Tracking Numbers | Receiving Numbers | Text Numbers | Target Numbers | Sources | Buy Numbers | Number Pools | Call Settings |
|--------|:----------------:|:-----------------:|:------------:|:--------------:|:-------:|:-----------:|:------------:|:-------------:|
| TrackingNumber | R/W | R | | R | R | | R | |
| ReceivingNumber | R | R/W | | | | | | |
| TextNumber | | | R/W | | | | | |
| TargetNumber | | | | R/W | | | | |
| TrackingSource | R | | | | R/W | | R | |
| NumberPool | | | | | | | R/W | |
| NumberPoolMember | | | | | | | R/W | |
| PortRequest | | | | | | R/W | | |
| CallSettings | | | | | | | | R/W |

### Flows Section

| Entity | Voice Menus | Queues | Smart Routers | Schedules | Geo Routers | Triggers | Webhooks | Bulk Msgs | Workflows | Lambdas | Form Reactors | Chat Widget | API Logs |
|--------|:-----------:|:------:|:-------------:|:---------:|:-----------:|:--------:|:--------:|:---------:|:---------:|:-------:|:--------------:|:-----------:|:--------:|
| VoiceMenu | R/W | | | | | | | | | | | | |
| Queue | | R/W | | | | | | | | | | | |
| SmartRouter | | | R/W | | | | | | | | | | |
| Schedule | | | | R/W | | | | | | | | | |
| GeoRouter | | | | | R/W | | | | | | | | |
| Trigger | | | | | | R/W | | | | | | | |
| Webhook | | | | | | | R/W | | | | | | |
| BulkMessage | | | | | | | | R/W | | | | | |
| Workflow | | | | | | | | | R/W | | | | |
| Lambda | | | | | | | | | | R/W | | | |
| FormReactorEntry | | | | | | | | | | | R/W | | |
| ChatWidget | | | | | | | | | | | | R/W | |
| ApiLogEntry | | | | | | | | | | | | | R |

### AI Tools Section

| Entity | Ask AI | Summaries | Knowledge Banks | Voice AI | Chat AI |
|--------|:------:|:---------:|:---------------:|:--------:|:-------:|
| AskAIConfig | R/W | | | | |
| SummaryConfig | | R/W | | | |
| KnowledgeBank | | | R/W | | R |
| KnowledgeBankDocument | | | R/W | | |
| VoiceAIAgent | | | | R/W | |
| ChatAIAgent | | | | | R/W |
| ChatAIConfig | | | | | R/W |
| DialogflowConfig | | | | | R/W |

### Reports Section

| Entity | Activity | Calls by Source | Agent Perf | Queue | Daily Calls | Missed Calls | Appts | Scoring | Map | Custom |
|--------|:--------:|:---------------:|:----------:|:-----:|:-----------:|:------------:|:-----:|:-------:|:---:|:------:|
| CallRecord | R | R | R | R | R | R | | R | R | R |
| CallDailySummary | R | R | R | R | R | R | | | R | R |
| Tag | | | | | | | | R | | |
| ScoringConfig | | | | | | | | R/W | | |
| Appointment | | | | | | | R/W | | | |
| CustomReport | | | | | | | | | | R/W |
| NotificationRule | | | | | | | | | | R/W |

### Trust Center Section

| Entity | Business Info | Local Text | Toll-Free Text | Voice Reg | Caller ID | Compliance |
|--------|:------------:|:----------:|:--------------:|:---------:|:---------:|:----------:|
| BusinessInfo | R/W | | | | | |
| AuthorizedContact | R/W | | | | | |
| A2PCampaign | | R/W | | | | |
| TollFreeRegistration | | | R/W | | | |
| VoiceRegistration | | | | R/W | | |
| CallerIdCnam | | | | | R/W | |
| ComplianceRequirement | | | | | | R |
| ComplianceApplication | | | | | | R/W |
| ComplianceAddress | R/W | | | | | R |

---

## High-Level Relationship Map

```
Account (tenant root)
 ├─ User/Agent ─── AgentStateLog
 │   └─ Notification
 │
 ├─ TrackingSource
 │   └─ TrackingNumber ─── NumberPool ─── NumberPoolMember
 │       ├─ ReceivingNumber (target)
 │       └─ TargetNumber (distribution)
 │
 ├─ CallRecord (highest volume)
 │   ├─ CallFlowEvent
 │   ├─ CallVisitorSession
 │   ├─ CallTranscriptionSegment
 │   ├─ CallAISummary
 │   ├─ CallKeywordHit
 │   └─ CallTag ─── Tag
 │
 ├─ TextRecord / FormRecord / ChatRecord / FaxRecord / VideoRecord
 │
 ├─ VoiceMenu ─── VoiceMenuOption
 ├─ Queue ─── QueueAgent
 ├─ SmartRouter ─── SmartRouterRule
 ├─ GeoRouter ─── GeoRouterRule
 ├─ Schedule ─── ScheduleHoliday
 ├─ VoicemailBox ─── VoicemailMessage
 ├─ RoutingTable ─── RoutingTableRoute
 │
 ├─ Workflow ─── WorkflowNode / WorkflowEdge
 ├─ Trigger ─── TriggerCondition / TriggerAction
 ├─ Webhook ─── WebhookSubscription / WebhookDelivery
 ├─ Lambda ─── LambdaEnvVar
 ├─ KeywordSpottingConfig ─── KeywordSpottingKeyword / KeywordSpottingNumber
 │
 ├─ KnowledgeBank ─── KnowledgeBankDocument
 ├─ VoiceAIAgent / ChatAIAgent / ChatAIConfig
 │
 ├─ ContactList ─── ContactListMember
 ├─ BlockedNumber / DncEntry / DntEntry
 │
 ├─ BusinessInfo ─── AuthorizedContact
 ├─ A2PCampaign / TollFreeRegistration / VoiceRegistration
 ├─ ComplianceAddress / ComplianceApplication
 │
 └─ CallSettings / ScoringConfig / SummaryConfig (1:1 per account)
```

---

## Existing RustPBX Entities

The following entities already exist in the RustPBX codebase (SeaORM models in `src/models/`). The shard documents note overlap where 4iiz requirements extend or diverge from these.

| RustPBX Table | Shard | 4iiz Equivalent | Gap Assessment |
|---------------|-------|-----------------|----------------|
| `rustpbx_users` | 02 | User/Agent | 4iiz needs role, initials, avatar_color, agent status tracking |
| `rustpbx_extensions` | 02 | (no direct equivalent) | SIP-level entity; 4iiz User is higher-level |
| `rustpbx_departments` | 02 | (no direct equivalent) | Organizational grouping; 4iiz uses Account hierarchy |
| `rustpbx_extension_departments` | 02 | (no direct equivalent) | M:M junction; may map to queue membership |
| `rustpbx_sip_trunks` | 03 | TrackingNumber (partial) | Trunks are SIP infra; TrackingNumbers are marketing layer above |
| `rustpbx_routes` | 04 | SmartRouter / RoutingTable | Overlapping concepts, different granularity |
| `rustpbx_call_records` | 01 | CallRecord | Significant extension needed (source attribution, scoring, tags, visitor data) |
| `rustpbx_frequency_limits` | 09 | FrequencyLimit | Direct match |
| `rustpbx_voicemails` | 04 | VoicemailMessage | Close match; 4iiz adds transcription_enabled, email notification |
| `rustpbx_voicemail_greetings` | 04 | (part of VoicemailBox) | 4iiz merges into VoicemailBox entity |
| `rustpbx_monitoring_events` | 09 | MonitoringEvent | Direct match |
| `rustpbx_queues` | 04 | Queue | 4iiz needs agent assignment, strategy, schedule, real-time metrics |
| `rustpbx_locations` | 09 | Location | Direct match (SIP registration bindings) |
| `presence_states` | 09 | Presence | Direct match |
