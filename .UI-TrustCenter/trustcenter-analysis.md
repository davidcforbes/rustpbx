# 4iiz Trust Center Section - UI Analysis & Prototype Plan

## Section Overview

The **Trust Center** section is accessed from the left icon nav and manages regulatory compliance for US outbound communications (A2P 10DLC text messaging, STIR/SHAKEN voice authentication, Caller ID CNAM) and global compliance (international requirements, applications, addresses). The layout uses the same 3-column shell as other sections but replaces data tables with registration/compliance cards.

---

## Sidebar Navigation

The Trust Center secondary nav has two subsections:

| Group | Items |
|-------|-------|
| US OUTBOUND COMPLIANCE | Business/Contact Information (default), Local Text Messaging, Toll Free Text Messaging, Voice Registration, Caller ID |
| GLOBAL COMPLIANCE | Requirements, Applications, Addresses |

---

## Page Inventory

| # | Page | State Variable | Key Content |
|---|------|---------------|-------------|
| 1 | Business/Contact Information | `business` | Business info card + all compliance sections |
| 2 | Local Text Messaging | `localtext` | Campaign management (placeholder) |
| 3 | Toll Free Text Messaging | `tollfreetext` | Toll-free registration (placeholder) |
| 4 | Voice Registration | `voicereg` | STIR/SHAKEN registration (placeholder) |
| 5 | Caller ID | `callerid` | CNAM management (placeholder) |
| 6 | Requirements | `requirements` | Global compliance requirements (placeholder) |
| 7 | Applications | `applications` | Global compliance applications (placeholder) |
| 8 | Addresses | `addresses` | Global compliance addresses (placeholder) |

---

## Detailed Page Analysis

### Page 1: Business/Contact Information (Main Page)

**Purpose**: Central compliance dashboard showing business registration details and status of all US outbound compliance registrations.

**Breadcrumb**: Trust Center > View > Caller ID (CNAM)

**Top bar**: Agency View link, Info link (top right)

This page displays 5 stacked cards:

---

#### Section 1: Business/Contact Information Card

**Purpose**: Read-only display of registered business entity information used for compliance filings.

**Fields**:
| Field | Value | Type |
|-------|-------|------|
| Legal Business Name | Diener Law, PA | Text |
| Address | 3333 Jaeckle Dr, Suite 130, Wilmington, NC 28403, US | Multi-line text |
| Company Type | Private | Text |
| Business Type | Corporation | Text |
| Industry Type | LEGAL | Text (uppercase) |
| EIN Number | 271813765 | Numeric |
| URL | https://dienerlaw.net | Link |

**Actions**:
- "Trust Center Contact" dropdown (top right of card) - selects the compliance contact person

**UX note**: All fields are read-only display, not editable inline. Business information is registered through a separate onboarding flow and displayed here for reference.

---

#### Section 2: Local Text Messaging Campaigns Card

**Purpose**: Register and manage A2P 10DLC campaigns required for sending outbound text messages from local numbers to US recipients.

**Header**: Title + subtitle with "A2P 10DLC" educational link

**Controls**:
- Toggle: "Show expired campaigns" (OFF by default)

**Campaign Table**:
| Column | Type | Notes |
|--------|------|-------|
| (View button) | Button | Eye icon or "View" link |
| Name | Text | Campaign display name |
| Created | Date | Format: YYYY-MM-DD |
| Status | Badge | Green checkmark + "Approved" text |
| Assigned Numbers | Status + count | Green check + fraction (e.g., 116/400) with edit pencil |
| Cost | Currency | Monthly recurring (e.g., $1.5/mo) |
| Carrier | Text | Carrier name (e.g., Carrier A) |

**Sample Data**:
| Name | Created | Status | Assigned Numbers | Cost | Carrier |
|------|---------|--------|-----------------|------|---------|
| General Campaign | 2023-05-22 | Approved | 116/400 | $1.5/mo | Carrier A |
| New Campaign | 2023-11-03 | Approved | 59/400 | $1.5/mo | Carrier A |

**Summary bar**: "Local Text Registration Status: approved" | "Campaigns: 2/50"

**Action**: "Add Campaigns" button (cyan)

---

#### Section 3: Toll-Free Text Messaging Campaign Card

**Purpose**: Register toll-free numbers for sending text messages to US and Canadian phone numbers (Toll-Free A2P).

**Content**:
- Title with educational subtitle
- Description: "Register your business with carriers to send text messages from toll-free numbers. Registration is free."

**Action**: "Manage Toll-Free Messaging" button (cyan)

---

#### Section 4: Outbound Calling Verification Card

**Purpose**: Display STIR/SHAKEN registration status for outbound calls to reduce call blocking and spam labeling.

**Content**:
- Title with educational subtitle
- Status badge: "Status: APPROVED" (green text, positioned top right)
- Description: "Your business is verified for outbound calling to U.S. numbers, reducing call blocking and spam labeling by carriers. To further improve answer rates, register your numbers with the Free Caller Registry."

**UX note**: This card is informational when status is APPROVED. The "Free Caller Registry" text is a link for further optimization.

---

#### Section 5: Caller ID (CNAM) Card

**Purpose**: Manage custom business name display for outbound calls from tracking numbers.

**Content**:
- Title with "Learn more" link
- Description: "Display a custom business name when placing outbound calls from your tracking numbers. This can improve answer rates on supported networks. Caller ID display depends on the recipient's carrier and device settings and is not guaranteed on every call."

**Action**: "Manage Caller ID" button (cyan)

---

## Shared UI Patterns

### Pattern: Compliance Card Layout
All 5 sections on the Business/Contact Information page use the same card pattern:
1. Card with white background, subtle border, rounded corners
2. Title (bold) + subtitle or educational link
3. Body content (description text, form fields, or data table)
4. Action button (cyan) or status badge at bottom/right

### Pattern: Status Indicators
- **Green checkmark + text**: Approved status (campaigns, voice registration)
- **Fraction display**: Capacity tracking (116/400 numbers assigned)
- **Badge text**: "Status: APPROVED" with green coloring
- **Toggle switch**: Filter control (show expired campaigns)

### Pattern: Breadcrumb Navigation
- Displayed at top of content area
- Format: "Trust Center > View > [Current Section]"
- Different from data-table pages which use title + subtitle

### Pattern: Agency View
- "Agency View" link in top bar suggests multi-tenant/reseller capability
- Switches between account-level and agency-level views

---

## Component Mapping to DaisyUI

| 4iiz Component | DaisyUI Equivalent |
|----------------|-------------------|
| Compliance card | `card` with `card-body` |
| Status badge (approved) | `badge badge-success` or custom green text |
| Campaign table | `table table-zebra` or CSS grid |
| Toggle switch | `toggle` |
| CTA button | `btn bg-iiz-cyan text-white` |
| Breadcrumb | `breadcrumbs text-sm` |
| Business info fields | Definition list with `grid grid-cols-2` |
| Capacity fraction | Custom `text-sm` with progress indicator |
| Agency View link | `link text-iiz-cyan` |
| Section divider | `divider` or border-bottom |
| Info/Learn more links | `link link-primary text-sm` with icon |
| Contact dropdown | `select select-bordered` or `dropdown` |
| Card title | `card-title text-lg font-semibold` |
| Subtitle text | `text-sm text-gray-500` |

---

## Observations for Production Implementation

1. **Business information is externally registered**: The business/contact info card is read-only, indicating data is populated through an onboarding or registration workflow, not edited from this page.

2. **A2P 10DLC campaign management is critical**: Campaign registration is required by carriers for sending local text messages. The 116/400 and 59/400 capacity fractions indicate per-campaign number limits that must be tracked and enforced.

3. **Campaign capacity limits**: Each campaign has a maximum number assignment (400 in sample data). The edit pencil next to the fraction suggests numbers can be reassigned between campaigns. Total campaign limit is 50.

4. **STIR/SHAKEN is informational once approved**: The voice registration section becomes a status display once approved. The actionable state would show a registration form or pending status.

5. **Toll-free messaging is separate from local**: Toll-free A2P has a different registration process from 10DLC, handled through a separate management page.

6. **CNAM is carrier-dependent**: The disclaimer about carrier and device settings acknowledges that Caller ID display is best-effort, not guaranteed.

7. **Global Compliance section suggests international expansion**: The Requirements, Applications, and Addresses subsections indicate support for international regulatory requirements beyond US-only compliance.

8. **Agency View indicates multi-tenant architecture**: The ability to switch between agency and account views suggests a reseller/white-label model where agencies manage multiple client accounts.

9. **Regulatory compliance is evolving**: The multiple registration types (10DLC, STIR/SHAKEN, CNAM, toll-free A2P) reflect the current US telecom regulatory landscape. New requirements may need to be added as regulations change.

---

## Prototype

**File**: `.UI-TrustCenter/prototype/trustcenter.html`

Covers the main Business/Contact Information page with all 5 compliance cards, plus placeholder pages for the remaining 7 sidebar items. Uses Alpine.js state variable `trustPage` switching between: `business`, `localtext`, `tollfreetext`, `voicereg`, `callerid`, `requirements`, `applications`, `addresses`.

All sample data from screenshots included.
