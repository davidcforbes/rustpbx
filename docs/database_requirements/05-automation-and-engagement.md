# 05 — Automation & Engagement

## Overview

This domain covers all event-driven automation, outbound engagement, and third-party integration capabilities of the 4iiz platform. It encompasses the visual workflow builder (drag-and-drop node graphs), rule-based triggers, serverless lambda functions, outbound webhooks, bulk SMS/MMS campaigns, smart dialer campaigns, lead reactor follow-up sequences, form reactor integrations, scheduled reminders, keyword spotting configurations, and the embeddable chat widget.

The unifying design principle is that every significant system event — an inbound call, a missed call, a form submission, an SMS received — can be intercepted and acted upon through one or more of these mechanisms. Triggers and workflows consume events and produce actions; webhooks propagate events to external systems; lambdas provide custom logic execution. Outbound engagement tools (bulk messages, smart dialer, reminders) initiate contact proactively. The keyword spotting and chat widget entities configure how AI-assisted and web-based engagement channels are set up and attributed.

---

### Workflow

**UI References:** Flows > Workflows page (canvas builder)

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many WorkflowNodes (one-to-many, cascade delete)
- Has many WorkflowEdges (one-to-many, cascade delete)

**Notes:** The canvas_json field is a full serialization of the node graph sufficient for the frontend renderer to reconstruct the visual canvas without querying WorkflowNode or WorkflowEdge individually. It serves as the source of truth for rendering and must be kept in sync whenever nodes or edges are mutated. Status transitions follow: Draft → Active, Active → Paused, Paused → Active. Only Active workflows are eligible for event-driven execution.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable workflow name |
| description | text | | Optional longer description of workflow purpose |
| canvas_json | json | | Full serialized node graph for UI canvas rendering, including node positions and edge paths |
| status | enum(Active, Draft, Paused) | NN | Execution eligibility; only Active workflows run |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### WorkflowNode

**UI References:** Flows > Workflows page (canvas nodes)

**Relationships:**
- Belongs to one Workflow (many-to-one, cascade delete)
- Referenced as from_node_id or to_node_id by WorkflowEdge (one-to-many)

**Notes:** node_type determines which of event_type, action_type, or condition_type is applicable and required. Event nodes are entry points — a workflow may have multiple entry event nodes, each independently triggering the graph. Action nodes execute side effects. Condition nodes branch execution based on runtime evaluation of condition_type logic. config_json carries all node-type-specific parameterization (e.g., SMS template, HTTP endpoint URL, delay duration, condition expression). The label field is the user-assigned display name shown on the canvas tile.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| workflow_id | uuid | FK(Workflow), NN | Parent workflow |
| node_type | enum(Event, Condition, Action) | NN | Structural role of this node in the graph |
| event_type | enum(Call Received, Call Answered, Call Missed, Voicemail Left, Form Submitted, Chat Started, SMS Received) | | Entry trigger event; only applicable when node_type = Event |
| action_type | enum(Send SMS, Send Email, Create Task, Assign Agent, Apply Tag, Transfer Call, HTTP Request, Delay) | | Side effect to perform; only applicable when node_type = Action |
| condition_type | short_text | MAX(100) | Condition expression identifier or DSL key; only applicable when node_type = Condition |
| config_json | json | | Node-specific configuration payload (template content, endpoint, delay value, condition operands, etc.) |
| label | short_text | MAX(80) | Display label shown on the canvas tile |
| position_x | float | NN | Horizontal canvas coordinate for rendering |
| position_y | float | NN | Vertical canvas coordinate for rendering |
| created_at | timestamp_tz | NN | Record creation time |

---

### WorkflowEdge

**UI References:** Flows > Workflows page (canvas connections)

**Relationships:**
- Belongs to one Workflow (many-to-one, cascade delete)
- References WorkflowNode as from_node_id (many-to-one)
- References WorkflowNode as to_node_id (many-to-one)

**Notes:** UQ(workflow_id, from_node_id, to_node_id) prevents duplicate connections between the same node pair within a workflow. Condition nodes typically emit multiple outgoing edges, each with a distinct label (e.g., "Yes", "No", "Timeout", or named branches). sort_order controls which branch is evaluated first when a condition node has ambiguous or ordered branch resolution. Both from_node_id and to_node_id must belong to the same workflow_id to maintain graph integrity.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| workflow_id | uuid | FK(Workflow), NN | Parent workflow; must match the workflow of both referenced nodes |
| from_node_id | uuid | FK(WorkflowNode), NN | Source node of the directed connection |
| to_node_id | uuid | FK(WorkflowNode), NN | Destination node of the directed connection |
| label | short_text | MAX(60) | Branch label displayed on the canvas edge (e.g., "Yes", "No", "Timeout") |
| sort_order | integer | default 0 | Evaluation order for ordered branch resolution from the same source node |

---

### Trigger

**UI References:** Flows > Triggers page

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many TriggerConditions (one-to-many, cascade delete)
- Has many TriggerActions (one-to-many, cascade delete)

**Notes:** A trigger is the simpler, rule-based alternative to a workflow — it evaluates a flat list of conditions and, when all are satisfied, executes a flat list of actions in order. run_on defines the scope of the trigger: "All Numbers" applies it account-wide, while a specific number or source name scopes it to matching inbound channels. runs_7d is a rolling counter updated by the event processing pipeline and is used for display-only purposes on the triggers list page; it is not authoritative for billing or audit. Only Active triggers participate in event evaluation.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable trigger name |
| trigger_event | enum(Call Received, Call Answered, Call Missed, Voicemail Left, Form Submitted, Chat Started, SMS Received) | NN | System event that initiates evaluation of this trigger |
| run_on | short_text | MAX(200) | Scope descriptor — "All Numbers", a specific number identifier, or a named source |
| runs_7d | counter | default 0 | Rolling 7-day execution count, maintained by the event pipeline for display |
| status | enum(Active, Paused) | NN | Only Active triggers are evaluated against incoming events |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### TriggerCondition

**UI References:** Flows > Triggers page (condition builder)

**Relationships:**
- Belongs to one Trigger (many-to-one, cascade delete)

**Notes:** Multiple TriggerConditions for a single trigger are evaluated with AND semantics by default — all conditions must pass for the trigger to fire. sort_order controls the display and evaluation sequence in the UI condition builder. The field attribute is a logical field path referencing properties of the event context (e.g., "caller_area_code", "call_duration", "source", "time_of_day", "to_number"). The value field stores the comparison operand as a string; the evaluation engine casts it to the appropriate type based on the operator and field.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| trigger_id | uuid | FK(Trigger), NN | Parent trigger |
| sort_order | integer | NN | Display and evaluation sequence order |
| field | short_text | NN, MAX(100) | Event context field to evaluate (e.g., "caller_area_code", "call_duration", "source", "time_of_day") |
| operator | enum(equals, not_equals, contains, starts_with, greater_than, less_than, in_list, not_in_list, between) | NN | Comparison operator applied between field value and value |
| value | text | NN | Comparison operand; comma-delimited for in_list/not_in_list, pipe-delimited for between |

---

### TriggerAction

**UI References:** Flows > Triggers page (action builder)

**Relationships:**
- Belongs to one Trigger (many-to-one, cascade delete)

**Notes:** Actions are executed sequentially in sort_order order after all conditions pass. action_config is a typed JSON payload whose schema is determined by action_type — for example, Send SMS requires a template body and sender number; HTTP Request requires a URL, method, and headers; Assign Agent requires an agent or queue identifier. Action execution is best-effort by default; failed actions may be retried depending on platform configuration and action_type.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| trigger_id | uuid | FK(Trigger), NN | Parent trigger |
| sort_order | integer | NN | Execution sequence order (ascending) |
| action_type | enum(Send SMS, Send Email, Create Task, Assign Agent, Apply Tag, HTTP Request, Notify, Transfer Call) | NN | Type of action to perform |
| action_config | json | NN | Type-specific configuration payload (template, recipients, URL, agent ID, tag name, etc.) |

---

### Lambda

**UI References:** Flows > Lambdas page (code editor)

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many LambdaEnvVars (one-to-many, cascade delete)

**Notes:** Lambdas are sandboxed serverless functions invoked synchronously or asynchronously by webhooks, triggers, workflows, or direct API calls. The runtime field determines the execution sandbox. timeout_ms and memory_mb are soft resource caps enforced by the execution environment. handler identifies the exported function entry point within the code. invocation_count and error_count are cumulative lifetime counters maintained by the execution pipeline; last_invoked_at is updated on every invocation regardless of success. Code is stored as plain text in the database; it is not compiled or validated at save time.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable lambda name, used for invocation addressing |
| description | text | | Optional description of purpose and behavior |
| runtime | enum(Node.js 18, Python 3.11, Go 1.21) | NN | Execution sandbox and language runtime |
| code | long_text | NN | Full function source code |
| handler | short_text | default "handler", MAX(100) | Exported entry point function name within the code |
| timeout_ms | integer | default 30000 | Maximum allowed execution time in milliseconds |
| memory_mb | integer | default 128 | Maximum memory allocation in megabytes |
| last_invoked_at | timestamp_tz | | Timestamp of most recent invocation (success or failure) |
| invocation_count | counter | default 0 | Cumulative lifetime invocation count |
| error_count | counter | default 0 | Cumulative lifetime error count |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### LambdaEnvVar

**UI References:** Flows > Lambdas page (env var editor)

**Relationships:**
- Belongs to one Lambda (many-to-one, cascade delete)

**Notes:** UQ(lambda_id, key) prevents duplicate variable names within a single lambda. Values are encrypted at rest because this table commonly stores API keys, signing secrets, database credentials, and other sensitive configuration. The encryption key management strategy is external to this schema. Values are injected into the runtime environment at invocation time and are never returned in plaintext via API responses.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| lambda_id | uuid | FK(Lambda), NN | Parent lambda function |
| key | short_text | NN, MAX(100) | Environment variable name; must be a valid identifier for the target runtime |
| value | encrypted_text | NN | Variable value, encrypted at rest; injected at invocation time |

---

### Webhook

**UI References:** Flows > Webhooks page

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many WebhookSubscriptions (one-to-many, cascade delete)
- Has many WebhookDeliveries (one-to-many)

**Notes:** A webhook defines the outbound HTTP destination and delivery configuration. The actual set of events it listens to is defined by its WebhookSubscriptions. secret is used for HMAC-SHA256 request signing — the platform computes a signature over the request body and includes it in a header so the recipient can verify authenticity. retry_count and retry_delay_secs control the exponential backoff retry behavior on non-2xx responses or connection failures. last_triggered_at is updated on every delivery attempt.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable webhook name |
| trigger_event | short_text | NN, MAX(100) | Primary event category description (denormalized summary; authoritative list is in WebhookSubscription) |
| callback_url | url | NN | Destination URL for HTTP delivery |
| method | enum(POST, PUT, GET) | NN, default POST | HTTP method for delivery requests |
| body_type | enum(JSON, Form, XML) | NN, default JSON | Serialization format of the request body |
| headers | json | | Custom HTTP headers to include with every delivery request |
| secret | encrypted_text | | HMAC signing secret; encrypted at rest; used for request signature generation |
| retry_count | integer | default 3 | Maximum number of retry attempts after initial failure |
| retry_delay_secs | integer | default 60 | Base delay in seconds between retry attempts |
| status | enum(Active, Paused) | NN, default Active | Only Active webhooks are evaluated for delivery |
| last_triggered_at | timestamp_tz | | Timestamp of most recent delivery attempt |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### WebhookSubscription

**UI References:** Flows > Webhooks page (event selection)

**Relationships:**
- Belongs to one Webhook (many-to-one, cascade delete)

**Notes:** UQ(webhook_id, event_type) prevents duplicate subscriptions for the same event on the same webhook. event_type values follow a dot-namespaced convention matching the platform event taxonomy (e.g., "call.completed", "call.missed", "sms.received", "form.submitted"). A webhook without any subscriptions will never be triggered.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| webhook_id | uuid | FK(Webhook), NN | Parent webhook |
| event_type | short_text | NN, MAX(100) | Dot-namespaced event identifier (e.g., "call.completed", "sms.received") |

---

### WebhookDelivery

**UI References:** Flows > Webhooks page (delivery log)

**Relationships:**
- Belongs to one Webhook (many-to-one)

**Notes:** This is an append-only audit log of every delivery attempt. Each retry creates a new row with an incremented attempt_number rather than updating an existing row. http_status_code is null when the delivery failed before receiving an HTTP response (e.g., DNS failure, connection refused, timeout). response_body should be truncated to a reasonable maximum (e.g., 4 KB) to bound storage growth. A retention policy should be applied to this table — suggested default is 90 days. delivered_at records when the attempt was made, not when it succeeded.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| webhook_id | uuid | FK(Webhook), NN | Parent webhook |
| event_type | short_text | NN, MAX(100) | Event type that triggered this delivery attempt |
| payload | json | NN | Full request body that was (or would have been) sent |
| http_status_code | integer | | HTTP response status code; null if no response was received |
| response_body | text | | Truncated HTTP response body for debugging |
| status | enum(Success, Failed, Pending) | NN | Outcome of this specific attempt |
| attempt_number | integer | NN, default 1 | Ordinal retry attempt (1 = initial, 2+ = retries) |
| delivered_at | timestamp_tz | NN | Timestamp when this attempt was made |

---

### BulkMessage

**UI References:** Flows > Bulk Messages page

**Relationships:**
- Belongs to one Account (many-to-one)
- References TrackingNumber as sender_number_id (many-to-one)
- References ContactList as contact_list_id (many-to-one, nullable)

**Notes:** sender_phone is a denormalized display copy of the sender number's E.164 value, retained for historical accuracy even if the TrackingNumber is later reassigned. recipient_count is set at campaign creation from the resolved contact list size. sent_count, delivered_count, and failed_count are incremented by the messaging pipeline as individual messages are dispatched and delivery receipts are received. Status transitions: Draft → Scheduled (if scheduled_at is set) or Draft → Sending (immediate dispatch), Sending → Completed or Failed. A Completed campaign is immutable. scheduled_at null means the campaign sends immediately upon activation.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| label | short_text | NN, MAX(120) | Campaign name for display and reporting |
| sender_number_id | uuid | FK(TrackingNumber), NN | Tracking number used as the SMS sender |
| sender_phone | e164 | NN | Denormalized E.164 sender number for historical display |
| message_body | long_text | NN | Message template content; may contain merge tags |
| msg_type | enum(SMS, MMS) | NN, default SMS | Message type; MMS supports media attachments |
| contact_list_id | uuid | FK(ContactList) | Source contact list; null if recipients were specified ad hoc |
| recipient_count | integer | NN | Total number of recipients in the campaign at creation time |
| sent_count | counter | default 0 | Number of messages dispatched to the carrier |
| delivered_count | counter | default 0 | Number of messages confirmed delivered by carrier receipt |
| failed_count | counter | default 0 | Number of messages that failed delivery |
| status | enum(Draft, Scheduled, Sending, Completed, Cancelled, Failed) | NN, default Draft | Campaign lifecycle state |
| scheduled_at | timestamp_tz | | Future send time; null means send immediately on activation |
| started_at | timestamp_tz | | Timestamp when sending began |
| completed_at | timestamp_tz | | Timestamp when all messages were processed |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### LeadReactorConfig

**UI References:** Flows > Lead Reactors page (stub)

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many LeadReactorActions (one-to-many, cascade delete)

**Notes:** A Lead Reactor defines an automated follow-up sequence triggered when a new lead arrives through a specific event type. delay_minutes introduces a hold period before the first action fires — useful for allowing agents a window to respond manually before automation intervenes. working_hours_only defers action execution until within the account's configured business hours. max_retries controls how many times the reactor re-attempts failed actions before giving up. is_active gates whether the reactor participates in event evaluation.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable reactor name |
| description | text | | Optional description of reactor purpose and intended use |
| trigger_event | enum(New Lead, Form Submission, Missed Call) | NN | Event type that activates this reactor |
| delay_minutes | integer | NN, default 0 | Minutes to wait before executing actions after event fires |
| is_active | boolean | NN, default true | Whether this reactor participates in event evaluation |
| working_hours_only | boolean | NN, default false | If true, defers execution until within configured business hours |
| max_retries | integer | default 3 | Maximum retry attempts for failed actions |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### LeadReactorAction

**UI References:** Flows > Lead Reactors page (action configuration)

**Relationships:**
- Belongs to one LeadReactorConfig (many-to-one, cascade delete)

**Notes:** Actions execute in ascending sort_order. template_content holds the message body for SMS/email actions and may contain merge tags referencing lead context fields. action_config carries supplementary type-specific parameters not represented by template_content (e.g., sender number for SMS, from address for email, task priority for task creation, agent or queue identifier for assignment). Actions of type Assign Agent may reference either a specific agent ID or a queue ID within action_config.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| config_id | uuid | FK(LeadReactorConfig), NN | Parent lead reactor configuration |
| sort_order | integer | NN | Execution sequence order (ascending) |
| action_type | enum(Send SMS, Send Email, Create Task, Assign Agent) | NN | Type of action to execute |
| template_content | long_text | | Message template for SMS and email actions; supports merge tag interpolation |
| action_config | json | | Additional type-specific configuration (sender, recipient, task priority, agent/queue ID, etc.) |

---

### SmartDialerConfig

**UI References:** Flows > Smart Dialer page (stub)

**Relationships:**
- Belongs to one Account (many-to-one)
- References ContactList as contact_list_id (many-to-one, nullable)
- References AgentScript as agent_script_id (many-to-one, nullable)

**Notes:** Smart Dialer manages outbound dialing campaigns with three modes: Preview (agent reviews the contact before dialing), Progressive (system dials when an agent becomes available), and Predictive (system dials ahead of agent availability using an algorithm). max_concurrent defines the ceiling on simultaneous outbound lines. active_days is a bitmask where each bit represents a day of the week (Monday = bit 0, value 1; Sunday = bit 6, value 64); the full week is value 127. start_time and end_time define the daily calling window within the specified timezone. is_active gates whether the campaign is currently running; setting to false suspends all outbound dialing.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Campaign name for display and reporting |
| description | text | | Optional description of campaign purpose |
| mode | enum(Preview, Progressive, Predictive) | NN, default Preview | Dialing mode governing agent-call coordination behavior |
| max_concurrent | integer | NN, default 1 | Maximum number of simultaneous outbound calls |
| ring_timeout_secs | integer | NN, default 30 | Seconds to ring before treating a call as unanswered |
| retry_attempts | integer | default 2 | Number of retry attempts for unanswered or failed calls |
| retry_interval_minutes | integer | default 60 | Minutes to wait before retrying an unanswered contact |
| outbound_number | e164 | NN | Caller ID (E.164) presented to called parties |
| outbound_cnam | short_text | MAX(15) | CNAM display name presented to called parties |
| start_time | time | NN | Earliest time of day to place calls (local timezone) |
| end_time | time | NN | Latest time of day to place calls (local timezone) |
| timezone | short_text | NN, MAX(60) | IANA timezone name governing calling window |
| active_days | bitmask | NN | Days of week bitmask; Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64 |
| contact_list_id | uuid | FK(ContactList) | Contact list providing numbers to dial; null if manually populated |
| agent_script_id | uuid | FK(AgentScript) | Script surfaced to agents during calls; null if no script assigned |
| is_active | boolean | NN, default false | Whether the campaign is currently running and placing calls |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### FormReactorEntry

**UI References:** Flows > Form Reactors page

**Relationships:**
- Belongs to one Account (many-to-one)
- References TrackingNumber as tracking_number_id (many-to-one, nullable)

**Notes:** A Form Reactor connects a third-party web form submission to the call tracking platform. When a form post is received (typically via a platform-generated webhook endpoint or JavaScript snippet), the entry records the submission and attributes it to the linked TrackingNumber for source reporting. form_fields describes the expected field mapping (e.g., "phone -> caller_phone, email -> contact_email") — this is a human-readable descriptor in this pre-analysis document; the actual mapping logic may be expressed as structured JSON in a later design iteration. call_count increments on each attributed form submission event.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable reactor name |
| form_fields | short_text | MAX(500) | Human-readable description of form field mapping to platform fields |
| tracking_number_id | uuid | FK(TrackingNumber) | Tracking number to attribute form submissions to; null for unattributed |
| call_count | counter | default 0 | Cumulative count of form submissions processed by this reactor |
| status | enum(Active, Paused) | NN, default Active | Whether this reactor is processing incoming form submissions |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### Reminder

**UI References:** Flows > Reminders page (stub), CallDetailPanel > Reminder tab

**Relationships:**
- Belongs to one Account (many-to-one)
- References ContactList as contact_list_id (many-to-one, nullable)
- References CallRecord as call_id (many-to-one, nullable)

**Notes:** A Reminder schedules a future outbound contact with a specific person or group via call, text, or email. contact_source determines how recipients are resolved: Recent Calls uses recent call history, Contact List uses the linked contact_list_id, and Manual uses the explicit contact_phone value. For recurring reminders, recurrence_rule stores an iCalendar RRULE string (e.g., "FREQ=WEEKLY;BYDAY=MO") and remind_at is the initial occurrence; the system computes future occurrences at runtime. When linked to a CallRecord via call_id, the reminder appears in the CallDetailPanel Reminder tab for context. Status transitions: Scheduled → Sent or Failed; can be Cancelled before firing.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable reminder name |
| description | text | | Optional description of reminder context or purpose |
| timezone | short_text | NN, MAX(60) | IANA timezone name for interpreting remind_at |
| remind_at | timestamp_tz | NN | Scheduled delivery time (first occurrence for recurring reminders) |
| is_recurring | boolean | NN, default false | Whether this reminder repeats on a schedule |
| recurrence_rule | rrule | | iCalendar RRULE string for recurring schedule; only applicable when is_recurring = true |
| contact_source | enum(Recent Calls, Contact List, Manual) | NN | Method for resolving reminder recipients |
| contact_phone | e164 | | Explicit E.164 phone number; only applicable when contact_source = Manual |
| contact_list_id | uuid | FK(ContactList) | Source contact list; only applicable when contact_source = Contact List |
| delivery_method | enum(Call, Text, Email) | NN | Channel used to deliver the reminder |
| recipient | short_text | NN, MAX(200) | Phone number or email address of the reminder recipient |
| message | long_text | NN | Reminder message content or script |
| status | enum(Scheduled, Sent, Cancelled, Failed) | NN, default Scheduled | Current lifecycle state of the reminder |
| call_id | uuid | FK(CallRecord) | Associated call record for context linkage; null if not call-linked |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### KeywordSpottingConfig

**UI References:** Flows > Keyword Spotting page (stub)

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many KeywordSpottingKeywords (one-to-many, cascade delete)
- Has many KeywordSpottingNumbers (one-to-many, cascade delete)

**Notes:** A Keyword Spotting configuration defines a named set of keywords and phrases to detect in call transcriptions. When apply_to_all_numbers is true, the configuration applies to all transcribed calls for the account and the KeywordSpottingNumber junction table is unused. When false, only calls through numbers listed in KeywordSpottingNumber are analyzed. sensitivity controls the transcription matching threshold — Low requires higher confidence, High allows fuzzier matches. is_active gates participation in post-transcription analysis.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable configuration name |
| description | text | | Optional description of detection intent |
| sensitivity | enum(Low, Medium, High) | NN, default Medium | Matching sensitivity threshold for transcription analysis |
| apply_to_all_numbers | boolean | NN, default true | If true, applies to all account numbers; if false, scoped to KeywordSpottingNumber records |
| is_active | boolean | NN, default true | Whether this configuration participates in post-transcription analysis |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### KeywordSpottingKeyword

**UI References:** Flows > Keyword Spotting page (keyword editor)

**Relationships:**
- Belongs to one KeywordSpottingConfig (many-to-one, cascade delete)

**Notes:** Each keyword or phrase is evaluated independently against call transcriptions. category classifies the keyword's business significance: Positive keywords indicate favorable outcomes (e.g., "ready to buy", "set appointment"), Negative keywords indicate unfavorable outcomes (e.g., "cancel", "refund"), and Neutral keywords are informational markers. score_weight modulates the keyword's contribution to the overall call quality or lead score computation; a weight of 2.0 for a Positive keyword doubles its impact relative to a weight of 1.0.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| config_id | uuid | FK(KeywordSpottingConfig), NN | Parent keyword spotting configuration |
| keyword | short_text | NN, MAX(200) | Keyword or phrase to detect in transcription text |
| category | enum(Positive, Negative, Neutral) | NN, default Neutral | Business classification of the keyword's sentiment or significance |
| score_weight | float | default 1.0 | Multiplier applied to this keyword's contribution to overall call scoring |

---

### KeywordSpottingNumber

**UI References:** Flows > Keyword Spotting page (number selection)

**Relationships:**
- Belongs to one KeywordSpottingConfig (many-to-one, cascade delete)
- References TrackingNumber (many-to-one)

**Notes:** UQ(config_id, tracking_number_id) prevents duplicate associations. This junction table is only consulted when the parent KeywordSpottingConfig has apply_to_all_numbers = false. Deleting a TrackingNumber should cascade-delete or nullify associated junction records; the preferred behavior is deletion to avoid orphaned references.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| config_id | uuid | FK(KeywordSpottingConfig), NN | Parent keyword spotting configuration |
| tracking_number_id | uuid | FK(TrackingNumber), NN | Tracking number scoped to this configuration |

---

### ChatWidget

**UI References:** Flows > Chat Widget page

**Relationships:**
- Belongs to one Account (many-to-one)
- References TrackingNumber as tracking_number_id (many-to-one, nullable)
- References Queue as queue_id (many-to-one, nullable)

**Notes:** A Chat Widget represents an embeddable web chat interface deployed on a customer's website. The platform generates a JavaScript embed snippet keyed to the widget's id. config_json contains the full visual and behavioral configuration: brand colors, widget position (bottom-right, etc.), welcome message, pre-chat form field definitions, operating hours, and offline behavior. tracking_number_id links chat interactions to a specific tracking number for source attribution in reporting. queue_id routes incoming chats to a specific agent queue. chat_count increments on each new chat session initiated through this widget. Status Live means the widget is deployed and accepting chats; Draft means it is configured but not yet published.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable widget name for internal reference |
| website_url | url | | Target website URL where the widget is intended to be deployed |
| tracking_number_id | uuid | FK(TrackingNumber) | Tracking number for attribution of chat sessions in source reporting |
| routing_type | short_text | MAX(60) | Chat routing strategy description (e.g., "Round Robin", "First Available", "Queue") |
| queue_id | uuid | FK(Queue) | Agent queue to route incoming chats to; null if routing is not queue-based |
| agent_count | integer | default 0 | Number of agents currently assigned to handle this widget's chats |
| custom_fields_count | integer | default 0 | Number of pre-chat form fields configured |
| status | enum(Live, Draft) | NN, default Draft | Deployment state; only Live widgets accept chat sessions |
| config_json | json | | Full visual and behavioral configuration (colors, position, welcome message, branding, pre-chat form definition, operating hours, offline message) |
| chat_count | counter | default 0 | Cumulative count of chat sessions initiated through this widget |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

## Event Processing Architecture

### Event Flow Overview

When a significant system event occurs — a call is received, a voicemail is left, a form is submitted — the platform emits a typed event into an internal event bus. Multiple consumers can subscribe to this event concurrently: the Trigger evaluation engine, the Workflow execution engine, the Webhook dispatch pipeline, and the Lead Reactor scheduler. This fan-out model means a single "Call Missed" event can simultaneously fire a trigger that sends an SMS, enqueue a webhook delivery to an external CRM, and schedule a Lead Reactor follow-up sequence.

### Trigger Evaluation

The trigger evaluation engine receives each event and queries all Active triggers for the account whose trigger_event matches the event type. For each candidate trigger, it evaluates the TriggerCondition list sequentially (AND semantics). If all conditions pass, it enqueues the TriggerAction list for execution. runs_7d is incremented atomically at the point of successful evaluation (all conditions passed), not at action completion. This means runs_7d reflects evaluation count, not delivery success count.

### Workflow Execution

Workflow execution is initiated when an Event node within an Active workflow matches the incoming event type. Each matching workflow spawns an independent execution context that traverses the node graph, evaluating Condition branches and dispatching Action nodes in order. Execution contexts are isolated — a single event can activate multiple workflows simultaneously without cross-contamination. The canvas_json on the Workflow entity provides the full graph structure for the execution engine without requiring per-execution queries against WorkflowNode and WorkflowEdge.

### Webhook Dispatch

The webhook pipeline evaluates all Active webhooks whose WebhookSubscriptions include the event's type. For each matching webhook, it constructs the delivery payload, signs it using the secret field (HMAC-SHA256), and dispatches the HTTP request to callback_url. Every delivery attempt — whether initial or retry — is recorded as a new WebhookDelivery row (append-only). On non-2xx responses or network failures, the pipeline schedules retry attempts up to retry_count times, separated by retry_delay_secs intervals. The append-only nature of WebhookDelivery means the delivery log grows unboundedly; a retention policy (default recommendation: 90 days) must be applied.

### Lead Reactor Scheduling

When a qualifying event arrives and a matching LeadReactorConfig is active, the reactor does not execute immediately. Instead, it creates a deferred execution job that fires after delay_minutes have elapsed. If working_hours_only is true, the job is further deferred until the next available business hours window. This two-stage deferral means the actual action execution time may differ significantly from the event arrival time, and idempotency guards must account for this gap.

### Idempotency and Ordering

Several concerns arise from concurrent fan-out event processing:

**Idempotency:** Every trigger, workflow execution, and webhook delivery should carry a stable event ID derived from the originating event. Re-delivery or retry must not create duplicate actions (duplicate SMS sends, duplicate task creation). The system should use the event ID as an idempotency key at the point of action execution, not at the point of evaluation. WebhookDelivery's append-only structure naturally handles this: retries are new rows, not updates, and the recipient is responsible for deduplication using the event ID in the payload.

**Event Ordering:** The platform does not guarantee strict ordering of action execution across independent consumers. A trigger SMS action and a workflow email action responding to the same event may execute in any order. Where ordering matters within a single trigger or workflow, the sort_order (TriggerAction) and node graph topology (Workflow) provide the authoritative sequence.

**Retry Semantics:** Failed webhook deliveries retry up to retry_count times. Failed trigger actions and lead reactor actions retry up to max_retries (on LeadReactorConfig) times. Lambda invocation failures are recorded via error_count but retry behavior is governed by the invoking pipeline, not the Lambda entity itself. Retry exhaustion should produce a terminal Failed status on the relevant delivery or execution record rather than silently dropping the event.

**Event Versioning:** Events emitted by the platform should carry a schema version identifier in their payload so that downstream consumers (webhooks, lambdas) can handle breaking changes gracefully. The trigger_event and event_type fields across Trigger, Webhook, WorkflowNode, and other entities use consistent enum values to ensure that a single event taxonomy governs all consumers and prevents mismatch between emitter and evaluator.

**Back-pressure and Rate Limiting:** BulkMessage campaigns and SmartDialerConfig campaigns generate high-volume outbound activity that must not starve the real-time event processing pipeline. These should execute from a separate worker pool with independent resource limits. sent_count, delivered_count, and failed_count on BulkMessage are eventually consistent metrics updated by async delivery receipt processing and should not be treated as real-time totals.
