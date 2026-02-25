# 03 - Numbers Section

> 10 pages | All well-defined
> Source: `ui/src/sections/numbers.rs`

## Side Navigation

Two groups:
- **Management:** Buy Numbers, Tracking Numbers, Receiving Numbers, Text Numbers, Port Numbers, Call Settings
- **Dynamic Numbers:** Number Pools, Target Numbers, Tracking Sources, Tracking Code

---

## 3.1 Tracking Numbers Page

- **Route:** `/numbers/tracking`
- **Status:** Well-defined

### UI Elements
- Header with title ("254 Tracking Numbers"), tabs (Info / Released Numbers / Number Log), search input, "Buy Numbers" button
- Data table with pagination

### Table Columns
Number, Source, Call Routing (type badge), Text (icon), Target, Config, Billing Date, Active (Yes/No)

### Data Model -- `TrackingNumber`

| Field | Type |
|-------|------|
| number | &str |
| source | &str |
| routing | &str |
| routing_type | &str (Queue/Smart Router) |
| text_enabled | bool |
| target | &str |
| config | &str |
| billing_date | &str |
| active | bool |
| number_type | &str |

### Actions
"Buy Numbers" button, tab switching, Edit per row, search, pagination

### Implied Functionality
Marketing call tracking number inventory with source attribution, call forwarding configuration, routing type assignment

---

## 3.2 Buy Numbers Page

- **Route:** `/numbers/buy`
- **Status:** Well-defined

### UI Elements
- Search tabs: Local, Toll-Free, Address, Near Number
- Search input with area code placeholder
- Results table with available numbers
- Right sidebar shopping cart: selected numbers, per-number remove, subtotal, "Purchase" button

### Table Columns
Number, Type, Region, Monthly Cost, Setup Fee, Features, Actions (Add to Cart)

### Data Model -- `BuyNumber`

| Field | Type |
|-------|------|
| number | &str |
| number_type | &str |
| region | &str |
| monthly | &str |
| setup | &str |
| features | &str |

### Actions
Tab switching (search mode), "Add to Cart" per row, remove from cart, "Purchase" button, area code search

### Implied Functionality
Number provisioning/purchase flow with shopping cart. Supports local, toll-free, address-based, and proximity-based number search.

---

## 3.3 Receiving Numbers Page

- **Route:** `/numbers/receiving`
- **Status:** Well-defined

### UI Elements
- Header with title, search input, "Add Receiving Number" button
- Data table with pagination

### Table Columns
Number, Description, Tracking Numbers (count), Total Calls, Updated, Created

### Data Model -- `ReceivingNumber`

| Field | Type |
|-------|------|
| number | &str |
| description | &str |
| tracking_count | u32 |
| total_calls | &str |
| updated | &str |
| created | &str |

### Actions
"Add Receiving Number" button, search, pagination

### Implied Functionality
Destination/forwarding number management -- where tracking numbers ring to

---

## 3.4 Text Numbers Page

- **Route:** `/numbers/text`
- **Status:** Well-defined

### UI Elements
- Header with title, search input
- Dual-list picker:
  - Left panel: "Available Numbers" with search, checkbox list
  - Right panel: "Assigned Numbers" with search, checkbox list
  - Center: Add (right arrow) / Remove (left arrow) buttons

### Data Model -- `TextNumber`

| Field | Type |
|-------|------|
| number | &str |
| name | &str |
| assigned | bool |

### Actions
Select numbers in left/right lists, move between Available/Assigned, search within lists

### Implied Functionality
Assignment of numbers for text/SMS capabilities using dual-list picker pattern

---

## 3.5 Port Numbers Page

- **Route:** `/numbers/port`
- **Status:** Well-defined

### UI Elements
- Header with title ("Port a Number"), description text
- Multi-step form wizard with collapsible sections:
  - **User Details:** First Name, Last Name, Email, Phone Number
  - **Billing Details:** Address Line 1, Address Line 2, City, State (select), Zip Code, Authorized Signature
- Numbers to port: textarea for entering numbers
- "Submit Port Request" button

### Actions
Collapse/expand form sections, submit port request

### Implied Functionality
Number porting (LNP) request workflow -- transfer phone numbers from another carrier

---

## 3.6 Call Settings Page

- **Route:** `/numbers/call-settings`
- **Status:** Well-defined

### UI Elements
- Header with title ("Call Settings"), "Save" button
- Settings card with 12 toggle switches:

| Setting | Description |
|---------|-------------|
| Call Recording | Record all inbound and outbound calls |
| Whisper Message | Play a message to the agent before connecting |
| Call Announce | Announce the caller's name or number to the agent |
| Voicemail | Enable voicemail for missed calls |
| Call Screening | Screen calls before connecting to an agent |
| Simultaneous Ring | Ring multiple numbers at the same time |
| Round Robin | Distribute calls evenly across agents |
| Smart Routing | Route calls based on caller location or time of day |
| After Hours | Enable after-hours routing rules |
| Holiday Routing | Special routing for holidays |
| Overflow | Route to backup when primary is busy |
| Caller ID | Display your business caller ID on outbound calls |

### Actions
Toggle each setting on/off, "Save" button

### Implied Functionality
Global or per-account call handling preferences

---

## 3.7 Number Pools Page

- **Route:** `/numbers/pools`
- **Status:** Well-defined

### UI Elements
- Header with title, "Save" button
- Three configuration cards:
  1. **General:** Name input, Description textarea
  2. **Tracking:** Tracking Source select, "Auto-manage numbers" toggle, Target Accuracy slider (1-100%), Current numbers count
  3. **Numbers Management:** Data table (Number, Status, Calls, Added), "Add Numbers" button, Remove per row

### Actions
Save, toggle auto-manage, adjust accuracy slider, add/remove numbers from pool

### Implied Functionality
Dynamic number pool management for website visitor-level call tracking (DNI -- Dynamic Number Insertion)

---

## 3.8 Target Numbers Page

- **Route:** `/numbers/targets`
- **Status:** Well-defined

### UI Elements
- Header with title, search input, "Add Target" button
- Data table with pagination

### Table Columns
Number, Name, Priority, Concurrency Cap, Weight, Status, Actions (Edit/Remove)

### Data Model -- `TargetNumber`

| Field | Type |
|-------|------|
| number | &str |
| name | &str |
| priority | u32 |
| concurrency | &str |
| weight | u32 |
| status | &str |

### Actions
"Add Target" button, Edit/Remove per row, search, pagination

### Implied Functionality
Call distribution targets with priority routing, concurrent call limits, and weighted distribution

---

## 3.9 Tracking Sources Page

- **Route:** `/numbers/sources`
- **Status:** Well-defined

### UI Elements
- Header with title, search input, "Add Source" button
- Data table with pagination

### Table Columns
Source, Type, Numbers (count), Calls (count), Status, Updated, Actions (Edit/Remove)

### Data Model -- `TrackingSource`

| Field | Type |
|-------|------|
| name | &str |
| source_type | &str |
| numbers | u32 |
| calls | u32 |
| status | &str |
| updated | &str |

### Actions
"Add Source" button, Edit/Remove per row, search, pagination

### Implied Functionality
Marketing source/channel definitions for call attribution (Google Ads, Organic, Direct, Referral, etc.)

---

## 3.10 Tracking Code Page

- **Route:** `/numbers/code`
- **Status:** Well-defined

### UI Elements
- Header with title, "Copy Code" button
- JavaScript code snippet in `<pre><code>` block
- Platform-specific installation tabs: WordPress, Squarespace, Wix, Shopify, Custom
- "Email to Developer" section with email input and send button
- Installation instructions text

### Actions
Copy code to clipboard, select platform, email code snippet to developer

### Implied Functionality
DNI (Dynamic Number Insertion) JavaScript tag installation guide for website visitor tracking
