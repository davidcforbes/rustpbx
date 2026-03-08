# ARIA: Architectural Requirements for Intelligent Agents

## Design Document

**Date:** 2026-03-07
**Status:** Draft
**Author:** David Forbes + Claude (brainstorming session)

---

## 1. Mission

ARIA is a general-purpose, AI-first specification language and platform for describing complete technology architectures across every layer of the stack — from infrastructure through application logic — so that AI agents can build, verify, maintain, and repair systems from a single source of truth.

### Core Insight

Existing approaches (CLAUDE.md, AGENTS.md, architecture.md, database.md) are hand-crafted codified context scattered across markdown files. ARIA formalizes this into a queryable, validated, version-controlled system where the primary consumer is an AI agent, not a human reader. Human presentation layers translate and interpret for human consumption, but the canonical data is in native format for AI agents to directly consume or contribute.

### What ARIA Combines

| Existing Concept | What It Does | What ARIA Adds |
|-----------------|-------------|----------------|
| CMDB (ServiceNow, NetBox) | Knows what *is* deployed | Adds *what should be* (requirements) |
| RMS (DOORS, Jama) | Knows what *should* exist | Adds machine-readable format for AI agents |
| IaC (Terraform, Ansible) | Implements infrastructure | Adds architectural intent above the implementation |
| Codified Context (arxiv) | Documents code conventions | Extends to every layer of the stack |
| Spec Kit (GitHub) | Structures development workflow | Adds cross-layer constraints and verification |

### Related Work

- **"Codified Context: Infrastructure for AI Agents in a Complex Codebase"** (arxiv 2602.20478) — three-tier memory for AI agents in codebases. Covers application layer only.
- **GitHub Spec Kit** — 4-phase spec-driven development workflow. Application development only.
- **Archgate CLI** — ADRs as executable rules for AI agents. Architecture enforcement, not full-stack description.
- **OASIS TOSCA 2.0** — machine-readable cloud topology specification. Infrastructure layer only.
- **MCP** (Model Context Protocol) — standardizes agent-to-tool communication. Protocol, not architecture description.
- **A2A** (Agent2Agent Protocol) — standardizes agent-to-agent communication. Protocol, not architecture description.
- **Backstage** (CNCF) — developer portal with software catalog. Catalog of what exists, not specification of what should be built.

**Gap:** No existing project provides a unified, machine-readable specification across all layers that an AI agent can consume to build, verify, and maintain the complete system.

---

## 2. Core Concepts

- **Spec** — the complete architecture description for a system, stored as a versioned collection of JSON documents in Dolt, validated against JSON Schemas.
- **Layer** — a domain-scoped slice of the spec (infrastructure, OS, network, database, application, security, observability, etc.). Each layer has its own schema, extensible by specialist agents.
- **Requirement** — an individual declarative statement within a layer ("PostgreSQL 16+", "TLS 1.3 required on all public endpoints"). Has a zoom level: strategic, tactical, or operational.
- **Constraint** — a cross-layer rule that must hold true ("if app.protocol includes 'webrtc' then network.firewall must include udp_range(20000,20100)").
- **Verification** — the success criteria for a requirement, defining how to prove it's met and how to confirm it stays healthy.
- **Agent Interface** — the query/pull/diff API that agents use to interact with the spec.

---

## 3. The Layer Model

Each layer is a self-contained JSON Schema that defines the vocabulary for its domain. Layers are pluggable — the platform ships with a standard set, but specialist agents can register new ones.

### Standard Layers

| Layer | Scope | Example Requirements |
|-------|-------|---------------------|
| `infrastructure` | VPS, cloud providers, regions, compute instances | "2 vCPU, 4GB RAM, Ubuntu 24.04, us-east-1" |
| `network` | DNS, firewalls, load balancers, VPNs, ports | "UDP 5060, TCP 8443, TLS termination at LB" |
| `os` | Packages, kernel config, users, systemd services | "libssl3, net.core.rmem_max=2097152" |
| `database` | Engines, schemas, indexes, RLS, backups, pools | "PostgreSQL 16, 4 pools, PITR 30-day retention" |
| `application` | Services, APIs, modules, dependencies, build | "Rust 1.80+, axum, REST + WebSocket, feature flags" |
| `security` | Auth, certificates, secrets, policies, compliance | "mTLS between services, JWT auth, OWASP top 10" |
| `observability` | Metrics, logging, tracing, alerting, dashboards | "Prometheus, 15s scrape, alert on p99 > 500ms" |
| `data` | Data flows, ETL, retention, archival, privacy | "Call recordings to S3, 90-day hot, 7-year cold" |
| `integration` | External APIs, SIP trunks, webhooks, third-party | "Telnyx SIP trunk, credential auth, DID +1-707-..." |
| `ui` | Frontend framework, design system, build tooling | "Leptos 0.8, DaisyUI 5, Tailwind v4, trunk serve" |

### Zoom Levels

Every requirement within a layer carries a `depth` field. Requirements form a tree — strategic requirements decompose into tactical, which decompose into operational. An agent can query at any depth.

```json
{
  "layer": "database",
  "depth": "strategic",
  "requirement": "Multi-tenant PostgreSQL with row-level security"
}

{
  "layer": "database",
  "depth": "tactical",
  "requirement": "RLS policy on all iiz-schema tables filtering by account_id",
  "parent": "req-db-001",
  "schema": { "table": "call_records", "policy": "tenant_isolation" }
}

{
  "layer": "database",
  "depth": "operational",
  "requirement": "Connection pool: 20 max for call_processing, 10 for api_crud",
  "parent": "req-db-002"
}
```

| Depth | Purpose | Agent Use |
|-------|---------|-----------|
| `strategic` | Architecture decisions, technology selection | Understand the system |
| `tactical` | Implementation details, schemas, code structure | Build the system |
| `operational` | Runtime config, monitoring, maintenance procedures | Run and maintain the system |

---

## 4. Cross-Layer Constraints

Constraints express relationships between layers that must hold true across the system.

### Constraint Types

**Dependency** — one layer requires something in another:

```json
{
  "id": "constraint-001",
  "type": "dependency",
  "description": "WebRTC support requires UDP port range on firewall",
  "if": { "layer": "application", "path": "protocols", "contains": "webrtc" },
  "then": { "layer": "network", "path": "firewall.rules", "must_include": {
    "protocol": "udp", "port_range": [20000, 20100]
  }}
}
```

**Compatibility** — values across layers must be consistent:

```json
{
  "id": "constraint-002",
  "type": "compatibility",
  "description": "Database engine version must support required features",
  "if": { "layer": "database", "path": "features", "contains": "row_level_security" },
  "then": { "layer": "database", "path": "engine.version", "minimum": "9.5" }
}
```

**Propagation** — a policy cascades to multiple layers:

```json
{
  "id": "constraint-003",
  "type": "propagation",
  "description": "TLS requirement cascades to all public-facing layers",
  "if": { "layer": "security", "path": "policies.transport", "equals": "tls_required" },
  "then_each": [
    { "layer": "network", "path": "load_balancer.tls", "equals": true },
    { "layer": "application", "path": "endpoints.https", "equals": true },
    { "layer": "infrastructure", "path": "certificates", "min_count": 1 }
  ]
}
```

### Constraint Modes

1. **Validation** — run all constraints against current spec, report violations.
2. **Propagation** — when a strategic requirement is added, auto-generate downstream requirements in other layers (subject to human or master agent approval).
3. **Impact analysis** — when a change is proposed to one layer, show which constraints are affected and what other layers might need updates.

The master agent uses constraints to decompose work — if a new feature touches the `application` layer, it checks constraints to discover which other layers need specialist attention.

---

## 5. Verification & Acceptance Criteria

Every requirement carries a `verification` block that defines how to prove it's been met and how to confirm it stays healthy.

### Verification Hierarchy

| Level | Purpose | Audience | Examples |
|-------|---------|----------|----------|
| `acceptance` | Prove the requirement is implemented correctly | Build agents | Unit tests, integration tests, schema validation |
| `operational` | Prove the system stays healthy in production | Monitoring agents | SLA thresholds, uptime targets, latency budgets |
| `compliance` | Prove regulatory/policy adherence | Audit agents | Data retention checks, encryption validation, access logs |
| `resilience` | Prove the system recovers from failure | Chaos/repair agents | Failover time, backup restore test, degraded mode behavior |

### Verification Schema

```json
{
  "id": "req-db-001",
  "layer": "database",
  "depth": "strategic",
  "requirement": "Multi-tenant PostgreSQL with row-level security",
  "verification": {
    "acceptance": [
      {
        "type": "test",
        "strategy": "integration",
        "description": "Tenant A cannot read tenant B records",
        "automatable": true
      }
    ],
    "operational": [
      {
        "type": "sla",
        "metric": "query_latency_p99",
        "threshold": "< 50ms",
        "window": "5m",
        "severity": "warning"
      }
    ]
  }
}
```

### Standard Verification Entry

```json
{
  "type": "test | sla | audit | chaos",
  "strategy": "unit | integration | e2e | load | penetration | manual",
  "description": "Human-readable intent",
  "automatable": true,
  "script_ref": "tests/rls_tenant_isolation.py",
  "schedule": "on_change | hourly | daily | weekly | on_deploy",
  "metric": "optional — named metric for SLA types",
  "threshold": "optional — pass/fail boundary",
  "window": "optional — measurement window",
  "severity": "info | warning | critical",
  "escalation": "optional — who/what to notify on failure"
}
```

### SLA Rollup

Operational thresholds can aggregate upward into composite SLAs:

```json
{
  "id": "sla-system-001",
  "type": "composite_sla",
  "description": "Overall system availability",
  "target": "99.9%",
  "composed_of": [
    { "ref": "req-infra-001", "metric": "instance_uptime" },
    { "ref": "req-db-001", "metric": "query_latency_p99" },
    { "ref": "req-net-001", "metric": "packet_loss" }
  ],
  "calculation": "min(components)",
  "reporting": "monthly"
}
```

### How Agents Use Verification

1. **Build agent** finishes implementing a requirement, pulls its `acceptance` verifications, generates or runs test scripts, marks requirement as `verified` or `failed`.
2. **Master agent** won't close a requirement until all `acceptance` verifications pass.
3. **Monitoring agent** reads `operational` verifications, configures Prometheus alerts, Grafana dashboards, or health check endpoints to match the declared SLAs.
4. **Repair agent** detects an SLA breach, reads the requirement and its constraints, determines if it's a config drift, capacity issue, or bug, delegates to the appropriate specialist.
5. **Audit agent** periodically runs `compliance` verifications, produces evidence reports.

---

## 6. Agent Orchestration Model

### Three Agent Roles

**Master Agent** — the coordinator:
- Reads the spec at strategic level to understand the full system
- Decomposes work into layer-scoped tasks
- Delegates to specialist sub-agents
- Validates results against cross-layer constraints
- Does not need deep domain expertise — it's a project manager, not an engineer
- May delegate technical decisions down to specialists

**Specialist Agents** — domain experts:
- Register capabilities (which layers they handle, what tools they can use)
- Pull the spec for their layer(s) at tactical/operational depth
- Produce implementation artifacts (IaC, SQL, code, configs)
- Run acceptance verifications for their requirements
- Can propose spec changes back (e.g., "this kernel version doesn't support the required feature, recommending upgrade")

**Observer Agents** — ongoing monitoring:
- Read operational verifications and SLA definitions
- Compare live system state against the spec (drift detection)
- Report discrepancies to the master agent
- Can trigger repair workflows when thresholds are breached

### Agent Interface (API)

```
# Query — ask a question across layers
POST /spec/query
{ "question": "what database engine and version?",
  "depth": "tactical",
  "layers": ["database"] }

# Pull — get the full spec for a layer
GET /spec/layers/database?depth=tactical

# Diff — compare spec vs. live state
POST /spec/drift
{ "layer": "network",
  "live_state": { "firewall_rules": [...] } }

# Propose — suggest a spec change
POST /spec/propose
{ "agent": "database-specialist",
  "changes": [{ "path": "database.engine.version",
                "from": "16", "to": "17",
                "reason": "Required for pg_stat_io views" }],
  "impact": ["constraint-002"] }

# Verify — report test results against requirements
POST /spec/verify
{ "requirement": "req-db-001",
  "verification": "acceptance-001",
  "result": "pass",
  "evidence": { "test_output": "...", "duration_ms": 342 } }
```

### Workflow Example — New Feature Request

```
1. Human adds strategic requirement to application layer
2. Master agent reads it, checks constraints
3. Constraints propagate requirements to database, network, security layers
4. Master delegates:
   -> database-specialist: "implement schema changes per req-db-047"
   -> network-specialist: "update firewall rules per req-net-012"
   -> security-specialist: "review new endpoint auth per req-sec-031"
5. Each specialist pulls tactical spec, implements, runs acceptance tests
6. Master validates cross-layer constraints
7. Observer agents update monitoring for new operational thresholds
```

---

## 7. Requirement Lifecycle

Every requirement moves through a defined lifecycle with 8 states.

### State Machine

```
                    +----------+
                    |  draft   |  <-- Human or AI proposes
                    +----+-----+
                         | approved
                    +----v-----+
                    |  active  |  <-- In the spec, not yet built
                    +----+-----+
                         | assigned to specialist
                    +----v---------+
                    | in_progress   |  <-- Specialist agent working
                    +----+---------+
                         | acceptance tests pass
                    +----v---------+
                    |  verified    |  <-- Built and proven
                    +----+---------+
                         | deployed + observer monitoring
                    +----v---------+
                    |  operational |  <-- Live, SLAs being tracked
                    +----+---------+
                         |
              +----------+----------+
              |                     |
     +--------v------+    +--------v-----------+
     |   disabled    |    |    degraded        |
     | (intentionally|    | (SLA breach/drift) |
     |  turned off)  |    +--------+-----------+
     +-------+-------+            |
             |                    | repair verified
             |                    +---> operational
             v
     +-------+----------+
     | decommissioned   |  <-- Permanently torn down
     +------------------+

     Any state --> deprecated (requirement no longer applies)
     Any state --> superseded (replaced by newer requirement)
     Any state --> decommissioned (permanently removed)
```

### State Definitions

| State | Meaning | Agent Behavior |
|-------|---------|----------------|
| `draft` | Proposed, not yet approved | Not actionable |
| `active` | Approved, awaiting implementation | Available for assignment |
| `in_progress` | Specialist agent working on it | Blocked from other agents |
| `verified` | Built, acceptance tests passing | Ready for deployment |
| `operational` | Live in production, SLAs tracked | Observer agents monitoring |
| `degraded` | SLA breach or drift detected | Triggers repair workflow |
| `disabled` | Intentionally turned off, preservable | Observer skips SLA checks, config preserved |
| `decommissioned` | Permanently torn down | Cleanup specialist removes artifacts |
| `deprecated` | Requirement no longer applies | Informational, no action |
| `superseded` | Replaced by a newer requirement | Links to replacement |

### Disabled vs Decommissioned

```json
{
  "status": "disabled",
  "reason": "Trunk registrar paused during Telnyx migration",
  "preservable": true,
  "reactivation_conditions": "New SIP provider credentials configured",
  "disable_actions": ["stop systemd service", "remove from LB pool"],
  "resources_retained": true
}

{
  "status": "decommissioned",
  "reason": "Replaced by WebSocket-based integration",
  "decommission_actions": ["drop tables", "delete firewall rules", "remove packages"],
  "cleanup_verified": true,
  "resources_retained": false
}
```

### State Rules

- A requirement can't move to `verified` without all `acceptance` verifications passing.
- A requirement can't move to `operational` without deployment confirmation + observer registered.
- `degraded` triggers automatic impact analysis across constraints.
- `superseded` requirements must link to their replacement.
- Child requirements (tactical/operational) can't advance past their parent's state.
- `disabled` can transition back to `operational` (re-enabled) or forward to `decommissioned`.

### Document Lifecycle

```
draft --> under_review --> approved --> decomposed --> archived
```

A document reaches `decomposed` once all its requirements have been extracted and linked. The document remains as the historical record, but agents work from the structured requirements.

---

## 8. Storage Architecture

Two-tier storage using Dolt for structured data and GitHub for file-based artifacts.

### Dolt — The Canonical Store

All requirements, constraints, verifications, and CMDB state live in Dolt tables with full git-style versioning. Dolt is MySQL-compatible, supporting SQL queries, branching, merging, and cell-level diffing.

```sql
-- Core tables
requirements (
  id, layer, depth, parent_id, document_id,
  status, requirement_json, source_excerpt,
  schema_version, created_by, created_at, updated_at
)

constraints (
  id, type, description, if_json, then_json
)

verifications (
  id, requirement_id, level, type, strategy, definition_json
)

verification_results (
  id, verification_id, agent_id, result, evidence_json, timestamp
)

documents (
  id, type, title, summary, source_url, full_text,
  layer, depth, status, created_by, created_at
)

layers (
  id, name, schema_ref, description
)

agent_registry (
  id, name, role, capabilities_json, layers_json
)

cmdb_state (
  id, layer, entity_type, entity_id, live_state_json, last_observed
)

drift_events (
  id, layer, requirement_id, expected_json, actual_json, severity, timestamp
)

change_proposals (
  id, agent_id, changes_json, impact_json, status, reviewed_by
)
```

**Why Dolt over plain git + JSON files:**
- SQL queries across requirements ("show me all unverified security requirements")
- Relational joins between tables ("which constraints reference this requirement?")
- Branch/merge/diff on structured data, not file-level diffs
- Conflict resolution on cell-level, not line-level
- Agent-friendly — SQL is a well-understood interface for LLMs

**Example Dolt operations:**

```sql
SELECT * FROM requirements WHERE layer = 'database' AND depth = 'tactical';
SELECT * FROM dolt_diff('main', 'feature-x', 'requirements');
SELECT * FROM dolt_log('requirements') WHERE author = 'database-specialist';
CALL dolt_checkout('-b', 'proposal/upgrade-pg-17');
CALL dolt_merge('proposal/upgrade-pg-17');
```

### GitHub — Implementation Artifacts and Spec Schemas

```
aria-project/
|-- schemas/                  # JSON Schemas per layer
|   |-- common.schema.json
|   |-- infrastructure.schema.json
|   |-- database.schema.json
|   |-- application.schema.json
|   +-- ...
|-- constraints/              # Constraint definitions
|   +-- cross-layer.json
|-- artifacts/                # Generated implementation files
|   |-- terraform/
|   |-- ansible/
|   |-- migrations/
|   +-- ...
|-- tests/                    # Verification scripts
|   |-- acceptance/
|   |-- compliance/
|   +-- resilience/
|-- docs/                     # Human-readable views (generated)
|   |-- architecture-overview.md
|   +-- layer-summaries/
+-- .aria/                    # Platform config
    |-- config.json
    +-- agent-registry.json
```

### Sync Rules

- Dolt is the source of truth for requirements data.
- GitHub is the source of truth for schemas, artifacts, and test scripts.
- The `artifacts/` directory contains files generated from the spec (Terraform, SQL migrations, etc.).
- Schema changes in GitHub trigger re-validation of requirements in Dolt.
- `docs/` is auto-generated from Dolt data — never hand-edited.

### PRD and Document Handling

Full PRD prose can be stored in the `documents` table (`full_text` column, LONGTEXT) or linked via `source_url` to a file in GitHub. The key is decomposition:

1. Full PRD stored as document (text blob + metadata).
2. AI agent (or human) decomposes it into structured requirements.
3. Each requirement traces back to the source document + excerpt via `document_id` and `source_excerpt`.
4. Agents query the structured requirements, not the prose.
5. Traceability is preserved — an agent working on `req-db-047` can trace it back to the exact paragraph in the original PRD.

---

## 9. Discovery & Reverse Engineering

For brownfield systems, ARIA needs to discover the existing architecture and populate the spec from what's already running.

### Discovery Agents

| Discovery Agent | Scans | Produces |
|----------------|-------|----------|
| `infra-discovery` | Cloud APIs, SSH into hosts | Instance specs, OS versions, installed packages |
| `network-discovery` | `iptables`, UFW, DNS records, LB configs | Firewall rules, port mappings, DNS entries |
| `db-discovery` | `information_schema`, `pg_catalog`, connection params | Tables, indexes, RLS policies, pool configs |
| `app-discovery` | Source code, Cargo.toml, package.json, running processes | Dependencies, services, API endpoints, feature flags |
| `security-discovery` | Certificates, auth configs, secrets managers | TLS status, auth mechanisms, key expiry dates |
| `observability-discovery` | Prometheus configs, Grafana dashboards, log pipelines | Current metrics, alert rules, retention policies |

### Discovery Workflow

```
1. Point ARIA at a live system (SSH host, cloud account, git repo)
2. Master agent dispatches discovery agents in parallel
3. Each discovery agent scans its domain, produces draft requirements
4. Results stored in Dolt as status = "draft", source = "discovered"
5. Human reviews — approves, adjusts, or rejects each discovered requirement
6. Approved requirements become the baseline spec
7. Drift detection starts immediately against the now-documented state
```

---

## 10. Platform Runtime

### Architecture

```
+-----------------------------------------------------+
|                   ARIA Platform                      |
|                                                      |
|  +----------+  +----------+  +-------------------+  |
|  | Spec API |  | Constraint|  |  Agent Gateway    |  |
|  | (REST +  |  |  Engine   |  |  (auth, routing,  |  |
|  |  GraphQL)|  |           |  |   rate limiting)  |  |
|  +----+-----+  +-----+----+  +--------+----------+  |
|       |              |                |              |
|  +----v--------------v----------------v----------+  |
|  |              Core Services                     |  |
|  |  +--------+ +----------+ +------------------+ |  |
|  |  |Lifecycle| |  Drift   | |  Impact Analysis | |  |
|  |  |Manager | | Detector | |  & Propagation   | |  |
|  |  +--------+ +----------+ +------------------+ |  |
|  +-------------------+---------------------------+  |
|                      |                               |
|  +-------------------v---------------------------+  |
|  |              Storage Layer                     |  |
|  |  +--------------+    +---------------------+  |  |
|  |  |  Dolt         |    |  GitHub / Git        |  |
|  |  |  (requirements|    |  (schemas, artifacts,|  |
|  |  |   CMDB, state)|    |   tests, docs)       |  |
|  |  +--------------+    +---------------------+  |  |
|  +-----------------------------------------------+  |
+-----------------------------------------------------+
         |                    |
    +----v----+          +---v----+
    |Specialist|          |Observer|
    | Agents   |          | Agents |
    +---------+          +--------+
```

### Component Responsibilities

- **Spec API** — dual interface: REST for simple CRUD and agent interactions; GraphQL for complex cross-layer queries.
- **Constraint Engine** — evaluates constraints on every spec change; validates, propagates, and runs impact analysis.
- **Agent Gateway** — controls agent access with authentication, layer-scoped permissions, audit trail, and rate limiting.
- **Lifecycle Manager** — enforces the state machine, blocks invalid transitions, checks verification prerequisites.
- **Drift Detector** — compares CMDB snapshots from observer agents against operational requirements, generates drift events.

### Interface Priority

1. **Native API** — programmatic, fastest, full-featured (REST + GraphQL)
2. **CLI** — human operators and shell scripts (`aria query`, `aria drift`, `aria propose`)
3. **MCP** — AI agents that speak Model Context Protocol natively (Claude Code, Cursor, etc.)
4. **Batch file handoff** — export/import JSON files for offline or air-gapped workflows

Each layer wraps the one above it — CLI calls the API, MCP adapter calls the API, batch processor calls the API. Single implementation, multiple access patterns.

### Technology Choices

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| API server | Rust (axum) or Go | Performance, single binary |
| Constraint engine | JSON Logic or Rego | Declarative, auditable |
| Dolt access | MySQL-compatible driver | Dolt speaks MySQL wire protocol |
| Git access | libgit2 or shell-out | Schema and artifact management |
| Agent comms | REST + SSE for streaming | Simple, stateless, firewall-friendly |
| Auth | JWT + API keys | Per-agent scoped tokens |

---

## 11. Implementation Roadmap

Each phase delivers usable value incrementally.

### Phase 0 — Spec Language Foundation

- Define the JSON Schema envelope (common fields all layers share)
- Define 3 starter layer schemas: `infrastructure`, `database`, `application`
- Define constraint schema
- Define verification schema
- Store everything in a GitHub repo as `.schema.json` files
- **Deliverable:** A validated spec format you can start writing requirements in

### Phase 1 — Dolt Backend + REST API

- Stand up Dolt instance with core tables
- Build REST API: CRUD for requirements, query by layer/depth, propose changes
- CLI wrapper over the API (`aria init`, `aria add`, `aria query`, `aria status`)
- **Deliverable:** A working requirements database you can populate and query

### Phase 2 — Constraint Engine + Lifecycle

- Implement constraint evaluation
- Implement state machine with transition validation
- Impact analysis on proposed changes
- Propagation of cross-layer constraints
- **Deliverable:** The spec becomes "smart" — it can validate itself and cascade changes

### Phase 3 — Discovery Agents

- Build discovery agents for immediate stack: Linux host, PostgreSQL, Rust app, network
- Discovery to draft requirements pipeline
- Human review/approval workflow
- **Deliverable:** Point ARIA at a live server, get a populated spec

### Phase 4 — Drift Detection + Observer Agents

- Observer agents that snapshot live state and push to CMDB tables
- Drift detector compares CMDB vs spec
- Alerting on drift events
- **Deliverable:** Continuous reconciliation — know when reality diverges from spec

### Phase 5 — Master Agent Orchestration

- Master agent reads spec, decomposes work, delegates to specialists
- Specialist agent interface (pull spec, report results, propose changes)
- Verification result tracking and state advancement
- **Deliverable:** AI agents can build from the spec end-to-end

### Phase 6 — MCP Adapter + Batch Handoff

- MCP server exposing ARIA as tools/resources for Claude Code, Cursor, etc.
- Batch export/import for offline workflows
- **Deliverable:** Full interface hierarchy complete

### Phase 7 — Human Presentation Layer

- Dashboard showing spec status across all layers
- Requirement traceability views (PRD to strategic to tactical to operational to verified)
- Drift and SLA dashboards
- Architecture diagrams auto-generated from spec
- **Deliverable:** Humans can see what the agents see, in human-friendly form

---

## 12. Design Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Primary consumer | AI agents (machine-first) | Human presentation layers built on top |
| Data format | JSON, validated by JSON Schema | Most deterministic for AI agent consumption |
| Storage | Dolt (structured data) + GitHub (schemas, artifacts, tests) | SQL-queryable with git-style versioning |
| Spec granularity | Three zoom levels: strategic, tactical, operational | Different agents need different depth |
| Cross-layer | Constraints with validation, propagation, and impact analysis | Systems are interconnected, specs should be too |
| Verification | Acceptance, operational/SLA, compliance, resilience | Every requirement defines what "done" and "healthy" mean |
| Lifecycle | 8+ states including disabled and decommissioned | Full lifecycle from proposal through retirement |
| Agent model | Master coordinator + specialist sub-agents + observer agents | Master delegates, doesn't need domain expertise |
| Interface priority | API first, CLI second, MCP third, batch fourth | Each wraps the layer above |
| Brownfield support | Discovery agents reverse-engineer existing systems | Most projects aren't greenfield |
| Change management | Git-style branching in Dolt, proposals with approval workflow | Structured change tracking with full history |
| Document handling | PRDs stored or linked, decomposed into structured requirements | Traceability from prose to implementation |
