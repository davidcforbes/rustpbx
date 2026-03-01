# Plan: Port Upstream RustPBX Changes

## Summary
Port high-value updates from upstream restsend/rustpbx (18 commits, 2026-02-23 to 2026-02-28) into our fork. Focus on bug fixes, SipFlow/transcription improvements, media proxy enhancements, and SipFlow upload functionality. Skip features that conflict with our 4iiz architecture (notifications, MFA, console-only features). Total estimated scope: ~3,500 LOC across 6 major features.

## Motivation
Upstream has fixed several issues relevant to our deployment:
- SipFlow audio format handling (PCM fallback) that could eliminate our u-law wrapper hack
- SipFlow upload to S3/HTTP for call recording management
- Media proxy auto-detection for mixed WebRTC/SIP scenarios
- Trunk registrar for SIP providers requiring outbound REGISTER
- MediaBridge refactoring reducing duplication
- Foundation for voicemail/IVR via Call Application Framework

Remaining 3 bug fixes already implemented in our fork independently. Skipping console-specific features (notifications, archiving UI), auth (MFA/LDAP, handled by 4iiz), and license system (not relevant).

## Files
**New files to create:**
- `src/sipflow_upload.rs` (SipFlow to S3/HTTP hook)
- `src/metrics.rs` (Prometheus instrumentation)
- `src/addons/observability/mod.rs` (metrics addon)
- `src/proxy/trunk_registrar.rs` (outbound REGISTER)
- `src/call/mod.rs`, `src/call/app/mod.rs` (CallApp framework)

**Modified files:**
- `src/proxy/proxy_call/media_bridge.rs` (refactor forwarders)
- `src/proxy/proxy_call/session.rs` (media proxy detection, WebRTC codec handling)
- `src/addons/transcript/handlers.rs` (PCM/SipFlow fallback)
- `src/sipflow/wav_utils.rs` (PCM format support)
- `src/config.rs` (SipFlow upload, trunk registrar config)
- `Cargo.toml` (new optional dependencies)

---

## Phase 1: SipFlow PCM Format + Transcription Fallback

**Description:**
Port transcription handler improvements from upstream to add PCM audio format support and SipFlow fallback when no recording file exists. This fixes compatibility with ASR systems that require 16-bit PCM instead of u-law, potentially eliminating our custom groq-sensevoice-wrapper.

**Design:**
- Add `AudioSource` enum (`File(PathBuf)` vs `SipFlow(SipFlowId)`) to abstract audio sources
- Modify `TranscriptHandler::handle()` to check for recording file, fall back to SipFlow query if missing
- Add `generate_wav_from_packets_ex()` with `force_pcm: bool` parameter in `wav_utils.rs`
- Remote SipFlow backend requests `format=pcm` by default
- Update `sensevoice_cli_command()` to accept PCM input directly (no u-law conversion)
- Retain backward compatibility: if `force_pcm=false`, use existing format detection

**Design References:**
- Upstream: `3d1fb10` (commit hash)
- Files: `src/addons/transcript/handlers.rs` (+151), `src/sipflow/wav_utils.rs`, `src/sipflow/backend/remote.rs`
- Test: Add unit test `test_transcription_fallback_to_sipflow_pcm()` with mock SipFlow backend

**Acceptance Criteria:**
- PCM format parameter added to `generate_wav_from_packets_ex()`
- `AudioSource` enum implemented with File and SipFlow variants
- Transcription handler queries SipFlow when recording file missing
- Remote SipFlow backend requests format=pcm
- groq-sensevoice-wrapper compatibility tested (transcription produces correct output from PCM input)
- Backward compatibility maintained for existing code paths
- Unit tests pass for both file and SipFlow sources
- No breaking changes to existing transcription API

**Parallel:** no

---

## Phase 2: MediaBridge Track Forwarder Refactor

**Description:**
Extract repeated track-forwarder setup logic in MediaBridge into a closure, reducing duplication and improving maintainability. This is a pure refactor with no behavior change.

**Design:**
- Identify repeated `track_rx.subscribe()` + `tokio::spawn()` + `loop { select! }` pattern in `start_forwarding_audio()` and `start_forwarding_video()`
- Extract into `start_forwarder()` closure that takes track kind, source/sink channels, and packet processor
- Reduce `media_bridge.rs` by ~50 lines while maintaining identical behavior
- Add comments documenting the forwarder lifecycle and error handling

**Design References:**
- Upstream: `a7ace3c` (commit hash)
- File: `src/proxy/proxy_call/media_bridge.rs` (+138, -98)

**Acceptance Criteria:**
- `start_forwarder()` closure defined and documented
- Both audio and video forwarders use the same closure
- No behavior change: all existing media forwarding tests pass
- Code duplication reduced by >50 lines
- Compilation succeeds without warnings
- No performance regression in media bridge throughput

**Parallel:** no

---

## Phase 3: Media Proxy Auto-Detection Improvement

**Description:**
Enhance media proxy auto-detection to properly handle mixed WebRTC/SIP scenarios. Currently only checks for SAVPF presence; should enable proxy when caller is WebRTC but targets are not (or vice versa).

**Design:**
- Modify `check_media_proxy()` signature to accept `all_webrtc_target: bool` parameter
- Update `Auto` mode logic: enable proxy if `(is_webrtc_caller && !all_webrtc_target) || (!is_webrtc_caller && all_webrtc_target)`
- Update all callers of `check_media_proxy()` to determine and pass `all_webrtc_target`
- Document the three scenarios: all-WebRTC (no proxy), all-SIP (no proxy), mixed (proxy required)

**Design References:**
- Upstream: contained in `cb4d772` commit
- File: `src/proxy/proxy_call/session.rs`
- Related test: `test_media_proxy_mixed_webrtc_sip()`

**Acceptance Criteria:**
- `all_webrtc_target` parameter added to `check_media_proxy()`
- Auto mode correctly identifies mixed scenarios
- All callers updated with target WebRTC detection
- Unit tests cover: all-WebRTC, all-SIP, mixed scenarios
- Backward compatibility maintained for explicit Enabled/Disabled modes
- No regressions in existing media proxy tests

**Parallel:** yes

---

## Phase 4: SipFlow Upload to S3/HTTP

**Description:**
After call completion, reconstruct WAV from SipFlow RTP packets and upload to configured S3 bucket or HTTP endpoint. Set `recording_url` and `recording_duration_secs` on call record for downstream access.

**Design:**
- New `SipFlowUploadHook` trait implementation in `src/callrecord/sipflow_upload.rs` (382 lines)
- New config enum `SipFlowUploadConfig` with `S3 { bucket, prefix, region }` and `HTTP { url, auth_header }` variants
- New `[sipflow.upload]` config section with `enabled = true`, `target = "s3"` or `"http"`
- On `CallRecord::completed()` event, instantiate `SipFlowUploadHook`, query SipFlow, reconstruct WAV, upload
- After successful upload, update `call_records` table with `recording_url` and `recording_duration_secs`
- Handle errors gracefully: log and continue if upload fails (don't fail the entire call completion)
- Cleanup: expire SipFlow data after successful upload (optional config: `cleanup_sipflow_after_upload`)

**Design References:**
- Upstream: `73c0d0b` (commit hash)
- New file: `src/callrecord/sipflow_upload.rs` (+382)
- Modified: `src/config.rs` (SipFlowUploadConfig enum)
- Modified: `src/callrecord/mod.rs` (hook integration)
- Test: Unit tests for S3 and HTTP upload paths, WAV reconstruction validation

**Acceptance Criteria:**
- `SipFlowUploadConfig` enum implemented with S3 and HTTP variants
- `SipFlowUploadHook` reconstructs WAV from SipFlow packets
- S3 upload works with AWS SDK (boto3 equivalent in Rust)
- HTTP upload works with custom auth headers
- `call_records.recording_url` and `recording_duration_secs` populated after upload
- Config validation: reject invalid S3 regions or HTTP URLs
- Error handling: failed uploads don't block call completion
- Unit tests pass for both S3 and HTTP backends
- Optional: test integration with real S3 bucket (gated behind test feature)

**Parallel:** yes

---

## Phase 5: Trunk Registrar for Outbound REGISTER

**Description:**
Implement outbound SIP REGISTER client for trunk providers requiring registration. Handles digest auth, periodic refresh before expiry, and graceful un-REGISTER on shutdown.

**Design:**
- New `TrunkRegistrar` struct managing registration state per trunk
- New `trunk_registrar.rs` module (427 lines) with:
  - `RegistrarTask`: Background task sending periodic REGISTER requests
  - `RegistrationState` enum: Unregistered, Registered (with expiry timer), Failed
  - Digest auth client for AUTH challenge responses
  - Config reconciliation on reload (stop/start registrations as needed)
- New model fields on `sip_trunk`:
  - `register_enabled: bool` (default: false)
  - `register_expires: i32` (seconds, default: 3600)
  - `register_extra_headers: Option<String>` (Contact, Route, etc.)
- New migration: add these 3 columns to `sip_trunks` table
- New API endpoint: `GET /ami/trunk_registrations` returns status for all trunks (registered, failed, next_refresh)
- Integration: Wire `TrunkRegistrar` into `SipServer::new()`, auto-start on init, clean shutdown on drop

**Design References:**
- Upstream: contained in `cb4d772` commit (extract only the registrar, ignore notifications/MFA)
- New file: `src/proxy/trunk_registrar.rs` (+427)
- Modified: `src/models/sip_trunk.rs` (add 3 fields)
- Modified: `src/app.rs` or `src/bin/rustpbx.rs` (initialization)
- Modified: `src/ami/handlers.rs` (new endpoint)
- Test: Mock SIP server with REGISTER/401 auth challenge response

**Acceptance Criteria:**
- `TrunkRegistrar` sends periodic REGISTER for enabled trunks
- Digest auth implemented and tested
- Registration state transitions: Unregistered → Registered (on 200 OK)
- Refresh scheduled before expiry (subtract margin, e.g., 5 minutes)
- Failed REGISTER retried with exponential backoff
- Config reload reconciles registrations (add/remove as needed)
- Graceful shutdown un-REGISTERs all trunks
- `GET /ami/trunk_registrations` returns accurate status for all trunks
- Unit tests cover: successful registration, auth challenge, expiry refresh, config reload, graceful shutdown
- No blocking of main SIP call processing during registration

**Parallel:** yes

---

## Phase 6: Observability/Prometheus Metrics

**Description:**
Instrument RustPBX with Prometheus metrics for monitoring call volume, duration, and system health. Add `/metrics` scrape endpoint and `/healthz` liveness probe.

**Design:**
- New `src/metrics.rs` (491 lines) with:
  - `rustpbx_calls_total` counter (label: direction=inbound|outbound, outcome=completed|failed|abandoned)
  - `rustpbx_call_duration_seconds` histogram (buckets: 1, 5, 10, 30, 60, 300)
  - `rustpbx_call_talk_time_seconds` histogram (same buckets)
  - `rustpbx_sip_registrations_active` gauge (per trunk)
  - `rustpbx_sip_registrations_failed` counter
- New addon: `src/addons/observability/mod.rs` (501 lines)
  - `ObservabilityAddon` implementing `Addon` trait
  - Registers `/metrics` handler (Prometheus format)
  - Registers `/healthz` handler (returns 200 if all services healthy)
  - Config: `[addons.observability]` with `enabled = true`, `metrics_path = "/metrics"`
- Feature flag: `addon-observability` (optional, default: off)
- Dependencies: `metrics 0.24`, `metrics-exporter-prometheus 0.16` (behind feature)
- Integration: Wire into `AddonRegistry` during app startup

**Design References:**
- Upstream: `be7b278` commits (metrics.rs + observability addon)
- Files: `src/metrics.rs` (+491), `src/addons/observability/mod.rs` (+501)
- Template: `templates/console/metrics.html` (view in console UI)

**Acceptance Criteria:**
- All 5 metrics exposed via `/metrics` endpoint
- Metrics collected for every call (inbound/outbound, success/failure)
- Prometheus format valid (can be scraped by real Prometheus server)
- `/healthz` endpoint returns 200 for healthy system, 503 for degraded
- Feature flag works: metrics disabled when `addon-observability` feature off
- Zero performance impact when addon disabled
- Console template renders metrics dashboard (nice-to-have)
- Unit tests mock metric recording and verify endpoint responses
- Tested with Prometheus scrape endpoint (local test)

**Parallel:** yes

---

## Phase 7: Call Application Framework (Voicemail/IVR Foundation)

**Description:**
Implement trait-based framework for building call applications (IVR, voicemail, etc.) with lifecycle hooks, controller API, and event loop. Foundation for voicemail and other call applications.

**Design:**
- New `src/call/` module with:
  - `CallApp` trait: `on_enter()`, `on_dtmf()`, `on_audio_complete()`, `on_timeout()`, `on_exit()`
  - `CallController` struct: `answer()`, `hangup()`, `play_audio()`, `collect_dtmf()`, `set_timeout()`, `get_variable()`, `set_variable()`
  - `ApplicationContext` struct: shared state, session variables, DB pool access, SIP session handle
  - `AppEventLoop` struct: drives app lifecycle with timer support, event dispatch
  - `MockCallStack` for testing apps without SIP stack
- New route action: `action = "application"` with fields `app` (app name) and `app_params` (JSON config)
- Integration:
  - `Dialplan` gets `with_application(app_name, params)` method
  - `CallSession::run_application()` builds controller and spawns event loop
  - Apps registered in `ApplicationRegistry` and instantiated by name
  - Lifecycle: app `on_enter()` → event loop → `on_exit()`
- Documentation: Example apps (simple IVR, voicemail stub)

**Design References:**
- Upstream: `be7b278` + `46ede97` commits
- Files: `src/call/mod.rs`, `src/call/app/mod.rs`, `src/call/app/controller.rs`, `src/call/app/event_loop.rs`, `src/call/app/testing.rs`
- Tests: `src/call/app/app_test.rs` (comprehensive test coverage)

**Acceptance Criteria:**
- `CallApp` trait defined with 5 lifecycle hooks
- `CallController` implements all 7 call control operations
- `ApplicationContext` provides variable storage and DB access
- `AppEventLoop` drives app with timer support (dtmf_timeout, play_complete)
- `MockCallStack` enables testing without SIP stack
- Route action parses `action = "application"` correctly
- App instantiation by name works (factory pattern)
- Lifecycle test: `on_enter()` → events → `on_exit()` called in order
- Example IVR app (collect digits, play back) passes all tests
- No SIP-layer regressions (call routing still works for non-app routes)
- Documentation covers trait interface and testing patterns

**Parallel:** yes

---

## Phase 8: Optional Enhancements (Lower Priority)

**Description:**
Port additional features if time permits. These are lower priority but valuable for future development.

**Design:**
1. **Addon Registry Refactoring** — Move from hardcoded addon init to trait-based registry (small, clean architecture improvement)
2. **Voicemail Addon Skeleton** — Placeholder for future voicemail implementation (11 lines, minimal content)
3. **License System Enhancements** — Free-trial support, in-process caching (if we plan to use licensing)
4. **Dependency Updates** — Optional: make `ldap3` feature-gated for enterprise auth (good hygiene)

**Design References:**
- Upstream: `be7b278` + `46ede97` commits
- Files: `src/addons/mod.rs`, `src/addons/registry.rs`, `src/license.rs`, `Cargo.toml`

**Acceptance Criteria:**
- Registry pattern understood and documented
- Addon initialization cleaner and more extensible
- No breaking changes to existing addon interface
- All existing addons work with new registry
- Voicemail skeleton structure matches upstream layout
- License enhancements don't break existing code
- Feature gates compile cleanly when disabled

**Parallel:** yes

---

## Dependencies to Add

```toml
# In Cargo.toml
metrics = "0.24"
metrics-exporter-prometheus = { version = "0.16", optional = true }
opentelemetry = { version = "0.27", optional = true }
opentelemetry_sdk = { version = "0.27", optional = true }
opentelemetry-otlp = { version = "0.27", optional = true }
tracing-opentelemetry = { version = "0.28", optional = true }

# Existing features, make ldap3 optional if not already
[features]
addon-observability = ["metrics-exporter-prometheus"]
addon-telemetry = ["opentelemetry", "opentelemetry_sdk", "opentelemetry-otlp", "tracing-opentelemetry"]
addon-enterprise-auth = ["ldap3"]
```

---

## Success Criteria (Overall)

✅ All 7 phases implemented and tested
✅ No conflicts with 4iiz architecture (notifications/auth delegated to 4iiz)
✅ Backward compatibility maintained for SIP call routing and recording
✅ New metrics exposed and validated with Prometheus
✅ SipFlow PCM support eliminates groq-sensevoice-wrapper hack
✅ All upstream commits cleanly ported (except console-only features)
✅ Git history: commits authored/co-authored with upstream maintainer acknowledgment
✅ No performance regressions (media bridge refactor, metric collection overhead <1%)
