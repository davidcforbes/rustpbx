# 4iiz Flows Section - UI Analysis & Prototype Plan

## Section Overview

The **Flows** section is the automation/routing engine of 4iiz. It is the second major top-level nav item (after Activities). The sidebar is organized into 3 subsections with approximately 20 pages total:

- **ROUTING** (8 pages): Voice Menus, Queues, Smart Routers, Geo Routers, Schedules, Agent Scripts, Routing Tables, Voicemails
- **AUTOMATION** (7 pages): Workflows, Triggers, Keyword Spotting, Lambdas, API Logs, Global, Webhooks
- **ENGAGEMENT** (7 pages): Bulk Messages, LeadReactor, Smart Dialers, FormReactor, Chat Widget, ChatAI (BETA), Dialogflow (BETA), Reminders

---

## Page Inventory (22 screenshots)

### ROUTING Section

| # | Page | Screenshot | Type | Records |
|---|------|-----------|------|---------|
| 1 | Voice Menus | `Flows - Voice Menus.jpg` | List + info banner | 7 Voice Menus |
| 2 | Queues | `Flows - Queues.jpg` | List table | 11 Queues (2 pages) |
| 3 | Smart Routers | `Flows - Smart Routers.jpg` | List with rule preview | 5 Smart Routers |
| 4 | Geo Routers | `Flows - Geo Routers.jpg` | New/Edit form | Form page |
| 5 | Schedules | `Flows - Schedules.jpg` | List table | 1 Schedule |
| 6 | Agent Scripts | `Flows - Agent Scripts.jpg` | New/Edit form | Form page |
| 7 | Routing Tables | `Flows - Routing Tables.jpg` | New/Edit form | Form page |
| 8 | Voicemails | `Flows - Voicemails.jpg` | New/Edit form | Form page |

### AUTOMATION Section

| # | Page | Screenshot | Type | Records |
|---|------|-----------|------|---------|
| 9 | Workflows - Events | `Flows - Automation - Workflows - Events.jpg` | Visual flow builder (zoomed out) | Complex workflow |
| 10 | Workflows - Activity Received | `Flows - Automation - Workflows - Activity Received.jpg` | Visual flow builder (zoomed in) | 3 triggers + conditions |
| 11 | Triggers | `Flows - Automation - Triggers.jpg` | List table | 26 Triggers (3 pages) |
| 12 | Keyword Spotting | `Flows - Automation - Key Word Spotting.jpg` | New/Edit form | Form page |
| 13 | Webhooks | `Flows - Automation - Webhooks.jpg` | List table | 24 Webhooks (3 pages) |
| 14 | API Logs | `Flows - Automation - API Logs.jpg` | Log viewer | Scrolling log |

### ENGAGEMENT Section

| # | Page | Screenshot | Type | Records |
|---|------|-----------|------|---------|
| 15 | Bulk Messages | `Flows - Engagement - Bulk Messages.jpg` | List table | 1695 messages (170 pages) |
| 16 | Bulk Messages Detail | `Flows - Engagement - Bulk Messages - Detail.jpg` | Detail dashboard | Stats + undelivered table |
| 17 | FormReactor | `Flows - Engagement - FormReactor.jpg` | List table | 30 FormReactors (3 pages) |
| 18 | Chat Widget | `Flows - Engagement - Chat Widget.jpg` | Empty list | 0 widgets |
| 19 | ChatAI | `Flows - Engagement - ChatAI.jpg` | New/Edit form | Form page (BETA) |
| 20 | Dialogflow | `Flows - Engagement - DialogFlow.jpg` | New/Edit form | Form page (BETA) |
| 21 | Reminders | `Flows - Engagement - Reminders.jpg` | New/Edit form | Form page |

Plus 10 MHTML files for: Call scripts, Chat bots, Geo routes, Keyword spots, Lambda actions, Lead reactors, New Dialogflow Agent, Reminders, Schedules, Smart Routers Detail.

---

## Detailed Page Analysis

### ROUTING Pages

#### Page 1: Voice Menus (IVR)

**Purpose**: Interactive Voice Response menu builder. Route callers via keypress or speech recognition.

**Layout**: Info banner + list table.

- **Info banner** (blue info icon): Explains IVR functionality with help links (Creating Voice Menus, Voice Menu Overview, Creating a Preset, Close, More Info, Walkthrough Video)
- **Table columns**: Name (with pencil edit icon), Greeting (play button + audio icon), Tag this call, Speech Recognition, Speech Language, Updated, Created
- **Sample data**: 7 menus including CM Agencies VM, Voicemail Language Selection, English/Spanish Voicemail, English/Spanish IVR, Initial Language Selection
- **CTA**: "New Voice Menu" (cyan), Restore
- **Per page**: 10 selector at bottom

#### Page 2: Queues (Call Queues)

**Purpose**: Hold callers in queue until next agent is available. ACD (Automatic Call Distribution).

**Layout**: Complex list table with many columns.

- **Table columns**: Name, Repeat callers, Distribute to agents, Prompt Agent, Caller ID, Post Call, Schedule, Agents (bulleted list), No Answer, Updated, Created
- **Key data points per row**:
  - Distribution: "Next available agents" + "Simultaneously" icon
  - Prompt Agent: checkmark when enabled
  - Caller ID: "Use Caller Number"
  - Schedule: link to "Jalisco" schedule (cyan)
  - Agents: bulleted list of agent names with "X more" overflow + "edit agent routing rules" link
  - No Answer: "Hangup" or voice menu reference ("Voicemail Language Selection" with "Voice menu" sublabel)
- **Sample data**: Collections FKM/RMs, SYSTANGO TESTING, PRIMING CALLS ONLY, Customer Service Queue (Official), Sales
- **CTA**: "New Queue" (cyan), Restore, Info
- **Pagination**: 2 pages, 11 Queues

#### Page 3: Smart Routers (Conditional Routing)

**Purpose**: Route callers based on contact properties using if/then/else rules.

**Layout**: List with expanded rule previews.

- **Table columns**: Name, Routes (rule preview), Updated, Created
- **Routes column** shows full rule logic inline:
  - "If **all** of the following rules match: Contact Category *includes any* ..."
  - "**Or** if all of the following rules match: ..."
  - "**Then** perform the following actions: CallQueue, Tag Call"
- **Sample routers**: "Current Client or Priming?", "Priming Routing", "Check if New Lead or Current Client"
- **CTA**: "New Smart Router" (cyan), Restore, Info
- **5 Smart Routers total**

#### Page 4: Geo Routers (Geographic Routing)

**Purpose**: Route calls based on caller's geographic location.

**Layout**: New/Edit form with multiple sections.

- **General section**: Name, Description (optional), Save Changes button
- **Prompts section**:
  - Options dropdown: "Silently route the call to the default action"
  - Explanation of why calls may not be geo-routed (no geographic data, outside service area)
  - "How should we handle a caller not within a service area?" dropdown: "Hang Up"
  - "How would you like calls routed when multiple receiving numbers matched?" dropdown: "Simultaneous (Ring All)"
  - Routing mode explanations: Simultaneous, Round Robin, Sequential, Route to
- **Routing section**: "How will you be routing?" dropdown: "Zip Code"

#### Page 5: Schedules

**Purpose**: Define recurring active times for agents, voice menus, call queues.

**Layout**: Simple list table.

- **Table columns**: Name, Times, (day columns), Time Zone, Updated, Created
- **Sample data**: 1 schedule "Jalisco" (PV/GDL Agents):
  - M-F: 07:55 AM - 10:00 PM
  - Sa: 07:55 AM - 09:00 PM
  - S: 09:00 AM - 09:00 PM
  - Time Zone: Eastern Time (US & Canada)
- **CTA**: "New Schedule" (cyan), Restore, Info

#### Page 6: Agent Scripts

**Purpose**: Rich text scripts displayed to agents during calls.

**Layout**: New/Edit form with breadcrumb "Agent Scripts > New > General"

- **General section**: Name (with red required icon), Description (optional), Advanced scripting options (collapsed)
- **Contents section**: Rich text editor with toolbar (B, I, U, Link, Color, Font color, List) + Code/Input Fields/Output Fields tabs
  - Helper text explains: "Script Markup. Add script to show your agents when an activity is connected."
  - **Input Fields**: Allow agents to input field data inline, saving to activity record
  - **Output Fields**: Allow agent to see field data from activity record inline
  - **ctm-module tag**: Embed a module by ID via data-ref-id attribute
- **Workflow section** (partially visible): "When a user completes a call script which panel should we load next"

#### Page 7: Routing Tables

**Purpose**: Static lookup-based routing.

**Layout**: New/Edit form with breadcrumb "Routing Tables > New > General"

- **General section**: Name, Description (optional), Default Route dropdown ("A default route if no matches found in your table")
  - Helper: "If no matches are found in your table - contacts will be routed here."
  - "Save changes to add mappings"

#### Page 8: Voicemails

**Purpose**: Voicemail box configuration.

**Layout**: New/Edit form with breadcrumb "Voicemail > New > General"

- **What's New banner**: Notifications now support multiple recipients, voicemails can be routed to tracking numbers/voice menus/queues/smart routers/geo routers/routing tables
- **General section**:
  - Name (with red required icon)
  - Tag this call: tag input with "voicemail" pre-filled
  - Greeting: TTS input "Please leave a message after the beep" with language (English) and voice (Awesome) selectors, plus audio icons (globe, microphone, play)
  - Email toggle: ON - "An email with your voicemail will be sent"
  - Transcribe toggle: OFF - "A text transcription of your voicemail will be captured"
- **Notifications section**: User Emails (select), User Groups (select) - "Email all of the users when a voicemail is left"

---

### AUTOMATION Pages

#### Pages 9-10: Workflows (Visual Flow Builder)

**Purpose**: Visual drag-and-drop workflow editor for event-driven automation.

**Layout**: Full-screen flow builder with left event panel + canvas.

- **Left panel - Workflow Events** with categories:
  - ACTIVITY EVENTS: Activity is received (3), Activity is sent (1), From another Workflow, Agent is assigned (1), Activity is transferred, Session data is available, Caller hangs up the call
  - COMPLETION EVENTS: End event immediately, End event with all... (19), 1 hour after end..., Contact panel is u... (1)
  - CONVERSIONS EVENTS: Sale is updated for an..., PPC data is retrieved f..., Cost data is retrieved f...
  - OTHER EVENTS: FormReactor activity l..., Caller inputs their zip c..., Keyword is spotted, Audio accessed, New agent is assig... (1), Transcription is ready, Task is completed
- **Canvas**: Node-based flow diagram with:
  - **Start node** (dark teal): "Activity is received" / "End event with all data ready" / "+ Click to add workflow"
  - **Trigger nodes** (gray): "Email agent when tiktok text is received", "Contact Responded to Your Text", "New Caller SMS Trigger"
  - **Condition nodes** (teal/green): "If condition" with rules like "If Type is any Inbound Text AND Tracking Number includes any"
  - **Action nodes** (gray): "Send Email" (with To/Subject details), "Send Text" (with number template + message)
  - Nodes connected by lines with (+) add buttons between them
- **Zoomed out view** (Events screenshot): Shows massive workflow with approximately 20+ parallel branches, each with condition, tag call, and run webhook pattern
- **Top bar**: "Triggers >" breadcrumb, Revisions button, Feedback button, Account Triggers link, API Logs link

#### Page 11: Triggers

**Purpose**: Event-driven rules that fire actions on activities.

**Layout**: List table with rule preview.

- **Table columns**: Name, Trigger, Run, Rules (full logic preview), Runs In The Last 7 Days, Updated, Created
- **Trigger types**: "End event with all data ready end", "New agent is assigned agent_change", "Through a trigger [route]"
- **Run**: "All Tracking Numbers"
- **Rules column** shows full logic (like Smart Routers but more detailed):
  - Complex if/or/then chains with conditions on Agent, Tags, Status, Contact Category, etc.
  - Actions: Tag Call, Run Webhook, Transcribe, Send Email, Add Contact to List
- **Sample triggers**: FKM Tagging - Outbound (1689 runs), #NA & Missed Calls Tickets (1074), AnswerHero Tickets (474), Force Transcription AH (474), Repeated Callers (1429), Assigned Agent (23274)
- **CTA**: "New Trigger" (cyan), Restore, Visual Workflows, API Logs, Info
- **Pagination**: 3 pages, 26 Triggers

#### Page 12: Keyword Spotting

**Purpose**: Monitor call transcriptions for specific keywords and trigger actions.

**Layout**: New/Edit form with breadcrumb "Keyword Spotting > New > General"

- **General section**: Name, Description (optional), Save Changes + Copy Section buttons
- **Workflow section**: "perform actions in response to keywords found in transcriptions"
  - "If **any** of the following keywords are found: **No keywords added.**"
  - "+ Add Keyword" button
  - "**Then** perform the following actions: **No actions added.**"
  - "Add Action" button
  - NOTE: "you can use triggers after keyword spotting to do many more advanced actions such as Ask AI"
- **Spot Activity Types section**: Toggles for which activity types to monitor
  - Calls: ON
  - Chats: OFF

#### Page 13: Webhooks

**Purpose**: Send HTTP callbacks to external servers on events.

**Layout**: List table with detailed columns.

- **Table columns**: Name, Trigger, Callback URL, Method, Body Type, Updated, Created
- **Trigger types**:
  - "After a text message has been sent [outbound_text]"
  - "After a text message has been received [inbound_text]"
  - "At the end of a call/form/chat, once all data has been captured [end]"
  - "Through a trigger [route]"
- **Callback URLs**: Zapier hooks, AWS Lambda, ngrok, Zoho sync endpoints, internal staging APIs
- **Method**: All POST
- **Body Type**: All "Log Data"
- **Row actions**: Edit link + "Test" button (cyan) + "De..." (deactivate, truncated)
- **CTA**: "New Webhook" (cyan), Restore, Global Webhooks, API Logs, Info, "+ Start"
- **Pagination**: 3 pages, 24 Webhooks

#### Page 14: API Logs

**Purpose**: View webhook execution logs with response codes.

**Layout**: Log viewer with filters.

- **Breadcrumb**: "Settings > Integrations > Webhooks > API Logs"
- **Filters**: Status Code (dropdown), Source (text), Select Call (dropdown), Search button (cyan), Reset button
- **Table columns**: (expand icon), Source, Request URL, Response Code, Date, Activity
- **Response codes shown**: 404 (most common), 200, 200 OK, 201 Created
- **Each row**: + expand icon, "Retry" button (cyan outline), Source "WebHook", full URL, response badge, UTC timestamp, "Log JSON" links
- **"Load More"** at bottom (no pagination -- infinite scroll)

---

### ENGAGEMENT Pages

#### Page 15: Bulk Messages

**Purpose**: Send mass SMS to groups of recipients.

**Layout**: Complex list table with status filters.

- **Warning banner** (red): "You have Bulk Messages that need attention. To ensure delivery, please deactivate the existing messages and recreate them."
- **Status filter tabs**: All | Sending | Pending | Completed | Cancelled | Failed | Export...
- **Table columns**: Recurring, Label, Virtual Phone Numbers, Body, Recipients, Send Time, Delivered On, Next Delivery, Status, Timezone Controls, Updated, Created
- **Sample data**: Messages in Spanish (appointment confirmations, marketing with YouTube links)
  - Recipients: 9-407 range
  - Status: "Completed" with x cancel icon
  - Created by: agents like "Maria Patino", "Christian Michelle Vazquez Ruvalcaba"
- **CTA**: "New Bulk Message" (cyan), Info
- **Pagination**: 170 pages, 1695 Bulk Messages

#### Page 16: Bulk Messages Detail

**Purpose**: View delivery stats for a specific bulk message campaign.

**Layout**: Dashboard-style detail view with stats cards.

- **Breadcrumb**: "Bulk Messages > [message label] > View"
- **Stats cards** (5 across): Total (61), Remaining (0), Sent (0), Delivered (60), Undelivered (1, red)
- **Info cards**: Message (full body), Description, Numbers (sent from 1 number with tracking link)
- **Timestamps**: First/Last Message Sent times
- **Timezone Controls**: "This bulk message does not have a timezone restriction"
- **Schedule**: Expiration time
- **Undelivered Messages table**: From Number, Recipient Number, Error Type ("Blocked"), Error Description ("This number has been added to the Do Not Text list" with link)
- **Export button** (cyan) for undelivered messages

#### Page 17: FormReactor

**Purpose**: Embeddable web forms that trigger phone calls, text messages, and emails.

**Layout**: List table with field preview.

- **Table columns**: Form Name, Fields (preview), Tracking Number, Updated, Created, Calls (count)
- **Fields column** shows form structure: "Name *, Phone *, Email *, Submit" with required field indicators
- **Form types**: "Gravity Forms" noted under each
- **Sample forms**: Permanent Residence Form (7 calls), Book - Spanish (5), Questions (335/552), Immigration Questionnaire (120), Footer Contact (747), Immigration Form Disclaimer (166/116), Book - English (34)
- **CTA**: "New FormReactor" (cyan), Restore, Info
- **Pagination**: 3 pages, 30 FormReactors

#### Page 18: Chat Widget

**Purpose**: Embeddable live chat widget for websites.

**Layout**: Empty list table.

- **Table columns**: Chat Widget, Fields, Active, Status, Tracking, Routing, Queue, Agents, Licenses, Updated, Created, Chats
- **No data** -- empty state
- **CTA**: "New Chat Widget" (cyan), "User licenses" button, Info

#### Page 19: ChatAI (BETA)

**Purpose**: AI-powered chat agent for customer interactions.

**Layout**: New/Edit form with breadcrumb "ChatAI's > New > General"

- **General section**: Name (with note: "user facing, pick a customer facing name"), Description (optional)
- **Knowledge Banks section**: "A set of document collections where the AI can query to assist in answering questions"
  - Document source: "Choose a Knowledge Bank" dropdown
  - Include Source toggle: ON - "When the bot uses a document to answer a prompt, it will also send a link"
  - "Manage Knowledge Banks" button
- **Instructions section**: "Provide instructions to the AI teaching it how to interact with your customers"
  - Large textarea for instructions

#### Page 20: Dialogflow (BETA)

**Purpose**: Google Dialogflow CX integration for conversational AI.

**Layout**: New/Edit form with breadcrumb "Dialogflow > New Dialogflow Agent > Pre-requisites"

- **Pre-requisites section**: Numbered setup steps:
  1. Set up a Google Cloud Project
  2. Set up your DialogFlow CX agent
  3. Create your Conversation Profile (must complete #1 and #2 first, enable Dialogflow, only one CX agent per profile)
  4. Additional config for Voice support
  5. Additional config for Chat support
- **General section**: Name (unique, must not match another Dialogflow agent), Description
- **Google Cloud Configuration section**:
  - Project ID (from GCP console)
  - Conversation Profile ID (Integration ID from agentassist.cloud.google.com)
  - Agent Location dropdown: "Select a location"

#### Page 21: Reminders

**Purpose**: Schedule callback reminders to contacts.

**Layout**: New/Edit form with breadcrumb "Reminders > New"

- **Top bar actions**: Reminder Settings, My Reminders
- **General section**: Name, Description (optional), Timezone (set to UTC, dropdown: "(GMT-05:00) Eastern Time (US & Canada)")
- **Scheduling section**: "Set the time and recurrence of the reminder"
  - Remind at: Start At input
  - Recurring: checkbox
- **Who To Invite section**: "Select a contact to contact from"
  - Toggle buttons: "Recent Calls" (active, cyan) | "Contact List"
  - "Choose a Contact" dropdown
- **Getting Connected section**:
  - How to Remind: "Select Contact Method" dropdown
  - Who to Remind: text input
  - Message: text input

---

## Shared UI Patterns

### Pattern: List Pages (7 pages)

Voice Menus, Queues, Smart Routers, Schedules, Triggers, Webhooks, FormReactor, Bulk Messages all follow:

1. Title + subtitle + CTA button
2. Search bar + filter + pagination
3. Sortable column headers
4. Edit link per row
5. Restore/Info secondary actions

### Pattern: New/Edit Form Pages (8 pages)

Geo Routers, Agent Scripts, Routing Tables, Voicemails, Keyword Spotting, ChatAI, Dialogflow, Reminders all follow:

1. Breadcrumb: "[Section] > New > General"
2. Multiple card sections (General, config-specific, Workflow)
3. Name + Description (optional) in General section
4. "Save Changes" button per section
5. Info link in top-right

### Pattern: Visual Flow Builder (Workflows)

Unique canvas-based UI with:

1. Left panel with categorized event types
2. Draggable node canvas
3. Trigger -> Condition -> Action node chain
4. Revisions history

### Pattern: Dashboard Detail (Bulk Messages Detail)

Stats cards + info cards + data table for drill-down.

---

## Key Observations for Production Implementation

1. **Workflow builder is the most complex UI** -- requires a node-based canvas editor (consider libraries like ReactFlow/SvelteFlow for Leptos equivalent, or a custom SVG/Canvas solution)

2. **Rule/condition engine is pervasive** -- Smart Routers, Triggers, and Workflows all use the same if/or/then rule syntax. This suggests a shared condition builder component.

3. **Many pages are "New" forms** -- Geo Routers, Agent Scripts, Routing Tables, Voicemails, Keyword Spotting, ChatAI, Dialogflow, Reminders all show empty "New" forms rather than populated list views, suggesting these features have low usage or are newer additions.

4. **Webhook ecosystem is central** -- Zapier, AWS Lambda, Zoho, and custom endpoints are all connected. The webhook + trigger system is the primary integration mechanism.

5. **Bulk Messages is high-volume** -- 1,695 campaigns with 170 pages shows this is a heavily used feature. The detail view with delivery stats, undelivered tracking, and DNT list integration is critical.

6. **AI features are BETA** -- ChatAI and Dialogflow are marked BETA, suggesting they are newer additions. ChatAI has knowledge bank + instruction-based configuration (similar to modern AI agent patterns).

7. **Agent Scripts has a rich text editor** -- with Input/Output Fields and ctm-module embedding, this is essentially a mini CMS for agent-facing content during calls.

8. **Schedules are timezone-aware** -- With different hours for weekdays vs weekends and Eastern Time zone support, schedule management needs robust timezone handling.

---

## Component Mapping to DaisyUI

| 4iiz Component | DaisyUI Equivalent |
|----------------|-------------------|
| Info banner (Voice Menus) | `alert alert-info` with icon |
| Help links | `btn btn-outline btn-xs` group |
| Rule preview (Smart Routers) | Formatted text block with `font-bold` keywords |
| Flow builder canvas | Custom SVG/Canvas (no DaisyUI equivalent) |
| Event panel | Sidebar with collapsible sections |
| Stats cards | `stats` component or card grid |
| Status filter tabs | `tabs tabs-bordered` |
| Toggle switches | `toggle` component |
| Tag inputs | `badge` with x close button |
| Rich text editor | Third-party (Tiptap/Quill equivalent) |
| TTS greeting | `input` with language/voice `select` dropdowns |

---

## Prototype Coverage

Given the complexity (22 pages), the prototype focuses on the **list view pages** which share common patterns:

- Voice Menus, Queues, Smart Routers, Schedules (ROUTING)
- Triggers, Webhooks (AUTOMATION)
- Bulk Messages, FormReactor (ENGAGEMENT)

Form pages and the workflow builder are documented for later implementation.
