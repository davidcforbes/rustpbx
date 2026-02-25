# 09 - Summary & Gaps

## Status Classification

### Well-Defined Pages (84 pages)

Complete UI layouts, data models, mock data, and clear implied functionality. Ready for backend integration.

| Section | Count | Pages |
|---------|-------|-------|
| Activities | 7 | Calls, Texts, Forms, Chats, Faxes, Videos, Export Log |
| Contacts | 4 | Contact Lists, Blocked Numbers, Do Not Call, Do Not Text |
| Numbers | 10 | Tracking Numbers, Buy Numbers, Receiving Numbers, Text Numbers, Port Numbers, Call Settings, Number Pools, Target Numbers, Tracking Sources, Tracking Code |
| Flows | 22 | Voice Menus, Queues, Smart Routers, Schedules, Geo Routers, Agent Scripts, Routing Tables, Voicemails, Workflows, Triggers, Keyword Spotting, Lambdas, API Logs, Global, Webhooks, Bulk Messages, LeadReactor, Smart Dialers, FormReactor, Flows ChatAI, Dialogflow, Reminders |
| AI Tools | 4 | AskAI, Summaries, VoiceAI, ChatAI |
| Reports | 29 | Activity Report, ROI, Accuracy, Overview, Today's Missed, Positive Daily, Google CA, Saturday Calls, Daily Calls, Weekly Missed, Priming, Missed Calls, Missed Daily 1st, CS Daily Missed, CS Daily Missed 2.0, Priming Missed, Daily Collection, Power BI, Real Time, Appointments, Real-Time Agents, Coaching, Queue Report, Agent Activity, Agency Usage, Custom Reports, Notifications, Scoring, Tags |
| Trust Center | 8 | Business Info, Local Text, Toll-Free Text, Voice Registration, Caller ID, Requirements, Applications, Addresses |

### Pages Needing Definition (1 page)

| Section | Count | Pages |
|---------|-------|-------|
| Reports | 1 | Activity Map (unique stub with map placeholder — needs map provider integration) |

### Stub-Only Pages (2 pages)

Table headers or layout skeleton only. Requires full design.

| Section | Page | What's Missing |
|---------|------|---------------|
| Flows | Chat Widget | Data model, config form, customization, embed code |
| AI Tools | Knowledge Banks | Upload flow, categories, import mechanism, content preview |

---

## Cross-Cutting Items Needing Definition

These items affect multiple pages and require architectural decisions:

### 1. Create/Edit Modal Flows
Every page with "New ..." or "Edit" buttons needs a creation/editing form. Some pages have inline forms (Geo Routers, Agent Scripts, Voicemails, etc.), but most list-view pages reference creation modals that don't exist yet.

**Affected pages:** ~30 pages with "New" buttons, ~40 pages with "Edit" row actions

**Decisions needed:**
- Modal dialog vs. dedicated page for creation?
- Inline editing vs. modal for row edits?
- Form validation rules per entity type
- Confirmation dialogs for destructive actions (Remove/Delete)

### 2. FilterBar Behavior
The shared FilterBar component has several undefined behaviors:

- **Filter panel contents:** What filter fields exist? (date range, status, source, agent, tags, etc.)
- **Desk Mode:** What changes in the UI when enabled?
- **Auto Load:** WebSocket streaming or polling? Interval?
- **Notification center:** What events generate notifications?
- **View modes:** What does each Person/Grid/Gear mode display?

### 3. CallDetailPanel Integrations
- **Zoho tab:** CRM data model, sync direction, field mapping
- **Script tab:** Script rendering, interactive steps, variable substitution display
- **Voice Analysis:** Audio player implementation, waveform rendering, real-time vs. post-call transcription

### 4. Workflows Canvas
The visual workflow builder needs significant design:
- Drag-and-drop interaction model
- Node types (events, conditions, actions)
- Connection/edge drawing between nodes
- Node configuration panels
- Execution preview/testing

### 5. Scoring Integration
- How scores are calculated and applied to calls
- Score thresholds / grade boundaries
- Where scores appear (call detail, reports, filters)

### 6. Real-Time Features
Several features imply real-time or near-real-time data:
- Calls page (Auto Load toggle)
- Real-Time Agents dashboard
- Coaching (Listen/Whisper/Barge)
- Queue Report (Calls Waiting)
- Real Time report

**Decision needed:** WebSocket vs. polling, refresh intervals, server-sent events

### 7. Multi-Account Architecture
The FilterBar shows "Main Account" selector, implying multi-account/agency support. This affects:
- Data isolation
- Permission model
- Account switching behavior
- Agency-level vs. account-level reports

---

## Recommended Next Steps

1. ~~**Define the 19 template reports**~~ -- DONE. All 19 now have unique KPI cards, charts, tables, and mock data.
2. **Design Activity Map** -- needs map provider integration (Google Maps / Mapbox / Leaflet), data overlay, and drill-down.
3. **Design Chat Widget and Knowledge Banks** -- the 2 stub pages need full design.
4. **Decide on create/edit pattern** -- modal vs. page for entity creation/editing.
5. **Specify real-time architecture** -- WebSocket/polling approach for live features.
6. **Design the Workflows canvas** -- the most complex single feature needing design.
