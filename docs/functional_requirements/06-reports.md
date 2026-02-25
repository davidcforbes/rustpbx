# 06 - Reports Section

> 30 pages | 10 well-defined, 20 need definition
> Source: `ui/src/sections/reports.rs`

## Side Navigation

Four groups:
- **Analytics (21):** Activity Reports, ROI Reports, Accuracy Reports, Activity Map, Overview, Today's Missed Calls, Positive Daily Reports, Google CA Report, Saturday Calls, Daily Calls, Weekly Missed Calls, Priming Calls, Missed Calls, Missed Calls Daily - 1st, CS Daily Missed Calls, CS Daily Missed 2.0, Priming Missed Calls, Daily Collection Calls, Power BI - Total Inbound, Real Time, Appointments
- **Connect (4):** Real-time Agents, Coaching, Queue Report, Agent Activity
- **Usage (1):** Agency Usage
- **Report Settings (4):** Custom Reports, Notifications, Scoring, Tags

---

## Well-Defined Pages

### 6.1 Activity Report Page

- **Route:** `/reports/activity`
- **Status:** Well-defined

**UI Elements:**
- Header with breadcrumbs, date range selector, "Export" button
- View-by selector: Source / Campaign / Keyword / Landing Page
- Time granularity buttons: Day / Week / Month
- Bar chart (SVG with 7 colored bars)
- Source data table with 14 metric columns

**Data Model -- `SourceRow`:**

| Field | Type | Description |
|-------|------|-------------|
| name | &str | Source name |
| badge_pct | &str | Percentage badge |
| badge_color | &str | Badge color class |
| total | &str | Total calls |
| total_pct | &str | Total percentage |
| period_unique | &str | Period-unique callers |
| period_unique_pct | &str | Period-unique percentage |
| globally_unique | &str | Globally-unique callers |
| globally_unique_pct | &str | Globally-unique percentage |
| ring_avg | &str | Average ring time |
| ring_total | &str | Total ring time |
| talk_avg | &str | Average talk time |
| talk_total | &str | Total talk time |
| total_time_avg | &str | Average total time |
| total_time_total | &str | Total time sum |

**Actions:** Export, change view-by dimension, change time granularity, date range filter

**Implied Functionality:** Marketing attribution analytics with multi-dimensional analysis, unique caller tracking, time-based aggregation

---

### 6.2 Activity Map Page

- **Route:** `/reports/map`
- **Status:** Needs definition (stub)

**UI:** Header with date range filter, map placeholder with geo-pin icon

> **NEEDS DEFINITION:**
> - Map provider integration (Google Maps / Mapbox / Leaflet)
> - Data overlay format (pins, heatmap, clusters)
> - Filtering by source, date, status
> - Drill-down behavior on click
> - Geographic granularity (city, state, zip)

---

### 6.3 Real-Time Agents Page

- **Route:** `/reports/realtime-agents`
- **Status:** Well-defined

**UI Elements:**
- Header with title, "Refresh" button
- 4 status summary cards: Available (green), On Call (blue), After Call Work (yellow), Offline (gray)
- Agent status table with pagination

**Table Columns:** Name (with avatar), Status (badge), Duration, Queue

**Actions:** Refresh

**Implied Functionality:** Real-time agent monitoring dashboard showing current agent states and queue assignment

---

### 6.4 Coaching Page

- **Route:** `/reports/coaching`
- **Status:** Well-defined

**UI Elements:**
- Header with title, "Refresh" button
- Active calls table: Agent (with avatar), Caller, Duration, Queue, Actions (Listen/Whisper/Barge buttons)

**Actions:** Listen (silent monitor), Whisper (speak to agent only), Barge (join call), Refresh

**Implied Functionality:** Supervisor call monitoring with listen/whisper/barge capabilities for real-time agent coaching

---

### 6.5 Queue Report Page

- **Route:** `/reports/queue-report`
- **Status:** Well-defined

**UI Elements:**
- Header with title, date range filter, "Refresh" button
- 3 stat cards: Calls Waiting, Avg Wait Time, Service Level (percentage)
- Queue performance table with pagination

**Table Columns:** Queue Name, Calls Waiting, Avg Wait, Avg Handle, Abandoned (count), Service Level (%), Agents Online

**Actions:** Refresh, date range filter

**Implied Functionality:** Queue performance monitoring with SLA tracking, abandonment rates, agent utilization

---

### 6.6 Agent Activity Page

- **Route:** `/reports/agent-activity`
- **Status:** Well-defined

**UI Elements:**
- Header with title, date range filter, "Export" button
- Agent performance table with pagination

**Table Columns:** Agent (with avatar), Calls Handled, Avg Handle Time, ACW Time, Availability %

**Actions:** Export, date range filter

**Implied Functionality:** Agent productivity reporting with handle time, after-call work, and availability metrics

---

### 6.7 Agency Usage Page

- **Route:** `/reports/agency-usage`
- **Status:** Well-defined

**UI Elements:**
- Header with title, date range filter, "Export" button
- 3 summary cards: Total Calls, Total Minutes, Text Messages
- Usage breakdown table with pagination

**Table Columns:** Account, Calls, Minutes, Texts, Cost

**Actions:** Export, date range filter

**Implied Functionality:** Multi-account/agency usage billing report with cost attribution

---

### 6.8 Custom Reports Page

- **Route:** `/reports/custom-reports`
- **Status:** Well-defined

**Table Columns:** Name, Type, Schedule, Last Run, Created, Actions (Edit/Run/Remove)

**Actions:** "New Report" button, Edit/Run/Remove per row

**Implied Functionality:** Saved custom report definitions with scheduling and on-demand execution

---

### 6.9 Notifications Page

- **Route:** `/reports/notifications`
- **Status:** Well-defined

**Table Columns:** Name, Type, Recipients, Trigger, Active (toggle), Actions (Edit/Remove)

**Actions:** "New Notification" button, Active toggle per row, Edit/Remove per row

**Implied Functionality:** Alert/notification rule management for report-based triggers (e.g., "alert when missed calls exceed threshold")

---

### 6.10 Scoring Page

- **Route:** `/reports/scoring`
- **Status:** Well-defined

**UI Elements:**
- Header with title, "Save" button
- Call Scoring Criteria card with 3 range sliders:
  - Answer Rate: 40% (0-100)
  - Talk Time: 35% (0-100)
  - Conversion: 25% (0-100)
- Total weight display (should sum to 100%)

**Actions:** Save, adjust weight sliders

**Implied Functionality:** Configurable call scoring algorithm with weighted criteria

> **NEEDS DEFINITION:** How scores are applied to calls, score thresholds/grades, integration with call detail view and reports

---

### 6.11 Tags Page

- **Route:** `/reports/tags`
- **Status:** Well-defined

**UI Elements:**
- Header with title, "New Tag" button
- Tag list: 8 tags, each with colored dot, name, usage count, Edit/Remove actions

**Mock Tags:** Lead (245), Appointment (189), Follow Up (134), VIP (78), Spam (56), Billing (23), Support (167), Sales (312)

**Actions:** "New Tag" button, Edit/Remove per tag

**Implied Functionality:** Tag taxonomy management for call classification with usage statistics

---

## Pages Needing Definition

The following 19 pages all use the identical `ReusableReportPage` template and need their own specific data models, chart types, metrics, and table columns defined.

### Reusable Template UI
- Header with title, date range filter, "Export" button
- 3 stat cards: "Total Calls" = 0, "Avg Duration" = "0:00", "Conversion Rate" = "0%"
- Chart placeholder with "Chart will appear here" text
- Empty data table with headers: Source, Calls, Duration, Conversion

### Reports Needing Specific Design

| # | Route | Title | Suggested Focus |
|---|-------|-------|-----------------|
| 1 | `/reports/roi` | ROI Reports | Revenue attribution per source, cost-per-call, ROAS |
| 2 | `/reports/accuracy` | Accuracy Reports | Source attribution accuracy, data quality metrics |
| 3 | `/reports/overview` | Overview | Executive dashboard with KPI summary |
| 4 | `/reports/todays-missed` | Today's Missed Calls | Real-time missed call list for current day |
| 5 | `/reports/positive-daily` | Positive Daily Reports | Daily positive outcome calls (appointments, conversions) |
| 6 | `/reports/google-ca` | Google CA Report | Google Ads call attribution metrics |
| 7 | `/reports/saturday-calls` | Saturday Calls | Weekend call volume analysis |
| 8 | `/reports/daily-calls` | Daily Calls | Day-by-day call volume trends |
| 9 | `/reports/weekly-missed` | Weekly Missed Calls | Weekly missed call trends |
| 10 | `/reports/priming` | Priming Calls | Initial/priming call analysis |
| 11 | `/reports/missed` | Missed Calls | Comprehensive missed call report |
| 12 | `/reports/missed-daily-1st` | Missed Calls Daily - 1st | First-time missed caller analysis |
| 13 | `/reports/cs-daily-missed` | CS Daily Missed Calls | Customer service daily missed calls |
| 14 | `/reports/cs-daily-missed-2` | CS Daily Missed 2.0 | Enhanced CS missed call analysis |
| 15 | `/reports/priming-missed` | Priming Missed Calls | Missed priming calls |
| 16 | `/reports/daily-collection` | Daily Collection Calls | Collections team daily call report |
| 17 | `/reports/power-bi` | Power BI - Total Inbound | Power BI embed or inbound call totals |
| 18 | `/reports/realtime` | Real Time | Live call activity dashboard |
| 19 | `/reports/appointments` | Appointments | Appointment booking tracking |

> **ACTION REQUIRED:** Each of these 19 reports needs:
> 1. Specific KPI cards (what 3-4 metrics to show at the top)
> 2. Chart type (bar, line, pie, heatmap, table-only)
> 3. Table columns specific to that report's purpose
> 4. Filter options beyond date range
> 5. Data model definition
