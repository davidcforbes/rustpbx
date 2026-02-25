# 07 - Trust Center Section

> 8 pages | All well-defined
> Source: `ui/src/sections/trust_center.rs`

## Side Navigation

Two groups:
- **US Outbound Compliance:** Business/Contact Info, Local Text Messaging, Toll Free Text Messaging, Voice Registration, Caller ID
- **Global Compliance:** Requirements, Applications, Addresses

---

## US Outbound Compliance

### 7.1 Business/Contact Info Page

- **Route:** `/trust-center/business`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, Info link
- 5 content cards:

**1. Business Information** (read-only data grid):
| Field | Value |
|-------|-------|
| Legal Business Name | Diener Law, PA |
| DBA | Diener Law |
| EIN | 46-2XXXXXX |
| Industry | Legal Services |
| Address | 100 Europa Dr Ste 525, Chapel Hill, NC 27517 |
| Phone | (919) 436-4235 |
| Email | admin@dienerlaw.com |
| Website | www.dienerlaw.com |

**2. Contact Information** (read-only): Name, Title, Phone, Email

**3. Local Text Messaging Campaigns:** table (Campaign Name, Use Case, Status, Submitted) + "Add Campaign" button

**4. Toll-Free Text Messaging:** status indicator + "Register" button

**5. Outbound Calling Verification:** STIR/SHAKEN status (APPROVED badge), EIN, Last Verified date

### Actions
Add Campaign, Register for toll-free, Info link

### Implied Functionality
Central compliance dashboard aggregating business registration, A2P messaging campaigns, and STIR/SHAKEN status

---

### 7.2 Local Text Messaging Page

- **Route:** `/trust-center/local-text`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "Add Campaign" button, Info link
- Campaign table: Campaign Name, Brand, Use Case, Status (badge), DLC Status, Submitted, Actions (View/Edit)
- Campaign creation form:
  - Campaign Name input
  - Use Case select (Marketing / Notifications / Customer Care / Delivery Notifications / Account Notifications)
  - Description textarea
  - Sample Messages textarea
  - "Submit Campaign" button

### Actions
Add Campaign, View/Edit existing, Submit Campaign form

### Implied Functionality
A2P 10DLC campaign registration for local text messaging compliance (TCR/CSP registration)

---

### 7.3 Toll Free Text Messaging Page

- **Route:** `/trust-center/toll-free-text`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, Info link
- Status card: "Not Registered" with info text
- Registration form:
  - Business Name input (pre-filled)
  - Contact Name input
  - Contact Phone input
  - Use Case select
  - Monthly Volume select (Under 1,000 / 1,000-10,000 / 10,000-100,000 / Over 100,000)
  - "Submit Registration" button

### Actions
Submit Registration

### Implied Functionality
Toll-free number verification for text messaging (carrier registration)

---

### 7.4 Voice Registration Page

- **Route:** `/trust-center/voice-reg`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, Info link
- Status card: green "APPROVED" badge with description
- Verified Business Information: Business Name, EIN, Address + "Re-verify" button
- Verification History table: Date, Action, Status (badge)

### Mock History
- 2023-05-15: Initial Registration (Completed)
- 2025-05-15: Annual Re-verification (Completed)

### Actions
Re-verify button

### Implied Functionality
STIR/SHAKEN voice registration status and verification history

---

### 7.5 Caller ID Page

- **Route:** `/trust-center/caller-id`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, Info link
- Info banner: explains CNAM functionality and 48-hour propagation delay
- Caller ID Numbers table

### Table Columns
Number, Current CNAM, Status, Updated, Actions ("Update CNAM" button)

### Status Badges
- Green = Active
- Yellow = Pending
- Gray = Not Configured

### Mock Data
5 numbers with varying CNAM states (3 Active with "DIENER LAW", 1 Pending, 1 Not Configured)

### Actions
"Update CNAM" button per number

### Implied Functionality
Outbound caller ID (CNAM) management per phone number with status tracking

---

## Global Compliance

### 7.6 Requirements Page

- **Route:** `/trust-center/requirements`
- **Status:** Well-defined

### UI Elements
- Header with title and description, Info link
- Requirements table: Country, Requirement, Status (badge), Documentation

### Status Badges
- Green = Completed
- Yellow = In Progress
- Gray = Not Started

### Mock Data
| Country | Requirement | Status |
|---------|-------------|--------|
| US | STIR/SHAKEN | Completed |
| US | A2P 10DLC | Completed |
| Canada | CRTC Compliance | In Progress |
| UK | Ofcom Registration | Not Started |
| Germany | BNetzA Filing | Not Started |

### Actions
None (read-only reference table)

### Implied Functionality
Regulatory compliance checklist across multiple jurisdictions

---

### 7.7 Applications Page

- **Route:** `/trust-center/applications`
- **Status:** Well-defined

### UI Elements
- Header with title and description, "New Application" button
- Applications table: Application, Country, Status (badge), Submitted, Updated

### Status Badges
- Green = Approved
- Yellow = Pending
- Red = Rejected

### Mock Data
3 applications (STIR/SHAKEN=Approved, A2P 10DLC Brand=Approved, CRTC Filing=Pending)

### Actions
"New Application" button

### Implied Functionality
Regulatory application tracking and management

---

### 7.8 Addresses Page

- **Route:** `/trust-center/addresses`
- **Status:** Well-defined

### UI Elements
- Header with title and description, "New Address" button
- Addresses table: Label, Address, Country, Verified (checkmark or "Unverified"), Updated

### Mock Data
- Headquarters: 100 Europa Dr, Chapel Hill, NC (Verified)
- Branch Office: 456 Oak Ave, Raleigh, NC (Unverified)

### Actions
"New Address" button

### Implied Functionality
Business address management for regulatory compliance with address verification status
