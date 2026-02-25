# 4iiz Numbers Section - UI Analysis & Prototype Plan

## Section Overview

The **Numbers** section is the second major area in the 4iiz left sidebar, managing all phone number acquisition, configuration, tracking, and dynamic number insertion. It is split into two subsections: **Management** (static number administration) and **Dynamic Numbers** (website visitor tracking via number pools). All pages share the same 3-column layout shell (icon nav | secondary nav | content) used across the application.

---

## Sidebar Navigation

| Section | Items |
|---------|-------|
| **MANAGEMENT** | Buy Numbers, Tracking Numbers, Receiving Numbers, Text Numbers, Port Numbers, Call Settings |
| **DYNAMIC NUMBERS** | Number Pools, Target Numbers, Tracking Sources, Tracking Code |

---

## Page Inventory (10 pages)

| # | Page | Type | Records | Key Action |
|---|------|------|---------|------------|
| 1 | Buy Numbers | Search + Purchase | ~8 results per search | Request a Number..., Buy Bulk |
| 2 | Tracking Numbers | Data Table | 254, paginated 10/page | Buy Numbers |
| 3 | Receiving Numbers | Data Table | 10 total | New Receiving Number |
| 4 | Text Numbers | Dual-list Picker | 250 available/assigned | Save Settings |
| 5 | Port Numbers | Multi-step Wizard | N/A (form) | Continue |
| 6 | Call Settings | Data Table | 2 settings | New Call Settings |
| 7 | Number Pools | Form / Wizard | N/A (form) | Save |
| 8 | Target Numbers | Data Table | 4 targets | New Target Number |
| 9 | Tracking Sources | Data Table | 46, paginated 10/page | New Tracking Source |
| 10 | Tracking Code | Installation Guide | N/A (code snippet) | Copy to Clipboard |

---

## Detailed Page Analysis

### Page 1: Buy Numbers

**Purpose**: Search and purchase phone numbers from the carrier inventory. Supports local, toll-free, address-based, and proximity-based searches.

**Breadcrumb**: Tracking Numbers > Buy

**Top bar elements**:
- "Info" link (help/documentation)
- "New Number Pool" button
- "Buy Bulk" button
- "Request a Number..." CTA button (cyan)

**Layout**: Search form at top, results table below, purchase sidebar on right.

**Country selector**: US +1 United States dropdown with "Regulations" outlined button.

**Tabs (4)**:
| Tab | State | Description |
|-----|-------|-------------|
| LOCAL NUMBER | Active (cyan underline) | Search by area code / rate center |
| TOLL-FREE | Inactive | Search toll-free prefixes (800, 888, etc.) |
| ADDRESS | Inactive | Search by street address proximity |
| NEAR NUMBER | Inactive | Search near an existing number |

**Search filters**:
- "Any" dropdown (field selector)
- Text input for search term
- "Search" button (cyan)
- "contains" dropdown (match type)
- Help icon (?)
- "Additional Filters" expandable section

**Purchase sidebar** (right panel):
- "0 numbers" count
- "$0.00 / month" running total
- Updates as numbers are added

**Results table columns**:
| Column | Type | Notes |
|--------|------|-------|
| Phone Number | Formatted + E.164 | e.g., (276) 285-5069 / +12762855069 |
| Rate Center & Features | Multi-line | City, State, ZIP + feature badges |
| Monthly Fee | Currency | All $1.26/month in sample data |
| (Add button) | Action | "+" icon to add to cart |

**Feature badges per number**:
- SMS: checkmark or X
- MMS: checkmark or X
- HIPAA: checkmark (compliance indicator)
- Fax: Yes/No
- e911: checkmark (emergency services)

**Sample data** (8 numbers shown):
| Phone Number | Rate Center | Monthly Fee |
|-------------|-------------|-------------|
| (276) 285-5069 | Bristol, VA 24201 | $1.26 |
| (276) 285-5075 | Bristol, VA 24201 | $1.26 |
| (276) 285-5076 | Bristol, VA 24201 | $1.26 |
| (276) 285-5078 | Bristol, VA 24201 | $1.26 |
| (276) 285-5079 | Bristol, VA 24201 | $1.26 |
| (276) 285-5080 | Bristol, VA 24201 | $1.26 |
| (276) 285-5081 | Bristol, VA 24201 | $1.26 |
| (276) 285-5082 | Bristol, VA 24201 | $1.26 |

---

### Page 2: Tracking Numbers

**Purpose**: Central registry of all numbers used to track advertising channel attribution. Each tracking number maps to a source (e.g., Google Ads, Facebook) and routes calls to agents or queues.

**Title**: "Tracking Numbers" with subtitle "Numbers used to track which advertising channels your customers are using"

**Top bar elements**:
- "Info" link
- "Released Numbers" link
- "Number Log" link
- "Buy Numbers" CTA button (cyan)

**Search + Pagination**: Filter icon (cyan) + Search bar + Pagination (1 2 3 ... 25 26) = 254 Tracking Numbers

**Table columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit) | Link | No | Pencil/edit icon |
| Number | Phone | Yes | Formatted US number |
| Tracking Source | Text | Yes | Advertising channel name |
| Call Routing | Text + badge | Yes | Queue or Smart Router name with type badge |
| Text Number | Text + badges | Yes | "Allow Texting" + Trust Center shield badge |
| Target Numbers | Text | Yes | Account-level or specific number with source name |
| Number Config | Text | Yes | Feature list (Inbound/Outbound Recordings, Caller ID) |
| (4 icons) | Actions | No | Calendar, email, phone, clipboard quick actions |
| Next Billing Date | Date | Yes | Format: YYYY-MM-DD |
| Active | Yes/No | Yes | Active status |
| Type | Badge | Yes | "Offsite Static" or "Onsite Dynamic" with pause icon |

**Sample data** (5 visible rows):

| Number | Tracking Source | Call Routing | Type |
|--------|----------------|-------------|------|
| (910) 991-0047 | Test source | SYSTANGO TESTING Queue | Offsite Static |
| (980) 553-2289 | Facebook Paid | Check if New Lead or Current Client Smart Router | Onsite Dynamic |
| (855) 614-1888 | Customer Service Line | Rescue Team Queue | Offsite Static |
| (919) 290-4449 | WhatsApp | Customer Service Queue (Official) Queue | Offsite Static |
| (832) 558-3313 | Facebook West Houston Office | Check if New Lead or... Smart Router | Onsite Dynamic |

**Number Config column** features: Inbound Recordings, Outbound Recordings, Caller ID (shown as comma-separated list per row).

**Billing column**: Shows renewal cost and per-minute rate.

---

### Page 3: Receiving Numbers

**Purpose**: Numbers that are ultimately dialed to reach an agent or business. These are the "destination" numbers that tracking numbers route to.

**Title**: "Receiving Numbers" with subtitle "numbers ultimately dialed to reach an agent or business"

**Top bar elements**:
- "Restore" link
- "Info" link
- "New Receiving Number" CTA button (cyan)

**Filter**: Filter icon + Search + "10 Receiving Numbers" count + Edit + Export buttons

**Table columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit) | Link | No | Pencil/edit icon |
| Receiving Number | Phone + description | Yes | Formatted number + name/description |
| Tracking Numbers | Count/link | Yes | Associated tracking numbers |
| (Speaker icon) | Action | No | Audio/greeting playback |
| Geo Routers | Text | Yes | Geographic routing rules |
| Total Calls | Number | Yes | Lifetime call count |
| Updated | Datetime | Yes | Last modification timestamp |
| Created | Datetime | Yes | Creation timestamp |

**Sample data** (10 numbers):

| Receiving Number | Description | Total Calls |
|-----------------|-------------|-------------|
| (252) 235-4100 | CASE MANAGERS QUEUE | 182 |
| (844) 707-4320 | RMS-FKM-PAYMENTS | 0 |
| (252) 351-2397 | CS QUEUE | 4,119 |
| (888) 361-3349 | SALES QUEUE | 13,228 |
| (888) 359-4517 | After Hours/Overflow Service DO NOT USE | 11,612 |
| (252) 367-9099 | LexReception DO NOT USE | 33 |
| (919) 626-1424 | Old Answering Service DO NOT USE | 0 |
| (888) 399-8387 | ANSWERHERO | 73,549 |
| (855) 563-5818 | MAIN CS LINE | 50,417 |
| (252) 235-4005 | PRIMING QUEUE | 180 |

**Per page**: 10 selector at bottom.

---

### Page 4: Text Numbers

**Purpose**: Configure which tracking numbers can send and receive text messages. Manages both inbound and outbound texting capabilities.

**Title**: "Text Message Numbers" with subtitle "choose which numbers can send and receive text messages"

**Top tabs**: "Incoming Messages" (active, cyan) | "Outgoing Messages"

**Section 1: Allow Text Messages**
- "refresh list" link at top right
- Dual-list picker UI:
  - Left panel: "Search available Tracking Numbers" (9 available out of 250)
  - Right panel: "Assigned Tracking Numbers" (241 assigned out of 250)
  - Arrow transfer buttons between lists (single and bulk)
  - "select all" / "unselect all" links with counts
- "Save Settings" button (cyan)

**Section 2: Outgoing Long Text Messages**
- Same dual-list picker pattern:
  - Left: Available (7 out of 250)
  - Right: Assigned (243 out of 250)
- Info text explaining SMS segmentation: 160-char standard SMS, up to 1600 chars (10 message segments), billing per segment
- "Save Settings" button (cyan)

---

### Page 5: Port Numbers

**Purpose**: Multi-step wizard for porting existing phone numbers from another carrier to 4iiz. Collects user details, billing information, and number list for the porting request.

**Breadcrumb**: Port Numbers > New > General

**Top bar elements**:
- "Info" link
- "Give us feedback!" link

**Name field** at top: Text input with help text "Provide a friendly name to remember this order for future reference"

**Multi-step wizard** with collapsible sections:

**Section 1: User Details** (expanded):
| Field | Type | Required | Notes |
|-------|------|----------|-------|
| First Name | Text input | Yes (*) | |
| Last Name | Text input | Yes (*) | |
| Business Name | Text input | Yes (*) | Red border when empty (validation) |
| Service Account Number | Text input | No | Carrier account reference |
| Account PIN Number | Text input | No | Carrier PIN for verification |

**Section 2: Billing Details** (collapsed):
- Visible at bottom as collapsible accordion section
- Presumably contains billing address and payment details

**Action**: "Continue" button (cyan) to proceed to next step.

---

### Page 6: Call Settings

**Purpose**: Manage reusable call configuration templates that can be applied to groups of tracking numbers. Controls greeting messages, recording, transcription, caller ID, and spam detection.

**Title**: "Call Settings" with subtitle "Adjust common settings for groups of tracking numbers"

**Top bar elements**:
- "Restore" link
- "Info" link
- "New Call Settings" CTA button (cyan)

**Search**: Search bar + "2 Call Settings" count

**Table columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit) | Link | No | Pencil/edit icon |
| Name | Text | Yes | Setting template name |
| Default | Checkbox | Yes | Whether this is the default config |
| Greeting Message | Checkbox | Yes | Greeting audio enabled |
| Whisper Message | Checkbox | Yes | Agent whisper enabled |
| Inbound Recordings | Checkbox | Yes | Record inbound calls |
| Outbound Recordings | Checkbox | Yes | Record outbound calls |
| Transcribe conversations | Checkbox | Yes | Transcription enabled |
| Caller ID | Checkbox | Yes | Caller ID enabled |
| Enhanced Caller ID | Checkbox | Yes | Enhanced lookup enabled |
| Override Caller ID | Checkbox | Yes | Override caller ID display |
| Spam Detection | Checkbox | Yes | Spam filter enabled |
| Updated | Datetime | Yes | Last modification |
| Created | Datetime | Yes | Creation date |

**Sample data** (2 settings):

| Name | Default | Greeting | Whisper | Inbound Rec | Outbound Rec | Transcribe | Caller ID |
|------|---------|----------|---------|-------------|-------------|-----------|-----------|
| Account Level | Yes | Yes | No | Yes | Yes | No | Yes |
| No Call Recording | Yes | No | No | Yes | No | No | Yes |

**Per page**: 10 selector at bottom.

---

### Page 7: Number Pools

**Purpose**: Configure dynamic number pools for website visitor tracking. Pools automatically manage a set of tracking numbers to replace target numbers on websites, enabling per-visitor attribution.

**Breadcrumb**: Number Pools > New > General

**Form sections**:

**General card**:
| Field | Type | Required | Notes |
|-------|------|----------|-------|
| Name | Text input | Yes | Red speech bubble validation when empty |
| Description | Textarea | No | Optional pool description |

**Tracking card**:
| Field | Type | Default | Notes |
|-------|------|---------|-------|
| Custom tracking source | Toggle | OFF | Use custom vs. auto-detected source |
| Visitor Type | Dropdown | "All Visitors" | With PPC regex help tooltip |
| Estimated Visitor Count | Number | 1 | Drives pool size recommendation |

**Numbers Management card**:
| Field | Type | Default | Notes |
|-------|------|---------|-------|
| Auto management | Toggle | ON (green) | Automatically adjust pool size |
| Target Accuracy | Slider | 99% | Desired visitor tracking accuracy |
| Recommendation panel | Info | Dynamic | "Based on your current visitor count, we recommend N tracking number(s)..." |
| Cost display | Text | $1.26/mo | Per-number monthly cost |
| Country code | Dropdown | US +1 | Country selector |
| Number type | Radio | Local | Local or Toll Free |
| Area code | Dropdown | 205 | Area code for local numbers |
| Allow Overlay | Toggle | OFF | Allow number overlay behavior |

---

### Page 8: Target Numbers

**Purpose**: Define the phone numbers displayed on a website that should be dynamically replaced with tracking numbers. Each target number maps to multiple tracking numbers for swap-based attribution.

**Title**: "Target Numbers" with subtitle "Numbers to replace with tracking numbers on your website"

**Top bar elements**:
- "Restore" link
- "Info" link
- "New Target Number" CTA button (cyan)

**Search**: Search bar + "4 Target Numbers" count

**Table columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit) | Link | No | Pencil/edit icon |
| Name | Text + description | Yes | Number + name label |
| Type | Badge | Yes | Always "Phone Match" in sample data |
| Tracking Numbers | Text list | Yes | Comma-separated E.164 numbers, with "+N more" overflow |
| Updated | Datetime | Yes | Last modification |
| Created | Datetime | Yes | Creation date |

**Sample data** (4 targets):

| Name | Description | Type | Tracking Numbers |
|------|-------------|------|-----------------|
| (252) 351-2397 | Description | Phone Match | Long list of E.164 numbers |
| (888) 361-3349 | Website | Phone Match | Many numbers + "9 more" |
| (888) 359-4517 | Google Adwords NC | Phone Match | Many numbers |
| (855) 563-5818 | Google Adwords | Phone Match | Many numbers + "15 more" |

---

### Page 9: Tracking Sources

**Purpose**: Define and manage the advertising channels being tracked (e.g., Facebook, Google Organic, Email Marketing). Each source is assigned tracking numbers and configured for attribution rules.

**Title**: "Tracking Sources" with subtitle "Advertising channels you want to track, such as Facebook or Google Organic"

**Top bar elements**:
- "Restore" link
- "Info" link
- "Export" button
- "New Tracking Source" CTA button (cyan)

**Search + Pagination**: Search bar + Pagination (1 2 3 4 5) + "46 Tracking Sources" count

**Table columns**:
| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| (Edit) | Link | No | Pencil/edit icon |
| Tracking Source Name | Text | Yes | Source channel name |
| Type | Badge | Yes | "Onsite Dynamic" or "Offsite Static" |
| Position | Number | Yes | Display/priority order |
| Numbers | Link | Yes | "Assigned" link to view associated numbers |
| Last Touch | Checkbox | Yes | Whether last-touch attribution applies |
| Geo | Text | Yes | Geographic targeting |
| Exclusions | Text | Yes | Exclusion rules |
| Updated | Datetime | Yes | Last modification |
| Created | Datetime | Yes | Creation date |

**Sample data** (first page of 46):

| Tracking Source Name | Type | Numbers | Last Touch |
|---------------------|------|---------|------------|
| Direct | Onsite Dynamic | 13 Assigned | No |
| Woosender | Offsite Static | Assigned | Yes |
| Email Marketing | Offsite Static | Assigned | No |
| Newsletter | Offsite Static | Assigned | No |
| WhatsApp Organic | Offsite Static | Assigned | No |
| YouTube Ads | Offsite Static | Assigned | No |
| Yelp | Offsite Static | Assigned | No |
| TikTok Organic | Offsite Static | Assigned | No |
| TikTok Paid | Offsite Static | Assigned | No |
| Instagram Organic | Offsite Static | Assigned | No |

---

### Page 10: Tracking Code

**Purpose**: Provide the JavaScript tracking code snippet for dynamic number insertion on websites. Includes installation instructions for various platforms and testing tools.

**Breadcrumb**: Tracking Code > Tracking Code Installation

**Top bar**: "Refresh Tracking Code" button (cyan outlined)

**Main info panel**: Explanation of tracking code purpose for dynamic number insertion on websites.

**Important Setup Notes** (3 bullet points):
1. Conflicts with other tracking scripts
2. Hardcoded numbers must match target numbers exactly
3. Embedded elements (iframes) require separate installation

**Tracking Code Script**:
```html
<script async src="//155169.tctm.co/t.js"></script>
```
- Displayed in a code box with monospace font
- "Copy to Clipboard" button (cyan)

**Email Developer section**:
- Email input field
- "Send Instructions" button (sends setup instructions to developer)

**Tabs (3)**:
| Tab | State | Description |
|-----|-------|-------------|
| STANDARD | Active | Platform-specific installation guides |
| DEVELOPER RESOURCES | Inactive | API and advanced integration docs |
| TESTING | Inactive | Testing and validation tools |

**Website Builder logos** (under STANDARD tab):
- AMP
- Google Tag Manager
- Magento
- Wix
- WordPress

**"Not Using One of These Platforms?"**: Manual installation instructions with step-by-step guide.

**"Advanced Options"**: Expandable section for advanced configuration.

---

## Shared UI Patterns

### Pattern: Data Table Pages
Pages 2, 3, 6, 8, 9 follow the standard template:
1. Title + subtitle + CTA button in top bar
2. Search bar + pagination (or count)
3. Sortable column headers with hover sort indicators
4. Rows with action link/icon (Edit/pencil) at left
5. Bottom pagination where applicable

### Pattern: Form/Wizard Pages
Pages 1, 5, 7 use form-based layouts:
1. Breadcrumb navigation at top
2. Card-based form sections with labels and inputs
3. Collapsible accordion sections for progressive disclosure
4. Required field indicators (red asterisk or red border)
5. CTA button (cyan) at bottom of each section

### Pattern: Dual-list Picker
Page 4 (Text Numbers) uses a transfer-list pattern:
1. Two side-by-side panels (available | assigned)
2. Arrow buttons to transfer items between panels
3. Search within each panel
4. "select all" / "unselect all" bulk actions
5. Count displays (N/Total format)

### Pattern: Pagination
- Page number buttons with active state (cyan bold text)
- Ellipsis (...) for large page ranges
- Arrow navigation buttons
- Total count display ("254 Tracking Numbers", "46 Tracking Sources")
- "Per page" selector on some pages (10, 25, 50, 100)

### Pattern: Action Buttons
- **Cyan filled buttons**: Buy Numbers, New Receiving Number, New Call Settings, etc. (primary CTAs)
- **Cyan outlined buttons**: Regulations, Refresh Tracking Code (secondary actions)
- **Cyan text links**: Edit, Info, Restore, Released Numbers, Number Log (tertiary actions)
- **Gray text links**: Help, Give us feedback (meta actions)

### Pattern: Feature Badges
- Checkmark badges for enabled features (SMS, MMS, HIPAA, e911)
- "No" text for disabled features (Fax: No)
- Trust Center shield icon on text-enabled numbers
- Type badges: "Offsite Static" (gray), "Onsite Dynamic" (with pause icon)

---

## Component Mapping to DaisyUI

| 4iiz Component | DaisyUI Equivalent |
|----------------|-------------------|
| Data table | `table` or CSS grid with `col-header` classes |
| Pagination | `join` + `btn btn-xs` group |
| Search bar | `input input-bordered` + search icon in `join` |
| CTA button | `btn bg-iiz-cyan text-white` |
| Outlined button | `btn btn-outline` with cyan border |
| Tab bar | `tabs` + `tab tab-bordered` or custom underline tabs |
| Toggle switch | `toggle toggle-sm` with green active state |
| Slider | `range range-sm` |
| Dropdown/select | `select select-bordered` |
| Radio buttons | `radio radio-sm` |
| Card/form section | `card bg-base-100 shadow` with `card-body` |
| Accordion section | `collapse collapse-arrow` |
| Dual-list picker | Two `card` panels with `btn btn-sm` transfer arrows |
| Code block | `mockup-code` or `bg-gray-900 text-green-400 font-mono` |
| Badge | `badge badge-sm` or inline `tag-badge` |
| Modal | `modal` with `modal-box` |
| Breadcrumb | `breadcrumbs text-sm` |
| Tooltip/help | `tooltip` with `?` icon |
| Per-page dropdown | `select select-sm select-bordered` |
| Sort indicator | Unicode arrow in `col-header` |
| Feature checkmark | Green checkmark SVG or `text-green-500` |
| Feature X | Red X SVG or `text-red-500` |

---

## Prototype

**File**: `.UI-Numbers/prototype/numbers.html`

Covers all 10 pages with Alpine.js state switching (`numbersPage`):
- `buy` - Buy Numbers search + results with purchase sidebar
- `tracking` - Tracking Numbers table (254 items, paginated)
- `receiving` - Receiving Numbers table (10 items)
- `text` - Text Message Numbers dual-list picker
- `port` - Port Numbers multi-step wizard
- `callsettings` - Call Settings table (2 items)
- `pools` - Number Pools form
- `targets` - Target Numbers table (4 items)
- `sources` - Tracking Sources table (46 items, paginated)
- `code` - Tracking Code installation guide

All sample data from screenshots included.

---

## Observations for Production Implementation

1. **Number provisioning is carrier-dependent**: The Buy Numbers page queries carrier inventory APIs (likely Telnyx/Twilio/Bandwidth). Production needs real-time availability checks, rate center lookups, and regulatory compliance per jurisdiction.

2. **Tracking number attribution is the core product**: The entire Numbers section revolves around mapping advertising channels to phone numbers for call attribution. This is the revenue-generating feature that justifies the platform.

3. **Dynamic number insertion requires JavaScript on client sites**: The Tracking Code (Page 10) deploys a JS snippet to customer websites. Production must handle high-traffic CDN delivery, DOM manipulation, cookie/session management for visitor tracking, and graceful fallback when pools are exhausted.

4. **Number pools need real-time accuracy management**: The auto-management toggle and 99% accuracy target in Number Pools mean the system must dynamically provision/release numbers based on concurrent visitor counts. Under-provisioning loses attribution data; over-provisioning wastes money.

5. **Port number management is a multi-day workflow**: Porting numbers between carriers involves LOA (Letter of Authorization) generation, carrier coordination, and scheduled cutover dates. The wizard UI is just the intake form; production needs status tracking, document upload, and carrier API integration.

6. **Call Settings are hierarchical templates**: The "Account Level" default setting and per-number overrides create an inheritance model. Production must resolve which settings apply at call time (number-specific > account default).

7. **Text number management has regulatory implications**: The dual-list picker for text enablement must integrate with carrier 10DLC registration, A2P campaign compliance, and TCPA opt-out handling.

8. **Tracking Sources drive the entire attribution model**: The 46 sources (Direct, Facebook Paid, Google Organic, etc.) map to UTM parameters, referrer headers, and manual overrides. Production needs robust source detection algorithms and last-touch vs. first-touch attribution logic.

9. **Billing integration is tightly coupled**: Monthly fees ($1.26/number), per-minute rates, and billing dates shown throughout the UI indicate real-time metered billing. Production needs usage tracking, invoice generation, and payment processing.

10. **Soft delete + Restore pattern**: Multiple pages (Receiving Numbers, Call Settings, Target Numbers, Tracking Sources) have "Restore" actions, indicating soft-delete with time-limited recovery capability.
