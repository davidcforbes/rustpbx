# MyLegalTeam — North Star Vision (Concept Capture)

> **Status:** Concept only. To be incorporated as the North Star section of
> ARCHITECTURE_VISION.md when that document is created.
>
> **Captured:** 2026-02-21

---

## Strategic Evolution

The RustPBX platform replacement is Phase 1 of a three-horizon evolution that
transforms the firm from a transactional legal practice into **MyLegalTeam**
(working name) — a subscription-based, AI-powered legal services platform
capable of serving millions of customers.

### Horizon 1: Own the Infrastructure

Replace CTM/Twilio with self-hosted RustPBX + Telnyx. Own the call path, call
data, recordings, transcriptions, and all downstream integrations to Zoho CRM
and Flow Legal Management. Eliminate vendor lock-in and double billing.

### Horizon 2: AI-Powered Operations (Firm Scale)

Deploy AI agents on the backend to:

- Organize and structure incoming information
- Convert legacy documents into legal data and forms
- Orchestrate call center staff proactive workloads (outbound contact campaigns,
  customer information collection, debt collection)
- Organize and oversee legal workflows for attorneys and paralegal assistants
- Automate as much operational overhead as possible

### Horizon 3: MyLegalTeam Consumer Platform (Millions Scale)

Launch a customer-facing mobile app and subscription service:

- **Client portal:** All legal information, attorney communications, case status
- **AI client agents:** Help customers input information, answer attorney
  questions, understand legal documents, cross-reference legal information
- **Proactive service:** Automated updates when laws change, proactive
  recommendations, ongoing legal guidance
- **Revenue model:** Monthly subscription fee for ongoing automated legal services
- **Scale target:** Millions of subscribers

### Business Model Shift

```text
TODAY                           FUTURE
─────────────────────────────   ─────────────────────────────
Transactional legal factory     Ongoing automated legal service
Per-case revenue                Monthly subscription revenue
Manual intake + processing      AI-orchestrated workflows
Attorney-dependent scaling      AI-agent scaling
Hundreds of active cases        Millions of subscribers
Call center as cost center      Call center as onboarding engine
```

### Full Vendor Independence

The long-term architecture replaces **every major SaaS vendor** with self-hosted
Rust-based programs backed by PostgreSQL:

```text
VENDOR                 REPLACEMENT                      HORIZON
──────────────────     ────────────────────────────────  ───────
CTM + Twilio           RustPBX + Telnyx SIP             H1
Zoho CRM               Custom Rust CRM engine + PG      H2
Flow Legal Mgmt        Custom Rust case engine + PG      H2
Snowflake              PostgreSQL analytics + views    H2
                       (replaces mgmt reporting &
                        financial/debt analytics)
```

This eliminates: per-seat SaaS licensing, data export restrictions, API rate
limits, vendor lock-in, and fragmented data across multiple cloud platforms.
All data lives in PostgreSQL under full ownership.

---

*This concept will be expanded in ARCHITECTURE_VISION.md with detailed AI agent
taxonomy, customer journey mapping, technical architecture, and phased
implementation roadmap.*
