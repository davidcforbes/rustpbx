# 4iiz Data Integration Flows

> **Status:** Draft
> **Date:** 2026-02-26
> **Prerequisites:**
> - [Data Element Dictionary](00-data-dictionary-index.md)
> - [Data Storage Architecture](../plans/2026-02-25-data-storage-architecture-design.md)
> **Purpose:** Trace every major data flow end-to-end from trigger through DB operations, API endpoints, and UI touchpoints. Each flow identifies testable checkpoints for automated verification.

---

## Connection Pool Reference

| Pool | Purpose | Max Connections | Statement Timeout |
|------|---------|:---:|:---:|
| `call_processing` | CDR inserts, routing lookups | 20 | 5s |
| `api_crud` | UI/API config reads and writes | 10 | 30s |
| `background` | Exports, bulk sends, aggregation, transcription | 5 | 300s |
| `reports` | Dashboard and report queries | 5 | 60s |

## Cache Reference

| Cache | Backing Store | Invalidation |
|-------|--------------|-------------|
| Routing config (moka) | PostgreSQL 3NF tables | PG LISTEN/NOTIFY on config write |
| Active calls (moka) | PostgreSQL UNLOGGED `active_calls` | PG LISTEN/NOTIFY on state change |
| Presence (moka) | PostgreSQL UNLOGGED `presence` | PG LISTEN/NOTIFY on state change |
| Locations (moka) | PostgreSQL UNLOGGED `locations` | PG LISTEN/NOTIFY on REGISTER |

---

## A. Real-Time Call Processing

Overview: These are the latency-critical hot-path flows that handle live phone calls. They use the `call_processing` connection pool and read routing config exclusively from the in-process moka cache. No flow in this category should require more than 5ms of DB access time.

### A1: Inbound Call Routing

**Trigger:** SIP INVITE arrives on a tracking number

**Data Flow:**

```
SIP INVITE received by RustPBX SIP stack
  └─ Extract To: number → moka cache lookup → TrackingNumber record
       └─ TrackingNumber.tracking_source_id → moka cache → TrackingSource (source attribution)
  └─ TrackingNumber.call_settings_id → moka cache → CallSettings
       └─ Resolve: recording_enabled, whisper_message, caller_id_mode
  └─ TrackingNumber.routing_type → moka cache → evaluate destination chain:
       ├─ Schedule   → check current UTC time against cached ScheduleHours
       │                └─ open path → inner routing_type recurse
       │                └─ closed path → voicemail or closed message
       ├─ VoiceMenu  → play greeting audio → await DTMF input
       │                └─ match VoiceMenuOption.key → route to option's destination
       ├─ SmartRouter → iterate SmartRouterRules by priority
       │                └─ first condition match → extract destination
       ├─ GeoRouter  → lookup caller area code → match GeoRouterRule.area_codes
       │                └─ matched rule → extract destination
       ├─ Queue      → evaluate queue strategy (Ring All / Round Robin / Longest Idle)
       │                └─ select available agent(s) → route to agent endpoint
       └─ Direct     → ReceivingNumber.phone_number → forward
  └─ INSERT active_calls (UNLOGGED): status=Ringing, answered_at=NULL
  └─ pg_notify('active_call_changed', call_id)
       └─ moka cache updated across all server instances
  └─ SIP INVITE forwarded to agent endpoint (SIP or WebRTC)
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| TrackingNumber lookup | SELECT (moka hit) | `tracking_numbers` | none | Fallback to `call_processing` on cache miss |
| TrackingSource lookup | SELECT (moka hit) | `tracking_sources` | none | Fallback to `call_processing` on cache miss |
| CallSettings lookup | SELECT (moka hit) | `call_settings` | none | Fallback to `call_processing` on cache miss |
| Routing chain evaluation | SELECT (moka hit) | `schedules`, `voice_menus`, `smart_router_rules`, `geo_router_rules`, `queues` | none | All routing reads from moka; zero DB queries on warm cache |
| Active call INSERT | INSERT | `active_calls` | `call_processing` | UNLOGGED table; status=Ringing |
| State notification | pg_notify | — | `call_processing` | Channel: `active_call_changed` |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| WS | `/api/v1/calls/live` | Real-time active call feed | — (subscribe) | `call.ringing` event with call_id, caller, callee, tracking_number_id |
| GET | `/api/v1/calls/active` | Poll current active calls list | — | Array of active call objects with status |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Coaching page — Active Calls panel | New call row appears with status "Ringing", caller ID, and tracking source label |
| Real Time report — Summary cards | Calls In Queue or Calls Ringing counter increments |
| PhoneDrawer (agent softphone) | Rings with inbound caller info; displays whisper message if configured |
| Queue dashboard | `calls_waiting` counter increments if call enters a queue |

**Testable Checkpoints:**

- [ ] Route evaluation returns the correct destination for a TrackingNumber configured for each routing type (Direct, Schedule, VoiceMenu, SmartRouter, GeoRouter, Queue)
- [ ] `active_calls` row exists with `status=Ringing`, correct `caller_phone`, `callee_number`, and `account_id` immediately after INVITE is processed
- [ ] WebSocket `/api/v1/calls/live` emits a `call.ringing` event to all connected subscribers within 200ms of INVITE receipt
- [ ] Agent's SIP or WebRTC endpoint receives the forwarded SIP INVITE
- [ ] `recording_enabled` and `whisper_message` from CallSettings are applied correctly per tracking number configuration
- [ ] `tracking_source_id` on the `active_calls` row matches the correct TrackingSource for the dialed number
- [ ] Schedule routing selects the open-hours path during business hours and the closed path outside them
- [ ] GeoRouter correctly matches a caller's area code to the configured GeoRouterRule destination
- [ ] SmartRouter evaluates rules in priority order and routes on the first match, ignoring subsequent rules
- [ ] Cache miss fallback to `call_processing` pool succeeds and re-populates the moka cache

---

### A2: Outbound Call Initiation

**Trigger:** Agent clicks dial in PhoneDrawer or CallDetailPanel

**Data Flow:**

```
Agent submits dial request via POST /api/v1/calls/dial
  └─ Authenticate agent session → verify account membership and dial permission
  └─ Resolve caller ID:
       ├─ If tracking_number_id provided → load CallSettings from moka cache
       │    └─ Apply caller_id_mode (tracking number, account default, or agent DID)
       └─ If no tracking_number_id → use account default outbound caller ID
  └─ Resolve outbound trunk:
       └─ Load trunk config from moka cache → select trunk by priority/account assignment
  └─ Send SIP INVITE to trunk with resolved caller ID and target number
  └─ INSERT active_calls (UNLOGGED):
       status=Ringing, direction=Outbound, agent_id=?, caller_phone=resolved_caller_id,
       callee_number=target_number, tracking_number_id=? (nullable)
  └─ pg_notify('active_call_changed', call_id)
  └─ Target answers:
       └─ UPDATE active_calls SET status=Active, answered_at=now()
       └─ pg_notify('active_call_changed', call_id)
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Agent auth check | SELECT (moka session cache) | `users`, `account_memberships` | none | Fallback to `api_crud` on session miss |
| CallSettings lookup | SELECT (moka hit) | `call_settings` | none | Only if tracking_number_id provided |
| Trunk config lookup | SELECT (moka hit) | `trunks` | none | Fallback to `call_processing` on cache miss |
| Active call INSERT | INSERT | `active_calls` | `call_processing` | UNLOGGED; direction=Outbound |
| State notification (ringing) | pg_notify | — | `call_processing` | Channel: `active_call_changed` |
| Active call UPDATE | UPDATE | `active_calls` | `call_processing` | On answer: status=Active, answered_at |
| State notification (active) | pg_notify | — | `call_processing` | Channel: `active_call_changed` |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | `/api/v1/calls/dial` | Initiate outbound call | `{ target_number, tracking_number_id?, caller_id? }` | `201 { call_id, status: "ringing" }` |
| WS | `/api/v1/calls/live` | Real-time state updates for the call | — (subscribe) | `call.ringing`, `call.active` events with call_id |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| PhoneDrawer | Transitions through: idle → dialing → ringing → connected; displays target number and elapsed call timer |
| Coaching page — Active Calls panel | Outbound call row appears with direction indicator and agent name |
| Real Time report — Summary cards | Outbound call count increments |

**Testable Checkpoints:**

- [ ] POST `/api/v1/calls/dial` returns `201` with a valid `call_id` UUID
- [ ] `active_calls` row created with `direction=Outbound`, correct `agent_id`, `caller_phone`, and `callee_number`
- [ ] SIP INVITE is sent to the correct trunk with the resolved caller ID in the From header
- [ ] On target answer, `active_calls.status` transitions from `Ringing` to `Active` and `answered_at` is populated
- [ ] WebSocket emits `call.ringing` then `call.active` events in correct sequence
- [ ] PhoneDrawer UI progresses through each state in order without requiring page reload
- [ ] Dial request with an unauthenticated session returns `401`
- [ ] Dial request for an agent without dial permission returns `403`

---

### A3: Call Completion + CDR Write

**Trigger:** SIP BYE received from either party, or call fails or times out

**Data Flow:**

```
SIP BYE processed by RustPBX SIP stack
  └─ Read active_calls row for this call_id (moka cache or call_processing pool)
  └─ Calculate durations:
       ├─ ring_duration_secs  = answered_at - created_at (NULL if never answered)
       ├─ hold_duration_secs  = sum of all hold intervals from active_calls.hold_segments
       └─ total_duration_secs = now() - created_at
  └─ Determine call outcome status: Answered / Missed / Voicemail / Failed
  └─ Resolve is_first_time_caller:
       └─ check moka recent-caller cache for (account_id, caller_phone)
            └─ cache miss → SELECT call_records WHERE caller_phone = ? AND account_id = ? LIMIT 1
            └─ result cached for TTL; boolean stored on CDR
  └─ Snapshot denormalized fields from moka cache:
       ├─ source_name  ← TrackingSource.name (point-in-time snapshot)
       ├─ agent_name   ← User.display_name (point-in-time snapshot)
       └─ queue_name   ← Queue.name (point-in-time snapshot, nullable)
  └─ INSERT call_records (immutable CDR) — call_processing pool
  └─ INSERT call_annotations (empty seed row) — call_processing pool
  └─ DELETE FROM active_calls WHERE call_id = ? — call_processing pool
  └─ pg_notify('active_call_changed', call_id) [removal event]
  └─ Emit "call.completed" to tokio broadcast channel → triggers Flow A4
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Active call read | SELECT (moka hit) | `active_calls` | none | Fallback to `call_processing` on miss |
| First-time caller check | SELECT (indexed) | `call_records` | `call_processing` | Only on moka cache miss; `(account_id, caller_phone)` index |
| CDR insert | INSERT | `call_records` | `call_processing` | Partitioned by `started_at` month; immutable after insert |
| Annotations seed | INSERT | `call_annotations` | `call_processing` | Empty row; FK to call_records.call_id |
| Active call delete | DELETE | `active_calls` | `call_processing` | UNLOGGED table |
| State notification | pg_notify | — | `call_processing` | Channel: `active_call_changed`; signals removal |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| WS | `/api/v1/calls/live` | Pushes `call.completed` event; call removed from active list | — | `call.completed` event with call_id and summary fields |
| GET | `/api/v1/activities/calls` | Call appears in activity log | `?account_id&page` | Paginated call list including new CDR |
| GET | `/api/v1/activities/calls/{id}` | Fetch full CDR for the completed call | — | Full call_record object with all CDR fields |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Coaching page — Active Calls panel | Call row disappears from active list |
| Activities > Calls page | New row appears at top if Auto Load is enabled; otherwise badge count increments |
| Real Time report — Summary cards | Active call count decrements; completed call count increments |
| PhoneDrawer | Returns to idle state; wrap-up timer starts if wrap-up mode is configured |

**Testable Checkpoints:**

- [ ] `call_records` row exists with all CDR fields populated: `call_id`, `account_id`, `caller_phone`, `callee_number`, `started_at`, `answered_at`, `ended_at`, `total_duration_secs`, `ring_duration_secs`, `status`, `tracking_source_id`, `agent_id`, `tracking_number_id`
- [ ] `call_annotations` seed row exists with correct `call_id` FK and empty annotation fields
- [ ] `active_calls` row is deleted; no orphan active call rows for this `call_id`
- [ ] `total_duration_secs` equals `ended_at - started_at` in seconds
- [ ] `ring_duration_secs` equals `answered_at - started_at`; `NULL` if call was never answered
- [ ] `hold_duration_secs` reflects sum of all hold intervals; zero for calls with no hold
- [ ] `is_first_time_caller=true` for a caller_phone with no prior CDR in the account; `false` for repeat callers
- [ ] Denormalized `source_name`, `agent_name`, and `queue_name` snapshot the values current at call time, not the values at query time
- [ ] `call.completed` event is emitted to the tokio broadcast channel and received by all A4 listeners
- [ ] WebSocket pushes `call.completed` event to connected clients within 500ms of BYE processing

---

### A4: Post-Call Event Cascade

**Trigger:** `call.completed` event received on tokio broadcast channel (emitted by Flow A3)

**Data Flow:**

```
"call.completed" event received from tokio broadcast channel
  └─ Dispatch all steps concurrently as independent async tasks:

  [Task 1 — Call Flow Events]
  └─ Collect call_flow_events captured in-memory during the call
       └─ INSERT batch into call_flow_events — background pool

  [Task 2 — Visitor Session]
  └─ Check if DNI (Dynamic Number Insertion) data is present in call metadata
       └─ If yes: INSERT call_visitor_sessions — background pool
       └─ If no: skip

  [Task 3 — Trigger Evaluation]
  └─ Load active trigger rules from moka cache (zero DB)
       └─ Evaluate each Trigger.conditions against call_record fields
       └─ For each matching Trigger:
            ├─ TriggerAction: send_sms  → enqueue outbound SMS (see Flow E1)
            ├─ TriggerAction: apply_tag → INSERT call_tags — api_crud pool
            ├─ TriggerAction: webhook   → INSERT webhook_deliveries (Pending) — background pool
            └─ TriggerAction: notify    → INSERT notifications — api_crud pool

  [Task 4 — Webhook Dispatch]
  └─ For each webhook_deliveries row with status=Pending from Task 3:
       └─ Fire HTTP POST to webhook_subscriptions.url asynchronously
       └─ UPDATE webhook_deliveries SET status=Success/Failed, response_code, responded_at

  [Task 5 — Transcription Queue]
  └─ Read CallSettings.transcription_enabled from moka cache
       └─ If enabled: submit ASR job to background task queue (no immediate DB write)
       └─ Job processor handles recording retrieval and ASR API call (see Flow C1)

  [Task 6 — Aggregation Update]
  └─ UPDATE call_daily_summary for dimensions: (date, account_id, tracking_source_id, agent_id, queue_id)
       └─ INSERT ON CONFLICT UPDATE: increment call_count, answered_count, missed_count, total_duration_secs
       └─ background pool

  [Task 7 — Counter Buffer Flush]
  └─ Increment in-process atomic counters:
       └─ TrackingSource.call_count_buffer += 1
       └─ Agent.calls_handled_buffer += 1 (if answered)
       └─ Periodic flush task writes buffer deltas to DB (batched, not per-call)

  [Task 8 — Missed Call Notification]
  └─ If call_record.status = Missed:
       └─ Determine notification recipients (account owner, assigned agent, queue supervisors)
       └─ INSERT notifications for each recipient — api_crud pool
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Call flow events | INSERT (batch) | `call_flow_events` | `background` | Batch insert of all events captured during call |
| Visitor session | INSERT | `call_visitor_sessions` | `background` | Skipped if no DNI data on the call |
| Trigger rule load | SELECT (moka hit) | `triggers`, `trigger_conditions` | none | Zero DB on warm cache |
| Tag application | INSERT | `call_tags` | `api_crud` | One row per auto-applied tag |
| Webhook delivery record | INSERT | `webhook_deliveries` | `background` | status=Pending; one row per matched subscription |
| Webhook delivery update | UPDATE | `webhook_deliveries` | `background` | After HTTP response: status=Success/Failed, response_code |
| ASR job | (none) | — | — | Queued to in-process task channel; DB write deferred to Flow C1 |
| Aggregation upsert | INSERT ON CONFLICT UPDATE | `call_daily_summary` | `background` | Dimension key: (date, account_id, source_id, agent_id, queue_id) |
| Counter increment | (atomic in-process) | — | — | Flushed in batch by a separate periodic task |
| Missed call notification | INSERT | `notifications` | `api_crud` | Only when call_record.status = Missed |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | `/api/v1/activities/calls/{id}/flow-events` | Retrieve call flow events after cascade completes | — | Ordered array of flow event objects |
| GET | `/api/v1/activities/calls/{id}/visitor` | Retrieve visitor session linked to the call | — | Visitor session object or 404 |
| GET | `/api/v1/activities/calls/{id}/tags` | Retrieve tags applied by triggers | — | Array of tag objects |
| WS | `/api/v1/notifications` | Push missed call notification to relevant users | — | `notification.new` event with type and call_id |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| CallDetailPanel > Flow tab | Call flow events rendered in chronological order after cascade completes |
| CallDetailPanel > Visitor Detail tab | DNI visitor session data rendered if present |
| CallDetailPanel > Tags section | Auto-applied tags visible alongside manually applied tags |
| Activities > Calls page — Tags column | Tag pills appear on the call row after cascade completes |
| Notification bell | Badge count increments; missed call notification appears in dropdown |

**Testable Checkpoints:**

- [ ] All `call_flow_events` rows are written with correct `call_id`, sequential `event_order`, and event type values matching the call's routing path
- [ ] `call_visitor_sessions` row is created when DNI metadata is present on the call; no row is created when DNI data is absent
- [ ] A trigger with conditions matching the completed call fires its actions; a trigger with non-matching conditions does not fire
- [ ] `call_tags` rows are created for each auto-applied tag from a matching trigger; tags are not duplicated on retry
- [ ] `webhook_deliveries` row is created with `status=Pending` before the HTTP call fires, and updated to `status=Success` or `status=Failed` after the response is received
- [ ] `webhook_deliveries.response_code` reflects the actual HTTP status code returned by the target URL
- [ ] `call_daily_summary` row for the correct dimension combination (date, account, source, agent, queue) is incremented by exactly 1 for `call_count`
- [ ] `answered_count` increments only for answered calls; `missed_count` increments only for missed calls
- [ ] A `notifications` row is created for each intended recipient when `call_record.status = Missed`; no notification is created for answered calls
- [ ] Failure in any single task (e.g., webhook HTTP timeout) does not prevent other tasks from completing
- [ ] Counter accumulator values increase after the periodic flush task runs

---

### A5: Active Call State Lifecycle

**Trigger:** Any state change during a live call: Ringing → Active → On Hold → Transferring → Wrapping

**Data Flow:**

```
SIP event or agent API action triggers a state change
  └─ Validate the requested transition against the allowed state machine:
       Ringing     → Active        (callee answers)
       Active      → On Hold       (agent presses hold)
       On Hold     → Active        (agent resumes)
       Active      → Transferring  (agent initiates transfer)
       Transferring → Active       (transfer completes; new agent connected)
       Active      → Wrapping      (hangup received; wrap-up period begins)
       Wrapping    → [deleted]     (agent closes wrap-up → triggers Flow A3)
  └─ If transition is invalid: return 409 Conflict; no DB write
  └─ If transition is valid:
       └─ UPDATE active_calls:
            ├─ SET status = new_state
            ├─ If → On Hold:    SET hold_start = now()
            ├─ If → Active (from Hold): SET hold_end = now(), accumulate hold_segments
            ├─ If → Transferring: SET transfer_target = ?, transfer_initiated_at = now()
            └─ If → Wrapping:   SET wrap_started_at = now()
       └─ pg_notify('active_call_changed', call_id)
       └─ moka cache updated across all instances
       └─ WS push: state change event to subscribed UI clients
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Current state read | SELECT (moka hit) | `active_calls` | none | Used to validate transition; fallback to `call_processing` on miss |
| State transition UPDATE | UPDATE | `active_calls` | `call_processing` | UNLOGGED table; includes timestamp fields for hold/transfer/wrap |
| State notification | pg_notify | — | `call_processing` | Channel: `active_call_changed`; payload includes call_id and new status |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| PUT | `/api/v1/calls/active/{call_id}/hold` | Toggle hold on or off | `{ action: "hold" \| "resume" }` | `200 { call_id, status }` or `409` on invalid transition |
| PUT | `/api/v1/calls/active/{call_id}/transfer` | Initiate blind or attended transfer | `{ target_number? , target_agent_id? }` | `200 { call_id, status: "transferring" }` |
| PUT | `/api/v1/calls/active/{call_id}/complete` | Agent closes wrap-up period | — | `200` — triggers Flow A3 |
| WS | `/api/v1/calls/live` | Pushes state change events to subscribers | — | `call.state_changed` event with call_id and new status |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Coaching page — Active Calls panel | Call status badge updates in real time (Ringing / Active / On Hold / Transferring / Wrapping) |
| PhoneDrawer | Call controls reflect current state: hold/resume button toggles, transfer controls appear, hangup disabled during transfer handoff |
| Real Time report — Status distribution | Active / On Hold / Wrapping counts update in real time |
| Queue dashboard | `agents_on_call` count changes on Active ↔ On Hold transitions |

**Testable Checkpoints:**

- [ ] Each valid state transition updates the `active_calls` row to the correct `status` value
- [ ] `hold_start` is set when transitioning to On Hold; `hold_end` is set and hold interval is appended to `hold_segments` when resuming
- [ ] `transfer_target` and `transfer_initiated_at` are populated when transitioning to Transferring
- [ ] `wrap_started_at` is set when transitioning to Wrapping
- [ ] An invalid state transition (e.g., On Hold → Wrapping, Wrapping → On Hold) returns `409 Conflict` and does not write to the DB
- [ ] `pg_notify` fires on every valid state change, not on rejected transitions
- [ ] WebSocket clients subscribed to `/api/v1/calls/live` receive the `call.state_changed` event within 100ms of the DB UPDATE
- [ ] Hold duration accumulated across multiple hold/resume cycles equals the sum of all individual hold intervals
- [ ] After transfer completes, `active_calls.agent_id` reflects the new agent, not the original transferring agent

---

### A6: Agent State Transitions

**Trigger:** Agent logs in, logs out, goes on break, completes a call, or manually changes availability status

**Data Flow:**

```
Agent action or system event triggers a status change:
  Agent-initiated:  PUT /api/v1/agents/{id}/status with { status, reason? }
  System-initiated: call answered → "On Call"; call wrap-up closed → "After Call Work" → "Available"

  └─ Read current presence row from moka cache (or UNLOGGED presence table on cache miss)
       └─ Calculate duration in previous state:
            duration_secs = now() - presence.last_updated
  └─ INSERT agent_state_log:
       └─ { agent_id, previous_status, duration_secs, started_at: last_updated, ended_at: now(), reason? }
  └─ UPDATE presence:
       └─ SET status = new_status, last_updated = now(), reason = ?
  └─ pg_notify('presence_changed', agent_id)
  └─ moka cache updated across all instances
  └─ WS push: presence update event to subscribed UI clients
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Current presence read | SELECT (moka hit) | `presence` | none | Fallback to `call_processing` pool on cache miss; UNLOGGED table |
| State log INSERT | INSERT | `agent_state_log` | `call_processing` | Backfill pattern: logs the previous state's duration |
| Presence UPDATE | UPDATE | `presence` | `call_processing` | UNLOGGED table; updates status and last_updated timestamp |
| Presence notification | pg_notify | — | `call_processing` | Channel: `presence_changed`; payload includes agent_id and new status |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| PUT | `/api/v1/agents/{id}/status` | Manual agent status change | `{ status: "Available" \| "On Break" \| "Offline" \| "After Call Work", reason? }` | `200 { agent_id, status, last_updated }` |
| GET | `/api/v1/agents` | List all agents with current status | `?account_id` | Array of agent objects with live status from moka cache |
| GET | `/api/v1/agents/{id}/state-log` | Historical agent state log | `?from&to&page` | Paginated array of state log entries with durations |
| WS | `/api/v1/agents/live` | Real-time presence updates | — (subscribe) | `agent.status_changed` event with agent_id and new status |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Agent header / PhoneDrawer status selector | Reflects the agent's current status; updates immediately on change |
| Coaching page — Agent Status panel | Agent status badges update in real time for all visible agents |
| Queue dashboard | `agents_available` count increments/decrements as agents move to/from Available |
| Reports > Agent Performance | State log feeds availability percentage and on-break duration calculations |

**Testable Checkpoints:**

- [ ] `agent_state_log` row is created for every status transition, with `previous_status` matching the status before the transition and `duration_secs` equal to `ended_at - started_at`
- [ ] `presence` row is updated to the new status with `last_updated = now()` (within 1 second of the transition event)
- [ ] `pg_notify` fires on every status change with the correct agent_id and new status in the payload
- [ ] WebSocket clients subscribed to `/api/v1/agents/live` receive the `agent.status_changed` event within 100ms of the presence UPDATE
- [ ] GET `/api/v1/agents` reflects the new status immediately after the UPDATE without requiring a cache flush delay
- [ ] System-triggered transition from call answer fires automatically (no manual API call required) and sets status to `On Call`
- [ ] System-triggered transition from wrap-up close fires automatically and sequences through `After Call Work` → `Available`
- [ ] `duration_secs` on the state log correctly accumulates time even when an agent holds a state for over an hour
- [ ] A manual status change during an active call is either rejected with `409` or queued, depending on configuration; it is never silently dropped

---

## B. Messaging & Multi-Channel

Overview: These flows handle non-voice communication channels — SMS, forms, chat, and fax. They share the same post-event pattern as calls (event bus → triggers → webhooks → aggregation) but have channel-specific processing.

---

### B1: Inbound SMS Receipt

**Trigger:** SMS message received on a tracking number (from carrier webhook)

**Data Flow:**

```
1. Carrier webhook delivers inbound SMS to platform API
2. Lookup tracking number → TrackingSource for attribution (moka cache)
3. Check DNC/DNT lists for sender number
   - If on DNT list: increment rejected_count (in-process counter buffer), log, and stop
4. INSERT text_messages (individual message record)
5. INSERT or UPDATE text_records (conversation summary — upsert by contact_phone + tracking_number_id)
6. Emit "sms.received" event to tokio broadcast channel
7. Trigger evaluation: match against active triggers with event=SMS Received (conditions from moka cache)
8. If STOP/UNSUBSCRIBE keyword detected in message body: INSERT dnt_entry automatically
9. INSERT notification for assigned agent or queue
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | Cache read | tracking_numbers | moka | Zero DB reads for warm cache |
| 3 | SELECT | dnt_entries | api_crud | Pre-loaded into moka cache if DNT list is small; fallback to DB |
| 4 | INSERT | text_messages | call_processing | direction=Inbound, status=Received |
| 5 | INSERT/UPDATE | text_records | call_processing | Upsert on (contact_phone, tracking_number_id); sets last_message_preview, last_message_at |
| 7 | Cache read + INSERT | triggers / trigger_actions | moka + background | Conditions evaluated in-process; action inserts via background pool |
| 8 | INSERT | dnt_entries | api_crud | Only if STOP keyword detected |
| 9 | INSERT | notifications | api_crud | Targets agent_id or queue_id derived from tracking number assignment |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/webhooks/sms/inbound | Carrier webhook receiver (internal, token-auth) | Carrier SMS payload | 200 OK |
| GET | /api/v1/activities/texts | Texts list — new row appears after insert | `?page&limit&direction&date_range` | Paginated text_records list |
| GET | /api/v1/activities/texts/{id} | Conversation thread detail | — | text_record + text_messages array |
| GET | /api/v1/calls/{call_id}/messages | Messages in CallDetailPanel > Text tab (contact match) | — | text_messages for contact |
| WS | /api/v1/notifications | Push notification to assigned agent | — | Notification event stream |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Activities > Texts | New row appears with contact phone, tracking number, direction=Inbound, preview of message body |
| CallDetailPanel > Text Message tab | Message thread updated if contact matches an existing call record |
| Notification bell | Badge count increments; clicking opens the conversation |
| PhoneDrawer | SMS notification indicator appears for the relevant contact |

**Testable Checkpoints:**

- [ ] text_messages row created with direction=Inbound and correct tracking_number_id
- [ ] text_records row upserted: last_message_preview matches received body, last_message_at updated
- [ ] DNT check prevents INSERT into text_messages for a blocked sender; processing halts at step 3
- [ ] rejected_count counter buffer incremented for blocked sender (visible after periodic flush)
- [ ] STOP keyword in message body auto-creates dnt_entry for sender e164
- [ ] Triggers with sms.received event type fire; trigger_actions executed via background pool
- [ ] Notification row created and delivered over WebSocket to correct agent/queue

---

### B2: Outbound SMS (Agent-Initiated)

**Trigger:** Agent sends SMS from CallDetailPanel > Text Message tab or PhoneDrawer

**Data Flow:**

```
1. Agent submits message via API with target phone, body, and tracking_number_id
2. Validate: check DNC/DNT lists for target number
   - If blocked: return 403 to UI — do not submit to carrier
3. Submit message to carrier SMS API (synchronous or async depending on carrier SDK)
4. INSERT text_messages with direction=Outbound, status=Pending
5. INSERT or UPDATE text_records (conversation summary)
6. Carrier async status callback received
7. UPDATE text_messages SET status=Delivered|Failed, delivered_at=now() (or failed_at)
8. UPDATE text_records SET last_message_at, delivery_status
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | SELECT | dnt_entries | api_crud | Checked before any carrier interaction |
| 4 | INSERT | text_messages | api_crud | Inserted after carrier accepts message; carries carrier message_sid |
| 5 | INSERT/UPDATE | text_records | api_crud | Upsert on (contact_phone, tracking_number_id) |
| 7 | UPDATE | text_messages | background | Async callback from carrier; low-urgency pool appropriate |
| 8 | UPDATE | text_records | background | Sets delivery_status and last_message_at |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/messages/send | Send outbound SMS | `{to, body, tracking_number_id, [call_id]}` | 201 Created with text_message_id |
| POST | /api/v1/webhooks/sms/status | Carrier delivery status callback (internal) | Carrier status payload | 200 OK |
| GET | /api/v1/activities/texts/{id} | Updated conversation thread | — | text_record + text_messages array |
| GET | /api/v1/calls/{call_id}/messages | Conversation in CallDetailPanel context | — | text_messages for contact |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| CallDetailPanel > Text Message tab | Outbound message appears as right-aligned bubble with status=Pending; updates to Delivered or Failed on callback |
| Activities > Texts | New outbound row appears in list with direction=Outbound |
| PhoneDrawer | SMS compose interface; send button disabled if DNC/DNT match detected |

**Testable Checkpoints:**

- [ ] DNC/DNT check blocks message to a listed number; API returns 403 with reason code
- [ ] text_messages row created with direction=Outbound, status=Pending, and carrier message_sid
- [ ] Carrier API receives correct to, body, and from (tracking number) values
- [ ] Delivery status callback updates text_messages.status to Delivered or Failed
- [ ] text_records.last_message_at and delivery_status reflect final state
- [ ] UI renders status update without page reload (WebSocket push or polling)

---

### B3: Bulk Message Campaign

**Trigger:** Admin starts a bulk SMS campaign from Flows > Bulk Messages page

**Data Flow:**

```
1. Admin creates campaign: sets label, message body, contact_list_id, sender tracking number, optional schedule
2. INSERT bulk_messages with status=Draft
3. Admin activates campaign:
   - If scheduled: UPDATE bulk_messages SET status=Scheduled, scheduled_at=target_time
   - If immediate: UPDATE bulk_messages SET status=Sending
4. Background worker picks up campaign (on scheduled_at or immediately):
   a. SELECT contact_list_members for the campaign's contact_list_id
   b. For each recipient:
      i.  Check DNC/DNT (skip blocked numbers — increment skipped_count in-process)
      ii. Submit to carrier SMS API
      iii.INSERT text_messages with bulk_message_id foreign key
      iv. Increment sent_count or failed_count (in-process counter)
   c. Periodic counter flush: UPDATE bulk_messages SET sent_count=?, skipped_count=?, failed_count=?
5. Carrier status callbacks update individual text_messages.status (delivered_count incremented in-process)
6. On worker completion: UPDATE bulk_messages SET status=Completed, completed_at=now(), final counters
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | INSERT | bulk_messages | api_crud | status=Draft on creation |
| 3 | UPDATE | bulk_messages | api_crud | status transition; scheduled_at set if deferred |
| 4a | SELECT | contact_list_members | background | Full list load; background pool allows long query timeout (300s) |
| 4b-i | SELECT (batch) | dnt_entries | background | Batch lookup by e164 array to minimize round trips |
| 4b-iii | INSERT (batch) | text_messages | background | Batched per configurable chunk size; bulk_message_id linked |
| 4c | UPDATE | bulk_messages | background | Periodic flush of in-process counters (e.g., every 100 sends) |
| 5 | UPDATE | text_messages | background | Carrier status callbacks; delivered_count flushed periodically |
| 6 | UPDATE | bulk_messages | background | Final status=Completed and all counters committed |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/campaigns/bulk-messages | Create campaign | `{label, body, contact_list_id, tracking_number_id, scheduled_at?}` | 201 Created with bulk_message_id |
| PUT | /api/v1/campaigns/bulk-messages/{id}/activate | Activate draft (immediate or scheduled send) | — | 200 OK with updated status |
| PUT | /api/v1/campaigns/bulk-messages/{id}/cancel | Cancel in-progress or scheduled campaign | — | 200 OK; status=Cancelled |
| GET | /api/v1/campaigns/bulk-messages/{id} | Campaign detail with live progress counters | — | bulk_message record with counters |
| GET | /api/v1/campaigns/bulk-messages | List all campaigns | `?status&page&limit` | Paginated campaign list |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Flows > Bulk Messages | Campaign row displays status badge and live sent/total progress (polling or WebSocket) |
| Campaign detail view | Progress bar rendered from sent_count / total_recipients; delivery stats (delivered, failed, skipped) shown as counters |
| Contact list selector | Dropdown populated from contact_lists; shows member count for selected list |

**Testable Checkpoints:**

- [ ] Status lifecycle enforced: Draft → Scheduled or Sending → Completed (or Cancelled)
- [ ] DNC/DNT recipients skipped: no text_messages row created, skipped_count incremented
- [ ] sent_count + failed_count + skipped_count equals total contact_list_members at completion
- [ ] Cancel transitions status to Cancelled; background worker halts processing after current batch
- [ ] Already-sent text_messages are NOT deleted or reversed on cancel
- [ ] Scheduled campaign does not start processing before scheduled_at timestamp
- [ ] Each text_messages row has bulk_message_id foreign key set to campaign id
- [ ] Counter flush updates bulk_messages at least once per 100 processed recipients

---

### B4: Form Submission

**Trigger:** Web form POST from customer website (via tracking pixel or JS integration)

**Data Flow:**

```
1. Form data received via public API endpoint (from customer's website, rate-limited)
2. Identify integration: lookup FormReactorEntry by form_key or tracking_number (moka cache)
3. Attribute source: resolve tracking_number → TrackingSource if phone field present
4. INSERT form_records with form_data JSON, status=New, source attribution fields
5. Emit "form.submitted" event to tokio broadcast channel
6. Trigger evaluation: match against active triggers with event=Form Submitted (conditions from moka cache)
7. FormReactor processing:
   - If matching FormReactorEntry configured with action=Callback: initiate outbound call (flow A2)
   - If action=Notification: dispatch alert only
8. INSERT notification for assigned agent or user
9. Increment FormReactorEntry.call_count in in-process counter buffer
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | Cache read | form_reactor_entries | moka | Lookup by form_key; fallback SELECT via api_crud pool on miss |
| 3 | Cache read | tracking_numbers | moka | Resolve attribution from phone number field if present |
| 4 | INSERT | form_records | call_processing | form_data stored as JSONB; status=New; tracking_number_id nullable |
| 6 | Cache read + INSERT | triggers / trigger_actions | moka + background | Conditions evaluated in-process; actions inserted via background pool |
| 7 | Various | calls / call_legs | call_processing | If callback: executes flow A2; uses call_processing pool |
| 8 | INSERT | notifications | api_crud | Targets user or team configured on FormReactorEntry |
| 9 | In-process buffer | — | — | call_count flushed periodically to form_reactor_entries via background pool |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/forms/submit | Public form submission (rate-limited, no auth required) | `{form_key, fields: {}, [tracking_number]}` | 200 OK (always; prevents form enumeration) |
| GET | /api/v1/activities/forms | Forms list — new submission appears | `?page&limit&date_range&source` | Paginated form_records list |
| GET | /api/v1/activities/forms/{id} | Form submission detail | — | form_record with form_data fields |
| WS | /api/v1/notifications | Push notification for new submission | — | Notification event |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Activities > Forms | New row appears with contact name, phone, email (extracted from form_data), form name, and source attribution |
| Notification bell | Badge increments; notification body includes form name and submitter contact |
| PhoneDrawer | If FormReactor callback fires, an outbound call is initiated (see flow A2) |

**Testable Checkpoints:**

- [ ] form_records row created with all submitted fields intact in form_data JSONB column
- [ ] source attribution correct when a tracking_number is provided in the submission
- [ ] form.submitted event fires on tokio broadcast channel
- [ ] Triggers with form.submitted event type evaluate and execute configured actions
- [ ] FormReactor initiates callback call (flow A2) when action=Callback is configured
- [ ] Rate limiter returns 429 after threshold exceeded from a single IP within the window
- [ ] Notification row created for the correct user or team
- [ ] FormReactorEntry.call_count incremented and flushed within periodic window

---

### B5: Chat Session Lifecycle

**Trigger:** Visitor opens chat widget on customer website

**Data Flow:**

```
1. Chat widget JS establishes WebSocket connection to platform (widget_id in URL)
2. Lookup ChatWidget config (moka cache): pre-chat form fields, routing rules, AI agent assignment
3. Pre-chat form data collected from visitor (if widget requires it)
4. INSERT chat_records with status=Active, started_at=now(), visitor metadata
5. Route session:
   - If ChatWidget.ai_agent_id set: assign to ChatAIAgent (load config from moka cache)
   - If queue routing: broadcast assignment to agent WebSocket subscribers
6. Real-time message exchange over WebSocket (messages not individually persisted — held in-memory unless configured)
7. Session close event (visitor disconnect, agent close action, or AI completion):
   - UPDATE chat_records SET status=Closed, ended_at=now(),
       duration_secs=(ended_at - started_at),
       message_count=total,
       agent_id=assigned_agent_id
8. Emit "chat.completed" event to tokio broadcast channel
9. Post-completion cascade (same pattern as A4):
   - Trigger evaluation from moka cache
   - INSERT notifications via background pool
   - UPDATE call_daily_summary aggregation via background pool
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | Cache read | chat_widgets | moka | Widget config pre-loaded; fallback SELECT via api_crud pool |
| 4 | INSERT | chat_records | call_processing | status=Active; visitor_name, visitor_email from pre-chat form |
| 5 | Cache read | chat_ai_agents | moka | AI agent config and prompt loaded from cache |
| 7 | UPDATE | chat_records | call_processing | All session-end fields set in single UPDATE |
| 9 | INSERT | notifications | background | New chat alert or completion alert depending on trigger config |
| 9 | UPDATE | call_daily_summary | background | Increments chat_count, total_chat_duration for the tracking source |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| WS | /api/v1/chat/widget/{widget_id} | Visitor-side WebSocket connection | — | Bidirectional message stream |
| WS | /api/v1/chat/agent/{session_id} | Agent-side WebSocket for chat session | — | Bidirectional message stream |
| POST | /api/v1/chat/sessions | Create chat session (internal; called at WS open) | `{widget_id, visitor_info}` | 201 Created with session_id |
| PUT | /api/v1/chat/sessions/{id}/close | Close session (agent-initiated) | — | 200 OK with duration and message_count |
| GET | /api/v1/activities/chats | Chat sessions list | `?status&date_range&agent_id` | Paginated chat_records list |
| GET | /api/v1/activities/chats/{id} | Chat session detail | — | chat_record with transcript if persisted |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Activities > Chats | New Active row appears on session open; status updates to Closed with duration on session end |
| Agent chat interface | Receives new chat assignment notification; message thread renders in real time |
| Coaching page | Active chat sessions visible with visitor and agent identifiers |
| Notification bell | New chat assignment alert with visitor name and widget source |

**Testable Checkpoints:**

- [ ] chat_records row created with correct widget_id, status=Active, and started_at timestamp
- [ ] Visitor pre-chat form data stored in chat_records.visitor_name and visitor_email
- [ ] AI agent responds autonomously when chat_widgets.ai_agent_id is set
- [ ] Queue routing delivers session assignment to the correct agent WebSocket subscriber
- [ ] message_count and duration_secs are accurately calculated on session close
- [ ] Status transitions: Active → Closed (no other valid terminal states)
- [ ] WebSocket connection remains stable throughout session; reconnect restores session context
- [ ] chat.completed event fires on tokio broadcast channel after UPDATE committed
- [ ] Triggers with chat.completed event type evaluate and execute actions

---

### B6: Fax Send/Receive

**Trigger:** Inbound fax received on a tracking number, or outbound fax initiated by a user

**Data Flow (Inbound):**

```
1. Fax carrier webhook delivers receipt notification with document reference
2. Download fax document from carrier URL
3. Store document binary in S3-compatible object storage; record object_key
4. INSERT fax_records with direction=Inbound, status=Received, document_url=object_key,
       page_count from carrier metadata
5. Emit "fax.received" event to tokio broadcast channel
6. INSERT notification for user or team assigned to the receiving tracking number
```

**Data Flow (Outbound):**

```
1. User uploads document (PDF or TIFF) and provides recipient number via API
2. Store uploaded document in S3-compatible object storage; record object_key
3. INSERT fax_records with direction=Outbound, status=Sending, document_url=object_key
4. Submit fax job to carrier API with document reference
5. Carrier async status callback received
6. UPDATE fax_records SET status=Sent|Failed, completed_at=now()
```

**DB Operations:**

| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Inbound 4 | INSERT | fax_records | call_processing | direction=Inbound; document_url is object store key, not raw URL |
| Inbound 6 | INSERT | notifications | api_crud | Targets user or team from tracking number assignment config |
| Outbound 3 | INSERT | fax_records | api_crud | status=Sending; user-facing request, api_crud pool appropriate |
| Outbound 6 | UPDATE | fax_records | background | Async carrier callback; low-urgency background pool |

**API Endpoints:**

| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/webhooks/fax/inbound | Carrier inbound fax webhook (internal, token-auth) | Carrier fax payload | 200 OK |
| POST | /api/v1/fax/send | Initiate outbound fax | multipart/form-data: `{to_number, document}` | 201 Created with fax_record_id |
| GET | /api/v1/activities/faxes | Fax list | `?direction&status&date_range` | Paginated fax_records list |
| GET | /api/v1/activities/faxes/{id} | Fax detail | — | fax_record with metadata |
| GET | /api/v1/activities/faxes/{id}/document | Presigned download URL for fax document | — | `{url, expires_at}` |

**UI Touchpoints:**

| Page / Component | Behavior |
|-----------------|----------|
| Activities > Faxes | New row appears with direction badge, page count, status, and timestamp; outbound row updates from Sending to Sent/Failed on callback |
| Notification bell | Inbound fax alert with sender number and page count |

**Testable Checkpoints:**

- [ ] Inbound: fax_records row created with direction=Inbound, status=Received, and document_url pointing to object store key (not a carrier URL)
- [ ] Inbound: page_count matches carrier-reported value in webhook payload
- [ ] Outbound: status lifecycle Sending → Sent or Sending → Failed on carrier callback
- [ ] Outbound: fax_records row created before carrier submission (ensures audit trail even on carrier failure)
- [ ] Document binary stored in object storage at a deterministic, retrievable key
- [ ] Presigned URL endpoint returns a time-limited URL that allows document download without platform credentials
- [ ] Notification created for inbound fax and delivered over WebSocket to assigned user

---

## C. Administration & Configuration

These flows handle system configuration by admins. All writes go through the `api_crud` pool and trigger cache invalidation via PostgreSQL LISTEN/NOTIFY so that the call processing hot path picks up changes within milliseconds. The pattern is consistent: UI form → API → PostgreSQL INSERT/UPDATE → pg_notify → moka cache invalidated in all instances.

---

### C1: Routing Config CRUD (Queue, Router, Schedule, IVR)

**Trigger:** Admin creates, edits, or deletes a routing entity from the Flows section of the UI (VoiceMenu, Queue, SmartRouter, GeoRouter, Schedule, RoutingTable, or VoicemailBox).

**Data Flow:**
1. Admin fills out form and submits (e.g., create a new Queue with agents and strategy)
2. API validates input: name uniqueness within account, valid foreign keys, valid enum values (e.g., queue strategy, match operator)
3. Within a single transaction on `api_crud` pool:
   - INSERT/UPDATE the parent entity (e.g., `queues`)
   - INSERT/UPDATE/DELETE child entities in bulk (e.g., `queue_agents`, `voice_menu_options`, `smart_router_rules`, `schedule_holidays`, `geo_router_rules`, `routing_table_routes`)
4. On COMMIT, a PostgreSQL trigger fires `pg_notify('config_changed', '{"table":"queues","id":"<uuid>","op":"upsert","account_id":"<uuid>"}')`
5. All application instances receive the NOTIFY on their listener connection → invalidate or reload the affected moka cache entries keyed by entity ID and account
6. Next inbound call that evaluates routing reads the updated config from moka cache, with no database round-trip required

This exact pattern applies to all routing entities: VoiceMenu + VoiceMenuOption, Queue + QueueAgent, SmartRouter + SmartRouterRule, GeoRouter + GeoRouterRule, Schedule + ScheduleHoliday, RoutingTable + RoutingTableRoute, VoicemailBox.

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 3a | INSERT or UPDATE | `queues` / `voice_menus` / `smart_routers` / `geo_routers` / `schedules` / `routing_tables` / `voicemail_boxes` | `api_crud` | Parent entity; single transaction |
| 3b | INSERT / UPDATE / DELETE (batch) | `queue_agents` / `voice_menu_options` / `smart_router_rules` / `geo_router_rules` / `schedule_holidays` / `routing_table_routes` | `api_crud` | Children replaced in full on PUT; same transaction as step 3a |
| 4 | `pg_notify()` via trigger | — | — | Fires on COMMIT; payload includes table name, entity ID, op, account_id |
| 5 | Moka cache invalidation | — | — | Listener goroutine receives NOTIFY, removes or refreshes keyed entries within ~10ms |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | `/api/v1/flows/{entity-type}` | Create entity with nested children | Full entity JSON including children array | `201 Created` with entity ID and created_at |
| GET | `/api/v1/flows/{entity-type}` | List all entities for account | Query: `page`, `per_page`, `search` | Paginated array of entity summaries |
| GET | `/api/v1/flows/{entity-type}/{id}` | Get entity with all children | — | Full entity with nested children |
| PUT | `/api/v1/flows/{entity-type}/{id}` | Full replace of entity and children | Full entity JSON | `200 OK` with updated entity |
| DELETE | `/api/v1/flows/{entity-type}/{id}` | Soft delete or hard delete | Optional `?force=true` for hard delete | `204 No Content` |

`{entity-type}` is one of: `voice-menus`, `queues`, `smart-routers`, `geo-routers`, `schedules`, `routing-tables`, `voicemail-boxes`

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Flows > [entity type] list page | Refreshes after save; new or edited row appears immediately via optimistic update or refetch |
| Flows > [entity type] > create/edit form | Form fields map 1:1 to entity schema; child items (agents, options, rules) managed inline |
| TrackingNumber config > routing dropdown | Dropdown reloads available entities after any routing config change |
| Toast notification | Confirms save success or displays validation error from API |

**Testable Checkpoints:**
- [ ] Parent and all child entities are persisted in a single atomic transaction (partial failures roll back entirely)
- [ ] `pg_notify` fires with correct table name and entity UUID on every INSERT, UPDATE, and DELETE
- [ ] Moka cache reflects updated config within 100ms of transaction COMMIT
- [ ] A simulated inbound call immediately after save routes using the new config without a cold DB read
- [ ] Deleting a routing entity referenced by an active TrackingNumber is rejected (FK constraint or pre-delete validation returns 409)
- [ ] Validation rejects invalid enum values (e.g., unknown queue strategy) with 422 and field-level error detail
- [ ] Duplicate entity name within the same account is rejected with 422
- [ ] DELETE on a parent cascades correctly to all child tables within the same transaction

---

### C2: Number Provisioning

**Trigger:** Admin buys a new DID, initiates a port request, or reconfigures an existing tracking number.

**Data Flow (Buy Number):**
1. Admin searches available numbers by area code, city, or prefix via the Numbers > Buy Numbers page
2. API proxies search request to carrier API and returns available DIDs with rate center metadata
3. Admin selects a number, assigns a source (campaign) and routing target (queue, IVR, etc.), and confirms purchase
4. API calls carrier provisioning API to reserve and activate the DID
5. On carrier success: INSERT `tracking_numbers` with `phone_number`, `account_id`, `source_id`, `routing_target_type`, `routing_target_id`, `status=Active`
6. If `text_enabled=true`: INSERT `text_numbers` linked to the tracking number
7. `pg_notify('config_changed', '{"table":"tracking_numbers","id":"<uuid>","op":"insert","account_id":"<uuid>"}')`
8. Moka cache updated — number is now routable for inbound calls within ~10ms

**Data Flow (Port Number):**
1. Admin fills port request form: list of DIDs, billing name, service address, LOA signature data
2. INSERT `port_requests` with `status=Draft`, store LOA as document reference
3. Admin reviews and submits: UPDATE `port_requests` SET `status=Submitted`, `submitted_at=now()`
4. API submits port request to carrier porting API with LOA document
5. Carrier sends status callbacks through the lifecycle: Submitted → In Progress → Completed or Rejected
6. Background worker receives callback: UPDATE `port_requests` SET `status=<new>`, `updated_at=now()`, `rejection_reason=<text if rejected>`
7. On Completed: INSERT `tracking_numbers` for each ported DID (same as Buy step 5), then `pg_notify`

**Data Flow (Configure Existing Number):**
1. Admin edits tracking number config (change source, routing target, toggle text_enabled, rename)
2. UPDATE `tracking_numbers` with new values
3. `pg_notify('config_changed', '{"table":"tracking_numbers","id":"<uuid>","op":"update"}')`
4. Moka cache entry invalidated; next call to this number reads updated routing config

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Buy 5 | INSERT | `tracking_numbers` | `api_crud` | After carrier confirms activation |
| Buy 6 | INSERT | `text_numbers` | `api_crud` | Conditional on `text_enabled`; same transaction as Buy 5 |
| Port 2 | INSERT | `port_requests` | `api_crud` | `status=Draft` |
| Port 3 | UPDATE | `port_requests` | `api_crud` | `status=Submitted`, `submitted_at=now()` |
| Port 6 | UPDATE | `port_requests` | `background` | Carrier callback handler; sets status + rejection_reason |
| Port 7 | INSERT | `tracking_numbers` | `api_crud` | One row per ported DID on completion; triggers pg_notify |
| Config 2 | UPDATE | `tracking_numbers` | `api_crud` | Triggers pg_notify |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | `/api/v1/numbers/available` | Search available DIDs from carrier | Query: `area_code`, `city`, `type=local\|toll-free` | Array of available numbers with rate center |
| POST | `/api/v1/numbers/tracking` | Provision and create tracking number | `{phone_number, source_id, routing_target_type, routing_target_id, text_enabled}` | `201 Created` with full tracking number record |
| GET | `/api/v1/numbers/tracking` | List all tracking numbers for account | Query: `page`, `per_page`, `search`, `status` | Paginated tracking number list |
| GET | `/api/v1/numbers/tracking/{id}` | Get tracking number detail | — | Full record with source and routing info |
| PUT | `/api/v1/numbers/tracking/{id}` | Update config | `{source_id, routing_target_type, routing_target_id, text_enabled, ...}` | `200 OK` with updated record |
| DELETE | `/api/v1/numbers/tracking/{id}` | Decommission (release to carrier + soft delete) | — | `204 No Content` |
| POST | `/api/v1/numbers/port-requests` | Create port request | `{phone_numbers[], billing_name, service_address, loa_document_ref}` | `201 Created` with port request ID |
| PUT | `/api/v1/numbers/port-requests/{id}/submit` | Submit port request to carrier | — | `200 OK` with updated status |
| GET | `/api/v1/numbers/port-requests/{id}` | Get port request status | — | Port request with status, timestamps, rejection_reason |
| POST | `/api/v1/webhooks/carrier/port-status` | Carrier port status callback (internal) | Carrier-specific payload | `200 OK` |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Numbers > Buy Numbers | Search form with area code / city inputs; results grid with DID and rate center; one-click provision wizard |
| Numbers > Tracking Numbers list | New number appears in list immediately after provision; status badge shows Active |
| Numbers > Tracking Numbers > edit dialog | Source and routing dropdowns; text-enabled toggle; saves via PUT |
| Numbers > Port Numbers wizard | Multi-step form: number list → billing info → LOA sign → confirm submit |
| Numbers > Port Numbers > status view | Status badge updates through lifecycle; rejection reason shown on Rejected |

**Testable Checkpoints:**
- [ ] Tracking number created with correct `source_id`, `routing_target_type`, and `routing_target_id`
- [ ] Number is immediately routable after provision: moka cache populated within 100ms of INSERT COMMIT
- [ ] `text_numbers` row created if and only if `text_enabled=true`
- [ ] Port request status lifecycle progresses correctly: Draft → Submitted → In Progress → Completed or Rejected
- [ ] On Completed port: `tracking_numbers` rows created for every DID in the port request
- [ ] Rejected port request has non-null `rejection_reason`
- [ ] Decommissioned number returns 404 or voicemail on next inbound call attempt
- [ ] Carrier API failure during provision rolls back the `tracking_numbers` INSERT (no orphaned records)

---

### C3: Automation Config (Triggers, Workflows, Webhooks)

**Trigger:** Admin creates or edits an automation rule — a trigger, workflow, webhook, or lambda — in the Flows section of the UI.

**Data Flow (Trigger):**
1. Admin selects event type (e.g., `call.completed`), configures conditions (field, operator, value), and adds actions (e.g., send webhook, tag call, add to list)
2. API validates: event type is a known enum value, condition fields match event schema, action types are valid
3. Single transaction on `api_crud` pool: INSERT `triggers` (parent) + batch INSERT `trigger_conditions` + batch INSERT `trigger_actions`
4. `pg_notify('config_changed', '{"table":"triggers","id":"<uuid>","op":"insert","account_id":"<uuid>"}')`
5. Moka cache updated with new trigger definition within ~10ms
6. Next event of the matching type evaluates the new trigger inline

**Data Flow (Workflow):**
1. Admin builds workflow on visual canvas: drag nodes (steps) onto canvas, draw edges (connections) between them
2. On save: API receives full workflow payload including normalized node list and edge list plus raw `canvas_json` blob
3. Single transaction on `api_crud` pool: INSERT/UPDATE `workflows` + batch upsert `workflow_nodes` + batch upsert `workflow_edges`; `canvas_json` stored as JSONB column on `workflows`
4. `pg_notify('config_changed', '{"table":"workflows","id":"<uuid>","op":"upsert"}')`
5. Moka cache updated; workflow available for execution

**Data Flow (Webhook):**
1. Admin configures: callback URL, HTTP method, authentication header, and list of subscribed event types
2. API validates URL format and that all subscribed event types are known enum values
3. Single transaction: INSERT `webhooks` + batch INSERT `webhook_subscriptions` (one row per event type)
4. `pg_notify('config_changed', '{"table":"webhooks","id":"<uuid>","op":"insert"}')`
5. Moka cache updated; webhook fires on next matching event

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Trigger 3a | INSERT or UPDATE | `triggers` | `api_crud` | Parent; single transaction |
| Trigger 3b | INSERT (batch) | `trigger_conditions` | `api_crud` | Full replace on PUT: delete old + insert new in same tx |
| Trigger 3c | INSERT (batch) | `trigger_actions` | `api_crud` | Full replace on PUT; ordered by `sequence` |
| Workflow 3a | INSERT or UPDATE | `workflows` | `api_crud` | `canvas_json` as JSONB; single transaction |
| Workflow 3b | Upsert (batch) | `workflow_nodes` | `api_crud` | Stale nodes deleted; same transaction |
| Workflow 3c | Upsert (batch) | `workflow_edges` | `api_crud` | Stale edges deleted; same transaction |
| Webhook 3a | INSERT or UPDATE | `webhooks` | `api_crud` | Single transaction |
| Webhook 3b | INSERT (batch) | `webhook_subscriptions` | `api_crud` | Full replace on PUT; same transaction |
| All | `pg_notify()` via trigger | — | — | Fires on COMMIT; payload includes table, id, op, account_id |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | `/api/v1/flows/triggers` | Create trigger with conditions and actions | `{event_type, status, conditions[], actions[]}` | `201 Created` |
| PUT | `/api/v1/flows/triggers/{id}` | Full replace of trigger | Full trigger JSON | `200 OK` |
| GET | `/api/v1/flows/triggers` | List triggers for account | Query: `page`, `per_page`, `event_type` | Paginated list with run_count |
| POST | `/api/v1/flows/workflows` | Create workflow with nodes and edges | `{name, canvas_json, nodes[], edges[]}` | `201 Created` |
| PUT | `/api/v1/flows/workflows/{id}` | Update canvas, nodes, and edges | Full workflow JSON | `200 OK` |
| GET | `/api/v1/flows/workflows/{id}` | Get workflow with full node/edge graph | — | Workflow with nodes[], edges[], canvas_json |
| POST | `/api/v1/flows/webhooks` | Create webhook with event subscriptions | `{url, method, headers, events[]}` | `201 Created` |
| PUT | `/api/v1/flows/webhooks/{id}` | Update webhook config and subscriptions | Full webhook JSON | `200 OK` |
| GET | `/api/v1/flows/webhooks` | List webhooks | Query: `page`, `per_page` | Paginated list with last_triggered_at |
| POST | `/api/v1/flows/lambdas` | Create lambda function | `{name, runtime, code, env_vars, timeout_ms}` | `201 Created` |
| PUT | `/api/v1/flows/lambdas/{id}` | Update lambda code and config | `{code, env_vars, timeout_ms}` | `200 OK` |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Flows > Triggers list | Displays event type, condition count, action count, run count, status badge; toggle active/paused inline |
| Flows > Triggers > create/edit form | Event type selector, condition builder (field + operator + value rows), action builder with ordering |
| Flows > Workflows canvas | Drag-and-drop node palette; edge drawing; node config panel on click; Save button submits full graph |
| Flows > Webhooks list | URL, subscribed events, last triggered timestamp; test-fire button |
| Flows > Lambdas | Code editor with runtime selector, env vars table (values masked), timeout setting |

**Testable Checkpoints:**
- [ ] Trigger + all conditions + all actions are persisted atomically; any DB error rolls back the entire transaction
- [ ] `workflow.canvas_json` is consistent with the normalized `workflow_nodes` and `workflow_edges` rows after save
- [ ] Webhook `events` array contains only valid, known event type enum values (invalid values rejected with 422)
- [ ] A newly created trigger with `status=Active` fires on the next matching event
- [ ] A trigger with `status=Paused` does not fire even when event conditions match
- [ ] Lambda `env_vars` values are encrypted at rest; GET response returns key names only, not plaintext values
- [ ] Full replace on PUT deletes stale conditions, actions, nodes, and edges not present in the new payload

---

### C4: User/Agent Management

**Trigger:** Admin creates, edits, deactivates, or reassigns queue membership for a user or agent.

**Data Flow:**
1. Admin fills the user form: display name, email, role (`Admin`, `Supervisor`, `Agent`, `Viewer`), and queue assignments with priority
2. API validates: email uniqueness across the account, role is a valid enum value, referenced queue IDs exist and belong to the account
3. INSERT or UPDATE `users` with hashed password (on create), role, and `is_active=true`
4. If queue assignments changed: DELETE existing `queue_agents` rows for this user, then batch INSERT new `queue_agents` rows with configured priority and wrap-up time — all in the same transaction as step 3
5. `pg_notify('config_changed', '{"table":"users","id":"<uuid>","op":"upsert","account_id":"<uuid>"}')`
6. `pg_notify('config_changed', '{"table":"queue_agents","user_id":"<uuid>","op":"replaced","account_id":"<uuid>"}')`
7. Moka cache updated for both `users` and `queue_agents` entries — agent appears in or disappears from queue distribution within ~10ms

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 3 | INSERT or UPDATE | `users` | `api_crud` | Password stored as bcrypt hash; single transaction with step 4 |
| 4a | DELETE | `queue_agents` | `api_crud` | Remove all existing queue memberships for this user; same transaction |
| 4b | INSERT (batch) | `queue_agents` | `api_crud` | Insert new memberships with `priority`, `wrap_up_time_seconds`; same transaction |
| 5–6 | `pg_notify()` via trigger | — | — | Two separate NOTIFY payloads on COMMIT: one for `users`, one for `queue_agents` |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | `/api/v1/users` | Create user | `{name, email, password, role, queue_assignments[{queue_id, priority}]}` | `201 Created` with user record (no password hash) |
| GET | `/api/v1/users` | List users for account | Query: `page`, `per_page`, `role`, `is_active` | Paginated user list |
| GET | `/api/v1/users/{id}` | User detail with queue assignments | — | User record with `queue_assignments[]` |
| PUT | `/api/v1/users/{id}` | Update user profile and queue assignments | Full user JSON (omit password to leave unchanged) | `200 OK` |
| PUT | `/api/v1/users/{id}/deactivate` | Soft deactivate user | — | `200 OK`; sets `is_active=false` |
| GET | `/api/v1/users/{id}/state-log` | Agent state history | Query: `from`, `to` (ISO 8601) | Array of state transitions with timestamps |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Admin > Users list | Table of users with role badge, status (active/inactive), queue count; create and edit actions |
| Admin > Users > create/edit form | Name, email, role picker, queue multi-select with priority input per queue |
| Flows > Queues > agent picker | Dropdown of active users refreshes when user roster changes |
| Coaching page | Deactivated agents no longer appear in agent selector |
| Reports > Agent Performance | Filters to active agents by default; deactivated agents shown only with explicit filter |

**Testable Checkpoints:**
- [ ] Created user record has correct `role`, `account_id`, and `is_active=true`
- [ ] Email uniqueness enforced within the account; duplicate email returns 422
- [ ] Password stored as bcrypt hash; plaintext password never written to any table or log
- [ ] Queue assignments updated atomically with user record — no partial state possible
- [ ] `pg_notify` fires for both `users` and `queue_agents` tables on every save
- [ ] Deactivated user (`is_active=false`) is rejected at authentication with 401
- [ ] Deactivated user is excluded from queue distribution in moka cache within 100ms
- [ ] `state-log` endpoint returns complete agent state history within the requested time range

---

### C5: Contact List Management

**Trigger:** Admin creates a contact list, imports contacts from CSV, or manually adds or removes individual members.

**Data Flow (Create List):**
1. Admin provides list name and optional description on the Contacts > Contact Lists page
2. API validates name uniqueness within the account
3. INSERT `contact_lists` with `member_count=0`, `account_id`, `name`, `description`, `created_at`

**Data Flow (Import Members):**
1. Admin uploads a CSV file from the list detail page; file must have a `phone_number` column; optional columns: `name`, `email`, `custom_fields`
2. API parses the CSV and validates each row: phone number must be E.164 format (`+1XXXXXXXXXX`); invalid rows collected for error report
3. Batch INSERT `contact_list_members` for all valid rows using `ON CONFLICT (contact_list_id, phone_number) DO NOTHING` to deduplicate within the list
4. UPDATE `contact_lists SET member_count = (SELECT COUNT(*) FROM contact_list_members WHERE contact_list_id = $1)` to sync accurate count

**Data Flow (Manual Add):**
1. Admin types a phone number and optional name into the inline add form on the list detail page
2. API validates E.164 format
3. INSERT `contact_list_members` with `ON CONFLICT (contact_list_id, phone_number) DO NOTHING`; returns 200 if duplicate (not an error)
4. UPDATE `contact_lists SET member_count = member_count + 1` only if the INSERT actually wrote a row (check `rows_affected`)

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| Create 3 | INSERT | `contact_lists` | `api_crud` | `member_count=0` initial |
| Import 3 | INSERT (batch) | `contact_list_members` | `api_crud` | `ON CONFLICT (contact_list_id, phone_number) DO NOTHING`; batch size ≤ 1000 rows per statement |
| Import 4 | UPDATE | `contact_lists` | `api_crud` | Full recount after batch insert; same transaction as Import 3 |
| Manual 3 | INSERT | `contact_list_members` | `api_crud` | `ON CONFLICT DO NOTHING` |
| Manual 4 | UPDATE | `contact_lists` | `api_crud` | Conditional increment only if `rows_affected = 1` |
| Remove | DELETE | `contact_list_members` | `api_crud` | Single row by `member_id` |
| Remove | UPDATE | `contact_lists` | `api_crud` | Decrement `member_count` by 1; same transaction |
| Delete list | DELETE | `contact_lists` | `api_crud` | Cascades to `contact_list_members` via FK ON DELETE CASCADE |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | `/api/v1/contacts/lists` | Create contact list | `{name, description}` | `201 Created` with list record |
| GET | `/api/v1/contacts/lists` | List all contact lists | Query: `page`, `per_page`, `search` | Paginated list with `member_count` |
| GET | `/api/v1/contacts/lists/{id}` | List detail | — | List record with `member_count`, `created_at` |
| GET | `/api/v1/contacts/lists/{id}/members` | Paginated member list | Query: `page`, `per_page`, `search` | Members with `phone_number`, `name`, `added_at` |
| POST | `/api/v1/contacts/lists/{id}/members` | Add single member | `{phone_number, name}` | `200 OK` (idempotent) |
| POST | `/api/v1/contacts/lists/{id}/import` | Bulk import CSV | `multipart/form-data` with CSV file | `202 Accepted` with `{imported_count, skipped_count, error_rows[]}` |
| DELETE | `/api/v1/contacts/lists/{id}/members/{member_id}` | Remove member | — | `204 No Content` |
| DELETE | `/api/v1/contacts/lists/{id}` | Delete list and all members | — | `204 No Content` |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Contacts > Contact Lists | Table of lists with name, member count, created date; create button opens inline form |
| Contact list detail | Member table with search; Add Member inline form; Import CSV button with drag-and-drop zone |
| Import result banner | Shows imported count, skipped duplicates, and downloadable error report for invalid rows |
| Bulk Messages > campaign setup | Contact list picker shows all lists with member counts |
| Reminders config | Contact list picker for outbound reminder campaigns |

**Testable Checkpoints:**
- [ ] CSV import deduplicates within the list: importing the same phone number twice results in one member row
- [ ] `member_count` on `contact_lists` remains accurate after add, remove, and bulk import operations
- [ ] CSV import validates E.164 format: invalid rows are rejected and included in the error report, not silently skipped
- [ ] Deleting a contact list cascades to remove all `contact_list_members` rows
- [ ] An empty list can be created with `member_count=0`
- [ ] Large CSV import (10,000+ rows) completes within the `api_crud` pool statement timeout (30s)
- [ ] Manual add of a duplicate phone number returns 200 (not 409) and does not increment `member_count`
- [ ] Remove member decrements `member_count` correctly and is transactional with the DELETE

---

### C6: AI Configuration

**Trigger:** Admin configures AI tools — knowledge banks with document indexing, summary settings, or voice/chat AI agent definitions.

**Data Flow (Knowledge Bank):**
1. Admin creates a knowledge bank with name, description, and category on the AI Tools > Knowledge Banks page
2. INSERT `knowledge_banks` with `status=Empty`, `document_count=0`, `account_id`
3. Admin uploads one or more documents (supported formats: PDF, DOCX, TXT, HTML) via drag-and-drop or file picker
4. API stores the raw document file in object storage and records the `file_ref` (storage path/key)
5. INSERT `knowledge_bank_documents` with `embedding_status=Pending`, `file_ref`, `file_name`, `file_size`, `knowledge_bank_id`
6. API emits a `document.uploaded` internal event; a background worker picks up the task asynchronously:
   a. Worker reads document from object storage, extracts and chunks text content
   b. Sends chunks to AI embedding model API; receives embedding vectors
   c. Batch INSERT `knowledge_bank_embeddings` (pgvector `vector` column) — one row per chunk, linked to `knowledge_bank_document_id`
   d. UPDATE `knowledge_bank_documents` SET `embedding_status=Indexed`, `chunk_count=N`, `indexed_at=now()`
   e. UPDATE `knowledge_banks` SET `document_count=(SELECT COUNT(*) WHERE status=Indexed)`, `status=Ready`
7. If embedding API call fails: UPDATE `knowledge_bank_documents` SET `embedding_status=Failed`, `error_message=<reason>`

**Data Flow (Summary Config):**
1. Admin toggles summary types (call summary, keyword extraction, sentiment, action items) and transcription on/off
2. UPDATE `summary_configs` (one singleton row per account) with new boolean flags and model preferences
3. `pg_notify('config_changed', '{"table":"summary_configs","account_id":"<uuid>","op":"update"}')`
4. Moka cache entry for this account's summary config invalidated within ~10ms
5. Next call completion event reads updated config and generates only the enabled summary types

**Data Flow (Voice/Chat AI Agent):**
1. Admin configures agent: name, system instructions, voice model, linked knowledge bank, escalation/handoff rules
2. INSERT or UPDATE `voice_ai_agents` or `chat_ai_agents` with all config fields
3. `pg_notify('config_changed', '{"table":"voice_ai_agents","id":"<uuid>","op":"upsert"}')`
4. Moka cache updated; next interaction routed to this agent uses the new config

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| KB 2 | INSERT | `knowledge_banks` | `api_crud` | `status=Empty` initial |
| KB 5 | INSERT | `knowledge_bank_documents` | `api_crud` | `embedding_status=Pending`; after object storage write succeeds |
| KB 6c | INSERT (batch) | `knowledge_bank_embeddings` | `background` | pgvector `vector` column; one row per text chunk |
| KB 6d | UPDATE | `knowledge_bank_documents` | `background` | `embedding_status=Indexed`, `chunk_count`, `indexed_at` |
| KB 6e | UPDATE | `knowledge_banks` | `background` | `document_count`, `status=Ready` |
| KB 7 | UPDATE | `knowledge_bank_documents` | `background` | `embedding_status=Failed`, `error_message` |
| Summary 2 | UPDATE | `summary_configs` | `api_crud` | Singleton row per account; upsert on conflict |
| Agent 2 | INSERT or UPDATE | `voice_ai_agents` / `chat_ai_agents` | `api_crud` | `pg_notify` on commit |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | `/api/v1/ai/knowledge-banks` | Create knowledge bank | `{name, description, category}` | `201 Created` with bank ID |
| GET | `/api/v1/ai/knowledge-banks` | List knowledge banks | Query: `page`, `per_page` | Paginated list with `document_count`, `status` |
| GET | `/api/v1/ai/knowledge-banks/{id}` | Get bank detail | — | Bank record with `document_count`, `status` |
| POST | `/api/v1/ai/knowledge-banks/{id}/documents` | Upload document for indexing | `multipart/form-data` with file | `202 Accepted` with document ID and `embedding_status=Pending` |
| GET | `/api/v1/ai/knowledge-banks/{id}/documents` | List documents with indexing status | Query: `page`, `per_page` | Documents with `embedding_status`, `chunk_count`, `indexed_at` |
| DELETE | `/api/v1/ai/knowledge-banks/{id}/documents/{doc_id}` | Remove document and embeddings | — | `204 No Content` |
| GET | `/api/v1/ai/summary-config` | Get account summary settings | — | Summary config flags and model preferences |
| PUT | `/api/v1/ai/summary-config` | Update summary settings | `{transcription_enabled, summary_enabled, sentiment_enabled, keywords_enabled, action_items_enabled}` | `200 OK` |
| POST | `/api/v1/ai/voice-agents` | Create voice AI agent | `{name, instructions, voice_model, knowledge_bank_id, handoff_config}` | `201 Created` |
| PUT | `/api/v1/ai/voice-agents/{id}` | Update voice AI agent | Full agent config JSON | `200 OK` |
| GET | `/api/v1/ai/voice-agents` | List voice AI agents | Query: `page`, `per_page` | Paginated agent list |
| POST | `/api/v1/ai/chat-agents` | Create chat AI agent | `{name, instructions, knowledge_bank_id, widget_config}` | `201 Created` |
| PUT | `/api/v1/ai/chat-agents/{id}` | Update chat AI agent | Full agent config JSON | `200 OK` |
| POST | `/api/v1/ai/ask` | Ad-hoc AI question against a call | `{call_id, prompt}` | `200 OK` with `{response, sources[]}` |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| AI Tools > Knowledge Banks list | Bank cards with name, document count, status badge (Empty / Indexing / Ready) |
| AI Tools > Knowledge Banks > detail | Document table with file name, size, `embedding_status` badge, chunk count; upload zone |
| Document upload progress | Polling or SSE updates `embedding_status` from Pending → Processing → Indexed in real time |
| AI Tools > Summaries | Toggle switches per summary type; transcription enable/disable; model selector |
| AI Tools > Voice AI | Agent config form: instructions textarea, voice picker, knowledge bank picker, handoff rules |
| AI Tools > Chat AI | Agent config form: instructions, knowledge bank picker, widget appearance settings |
| CallDetailPanel > Voice Analysis tab | AI-generated summaries, sentiment score, keywords, and action items appear after call processing |

**Testable Checkpoints:**
- [ ] Knowledge bank document file is stored in object storage at the expected path before `knowledge_bank_documents` INSERT
- [ ] Embedding pipeline transitions document status: `Pending` → `Processing` → `Indexed` (or `Failed`)
- [ ] An `Indexed` document has `chunk_count > 0` and a corresponding number of `knowledge_bank_embeddings` rows
- [ ] A `Failed` document has a non-null `error_message` explaining the failure reason
- [ ] Deleting a document removes both the `knowledge_bank_documents` row and all associated `knowledge_bank_embeddings` rows
- [ ] Summary config change is reflected in moka cache within 100ms; the following call uses the updated config
- [ ] Voice AI agent config is accessible from routing evaluation (moka cache hit, no DB query on hot path)
- [ ] Ad-hoc `/api/v1/ai/ask` returns a response and cites source documents from the correct knowledge bank
- [ ] `knowledge_banks.status` correctly reflects `Ready` only when at least one document is `Indexed`

---

## D. Analytics, Reporting & Annotations

Overview: These flows serve the 30 report pages and the agent annotation workflow. Read-heavy flows use the `reports` connection pool with 60-second statement timeout. Annotation writes use `api_crud`. The aggregation refresh job uses `background`. The key performance optimization is that standard reports query the pre-aggregated `call_daily_summary` fact table, while custom reports with non-dimension filters fall back to partitioned `call_records`.

---

### D1: Call Annotation (Scoring, Tagging, Notes)

**Trigger:** Agent scores, tags, or adds notes to a call from the CallDetailPanel.

**Data Flow:**
1. Agent opens CallDetailPanel for a specific call
2. API fetches `call_records` JOIN `call_annotations` JOIN `call_tags` for display
3. Agent modifies: sets score (1–5), toggles converted, selects outcome, types notes, adds/removes tags
4. On save:
   - a. UPDATE `call_annotations` SET score, converted, outcome, notes, updated_at, updated_by_id
   - b. For added tags: INSERT `call_tags` with applied_by_type=Manual, applied_by_id=agent
   - c. For removed tags: DELETE `call_tags` WHERE call_id=? AND tag_id=?
   - d. Increment/decrement `tags.usage_count` via in-process counter buffer (flushed periodically — no per-save DB write)
5. If appointment_set toggled to true:
   - a. UPDATE `call_annotations` SET appointment_set=true
   - b. INSERT `appointments` with call_id link, agent_id, source inherited from the call record

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | SELECT (JOIN) | call_records, call_annotations, call_tags | reports | Single read for full display state |
| 4a | UPDATE | call_annotations | api_crud | call_records is never modified (immutable) |
| 4b | INSERT | call_tags | api_crud | applied_by_type = 'Manual' |
| 4c | DELETE | call_tags | api_crud | Keyed on (call_id, tag_id) |
| 4d | In-process buffer | — | — | No DB write per save; flushed on interval |
| 5a | UPDATE | call_annotations | api_crud | Same row as 4a; can be combined |
| 5b | INSERT | appointments | api_crud | source copied from call_records at write time |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | /api/v1/activities/calls/{id} | Full call detail (CDR + annotations + tags) | — | call_record + call_annotations + tag list |
| PUT | /api/v1/activities/calls/{id}/annotations | Update score, outcome, notes, converted, appointment_set | {score, converted, outcome, notes, appointment_set} | Updated call_annotations row |
| POST | /api/v1/activities/calls/{id}/tags | Add a tag to a call | {tag_id} | 201 Created, updated tag list |
| DELETE | /api/v1/activities/calls/{id}/tags/{tag_id} | Remove a tag from a call | — | 204 No Content |
| POST | /api/v1/appointments | Create appointment linked to call | {call_id, agent_id, scheduled_at, notes} | Created appointment row |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| CallDetailPanel > Score tab | Score slider (1–5), converted toggle, outcome dropdown, reporting tag selector, notes textarea; all fields pre-populated from call_annotations |
| CallDetailPanel > header tag area | Tag pills with X to remove; tag search/add input |
| Activities > Calls list | Score column and tag pills update immediately after save (optimistic update or refetch) |
| Reports > Appointments page | New appointment row appears after appointment_set toggle and POST /appointments |

**Testable Checkpoints:**
- [ ] `call_annotations` row updated with new score, outcome, notes, converted values; `call_records` row is unchanged (immutability verified by comparing before/after row hash)
- [ ] `call_tags` rows added correctly with applied_by_type='Manual' and correct applied_by_id
- [ ] `call_tags` rows removed correctly; no orphan rows for the deleted (call_id, tag_id) pair
- [ ] `tags.usage_count` reflects net changes after the in-process counter buffer is flushed
- [ ] `appointments` row created with correct call_id FK, agent_id, and source value inherited from the originating call
- [ ] Updated score and tags are visible in the Activities > Calls list on next page load
- [ ] Two agents annotating different calls concurrently produce no lock contention or data conflict
- [ ] Annotating a call with no prior `call_annotations` row creates one (upsert / insert-on-first-use semantics)

---

### D2: Standard Report Generation

**Trigger:** User navigates to any of the 30 report pages.

**Data Flow:**
1. UI loads the report page with a default date range (e.g., last 30 days) and default dimension filters cleared
2. API queries `call_daily_summary` for KPI card values and time-series chart data, filtered by account_id, date range, and any active dimension filters (source_id, agent_id, queue_id)
3. For reports that require a dimensional breakdown table (e.g., Calls by Source), the same summary table is queried with GROUP BY on the relevant dimension, JOINed to dimension label tables (tracking_sources, users, queues) for human-readable names
4. For reports that require per-call detail rows (e.g., Missed Calls list, call detail drilldown), the query falls back to partitioned `call_records` JOINed with `call_annotations`; the date-range predicate prunes to the relevant monthly partitions
5. UI renders KPI cards (top), interactive chart (middle), and detail/breakdown table (bottom)

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | SELECT SUM/AVG/COUNT | call_daily_summary | reports | 60s statement timeout; date range drives partition or index range scan |
| 3 | SELECT GROUP BY + JOIN | call_daily_summary, tracking_sources, users, queues | reports | Dimension label joins are small lookups |
| 4 | SELECT + JOIN | call_records, call_annotations | reports | Partition-pruned by started_at range; only for per-call detail reports |

**Report-to-Query Mapping:**

| Report | Primary Table | Notes |
|--------|--------------|-------|
| Activity Report | call_daily_summary | KPIs + time series |
| Calls by Source | call_daily_summary | GROUP BY source_id |
| Daily Calls | call_daily_summary | One row per day |
| Agent Performance | call_daily_summary | GROUP BY agent_id |
| Queue Report | call_daily_summary | GROUP BY queue_id |
| Scoring Report | call_daily_summary | AVG score from summary |
| Activity Map | call_daily_summary | + geographic dimension join |
| All 19 industry template reports | call_daily_summary | Industry-specific KPI label mapping in app layer |
| Missed Calls (detail list) | call_records | Partition-pruned; status filter |
| Real Time (historical portion) | call_daily_summary + call_records | Summary for today's totals; records for recent call list |
| Appointments Report | appointments | Joined to call_records for source/agent context |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | /api/v1/reports/activity | Activity KPIs + time series | start, end, source_id, agent_id, queue_id | {kpis: {}, series: []} |
| GET | /api/v1/reports/calls-by-source | Source breakdown table | start, end, page, per_page | {rows: [], total} |
| GET | /api/v1/reports/agent-performance | Per-agent metric table | start, end, agent_id, page, per_page | {rows: [], total} |
| GET | /api/v1/reports/queue | Per-queue metric table | start, end, queue_id, page, per_page | {rows: [], total} |
| GET | /api/v1/reports/daily-calls | Day-by-day call volume | start, end, source_id | {rows: []} |
| GET | /api/v1/reports/missed-calls | Per-call missed call list | start, end, page, per_page | {rows: [], total} |
| GET | /api/v1/reports/appointments | Appointment volume + list | start, end, agent_id, page, per_page | {kpis: {}, rows: [], total} |
| GET | /api/v1/reports/scoring | Agent scoring summary | start, end, agent_id | {kpis: {}, rows: []} |
| GET | /api/v1/reports/activity-map | Geographic heatmap data | start, end, source_id | {regions: []} |
| GET | /api/v1/reports/template/{template-name} | Industry template report | start, end, + dimension filters | {kpis: {}, series: [], rows: [], total} |

All report endpoints accept: `start`, `end` (ISO date, required), `source_id`, `agent_id`, `queue_id` (optional dimension filters), `page`, `per_page` (for paginated detail tables).

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Reports > [any report page] — KPI cards | Populated from summary aggregates; show current period value and delta vs. prior period |
| Reports > [any report page] — chart | Interactive time series or bar chart; hover for tooltip; click bar/point to drill to call list |
| Reports > [any report page] — detail table | Paginated; sortable columns; links to CallDetailPanel for per-call rows |
| FilterBar (shared component) | Date range picker, source/agent/queue dropdowns; on change, refetches all three sections |
| Export button | Triggers flow D4 with the current filters as the export spec |

**Testable Checkpoints:**
- [ ] KPI card values exactly match manual SUM/AVG applied to `call_daily_summary` rows for the selected date range and account
- [ ] Time-series chart data points correspond 1:1 to daily summary rows (one point per calendar day in range)
- [ ] Date range filter excludes records outside the range (query boundary is inclusive on start, inclusive on end)
- [ ] Dimension filters (source_id, agent_id, queue_id) correctly narrow both KPI aggregates and table rows
- [ ] Reports backed by `call_daily_summary` return within 2 seconds for up to 365 days of data
- [ ] Reports backed by `call_records` use partition pruning (query plan shows only relevant monthly partitions scanned)
- [ ] Paginated detail tables return correct `total` count and correct page slice
- [ ] Reports with zero matching rows render an empty-state UI element, not an error or spinner
- [ ] All 19 industry template reports resolve to the same underlying `call_daily_summary` query pattern with label remapping in the app layer

---

### D3: Custom Report Execution

**Trigger:** User creates and runs a saved custom report from Reports > Custom Reports.

**Data Flow:**
1. User configures a new report: selects base type, chooses columns from available fields, builds filter predicates, sets date range, chooses sort column and direction
2. On save, INSERT `custom_reports` with the full configuration serialized as a JSON column (columns list, filter predicates, date_range defaults, sort spec, optional cron schedule and recipient list)
3. User triggers a report run:
   - a. Fast path — if all active filter predicates align with `call_daily_summary` dimensions (date, source, agent, queue): query `call_daily_summary` with the selected columns projected from available summary measures
   - b. Slow path — if any filter predicate references a non-dimension field (tag presence, disposition value, transcript keyword match, geographic subdivision beyond region): query partitioned `call_records` with appropriate JOINs to `call_annotations`, `call_tags`, and other tables; partition pruning applied via date range
4. Results rendered in the UI as a configurable table with the user's selected columns and sort order
5. If the report has a cron schedule set, the background job scheduler runs the report on schedule, streams results to a file (same as D4 export), and emails the file to the configured `schedule_recipients` list

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | INSERT | custom_reports | api_crud | JSON config column; idempotent on re-save via PUT |
| 3a | SELECT (fast path) | call_daily_summary | reports | 60s timeout; column projection from summary measures |
| 3b | SELECT (slow path) | call_records, call_annotations, call_tags | reports | 60s timeout; partition-pruned; JOINs added per active filter |
| 5 | SELECT (same as 3a/3b) | (as above) | background | 300s timeout for scheduled runs; results streamed to file |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/reports/custom | Create saved custom report | {name, base_type, columns, filters, date_range, sort, schedule?, recipients?} | Created custom_reports row |
| GET | /api/v1/reports/custom | List all saved custom reports for account | page, per_page | {rows: [{id, name, last_run_at, row_count}], total} |
| GET | /api/v1/reports/custom/{id} | Fetch report config | — | Full custom_reports row with JSON config |
| PUT | /api/v1/reports/custom/{id} | Update report config | Partial or full config body | Updated row |
| POST | /api/v1/reports/custom/{id}/run | Execute report, return data | {date_range?, filters?} overrides | {rows: [], total, query_path: 'fast'|'slow', elapsed_ms} |
| DELETE | /api/v1/reports/custom/{id} | Delete saved report | — | 204 No Content |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Reports > Custom Reports — report list | Table of saved reports: name, last run time, row count, schedule badge; Run and Edit actions per row |
| Custom report builder — column selector | Drag-and-drop or checkbox list of available columns, grouped by dimension vs. measure |
| Custom report builder — filter builder | Add/remove filter rows: field selector, operator (=, !=, contains, in), value input; non-dimension fields show slow-path warning badge |
| Custom report builder — schedule panel | Toggle to enable scheduling; cron picker (daily/weekly/monthly shortcuts + custom); email recipient list input |
| Report results view | Configurable table with selected columns; sort by clicking column headers; export button triggers D4 with current filters |

**Testable Checkpoints:**
- [ ] Fast path engaged when all filter fields are date, source_id, agent_id, or queue_id; `query_path: 'fast'` returned in response
- [ ] Slow path engaged when any filter field is tag, disposition, transcript keyword, or geographic subdivision; `query_path: 'slow'` returned
- [ ] Slow path uses partition pruning — query plan shows only the monthly `call_records` partitions covered by the date range
- [ ] Saved report config round-trips correctly: POST config, GET same id, compare JSON field-for-field
- [ ] Scheduled report executes within 60 seconds of the cron trigger time
- [ ] Scheduled report email is sent to all `schedule_recipients` with the correct file attachment
- [ ] A report with a tag filter correctly JOINs `call_tags` and returns only calls bearing that tag
- [ ] A slow-path report spanning a date range that would exceed 60 seconds returns an error response with a descriptive timeout message, not a hung connection
- [ ] Deleting a saved report removes it from the list and cancels any scheduled runs

---

### D4: Data Export

**Trigger:** User clicks "Export" on any report page or activity list page.

**Data Flow:**
1. User selects export format (CSV, PDF, or Excel) in the export dialog and confirms; the active filters and date range from the source page are captured as a snapshot at this moment
2. API inserts `export_records` row with status=Processing, requested_by_id, source_type, format, and filters_applied (JSON snapshot of the exact filters in effect at request time)
3. The insert emits an internal event (or enqueues a background job message) to the export worker
4. Export worker executes:
   - a. Runs the same query as the source report or list page, using the filters_applied snapshot, but without pagination (full result set)
   - b. Streams rows into the selected file format; for PDF, applies account branding/logo
   - c. Uploads the completed file to object storage; receives a download URL (presigned, time-limited)
   - d. UPDATE `export_records` SET status=Complete, download_url, record_count, file_size_bytes, completed_at=now()
5. On status=Complete, inserts a notification row and pushes a WebSocket event to the requesting user's session
6. User clicks the notification or navigates to Export Log and downloads via the presigned URL redirect

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | INSERT | export_records | api_crud | status='Processing'; filters_applied is a JSON snapshot |
| 4a | SELECT (full result) | call_daily_summary or call_records + joins | background | 300s timeout; no LIMIT applied |
| 4c | Upload | — | — | Object storage, no DB |
| 4d | UPDATE | export_records | background | Same connection as 4a worker |
| 5 | INSERT | notifications | api_crud | Linked to export_records.id; type='export_ready' |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/exports | Request a new export | {source_type, filters, format: 'csv'|'pdf'|'xlsx'} | {id, status: 'Processing'} |
| GET | /api/v1/exports/{id} | Poll export status and retrieve download URL | — | {id, status, download_url?, record_count?, error_message?} |
| GET | /api/v1/exports | List export history for account | page, per_page | {rows: [{id, status, format, record_count, created_at, completed_at}], total} |
| GET | /api/v1/exports/{id}/download | Redirect to presigned object storage URL | — | 302 redirect to presigned URL |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Export dialog (any page) | Format selector (CSV / PDF / Excel); displays active filter summary; Confirm triggers POST /exports |
| Activities > Export Log page | Full export history table with status badges (Processing / Complete / Failed), format, row count, file size, created/completed timestamps, Download button |
| Notification bell | "Your export is ready" alert with inline Download link appears when status transitions to Complete |
| Browser download | GET /exports/{id}/download triggers file download via 302 redirect to presigned URL |

**Testable Checkpoints:**
- [ ] `export_records` row created with status='Processing' immediately on POST; response returns the new id
- [ ] `filters_applied` JSON in `export_records` matches the exact filters present at request time (not modified by later UI interaction)
- [ ] Background worker generates a valid, parseable file for each supported format (CSV, PDF, XLSX)
- [ ] File is present in object storage at the expected path after worker completes
- [ ] `export_records` updated to status='Complete' with non-null download_url, record_count, and completed_at
- [ ] GET /exports/{id}/download returns a 302 redirect to a URL that delivers the correct file
- [ ] Exporting a result set with zero rows produces a valid file containing only column headers (not an empty or corrupt file)
- [ ] An export whose background query exceeds 300 seconds transitions to status='Failed' with a descriptive error_message; no partial file is uploaded
- [ ] A notification row is created and delivered via WebSocket to the requesting user's session on completion
- [ ] Concurrent exports from different users do not interfere with each other's `export_records` rows

---

### D5: Aggregation Refresh

**Trigger:** Scheduled background job (default cron: top of every hour) or on-demand via admin API.

**Data Flow:**
1. Job starts and determines which date/dimension combinations are stale:
   - Today's date is always marked stale (new calls may have arrived since last refresh)
   - For prior dates: compare the `computed_at` timestamp on each `call_daily_summary` row against the MAX(ended_at) of `call_records` for that date; if any call ended after the last computation, the date is stale
2. For each stale date, the job iterates over all relevant dimension combinations (source_id, agent_id, queue_id) for the account:
   - a. Aggregate `call_records` for that date and dimension combination: total_calls, answered_calls, missed_calls, total_duration_secs, average_duration_secs, etc.
   - b. JOIN `call_annotations` to compute converted_calls count, appointments_set count, average_score
   - c. UPSERT into `call_daily_summary` ON CONFLICT (account_id, summary_date, source_id, agent_id, queue_id) DO UPDATE SET all measure columns and computed_at=now()
3. After dimension-level rows, compute the account-level totals by aggregating across all dimension values for the date (using NULL as the dimension key) and upsert those rows as well
4. Job records the refresh completion time and stale-date count in an internal job_log table (or structured log output)

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 1 (staleness check) | SELECT MAX(ended_at) | call_records | background | Partition-pruned; one query per candidate date |
| 1 (staleness check) | SELECT computed_at | call_daily_summary | background | Keyed on (account_id, summary_date) |
| 2a | SELECT COUNT/SUM/AVG | call_records | background | Partition-pruned to date's monthly partition |
| 2b | SELECT COUNT (JOIN) | call_annotations | background | Filtered by call_id IN (subquery for date) |
| 2c | INSERT ON CONFLICT DO UPDATE | call_daily_summary | background | Upsert is idempotent; safe for concurrent runs |
| 3 | INSERT ON CONFLICT DO UPDATE | call_daily_summary | background | NULL dimension keys for account-level totals |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/admin/aggregation/refresh | Trigger an immediate manual refresh | {date_range?: {start, end}, account_id?: UUID} | {job_id, status: 'Queued'} |
| GET | /api/v1/admin/aggregation/status | Inspect last refresh state | — | {last_completed_at, stale_date_count, next_scheduled_at, current_job_status} |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| No direct end-user UI | Report pages automatically reflect fresh data after the next refresh cycle completes |
| Admin dashboard (if present) | Aggregation health card: last refresh time, stale date count, next scheduled time, manual trigger button |

**Testable Checkpoints:**
- [ ] After a refresh, `call_daily_summary` row counts for today match a direct COUNT on `call_records` for the same account and date
- [ ] All dimension combinations present in today's `call_records` (source_id, agent_id, queue_id cross-product) have a corresponding `call_daily_summary` row
- [ ] NULL-dimension (account-level total) rows are present and their measure values equal the SUM of all dimension-level rows for the same date
- [ ] `computed_at` on each upserted row reflects the time of the refresh run, not the time of the underlying call
- [ ] Staleness detection correctly skips dates whose `computed_at` is already newer than the latest `call_records.ended_at` for that date (no unnecessary work)
- [ ] Two refresh jobs triggered concurrently produce identical results and no data corruption (upsert idempotency)
- [ ] The full refresh for an account with 90 days of stale data completes within the 300-second background pool statement timeout
- [ ] KPI card values on a standard report page match the `call_daily_summary` values immediately after a refresh (no caching layer serving stale summary data)

---

### D6: Real-Time Dashboard

**Trigger:** User opens Reports > Real Time page or the Coaching page.

**Data Flow:**
1. UI sends an HTTP upgrade request to establish a WebSocket connection on /api/v1/dashboard/realtime; server registers the connection under the user's account_id
2. Server performs an initial state read from the moka in-process cache (active_calls map and presence map for the account); this is a sub-microsecond in-memory read with no DB involvement
3. Server sends the full current state to the client as the initial WebSocket message: active call list, agent presence statuses, and queue depth counts
4. UI renders the active call table, agent status grid, and queue depth gauges from the initial state
5. As calls and agent states change, each state-changing operation (call connect, call end, agent status update) writes to the UNLOGGED active_calls or presence table AND issues a PG NOTIFY on a per-account channel
6. The server's LISTEN subscriber receives the NOTIFY, updates the moka cache entry, and fans out a WebSocket push message to all connected clients subscribed to that account channel
7. UI applies the incremental update to its local state without re-fetching or re-rendering the full dashboard
8. For historical comparison panels (e.g., "calls today vs. yesterday at this hour"), the UI issues a separate REST request to the reports endpoint, which queries `call_daily_summary`; this is a one-time load, not part of the WebSocket stream

**Derived Metrics (computed in-app from cache, never stored):**

| Metric | Derivation |
|--------|-----------|
| Calls waiting per queue | COUNT active_calls WHERE queue_id=Q AND status=Ringing |
| Average wait time | AVG(now() - started_at) for status=Ringing calls |
| Service level % | Answered-within-threshold calls / total calls for today (from call_daily_summary) |
| Agents available | COUNT presence WHERE status=Available AND queue membership includes Q |
| Agents on call | COUNT presence WHERE status=OnCall |

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | READ | moka cache (active_calls, presence) | — | Zero DB; in-process memory only |
| 6 | READ | moka cache (post-NOTIFY update) | — | Zero DB; NOTIFY triggers cache update |
| 8 | SELECT | call_daily_summary | reports | One-time REST fetch; not on WebSocket path |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| WS | /api/v1/dashboard/realtime | Persistent live stream of active calls and presence | — (upgrade) | Initial full-state message, then incremental delta messages |
| GET | /api/v1/dashboard/realtime/snapshot | Current state snapshot (non-WebSocket fallback) | — | {active_calls: [], presence: [], queue_depths: {}} |
| GET | /api/v1/reports/activity | Today's aggregates for historical comparison panels | start=today, end=today | {kpis: {}, series: []} |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Reports > Real Time — active call table | Live rows: caller ID, called number, duration (ticking), agent, queue, status; rows appear on call connect, disappear on call end |
| Reports > Real Time — queue depth gauges | Calls waiting and average wait time per queue; update in real time as calls enter/leave Ringing state |
| Reports > Real Time — agent status grid | Card per agent showing status badge (Available, On Call, Wrap-Up, Away); updates within 100ms of agent state change |
| Coaching page — active call list | Same as real-time table plus Listen, Whisper, Barge action buttons per row; agent status badges alongside each call |
| Queue Report — live metrics header | Calls waiting, average wait, service level % panels at page top; populated from derived metrics, not a DB query |

**Testable Checkpoints:**
- [ ] Initial WebSocket message delivers a complete snapshot of current active_calls and presence for the account
- [ ] A new call connecting causes a WebSocket push to all connected dashboard viewers within 100ms
- [ ] A call ending removes its row from the active call table on all connected viewers without a page reload
- [ ] Queue depth counts computed from active_calls cache match a direct COUNT on the UNLOGGED active_calls table at the same instant
- [ ] Agent available/on-call counts match the presence table at the same instant
- [ ] Zero DB queries are issued during steady-state real-time display for an account with no state changes (verified by query log)
- [ ] Historical comparison data (today vs. yesterday) loads from `call_daily_summary`, not from `call_records`
- [ ] A client that disconnects and reconnects receives a full state refresh on reconnection, not a delta from the disconnection point
- [ ] Multiple concurrent dashboard viewers (e.g., 10 supervisors) receive consistent and identical state updates
- [ ] GET /dashboard/realtime/snapshot returns equivalent data to the initial WebSocket message for clients that cannot use WebSocket

---

## E. Compliance & Integrations

Overview: These flows handle regulatory compliance, third-party webhook integrations, AI processing pipelines, and DNC/DNT enforcement. Compliance flows are low-volume but high-importance — data integrity and audit trails are critical. AI processing flows are asynchronous and may take seconds to minutes. Integration flows (webhooks) must handle external service failures gracefully with retry logic.

### E1: Compliance Registration Lifecycle

**Trigger:** Admin submits a compliance registration (A2P 10DLC, toll-free messaging, or STIR/SHAKEN voice)

**Data Flow:**
1. Admin configures BusinessInfo first (prerequisite — must exist before any registration can be created)
2. Admin fills A2P campaign form: campaign name, use case, description, sample messages
3. INSERT a2p_campaigns with status=Draft
4. Admin reviews and submits — UPDATE a2p_campaigns SET status=Pending, submitted_at=now()
5. Server-side validates: BusinessInfo present, required fields non-empty, then calls carrier registration API
6. Carrier processes asynchronously (1–7 business days); system waits for callback
7. Carrier webhook callback arrives with status update:
   - If Approved: UPDATE a2p_campaigns SET status=Approved, approved_at=now(), dlc_campaign_id=carrier_ref
   - If Rejected: UPDATE a2p_campaigns SET status=Rejected, rejection_reason=carrier_message
8. For VoiceRegistration specifically: INSERT voice_registration_history row (append-only audit trail) on every status change
9. INSERT notifications row targeting account admins with registration status summary

The same lifecycle pattern applies to:
- TollFreeRegistration: Not Registered → Pending → Approved / Rejected
- VoiceRegistration: Not Registered → Pending → Approved / Rejected (with per-transition history rows)
- ComplianceApplication: Draft → Submitted → Under Review → Approved / Rejected / Expired

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 1 | SELECT | business_info | api_crud | Verify prerequisite exists before allowing form submission |
| 1 | INSERT | business_info | api_crud | Created if not yet present via PUT business-info endpoint |
| 3 | INSERT | a2p_campaigns | api_crud | Initial row with status=Draft |
| 4 | UPDATE | a2p_campaigns | api_crud | Sets status=Pending, submitted_at=now() |
| 7 | UPDATE | a2p_campaigns | background | Carrier callback handler updates status and carrier ref |
| 7 | UPDATE | toll_free_registrations | background | Same pattern for TF registrations |
| 7 | UPDATE | voice_registrations | background | Same pattern for voice registrations |
| 8 | INSERT | voice_registration_history | background | Append-only; one row per status transition |
| 9 | INSERT | notifications | api_crud | Status change notification for admin users |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | /api/v1/trust/business-info | Retrieve account business identity | — | BusinessInfo object (EIN masked) |
| PUT | /api/v1/trust/business-info | Create or update business identity | BusinessInfo fields | Updated BusinessInfo |
| POST | /api/v1/trust/a2p-campaigns | Create A2P campaign in Draft | Campaign fields | Campaign with id, status=Draft |
| PUT | /api/v1/trust/a2p-campaigns/{id}/submit | Submit campaign to carrier | — | Campaign with status=Pending, submitted_at |
| GET | /api/v1/trust/a2p-campaigns | List campaigns with status | ?status=, ?page= | Paginated campaign list |
| GET | /api/v1/trust/a2p-campaigns/{id} | Campaign detail | — | Full campaign object |
| POST | /api/v1/trust/toll-free-registrations | Create toll-free registration | TF fields | Registration with id, status=Not Registered |
| PUT | /api/v1/trust/toll-free-registrations/{id}/submit | Submit TF registration | — | Registration with status=Pending |
| POST | /api/v1/trust/voice-registrations | Create STIR/SHAKEN registration | Voice reg fields | Registration with id |
| PUT | /api/v1/trust/voice-registrations/{id}/submit | Submit voice registration | — | Registration with status=Pending |
| GET | /api/v1/trust/voice-registrations/{id}/history | Audit trail for voice registration | — | Ordered list of history rows |
| POST | /api/v1/webhooks/carrier/registration-status | Carrier status callback (internal) | Carrier payload | 200 OK |
| GET | /api/v1/trust/compliance | Compliance requirements checklist | — | List of requirements with status |
| PUT | /api/v1/trust/compliance/{id} | Update requirement status manually | {status} | Updated requirement |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Trust Center > Business Info | Multi-field identity form; must be saved before any registration tab allows submission |
| Trust Center > Local Text Messaging | A2P campaign list with colored status badges (Draft, Pending, Approved, Rejected); inline rejection reason on hover |
| Trust Center > Toll-Free Text Messaging | TF registration card with status badge and submit button; disabled until BusinessInfo complete |
| Trust Center > Voice Registration | STIR/SHAKEN form with status badge; history section shows chronological timeline of all status transitions |
| Trust Center > Compliance | Requirements checklist; items auto-populated by account country and active number types; status icons per item |
| Notification bell | Alert pushed for every status transition (Pending, Approved, Rejected) on all registration types |

**Testable Checkpoints:**
- [ ] POST to any registration endpoint without BusinessInfo present returns 422 with clear prerequisite error
- [ ] Status transitions are forward-only: Draft → Pending → Approved/Rejected (API rejects out-of-order transitions)
- [ ] Submitted registrations have submitted_at timestamp set and non-null
- [ ] Carrier callback matches registration by dlc_campaign_id or external carrier reference; mismatched references return 404
- [ ] voice_registration_history row inserted on every status change, not just final states
- [ ] GET /api/v1/trust/business-info returns EIN as masked indicator (e.g., "***-**-1234"), never plaintext
- [ ] Rejected registration returns rejection_reason field in GET response
- [ ] Notification row created in notifications table on each status transition
- [ ] Compliance requirements auto-populated on account creation based on country code and number types in use
- [ ] Resubmitting an already-Approved registration returns 409 Conflict

---

### E2: Webhook Delivery

**Trigger:** System event matches a webhook subscription (called from post-event cascades: A4, B1, B4, B5)

**Data Flow:**
1. Internal event emitted with event_type string (e.g., "call.completed", "sms.received", "form.submitted")
2. Load all webhooks for the account from moka cache; filter to those with matching event_type subscription and status=Active
3. For each matching webhook:
   a. Serialize event data to JSON payload, including event_type, account_id, timestamp, and entity-specific fields
   b. If webhook.secret is configured, compute HMAC-SHA256 signature over raw payload bytes using the secret; attach as X-Signature header
   c. INSERT webhook_deliveries with status=Pending, payload=serialized JSON, attempt_number=1, webhook_id
   d. Fire HTTP POST to webhook.callback_url with configured method, headers, timeout (default 10s)
   e. On 2xx response: UPDATE webhook_deliveries SET status=Success, http_status_code=code, responded_at=now()
   f. On non-2xx or timeout:
      - UPDATE webhook_deliveries SET status=Failed, http_status_code=code (null if timeout), response_body=first 4KB of response
      - If attempt_number < webhook.retry_count: compute next_attempt_at using exponential backoff (base 30s, factor 2^attempt)
      - INSERT new webhook_deliveries row for the retry with attempt_number incremented, same payload
4. UPDATE webhooks SET last_triggered_at=now() regardless of delivery outcome

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | READ | moka cache (webhooks) | — | Zero DB; cache populated from webhooks table; invalidated via PG LISTEN/NOTIFY on webhook changes |
| 3c | INSERT | webhook_deliveries | background | Initial pending row before HTTP attempt |
| 3e | UPDATE | webhook_deliveries | background | status=Success, http_status_code, responded_at |
| 3f | UPDATE | webhook_deliveries | background | status=Failed, http_status_code, response_body (truncated to 4KB) |
| 3f retry | INSERT | webhook_deliveries | background | New row per retry attempt; attempt_number incremented |
| 4 | UPDATE | webhooks | background | last_triggered_at=now() on every delivery attempt |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | /api/v1/flows/webhooks/{id}/deliveries | Delivery log for a webhook | ?status=, ?page= | Paginated delivery rows with status, attempt_number, responded_at |
| GET | /api/v1/flows/webhooks/{id}/deliveries/{delivery_id} | Delivery detail | — | Full payload JSON, response_body, http_status_code, timing |
| POST | /api/v1/flows/webhooks/{id}/test | Fire test delivery with sample payload | {event_type} | Delivery result with http_status_code and response_body |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Flows > Webhooks list | last_triggered_at column refreshes after each delivery; status badge reflects webhook health (Active, Paused, Error) |
| Webhook detail > Deliveries tab | Sortable log of delivery attempts: status chip, attempt_number, http_status_code, timestamp; paginated |
| Delivery detail modal | Full serialized payload in code block; response_body in code block; timing breakdown; retry history chain |
| Test button | Fires test delivery inline; shows real HTTP response within UI without leaving the page |

**Testable Checkpoints:**
- [ ] Webhook with matching event_type subscription fires; webhook with non-matching subscription does not
- [ ] Payload contains correct event_type, account_id, and entity-specific fields for the triggering event
- [ ] HMAC-SHA256 signature in X-Signature header matches recomputed signature using shared secret
- [ ] 2xx response sets status=Success with correct http_status_code
- [ ] Non-2xx response sets status=Failed with http_status_code and first 4KB of response_body stored
- [ ] Timeout (no response within 10s) sets status=Failed with null http_status_code
- [ ] Retry INSERT occurs only when attempt_number < webhook.retry_count; no retry beyond max
- [ ] Retry timing uses exponential backoff: attempt 2 at +30s, attempt 3 at +60s, attempt 4 at +120s, etc.
- [ ] Webhooks with status=Paused do not fire for any event
- [ ] POST /test endpoint creates a real webhook_deliveries row and returns live HTTP response data
- [ ] last_triggered_at updated on delivery even when response is non-2xx

---

### E3: AI Processing Pipeline

**Trigger:** Call completes with transcription or AI analysis enabled (called from post-call cascade A4, step 6)

**Data Flow:**
1. Background worker dequeues transcription job containing call_id and recording file reference
2. Read recording file from object storage using file_ref path
3. Submit audio bytes to configured ASR (Automatic Speech Recognition) service; receive ordered segments
4. Parse ASR response into speaker-labeled segments with start_ms and end_ms offsets
5. INSERT call_transcription_segments in batch — one row per speaker segment with speaker label, start_ms, end_ms, text
6. If a KeywordSpottingConfig is active for the tracking number that received this call:
   a. Load keyword_spotting_keywords for the config from moka cache
   b. Scan each transcription segment text against the keyword list (case-insensitive, whole-word match)
   c. INSERT call_keyword_hits for each match: keyword_id, segment reference, matched_text, speaker, timestamp_ms
7. If SummaryConfig has one or more enabled summary_types:
   a. Load summary_configs for the tracking number from moka cache
   b. For each enabled summary_type (Classic Summary, Sentiment Analysis, Action Items, Custom, etc.):
      - Assemble prompt: type-specific system prompt + full transcription text
      - If a KnowledgeBank is referenced by the associated AI agent, execute pgvector similarity search using transcription as query; prepend top-K retrieved chunks to the prompt context
      - Submit assembled prompt to AI model API
      - INSERT call_ai_summaries with summary_type, generated content, model_used, prompt_tokens, completion_tokens
8. If AskAIConfig is active for the tracking number:
   a. Load ask_ai_configs from moka cache
   b. Execute configured analysis: custom question, lead score computation, sentiment score, conversion signal, etc.
   c. Store result as call_ai_summaries row or trigger output_action (e.g., update contact tag, fire webhook)
9. On completion, UPDATE call_records SET ai_processed=true, transcription_status=Completed

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | READ | object storage | — | No DB; direct object store fetch by file_ref |
| 5 | INSERT (batch) | call_transcription_segments | background | Bulk insert all segments in single statement |
| 6a | READ | moka cache (keyword_spotting_keywords) | — | Cache key: keyword_spotting_config_id |
| 6c | INSERT | call_keyword_hits | background | One row per keyword match per segment |
| 7a | READ | moka cache (summary_configs) | — | Cache key: tracking_number_id |
| 7b | SELECT | knowledge_bank_embeddings (pgvector) | background | Cosine similarity search; top-K chunks for RAG context |
| 7b | INSERT | call_ai_summaries | background | One row per summary_type; parallel where model supports it |
| 8a | READ | moka cache (ask_ai_configs) | — | Cache key: tracking_number_id |
| 8c | INSERT | call_ai_summaries | background | Ask AI results stored same table, distinct summary_type |
| 9 | UPDATE | call_records | background | Mark ai_processed=true, transcription_status=Completed |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| GET | /api/v1/activities/calls/{id}/transcription | Ordered transcription segments | — | Array of segments with speaker, start_ms, end_ms, text |
| GET | /api/v1/activities/calls/{id}/ai-summaries | AI summaries grouped by type | — | Array of summaries with summary_type and content |
| GET | /api/v1/activities/calls/{id}/keyword-hits | Keyword spotting results | — | Array of hits with keyword, timestamp_ms, speaker, matched_text |
| POST | /api/v1/ai/ask | Ad-hoc AI question against a call | {call_id, prompt, knowledge_bank_id?} | {answer, model_used, tokens_used} |
| GET | /api/v1/activities/calls/{id}/voice-analysis | Combined transcription + summaries + keyword hits | — | Merged object for CallDetailPanel rendering |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| CallDetailPanel > Transcript area | Speaker-labeled segments in chronological order; speaker chips color-coded (Agent vs. Caller); timestamps shown on hover |
| CallDetailPanel > Voice Analysis tab | AI summaries rendered by type: Classic Summary as prose, Sentiment as gauge, Action Items as checklist, Custom as freeform text |
| CallDetailPanel > Voice Analysis tab (keywords) | Matched keywords highlighted inline within transcript; keyword hit count badge per configured keyword |
| Reports > Keyword Spotting report | Aggregate keyword detection counts across filtered calls; top keywords ranked by frequency |

**Testable Checkpoints:**
- [ ] Transcription segments are ordered by start_ms ascending with no gaps or overlaps
- [ ] Speaker labels (Agent / Caller) correctly assigned and consistent across all segments for the same speaker
- [ ] Keyword hits reference correct segment, timestamp_ms, and speaker label
- [ ] Each enabled summary_type in SummaryConfig produces exactly one call_ai_summaries row per call
- [ ] call_ai_summaries content field is non-empty for successfully processed summaries
- [ ] AI processing job enqueued after call_records commit completes (not before); call completion is not blocked
- [ ] pgvector similarity search returns chunks with cosine distance below threshold before including in prompt
- [ ] ASR failure (bad audio, service timeout) sets transcription_status=Failed on call_records and logs error; remaining AI steps are skipped
- [ ] AskAI output_action (e.g., webhook trigger) fires after result is stored, not before
- [ ] Multiple summary types are processed with parallelism where the AI model API allows concurrent requests

---

### E4: DNC/DNT Enforcement

**Trigger:** Outbound call or SMS attempted — checked synchronously before any carrier interaction (flows A2, B2, B3)

**Data Flow:**
1. Outbound action initiated: agent clicks dial, system triggers auto-dial, or bulk message send begins
2. Normalize target phone number to E.164 format; reject immediately if normalization fails
3. Perform compliance list lookup against the account's lists:
   a. For outbound calls: SELECT dnc_entries WHERE e164=normalized AND account_id=account
   b. For outbound SMS: SELECT dnt_entries WHERE e164=normalized AND account_id=account
   c. For bulk SMS campaigns: check each recipient's e164 in batch before enqueuing any messages
4. If a match is found:
   - Abort the outbound action before any carrier API call or SIP INVITE is issued
   - Return a structured error response to the caller or system with reason code (DNC_BLOCKED or DNT_BLOCKED)
   - For DNT: increment dnt_entries.rejected_count via in-process counter buffer (flushed to DB every 60s)
   - Optionally INSERT api_log_entries row for blocked attempt (configurable per account)
5. If no match found: allow the outbound action to proceed to the next flow step

Note: For large accounts where compliance lists exceed moka cache capacity, lookups hit the api_crud pool directly. For smaller accounts with lists under the cache size threshold, dnc_entries and dnt_entries are pre-loaded into moka cache per account and invalidated via PG LISTEN/NOTIFY when entries change.

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 3a | SELECT | dnc_entries | api_crud (or moka cache) | Point lookup by e164 + account_id; index required |
| 3b | SELECT | dnt_entries | api_crud (or moka cache) | Point lookup by e164 + account_id; index required |
| 3c bulk | SELECT (batch) | dnc_entries / dnt_entries | api_crud | Batch lookup for bulk campaigns; IN clause with recipient list |
| 4 counter | WRITE | in-process counter buffer | — | Flushed to dnt_entries.rejected_count periodically (every 60s) |
| 4 counter flush | UPDATE | dnt_entries | background | Batch update rejected_count for all incremented entries |
| 4 log | INSERT | api_log_entries | background | Optional; blocked attempt audit record |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/contacts/dnc | Add number to DNC list | {e164, reason?} | Created dnc_entry |
| DELETE | /api/v1/contacts/dnc/{id} | Remove entry from DNC list | — | 204 No Content |
| GET | /api/v1/contacts/dnc | List DNC entries | ?page=, ?search= | Paginated dnc_entries |
| POST | /api/v1/contacts/dnc/import | Bulk import DNC list from CSV | multipart CSV | {imported_count, rejected_count, errors[]} |
| POST | /api/v1/contacts/dnt | Add number to DNT list | {e164, reason?} | Created dnt_entry |
| DELETE | /api/v1/contacts/dnt/{id} | Remove entry from DNT list | — | 204 No Content |
| GET | /api/v1/contacts/dnt | List DNT entries | ?page=, ?search= | Paginated dnt_entries with rejected_count |
| POST | /api/v1/contacts/dnt/import | Bulk import DNT list from CSV | multipart CSV | {imported_count, rejected_count, errors[]} |
| GET | /api/v1/contacts/dnc/check | Check if number is on DNC | ?number=+1... | {blocked: bool, reason?: string} |
| GET | /api/v1/contacts/dnt/check | Check if number is on DNT | ?number=+1... | {blocked: bool, rejected_count: int} |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| Contacts > Do Not Call | Table of DNC entries with e164, reason, added_at; Add Number button; Remove action per row; CSV import button with validation preview |
| Contacts > Do Not Text | Table of DNT entries with e164, reason, added_at, rejected_count column; same add/remove/import controls |
| PhoneDrawer (dial pad) | DNC check fires on number entry; if blocked, dial button disabled and inline error message shown: "This number is on the Do Not Call list" |
| Bulk Messages > Recipient step | DNC/DNT numbers flagged during recipient validation before campaign launch; blocked numbers shown in exclusion preview with count |

**Testable Checkpoints:**
- [ ] POST SIP INVITE to a DNC number is never issued — the block occurs before any carrier interaction
- [ ] SMS carrier API is never called for a DNT number — block occurs before HTTP request to carrier
- [ ] DNC/DNT check uses E.164 normalized number; non-E.164 input rejected before lookup
- [ ] GET /api/v1/contacts/dnc/check returns {blocked: true, reason} for a known DNC number
- [ ] Blocked attempt returns HTTP 422 with structured error body including reason code
- [ ] dnt_entries.rejected_count increments by 1 after the 60s counter flush for each blocked SMS attempt
- [ ] CSV import correctly parses E.164 numbers; rejects duplicates within the same account
- [ ] CSV import rejects rows with non-E.164 phone numbers and returns per-row errors
- [ ] DELETE dnc/{id} followed by GET dnc/check returns {blocked: false} for that number
- [ ] Bulk campaign send excludes DNT/DNC numbers without blocking the entire campaign
- [ ] SMS STOP keyword processing (flow B1) inserts dnt_entry automatically, reflected in subsequent DNT checks

---

### E5: Knowledge Bank Indexing Pipeline

**Trigger:** Admin uploads a document to a knowledge bank (initiated from AI Tools UI, referenced in flow C6)

**Data Flow:**
1. Admin submits multipart form upload with document file (PDF, DOCX, HTML, TXT, or plain text)
2. Server validates file type and size limits; store original file bytes to object storage at path: knowledge-banks/{bank_id}/docs/{uuid}.{ext}
3. INSERT knowledge_bank_documents with embedding_status=Pending, file_ref=object_storage_path, original_filename, file_size_bytes, mime_type
4. Return 202 Accepted to client with document id and embedding_status=Pending
5. Background worker picks up indexing job by document id:
   a. UPDATE knowledge_bank_documents SET embedding_status=Processing, processing_started_at=now()
   b. Read file bytes from object storage using file_ref
   c. Extract plain text content using format-appropriate parser (PDF: pdfium/pdfextract, DOCX: docx parser, HTML: HTML stripper, TXT: direct read)
   d. Compute SHA-256 content_hash over extracted text bytes
   e. SELECT knowledge_bank_documents WHERE content_hash=hash AND knowledge_bank_id=bank_id; if an Indexed document with same hash exists, skip embedding step and mark new document as Indexed with existing chunk_count
   f. Split extracted text into overlapping chunks: 512-token target size, 50-token overlap; assign sequential chunk_index values
   g. For each chunk: call embedding model API (e.g., OpenAI text-embedding-3-small, 1536 dimensions); on API response, INSERT knowledge_bank_embeddings with chunk_index, chunk_text, embedding vector (pgvector)
   h. UPDATE knowledge_bank_documents SET embedding_status=Indexed, chunk_count=N, indexed_at=now(), content_hash=hash
   i. UPDATE knowledge_banks SET document_count=active_doc_count, total_size_bytes=sum_of_file_sizes, status=Ready, last_updated_at=now()
6. On failure at any step after step 5a:
   - UPDATE knowledge_bank_documents SET embedding_status=Failed, error_message=description_of_failure
   - If all documents in bank are Failed: UPDATE knowledge_banks SET status=Error

**DB Operations:**
| Step | Operation | Table | Pool | Notes |
|------|-----------|-------|------|-------|
| 2 | WRITE | object storage | — | No DB; returns file_ref path |
| 3 | INSERT | knowledge_bank_documents | api_crud | Returns document id immediately; embedding_status=Pending |
| 5a | UPDATE | knowledge_bank_documents | background | embedding_status=Processing, processing_started_at |
| 5d | SELECT | knowledge_bank_documents | background | content_hash dedup check; avoids redundant embedding API calls |
| 5g | INSERT | knowledge_bank_embeddings (pgvector) | background | One row per chunk; vector dimensions must match model output |
| 5h | UPDATE | knowledge_bank_documents | background | embedding_status=Indexed, chunk_count, indexed_at, content_hash |
| 5i | UPDATE | knowledge_banks | background | Aggregate counts and status refresh after all chunks committed |
| 6 | UPDATE | knowledge_bank_documents | background | embedding_status=Failed, error_message |
| 6 | UPDATE | knowledge_banks | background | status=Error if all documents in bank have failed |

**API Endpoints:**
| Method | Path | Purpose | Request | Response |
|--------|------|---------|---------|----------|
| POST | /api/v1/ai/knowledge-banks/{id}/documents | Upload document to bank | multipart: file, name? | 202 {document_id, embedding_status=Pending} |
| GET | /api/v1/ai/knowledge-banks/{id}/documents | List documents with current status | ?status=, ?page= | Paginated list with embedding_status, chunk_count, indexed_at |
| GET | /api/v1/ai/knowledge-banks/{id}/documents/{doc_id} | Document detail | — | Full document metadata including chunk_count, error_message |
| DELETE | /api/v1/ai/knowledge-banks/{id}/documents/{doc_id} | Remove document and its embeddings | — | 204 No Content |
| POST | /api/v1/ai/knowledge-banks/{id}/reindex | Re-index all documents in bank | — | 202 {queued_count} |
| POST | /api/v1/ai/knowledge-banks/{id}/search | Test vector similarity search | {query: string, top_k: int} | Array of {chunk_text, similarity_score, document_name} |

**UI Touchpoints:**
| Page / Component | Behavior |
|-----------------|----------|
| AI Tools > Knowledge Banks list | Per-bank card shows document_count, total_size_bytes, overall status badge (Ready, Processing, Error, Empty) |
| Knowledge bank detail > Documents tab | Table of documents with per-row embedding_status badge: Pending (gray), Processing (spinner), Indexed (green), Failed (red with error tooltip) |
| Document upload area | Drag-and-drop zone with file type and size validation; upload progress bar during multipart POST; status badge updates via polling GET on document id |
| Search test panel | Query input and top_k slider; results list shows matched chunk_text and similarity score; used for validating bank content before deploying to AI agents |

**Testable Checkpoints:**
- [ ] Original file stored in object storage at path knowledge-banks/{bank_id}/docs/{uuid}.{ext} before DB insert
- [ ] POST /documents returns 202 immediately with embedding_status=Pending (not 201 after indexing)
- [ ] knowledge_bank_documents transitions: Pending → Processing → Indexed (or Failed) — no skipped states
- [ ] chunk_count > 0 for every successfully Indexed document
- [ ] knowledge_bank_embeddings row count matches chunk_count on the parent document row
- [ ] Each embedding vector has the correct number of dimensions for the configured embedding model (e.g., 1536 for text-embedding-3-small)
- [ ] content_hash deduplication: uploading identical file content skips embedding API calls and reuses chunk_count
- [ ] POST /search returns chunks with cosine similarity scores and correct document attribution
- [ ] DELETE /documents/{doc_id} cascades to delete all knowledge_bank_embeddings rows for that document
- [ ] POST /reindex clears all existing knowledge_bank_embeddings for the bank before re-inserting; chunk counts reflect fresh run
- [ ] Embedding model API failure sets embedding_status=Failed with non-empty error_message; document is not stuck in Processing
- [ ] A document larger than the background pool statement timeout (300s) completes because chunk processing commits incrementally rather than in one transaction

---

## Appendix: Flow Dependencies

The following diagram shows which flows trigger or depend on other flows. Arrows denote a causal relationship: the left side emits an event or calls the right side as part of its execution.

```
A1 (Inbound Call) → A5 (Active Call State) → A3 (Call Completion) → A4 (Post-Call Cascade)
                                                                      ├→ E2 (Webhook Delivery)
                                                                      ├→ E3 (AI Processing)
                                                                      └→ D5 (Aggregation update)

A2 (Outbound Call) → E4 (DNC Check) → A5 → A3 → A4

B1 (Inbound SMS) → E2 (Webhook) + E4 (DNT Check for STOP)
B2 (Outbound SMS) → E4 (DNT Check)
B3 (Bulk Message) → E4 (DNT Check per recipient) + B2 (per message)
B4 (Form Submission) → E2 (Webhook) + A2 (if FormReactor callback)

C1-C6 (All Config) → PG LISTEN/NOTIFY → moka cache invalidation → affects A1, A2, A5, A6

D1 (Annotation) → standalone (no downstream flows)
D2-D3 (Reports) → standalone (read-only)
D4 (Export) → D2 or D3 (reuses report query)
D5 (Aggregation) → feeds D2, D6
D6 (Real-Time) → reads from A5, A6 (active calls + presence)

E1 (Compliance) → standalone (carrier lifecycle)
E2 (Webhook) → triggered by A4, B1, B4, B5
E3 (AI Pipeline) → triggered by A4
E4 (DNC/DNT) → called by A2, B2, B3
E5 (Knowledge Indexing) → triggered by C6, consumed by E3
```
