# RustPBX Testing — Plan of Action

## Overview
This document is the master checklist for designing, documenting, and tracking
the complete testing infrastructure for the RustPBX VoIP platform. Each stage
produces specific deliverables. Once all stages are complete, beads records
are created so work can be delegated to sub-agents.

**Created**: 2026-02-20
**Workspace**: `C:\Development\RustPBX`
**Beads Project**: RustPBX (initialized at `.beads\beads.db`)

---

## Stages & Deliverables

### Stage 1: Testing Strategy Document
**File**: `docs/TESTING_STRATEGY.md`
**Status**: 🔲 Not Started
**Contents**:
- Testing philosophy and guiding principles
- Test levels matrix (L0 Smoke through L8 Security)
- Test environment architecture (Docker Compose topology)
- Tool selection and justification for each level
- Test data management strategy
- Pass/fail criteria per level
- Defect severity classification
- Reporting and metrics

### Stage 2: Test Environment Setup Guide
**File**: `docs/TEST_ENVIRONMENT_SETUP.md`
**Status**: 🔲 Not Started
**Contents**:
- docker-compose.test.yml for isolated test stack
- Test configuration TOML (test users, routes, trunks)
- Test tools container (SIPp, PJSUA, Python pytest)
- PostgreSQL test database initialization
- Network topology and port mapping
- Prerequisites and host machine setup
- Teardown and cleanup procedures

### Stage 3: Test Cases — L0 Smoke Tests
**File**: `docs/tests/L0_SMOKE_TESTS.md`
**Status**: 🔲 Not Started
**Contents**:
- TC-L0-001: Container starts and stays running
- TC-L0-002: HTTP port 8080 responds
- TC-L0-003: SIP port 5060 accepts UDP
- TC-L0-004: SIP port 15060 accepts UDP
- TC-L0-005: Database migrations applied (tables exist)
- TC-L0-006: Console login page loads
- TC-L0-007: Super user creation via CLI succeeds
- TC-L0-008: Health/version endpoint responds

### Stage 4: Test Cases — L1 Infrastructure Tests
**File**: `docs/tests/L1_INFRASTRUCTURE_TESTS.md`
**Status**: 🔲 Not Started
**Contents**:
- TC-L1-001: SIP OPTIONS ping to RustPBX returns 200 OK
- TC-L1-002: SIP REGISTER with valid credentials succeeds (401→200)
- TC-L1-003: SIP REGISTER with invalid credentials returns 403
- TC-L1-004: WebSocket upgrade at /ws succeeds
- TC-L1-005: TLS certificate validation (when TLS enabled)
- TC-L1-006: RTP port range accessible (UDP probes)
- TC-L1-007: PostgreSQL connectivity from RustPBX container
- TC-L1-008: Telnyx trunk registration (requires credentials)
- TC-L1-009: DNS resolution for sip.telnyx.com

### Stage 5: Test Cases — L2 API Contract Tests
**File**: `docs/tests/L2_API_CONTRACT_TESTS.md`
**Status**: 🔲 Not Started
**Contents**:
- TC-L2-001: Console authentication (login/logout/session)
- TC-L2-002: Extension CRUD (create, read, update, delete)
- TC-L2-003: Route CRUD and pattern matching
- TC-L2-004: Trunk CRUD and health status
- TC-L2-005: Call records listing and filtering
- TC-L2-006: AMI API authentication (IP restriction)
- TC-L2-007: AMI reload endpoints (routes, trunks, ACL)
- TC-L2-008: WebSocket event subscription
- TC-L2-009: Recording download endpoint
- TC-L2-010: API error handling (400, 401, 404, 500)

### Stage 6: Test Cases — L3 SIP Functional Tests
**File**: `docs/tests/L3_SIP_FUNCTIONAL_TESTS.md`
**Status**: 🔲 Not Started
**Contents**:
- TC-L3-001: Internal call between two registered extensions
- TC-L3-002: Call hold (re-INVITE with sendonly) and resume
- TC-L3-003: Blind transfer (REFER)
- TC-L3-004: Attended transfer
- TC-L3-005: Call forward on no answer
- TC-L3-006: Call forward on busy
- TC-L3-007: Voicemail routing (no answer timeout)
- TC-L3-008: Simultaneous ring (multi-device)
- TC-L3-009: DTMF relay (RFC 2833)
- TC-L3-010: Codec negotiation (PCMU, PCMA, G722, Opus)
- TC-L3-011: Call recording triggers and file creation
- TC-L3-012: CDR generation on call completion
- TC-L3-013: Graceful BYE from caller and callee
- TC-L3-014: Abnormal termination handling (TCP reset, timeout)

### Stage 7: Test Cases — L4 through L8
**File**: `docs/tests/L4_L8_ADVANCED_TESTS.md`
**Status**: 🔲 Not Started
**Contents**:
- L4 Integration: Telnyx inbound/outbound PSTN calls
- L5 Media Quality: MOS scoring, recording playback, codec verification
- L6 Load: SIPp concurrent registrations, concurrent calls, API throughput
- L7 Failover: Container crash recovery, DB reconnect, trunk failover
- L8 Security: Auth bypass attempts, SIP fuzzing, API injection

### Stage 8: Test Harness & Scripts
**File**: `tests/` directory structure
**Status**: 🔲 Not Started
**Contents**:
- `tests/conftest.py` — pytest fixtures (Docker, API client, SIP client)
- `tests/test_L0_smoke.py` — Automated smoke tests
- `tests/test_L1_infra.py` — Automated infrastructure tests
- `tests/test_L2_api.py` — Automated API contract tests
- `tests/sipp/` — SIPp XML scenarios for L3 SIP tests
- `tests/docker-compose.test.yml` — Test stack orchestration
- `tests/config/test-config.toml` — RustPBX test configuration
- `tests/Dockerfile.test-tools` — SIPp + PJSUA + Python test image
- `tests/run_all.sh` / `tests/run_all.ps1` — Master test runner
- `tests/requirements.txt` — Python test dependencies

### Stage 9: Beads Work Breakdown
**Tool**: beads CLI (`.beads/beads.db`)
**Status**: 🔲 Not Started
**Contents**:
- 1 Epic: Testing Infrastructure
- 1 Epic: Test Automation (L0-L3)
- 1 Epic: Advanced Testing (L4-L8)
- Features and tasks broken out per test level
- Dependencies mapped (setup before tests, L0 before L1, etc.)
- Priority and assignee fields for delegation

---

## Execution Order

```
Stage 1 ──→ Stage 2 ──→ Stage 3 ──→ Stage 4 ──→ Stage 5
   │            │           │           │           │
   │  Testing   │  Env      │  Smoke    │  Infra    │  API
   │  Strategy  │  Setup    │  Tests    │  Tests    │  Tests
   │            │           │           │           │
   ▼            ▼           ▼           ▼           ▼
Stage 6 ──→ Stage 7 ──→ Stage 8 ──→ Stage 9
   │            │           │           │
   │  SIP       │  Advanced │  Harness  │  Beads
   │  Tests     │  L4-L8   │  Scripts  │  Records
   │            │           │           │
   ▼            ▼           ▼           ▼
                                      DONE
                                   (delegate)
```

---

## Progress Tracker

| Stage | Deliverable | Status | Notes |
|-------|-------------|--------|-------|
| 1 | TESTING_STRATEGY.md | ✅ | 10 sections, 16KB |
| 2 | TEST_ENVIRONMENT_SETUP.md | ✅ | Docker Compose + docs |
| 3 | L0_SMOKE_TESTS.md | ✅ | 8 test cases |
| 4 | L1_INFRASTRUCTURE_TESTS.md | ✅ | 9 test cases |
| 5 | L2_API_CONTRACT_TESTS.md | ✅ | 10 test cases |
| 6 | L3_SIP_FUNCTIONAL_TESTS.md | ✅ | 14 test cases |
| 7 | L4_L8_ADVANCED_TESTS.md | ✅ | 29 test cases (L4-L8) |
| 8 | tests/ scripts & harness | ✅ | 8 pytest files, 6 SIPp XML, runners |
| 9 | Beads work breakdown | ✅ | 3 epics, 3 features — all closed |
