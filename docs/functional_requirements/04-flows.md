# 04 - Flows Section

> 23 pages | 22 well-defined, 1 stub
> Source: `ui/src/sections/flows.rs`

## Side Navigation

Three groups:
- **Routing:** Voice Menus, Queues, Smart Routers, Schedules, Geo Routers, Agent Scripts, Routing Tables, Voicemails
- **Automation:** Workflows, Triggers, Keyword Spotting, Lambdas, API Logs, Global, Webhooks
- **Engagement:** Bulk Messages, LeadReactor, Smart Dialers, FormReactor, Chat Widget, ChatAI, Dialogflow, Reminders

---

## Routing

### 4.1 Voice Menus Page

- **Route:** `/flows/voice-menus`
- **Status:** Well-defined

**Table Columns:** Name, Options (count), Greeting, Ring-To, Updated, Created, Actions (Edit/Remove)

**Data Model -- `VoiceMenu`:** name, options (u32), greeting, ring_to, updated, created

**Actions:** "New Voice Menu" button, Edit/Remove, search, pagination

**Implied Functionality:** IVR (Interactive Voice Response) menu builder with DTMF option routing

---

### 4.2 Queues Page

- **Route:** `/flows/queues`
- **Status:** Well-defined

**Table Columns:** Name, Agents (count), Strategy, Max Wait, Music On Hold, Updated, Created, Actions (Edit/Remove)

**Data Model -- `Queue`:** name, agents (u32), strategy, max_wait, moh, updated, created

**Actions:** "New Queue" button, Edit/Remove, search, pagination

**Implied Functionality:** ACD (Automatic Call Distribution) queue management with ring strategies (Ring All, Round Robin, Longest Idle), hold music

---

### 4.3 Smart Routers Page

- **Route:** `/flows/smart-routers`
- **Status:** Well-defined

**Table Columns:** Name, Rules (count), Priority, Fallback, Updated, Created, Actions (Edit/Remove)

**Data Model -- `SmartRouter`:** name, rules (u32), priority, fallback, updated, created

**Actions:** "New Smart Router" button, Edit/Remove, search, pagination

**Implied Functionality:** Conditional call routing with rule-based logic, priority levels, fallback destinations

---

### 4.4 Schedules Page

- **Route:** `/flows/schedules`
- **Status:** Well-defined

**Table Columns:** Name, Timezone, Open Hours, Closed Action, Holiday Count, Updated, Created, Actions (Edit/Remove)

**Data Model -- `Schedule`:** name, timezone, open_hours, closed_action, holidays (u32), updated, created

**Actions:** "New Schedule" button, Edit/Remove, search, pagination

**Implied Functionality:** Business hours scheduling with timezone support, after-hours routing, holiday management

---

### 4.5 Geo Routers Page

- **Route:** `/flows/geo-routers`
- **Status:** Well-defined

**UI:** Form card (Name, Description, Default destination) + Rules section (Region select by US State, Destination input, Add/Remove rule)

**Actions:** Save, Add Rule, Remove Rule

**Implied Functionality:** Geographic call routing based on caller location (area code / geo-IP)

---

### 4.6 Agent Scripts Page

- **Route:** `/flows/agent-scripts`
- **Status:** Well-defined

**UI:** Script editor form (Name, Description, Script Content textarea with template variable syntax) + Variables reference section listing: `{{caller_name}}`, `{{caller_number}}`, `{{source}}`, `{{agent_name}}`, `{{date}}`, `{{time}}`

**Actions:** Save

**Implied Functionality:** Agent-facing call scripts with dynamic variable substitution

---

### 4.7 Routing Tables Page

- **Route:** `/flows/routing-tables`
- **Status:** Well-defined

**UI:** Form card (Name, Description) + Routes table (Priority #, Match Pattern, Destination, Weight) with Add/Remove route

**Actions:** Save, Add Route, Remove Route

**Implied Functionality:** Explicit routing table with pattern matching, priority ordering, weighted distribution

---

### 4.8 Voicemails Page

- **Route:** `/flows/voicemails`
- **Status:** Well-defined

**UI:** General card (Name, Description) + Settings card:
- Max Message Length select (30s/60s/120s/300s)
- Greeting select (Default/Custom/None)
- Transcription toggle
- Email Notification toggle + Notification Email input

**Actions:** Save, toggle transcription, toggle email notification

**Implied Functionality:** Voicemail box configuration with transcription and email notification

---

## Automation

### 4.9 Workflows Page

- **Route:** `/flows/workflows`
- **Status:** Well-defined

**UI:** Two-panel layout:
- Left panel (256px): 6 draggable event types (Call Received, Call Answered, Call Missed, Voicemail Left, Form Submitted, Chat Started)
- Right panel: Canvas/workspace with dashed drop zone
- Toolbar: Zoom In / Zoom Out / Fit buttons, Save button

**Actions:** Zoom controls, Save, drag events onto canvas

**Implied Functionality:** Visual workflow builder (no-code automation) with event-driven triggers

> **NEEDS DEFINITION:** Drag-and-drop canvas interaction model, node connections, condition/action configuration within nodes

---

### 4.10 Triggers Page

- **Route:** `/flows/triggers`
- **Status:** Well-defined

**Table Columns:** Name, Event, Conditions (count), Actions (count), Status (Active/Paused), Updated, Created, Actions (Edit/Pause/Remove)

**Data Model -- `Trigger`:** name, event, conditions (u32), actions (u32), status, updated, created

**Actions:** "New Trigger" button, Edit/Pause/Remove, search, pagination

**Implied Functionality:** Event-driven automation triggers with conditional logic and configurable actions

---

### 4.11 Keyword Spotting Page

- **Route:** `/flows/keyword-spotting`
- **Status:** Well-defined

**UI:** General card (Name, Description) + Keywords card (keyword input, Category select: Positive/Negative/Neutral, Score input, Add/Remove) + Settings card (Sensitivity select: Low/Medium/High, "Apply to all calls" toggle, Tracking numbers multi-select)

**Actions:** Save, Add/Remove Keyword, adjust sensitivity

**Implied Functionality:** Post-call speech analytics with keyword/phrase detection, sentiment scoring, category classification

---

### 4.12 Lambdas Page

- **Route:** `/flows/lambdas`
- **Status:** Well-defined

**UI:** General card (Name, Description, Runtime select: Node.js 18/Python 3.11/Go 1.21) + Code Editor (12-row textarea with handler template, "Test" button) + Environment Variables (Key/Value pairs, Add/Remove)

**Actions:** Save, Test, Add/Remove environment variables

**Implied Functionality:** Serverless function execution for custom call handling logic, webhook processing, data transformation

---

### 4.13 API Logs Page

- **Route:** `/flows/api-logs`
- **Status:** Well-defined

**UI:** Filter bar (date range, status filter: All/Success/Error, endpoint search) + Data table with pagination

**Table Columns:** Timestamp, Method (badge: GET/POST/PUT/DELETE), Endpoint, Status Code, Duration, Response Size, Actions (Retry/View)

**Data Model -- `ApiLogEntry`:** timestamp, method, endpoint, status_code (u16), duration, response_size

**Actions:** Refresh, filter by date/status/endpoint, Retry/View per row, pagination

**Implied Functionality:** API call logging with retry capability

---

### 4.14 Global Page

- **Route:** `/flows/global`
- **Status:** Well-defined

**UI:** Two cards:
1. **Account Variables:** table (Name, Value, Description) with Add/Edit/Remove
2. **Webhooks:** table (URL, Events, Status) with Add/Edit/Remove

**Actions:** Save, Add/Edit/Remove variables and webhooks

**Implied Functionality:** Global configuration variables and webhook endpoint management across the entire account

---

### 4.15 Webhooks Page

- **Route:** `/flows/webhooks`
- **Status:** Well-defined

**Table Columns:** Name, URL, Events (count), Status (Active/Paused), Last Triggered, Updated, Created, Actions (Edit/Test/Remove)

**Data Model -- `Webhook`:** name, url, events (u32), status, last_triggered, updated, created

**Actions:** "New Webhook" button, Edit/Test/Remove, search, pagination

**Implied Functionality:** Webhook endpoint management with event subscription, test/ping capability, status tracking

---

## Engagement

### 4.16 Bulk Messages Page

- **Route:** `/flows/bulk-messages`
- **Status:** Well-defined

**UI:** Status filter tabs (All, Sending, Pending, Completed, Cancelled, Failed) + Data table with pagination

**Table Columns:** Name, Type, Recipients (count), Sent/Total, Status (badge), Scheduled, Updated, Created, Actions (View/Pause/Remove)

**Data Model -- `BulkMessage`:** name, msg_type, recipients (u32), sent (u32), total (u32), status, scheduled, updated, created

**Actions:** "New Campaign" button, status tab filtering, View/Pause/Remove, search, pagination

**Implied Functionality:** Bulk SMS/MMS campaign management with scheduling, progress tracking, status lifecycle

---

### 4.17 LeadReactor Page

- **Route:** `/flows/lead-reactor`
- **Status:** Well-defined

**UI:** General card (Name, Description) + Trigger card (Event select: New Lead/Form Submission/Missed Call, Delay input in minutes) + Actions card (Action Type select: Send SMS/Send Email/Create Task/Assign Agent, Template select, Add/Remove) + Settings card ("Active" toggle, "Working hours only" toggle, Max retries input)

**Actions:** Save, Add/Remove Actions, toggle active/working hours

**Implied Functionality:** Automated lead follow-up workflows triggered by events, with configurable action sequences and retry logic

---

### 4.18 Smart Dialers Page

- **Route:** `/flows/smart-dialers`
- **Status:** Well-defined

**UI:** General card (Name, Description) + Dialer Settings (Mode: Preview/Progressive/Predictive, Max Concurrent, Ring Timeout, Retry Attempts, Retry Interval) + Caller ID card (Outbound Number, CNAM) + Schedule card (Start Time, End Time, Timezone, Active Days checkboxes Mon-Sun)

**Actions:** Save, configure dialer parameters, set schedule

**Implied Functionality:** Outbound dialer campaigns with multiple dialing modes, caller ID management, timezone-aware scheduling

---

### 4.19 FormReactor Page

- **Route:** `/flows/form-reactor`
- **Status:** Well-defined

**Table Columns:** Name, Form, Action, Delay, Status, Updated, Created, Actions (Edit/Remove)

**Data Model -- `FormReactorEntry`:** name, form, action, delay, status, updated, created

**Actions:** "New Form Reactor" button, Edit/Remove, search, pagination

**Implied Functionality:** Automated actions triggered by form submissions (auto-call, send SMS on web form submit)

---

### 4.20 Chat Widget Page

- **Route:** `/flows/chat-widget`
- **Status:** STUB ONLY

**UI:** Header with "New Widget" button, empty data table with headers only (Name, Website, Status, Conversations, Updated, Created, Actions)

> **NEEDS DEFINITION:** Data model, widget configuration form, customization options (colors, position, welcome message), embed code generation

---

### 4.21 Flows ChatAI Page

- **Route:** `/flows/chat-ai`
- **Status:** Well-defined

**UI:** General card (Name, Description) + Knowledge Base select (General Knowledge/Product FAQ/Support Docs) + Behavior card (Instructions textarea, Max Turns input, Handoff Threshold: Low/Medium/High) + Integration card ("Connect to CRM" toggle, CRM select)

**Actions:** Save, configure knowledge base, set behavior, toggle CRM integration

**Implied Functionality:** AI chatbot configuration within the Flows automation context

---

### 4.22 Dialogflow Page

- **Route:** `/flows/dialogflow`
- **Status:** Well-defined

**UI:** Connection card (Project ID, Service Account JSON textarea, "Test Connection" button, connection status indicator) + Settings card (Language select: en-US/es/fr/de, Default Intent select, Fallback Message textarea)

**Actions:** Save, Test Connection

**Implied Functionality:** Google Dialogflow integration for NLU-powered voice/chat bots

---

### 4.23 Reminders Page

- **Route:** `/flows/reminders`
- **Status:** Well-defined

**UI:** Four form cards:
1. **General:** Name, Description, Timezone select
2. **Scheduling:** Remind-at datetime picker, "Recurring" checkbox
3. **Who To Invite:** Tab toggle (Recent Calls / Contact List), Contact select
4. **Getting Connected:** How to Remind (Call/Text/Email), Who to Remind input, Message textarea

**Actions:** Configure reminder, set scheduling, select contact, choose delivery method

**Implied Functionality:** Appointment/follow-up reminder system with multi-channel delivery, recurring support, contact integration
