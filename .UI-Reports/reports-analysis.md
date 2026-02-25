# 4iiz Reports Section - UI Analysis & Prototype Plan

## Section Overview

The **Reports** section is the analytics and business intelligence hub of the 4iiz platform. Accessed via the bar-chart icon in the left icon nav, it provides call tracking analytics, source attribution, agent performance monitoring, and custom reporting. The section is divided into four sidebar groups: **Analytics** (20+ saved reports), **Connect** (real-time agent monitoring), **Usage** (agency-level metrics), and **Report Settings** (customization and configuration).

---

## Page Inventory (1 screenshot, 13 MHTML captures)

| # | Page | Source File | Section | Key Feature |
|---|------|-----------|---------|-------------|
| 1 | Activity Report | `Activity Report.jpg` + `.mhtml` | ANALYTICS | Main analytics dashboard with chart + data table |
| 2 | ROI Report | `.mhtml` | ANALYTICS | Return on investment tracking |
| 3 | Daily Calls | `.mhtml` | ANALYTICS | Daily call volume report |
| 4 | Google CA Report | `.mhtml` | ANALYTICS | Google call analytics integration |
| 5 | Overview / Calls by Source | `.mhtml` | ANALYTICS | Call distribution overview |
| 6 | Positive Daily Reports | `.mhtml` | ANALYTICS | Daily positive outcome tracking |
| 7 | Saturday Calls | `.mhtml` | ANALYTICS | Weekend call report |
| 8 | Today's Missed Calls | `.mhtml` | ANALYTICS | Real-time missed call report |
| 9 | Agent Activity | `.mhtml` | CONNECT | Per-agent performance metrics |
| 10 | Real-time Agents - Coaching | `.mhtml` | CONNECT | Live coaching dashboard |
| 11 | Real-time Agents - Manager | `.mhtml` | CONNECT | Manager overview dashboard |
| 12 | Real-time Agents - Queue Report | `.mhtml` | CONNECT | Queue status and wait times |
| 13 | Real-time Agents - Reporting | `.mhtml` | CONNECT | Agent reporting dashboard |

---

## Sidebar Navigation Structure

### ANALYTICS (20+ items)
Saved and built-in analytics reports. The sidebar is scrollable with many entries:
- **Activity Reports** (default/selected)
- ROI Reports
- Accuracy Reports
- Activity Map
- Overview
- Today's Missed Calls
- Positive Daily Reports
- Google CA Report
- saturday calls
- Daily Calls
- Weekly Missed Calls
- Priming Calls
- Missed Calls
- Missed Calls Daily - First Contact
- Customer Service Daily Missed Calls
- Customer Service Daily Missed Calls - 2.0
- Priming Missed Calls
- Daily Collection Calls
- Power BI - Total Inbound
- real time
- Appointments

### CONNECT (4 items)
Real-time agent monitoring and coaching:
- Real-time Agents
- Coaching
- Queue Report
- Agent Activity

### USAGE (1 item)
- Agency Usage

### REPORT SETTINGS (4 items)
Report configuration and customization:
- Custom Reports
- Notifications
- Scoring
- Tags

---

## Detailed Page Analysis

### Page 1: Activity Report (Primary Dashboard)

**Purpose**: The main analytics view providing source attribution, call volume trends, and performance metrics across all tracking sources over a configurable time range.

**Layout**: Three-tier vertical layout: toolbar, bar chart, data table.

#### Top Bar Elements
- **"Reports"** title in secondary nav header
- **Filter button** (funnel icon with "Filter" label)
- **Search bar** with settings gear icon and magnifying glass
- **Info** link (circle-i icon)
- **Call Log** link (phone icon)
- **Export** dropdown (with caret)
- **"Schedules..."** CTA button (cyan background, white text) - for scheduling automated reports

#### View Selector Row
- **"View by" dropdown**: Currently set to "Tracking Source", with dropdown caret
- **"+" button**: Add new view/grouping
- **Time range tabs** (right-aligned): Hour | **Day** (active, cyan background) | Week | Month | Quarter | Year

#### Bar Chart Area
- **Type**: Stacked bar chart (vertical bars)
- **X-axis**: Date range, approximately Jan 26 through Feb 23 (~30 days)
- **Y-axis**: Call volume, scale 0 to 6k (gridlines at 1k, 2k, 3k, 4k, 5k, 6k)
- **Bar colors**: Green (answered calls), Red (missed calls), Gray/other (additional categories)
- **Data labels**: Numbers shown above each bar (e.g., 4705, 4790, 4522, 4293, etc.)
- **Peak value**: 5,032 calls on a single day
- **Low values**: Weekend dips visible (769 on one day, 578 on another)
- **Chart controls**: Small icon buttons in top-right of chart area (bar chart type toggles)

#### Data Table
**Columns** (left to right):

| Column | Type | Notes |
|--------|------|-------|
| Source | Text + color badge | Color-coded percentage badge (e.g., "73%" with green/red/gray bg) |
| Total | Number + % | Total calls with percentage below |
| Period Unique | Number + % | Unique callers within period |
| Globally Unique | Number + % | Globally unique callers |
| Ring Time (minutes) | Duration | Format: `M:SS avg` with secondary line showing total |
| Talk Time (minutes) | Duration | Same format as Ring Time |
| Total Time (minutes) | Duration | Same format |
| Score | Number | Call scoring (0.00 for most) |
| Conversions | Number | Conversion count |
| Conversion Rate | Percentage | Conversion rate |
| Revenue | Currency | Dollar amount |
| Social Selling New Leads | Number | New lead count |

#### Totals Row
| Metric | Value |
|--------|-------|
| Total | 110,050 |
| Period Unique | 31,721 |
| Globally Unique | 9,671 |
| Ring Time | 0:27 avg / 51,342.78 total |
| Talk Time | 2:18 avg / 254,905.82 total |
| Total Time | 2:52 avg / 316,130.67 total |
| Score | 0.00 avg |
| Conversions | 0 total |
| Conversion Rate | 0.00% rate |
| Revenue | $0.00 total |
| Social Selling New Leads | 3,892 total |

#### Source Rows (14 visible, sorted by total descending)

| # | Source | Badge | Total | % | Period Unique | Globally Unique |
|---|--------|-------|-------|---|---------------|-----------------|
| 1 | Google Organic | 73% (green) | 80,374 | 73.03% | 19,988 / 18.16% | 6,260 / 5.69% |
| 2 | Customer Service Line | 20% (orange) | 22,270 | 20.24% | 6,809 / 6.19% | 425 / 0.39% |
| 3 | Tiktok Organic | 2% (red) | 2,526 | 2.30% | 1,746 / 1.59% | 1,219 / 1.11% |
| 4 | Facebook Paid | 3% (red) | 1,942 | 1.76% | 1,178 / 1.07% | 771 / 0.70% |
| 5 | Facebook Organic | 1% (green) | 701 | 0.64% | 556 / 0.51% | 410 / 0.37% |
| 6 | Book of Truths Trum | 1% (green) | 625 | 0.57% | 326 / 0.30% | 81 / 0.07% |
| 7 | Radio La Ley | — (gray) | 425 | 0.39% | 302 / 0.27% | 136 / 0.12% |
| 8 | Website | 0% (red) | 288 | 0.26% | 194 / 0.18% | 88 / 0.08% |
| 9 | Instagram Organic | 0% (red) | 187 | 0.17% | 127 / 0.12% | 86 / 0.08% |
| 10 | Yelp Organic | — | 173 | 0.16% | 119 / 0.11% | 59 / 0.05% |
| 11 | Mass SMS | 0% (green) | 167 | 0.15% | 129 / 0.12% | 8 / 0.01% |
| 12 | WhatsApp | 0% (red) | 110 | 0.10% | 68 / 0.06% | 38 / 0.03% |
| 13 | Mystery Shopper | 0% (green) | 74 | 0.07% | 35 / 0.03% | 4 / 0.00% |
| 14 | Google Ads | 0% (red) | 67 | 0.06% | 45 / 0.04% | 20 / 0.02% |

#### Color Badge Pattern
Each source row has a small rounded badge to the left of the source name showing a percentage with a background color:
- **Green badges**: Google Organic (73%), Facebook Organic (1%), Book of Truths (1%), Mass SMS (0%), Mystery Shopper (0%)
- **Red badges**: Tiktok Organic (2%), Facebook Paid (3%), Website (0%), Instagram Organic (0%), WhatsApp (0%), Google Ads (0%)
- **Orange/amber badge**: Customer Service Line (20%)
- **Gray/no badge**: Radio La Ley, Yelp Organic

The badge color likely indicates trend direction (green = up, red = down, orange = warning) or source category.

---

### Pages 2-8: Analytics Sub-Reports

These are saved report configurations under the ANALYTICS section. Based on naming patterns:

| Report | Likely Purpose |
|--------|---------------|
| ROI Report | Revenue attribution per tracking source |
| Accuracy Reports | Call scoring accuracy and quality metrics |
| Activity Map | Geographic visualization of call origins |
| Overview / Calls by Source | Simplified source distribution view |
| Today's Missed Calls | Real-time dashboard of missed calls for current day |
| Positive Daily Reports | Calls meeting positive outcome criteria |
| Google CA Report | Google call analytics integration report |
| saturday calls | Weekend-specific call volume filter |
| Daily Calls | Day-by-day call volume without source breakdown |
| Weekly Missed Calls | Aggregated weekly missed call trends |
| Priming Calls / Priming Missed Calls | Pre-qualification call tracking |
| Missed Calls / Missed Calls Daily - First Contact | Missed call analysis for new contacts |
| Customer Service Daily Missed Calls (1.0 & 2.0) | Customer service queue missed call tracking |
| Daily Collection Calls | Collections department call tracking |
| Power BI - Total Inbound | External BI integration report |
| real time | Live call dashboard |
| Appointments | Appointment scheduling conversion tracking |

Most of these share the same Activity Report layout (chart + table) but with different filters, date ranges, or source selections pre-configured.

---

### Pages 9-13: CONNECT Section (Real-time Agent Monitoring)

#### Real-time Agents
Live dashboard showing current agent status (available, on-call, after-call work, offline).

#### Coaching
Supervisor coaching interface with listen/whisper/barge capabilities tied to real-time call monitoring.

#### Queue Report
Real-time queue metrics: calls waiting, average wait time, service level, abandoned calls.

#### Agent Activity
Historical agent performance: calls handled, average handle time, after-call work time, availability percentage.

---

## Shared UI Patterns

### Pattern: Report Dashboard Layout
All Analytics reports follow the same three-tier template:
1. **Toolbar**: Filter + Search + Info + Call Log + Export + Schedules CTA
2. **Chart area**: Configurable chart (bar, line, etc.) with time range selector
3. **Data table**: Sortable columns with totals row and source/dimension rows

### Pattern: Time Range Selector
- Six options: Hour, Day, Week, Month, Quarter, Year
- Active tab has cyan background with white text
- Inactive tabs have white background with gray text
- Tabs are right-aligned in the view selector row

### Pattern: View By Selector
- Dropdown to change the grouping dimension (Tracking Source, Agent, etc.)
- "+" button to add additional grouping dimensions
- Left-aligned in the view selector row

### Pattern: Color-Coded Badges
- Small rounded rectangle badges with percentage text
- Background color indicates trend direction or category
- Positioned to the left of the source/dimension name

### Pattern: Dual-Line Cell Values
- Primary value (large, bold): The main metric
- Secondary value (small, gray): Percentage, total, or contextual detail
- Used in Total, Period Unique, Globally Unique, and time columns

### Pattern: Duration Formatting
- Format: `M:SS` for average values
- Secondary line shows total in minutes with "total" label
- Ring Time, Talk Time, and Total Time all use this pattern

---

## Component Mapping to DaisyUI

| 4iiz Component | DaisyUI Equivalent |
|----------------|-------------------|
| Data table | `table table-sm` with custom header styling |
| Time range tabs | `join` + `btn btn-sm` group with active state |
| View by dropdown | `select select-sm select-bordered` |
| Bar chart area | Custom `div` (would use Chart.js or similar in production) |
| Filter button | `btn btn-sm btn-ghost` with funnel icon |
| Search bar | `input input-bordered input-sm` with search icon |
| Export dropdown | `dropdown` + `btn btn-sm btn-ghost` |
| Schedules CTA | `btn btn-sm bg-iiz-cyan text-white` |
| Color badges | `badge badge-sm` with dynamic background color |
| Totals row | `table` row with `font-bold bg-gray-50` |
| Sidebar sections | Grouped `h3` headers + `a.side-nav-item` links |
| Chart type toggles | `btn-group btn-xs` icon buttons |
| Info link | `link` with circle-info icon |
| Call Log link | `link` with phone icon |
| Pagination (if applicable) | `join` + `btn btn-xs` group |

---

## Prototype

**File**: `.UI-Reports/prototype/reports.html`

Covers the primary Activity Report page with full data table, plus placeholder pages for all sidebar navigation items. Uses Alpine.js state switching (`reportPage`) for page navigation:

- `activity` - Activity Report (full chart placeholder + data table with 14 source rows)
- `roi` - ROI Reports placeholder
- `accuracy` - Accuracy Reports placeholder
- `activitymap` - Activity Map placeholder
- `overview` - Overview placeholder
- `todaysmissed` - Today's Missed Calls placeholder
- `positivedaily` - Positive Daily Reports placeholder
- `googleca` - Google CA Report placeholder
- `saturdaycalls` - Saturday Calls placeholder
- `dailycalls` - Daily Calls placeholder
- `weeklymissed` - Weekly Missed Calls placeholder
- `primingcalls` - Priming Calls placeholder
- `missedcalls` - Missed Calls placeholder
- `misseddaily1st` - Missed Calls Daily - First Contact placeholder
- `csdailymissed` - Customer Service Daily Missed Calls placeholder
- `csdailymissed2` - Customer Service Daily Missed Calls 2.0 placeholder
- `primingmissed` - Priming Missed Calls placeholder
- `dailycollection` - Daily Collection Calls placeholder
- `powerbi` - Power BI - Total Inbound placeholder
- `realtime` - Real-time placeholder
- `appointments` - Appointments placeholder
- `realtimeagents` - Real-time Agents placeholder
- `coaching` - Coaching placeholder
- `queuereport` - Queue Report placeholder
- `agentactivity` - Agent Activity placeholder
- `agencyusage` - Agency Usage placeholder
- `customreports` - Custom Reports placeholder
- `notifications` - Notifications placeholder
- `scoring` - Scoring placeholder
- `tags` - Tags placeholder

All sample data from the Activity Report screenshot is included.

---

## Observations for Production Implementation

1. **Source attribution is the core value**: Google Organic drives 73% of call volume. The ability to track and attribute calls to marketing sources (organic, paid, social, radio, etc.) is the primary business value of the Reports section.

2. **Chart library required**: The bar chart is a key visual element. Production implementation should use Chart.js, D3, or a similar library. The prototype uses a placeholder div.

3. **Custom reports are user-created**: The 20+ entries under ANALYTICS suggest users can save filtered views as named reports. A "Custom Reports" builder under Report Settings confirms this.

4. **Real-time monitoring is separate**: The CONNECT section serves a different purpose (live agent supervision) than ANALYTICS (historical reporting). These may warrant different backend architectures (WebSocket for real-time vs. REST for historical).

5. **Dual-value cells are information-dense**: Each numeric cell shows both an average/primary value and a total/secondary value. This requires careful responsive design to avoid truncation on smaller screens.

6. **Scoring and conversions are underutilized**: All rows show 0.00 score, 0 conversions, 0.00% conversion rate, and $0.00 revenue. These features exist but are not configured for this account, suggesting they require setup in Report Settings > Scoring.

7. **Export and scheduling are premium features**: The Export dropdown and "Schedules..." CTA indicate automated report delivery (email, PDF, etc.) on a schedule.

8. **Time granularity affects chart type**: Hour/Day likely show bar charts, while Week/Month/Quarter/Year may switch to line charts or area charts for trend visualization.

9. **Many sidebar items are custom saved reports**: Items like "saturday calls" (lowercase), "real time", "Priming Calls" appear to be user-created saved filters rather than built-in report types. The naming inconsistency (mixed case, abbreviations) confirms this.

10. **Agency Usage suggests multi-tenant**: The USAGE section with "Agency Usage" implies the platform supports agency accounts managing multiple client sub-accounts, with usage tracking for billing purposes.
