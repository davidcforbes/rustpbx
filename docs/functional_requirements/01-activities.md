# 01 - Activities Section

> 7 pages | All well-defined
> Source: `ui/src/sections/activities.rs`

## Side Navigation

Two groups under the Activities icon:
- **Activity Logs:** Calls, Texts, Forms, Chats, Faxes, Videos, Export Log
- **Contacts:** Lists, Blocked Numbers, Do Not Call List, Do Not Text List *(see [02-contacts.md](02-contacts.md))*

---

## 1.1 Calls Page

- **Route:** `/activities/calls`
- **Status:** Well-defined

### UI Elements
- FilterBar (shared component)
- Sortable data table with selectable rows (checkbox per row + select-all)
- CallDetailPanel slide-out (triggered on row click)
- Pagination footer ("1-50 of 3,694,942 calls")

### Table Columns
Checkbox, Name/Number, Source, Source Number, Audio (play icon), Duration, Date, Status, Agent, Automation, Tags

### Data Model -- `CallRecord`

| Field | Type | Description |
|-------|------|-------------|
| id | String | Call record identifier |
| name | String | Caller name |
| phone | String | Caller phone number |
| location | String | Caller geo location (city, state) |
| source | String | Marketing source attribution |
| source_number | String | Tracking number dialed |
| source_name | String | Source name label |
| has_audio | bool | Whether call recording exists |
| duration | String | Call duration (MM:SS) |
| date | String | Date string |
| time | String | Time string |
| status | String | Answered / Missed / Voicemail |
| agent | String | Agent who handled the call |
| agent_initials | String | Agent initials for avatar |
| agent_color | String | Hex color for agent avatar |
| automation | String | Automation rule applied |
| tags | Vec\<String\> | Applied tags |

### Row Indicators
- Arrow color: green = Answered, red = Missed, blue = Voicemail
- Status badge: green/red/yellow
- Agent avatar circle with initials

### Actions
- Row click opens CallDetailPanel (10-tab slide-out; see [08-shared-components.md](08-shared-components.md))
- Audio play button per row
- Checkbox selection for bulk operations
- Pagination controls

### Implied Functionality
- Full call activity log with real-time or near-real-time updates
- Call recording playback
- Multi-select for bulk operations (tag, export, delete)
- Sorting by any column
- Marketing source attribution tracking
- Agent assignment tracking
- Tag-based organization

---

## 1.2 Texts Page

- **Route:** `/activities/texts`
- **Status:** Well-defined

### UI Elements
- FilterBar (shared)
- Data table with pagination ("1-50 of 12,847 texts")

### Table Columns
Checkbox, Contact (name + phone), Tracking Number, Direction, Date, Status

### Data Model -- `TextRecord`

| Field | Type |
|-------|------|
| name | &str |
| phone | &str |
| tracking_number | &str |
| direction | &str (Inbound/Outbound) |
| date | &str |
| time | &str |
| status | &str (Delivered/Failed/Pending) |

### Actions
Checkbox selection, pagination

### Implied Functionality
SMS/text message activity log with direction tracking and delivery status

---

## 1.3 Forms Page

- **Route:** `/activities/forms`
- **Status:** Well-defined

### UI Elements
- Header with title ("Form Submissions"), search input, date filter, "Export" button
- Data table with pagination ("1-50 of 1,823 submissions")

### Table Columns
Name, Phone, Email, Form Name, Source, Submitted (date/time)

### Data Model -- `FormRecord`

| Field | Type |
|-------|------|
| name | &str |
| phone | &str |
| email | &str |
| form_name | &str |
| source | &str |
| date | &str |
| time | &str |

### Actions
Export, search, date filter, pagination

### Implied Functionality
Web form submission tracking with source attribution

---

## 1.4 Chats Page

- **Route:** `/activities/chats`
- **Status:** Well-defined

### UI Elements
- FilterBar (shared)
- Data table with pagination ("1-50 of 5,412 chats")

### Table Columns
Checkbox, Visitor, Agent, Duration, Date, Status

### Data Model -- `ChatRecord`

| Field | Type |
|-------|------|
| visitor | &str |
| agent | &str |
| duration | &str |
| date | &str |
| time | &str |
| status | &str (Active/Completed/Missed) |

### Actions
Checkbox selection, pagination

### Implied Functionality
Live chat session history with agent assignment tracking

---

## 1.5 Faxes Page

- **Route:** `/activities/faxes`
- **Status:** Well-defined

### UI Elements
- FilterBar (shared)
- Data table with pagination ("1-50 of 892 faxes")

### Table Columns
Checkbox, From, To, Pages, Date, Status

### Data Model -- `FaxRecord`

| Field | Type |
|-------|------|
| from | &str |
| to | &str |
| pages | u32 |
| date | &str |
| time | &str |
| status | &str (Delivered/Failed/Sending) |

### Actions
Checkbox selection, pagination

### Implied Functionality
Fax activity log with page count and delivery status

---

## 1.6 Videos Page

- **Route:** `/activities/videos`
- **Status:** Well-defined

### UI Elements
- FilterBar (shared)
- Data table with pagination ("1-50 of 2,156 videos")

### Table Columns
Checkbox, Participant, Host, Duration, Date, Status

### Data Model -- `VideoRecord`

| Field | Type |
|-------|------|
| participant | &str |
| host | &str |
| duration | &str |
| date | &str |
| time | &str |
| status | &str (Completed/Missed/In Progress) |

### Actions
Checkbox selection, pagination

### Implied Functionality
Video call/meeting history log

---

## 1.7 Export Log Page

- **Route:** `/activities/export`
- **Status:** Well-defined

### UI Elements
- Header with title ("Export Log"), date range filter, "New Export" button
- Data table with pagination ("1-10 of 47 exports")

### Table Columns
Export Name, Type, Records, Status, Created, Download (action)

### Data Model -- `ExportRecord`

| Field | Type |
|-------|------|
| name | &str |
| export_type | &str (CSV/PDF/Excel) |
| records | &str |
| status | &str (Complete/Processing/Failed) |
| created | &str |

### Actions
"New Export" button, Download link per row, date filter, pagination

### Implied Functionality
Export history with downloadable files in CSV/PDF/Excel formats
