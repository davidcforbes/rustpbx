# 4iiz UI Analysis & Figma Design Action Plan

## Application Overview

4iiz is a call center / lead management SaaS platform used by law firms. It tracks inbound/outbound calls, texts, forms, chats, faxes, and videos — tying each activity to a contact record with CRM integration (Zoho), Google Analytics visitor tracking, automation flows, voice analysis/transcription, scoring, and agent routing.

---

## 1. Global Architecture

### 1.1 Primary Navigation (Left Icon Bar — persistent across all sections)

| Icon | Section | Description |
|------|---------|-------------|
| Phone | **Activities** | Call/text/form/chat logs + contacts (current screenshots) |
| Grid | **Numbers** | Phone number management (not captured yet) |
| Routes | **Flows** | Automation/workflow builder (not captured yet) |
| Sparkle | **AI Tools** | AI-powered features (not captured yet) |
| Chart | **Reports** | Analytics/reporting (not captured yet) |
| Shield | **Trust Center** | Compliance/trust settings (not captured yet) |
| ? | **Help** | Help/support (bottom) |
| Gear | **Settings** | Account settings (bottom) |

### 1.2 Top Bar (persistent)

- Back arrow + 4iiz logo
- Page title ("Activities")
- Filter button + Search bar + record count + Auto Load toggle
- Right cluster: Desk Mode toggle, notification bell (with badge count), view mode toggles (list/card/grid), settings gear, **Phone** button (green, prominent)
- Account: "Account ID 155169 - 4iiz" + user email + avatar initials

### 1.3 Secondary Navigation (Left Panel — context-dependent)

For **Activities** section:
- **ACTIVITY LOGS**: Calls, Texts, Forms, Chats, Faxes, Videos, Export Log
- **CONTACTS**: Lists, Blocked Numbers, Do Not Call List, Do Not Text List
- Collapsible with chevron toggle

---

## 2. Page-by-Page Analysis

### Page 1: Activities > Calls (Home Page)
**Screenshot**: `4iiz-home page.jpg`

**Layout**: Master list view with sortable columns
- **Columns**: Contact, Source, Session Data, Score, Audio, Metrics, Routing
- **Each row contains**:
  - Contact: Name, phone, location, CRM IDs, case subtype, matters stage, tags
  - Action buttons: Call (blue), Listen (orange speaker), Edit (blue person)
  - Green phone icon = active/recent call indicator
  - Source: tracking source name + number + business name
  - Session Data: bar chart icon (clickable for analytics)
  - Score: score icon (blue)
  - Audio: status (audio/no audio) + duration + play status
  - Metrics: Date, time, status (Answered/in progress)
  - Routing: Agent avatar (initials) + name + automation label
  - Actions: Email icon, Flag icon

**Key interactions**:
- Row click opens call detail slide-out panel
- Filter bar at top with active filter count badge
- "3,694,942 calls" count display
- Auto Load with spinner

**Figma design**: 1 frame (desktop 1440px or 1920px width)

---

### Page 2: Call Detail > Text Message Tab
**Screenshot**: `4iiz-home page - call details - Text Messages.jpg`

**Layout**: Detail slide-out panel with left tab nav + chat thread
- **Header bar**: Contact info (name, phone, location, CRM IDs, case info, tags) | Source | Score | Audio player | Metrics | Agent | Email + Flag actions
- **Left tab nav**: Text Message (active), Contact, Visitor Detail, Score, Email, Voice Analysis, Flow, Reminder, Zoho, Script
- **Content**: Chat-style message thread
  - "load more" at top
  - Messages in cyan/teal bubbles (outbound) with timestamps
  - Pink/red bubbles for different message types
  - Lightning bolt icons on outbound messages (automation indicator)
  - Full conversation history with dates

**Figma design**: 1 frame (detail panel overlay)

---

### Page 3: Call Detail > Contact Tab
**Screenshot**: `4iiz-home page - call details - contact.jpg`

**Layout**: Form-based contact profile editor
- **Header**: "Contact caller profile" + Activity ID + close button
- **Form fields** (4-column grid):
  - Contact Name, Email, Contact Number (with country flag), Street
  - City, State (with red SMS indicator), Country, Postal Code
  - crm_contact_id, crm_matter_id, Contact Category (dropdown), Appointment Set? (dropdown)
  - Doc Drop Appointment? (toggle), Payment Appointment? (toggle), NPS? (toggle), WC Call? (toggle)
  - Answered? (toggle), Google Review (dropdown), RM Call Type, Social Selling New Lead (toggle)
  - Matters Stage, Next Stage Appointment Set? (toggle), Amount Collected, Next Stage Agent (dropdown)
  - Call Outcome (dropdown)
  - Tag Call (text input with placeholder)
- **Buttons**: Save Changes (cyan), Save & Close (outline)
- **Notes section** below: New Note (supports Markdown), Save Note, Save & Close, "load more notes"

**Figma design**: 1 frame (scrollable)

---

### Page 4: Call Detail > Visitor Detail Tab
**Screenshot**: `4iiz-home page - call details - Visitor Details.jpg`

**Layout**: Analytics/session data panel
- **Header**: "Visitor Detail user/visitor activity for this contact" + Activity ID + close
- **Content**: "No session data" (empty state)
- **Action buttons**: GA4 Activity Log (cyan), Check Analytics (GA3) (cyan)
- **Close button** at bottom right

**Figma design**: 1 frame (with both empty state and populated state variants)

---

### Page 5: Call Detail > Score Tab
**Screenshot**: `4iiz-home page - call details - score.jpg`

**Layout**: Simple scoring form
- **Header**: "Score call revenue / sales rating" + Activity ID + close
- **Fields**:
  - Reporting Tag: text input (Select tag)
  - Score Call: dropdown (No rating)
  - Converted: toggle (OFF)
- **Buttons**: Save Changes (cyan), Remove Score (outline, bottom right)

**Figma design**: 1 frame

---

### Page 6: Call Detail > Email Tab
**Screenshot**: `4iiz-home page - call details - Email.jpg`

**Layout**: Email compose form
- **Header**: "Email send an email of this call" + Activity ID + close
- **Fields**:
  - To: email input + "CC Email BCC Email" links
  - Subject: prefilled "Follow-up from our call"
  - Message: textarea
  - Checkbox: "Include Call Record" with helper text
- **Buttons**: Send Call (cyan), Close (outline)

**Figma design**: 1 frame

---

### Page 7: Call Detail > Voice Analysis Tab
**Screenshot**: `4iiz-home page - call details - Voice Analysis.jpg`

**Layout**: Audio player + transcription viewer
- **Header**: "Voice Analysis transcription of the call" + Activity ID + close
- **Access Logs button** (outline)
- **Audio player**:
  - Speed control (1.00x), play button, waveform visualization (cyan), timestamp (00:03), volume
  - Download dropdown + Share link
  - Delete button (right)
- **Transcription section**: "No transcription available" (empty state)

**Figma design**: 1 frame (with transcription populated variant)

---

### Page 8: Call Detail > Flow Tab
**Screenshot**: `4iiz-home page - call details - Flow.jpg`

**Layout**: Vertical timeline of automation events
- **Header**: "Flow key events of the activity" + Activity ID + close
- **Action buttons**: Debug Flow (cyan outline), Agent Logs (outline)
- **Timeline**: Vertical line with blue dot nodes
  - Each node: timestamp (linked) + event name
  - Events: Abandoned Call Notification, SMS for AHC, Zoho Integration2, Tag calls forwarded to Lex, Tag call if not answered, Enter Workflow, FKM Tagging - Outbound, Voicemail to Tickets, AnswerHero Tickets, Repeated Callers, Force Transcription AH, User accessed recording...
  - Final node: "answered" with chat bubble icon

**Figma design**: 1 frame (scrollable timeline)

---

### Page 9: Call Detail > Reminder Tab
**Screenshot**: `4iiz-home page - call details - Reminder.jpg`

**Layout**: Callback reminder scheduler
- **Header**: "Callback Reminder" + close
- **Fields**:
  - How to remind: dropdown (Email)
  - Remind at: datetime input (2026-02-24 11:48)
  - Timezone: dropdown (GMT-05:00 America/New_York)
  - Reminder message: textarea
  - Template variable dropdowns: Activity, Contact, Score, Enhanced
- **Buttons**: Save (cyan), Close (outline)

**Figma design**: 1 frame

---

### Page 10: Activities > Texts
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Texts.jpg`

**Layout**: Same master list structure as Calls but for text messages
- **Columns**: Contact, Message (text preview), Source, Score, Metrics, Routing
- **Header**: "1,410,190 texts" + "New Message" button (cyan, top right)
- **Each row**: Reply/Edit actions, contact info, message preview text, source tracking, score, date/time, status (Delivered/Received), agent assignment
- Messages include: appointment confirmations, legal communications, images (MMS), multi-language (Spanish/English)

**Figma design**: 1 frame

---

### Page 11: Activities > Forms
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Forms.jpg`

**Layout**: Same master list structure for form submissions
- **Columns**: Contact, Source (with form name), Session Data (with URL + completion %), Score, Audio, Metrics, Routing
- **Header**: "1,621 calls" count
- **Unique elements**:
  - Form name shown under source (e.g., "Immigration Questionnaire", "Immigration Form Disclaimer")
  - Session data shows website URL + referral source
  - Completion percentage bar (100%)
  - Pin/location icons for visitor data

**Figma design**: 1 frame

---

### Page 12: Activities > Chats
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Chatsjpg.jpg`

**Layout**: Same master list for chat interactions
- **Unique**: Chat icon (yellow speech bubble) instead of phone icon
- "Chat" action button instead of "Call"
- Source: "Woosender" platform
- Two-section table (appears to have inbound/outbound split)

**Figma design**: 1 frame

---

### Page 13: Activities > Faxes
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Faxes.jpg`

**Layout**: Same master list — empty state
- "No Results" centered
- **Unique**: "New Fax" button (dark, top right)
- Same column headers as other activity types

**Figma design**: 1 frame (empty state + populated variant)

---

### Page 14: Activities > Videos
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Videos.jpg`

**Layout**: Same master list — empty state
- "No Results" centered
- "Phone" button in top right instead of type-specific action

**Figma design**: 1 frame (empty state + populated variant)

---

### Page 15: Activities > Export Log
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Export Log.jpg`
**MHTML**: Full page archive available for detailed inspection

**Layout**: Multi-section export configuration form
- **Header**: "Export Calls calls, forms, texts > Export Delivery"
- **Section 1 — Export Delivery**:
  - Export method: dropdown (Email / FTP)
  - Email address: tag input (multiple emails)
  - Helper text about email delivery + 7-day expiration
- **Section 2 — Search**: keyword/phrase filter
- **Section 3 — Filters**:
  - Toggle: "Export tags in separate columns"
  - Date range: from/to + quick selectors (Today, Yesterday, Last 7 days, Last 30 days, Last Month, This Month, All Time)
  - Custom filter: "Not Set" with edit/clear
  - Tag includes: search input
  - Agent includes: search input
  - Type includes: chip selector (Inbound Call, Outbound Call, Form, Chat) with X to remove
  - Tracking Source includes: search input
  - Tracking Number includes: search input
  - (scrolls further — more filters likely below)

**Figma design**: 1 frame (scrollable, long form)

---

### Page 16: Left Navigation Close-up
**Screenshot**: `4iiz-home page - lefthand navbar - Activity Logs - Calls.jpg`

Shows the full nav hierarchy clearly. Already documented in Section 1.3.

---

## 3. Figma Design Plan

### Phase 1: Design System Foundation
Create shared components before individual pages:

| Component | Description | Priority |
|-----------|-------------|----------|
| Color tokens | Cyan/teal primary (#00BCD4~), white bg, light gray panels, blue links, red flags | P0 |
| Typography | System font stack, sizes for headers/body/labels/metadata | P0 |
| Icon library | Phone, email, chat, fax, video, score, audio, flag, filter, search, etc. | P0 |
| Global nav bar (left icons) | 8-icon vertical bar with active states | P0 |
| Top bar | Logo, search, filters, count, view toggles, Phone button, account | P0 |
| Secondary nav panel | Collapsible left panel with section headers + items | P0 |
| Activity row | Reusable row component with all column variants | P0 |
| Detail panel header | Contact info + source + score + audio + metrics + agent | P0 |
| Detail panel tab nav | 10-tab vertical nav with active state | P0 |
| Form controls | Input, dropdown, toggle, textarea, tag input, date picker | P0 |
| Buttons | Primary (cyan filled), Secondary (outline), Danger (red) | P0 |
| Avatar/initials | Circle with 2-letter initials + color variants | P0 |
| Chat bubble | Outbound (cyan) and inbound (white/pink) message styles | P1 |
| Audio player | Waveform + controls + speed + download | P1 |
| Timeline | Vertical dot-line timeline for Flow events | P1 |

### Phase 2: Page Designs (16 frames)

| # | Frame Name | Source Screenshot | Complexity | Est. Hours |
|---|-----------|-------------------|------------|-----------|
| 1 | Activities > Calls (Home) | `home page.jpg` | High — master data table | 3 |
| 2 | Call Detail > Text Message | `call details - Text Messages.jpg` | Medium — chat thread | 2 |
| 3 | Call Detail > Contact | `call details - contact.jpg` | High — dense form | 3 |
| 4 | Call Detail > Visitor Detail | `call details - Visitor Details.jpg` | Low — mostly empty state | 1 |
| 5 | Call Detail > Score | `call details - score.jpg` | Low — simple form | 0.5 |
| 6 | Call Detail > Email | `call details - Email.jpg` | Low — compose form | 1 |
| 7 | Call Detail > Voice Analysis | `call details - Voice Analysis.jpg` | Medium — audio player | 2 |
| 8 | Call Detail > Flow | `call details - Flow.jpg` | Medium — timeline | 1.5 |
| 9 | Call Detail > Reminder | `call details - Reminder.jpg` | Low — simple form | 0.5 |
| 10 | Activities > Texts | `Activity Logs - Texts.jpg` | Medium — variant of Calls | 1.5 |
| 11 | Activities > Forms | `Activity Logs - Forms.jpg` | Medium — variant of Calls | 1.5 |
| 12 | Activities > Chats | `Activity Logs - Chatsjpg.jpg` | Low — variant of Calls | 1 |
| 13 | Activities > Faxes | `Activity Logs - Faxes.jpg` | Low — empty state | 0.5 |
| 14 | Activities > Videos | `Activity Logs - Videos.jpg` | Low — empty state | 0.5 |
| 15 | Activities > Export Log | `Activity Logs - Export Log.jpg` | High — complex form | 2.5 |
| 16 | Navigation States | `Activity Logs - Calls.jpg` (nav closeup) | Low — component variants | 0.5 |

**Total estimated: ~22 design-hours for this 25% batch**

### Phase 3: Missing Pages (anticipated from remaining 75% screenshots)

Based on the navigation structure, the following sections are NOT yet captured:
- **Numbers** — phone number management, tracking numbers
- **Flows** — automation workflow builder/editor
- **AI Tools** — AI-powered analysis features
- **Reports** — analytics dashboards, charts
- **Trust Center** — compliance settings
- **Settings** — account, user, integration settings
- **Contacts sub-pages** — Lists, Blocked Numbers, Do Not Call List, Do Not Text List
- **Call Detail > Zoho tab** — CRM integration view
- **Call Detail > Script tab** — call script display
- **Phone/Softphone** — the green Phone button likely opens a softphone widget

---

## 4. Execution Order

### Batch 1 (Design System + Shared Components)
1. Create Figma project: "4iiz Redesign"
2. Set up color tokens, typography, spacing scale
3. Build atomic components (buttons, inputs, toggles, avatars)
4. Build composite components (nav bars, row templates, detail header)

### Batch 2 (Primary Pages)
5. Frame 1: Calls home page (establishes the master list pattern)
6. Frame 10-14: Texts, Forms, Chats, Faxes, Videos (variants of master list)
7. Frame 15: Export Log (complex standalone form)

### Batch 3 (Call Detail Tabs)
8. Frame 2: Text Message (chat thread)
9. Frame 3: Contact (dense form)
10. Frame 4-9: Visitor Detail, Score, Email, Voice Analysis, Flow, Reminder

### Batch 4 (States & Interactions)
11. Empty states for all list pages
12. Loading/skeleton states
13. Hover, active, selected states for rows
14. Mobile responsive variants (if needed)

---

## 5. Design Notes & Observations

### Visual Language
- **Color palette**: Cyan/teal (#00BCD4-ish) as primary action color, light blue for links, red for flags/alerts, gray for borders and metadata
- **Style**: Clean, minimal, data-dense — optimized for agents processing high volumes
- **Typography**: Sans-serif (likely system fonts or Inter/Roboto), small font sizes for density
- **Layout**: Fixed left nav + scrollable content area, no horizontal scroll
- **Spacing**: Tight — optimized for information density over whitespace

### UX Patterns
- **Master-detail**: List view → slide-out detail panel (not full page navigation)
- **Tabbed detail**: 10+ tabs in a single detail view — could benefit from grouping
- **Inline actions**: Call/Listen/Edit/Reply buttons on each row
- **Real-time**: Auto Load toggle, "in progress" status indicators, live call count
- **Multi-language**: Interface in English but content heavily Spanish (law firm serving Hispanic community)
- **CRM integration**: Deep Zoho CRM fields (contact_id, matter_id, case subtype, matters stage)
- **Automation visibility**: Flow tab shows full automation audit trail per call

### Data Architecture Insights
- Activities are the central entity (calls, texts, forms, chats, faxes, videos)
- Each activity links to: Contact, Source (tracking number), Agent, Score, Session (analytics)
- Heavy tag-based workflow: tags drive automation flows
- Dual CRM: internal contact system + Zoho CRM integration
- Multi-channel: phone, SMS, web forms, chat (Woosender), fax, video
