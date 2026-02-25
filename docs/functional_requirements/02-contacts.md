# 02 - Contacts Section

> 4 pages | All well-defined
> Source: `ui/src/sections/contacts.rs`
> Navigation: Appears under Activities side nav as a "Contacts" sub-group

---

## 2.1 Contact Lists Page

- **Route:** `/contacts/lists`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs (Activities > Contacts > Lists), "New List" button
- Search input
- Data table with pagination ("1-10 of 23 lists")

### Table Columns
Name, Description, Members (count), Updated, Created, Actions (Edit/Remove)

### Data Model -- `ContactList`

| Field | Type |
|-------|------|
| name | &str |
| description | &str |
| members | u32 |
| updated | &str |
| created | &str |

### Actions
"New List" button, Edit/Remove per row, search, pagination

### Implied Functionality
Named contact list management for segmentation, bulk messaging, and targeting

---

## 2.2 Blocked Numbers Page

- **Route:** `/contacts/blocked`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "New Blocked Number" button, Restore/Import/Info links
- Search input
- Data table with pagination ("1-10 of 30 blocked numbers")

### Table Columns
Blocked Number, CNAM, Calls Blocked (count), Last Blocked (date), Updated, Created, Actions (Edit/Remove)

### Data Model -- `BlockedNumber`

| Field | Type |
|-------|------|
| number | &str |
| cnam | &str |
| calls_blocked | u32 |
| last_blocked | &str |
| updated | &str |
| created | &str |

### Actions
"New Blocked Number" button, Restore, Import, Edit/Remove per row, search, pagination

### Implied Functionality
Number blocking with call-count statistics and CNAM lookup display

---

## 2.3 Do Not Call Page

- **Route:** `/contacts/do-not-call`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "Add Number" button
- Search input
- Data table with pagination ("1-10 of 89 entries")

### Table Columns
Number, Added By, Created At, Actions (Remove)

### Data Model -- `DncEntry`

| Field | Type |
|-------|------|
| number | &str |
| added_by | &str |
| created_at | &str |

### Actions
"Add Number" button, Remove per row, search, pagination

### Implied Functionality
DNC list management for outbound call compliance (TCPA)

---

## 2.4 Do Not Text Page

- **Route:** `/contacts/do-not-text`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "Add Number" button
- Search input
- Data table with pagination ("1-10 of 45 entries")

### Table Columns
Number, E.164, Rejected Count, Last Rejected, Added By, Created At, Actions (Remove)

### Data Model -- `DntEntry`

| Field | Type |
|-------|------|
| number | &str |
| e164 | &str |
| rejected_count | u32 |
| last_rejected | &str |
| added_by | &str |
| created_at | &str |

### Actions
"Add Number" button, Remove per row, search, pagination

### Implied Functionality
Do-not-text list with E.164 normalization and rejection counting
