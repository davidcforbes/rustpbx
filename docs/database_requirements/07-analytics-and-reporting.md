# 07 — Analytics & Reporting

## Overview

The analytics and reporting domain provides the data structures that power all 30 report pages in the 4iiz platform. This domain spans two broad categories of entities.

The first category is **configuration entities** — records that are directly authored and managed by account administrators and users. These include `Tag` and `CallTag` (flexible classification applied to calls for filtering and segmentation), `ScoringConfig` (account-level weights defining how agent and call quality scores are calculated), `CustomReport` (saved report definitions with column selections, filters, date ranges, and optional delivery schedules), and `NotificationRule` (threshold-based alert configurations that fire when a monitored metric crosses a boundary). Configuration entities are written infrequently but read constantly in the reporting pipeline as filter and display parameters.

The second category is **computed entities** — records that are not authored by users but are derived from source data by batch jobs or materialized view processes. `CallDailySummary` is the sole computed entity in this shard and is central to report performance. It pre-aggregates the raw `CallRecord` table (documented in shard 01) into daily dimension slices keyed by account, date, source, agent, and queue. Because individual call records may number in the tens of millions for large accounts, the summary table is what makes sub-second rendering of the Activity Report, Agent Performance, Queue Report, and all template reports feasible.

The third structural element is `Appointment`, which sits between these two categories. It is created by human agents as a conversion outcome tied to a specific call, but it also feeds directly into computed metrics such as `appointments_set` in `CallDailySummary`.

`CallRecord` itself (shard 01) remains the authoritative source of truth. The entities in this shard either annotate it (Tag, CallTag, Appointment), configure how it is evaluated (ScoringConfig, NotificationRule), define how it is queried and presented (CustomReport), or summarize it for fast retrieval (CallDailySummary).

---

### Tag

**UI References:** Activities > Calls page (tag column), CallDetailPanel > Score tab (tag application), Reports > by Tag, Trigger actions (Apply Tag)

**Relationships:**
- belongs to one Account (many Tags per Account)
- linked to many CallRecords through CallTag (many-to-many)
- may be applied by a Trigger (referenced in CallTag.trigger_id)

**Notes:** Tag names must be unique within an account but the same label may exist across accounts. The `usage_count` field is a denormalized counter maintained by the application layer — it is incremented when a CallTag row is inserted and decremented when a CallTag row is deleted. This avoids expensive COUNT queries when rendering tag lists with usage indicators. Tags are soft-deleted by convention (not modeled here) to preserve historical CallTag associations.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, UQ(account_id), MAX(64) | Display name for the tag, unique within the account |
| color | hex_color | | Six-character hex color used to render the tag badge in the UI (e.g., #00bcd4) |
| description | text | MAX(500) | Optional human-readable explanation of the tag's purpose |
| usage_count | counter | NN, default 0 | Denormalized count of CallTag rows referencing this tag; maintained by application logic on insert/delete of CallTag |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Record last-modified timestamp |

---

### CallTag

**UI References:** Activities > Calls page, CallDetailPanel (Score tab, all tabs that surface tags), Reports > by Tag

**Relationships:**
- belongs to one CallRecord
- belongs to one Tag
- optionally belongs to one User (the agent who applied it manually)
- optionally belongs to one Trigger (the automation rule that applied it)

**Notes:** The combination of `call_id` and `tag_id` must be unique — a tag may only be applied to a given call once regardless of how many times an automation runs. When a CallTag row is inserted, the corresponding `Tag.usage_count` must be incremented atomically. When a CallTag row is deleted, it must be decremented. The `applied_by_type` discriminator determines which of the optional foreign keys (`applied_by_id` for Manual, `trigger_id` for Automation) will be populated. AI-applied tags (from transcript analysis) populate neither foreign key but set `applied_by_type` to AI.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| call_id | uuid | FK(CallRecord), NN | The call this tag is applied to |
| tag_id | uuid | FK(Tag), NN | The tag being applied |
| applied_at | timestamp_tz | NN | Timestamp when the tag was applied |
| applied_by_type | enum(Manual, Automation, AI) | NN | Discriminator indicating the source of the tag application |
| applied_by_id | uuid | FK(User) | Set when applied_by_type is Manual; the user who applied the tag |
| trigger_id | uuid | FK(Trigger) | Set when applied_by_type is Automation; the trigger rule that caused the tag to be applied |

**Composite unique constraint:** UQ(call_id, tag_id) — one row per tag per call.

---

### ScoringConfig

**UI References:** Reports > Scoring page

**Relationships:**
- belongs to exactly one Account (one-to-one)

**Notes:** This entity is a singleton per account — at most one row per `account_id`. The three weight fields (`answer_rate_weight`, `talk_time_weight`, `conversion_weight`) must sum to exactly 100 at the application layer before any write is accepted. The computed score stored on `CallRecord.score` is a decimal in the range 1.0–5.0. The scoring formula applies each weight as a fraction of the maximum achievable value for that dimension: an agent's answer rate is compared to `target_answer_rate`, talk time is evaluated against `min_talk_time_secs` as the floor, and conversion is a binary outcome per call. The resulting weighted sum is normalized to the 1–5 scale.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN, UQ | Owning account; one ScoringConfig per account |
| answer_rate_weight | percentage | NN, default 34 | Weight (0–100) assigned to the answer rate dimension of the score formula |
| talk_time_weight | percentage | NN, default 33 | Weight (0–100) assigned to the talk time dimension; answer_rate_weight + talk_time_weight + conversion_weight must equal 100 |
| conversion_weight | percentage | NN, default 33 | Weight (0–100) assigned to the conversion (appointment set) dimension |
| min_talk_time_secs | duration_sec | NN, default 60 | Minimum talk time in seconds for a call to count as a quality call in the talk time dimension |
| target_answer_rate | percentage | NN, default 90 | The benchmark answer rate (0–100) used as the 100% reference point when scoring the answer rate dimension |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Record last-modified timestamp |

---

### Appointment

**UI References:** Reports > Appointments page, CallDetailPanel > Score tab (appointment_set flag), Reports > template reports (conversion KPIs)

**Relationships:**
- belongs to one Account
- linked to one CallRecord (the originating call)
- optionally linked to one TrackingSource (inherited from the call)
- optionally linked to one User (the booking agent)

**Notes:** When an Appointment row is created, the application must set `CallRecord.appointment_set = true` on the associated call and increment `CallDailySummary.appointments_set` for the relevant date/dimension slice. The `revenue` field is optional and populated when the account has revenue tracking enabled; it feeds ROI calculations on the Appointments report and template report pages. The `source_id` is denormalized from the call's tracking source to allow appointment reports to be broken down by marketing source without joining back through CallRecord.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| call_id | uuid | FK(CallRecord) | The call that generated this appointment; may be null if appointment is entered manually without a call |
| scheduled_at | timestamp_tz | NN | The date and time the appointment is scheduled to occur |
| caller_name | short_text | MAX(128) | Name of the caller / patient / customer for whom the appointment is booked |
| caller_phone | e164 | | Phone number of the caller, in E.164 format |
| source_id | uuid | FK(TrackingSource) | Marketing source inherited from the originating call; used for source-level appointment reporting |
| agent_id | uuid | FK(User) | The agent who booked the appointment |
| appointment_type | enum(New, Follow-up, Consultation, Demo, Other) | NN, default New | Categorization of the appointment type |
| status | enum(Confirmed, Pending, Completed, No-Show, Cancelled) | NN, default Pending | Current lifecycle status of the appointment |
| revenue | money | | Revenue amount associated with this appointment, if tracked; used for ROI and conversion value reporting |
| notes | text | MAX(2000) | Free-text notes about the appointment |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Record last-modified timestamp |

---

### CustomReport

**UI References:** Reports > Custom Reports page

**Relationships:**
- belongs to one Account
- created by one User
- may be shared with all Users in the Account (when is_shared = true)

**Notes:** A CustomReport row stores a complete query specification — it does not store results. When a user runs or schedules the report, the engine reads `report_type`, `columns`, `filters`, and `date_range_type` to construct the query at runtime. If `date_range_type` is not Custom, the start and end dates are computed dynamically at run time (e.g., "This Month" resolves to the current calendar month). When `schedule` is set, a background job evaluates the cron expression and delivers rendered output to all addresses in `schedule_recipients`. The `is_shared` flag controls visibility: when false, the report is private to `created_by_id`; when true, all users in the account can view and run it (but only the owner or an admin may edit it).

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(128) | Display name for the saved report |
| description | text | MAX(500) | Optional description of the report's purpose |
| report_type | short_text | NN, MAX(64) | Base report type identifier (e.g., "calls", "agents", "sources", "queues") that determines which base query and available columns apply |
| columns | json | NN | Ordered array of column definition objects specifying which fields to display, their labels, and any aggregation functions |
| filters | json | NN | Saved filter criteria object specifying dimension values, date ranges, and metric conditions to apply to the query |
| date_range_type | enum(Today, Yesterday, This Week, This Month, Last 30 Days, Custom) | NN | Controls how the report's date window is resolved at run time; Custom requires custom_start_date and custom_end_date |
| custom_start_date | date | | Inclusive start date when date_range_type is Custom |
| custom_end_date | date | | Inclusive end date when date_range_type is Custom |
| sort_column | short_text | MAX(64) | Column identifier by which results are sorted |
| sort_direction | enum(ASC, DESC) | NN, default DESC | Sort order applied to sort_column |
| schedule | cron | | Cron expression defining when the report is automatically generated and delivered; null indicates manual-run-only |
| schedule_recipients | json | | Array of email address strings to receive scheduled report deliveries; relevant only when schedule is set |
| last_run_at | timestamp_tz | | Timestamp of the most recent execution of this report, whether manual or scheduled |
| created_by_id | uuid | FK(User), NN | User who created the report |
| is_shared | boolean | NN, default false | When true, all users in the account can view and run this report |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Record last-modified timestamp |

---

### NotificationRule

**UI References:** Reports section (alert configuration panel), global notification system (in-app notification center)

**Relationships:**
- belongs to one Account

**Notes:** A NotificationRule is evaluated periodically (typically every `time_window_minutes` minutes by a background worker) by querying the relevant metric against live or summary data. If the metric's current value satisfies the `condition_operator` and `threshold_value`, a notification is dispatched via `notification_method` to all targets in `recipients`. The `cooldown_minutes` field prevents alert storms by enforcing a minimum gap between successive firings of the same rule. `trigger_count` is a denormalized counter incremented each time the rule fires, useful for audit and health dashboards. Rules are independently enabled or disabled via `is_active` without deleting the configuration.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(128) | Human-readable name for this alert rule |
| description | text | MAX(500) | Optional explanation of what this rule monitors and why |
| metric | short_text | NN, MAX(64) | Identifier of the metric being monitored (e.g., "missed_calls", "avg_wait_time", "answer_rate") |
| condition_operator | enum(greater_than, less_than, equals) | NN | Comparison operator applied between the metric's current value and threshold_value |
| threshold_value | float | NN | The numeric threshold against which the metric is compared; units depend on the metric (count, percentage, seconds, etc.) |
| time_window_minutes | integer | NN, default 60 | Rolling time window in minutes over which the metric is evaluated |
| notification_method | enum(Email, SMS, Webhook, In-App) | NN | Delivery channel for the alert notification |
| recipients | json | NN | Array of delivery targets appropriate for the notification_method (email addresses, phone numbers in E.164, or webhook URLs) |
| cooldown_minutes | integer | NN, default 60 | Minimum number of minutes that must elapse between successive firings of this rule, preventing repeated alerts for the same sustained condition |
| is_active | boolean | NN, default true | Whether this rule is currently being evaluated; false disables evaluation without deleting the rule |
| last_triggered_at | timestamp_tz | | Timestamp of the most recent alert firing; used to enforce cooldown_minutes |
| trigger_count | counter | NN, default 0 | Cumulative count of times this rule has fired since creation |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Record last-modified timestamp |

---

### CallDailySummary

**UI References:** Reports > Activity Report, Reports > Calls by Source, Reports > Agent Performance, Reports > Queue Report, Reports > Daily Calls, Reports > Missed Calls, Reports > Activity Map, all 19 industry template reports

**Relationships:**
- belongs to one Account
- optionally scoped to one TrackingSource
- optionally scoped to one User (agent)
- optionally scoped to one Queue

**Notes:** This is a system-computed aggregate entity. No user or API consumer writes to this table directly. Rows are created and updated by a batch job that runs at end-of-day (or incrementally throughout the day for near-real-time dashboards) by grouping CallRecord rows by the four dimensions: date, source, agent, and queue. A null value in any dimension column represents the "all" aggregate for that dimension — for example, a row with `source_id = null` and non-null `agent_id` represents that agent's totals across all sources for that day. The unique constraint enforces that each dimension combination has exactly one summary row per account per day. The `computed_at` field records when the row was last refreshed and is used by monitoring jobs to detect stale summaries.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| summary_date | date | NN | The calendar date this row represents |
| source_id | uuid | FK(TrackingSource) | Tracking source dimension; null indicates this row aggregates across all sources |
| agent_id | uuid | FK(User) | Agent (user) dimension; null indicates this row aggregates across all agents |
| queue_id | uuid | FK(Queue) | Queue dimension; null indicates this row aggregates across all queues |
| total_calls | integer | NN, default 0 | Total number of calls received on summary_date for this dimension slice |
| answered_calls | integer | NN, default 0 | Number of calls that were answered |
| missed_calls | integer | NN, default 0 | Number of calls that were not answered (ringing abandoned or routed to voicemail without pickup) |
| voicemail_calls | integer | NN, default 0 | Number of calls that resulted in a voicemail recording |
| total_duration_secs | integer | NN, default 0 | Sum of talk time in seconds across all answered calls in this slice |
| total_ring_duration_secs | integer | NN, default 0 | Sum of ring (pre-answer) duration in seconds across all calls |
| total_hold_duration_secs | integer | NN, default 0 | Sum of hold time in seconds across all answered calls |
| avg_duration_secs | float | | Mean talk time in seconds for answered calls; derived from total_duration_secs / answered_calls |
| avg_ring_duration_secs | float | | Mean ring duration in seconds; derived from total_ring_duration_secs / total_calls |
| unique_callers | integer | NN, default 0 | Count of distinct caller phone numbers for the summary_date period (period-unique, not globally unique) |
| first_time_callers | integer | NN, default 0 | Number of callers for whom this was their first ever call to the account (globally unique) |
| repeat_callers | integer | NN, default 0 | Number of callers who had called at least once before summary_date |
| converted_calls | integer | NN, default 0 | Number of calls where CallRecord.converted is true |
| appointments_set | integer | NN, default 0 | Number of calls where CallRecord.appointment_set is true |
| computed_at | timestamp_tz | NN | Timestamp when this row was last computed or refreshed by the batch job |

**Composite unique constraint:** UQ(account_id, summary_date, source_id, agent_id, queue_id) — one row per unique combination of all four dimensions per account per day.

---

## Aggregation Strategy & Report Performance

### Why CallDailySummary Exists

The 4iiz platform is designed to serve accounts that may accumulate hundreds of thousands to tens of millions of individual call records over their lifetime. Rendering a report like the Activity Report — which displays KPI cards, a time-series chart, and a source breakdown table — requires summing call volumes, computing answer rates, and calculating average durations across potentially years of data filtered to a selected date range. Executing that aggregation on the raw `CallRecord` table at query time would produce unacceptable latency for any non-trivial account.

`CallDailySummary` solves this by pre-computing the sums and counts once per day (or incrementally during the day) and storing them in a narrow, indexed table. Report queries then sum a few hundred `CallDailySummary` rows for a given date range rather than scanning millions of `CallRecord` rows. The result is sub-second rendering for all standard report pages regardless of account size.

### The Dimension Model: Date x Source x Agent x Queue

Each `CallDailySummary` row is keyed by a four-part dimension tuple: `(account_id, summary_date, source_id, agent_id, queue_id)`. Null values in the optional dimensions act as wildcard aggregates — this is a sparse cube model rather than a fully materialized hypercube.

The sparse approach means the number of rows scales with actual usage patterns rather than with the full Cartesian product of all dimensions. An account with 10 tracking sources, 5 agents, and 3 queues would at most generate (10 + 1) x (5 + 1) x (3 + 1) = 264 rows per day to cover all dimension combinations (including the "all" null rows). In practice, not every dimension combination will have activity on every day, so the actual row count is lower.

Report queries filter on whichever dimensions are relevant and aggregate the matching rows:

- **Activity Report** — filters `source_id IS NULL AND agent_id IS NULL AND queue_id IS NULL`, grouping by `summary_date` to produce daily totals. One row per day equals the "all" aggregate.
- **Calls by Source** — filters `agent_id IS NULL AND queue_id IS NULL`, grouping by `source_id` for the selected date range. Produces one result row per source.
- **Agent Performance** — filters `source_id IS NULL AND queue_id IS NULL`, grouping by `agent_id`. Produces one result row per agent.
- **Queue Report** — filters `source_id IS NULL AND agent_id IS NULL`, grouping by `queue_id`. Produces one result row per queue.
- **Daily Calls** — same as Activity Report but chart is the primary output, with first_time_callers vs. repeat_callers breakdown.
- **Activity Map** — joins source geographic metadata to summary rows, grouping by state/region to produce heat map intensities.
- **All 19 industry template reports** — use the same queries as the above base reports, with industry-specific KPI card labels applied at the presentation layer (e.g., "Policies Quoted" instead of "Converted Calls" for Insurance).

### The 30 Report Pages and Their Query Paths

Of the 30 report pages in the Reports section, 28 map cleanly to queries against `CallDailySummary` alone or joined with dimension tables (TrackingSource, User, Queue) for labels. These queries are fast because they operate on the pre-aggregated data.

Two categories of reports may require additional joins or fallback to raw data:

1. **Missed Calls report** — while aggregate counts come from `CallDailySummary.missed_calls`, the detail panel listing individual missed calls with caller ID and callback status requires a filtered query against `CallRecord`.
2. **Scoring report** — the per-agent scoring calculation defined by `ScoringConfig` weights may be applied either to `CallDailySummary` aggregates (for period totals) or to individual `CallRecord` rows (for call-level granularity). The score stored on `CallRecord.score` is the per-call value; the agent score shown in the Scoring report is a weighted average of those values.

### Pre-Aggregation vs. Ad-Hoc Queries

The pre-aggregation strategy is optimal for the 28+ standard reports that operate on fixed dimension combinations. The trade-off is storage overhead and the need for a reliable batch refresh pipeline. If the batch job fails or runs late, `CallDailySummary` rows become stale (detectable via `computed_at`) and reports may undercount recent calls.

To mitigate staleness, the batch job should run both end-of-day (for finalized daily totals) and intraday (for near-real-time dashboard updates). The `computed_at` timestamp allows the UI to display a "data as of" indicator when summaries are more than a configured threshold old.

### Custom Reports and Raw Table Fallback

`CustomReport` rows define arbitrary filter combinations that may not align with the pre-computed dimension slices in `CallDailySummary`. A custom report filtering by a specific tag, a call disposition value, a keyword in the transcript, or a caller geographic region cannot be satisfied by the summary table because those dimensions are not stored there.

When the report engine evaluates a `CustomReport`, it must inspect the `filters` JSON to determine whether all active filter dimensions are present in `CallDailySummary`. If they are, the query is routed to the summary table for performance. If any filter dimension falls outside the four pre-aggregated dimensions (date, source, agent, queue), the engine falls back to querying the raw `CallRecord` table — joined as needed to `CallTag`, `Appointment`, `TranscriptSegment`, and other related entities.

The fallback path carries a performance cost proportional to the account's call volume and the selectivity of the filters. For large accounts, custom reports with non-dimension filters should display a latency warning and may be subject to a background execution model (enqueue, compute, notify when ready) rather than synchronous rendering.
