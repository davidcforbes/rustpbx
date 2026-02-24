# RustPBX Master Roadmap

**Created:** 2026-02-24
**Tracking:** Beads issue tracker (prefix: rpbx-)
**Total Issues:** 78 (12 epics + 66 tasks)

---

## Epic Summary

| # | Epic ID | Title | Priority | Tasks | Status |
|---|---------|-------|----------|-------|--------|
| 1 | rpbx-48g | Fix stereo transcriptions L/R | P1 | 5 | Ready |
| 2 | rpbx-101 | Add voicemail system | P2 | 8 | Ready |
| 3 | rpbx-09k | Baseline test regression (100% automated) | P1 | 10 | Ready |
| 4 | rpbx-lld | Performance/scalability tests (100 calls) | P2 | 8 | Ready |
| 5 | rpbx-eid | Architecture documentation | P2 | 8 | Ready |
| 6 | rpbx-f5n | Code review (security/stability/perf) | P2 | 6 | Ready |
| 7 | rpbx-c68 | Reverse engineer CTM UI | P3 | 2 | Blocked (screenshots) |
| 8 | rpbx-p2v | Reverse engineer Zoho UI | P3 | 2 | Blocked (screenshots) |
| 9 | rpbx-cr1 | Reverse engineer Flow Legal UI | P3 | 2 | Blocked (screenshots) |
| 10 | rpbx-5bl | Reverse engineer 4E's UI | P3 | 2 | Blocked (screenshots) |
| 11 | rpbx-mwi | Resilient infrastructure/clustering | P2 | 7 | Ready |
| 12 | rpbx-kng | Backup and recovery | P2 | 6 | Ready |

---

## 1. Fix Stereo Transcriptions (rpbx-48g) - P1

**Problem:** Recording creates stereo WAV (Leg A=left=caller, Leg B=right=callee). The `groq-sensevoice-wrapper` transcribes both channels correctly when run manually, but the RustPBX transcript addon doesn't auto-trigger after recording, and channel labels don't map to caller/callee.

**Key Files:**
- `src/addons/transcript/mod.rs` - Addon trigger mechanism
- `src/media/recorder.rs:70-90` - Leg A/B buffer assignment
- `src/proxy/proxy_call/reporter.rs` - Recording completion handler
- `src/models/call_record.rs` - CDR schema (needs transcript_text column)
- `src/console/handlers/call_record.rs` - UI display

**Tasks:**
1. `rpbx-78r` [P1/bug] Fix auto-transcription trigger after recording completes
2. `rpbx-1h3` [P1] Add caller/callee channel labels to transcript output
3. `rpbx-v7x` [P1] Store per-channel transcripts in call records DB
4. `rpbx-0sl` [P2] Display per-channel transcripts in console UI
5. `rpbx-gkj` [P3] Add transcript search capability to call records

**Dependency Chain:** rpbx-78r -> rpbx-v7x -> rpbx-0sl

---

## 2. Add Voicemail System (rpbx-101) - P2

**Current State:** Only a `voicemail_disabled` boolean flag on extensions. No voicemail functionality.

**Architecture:**
- New `src/addons/voicemail/` addon module
- Database tables: `voicemail_mailbox`, `voicemail_message`
- Storage: `config/voicemail/{ext}/` for greetings and messages
- Integration points: call session (no-answer forwarding), DTMF menu (*97/*98), MWI (SIP NOTIFY)

**Tasks:**
1. `rpbx-101.1` [P1] Design voicemail storage schema and data model
2. `rpbx-101.2` [P2] Implement voicemail greeting recording and management
3. `rpbx-101.3` [P1] Implement voicemail message recording
4. `rpbx-101.4` [P1] Implement call forwarding to voicemail on no-answer/busy
5. `rpbx-101.5` [P2] Implement MWI via SIP NOTIFY
6. `rpbx-101.6` [P2] Implement voicemail retrieval via DTMF menu
7. `rpbx-101.7` [P3] Add voicemail-to-email notification
8. `rpbx-101.8` [P2] Add voicemail management to console UI

**Execution Order:** 1 first -> 2 and 4 in parallel -> 3 (after 4) -> 5/6/7/8 in parallel

---

## 3. Baseline Test Regression (rpbx-09k) - P1

**Current State:** Existing tests: `call_quality_test.py` (smoke/fidelity/transcription), `e2e_call_test.py`, `sip_test_call.py`, Python framework with L0-L8 stubs, 307 Rust unit tests passing.

**Target:** 100% automated end-to-end coverage, zero manual intervention, CI/CD integrated.

**Tasks:**
1. `rpbx-09k.1` [P1] L0 smoke tests (health, ports, config)
2. `rpbx-09k.2` [P1] L1 infrastructure tests (TLS, disk, memory)
3. `rpbx-09k.3` [P1] L2 API tests (AMI, console, CRUD)
4. `rpbx-09k.4` [P1] L3 SIP protocol tests (REGISTER, INVITE, BYE, CANCEL, REFER)
5. `rpbx-09k.5` [P1] L5 media/codec tests (Opus, G722, PCMU, DTMF, hold music)
6. `rpbx-09k.6` [P2] L6 load tests (10-25 concurrent calls)
7. `rpbx-09k.7` [P2] L7 failover tests (restart, reconnect, drain)
8. `rpbx-09k.8` [P2] L8 security tests (injection, auth, rate limiting)
9. `rpbx-09k.9` [P1] End-to-end call flow tests (internal, PSTN, transfer, queue)
10. `rpbx-09k.10` [P2] CI/CD pipeline with GitHub Actions

---

## 4. Performance & Scalability Tests (rpbx-lld) - P2

**Target:** Verify RustPBX handles 100 simultaneous calls.

**Key Constraints:**
- Default RTP port range: 20000-20100 (100 ports) - needs expansion for 100 calls
- Single-threaded Tokio runtime may bottleneck
- SQLite concurrent CDR writes could contend

**Tasks:**
1. `rpbx-lld.1` [P1] Build SIP load generator with concurrent UA pool
2. `rpbx-lld.2` [P1] 10-call concurrent test (baseline)
3. `rpbx-lld.3` [P2] 25-call concurrent test
4. `rpbx-lld.4` [P2] 50-call concurrent test
5. `rpbx-lld.5` [P2] 100-call concurrent test (target)
6. `rpbx-lld.6` [P2] Long-duration stability test (1hr+)
7. `rpbx-lld.7` [P3] Performance metrics dashboard
8. `rpbx-lld.8` [P3] Scalability limits documentation

**Chain:** 1 -> 2 -> 3 -> 4 -> 5 -> 7,8

---

## 5. Architecture Documentation (rpbx-eid) - P2

**Scope:** Document the entire system for onboarding, operations, and future development.

**Tasks:**
1. `rpbx-eid.1` [P1] System architecture overview (Mermaid diagrams)
2. `rpbx-eid.2` [P1] SIP call flow sequences
3. `rpbx-eid.3` [P2] Media pipeline architecture
4. `rpbx-eid.4` [P2] Database schema and data model
5. `rpbx-eid.5` [P2] API reference documentation
6. `rpbx-eid.6` [P2] Configuration reference
7. `rpbx-eid.7` [P2] Deployment and operations guide
8. `rpbx-eid.8` [P3] Developer onboarding guide

**Output:** `docs/architecture/` directory

---

## 6. Code Review (rpbx-f5n) - P2

**Scope:** Systematic security, stability, scalability, and performance audit.

**Tasks:**
1. `rpbx-f5n.1` [P1] Security audit - SIP protocol layer
2. `rpbx-f5n.2` [P1] Security audit - web and API layer
3. `rpbx-f5n.3` [P1] Stability audit - error handling and panics
4. `rpbx-f5n.4` [P2] Scalability audit - async and concurrency
5. `rpbx-f5n.5` [P2] Performance audit - hot paths
6. `rpbx-f5n.6` [P3] Code quality audit - maintainability

**Key Targets:** session.rs (202KB), media_bridge.rs (32KB), auth.rs, routing engine

---

## 7-10. UI Reverse Engineering (rpbx-c68, rpbx-p2v, rpbx-cr1, rpbx-5bl) - P3

**Status: BLOCKED** - Awaiting user-provided screenshots for:
- **7. CTM** (rpbx-c68) - CallTrackingMetrics contact center platform
- **8. Zoho** (rpbx-p2v) - Zoho Voice/CRM telephony
- **9. Flow Legal** (rpbx-cr1) - Legal case management
- **10. 4E's** (rpbx-5bl) - Pending clarification

Each has 2 tasks: (1) Screenshot analysis -> feature requirements, (2) Gap mapping -> implementation plan.

---

## 11. Resilient Infrastructure (rpbx-mwi) - P2

**Current State:** Single-instance only. No clustering, no HA, all state local.

**Tasks:**
1. `rpbx-mwi.1` [P1] Design SIP load balancing architecture
2. `rpbx-mwi.2` [P1] Design shared state architecture for clustering
3. `rpbx-mwi.3` [P2] Design media server separation
4. `rpbx-mwi.4` [P1] Design health monitoring and automatic failover
5. `rpbx-mwi.5` [P3] Design geographic redundancy
6. `rpbx-mwi.6` [P2] Design trunk registration coordination
7. `rpbx-mwi.7` [P2] Design configuration sync across cluster

**Key Decisions Needed:**
- SIP LB: DNS SRV vs OpenSIPS vs HAProxy vs custom
- State sharing: Redis vs distributed DB vs MySQL Galera
- Media plane: separated vs co-located

---

## 12. Backup and Recovery (rpbx-kng) - P2

**Data to Protect:** Config files, database (CDR, extensions, routes), recordings (WAV, potentially TB), certificates, transcripts.

**Tasks:**
1. `rpbx-kng.1` [P1] Design backup strategy and RTO/RPO targets
2. `rpbx-kng.2` [P1] Implement automated database backup
3. `rpbx-kng.3` [P2] Implement recording backup with incremental sync
4. `rpbx-kng.4` [P2] Implement configuration backup and version control
5. `rpbx-kng.5` [P2] Build disaster recovery runbook
6. `rpbx-kng.6` [P3] Implement backup monitoring and alerting

---

## Recommended Execution Order

### Phase 1 - Foundation (Immediate)
1. **rpbx-48g** Fix stereo transcriptions (P1, 5 tasks) - quick win, already mostly working
2. **rpbx-09k** Baseline tests (P1, 10 tasks) - establishes safety net for all future work

### Phase 2 - Quality & Confidence
3. **rpbx-f5n** Code review (P2, 6 tasks) - find and fix issues before adding features
4. **rpbx-eid** Architecture docs (P2, 8 tasks) - document current state before changing it

### Phase 3 - Features
5. **rpbx-101** Voicemail (P2, 8 tasks) - major user-facing feature
6. **rpbx-lld** Performance tests (P2, 8 tasks) - validate capacity

### Phase 4 - Infrastructure
7. **rpbx-kng** Backup/recovery (P2, 6 tasks) - protect existing data
8. **rpbx-mwi** Clustering/HA (P2, 7 tasks) - production readiness

### Phase 5 - Competitive Analysis (when screenshots provided)
9. **rpbx-c68** CTM analysis (P3)
10. **rpbx-p2v** Zoho analysis (P3)
11. **rpbx-cr1** Flow Legal analysis (P3)
12. **rpbx-5bl** 4E's analysis (P3)
