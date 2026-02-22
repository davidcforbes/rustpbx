# RustPBX Testing Strategy

**Version:** 1.0
**Created:** 2026-02-21
**Project:** RustPBX VoIP Platform

---

## 1. Testing Philosophy

### Guiding Principles

1. **Automate everything that can be automated.** Manual testing is reserved only for subjective quality assessments (audio quality, UI usability) and one-time setup verification.
2. **Test at the right level.** Prefer fast, deterministic tests (L0-L2) for regression catching. Use slower integration tests (L3-L6) for protocol correctness. Reserve expensive tests (L7-L8) for release gates.
3. **Fail fast, fail loud.** Smoke tests run first. If L0 fails, nothing else runs.
4. **Reproducible environments.** Every test runs against a Docker Compose stack with deterministic configuration. No shared state between test runs.
5. **SIP-protocol-aware assertions.** Tests assert on SIP response codes, header fields, and RTP media flow — not just HTTP status codes.
6. **Real carrier testing is separate.** Telnyx PSTN tests (L4) require credentials and cost money. They run on-demand, never in CI loops.

### What We Test

| Layer | Examples | Primary Tool |
|-------|----------|-------------|
| Network transport | UDP/TCP ports, WebSocket upgrade, TLS | pytest + socket probes |
| SIP signaling | REGISTER, INVITE, BYE, REFER, re-INVITE | SIPp scenarios |
| Media/RTP | Port allocation, codec negotiation, audio flow | SIPp RTP, PJSUA |
| HTTP/API | Console auth, CRUD, AMI endpoints | pytest + requests |
| Recording | WAV file creation, playback, transcript | pytest + file assertions |
| CDR | Call record accuracy, timing, field completeness | pytest + API queries |
| Integration | Telnyx PSTN inbound/outbound | Manual + scripted |
| Resilience | Container restart, DB reconnect, trunk failover | pytest + Docker API |

---

## 2. Test Levels Matrix

### Overview

```
L0 Smoke ──► L1 Infrastructure ──► L2 API Contract ──► L3 SIP Functional
                                                              │
                        L4 Integration ◄──────────────────────┘
                              │
                L5 Media Quality ◄────┘
                              │
                    L6 Load/Stress ◄──┘
                              │
                      L7 Failover ◄───┘
                              │
                      L8 Security ◄───┘
```

### Level Details

| Level | Name | Scope | Duration | Gate | Frequency |
|-------|------|-------|----------|------|-----------|
| L0 | Smoke | Container starts, ports respond | < 30s | Must pass for any other test | Every commit |
| L1 | Infrastructure | SIP/HTTP/WS connectivity | < 60s | Must pass for L2+ | Every commit |
| L2 | API Contract | HTTP API CRUD and auth | < 120s | Must pass for L3+ | Every commit |
| L3 | SIP Functional | Call flows, features, CDR | < 300s | Must pass for release | Every PR |
| L4 | Integration | Telnyx PSTN calls | < 600s | Advisory (costs money) | On-demand |
| L5 | Media Quality | Audio/codec/recording | < 300s | Must pass for release | Every PR |
| L6 | Load/Stress | Concurrent calls/registrations | < 600s | Advisory | Pre-release |
| L7 | Failover | Crash recovery, reconnection | < 300s | Must pass for release | Pre-release |
| L8 | Security | Auth bypass, fuzzing, injection | < 300s | Must pass for release | Pre-release |

### Entry/Exit Criteria Per Level

| Level | Entry Criteria | Exit Criteria (Pass) | Exit Criteria (Fail) |
|-------|---------------|---------------------|---------------------|
| L0 | Docker image built | All 8 checks green | Block: container broken |
| L1 | L0 passed | All 9 connectivity checks pass | Block: network/config issue |
| L2 | L1 passed | All 10 API tests pass, no 500s | Block: API regression |
| L3 | L2 passed | All 14 SIP scenarios complete with correct responses | Block: SIP logic regression |
| L4 | L3 passed + Telnyx credentials | Inbound + outbound PSTN calls connect with audio | Advisory: trunk config issue |
| L5 | L3 passed | Recording files exist with correct audio, codec match | Block: media pipeline broken |
| L6 | L3 passed | Handles 50 concurrent registrations, 10 concurrent calls | Advisory: capacity concern |
| L7 | L3 passed | Recovery within 30s for each scenario | Block: resilience issue |
| L8 | L2 passed | No auth bypasses, no crashes from fuzzing | Block: security vulnerability |

---

## 3. Test Environment Architecture

### Docker Compose Topology

```
┌─────────────────────────────────────────────────────┐
│                rustpbx-test network                 │
│                  (bridge, isolated)                 │
│                                                     │
│  ┌──────────────┐  ┌──────────────┐                │
│  │   rustpbx    │  │  postgres    │                │
│  │              │  │  (16-alpine) │                │
│  │  SIP: 5060   │  │  Port: 5432  │                │
│  │  HTTP: 8080  │  │              │                │
│  │  WS: 8080/ws │  │  DB: rustpbx │                │
│  │  RTP: 20000- │  │  User: test  │                │
│  │       20100  │  │  Pass: test  │                │
│  └──────┬───────┘  └──────────────┘                │
│         │                                           │
│  ┌──────┴───────┐                                  │
│  │  test-tools  │                                  │
│  │              │                                  │
│  │  SIPp        │                                  │
│  │  PJSUA       │                                  │
│  │  Python 3.12 │                                  │
│  │  pytest      │                                  │
│  │  curl        │                                  │
│  └──────────────┘                                  │
└─────────────────────────────────────────────────────┘
```

### Container Details

| Container | Image | Purpose | Ports |
|-----------|-------|---------|-------|
| `rustpbx` | `ghcr.io/restsend/rustpbx:latest` | System under test | 5060/udp, 5060/tcp, 8080/tcp, 20000-20100/udp |
| `postgres` | `postgres:16-alpine` | Test database | 5432/tcp (internal only) |
| `test-tools` | `rustpbx-test-tools:latest` (custom) | Test runner + SIP tools | None (initiates connections) |

### Test Configuration

The test environment uses a dedicated `test-config.toml` with:

- **2 memory-backend test users**: 1001/test1001, 1002/test1002
- **1 extension-backend**: for dynamically created users
- **Internal route**: `^(100[0-9])$` → local
- **Mock trunk route**: `^(555\d{7})$` → loopback SIPp UAS
- **Recording enabled**: auto_start = true
- **CDR**: local SQLite storage
- **No Telnyx credentials**: L4 tests use separate config

---

## 4. Tool Selection

| Tool | Version | Purpose | Test Levels |
|------|---------|---------|-------------|
| **pytest** | 8.x | Test harness, fixtures, assertions | All |
| **requests** | 2.x | HTTP API testing | L1, L2, L5 |
| **SIPp** | 3.7+ | SIP traffic generation and validation | L1, L3, L6 |
| **PJSUA** | 2.x | SIP UA automation (registration, calls) | L3, L4, L5 |
| **Docker SDK** | 7.x | Container lifecycle control | L0, L7 |
| **websockets** | 12.x | WebSocket SIP testing | L1, L3 |
| **pydub** | 0.25+ | Audio file analysis for recording tests | L5 |
| **Docker Compose** | v2 | Test environment orchestration | All |
| **bash/curl** | - | Quick validation scripts | L0, L1 |

### Why These Tools

- **SIPp over PJSUA for load**: SIPp is purpose-built for SIP load testing with XML scenario files. It handles thousands of concurrent calls with low overhead.
- **PJSUA for functional**: PJSUA provides a real SIP UA with audio capabilities, needed for codec negotiation and media verification tests.
- **pytest over unittest**: Better fixtures, parametrization, and plugin ecosystem. Docker fixtures via `pytest-docker` or custom.
- **requests over httpx**: Simpler API, synchronous is fine for test assertions. No async complexity needed.

---

## 5. Test Data Management

### Static Test Data

| Data | Location | Contents |
|------|----------|----------|
| Test config | `tests/config/test-config.toml` | Pre-defined users, routes, trunk stubs |
| SIPp scenarios | `tests/sipp/*.xml` | SIP message flows for each test case |
| Audio samples | `tests/fixtures/` | Reference WAV files for media tests |
| Expected responses | Inline in test code | SIP response codes, JSON schemas |

### Dynamic Test Data

- **Extensions created via API** are cleaned up in test teardown
- **CDR records** are verified then left in the test database (ephemeral container)
- **Recording files** are verified then left in the container filesystem
- **Each test run gets a fresh container set** — no state leaks between runs

### Credential Management

| Credential | Storage | Used In |
|------------|---------|---------|
| Test user passwords | `test-config.toml` (not secret) | L1-L3 |
| Console admin | `conftest.py` fixture (not secret) | L2 |
| Telnyx SIP credentials | Environment variables only | L4 |
| Groq API key | Environment variables only | Transcript tests |

---

## 6. Pass/Fail Criteria

### Per-Test Criteria

| Assertion Type | Pass | Fail |
|----------------|------|------|
| SIP response code | Matches expected (e.g., 200, 401, 486) | Wrong code or timeout |
| HTTP status | Matches expected | Wrong status or connection refused |
| File exists | Recording WAV file present and size > 0 | File missing or empty |
| JSON schema | Response matches expected structure | Missing fields or wrong types |
| Audio duration | Within 10% of expected call duration | Off by more than 10% |
| Timing | Response within timeout threshold | Timeout exceeded |
| CDR fields | All required fields present and correct | Missing or incorrect values |

### Per-Level Criteria

| Level | Pass | Fail |
|-------|------|------|
| L0-L2 | 100% of tests pass | Any failure blocks |
| L3 | 100% of tests pass | Any failure blocks release |
| L4 | Core scenarios pass (connect + audio) | Advisory only |
| L5 | Recording exists with audio, codecs match | Blocks release |
| L6 | Meets capacity thresholds | Advisory |
| L7 | Recovery within SLA times | Blocks release |
| L8 | No security vulnerabilities found | Blocks release |

---

## 7. Defect Severity Classification

| Severity | Definition | Example | Response |
|----------|-----------|---------|----------|
| **S0 Critical** | System completely unusable | Container crash loop, SIP port not listening | Fix immediately, block release |
| **S1 Major** | Core feature broken | Calls fail to connect, no audio, registration broken | Fix before release |
| **S2 Moderate** | Feature degraded | Recording missing audio, CDR timing off, transfer fails | Fix in next sprint |
| **S3 Minor** | Cosmetic or edge case | Console UI alignment, rare codec mismatch | Backlog |
| **S4 Enhancement** | Working but could be better | Performance optimization, additional logging | Backlog |

### Severity by Test Level

| Level | Typical Severity |
|-------|-----------------|
| L0 failure | S0 Critical |
| L1 failure | S0-S1 |
| L2 failure | S1-S2 |
| L3 failure | S1-S2 |
| L4 failure | S2 (may be carrier-side) |
| L5 failure | S1-S2 |
| L6 failure | S2-S3 |
| L7 failure | S1-S2 |
| L8 failure | S0-S1 |

---

## 8. Reporting and Metrics

### Test Run Report Format

Each test run produces:

```
=== RustPBX Test Report ===
Date:       2026-02-21 14:30:00 UTC
Version:    0.3.18
Image:      ghcr.io/restsend/rustpbx:latest
Duration:   4m 32s

Level   Tests   Pass   Fail   Skip   Duration
L0        8      8      0      0     12s
L1        9      9      0      0     28s
L2       10     10      0      0     45s
L3       14     13      1      0     3m 15s
L5        5      5      0      0     42s

TOTAL:   46     45      1      0     5m 22s
STATUS: FAILED (L3: TC-L3-004 attended transfer timeout)
```

### Key Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| L0-L2 pass rate | 100% | Per commit |
| L3 pass rate | 100% | Per PR |
| Test execution time (L0-L3) | < 10 minutes | Per run |
| Test flake rate | < 2% | Rolling 30 days |
| Defect escape rate | < 1 S0/S1 per release | Per release |

### Results Storage

- **Console output**: pytest default (immediate feedback)
- **JUnit XML**: `tests/results/junit.xml` (CI integration)
- **HTML report**: `tests/results/report.html` (pytest-html, optional)
- **SIPp logs**: `tests/results/sipp/` (per-scenario CSV and logs)

---

## 9. Test Execution

### Local Development

```bash
# Start test environment
docker compose -f tests/docker-compose.test.yml up -d

# Wait for services
python tests/wait_for_services.py

# Run all automated tests (L0-L3, L5)
docker compose -f tests/docker-compose.test.yml run test-tools \
    pytest /tests/ -v --tb=short --junitxml=/tests/results/junit.xml

# Run specific level
docker compose -f tests/docker-compose.test.yml run test-tools \
    pytest /tests/test_L0_smoke.py -v

# Teardown
docker compose -f tests/docker-compose.test.yml down -v
```

### Master Test Runner

```bash
# One-command full test suite
./tests/run_all.sh          # Linux/Mac
pwsh tests/run_all.ps1      # Windows
```

### On-Demand Tests (L4 Telnyx)

```bash
# Requires TELNYX_SIP_USER and TELNYX_SIP_PASS environment variables
docker compose -f tests/docker-compose.test.yml run \
    -e TELNYX_SIP_USER -e TELNYX_SIP_PASS \
    test-tools pytest /tests/test_L4_integration.py -v
```

---

## 10. Directory Structure

```
tests/
├── conftest.py                    # pytest fixtures (Docker, API, SIP clients)
├── requirements.txt               # Python test dependencies
├── docker-compose.test.yml        # Test environment orchestration
├── Dockerfile.test-tools          # SIPp + PJSUA + Python test image
├── wait_for_services.py           # Service readiness checker
├── run_all.sh                     # Master test runner (Linux/Mac)
├── run_all.ps1                    # Master test runner (Windows)
├── config/
│   └── test-config.toml           # RustPBX test configuration
├── fixtures/
│   └── silence_8k_mono.wav        # Reference audio file
├── sipp/
│   ├── register.xml               # SIPp REGISTER scenario
│   ├── register_auth.xml          # SIPp REGISTER with 401 auth
│   ├── uac_invite.xml             # SIPp UAC INVITE scenario
│   ├── uas_answer.xml             # SIPp UAS answer scenario
│   ├── options_ping.xml           # SIPp OPTIONS scenario
│   └── load_register.xml          # SIPp load test: concurrent registrations
├── test_L0_smoke.py               # L0: Container health, ports, config
├── test_L1_infra.py               # L1: SIP/HTTP/WS connectivity
├── test_L2_api.py                 # L2: Console and AMI API tests
├── test_L3_sip.py                 # L3: Call flows, features, CDR
├── test_L4_integration.py         # L4: Telnyx PSTN (on-demand)
├── test_L5_media.py               # L5: Recording, codec, audio quality
├── test_L6_load.py                # L6: SIPp load/stress tests
├── test_L7_failover.py            # L7: Crash recovery, reconnection
├── test_L8_security.py            # L8: Auth bypass, fuzzing, injection
└── results/                       # Test output (gitignored)
    ├── junit.xml
    └── sipp/
```

---

*Document: `docs/TESTING_STRATEGY.md` — Stage 1 deliverable for RustPBX-y21*
