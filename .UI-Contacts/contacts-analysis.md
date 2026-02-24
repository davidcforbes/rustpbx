# 4iiz Contacts Section - UI Analysis & Prototype Plan

## Section Overview

The **Contacts** sub-section lives under the Activities area in the left sidebar. It manages contact lists, blocked callers, and regulatory compliance lists (Do Not Call / Do Not Text). All 4 pages share the same layout shell as the Activity Logs pages but display data-management tables rather than activity feeds.

---

## Page Inventory (5 screenshots, 4 pages)

| # | Page | Screenshot | Records | Key Action |
|---|------|-----------|---------|------------|
| 1 | Contact Lists | `Contacts - Lists.jpg` | 838 lists, paginated 10/page | New Lists, Edit, Restore |
| 1b | List Detail | `Contacts - Lists - Detail.jpg` | Single list edit form | Save Changes, Delete Lists |
| 2 | Blocked Numbers | `Contacts - Blocked Numbers.jpg` | 30 blocked numbers | New Blocked Number, Import |
| 3 | Do Not Call List | `Contacts - Do Not Call List.jpg` | 174 numbers, 2 pages | New Numbers, Remove |
| 4 | Do Not Text List | `Contacts - Do Not Text List.jpg` | 5,910 numbers, 60 pages | New Do Not Text, Remove |

---

## Detailed Page Analysis

### Page 1: Contact Lists

**Purpose**: Manage per-user/per-agent contact lists used for segmentation, automation triggers, and CRM syncing.

**Layout**: Data table with search + pagination above and below.

**Columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit action) | Link | No | Cyan "Edit" link opens detail modal |
| Name | Text + description | Yes | Bold list name + description with pencil edit icon |
| Members | Number link | Yes | Cyan clickable count (drills into list members) |
| Updated | Datetime | Yes | Format: `YYYY-MM-DD HH:MM:SS AM/PM` |
| Created | Datetime | Yes | Same format |

**Top bar elements**:
- "Restore" link (for recovering deleted lists)
- "Info" link (help/documentation)
- "New Lists" CTA button (cyan)

**Pagination**: Full pagination with page numbers, ellipsis for large ranges, "Per page: 10" selector at bottom-right.

**Data pattern**: Lists are auto-generated per agent (`User {ID} Activity Contacts`) with email-based descriptions. Member counts range from 403 to 4,930.

---

### Page 1b: List Detail (Edit Modal)

**Purpose**: Edit a contact list's name and description.

**Layout**: Simple card/modal with two fields.

**Fields**:
- **Name** (text input): Pre-filled with list name
- **Description** (optional, text input): Pre-filled with description string

**Actions**:
- "Save Changes" (blue/slate button, left)
- "Delete Lists..." (outlined orange button, right) - destructive action with confirmation implied by "..."

**UX note**: This is a lightweight edit form, not a full-page editor. In the screenshot it appears as a separate view, but functionally it should be a modal overlay.

---

### Page 2: Blocked Numbers

**Purpose**: Manage caller IDs that are prevented from reaching tracking numbers. Used to block spam, robocalls, or unwanted callers.

**Columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit action) | Icon | No | Pencil icon for editing |
| Blocked Number | Phone | Yes | Formatted US number |
| CNAM | Text | Yes | Caller Name lookup - usually empty |
| Calls blocked | Number | Yes | Count of blocked attempts (0-256 range) |
| Last blocked | Datetime | Yes | When last blocked call occurred, or empty |
| Updated | Datetime | Yes | |
| Created | Datetime | Yes | |

**Top bar elements**:
- "Restore" link
- "Import" button (bulk upload of blocked numbers)
- "Info" link
- "New Blocked Number" CTA button (cyan)

**No pagination** (only 30 numbers, fits on one page). Search bar + count display.

**Data pattern**: Most numbers have 0 blocked calls. One entry (919) 553-4064 has 256 blocked calls, suggesting active spam number. Dates span 2019-2026.

---

### Page 3: Do Not Call List (DNC)

**Purpose**: Regulatory compliance list - numbers that agents are prohibited from calling. Required by TCPA/FCC regulations.

**Columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Remove action) | Link | No | Cyan "Remove" link |
| Number | Phone | Yes | Formatted US number |
| Added By | Email | Yes | User who added the restriction |
| Created at | Datetime | Yes | UTC timestamp |

**Top bar elements**:
- "New Numbers" CTA button (cyan)

**Pagination**: 2 pages, 174 total DNC entries.

**Data pattern**: Added by specific staff members (luis.barba, carlos.diaz, armando, ramon.acosta, rafael.martindelcampo, ruben.rodriguez). Recent additions cluster in Jan-Feb 2026.

**Compliance note**: This is a critical regulatory feature. The DNC list must be checked before any outbound call is placed.

---

### Page 4: Do Not Text List (DNT)

**Purpose**: Numbers that have opted out of receiving text messages. Required by TCPA for SMS compliance (STOP keyword opt-outs).

**Columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Remove action) | Link | No | Cyan "Remove" link |
| Number | Phone (dual) | Yes | Formatted number + E.164 format underneath |
| Rejected count | Number | Yes | How many texts were rejected |
| Last rejected | Datetime/text | Yes | "Never" or datetime of last rejection |
| Added by | Text | Yes | Usually "system" (auto-added from STOP messages) |
| Created at | Datetime | Yes | UTC timestamp |

**Top bar elements**:
- "New Do Not Text" CTA button (cyan)

**Pagination**: 60 pages, 5,910 total DNT entries.

**Key differences from DNC list**:
- Shows both formatted and E.164 number formats
- Tracks rejection count and last rejection date
- "Added by" is mostly "system" (automated STOP keyword processing) vs. manual staff additions in DNC
- Much larger volume (5,910 vs 174) since text opt-outs accumulate faster

---

## Shared UI Patterns

### Pattern: Data Table Pages
All 4 Contacts pages follow the same template:
1. Title + subtitle + CTA button in top bar
2. Search bar + pagination
3. Sortable column headers
4. Rows with action link (Edit/Remove/pencil) at left
5. Bottom pagination (some pages)

### Pattern: Pagination
- Page number buttons with active state (cyan background)
- Ellipsis (...) for large ranges
- Arrow navigation (← →)
- Total count display ("838 Lists", "174 Do Not Calls", etc.)
- "Per page" selector only on Lists page

### Pattern: Action Buttons
- **Cyan text links**: Edit, Remove (inline row actions)
- **Cyan filled buttons**: New Lists, New Blocked Number, New Numbers, New Do Not Text
- **Gray text links**: Restore, Import, Info (secondary actions in top bar)

---

## Component Mapping to DaisyUI

| 4iiz Component | DaisyUI Equivalent |
|----------------|-------------------|
| Data table | `table` or CSS grid with `col-header` classes |
| Pagination | `join` + `btn btn-xs` group |
| Search bar | `input input-bordered` + search icon button |
| CTA button | `btn bg-iiz-cyan text-white` |
| Edit modal | `modal` with `modal-box` |
| Remove link | `link link-primary text-sm` |
| Per-page dropdown | `select select-sm select-bordered` |
| Sort indicator | Unicode arrow in `col-header` |

---

## Prototype

**File**: `.UI-Contacts/prototype/contacts.html` (646 lines)

Covers all 4 pages with Alpine.js state switching (`contactsPage`):
- `lists` - Contact Lists with edit modal
- `blocked` - Blocked Numbers table
- `dncall` - Do Not Call List
- `dntext` - Do Not Text List

All sample data from screenshots included.

---

## Observations for Production Implementation

1. **Pagination is server-side**: The large record counts (838 lists, 5,910 DNT entries) mean the prototype shows static sample data but production will need API-driven pagination.

2. **DNC/DNT are compliance-critical**: These lists must integrate with outbound call/text engines to block prohibited communications in real-time.

3. **STOP keyword automation**: The DNT list's "system" added-by and automatic rejection tracking suggests an automated SMS opt-out handler that processes STOP/UNSUBSCRIBE keywords.

4. **Import/Export**: Blocked Numbers has an "Import" action, suggesting bulk upload capability (CSV). Other lists may need this too.

5. **Soft delete + Restore**: The "Restore" action on Lists and Blocked Numbers indicates soft-delete with recovery capability.

6. **Member drill-through**: The cyan member count on Lists is clickable, implying a sub-page or modal showing individual contacts in that list.
