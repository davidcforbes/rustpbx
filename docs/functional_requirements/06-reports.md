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

## Analytics Reports (Previously Template-Only, Now Well-Defined)

The following 19 reports were previously using a generic template. Each now has unique KPI cards, chart visualizations, report-specific table columns, and mock data.

### 6.12 ROI Reports Page

- **Route:** `/reports/roi`
- **Status:** Well-defined

**UI Elements:**
- Header with "Revenue attribution per tracking source" subtitle
- View-by selector: Tracking Source (dropdown)
- Time granularity: Hour / Day / Week / Month
- Source data table with percentage badges

**Table Columns:** Source (with % badge), Total, Period Unique, Avg Talk Time

**Implied Functionality:** Marketing ROI attribution showing call volume and engagement by traffic source

---

### 6.13 Accuracy Reports Page

- **Route:** `/reports/accuracy`
- **Status:** Well-defined

**UI Elements:**
- Header with "Source attribution accuracy and data quality" subtitle
- 4 KPI cards: Accuracy Rate (94.2%), Verified Sources (87.3%), Unattributed (5.8%), Data Quality Score (91/100)
- CSS donut chart showing attribution breakdown
- Quality metrics table

**Table Columns:** Source, Total Calls, Attributed, Unattributed, Accuracy %, Quality Score

**Implied Functionality:** Data quality monitoring for marketing attribution accuracy

---

### 6.14 Overview Page

- **Route:** `/reports/overview`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Total Calls (110,050), Answered (89,241 / 81.1%), Missed (15,407 / 14.0%), Avg Duration (2:18)
- Stacked bar chart showing Answered vs Missed by day of week
- Weekly breakdown table

**Table Columns:** Day, Total, Answered, Missed, Voicemail, Avg Ring, Avg Talk

**Implied Functionality:** Executive KPI dashboard with weekly call distribution and answer rate trends

---

### 6.15 Today's Missed Calls Page

- **Route:** `/reports/todays-missed`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Missed Today (47), vs Yesterday (+12), Callback Rate (34.0%), Avg Wait Before Abandon (0:23)
- Hourly distribution bar chart
- Missed call detail table

**Table Columns:** Time, Caller, Phone, Source, Duration, Queue, Status (badge: New/Callback Scheduled/No Answer)

**Implied Functionality:** Real-time missed call monitoring for same-day callback prioritization

---

### 6.16 Positive Daily Reports Page

- **Route:** `/reports/positive-daily`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Positive Outcomes (156), Conversion Rate (14.2%), Appointments Set (89), Revenue ($45,230)
- Daily trend bar chart (7 days)
- Positive outcomes table

**Table Columns:** Date, Total Calls, Positive, Appointments, Conversions, Revenue, Rate %

**Implied Functionality:** Daily tracking of positive call outcomes (appointments, conversions, revenue)

---

### 6.17 Google CA Report Page

- **Route:** `/reports/google-ca`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Google Calls (12,456), Cost per Call ($8.42), Conversion Rate (11.2%), ROAS (4.2x)
- Campaign performance bar chart
- Campaign detail table

**Table Columns:** Campaign, Calls, Spend, Cost/Call, Conversions, Conv Rate, Revenue, ROAS

**Implied Functionality:** Google Ads call attribution with cost analysis and ROAS tracking

---

### 6.18 Saturday Calls Page

- **Route:** `/reports/saturday-calls`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Saturday Total (2,340), vs Weekday Avg (-62%), Answer Rate (71.2%), Avg Wait (0:45)
- Hourly distribution chart (8 AM - 5 PM)
- Hourly breakdown table

**Table Columns:** Hour, Calls, Answered, Missed, Answer Rate, Avg Wait, Avg Talk

**Implied Functionality:** Weekend call volume analysis for staffing optimization

---

### 6.19 Daily Calls Page

- **Route:** `/reports/daily-calls`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Today (3,847), vs Yesterday (+5.2%), 7-Day Avg (3,654), Peak Hour (10 AM)
- 7-day trend bar chart
- Daily breakdown table

**Table Columns:** Date, Total, Answered, Missed, First-Time, Repeat, Avg Duration

**Implied Functionality:** Day-by-day call volume trends with first-time vs repeat caller tracking

---

### 6.20 Weekly Missed Calls Page

- **Route:** `/reports/weekly-missed`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: This Week (342), vs Last Week (-8.3%), Miss Rate (14.1%), Callback Success (67.2%)
- Week-over-week comparison bar chart (4 weeks)
- Weekly detail table

**Table Columns:** Week, Total Missed, Mon-Fri counts, Weekend, Miss Rate %, Callbacks

**Implied Functionality:** Weekly missed call trend analysis with day-of-week distribution

---

### 6.21 Priming Calls Page

- **Route:** `/reports/priming`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Total Priming (1,245), Contact Rate (78.3%), Qualified Rate (34.2%), Conversion Rate (12.1%)
- Funnel visualization: Attempts > Contacted > Qualified > Converted
- Priming campaign table

**Table Columns:** Campaign, Attempts, Contacted, Contact Rate, Qualified, Converted, Conv Rate

**Implied Functionality:** Initial/priming call funnel analysis from attempt through conversion

---

### 6.22 Missed Calls Page

- **Route:** `/reports/missed`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Total Missed (4,521), After Hours (1,823 / 40.3%), During Hours (2,698 / 59.7%), Avg Ring Time (0:18)
- 24-hour distribution chart
- Hourly breakdown table

**Table Columns:** Hour, Missed Count, % of Total (with bar), After Hours flag, Avg Ring, Top Source

**Implied Functionality:** Comprehensive missed call analysis with business-hours vs after-hours breakdown

---

### 6.23 Missed Calls Daily - 1st Page

- **Route:** `/reports/missed-daily-1st`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: First-Time Missed (892), % of All Missed (42.1%), Callback Within 1hr (34.5%), Lost Leads Est. (156)
- Daily trend chart (7 days)
- First-time missed caller table

**Table Columns:** Date, First-Time Missed, Repeat Missed, First-Time %, Callback Rate, Est. Lost Value

**Implied Functionality:** First-time missed caller analysis for lead recovery prioritization

---

### 6.24 CS Daily Missed Calls Page

- **Route:** `/reports/cs-daily-missed`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: CS Missed Today (23), Service Level (82.4%), Avg Wait (1:12), Escalations (4)
- Queue distribution chart
- Queue-level detail table

**Table Columns:** Queue, Missed, Total, Miss Rate, Avg Wait, Longest Wait, Agents Online

**Implied Functionality:** Customer service queue-level missed call analysis for SLA monitoring

---

### 6.25 CS Daily Missed 2.0 Page

- **Route:** `/reports/cs-daily-missed-2`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Total Missed (31), First Contact Resolution (76.8%), Repeat Callers (12.3%), Sentiment Score (3.8/5)
- Agent-level breakdown chart
- Agent detail table

**Table Columns:** Agent, Missed, Handled, Miss Rate, Avg Handle, FCR %, Sentiment

**Implied Functionality:** Enhanced CS missed call analysis with agent-level breakdown and sentiment tracking

---

### 6.26 Priming Missed Calls Page

- **Route:** `/reports/priming-missed`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Priming Missed (234), Miss Rate (18.8%), Best Retry Window (10-11 AM), Retry Success (45.2%)
- Weekly trend chart (4 weeks)
- Daily detail table

**Table Columns:** Date, Attempted, Missed, Miss Rate, Best Hour, Retried, Retry Success %

**Implied Functionality:** Missed priming call analysis with optimal retry window identification

---

### 6.27 Daily Collection Calls Page

- **Route:** `/reports/daily-collection`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Calls Today (445), Promises (67), Amount Promised ($34,560), Contact Rate (62.3%)
- Daily trend chart (7 days)
- Agent collection table

**Table Columns:** Agent, Calls, Contacts, Contact Rate, Promises, Amount Promised, Collected

**Implied Functionality:** Collections team daily call tracking with promise-to-pay metrics

---

### 6.28 Power BI - Total Inbound Page

- **Route:** `/reports/power-bi`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Total Inbound (15,234), vs Prior Period (+8.3%), Peak Hour (10 AM / 1,245 calls), Service Level (87.2%)
- Hourly distribution chart
- Hourly detail table

**Table Columns:** Hour, Inbound, Answered, Missed, Answer Rate, Avg Speed, Service Level

**Implied Functionality:** Hourly inbound call distribution for capacity planning

---

### 6.29 Real Time Page

- **Route:** `/reports/realtime`
- **Status:** Well-defined

**UI Elements:**
- 4 status indicator cards: Active Now (23), In Queue (7), Avg Wait (0:34), Agents Available (12)
- Live status indicators with pulsing green dots
- Active calls table

**Table Columns:** Caller, Agent, Queue, Duration (live), Status (badge: Active/Ringing/On Hold/Wrapping), Wait Time

**Implied Functionality:** Live call activity dashboard with real-time status indicators

---

### 6.30 Appointments Page

- **Route:** `/reports/appointments`
- **Status:** Well-defined

**UI Elements:**
- 4 KPI cards: Appointments Today (12), This Week (67), Conversion Rate (8.4%), No-Show Rate (14.2%)
- Weekly appointments trend chart (4 weeks)
- Appointment detail table with type and status badges

**Table Columns:** Date/Time, Caller, Phone, Source, Agent, Type (badge: New/Follow-up/Consultation), Status (badge: Confirmed/Pending/Completed/No-Show), Notes

**Implied Functionality:** Appointment booking tracking with type classification and status management
