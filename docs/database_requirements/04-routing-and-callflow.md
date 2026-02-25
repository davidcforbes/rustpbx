# 04 — Routing & Call Flow

> **Status:** Pre-analysis / Data element reconciliation
> **Date:** 2026-02-25
> **Shard:** 04 of 09 — Routing & Call Flow

---

## Overview

The Routing & Call Flow domain governs everything that happens to a call between the moment it enters the platform and the moment it reaches a live agent, voicemail, or external destination. When a tracked number receives an inbound call, the platform evaluates a chain of routing objects — Schedules determine whether the business is open, GeoRouters and SmartRouters apply conditional logic based on caller attributes, IVR menus (VoiceMenus) collect DTMF input, Queues distribute calls across available agents, and RoutingTables provide a fallback priority ladder. At any step in this chain a call may be handed to a VoicemailBox if no agent is reachable.

Every routing object in this domain uses the same **polymorphic routing pattern**: a `destination_type` / `destination_id` pair that can point to any other entity in the routing chain — or to an external phone number. This makes it possible to construct arbitrarily deep call flows without requiring a hard-coded foreign key for each possible destination type. The pattern is described in detail in the final section of this document.

Child entities (VoiceMenuOption, QueueAgent, SmartRouterRule, GeoRouterRule, ScheduleHoliday, RoutingTableRoute) follow a parent-scoped lifecycle — they are always created, listed, and deleted through their parent, and they cascade-delete when the parent is removed.

AgentScript is included in this domain because it is consumed at call-answer time and is authored alongside queue and routing configuration in the Flows section of the UI.

---

## Entities

### VoiceMenu

**UI References:** Flows > Voice Menus page

**Relationships:**
- Many-to-one with Account (each VoiceMenu belongs to one Account)
- One-to-many with VoiceMenuOption (a menu has 0–12 DTMF options)
- Referenced as a destination by VoiceMenuOption, SmartRouterRule, GeoRouterRule, RoutingTableRoute, and Schedule (via destination_type / destination_id)

**Notes:** When `greeting_type` is `TTS`, `greeting_tts_text` must be present and `greeting_audio_url` is ignored. When `greeting_type` is `Audio`, `greeting_audio_url` must be present. When `greeting_enabled` is false, the call proceeds immediately to DTMF collection. If `speech_recognition` is true, the platform also accepts spoken words in addition to DTMF tones; `speech_language` controls the ASR locale. If no input is received after `timeout_secs` and `max_retries` attempts are exhausted, the call is forwarded to `no_input_destination_type` / `no_input_destination_id`.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account (tenant isolation) |
| name | short_text | NN, MAX(255) | Human-readable label shown in the Flows UI |
| greeting_enabled | boolean | NN | When false, the greeting is skipped entirely |
| greeting_type | enum(Audio, TTS) | NN | Whether the greeting is a pre-recorded file or synthesized speech |
| greeting_audio_url | file_ref | | Object-store path or URL of the pre-recorded greeting; required when greeting_type = Audio |
| greeting_tts_text | text | | Text rendered by the TTS engine; required when greeting_type = TTS |
| tag | short_text | MAX(100) | Optional classification label (e.g., "Sales", "Support") |
| speech_recognition | boolean | NN | When true, spoken input is accepted in addition to DTMF |
| speech_language | short_text | MAX(20) | IETF BCP 47 locale for ASR (e.g., "en-US", "es-MX"); defaults to "en-US" |
| timeout_secs | integer | NN | Seconds to wait for caller input before retrying or routing to no_input destination; default 10 |
| max_retries | integer | NN | Number of times to replay the greeting on no/invalid input before giving up; default 3 |
| no_input_destination_type | short_text | MAX(50) | Entity type to route to when all retries are exhausted with no valid input |
| no_input_destination_id | uuid | | Entity ID corresponding to no_input_destination_type |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### VoiceMenuOption

**UI References:** Flows > Voice Menus page (option configuration panel)

**Relationships:**
- Many-to-one with VoiceMenu (cascade delete when parent VoiceMenu is removed)
- References a destination entity via destination_type / destination_id (polymorphic)

**Notes:** `dtmf_digit` accepts the characters 0–9, `*`, and `#`. The combination of `menu_id` and `dtmf_digit` must be unique — two options on the same menu cannot share a key. When `destination_type` is `Number`, `destination_number` carries the E.164 target and `destination_id` is null. For all other destination types, `destination_id` carries the foreign key and `destination_number` is null. Options are presented to callers in `sort_order` sequence.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| menu_id | uuid | FK(VoiceMenu), NN | Parent IVR menu; cascade delete |
| dtmf_digit | short_text | NN, MAX(1), UQ(menu_id) | The key the caller presses: 0–9, *, or # |
| description | short_text | MAX(255) | Internal label describing this option (e.g., "Press 1 for Sales") |
| destination_type | short_text | NN, MAX(50) | Type of the routing destination (Queue, VoiceMenu, Number, Voicemail, etc.) |
| destination_id | uuid | | Primary key of the destination entity; null when destination_type = Number |
| destination_number | e164 | | E.164 number to dial; populated only when destination_type = Number |
| sort_order | integer | NN | Display and announcement ordering within the menu |

---

### Queue

**UI References:** Flows > Queues page, Reports > Queue Report, Flows > Coaching page

**Relationships:**
- Many-to-one with Account
- Many-to-one with Schedule (optional; queue may operate without schedule enforcement)
- One-to-many with QueueAgent (agents assigned to this queue)
- Referenced as a destination by VoiceMenuOption, SmartRouterRule, GeoRouterRule, RoutingTableRoute, and TrackingNumber

**Notes:** `strategy` controls the order in which agents are offered calls. `Ring All` alerts every available agent simultaneously. `Round Robin` cycles through agents in rotation. `Longest Idle` prefers the agent who has been available longest since their last call. `Weighted` uses per-agent priority values from QueueAgent. `repeat_callers` is a free-enumeration field (e.g., "Same Agent", "Any Agent", "Priority Queue") controlling preferential routing for callers who have called before. When `max_wait_secs` is exceeded, the call routes to `no_answer_destination_type` / `no_answer_destination_id`. `wrap_up_secs` locks an agent in After Call Work status after each call before making them available again.

**Existing RustPBX overlap:** `rustpbx_queues` provides `id`, `name`, `spec` (JSON blob), and `is_active`. The 4iiz Queue entity replaces the JSON spec with discrete structured columns and adds schedule linkage, overflow routing, music-on-hold, and agent management.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Display name for the queue |
| strategy | enum(Ring All, Round Robin, Longest Idle, Weighted) | NN | Call distribution algorithm; default Ring All |
| repeat_callers | short_text | MAX(50) | Policy for routing callers who have called before (e.g., "Same Agent", "Any Agent") |
| prompt_enabled | boolean | NN | When true, an announcement is played to the caller when they enter the queue; default false |
| caller_id_display | enum(Caller Number, Tracking Number, Custom) | NN | What to show as caller ID on agent devices; default Caller Number |
| schedule_id | uuid | FK(Schedule) | Business hours schedule; null means the queue operates 24/7 |
| max_wait_secs | integer | NN | Seconds a caller waits before overflow routing applies; default 300 |
| no_answer_destination_type | short_text | MAX(50) | Entity type for overflow routing when max_wait_secs is exceeded |
| no_answer_destination_id | uuid | | Entity ID for overflow routing |
| moh_audio_url | file_ref | | Music-on-hold audio file path in object store |
| wrap_up_secs | integer | NN | After-call work timer in seconds before agent is made available; default 30 |
| is_active | boolean | NN | Whether the queue accepts calls; default true |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### QueueAgent

**UI References:** Flows > Queues page (agent assignment panel)

**Relationships:**
- Many-to-one with Queue (cascade delete when parent Queue is removed)
- Many-to-one with User/Agent
- An agent may belong to multiple queues (no uniqueness constraint at the agent level, only within a given queue)

**Notes:** The combination of `queue_id` and `agent_id` must be unique — an agent cannot be listed twice in the same queue. `priority` is used by the `Weighted` distribution strategy; lower numeric values indicate higher priority (1 = highest). When `is_active` is false the agent remains assigned but does not receive calls from this queue, which allows temporary suspension without losing queue membership configuration.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| queue_id | uuid | FK(Queue), NN | Parent queue; cascade delete |
| agent_id | uuid | FK(User), NN, UQ(queue_id) | The agent being assigned; unique per queue |
| priority | integer | NN | Routing priority for Weighted strategy; lower = higher priority; default 1 |
| is_active | boolean | NN | Whether the agent is currently eligible to receive calls from this queue; default true |
| added_at | timestamp_tz | NN | Timestamp when the agent was added to the queue |

---

### SmartRouter

**UI References:** Flows > Smart Routers page

**Relationships:**
- Many-to-one with Account
- One-to-many with SmartRouterRule (the ordered rule set evaluated at call time)
- Referenced as a destination by VoiceMenuOption, GeoRouterRule, RoutingTableRoute, and TrackingNumber

**Notes:** Rules are evaluated in `sort_order` ascending order; the first matching rule determines the destination. If no rule matches, the call falls through to `fallback_destination_type` / `fallback_destination_id`. Multiple SmartRouters may exist per account; `priority` can be used when a TrackingNumber references a router to resolve ambiguity, though typically one router is assigned per number.

**Existing RustPBX overlap:** `rustpbx_routes` supports priority-ordered matching with flexible conditions. SmartRouter operates at a higher level — it is marketing-context-aware (source attribution, repeat-caller status, campaign data) whereas RustPBX routes operate on raw SIP headers.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Display name for the router |
| description | text | | Optional extended description of the router's purpose |
| priority | integer | NN | Relative priority when multiple routers may apply; default 1 |
| fallback_destination_type | short_text | MAX(50) | Entity type to route to when no rules match |
| fallback_destination_id | uuid | | Entity ID for fallback routing |
| is_active | boolean | NN | Whether the router is evaluated at call time; default true |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### SmartRouterRule

**UI References:** Flows > Smart Routers page (rule editor)

**Relationships:**
- Many-to-one with SmartRouter (cascade delete when parent SmartRouter is removed)
- References a destination entity via destination_type / destination_id (polymorphic)

**Notes:** Rules are evaluated in ascending `sort_order`; the first rule whose condition evaluates to true determines the call destination. `condition_field` identifies what attribute of the call to inspect (e.g., `caller_area_code`, `time_of_day`, `source`, `repeat_caller`, `caller_state`, `score`). `condition_operator` and `condition_value` together form the predicate. For `in_range`, `condition_value` is expected to encode a range (e.g., "09:00-17:00" for time or "200-299" for score). When `destination_type` is `Number`, `destination_number` carries the E.164 target and `destination_id` is null.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| router_id | uuid | FK(SmartRouter), NN | Parent smart router; cascade delete |
| sort_order | integer | NN | Evaluation order; lower values are evaluated first |
| condition_field | short_text | NN, MAX(100) | Call attribute to evaluate (e.g., "caller_area_code", "time_of_day", "source", "repeat_caller") |
| condition_operator | enum(equals, not_equals, contains, starts_with, in_range, greater_than, less_than) | NN | Comparison operator applied to condition_field |
| condition_value | text | NN | Value to compare against; encoding depends on condition_field and condition_operator |
| destination_type | short_text | NN, MAX(50) | Entity type for the routing destination when this rule matches |
| destination_id | uuid | | Primary key of the destination entity; null when destination_type = Number |
| destination_number | e164 | | E.164 number to dial; populated only when destination_type = Number |

---

### GeoRouter

**UI References:** Flows > Geo Routers page

**Relationships:**
- Many-to-one with Account
- One-to-many with GeoRouterRule (the region-to-destination mapping table)
- Referenced as a destination by VoiceMenuOption, SmartRouterRule, RoutingTableRoute, and TrackingNumber

**Notes:** GeoRouter performs geographic call routing using the caller's area code, US state, or country derived from their E.164 number. When a caller's region does not match any GeoRouterRule, the call is sent to `default_destination_type` / `default_destination_id`. Geographic lookup is performed at call-entry time using the platform's internal NPA/NXX or geo-IP database; no additional configuration is required in the GeoRouter entity itself.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Display name for the geo router |
| description | text | | Optional extended description |
| default_destination_type | short_text | NN, MAX(50) | Entity type to route to when no region rule matches |
| default_destination_id | uuid | | Entity ID for the default destination |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### GeoRouterRule

**UI References:** Flows > Geo Routers page (region mapping panel)

**Relationships:**
- Many-to-one with GeoRouter (cascade delete when parent GeoRouter is removed)
- References a destination entity via destination_type / destination_id (polymorphic)

**Notes:** `region` holds a canonical code whose interpretation depends on `region_type`: for `State` it is a 2-letter US state abbreviation (e.g., "CA"), for `Country` it is an ISO 3166-1 alpha-2 code (e.g., "MX"), and for `Area Code` it is a 3-digit NANP area code string (e.g., "415"). The combination of `router_id` and `region` must be unique — two rules on the same router cannot map the same region. Rules are not explicitly ordered; the platform matches by exact region value.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| router_id | uuid | FK(GeoRouter), NN | Parent geo router; cascade delete |
| region | short_text | NN, MAX(10), UQ(router_id) | Geographic region code; interpretation depends on region_type |
| region_type | enum(State, Country, Area Code) | NN | Classification of the region code format |
| destination_type | short_text | NN, MAX(50) | Entity type for the routing destination |
| destination_id | uuid | | Primary key of the destination entity; null when destination_type = Number |
| destination_number | e164 | | E.164 number to dial; populated only when destination_type = Number |

---

### Schedule

**UI References:** Flows > Schedules page, Queue configuration (schedule_id), TrackingNumber configuration

**Relationships:**
- Many-to-one with Account
- One-to-many with ScheduleHoliday (holiday date overrides)
- One-to-many with Queue (a schedule can govern multiple queues)
- Referenced as a routing step by TrackingNumber and other routing entities via destination_type / destination_id

**Notes:** The column-per-day design is used here because the mock UI explicitly presents a grid of open/close times per weekday. A null `*_open` value means the business is closed on that day; `*_close` is also null in that case. `timezone` must be a valid IANA timezone identifier (e.g., "America/New_York", "America/Los_Angeles"). Holiday overrides in ScheduleHoliday take precedence over the weekly schedule for their specific date. When a call arrives outside business hours (or on a closed holiday), it is routed to `closed_destination_type` / `closed_destination_id`.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Display name for the schedule |
| timezone | short_text | NN, MAX(64) | IANA timezone identifier used when evaluating open/close times |
| monday_open | time | | Opening time on Monday; null = closed all day |
| monday_close | time | | Closing time on Monday; null when monday_open is null |
| tuesday_open | time | | Opening time on Tuesday; null = closed all day |
| tuesday_close | time | | Closing time on Tuesday; null when tuesday_open is null |
| wednesday_open | time | | Opening time on Wednesday; null = closed all day |
| wednesday_close | time | | Closing time on Wednesday; null when wednesday_open is null |
| thursday_open | time | | Opening time on Thursday; null = closed all day |
| thursday_close | time | | Closing time on Thursday; null when thursday_open is null |
| friday_open | time | | Opening time on Friday; null = closed all day |
| friday_close | time | | Closing time on Friday; null when friday_open is null |
| saturday_open | time | | Opening time on Saturday; null = closed all day |
| saturday_close | time | | Closing time on Saturday; null when saturday_open is null |
| sunday_open | time | | Opening time on Sunday; null = closed all day |
| sunday_close | time | | Closing time on Sunday; null when sunday_open is null |
| closed_destination_type | short_text | MAX(50) | Entity type to route to when the call arrives outside business hours |
| closed_destination_id | uuid | | Entity ID for closed-hours routing |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### ScheduleHoliday

**UI References:** Flows > Schedules page (holiday management panel)

**Relationships:**
- Many-to-one with Schedule (cascade delete when parent Schedule is removed)
- References a destination entity via override_destination_type / override_destination_id when the holiday has a distinct routing target

**Notes:** When `is_closed` is true the business is treated as fully closed on `date`, regardless of the normal weekly hours. When `is_closed` is false, `custom_open` and `custom_close` define special hours for that date (e.g., an early close on Christmas Eve). `override_destination_type` / `override_destination_id` allow routing to a holiday-specific destination (e.g., a special holiday voicemail greeting) instead of the schedule's standard `closed_destination`. If override fields are null, the schedule's normal `closed_destination` is used. The combination of `schedule_id` and `date` must be unique — only one holiday record may exist per date per schedule.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| schedule_id | uuid | FK(Schedule), NN | Parent schedule; cascade delete |
| date | date | NN, UQ(schedule_id) | The specific calendar date of the holiday override |
| name | short_text | NN, MAX(100) | Holiday name for display (e.g., "Thanksgiving", "Christmas Eve") |
| is_closed | boolean | NN | When true, the business is fully closed on this date; default true |
| custom_open | time | | Custom opening time when is_closed = false |
| custom_close | time | | Custom closing time when is_closed = false |
| override_destination_type | short_text | MAX(50) | Optional entity type for holiday-specific routing; null = use schedule's closed_destination |
| override_destination_id | uuid | | Optional entity ID for holiday-specific routing |

---

### RoutingTable

**UI References:** Flows > Routing Tables page

**Relationships:**
- Many-to-one with Account
- One-to-many with RoutingTableRoute (the ordered route entries)
- Referenced as a destination by other routing entities via destination_type / destination_id

**Notes:** RoutingTable provides a more flexible, lower-level routing mechanism than SmartRouter. While SmartRouter rules match on call-context attributes (marketing source, caller geography, time of day), RoutingTable routes match on raw patterns such as caller number prefixes, making it suitable for complex multi-destination distribution or fallback ladder scenarios. Routes within a table at the same `priority` level may use `weight` for proportional traffic splitting.

**Existing RustPBX overlap:** Conceptually similar to `rustpbx_routes`, which uses priority-ordered matching. The 4iiz RoutingTable adds weight-based distribution and sits above the SIP layer.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Display name for the routing table |
| description | text | | Optional extended description |
| is_active | boolean | NN | Whether this table is evaluated at call time; default true |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### RoutingTableRoute

**UI References:** Flows > Routing Tables page (route editor)

**Relationships:**
- Many-to-one with RoutingTable (cascade delete when parent RoutingTable is removed)
- References a destination entity via destination_type / destination_id (polymorphic)

**Notes:** Routes are evaluated in ascending `priority` order. Among routes at the same priority, `weight` controls proportional distribution — a route with weight 2 receives approximately twice the traffic of a route with weight 1 at the same priority level. `match_pattern` is a regular expression applied to the caller's E.164 number; a null pattern matches all callers (catch-all). When `destination_type` is `Number`, `destination_number` carries the E.164 target and `destination_id` is null.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| table_id | uuid | FK(RoutingTable), NN | Parent routing table; cascade delete |
| priority | integer | NN | Evaluation order; lower values are evaluated first |
| match_pattern | regex | | Regular expression matched against the caller's E.164 number; null = match all |
| destination_type | short_text | NN, MAX(50) | Entity type for the routing destination |
| destination_id | uuid | | Primary key of the destination entity; null when destination_type = Number |
| destination_number | e164 | | E.164 number to dial; populated only when destination_type = Number |
| weight | integer | NN | Proportional weight for traffic splitting among same-priority routes; default 1 |

---

### AgentScript

**UI References:** Flows > Agent Scripts page, CallDetailPanel > Script tab

**Relationships:**
- Many-to-one with Account
- Referenced by Queue or TrackingNumber configuration to associate a script with a call flow
- Read at call-answer time to render variables in the context of the active CallRecord

**Notes:** `content` uses a double-brace template syntax (e.g., `{{caller_name}}`, `{{source}}`, `{{tracking_number}}`, `{{score}}`). Variable resolution happens at render time using the live call context — variables are not stored in this entity. Scripts are authored in the Flows section and surfaced to agents in the CallDetailPanel's Script tab during an active call. Multiple scripts may exist per account; assignment to a specific call flow is handled via the Queue or TrackingNumber configuration, not stored on this entity.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Display name for the script |
| description | text | | Optional summary of the script's purpose and intended use |
| content | long_text | NN | Script body with {{variable}} template placeholders rendered at call time |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### VoicemailBox

**UI References:** Flows > Voicemail page, TrackingNumber configuration, Queue overflow configuration

**Relationships:**
- Many-to-one with Account
- One-to-many with VoicemailMessage (messages deposited in this mailbox)
- Referenced as a destination by VoiceMenuOption, SmartRouterRule, GeoRouterRule, RoutingTableRoute, Queue, and Schedule via destination_type / destination_id

**Notes:** `greeting_type` of `Default` plays the platform's built-in greeting. `Custom` plays `greeting_audio_url`. `None` skips the greeting and begins recording immediately. When `transcription_enabled` is true, each deposited message is automatically transcribed and stored on VoicemailMessage. When `email_notification_enabled` is true, a notification containing the transcription (if available) and a link to the audio is sent to `notification_email` each time a new message is received. `max_messages` is a soft cap; when the mailbox is full, callers are notified and the call follows a configurable overflow path (implementation detail, not stored on this entity).

**Existing RustPBX overlap:** `rustpbx_voicemail_greetings` stores greeting audio separately. `rustpbx_voicemails` stores individual messages. The 4iiz VoicemailBox consolidates mailbox identity, capacity configuration, and greeting management into one entity.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(255) | Mailbox display name |
| description | text | | Optional description of the mailbox's purpose |
| max_message_length_secs | integer | NN | Maximum recording length per message in seconds (e.g., 30, 60, 120, 300); default 120 |
| greeting_type | enum(Default, Custom, None) | NN | Greeting playback mode; default Default |
| greeting_audio_url | file_ref | | Object-store path of the custom greeting; required when greeting_type = Custom |
| transcription_enabled | boolean | NN | When true, deposited messages are automatically transcribed; default true |
| email_notification_enabled | boolean | NN | When true, an email is sent to notification_email on each new message; default false |
| notification_email | email | | Email address for new-message notifications; required when email_notification_enabled = true |
| max_messages | integer | NN | Soft cap on stored messages; default 100 |
| created_at | timestamp_tz | NN | Record creation timestamp |
| updated_at | timestamp_tz | NN | Last modification timestamp |

---

### VoicemailMessage

**UI References:** Flows > Voicemail page (message list), CallDetailPanel (voicemail tab)

**Relationships:**
- Many-to-one with VoicemailBox (cascade delete when parent VoicemailBox is removed)
- Many-to-one with CallRecord (optional; links the voicemail to the originating call)

**Notes:** `call_id` links the voicemail message back to the CallRecord that resulted in the deposit, enabling full call-flow tracing. It may be null if the voicemail was deposited outside a tracked call (e.g., via direct SIP extension). `transcription` is populated asynchronously after the recording is processed; it may be null while transcription is pending. `is_read` is toggled when an agent opens the message in the UI or plays the audio. `recorded_at` represents the moment recording began, while `created_at` represents the moment the record was persisted (these may differ slightly due to processing lag).

**Existing RustPBX overlap:** `rustpbx_voicemails` provides `id`, `mailbox_id`, `caller_id`, `recording_path`, `is_read`, and `transcript_text`. The 4iiz VoicemailMessage adds `call_id` for call-record linkage, `caller_name`, `duration_secs`, `audio_url` (object-store reference replacing raw path), and `recorded_at` for precise timestamping.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| mailbox_id | uuid | FK(VoicemailBox), NN | Parent mailbox; cascade delete |
| call_id | uuid | FK(CallRecord) | Link to the originating CallRecord; null if call was not tracked |
| caller_number | e164 | NN | E.164 number of the caller who left the message |
| caller_name | short_text | MAX(255) | Caller name from CNAM lookup or contact record, if available |
| duration_secs | duration_sec | NN | Length of the recorded message in whole seconds |
| audio_url | file_ref | NN | Object-store path or URL of the voicemail recording |
| transcription | long_text | | ASR-generated text transcript; null while pending or if transcription is disabled |
| is_read | boolean | NN | Whether the message has been opened/listened to by an agent; default false |
| recorded_at | timestamp_tz | NN | Timestamp when recording of the message began |
| created_at | timestamp_tz | NN | Timestamp when the record was persisted to the database |

---

## Polymorphic Routing Pattern

Throughout this domain, call routing destinations are expressed as a pair of columns rather than a typed foreign key:

- `destination_type` (short_text) — the logical name of the target entity kind
- `destination_id` (uuid) — the primary key of the specific record within that entity kind

This pattern appears on VoiceMenuOption, SmartRouterRule, GeoRouterRule, RoutingTableRoute, VoiceMenu (no_input destination), Queue (overflow destination), Schedule (closed destination and holiday override destination), and ScheduleHoliday.

### Why Polymorphic Rather Than Typed

A call at any routing step may be directed to a Queue, another VoiceMenu, a VoicemailBox, a GeoRouter, a SmartRouter, a RoutingTable, a Schedule, or an external phone number. If each possible target were expressed as a separate nullable foreign key column (e.g., `destination_queue_id`, `destination_menu_id`, `destination_voicemail_id`, …), every routing entity would carry seven or more always-mostly-null columns, and adding a new routable entity type would require schema migrations across every routing table. The polymorphic pair requires no schema change when new destination types are introduced.

### Destination Type Registry

The following destination type string values are defined for this domain. The application layer is responsible for enforcing that the referenced entity exists.

| destination_type value | Target Entity | Notes |
|------------------------|--------------|-------|
| `Queue` | Queue | Routes to an ACD queue for agent distribution |
| `VoiceMenu` | VoiceMenu | Routes to an IVR menu for DTMF collection |
| `GeoRouter` | GeoRouter | Routes to geographic routing logic |
| `SmartRouter` | SmartRouter | Routes to conditional rule-based routing |
| `RoutingTable` | RoutingTable | Routes to a priority/weighted routing table |
| `Schedule` | Schedule | Routes through a business-hours gate |
| `VoicemailBox` | VoicemailBox | Routes to a voicemail mailbox |
| `Number` | (none) | Routes to an external E.164 number; destination_id is null, destination_number is populated |
| `AgentExtension` | User | Routes directly to a specific agent's SIP extension |
| `Hangup` | (none) | Terminates the call gracefully; both destination_id and destination_number are null |

### Referential Integrity Considerations

Because standard foreign-key constraints cannot enforce polymorphic references across multiple target tables, referential integrity for this pattern must be enforced at the application layer. Recommended practices:

- Validate that `destination_id` resolves to a record in the correct table for the given `destination_type` on every write.
- When a routable entity is deleted, search all routing tables for references to it and either cascade-nullify or reassign to a safe fallback destination, logged as a system event.
- Prevent circular routing chains (e.g., VoiceMenu A → VoiceMenu B → VoiceMenu A) at configuration time through graph traversal validation, not at the database layer.

### Destination_number and destination_id Mutual Exclusivity

When `destination_type` is `Number`, `destination_id` must be null and `destination_number` must be a valid E.164 value. For all other destination types, `destination_number` must be null and `destination_id` must reference a valid record. This mutual exclusivity is enforced by the application layer and should be documented as a constraint in API validation logic.
