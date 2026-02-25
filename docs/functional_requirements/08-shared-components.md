# 08 - Shared Components

> 2 components used across multiple pages
> Source: `ui/src/components/`

---

## FilterBar

- **File:** `ui/src/components/filter_bar.rs`
- **Used By:** CallsPage, TextsPage, ChatsPage, FaxesPage, VideosPage

### UI Elements
| Element | Description |
|---------|-------------|
| Back button | Left arrow navigation |
| Filter button | Funnel icon, opens filter panel |
| Search input | Magnifying glass icon, text search |
| Active filters badge | "22 active" count indicator |
| Record count | e.g., "3,694,942 calls" |
| Auto Load toggle | Auto-refresh / real-time streaming |
| Desk Mode toggle | Simplified view for phone agents |
| Notification bell | Count badge ("4950") |
| View mode toggle | Person / Grid / Gear icons |
| Phone button | Green, right side -- softphone/dialer |
| Account info | Company name ("Diener Law") + Account selector ("Main Account") |

### Implied Functionality
- Saved filter presets with active count display
- Auto-load: real-time streaming or periodic auto-refresh toggle
- Desk mode: simplified agent-focused view
- Notification center with unread count
- Multiple view modes (list, grid, settings)
- Softphone/dialer quick access button
- Multi-account switching

> **NEEDS DEFINITION:**
> - Filter panel contents (what filter fields exist? date range, status, source, agent, tags, etc.)
> - "22 active" filters -- what constitutes an "active filter"?
> - Desk Mode behavior -- what changes in the view?
> - Auto Load -- WebSocket streaming or polling interval?
> - Notification types -- what events generate notifications?
> - View modes -- what does each mode display?

---

## CallDetailPanel

- **File:** `ui/src/components/detail_panel.rs`
- **Used By:** CallsPage (triggered on row click)
- **Architecture:** Slide-out panel (900px wide) with backdrop overlay, controlled by `RwSignal<Option<CallRecord>>`

### Header (`DetailHeader`)
Mirrors call row data in compact format with Close (X) button, Email button, Flag button

### 10 Tabs

#### Tab 1: Text Messages
- Chat bubble UI (outbound = cyan right-aligned, inbound = dark left-aligned)
- Reply input field with send button
- Message count display
- Mock: 3 messages

#### Tab 2: Contact
- 10-field form: First Name, Last Name, Email, Phone, Street, City, State, Country, Category (select: New/Existing/VIP/Lead), Outcome (select: Converted/In Progress/Lost/No Response)
- 3 toggle switches: Appointment Set, Answered, NPS
- Notes section with textarea

#### Tab 3: Visitor Details
- Read-only data grid (10 fields): IP Address, Device, Browser, OS, Referrer, Landing Page, Keywords, Campaign, Visit Duration, Pages Viewed

#### Tab 4: Score
- Reporting Tag input
- Score select (1-5 scale)
- "Converted" toggle
- Save / Remove buttons

#### Tab 5: Email
- Compose form: To, CC, BCC, Subject, Message textarea
- "Include call record details" checkbox
- Send button

#### Tab 6: Voice Analysis
- Audio player: waveform placeholder, play/pause, time display, speed selector (1x/1.5x/2x)
- Action links: Download, Share, Delete
- Transcription section: 3 entries (timestamp + speaker + text)

#### Tab 7: Flow
- Call flow timeline with 13 events as vertical timeline:
  - Call Received, IVR Start, DTMF Input 1, Queue Enter, Agent Ring, Agent Answer, Hold Start, Hold End, Transfer Initiated, Transfer Connected, Call Wrap-up, Tags Applied, Call Complete
  - Each event: colored dot + timestamp + event name

#### Tab 8: Reminder
- Remind via (Email/SMS select)
- Date/Time input, Timezone select
- Message textarea with template variables (`{{caller_name}}`, `{{agent_name}}`, `{{date}}`)
- "Set Reminder" button

#### Tab 9: Zoho (Placeholder)
- Integration icon + "Zoho CRM integration details will appear here"

> **NEEDS DEFINITION:** Zoho CRM data model, sync direction, field mapping

#### Tab 10: Script (Placeholder)
- Script icon + "Agent script will appear here"

> **NEEDS DEFINITION:** Script rendering, variable substitution display, interactive script steps
