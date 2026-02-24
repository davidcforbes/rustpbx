# RustPBX Comprehensive Code Review Report

**Date:** 2026-02-23
**Reviewer:** Claude Opus 4.6 (automated code audit)
**Scope:** Security, Stability, Scalability, Performance
**Codebase Version:** Commit `d66a780` (main branch)

---

## Executive Summary

RustPBX is a well-structured Rust PBX system with generally sound architecture. The use of Rust's type system provides inherent memory safety. However, this audit identified several issues across security, stability, and performance categories that should be addressed. The most critical findings relate to missing CSRF protection on web forms, IP spoofing via trusted proxy headers, missing `Secure` cookie flag, and potential panics from `unwrap()` calls on mutex locks in production code paths.

---

## 1. Security Findings

### 1.1 SIP Layer Security

#### SEC-SIP-01: SIP Digest Authentication Uses MD5 Only
- **File:** `src/proxy/auth.rs`, lines 174-188
- **Severity:** Medium
- **Description:** The authentication challenge exclusively uses MD5 for digest authentication (`algorithm=MD5`). MD5 is considered cryptographically weak and susceptible to collision attacks. While SIP digest auth is not a simple hash (it incorporates nonce), upgrading to SHA-256 would improve security posture.
- **Suggested Fix:** Support `algorithm=SHA-256` (RFC 8760) in addition to MD5, preferring SHA-256 when the client supports it.

#### SEC-SIP-02: Nonce Not Validated Against Replay Attacks
- **File:** `src/proxy/auth.rs`, lines 173-188
- **Severity:** Medium
- **Description:** The nonce generated for authentication challenges is random but there is no server-side tracking or validation. The server generates a nonce for the challenge but does not verify that the nonce in the client's response was actually issued by the server, nor does it check for nonce reuse. This opens the possibility of replay attacks where a captured valid authentication can be replayed.
- **Suggested Fix:** Implement a nonce cache (with TTL) to track issued nonces. Validate that the nonce in the Authorization header was recently issued by the server and has not been used before. Consider `qop=auth` with nonce-count (nc) validation.

#### SEC-SIP-03: ACL Default Rules Allow All Traffic
- **File:** `src/proxy/acl.rs`, lines 358-363
- **Severity:** Low
- **Description:** When no ACL rules are configured (`acl_rules: None`), the default rules are `["allow all", "deny all"]`. Since rules are evaluated in order and `allow all` comes first, this effectively allows all traffic. The `deny all` rule is unreachable.
- **Suggested Fix:** This is by design for ease of initial setup, but consider logging a warning on startup when the effective ACL is "allow all" in production environments.

#### SEC-SIP-04: User-Agent Whitelist Uses Exact String Match
- **File:** `src/proxy/acl.rs`, lines 203-211
- **Severity:** Low
- **Description:** The UA whitelist/blacklist uses exact string matching (`HashSet::contains`). Attackers can trivially bypass this by slightly modifying their User-Agent string. The whitelist also matches the entire UA string, which is fragile since UA strings frequently change between software versions.
- **Suggested Fix:** Consider supporting substring or regex matching for UA filtering, or at minimum prefix matching.

#### SEC-SIP-05: ACL Bypass for Trunk IPs Skips All Checks
- **File:** `src/proxy/acl.rs`, lines 325-333
- **Severity:** Low
- **Description:** When a request comes from a recognized trunk IP, the ACL module immediately returns `Continue` and sets a `TrunkContext`, bypassing all further ACL rules. This is expected behavior, but if a trunk IP is compromised, there is no defense-in-depth.
- **Suggested Fix:** Consider adding optional per-trunk ACL rules or rate limiting even for trusted trunk IPs.

### 1.2 Web/API Layer Security

#### SEC-WEB-01: Missing CSRF Protection on All Form Endpoints
- **File:** `src/console/handlers/user.rs`, lines 76-152 (login), 224-328 (register), 341-395 (forgot), 444-524 (reset)
- **Severity:** High
- **Description:** All form-based POST endpoints (login, register, forgot password, reset password) lack CSRF protection. There are no CSRF tokens generated or validated. An attacker can craft a malicious page that submits forms to these endpoints while the victim is authenticated.
- **Suggested Fix:** Implement CSRF tokens. Generate a random token per session, embed it in forms as a hidden field, and validate it on each POST request. Alternatively, use the `SameSite` cookie attribute (see SEC-WEB-02).

#### SEC-WEB-02: Session Cookie Missing SameSite and Secure Attributes
- **File:** `src/console/auth.rs`, lines 63-113
- **Severity:** High
- **Description:** The session cookie is set with `HttpOnly` but is missing the `SameSite` attribute entirely. The `Secure` flag computation is present (`_is_secure` variable at lines 64, 97) but is deliberately overridden with `let secure_attr = "";` -- the Secure attribute is never actually applied. This means:
  - Without `SameSite`, the cookie is sent on cross-site requests, enabling CSRF attacks.
  - Without `Secure`, the session cookie can be transmitted over unencrypted HTTP connections, allowing session hijacking via network sniffing.
- **Suggested Fix:** Set `SameSite=Lax` (or `Strict`) on all session cookies. Restore the `Secure` flag when HTTPS is configured. The code already computes `_is_secure` but does not use it.

#### SEC-WEB-03: IP Address Spoofing via X-Forwarded-For
- **File:** `src/handler/middleware/clientaddr.rs`, lines 39-54
- **Severity:** High
- **Description:** The `ClientAddr` extractor unconditionally trusts `X-Forwarded-For`, `X-Real-IP`, `X-Client-IP`, and `CF-Connecting-IP` headers from any client. When the server is not behind a trusted reverse proxy, an attacker can send a request directly with a spoofed `X-Forwarded-For` header to:
  - Bypass AMI IP-based access controls (`src/handler/middleware/ami_auth.rs`)
  - Fake their login IP in the audit log (`mark_login`)
  - Bypass any IP-based ACL or rate limiting
- **Suggested Fix:** Only trust proxy headers when the request comes from a known proxy IP. Add a configurable list of trusted proxy IPs. When the direct connection is not from a trusted proxy, ignore `X-Forwarded-For` and related headers.

#### SEC-WEB-04: `is_secure_request` Trusts X-Forwarded-Proto Unconditionally
- **File:** `src/console/handlers/user.rs`, lines 20-27
- **Severity:** Medium
- **Description:** The `is_secure_request()` function trusts the `X-Forwarded-Proto` header from any client. An attacker can send `X-Forwarded-Proto: https` on an HTTP connection to influence cookie security attributes and other HTTPS-dependent behavior.
- **Suggested Fix:** Only trust this header from known reverse proxy IPs (same fix as SEC-WEB-03).

#### SEC-WEB-05: Password Reset Token Logged in Plaintext
- **File:** `src/console/handlers/user.rs`, lines 363
- **Severity:** Medium
- **Description:** The password reset link (containing the reset token) is logged at INFO level: `info!("password reset link generated for {}: {}", email, link)`. This token is sensitive -- anyone with log access can use it to reset any user's password.
- **Suggested Fix:** Do not log the full reset token. Log only that a reset was generated for a specific email.

#### SEC-WEB-06: Password Reset Link Returned in HTTP Response
- **File:** `src/console/handlers/user.rs`, lines 358-364
- **Severity:** Medium
- **Description:** When a password reset is requested, the reset link is embedded directly in the response HTML (`reset_link = Some(link)`). In a production system, reset links should be sent via email, not displayed in the browser. The current behavior means anyone who can see the user's screen during the forgot-password flow can capture the token.
- **Suggested Fix:** Send the reset token via email. Only display a generic "check your email" message in the response. The current approach may be acceptable for small self-hosted deployments but should be documented as a limitation.

#### SEC-WEB-07: Open Redirect in Login Next Parameter
- **File:** `src/console/handlers/user.rs`, lines 154-170
- **Severity:** Low
- **Description:** The `resolve_next_redirect` function validates that the `next` parameter starts with `/` and does not start with `//` or contain `://`. This prevents most open redirect attacks but the validation could be more robust. The function prepends `base_path` to the candidate, which could lead to unexpected paths if `base_path` itself is manipulated.
- **Suggested Fix:** The current validation is reasonable. Consider additionally verifying the final redirect URL against an allowlist of valid path prefixes.

#### SEC-WEB-08: SIP Password Stored in Plaintext
- **File:** `src/console/handlers/extension.rs`, line 434
- **Severity:** Medium
- **Description:** SIP passwords for extensions are stored directly in the database without hashing: `sip_password: Set(payload.sip_password)`. While SIP digest authentication requires access to the plaintext password (or HA1 hash), the current storage means anyone with database access can read all SIP passwords.
- **Suggested Fix:** Store the HA1 hash (`MD5(username:realm:password)`) instead of the plaintext password. SIP digest authentication can work with the pre-computed HA1 value.

#### SEC-WEB-09: AMI Shutdown Endpoint Accessible to Superusers
- **File:** `src/handler/ami.rs`, lines 84-88
- **Severity:** Low
- **Description:** The `/ami/v1/shutdown` POST endpoint allows initiating a system shutdown. While protected by AMI auth (IP whitelist + console superuser), there is no confirmation or additional authentication step for this destructive action.
- **Suggested Fix:** Consider requiring an additional confirmation parameter or a separate, more restrictive auth mechanism for the shutdown endpoint.

#### SEC-WEB-10: Template Rendering May Expose Internal Errors
- **File:** `src/console/middleware.rs`, lines 84-106
- **Severity:** Low
- **Description:** Template rendering errors are returned to the client with the error message: `format!("Internal Server Error: {}", err)`. This could expose internal file paths or template structure details to attackers.
- **Suggested Fix:** Return a generic error message to the client. Log the detailed error server-side only.

### 1.3 Configuration and Credential Security

#### SEC-CFG-01: S3 Credentials in Config File
- **File:** `src/config.rs`, lines 311-321
- **Severity:** Medium
- **Description:** The `CallRecordConfig::S3` variant stores `access_key` and `secret_key` directly in the configuration struct. These are serialized to/from the config TOML file in plaintext. If the config file has excessive permissions, these credentials could be exposed.
- **Suggested Fix:** Support reading credentials from environment variables or a secrets manager. At minimum, document that config files containing S3 credentials should have restricted file permissions (600).

#### SEC-CFG-02: Trunk Auth Credentials in Config/Database
- **File:** `src/proxy/data.rs`, lines 835-836
- **Severity:** Medium
- **Description:** SIP trunk authentication credentials (username/password) are stored in plaintext in both config files and the database. The `TrunkConfig` struct contains `username: Option<String>` and `password: Option<String>`.
- **Suggested Fix:** Consider encrypting trunk passwords at rest. At minimum, ensure the password is not included in serialized output (e.g., add `#[serde(skip_serializing)]` to the password field in non-file contexts).

---

## 2. Stability Findings

### 2.1 Panic-Prone Code

#### STAB-01: `unwrap()` on Config Clone via Serialization
- **File:** `src/config.rs`, lines 808-809
- **Severity:** High
- **Description:** The `Clone` implementation for `Config` uses `toml::to_string(self).unwrap()` and `toml::from_str(&s).unwrap()`. If any field cannot be serialized or the serialized form cannot be deserialized (e.g., due to a schema change or unsupported value), this will panic and crash the entire process.
- **Suggested Fix:** Implement `Clone` properly using derive macro or manual field-by-field clone. If serialization-based cloning is required for specific fields, handle errors gracefully with `.expect("Config round-trip serialization must succeed")` at minimum, or better yet, return a `Result`.

#### STAB-02: `unwrap()` on RwLock in `get_sip_server`
- **File:** `src/console/mod.rs`, line 277
- **Severity:** High
- **Description:** `self.sip_server.read().unwrap().clone()` will panic if the RwLock is poisoned (which happens when a thread panics while holding the lock). This is in contrast to the safe pattern used elsewhere in the same file (line 200: `self.sip_server.read().ok().and_then(...)`) which gracefully handles poisoned locks.
- **Suggested Fix:** Use the same safe pattern: `self.sip_server.read().ok().and_then(|guard| guard.clone())`. Note that `get_sip_server()` and `sip_server()` are two different methods on the same struct with different safety levels -- this is confusing and error-prone.

#### STAB-03: `unwrap()` on RwLock Throughout `ProxyDataContext`
- **File:** `src/proxy/data.rs`, lines 79, 83, 87, 91, 95, 99, 110, 129, 153, 209, 212, 255, 281, 284, 322, 349, 352, 400, 426, 429, 475, 501, 546
- **Severity:** Medium
- **Description:** All `RwLock` accesses in `ProxyDataContext` use `.unwrap()`. If any thread panics while holding a lock, the RwLock becomes poisoned and all subsequent accesses will panic, cascading into a full system crash. This is particularly concerning because `ProxyDataContext` is a central shared state object accessed from many async tasks.
- **Suggested Fix:** Use `.read().unwrap_or_else(|e| e.into_inner())` to recover from poisoned locks, or use `parking_lot::RwLock` which does not have lock poisoning.

#### STAB-04: `unwrap()` on Mutex in ACME Handlers
- **File:** `src/addons/acme/handlers.rs`, lines 30, 42, 94, 120, 138, 160, 180, 190, 299, 308, 339, 361, 371, 416, 490
- **Severity:** Medium
- **Description:** All ACME state RwLock accesses use `.unwrap()`, creating potential for cascading panics if any ACME operation panics.
- **Suggested Fix:** Same as STAB-03.

#### STAB-05: `unwrap()` on Mutex in Registrar Module
- **File:** `src/proxy/registrar.rs`, lines 554, 656
- **Severity:** Medium
- **Description:** The registrar accesses `tx.endpoint_inner.allows.lock().unwrap()` which could panic on a poisoned mutex. Since this is in the SIP registration hot path, a panic here would prevent all future registrations.
- **Suggested Fix:** Use `.lock().unwrap_or_else(|e| e.into_inner())` or handle the error gracefully.

#### STAB-06: `unwrap()` in Transcript Handler
- **File:** `src/addons/transcript/handlers.rs`, line 522
- **Severity:** Low
- **Description:** `serde_json::to_string_pretty(&stored_transcript).unwrap()` can panic if the transcript data contains values that cannot be serialized (unlikely but possible with NaN/Infinity floats).
- **Suggested Fix:** Handle serialization errors gracefully.

### 2.2 Error Handling

#### STAB-07: Silent Error Swallowing in Multiple Locations
- **File:** Multiple files
- **Severity:** Medium
- **Description:** Many operations use `.ok()` to silently discard errors. For example:
  - `src/proxy/registrar.rs:568`: `tx.reply_with(...).await.ok()` - silently drops reply failures
  - `src/proxy/server.rs:358`: `tx.reply(...).await.ok()` - drops challenge response failures
  - `src/proxy/registrar.rs:541`: `.unregister(...).await.ok()` - drops unregistration failures

  While some of these are intentional (e.g., best-effort reply), others could mask important failures.
- **Suggested Fix:** At minimum, log discarded errors at debug/trace level. For critical operations like sending authentication challenges, consider propagating the error.

### 2.3 Potential Deadlocks

#### STAB-08: Multiple Lock Acquisitions in Sequence Without Ordering
- **File:** `src/proxy/data.rs`, various methods
- **Severity:** Low
- **Description:** Methods like `reload_trunks` acquire the `config` RwLock for read, then later acquire the `trunks` RwLock for write. While this particular pattern is unlikely to deadlock (since config is always acquired first), there is no formal lock ordering documented. The use of `std::sync::RwLock` in async code is also concerning -- holding the lock across `.await` points would block the executor thread. Current code appears to clone and release locks before awaiting, which is correct.
- **Suggested Fix:** Consider using `tokio::sync::RwLock` for data accessed in async contexts, or document the lock ordering invariant. Alternatively, consider using `parking_lot::RwLock` which has better performance characteristics and no poisoning.

#### STAB-09: `std::sync::Mutex` Used in Async Context
- **File:** `src/media/call_quality.rs`, lines 45-46 (two locks acquired simultaneously); `src/proxy/active_call_registry.rs`; `src/media/mod.rs`
- **Severity:** Low
- **Description:** `std::sync::Mutex` is used in multiple locations within async code. While the locks are held briefly and not across `.await` points (which is correct), using `std::sync::Mutex` in an async runtime can block the executor thread. In `call_quality.rs` lines 45-46, two Mutex locks are acquired in the same scope -- if another code path acquires them in reverse order, a deadlock would occur.
- **Suggested Fix:** Verify that no code path acquires the `last_arrival` and `last_rtp_ts` locks in reverse order. Consider combining these into a single lock to eliminate the risk.

---

## 3. Performance Findings

### PERF-01: Blocking File I/O in Async Context
- **File:** `src/proxy/data.rs`, lines 169, 302-303, 603, 633, 663, etc.; `src/console/mod.rs`, line 170; `src/proxy/user_plain.rs`, line 26
- **Severity:** Medium
- **Description:** Multiple locations use `std::fs::read_to_string()`, `std::fs::write()`, `std::fs::read_dir()`, and `std::fs::File::open()` directly in async functions. These are blocking operations that will block the Tokio runtime thread, potentially causing latency spikes for all concurrent async tasks. Key locations:
  - `ProxyDataContext` file loading (trunks, routes, ACL files)
  - Template loader in `ConsoleState::render()`
  - `PlainTextBackend::load()` (reads user file with blocking I/O)
- **Suggested Fix:** Use `tokio::fs` equivalents or `tokio::task::spawn_blocking()` for file I/O operations. For the template loader, consider caching templates in memory.

### PERF-02: Template Re-loading on Every Request
- **File:** `src/console/mod.rs`, lines 122-187
- **Severity:** Medium
- **Description:** The `render()` method creates a new `minijinja::Environment` and sets up a file-system loader for every single template render call. This means:
  - The template is read from disk on every request
  - The template is parsed/compiled on every request
  - Custom filters are re-registered on every request

  For a production system, this creates unnecessary I/O and CPU overhead.
- **Suggested Fix:** Create the `Environment` once at startup (or lazily with caching) and store it in `ConsoleState`. MiniJinja supports template auto-reloading in development mode. At minimum, cache compiled templates.

### PERF-03: RwLock Snapshot Clones in Hot Paths
- **File:** `src/proxy/data.rs`, lines 86-88 (`trunks_snapshot`), 94-96 (`routes_snapshot`), 98-100 (`acl_rules_snapshot`)
- **Severity:** Medium
- **Description:** Every call to `trunks_snapshot()`, `routes_snapshot()`, or `acl_rules_snapshot()` clones the entire HashMap/Vec. For the trunks map especially, this involves cloning potentially many `TrunkConfig` structs with their string fields. These snapshots are called on every incoming SIP transaction (via `DefaultRouteInvite::build_context`).
- **Suggested Fix:** Use `Arc<HashMap<...>>` behind the RwLock so that snapshots only require an Arc clone (incrementing a reference count) rather than a deep copy. Update the data by swapping the Arc.

### PERF-04: Config Clone via Serialization Round-Trip
- **File:** `src/config.rs`, lines 804-811
- **Severity:** Low
- **Description:** `Config::clone()` serializes the entire config to TOML string and parses it back. This is much slower than a field-by-field clone. While the comment notes it is not called in hot paths, it is still an anti-pattern.
- **Suggested Fix:** Derive `Clone` on `Config` and all nested types, or implement `Clone` manually.

### PERF-05: Redundant Duplicate Method Removal with O(n^2) Algorithm
- **File:** `src/proxy/server.rs`, lines 585-596
- **Severity:** Low
- **Description:** The deduplication of `allow_methods` uses a nested loop (O(n^2) algorithm). While the list is small (typically <10 methods), this could be replaced with a more idiomatic approach.
- **Suggested Fix:** Use a `HashSet` or `BTreeSet` to collect unique methods, or use `sort` + `dedup`.

### PERF-06: Unnecessary String Allocations in SIP Processing
- **File:** `src/proxy/registrar.rs`, lines 199-216 (collect_header_values)
- **Severity:** Low
- **Description:** `collect_header_values` converts every header to a string representation, then parses it to extract the header name. This creates unnecessary string allocations for headers that don't match. The function is called for every REGISTER request.
- **Suggested Fix:** Use the header's type-based matching rather than string conversion for comparison.

---

## 4. Scalability Concerns

### SCALE-01: Single-Process Architecture
- **Severity:** Informational
- **Description:** The system runs as a single process. All SIP, RTP, HTTP, and WebSocket handling occurs in one process. This limits horizontal scalability and means a panic in any component can affect all services.
- **Suggested Fix:** This is an architectural note, not a bug. For high-availability deployments, consider documenting recommended deployment patterns (e.g., process supervision, load balancing).

### SCALE-02: In-Memory Locator Default
- **File:** `src/config.rs`, lines 753-756
- **Severity:** Informational
- **Description:** The default locator is `Memory`, which stores all SIP registrations in memory. This limits deployment to single-instance configurations. A restart loses all registrations.
- **Suggested Fix:** Document this limitation. The database and HTTP locator options exist for multi-instance deployments.

---

## 5. Summary Table

| ID | Category | Severity | Description |
|---|---|---|---|
| SEC-WEB-01 | Security | **High** | Missing CSRF protection on all form endpoints |
| SEC-WEB-02 | Security | **High** | Session cookie missing SameSite and Secure attributes |
| SEC-WEB-03 | Security | **High** | IP address spoofing via X-Forwarded-For headers |
| STAB-01 | Stability | **High** | Config clone panics on serialization failure |
| STAB-02 | Stability | **High** | unwrap() on RwLock in get_sip_server |
| SEC-SIP-01 | Security | Medium | MD5-only digest authentication |
| SEC-SIP-02 | Security | Medium | No nonce replay protection |
| SEC-WEB-04 | Security | Medium | Trusts X-Forwarded-Proto unconditionally |
| SEC-WEB-05 | Security | Medium | Reset token logged in plaintext |
| SEC-WEB-06 | Security | Medium | Reset link returned in HTTP response |
| SEC-WEB-08 | Security | Medium | SIP passwords stored in plaintext |
| SEC-CFG-01 | Security | Medium | S3 credentials in config file |
| SEC-CFG-02 | Security | Medium | Trunk auth credentials in plaintext |
| STAB-03 | Stability | Medium | unwrap() on RwLock in ProxyDataContext |
| STAB-04 | Stability | Medium | unwrap() on Mutex in ACME handlers |
| STAB-05 | Stability | Medium | unwrap() on Mutex in registrar |
| STAB-07 | Stability | Medium | Silent error swallowing |
| PERF-01 | Performance | Medium | Blocking file I/O in async context |
| PERF-02 | Performance | Medium | Template re-loading on every request |
| PERF-03 | Performance | Medium | Full HashMap clone on every SIP transaction |
| SEC-SIP-03 | Security | Low | Default ACL allows all traffic |
| SEC-SIP-04 | Security | Low | UA whitelist exact match only |
| SEC-SIP-05 | Security | Low | Trunk IP bypass skips all ACL |
| SEC-WEB-07 | Security | Low | Open redirect potential in next param |
| SEC-WEB-09 | Security | Low | Shutdown endpoint accessible to superusers |
| SEC-WEB-10 | Security | Low | Internal errors exposed in responses |
| STAB-06 | Stability | Low | unwrap() in transcript handler |
| STAB-08 | Stability | Low | No formal lock ordering |
| STAB-09 | Stability | Low | std::sync::Mutex in async context |
| PERF-04 | Performance | Low | Config clone via serialization |
| PERF-05 | Performance | Low | O(n^2) method deduplication |
| PERF-06 | Performance | Low | Unnecessary allocations in registrar |

---

## 6. Positive Observations

1. **Argon2 for console password hashing** (`src/console/auth.rs`): The web console uses Argon2 with random salt for password hashing, which is the current best practice.

2. **HMAC-SHA256 session tokens** (`src/console/auth.rs`): Session tokens use HMAC-SHA256 with a derived key from the session secret, with proper expiry checking.

3. **Input validation**: Registration forms validate email format, username length, and password length. The `next` redirect parameter has reasonable validation.

4. **SeaORM parameterized queries**: All database queries use SeaORM's query builder with parameterized inputs, preventing SQL injection.

5. **HttpOnly cookie flag**: Session cookies are set with `HttpOnly`, preventing JavaScript access.

6. **Filename sanitization**: Recording filenames are sanitized (`sanitize_filename_component` in `src/proxy/call.rs`), preventing path traversal in generated file names.

7. **AuthRequired middleware**: Console endpoints consistently use the `AuthRequired` extractor, ensuring authentication is checked.

8. **Cancellation token propagation**: The async architecture properly uses `CancellationToken` for graceful shutdown coordination.

9. **Max concurrency limiting**: The SIP server has configurable max concurrency (`max_concurrency`) to prevent resource exhaustion.

10. **Spam protection**: The system has multiple anti-spam mechanisms: IP blacklisting, UA filtering, `ensure_user` mode, and frequency limiting.

---

## 7. Code Quality and Maintainability

### 7.1 Code Duplication

#### DUP-01: RtpTrackBuilder Configuration Repeated 5 Times
- **File:** `src/proxy/proxy_call/session.rs`, lines 573-618, 755-786, 898-927, 1092-1123, 1197-1205
- **Severity:** High
- **Description:** The RtpTrackBuilder initialization and configuration pattern is repeated five times in `session.rs` across `optimize_caller_codec()`, `negotiate_final_codec()`, `create_caller_answer_from_offer()`, `create_callee_track()`, and `setup_callee_track()`. Each repetition follows the same pattern:
  1. Create `RtpTrackBuilder::new(track_id)` with cancel token, codec info, and latching config
  2. Optionally set external IP
  3. Determine WebRTC vs RTP port range based on SDP type
  4. Set port range
  5. Set WebRTC mode if applicable

  This is ~30 lines of boilerplate per repetition (~150 lines total).
- **Suggested Fix:** Extract a helper method like `fn build_track(&self, track_id: &str, codec_info: Vec<CodecInfo>, is_webrtc: bool) -> RtpTrackBuilder` that encapsulates the common configuration.

#### DUP-02: Codec Extraction Pattern Duplicated in `accept_call()` and `start_ringing_internal()`
- **File:** `src/proxy/proxy_call/session.rs`, lines 1317-1378 and 1744-1812
- **Severity:** High
- **Description:** The codec_a determination logic (extract from caller's answer for WebRTC, from offer for SIP, with fallback chain) is duplicated almost verbatim in two places: inside `start_ringing_internal()` (early media path, ~60 lines) and inside `accept_call()` (answer path, ~70 lines). Both follow the same pattern: check if WebRTC caller, extract from answer, fall back to offer, fall back to default PCMU. The MediaBridge construction call is also duplicated between these two methods with identical 14 parameters.
- **Suggested Fix:** Extract a method like `fn determine_bridge_codecs(&self, callee_sdp: &str) -> (RtpCodecParameters, Option<u8>, CodecType, RtpCodecParameters, Option<u8>, CodecType, Option<u32>, Option<u32>)` and a `fn create_media_bridge_from_sdp(&mut self, callee_sdp: &str)` method.

#### DUP-03: Duplicated `with_ice_servers` Calls (Copy-Paste Bug)
- **File:** `src/proxy/proxy_call/session.rs`, lines 580-594
- **Severity:** Medium
- **Description:** In `optimize_caller_codec()`, the line `track_builder = track_builder.with_ice_servers(servers.clone())` is repeated **six consecutive times** within identical `if let Some(ref servers) = self.context.media_config.ice_servers` blocks. This is clearly a copy-paste error -- the ICE servers are being set six times on the same builder, with no effect beyond the first call.
- **Suggested Fix:** Remove the five duplicate blocks, keeping only the first one.

#### DUP-04: File Loading Functions Follow Identical Pattern
- **File:** `src/proxy/data.rs`, lines 591-680
- **Severity:** Medium
- **Description:** Three functions -- `load_trunks_from_files()`, `load_routes_from_files()`, and `load_acl_rules_from_files()` -- follow an identical pattern: iterate glob patterns, read files, parse TOML, collect results and file paths. The only differences are the deserialization target type and logging messages. Each function is ~30 lines.
- **Suggested Fix:** Create a generic `load_from_files<T, R>(patterns: &[String], extractor: fn(T) -> Vec<R>, label: &str) -> Result<(Vec<R>, Vec<String>)>` function parameterized by the include file type.

#### DUP-05: Reload Methods Follow Identical Pattern
- **File:** `src/proxy/data.rs`, `reload_trunks()` (lines 203-273), `reload_routes()` (lines 343-418), `reload_queues()` (lines 275-341), `reload_acl_rules()` (lines 420-493)
- **Severity:** Medium
- **Description:** All four reload methods follow the same structure: optionally update config, record start time, load from embedded config, load from files, load from generated files, update RwLock, record end time, log metrics, return ReloadMetrics. This is ~70-80 lines per method with most of the structure being identical.
- **Suggested Fix:** Consider a generic reload framework or at least extracting the common timing/metrics/locking logic into a helper.

### 7.2 Function Complexity

#### CMPLX-01: `session.rs` is 4,748 Lines
- **File:** `src/proxy/proxy_call/session.rs`
- **Severity:** High
- **Description:** This single file contains the entire `CallSession` struct and its implementation, totaling 4,748 lines including ~750 lines of tests. The `CallSession` struct itself has 38 fields. Key oversized functions include:
  - `start_ringing_internal()`: ~230 lines (lines 1257-1529) -- mixes early media setup, codec negotiation, media bridge creation, ringback logic, and 183 response sending
  - `accept_call()`: ~260 lines (lines 1654-1917) -- mixes codec optimization, media bridge setup, callee track configuration, and 200 OK sending
  - `dial_parallel()`: ~350 lines (lines 2602-2949) -- manages parallel INVITE state machine
  - `run_server_events_loop()`: ~300 lines (lines 3350-3668) -- handles all server dialog events and session timer refresh
  - `process()`: ~125 lines (lines 3849-3973) -- main session orchestration with 5-way tokio::select
  - `serve()`: ~200 lines (lines 3670-3847) -- session setup and server dialog handling

  The file handles media negotiation, call signaling, call forwarding, queue management, session timers, parallel/sequential dialing, re-INVITE handling, and call recording setup all in one place.
- **Suggested Fix:** Split into focused modules:
  - `session_media.rs` -- track creation, codec negotiation, media bridge setup
  - `session_dial.rs` -- sequential/parallel dialing, target execution
  - `session_events.rs` -- server event loop, session timer management
  - `session_queue.rs` -- queue plan execution, hold music management
  - `session.rs` -- core struct, `serve()`, `process()`, and cross-cutting coordination

#### CMPLX-02: `run_server_events_loop()` Static Method with 12 Parameters
- **File:** `src/proxy/proxy_call/session.rs`, lines 3350-3668
- **Severity:** Medium
- **Description:** `run_server_events_loop` is a static method that takes 12 parameters: `context`, `proxy_config`, `dialog_layer`, `state_rx`, `callee_state_rx`, `server_timer`, `client_timer`, `callee_dialogs`, `server_dialog`, `handle`, `cancel_token`, `pending_hangup`, and `_shared`. This many parameters indicate the function is doing too much and/or needs a parameter object.
- **Suggested Fix:** Group related parameters into a `ServerEventLoopContext` struct.

#### CMPLX-03: `CallSession` struct has 38 Fields
- **File:** `src/proxy/proxy_call/session.rs`, lines 164-202
- **Severity:** Medium
- **Description:** The `CallSession` struct has 38 fields, mixing concerns: SIP signaling state (server_dialog, callee_dialogs, connected_dialog_id), media state (caller_peer, callee_peer, media_bridge, caller_offer, callee_offer, answer), call lifecycle state (ring_time, answer_time, hangup_reason), configuration (use_media_proxy, recorder_option), routing state (routed_caller, routed_callee, routed_contact, routed_destination), and timer state (server_timer, client_timer).
- **Suggested Fix:** Group related fields into sub-structs: `MediaState`, `RoutingState`, `TimerState`, `SignalingState`.

### 7.3 Module Organization

#### MOD-01: `pub mod tests` Compiled in Non-Test Builds
- **File:** `src/proxy/mod.rs`, line 25
- **Severity:** Medium
- **Description:** The test module `pub mod tests` is declared without `#[cfg(test)]`, meaning all test code in `src/proxy/tests/` is compiled into the release binary. The `src/proxy/tests/` directory contains 15 test files totaling ~6,000 lines. This increases binary size and compile time unnecessarily.
- **Suggested Fix:** Add `#[cfg(test)]` attribute: `#[cfg(test)] pub mod tests;`

#### MOD-02: Flat Module Structure in `src/proxy/`
- **File:** `src/proxy/mod.rs`
- **Severity:** Low
- **Description:** The proxy module declares 20 submodules (`acl`, `active_call_registry`, `auth`, `call`, `data`, `locator`, `locator_db`, `locator_webhook`, `nat`, `presence`, `proxy_call`, `registrar`, `routing`, `server`, `trunk_register`, `tests`, `user`, `user_db`, `user_extension`, `user_http`, `user_plain`, `ws`). The user-related modules (`user`, `user_db`, `user_extension`, `user_http`, `user_plain`) could be grouped under a `user/` subdirectory. Similarly, the locator modules could be grouped.
- **Suggested Fix:** Consider reorganizing: `proxy/user/mod.rs`, `proxy/user/db.rs`, `proxy/user/http.rs`, `proxy/user/plain.rs`, `proxy/user/extension.rs`, and similarly `proxy/locator/mod.rs`, `proxy/locator/db.rs`, `proxy/locator/webhook.rs`.

#### MOD-03: Console Handlers are Very Large
- **File:** `src/console/handlers/` directory
- **Severity:** Low
- **Description:** Several console handler files are quite large: `diagnostics.rs` (2,459 lines), `setting.rs` (2,114 lines), `call_record.rs` (1,662 lines), `routing.rs` (1,488 lines). These files mix HTTP handler functions with data transformation, database query logic, and business rules. Each is effectively a mini-application.
- **Suggested Fix:** Consider separating database query/transformation logic into service modules, leaving handlers as thin HTTP endpoint wrappers.

### 7.4 Error Handling Strategy

#### ERR-01: No Domain-Specific Error Types
- **Severity:** Medium
- **Description:** The project uses `anyhow::Result` and `anyhow::Error` almost exclusively (60+ files). The only custom error type is `AuthError` in `src/proxy/auth.rs`. This means:
  - Callers cannot match on specific error types to handle them differently
  - Error chains are string-based rather than typed
  - SIP status codes are sometimes returned as `Result<(), (StatusCode, Option<String>)>` (a tuple, not a proper error type) -- see `try_single_target()` at line 2951
  - Some functions return `Result<..., (anyhow::Error, Option<rsip::StatusCode>)>` -- another ad-hoc error tuple

  The inconsistency makes error handling unpredictable: some call sites use `.ok()` to discard errors, others use `?` to propagate, and others match on tuple variants.
- **Suggested Fix:** Define domain-specific error types for major subsystems: `ProxyError` (with SIP status code), `MediaError`, `ConfigError`. Use `thiserror` for structured error types that implement `std::error::Error`.

#### ERR-02: Mixed Error Return Conventions
- **Severity:** Medium
- **Description:** Functions within the same module use different error return conventions:
  - `try_single_target()` returns `Result<(), (StatusCode, Option<String>)>`
  - `accept_call()` returns `Result<()>` (anyhow)
  - `handle_reinvite()` returns nothing (void) and swallows errors internally
  - `start_ringing()` returns nothing (void) and logs errors
  - `report_failure()` returns `Result<()>` (anyhow)
  - `init_server_timer()` returns `Result<(), (StatusCode, Option<String>)>` (yet another convention)

  This means callers must know the specific error convention of each method.
- **Suggested Fix:** Standardize on a single error type per module, with conversion traits.

### 7.5 Documentation

#### DOC-01: Minimal Public API Documentation
- **Severity:** Medium
- **Description:** In the 4,748-line `session.rs`, only 7 functions have `///` doc comments out of ~40 public/internal methods. The documented functions are all test helpers. Key public methods like `serve()`, `process()`, `accept_call()`, `start_ringing()`, `handle_reinvite()`, `execute_dialplan()`, `dial_sequential()`, `dial_parallel()` have no documentation explaining their contracts, preconditions, or state transitions.

  Similarly, `src/proxy/data.rs` (1,240 lines) has zero doc comments. `src/proxy/server.rs` (918 lines) has zero doc comments. The `ProxyModule` trait in `src/proxy/mod.rs` has no trait-level or method-level documentation.

- **Suggested Fix:** At minimum, document:
  - All public traits (especially `ProxyModule`, `Track`, `MediaPeer`, `RouteInvite`, `CallRouter`, `Locator`, `UserBackend`)
  - State machine transitions in `CallSession`
  - The call flow lifecycle from INVITE to BYE
  - Any function with complex preconditions or side effects

#### DOC-02: No Architecture Documentation in Code
- **Severity:** Low
- **Description:** There are no module-level `//!` documentation comments anywhere in the codebase. The relationship between modules (e.g., how `proxy_call`, `call`, `routing`, and `server` interact) is not documented in code. A new developer would need to trace through `serve()` -> `process()` -> `execute_dialplan()` -> `run_targets()` -> `dial_sequential()` / `dial_parallel()` -> `try_single_target()` -> `execute_invite()` to understand the call flow.
- **Suggested Fix:** Add `//!` module documentation to at least `src/proxy/mod.rs`, `src/proxy/proxy_call/session.rs`, `src/media/mod.rs`, and `src/console/mod.rs`.

### 7.6 Test Coverage

#### TEST-01: No Tests for Console Handlers
- **Severity:** Medium
- **Description:** The `src/console/handlers/` directory contains 12,031 lines of handler code across 15 files. Only `extension.rs`, `call_record.rs`, and `diagnostics.rs` have inline `mod tests` blocks. The handlers for `user.rs` (login/register/password reset), `sip_trunk.rs`, `setting.rs`, `sipflow.rs`, `dashboard.rs`, `addons.rs`, `call_control.rs`, `presence.rs`, and `utils.rs` have zero tests. These handlers contain significant business logic including form validation, data transformation, and authorization checks.
- **Suggested Fix:** Add integration tests for at least the authentication handlers (login, register, password reset) and the CRUD operations (sip_trunk, routing).

#### TEST-02: No Tests for `src/proxy/call.rs` (Call Module)
- **Severity:** Medium
- **Description:** `src/proxy/call.rs` (1,061 lines) implements `DefaultRouteInvite`, `CallModule` (the ProxyModule for call handling), and `DefaultCallRouter`. It contains the core routing logic that translates incoming SIP INVITEs into Dialplans. The `build_dialplan()` method (~200 lines) handles trunk selection, codec configuration, recording policy, and call forwarding setup. There are no unit tests for this module.
- **Suggested Fix:** Add tests for `build_dialplan()`, `DefaultCallRouter::resolve()`, and the recording policy resolution logic.

#### TEST-03: No Tests for `src/proxy/nat.rs` or `src/proxy/trunk_register.rs`
- **Severity:** Low
- **Description:** `nat.rs` (the NatInspector module critical to fixing the ACK routing bug documented in MEMORY.md) and `trunk_register.rs` (trunk registration for inbound PSTN calls) have no dedicated tests.
- **Suggested Fix:** Add unit tests, especially for the NatInspector's Contact header rewriting logic and the Record-Route skip condition.

#### TEST-04: Test-to-Source Ratio
- **Severity:** Informational
- **Description:** The codebase has ~50,761 lines of non-test source code and ~10,949 lines of test code, a ratio of approximately 1:4.6. While the proxy module has reasonable test coverage (12 test files), the console, media, and config modules are undertested.

### 7.7 Dead Code and Stale Code

#### DEAD-01: `println!` Debug Statement in Production Code
- **File:** `src/proxy/proxy_call/session.rs`, line 1598
- **Severity:** Medium
- **Description:** A `println!("DEBUG: Received re-INVITE but no answer SDP available")` statement exists in production code path `handle_reinvite()`. All other logging uses the `tracing` crate (`info!`, `warn!`, `debug!`). This `println!` will go to stdout rather than the structured logging system and cannot be filtered by log level.
- **Suggested Fix:** Replace with `warn!(session_id = %self.context.session_id, "Received re-INVITE but no answer SDP available")`.

#### DEAD-02: Commented-Out Code in Multiple Locations
- **File:** `src/proxy/proxy_call/session.rs`, lines 426-428, 1661, 2144-2148
- **Severity:** Low
- **Description:** Several blocks of commented-out code remain:
  - `last_queue_name()` returns `None` with a commented-out `self.last_queue_name.clone()` (line 427-428)
  - `accept_call()` has a commented-out `self.stop_queue_hold().await` (line 1661)
  - `apply_session_action` for `StartRinging` has commented-out passthrough ringback logic (lines 2144-2148)

  Commented-out code adds noise and confusion about intended behavior.
- **Suggested Fix:** Remove commented-out code. If the functionality is planned, track it as a TODO or in a task tracker.

#### DEAD-03: `#[allow(dead_code)]` Annotations
- **File:** `src/proxy/proxy_call/session.rs`, lines 53, 111, 2037
- **Severity:** Low
- **Description:** Three items are annotated with `#[allow(dead_code)]`:
  - `FlowFailureHandling::Propagate` variant (line 53) -- unused enum variant
  - `ParallelEvent::Failed._idx` field (line 111) -- unused field
  - `store_pending_hangup()` method (line 2037) -- actually IS used on lines 2127, 3378, 3482, but was marked dead code presumably by mistake (or it was dead at some point and the annotation was never removed)
- **Suggested Fix:** Remove `FlowFailureHandling::Propagate` if not planned. For `_idx` in `ParallelEvent::Failed`, the leading underscore already suppresses the warning -- the `#[allow(dead_code)]` is redundant. Remove the incorrect `#[allow(dead_code)]` from `store_pending_hangup()`.

#### DEAD-04: `println!` Statements in Non-Test Production Code
- **File:** `src/fixtures.rs` (lines 12, 27, 58, 76)
- **Severity:** Low
- **Description:** The fixtures module uses `println!()` for output instead of the `tracing` logging framework used everywhere else. While fixtures are typically run at startup, they should still use structured logging for consistency and log-level filtering.
- **Suggested Fix:** Replace `println!()` with `info!()` from tracing.

### 7.8 Naming Conventions

#### NAME-01: Inconsistent Spelling of "canceled"/"cancelled"
- **File:** `src/proxy/proxy_call/session.rs`, line 3823
- **Severity:** Low
- **Description:** The variable `canceld` (line 3823) is misspelled -- it should be `cancelled`. The rest of the codebase consistently uses the British spelling "cancelled" (matching Tokio's `CancellationToken::cancelled()`).
- **Suggested Fix:** Rename to `cancelled` for consistency.

#### NAME-02: Inconsistent Method Visibility
- **File:** `src/proxy/proxy_call/session.rs`
- **Severity:** Low
- **Description:** Methods within `CallSession` have inconsistent visibility without clear reasoning:
  - `pub fn new()`, `pub fn add_callee_guard()`, `pub fn note_attempt_failure()` are public
  - `fn check_media_proxy()`, `fn is_webrtc_sdp()`, `fn find_track()` are private
  - `pub async fn create_callee_track()` is public but only called internally
  - `pub fn apply_session_action()` is public but is `pub(crate)` on the module

  Since `CallSession` is `pub(crate)` (the `session` module is `pub(crate)`), the `pub` visibility on individual methods has no effect beyond the crate. This creates a false impression of an API boundary.
- **Suggested Fix:** Since the module is `pub(crate)`, either make all methods private (default) or document which methods constitute the session's internal API.

#### NAME-03: Mixed Naming for SDP-Related Variables
- **File:** `src/proxy/proxy_call/session.rs`
- **Severity:** Low
- **Description:** SDP-related variables use inconsistent naming:
  - `caller_offer` / `callee_offer` / `answer` (Option<String> fields on CallSession)
  - `callee_answer_sdp` (field)
  - `offer_sdp` / `orig_offer_sdp` (local variables)
  - `callee_early_sdp` (local variable in early media path)
  - `answer_for_caller` / `processed_answer` / `optimized_answer` (return values)

  The distinction between "offer", "answer", "sdp", and the many variations makes it hard to track which SDP belongs to which party at which stage.
- **Suggested Fix:** Adopt consistent naming: `caller_offer_sdp`, `callee_offer_sdp`, `caller_answer_sdp`, `callee_answer_sdp`.

### 7.9 Additional Quality Findings

#### QUAL-01: Redundant `#[cfg(target_os = "windows")]` Branches
- **File:** `src/config.rs`, lines 22-25 and 817-821
- **Severity:** Low
- **Description:** Two functions have platform-conditional returns that return the same value for both platforms:
  ```rust
  pub(crate) fn default_config_recorder_path() -> String {
      #[cfg(target_os = "windows")]
      return "./config/recorders".to_string();
      #[cfg(not(target_os = "windows"))]
      return "./config/recorders".to_string();
  }
  ```
  And similarly for `CallRecordConfig::default()`. Both branches return identical strings.
- **Suggested Fix:** Remove the conditional compilation and return the string directly.

#### QUAL-02: `CallSessionBuilder::report_failure()` Constructs Dummy Objects
- **File:** `src/proxy/proxy_call.rs`, lines 113-193
- **Severity:** Low
- **Description:** `report_failure()` constructs a `CallContext` with full `MediaConfig` and a `CallSessionRecordSnapshot` with an empty `DialogId` (`call_id: "".into()`, `local_tag: "".into()`, `remote_tag: "".into()`). The `MediaConfig` is fully populated (copied from `build_and_serve()`) but never used since no media is involved. This is ~80 lines of boilerplate for early failure reporting. The `CallContext` construction (lines 139-157) is duplicated from `build_and_serve()` (lines 90-108).
- **Suggested Fix:** Extract a `fn build_context(&self, server: &SipServerRef) -> CallContext` method to share the construction, and use a dedicated `EarlyFailureReport` struct instead of repurposing `CallSessionRecordSnapshot`.

#### QUAL-03: Large Console Handler Files Mix Concerns
- **Severity:** Medium
- **Description:** Console handler files contain database queries, business logic, data transformation, and HTTP response construction all in single files. For example:
  - `diagnostics.rs` (2,459 lines) performs network diagnostics, system health checks, and configuration validation
  - `setting.rs` (2,114 lines) handles all application settings with complex form processing
  - `routing.rs` (1,488 lines) handles route CRUD with complex route-to-model conversion

  Each handler directly accesses SeaORM entities, performs business logic, and renders templates. There is no service layer separation.
- **Suggested Fix:** Consider introducing a service layer (`src/services/`) that handles business logic and database operations, keeping handlers as thin HTTP adapters.

### 7.10 Code Quality Summary Table

| ID | Category | Severity | Description |
|---|---|---|---|
| DUP-01 | Duplication | **High** | RtpTrackBuilder config repeated 5 times (~150 lines) |
| DUP-02 | Duplication | **High** | Codec extraction + MediaBridge creation duplicated in accept/ringing |
| CMPLX-01 | Complexity | **High** | session.rs is 4,748 lines with 38-field struct |
| DUP-03 | Duplication | Medium | `with_ice_servers` copy-pasted 6 times (bug) |
| DUP-04 | Duplication | Medium | File loading functions follow identical pattern |
| DUP-05 | Duplication | Medium | Reload methods follow identical pattern |
| CMPLX-02 | Complexity | Medium | `run_server_events_loop()` has 12 parameters |
| CMPLX-03 | Complexity | Medium | `CallSession` has 38 fields mixing concerns |
| MOD-01 | Organization | Medium | Test module compiled in release builds |
| ERR-01 | Error handling | Medium | No domain-specific error types (anyhow everywhere) |
| ERR-02 | Error handling | Medium | Mixed error return conventions in same module |
| DOC-01 | Documentation | Medium | Minimal public API documentation |
| TEST-01 | Test coverage | Medium | No tests for console handlers (12,031 lines) |
| TEST-02 | Test coverage | Medium | No tests for call routing module (1,061 lines) |
| DEAD-01 | Dead code | Medium | `println!` debug statement in production |
| QUAL-03 | Organization | Medium | Console handlers mix concerns (no service layer) |
| MOD-02 | Organization | Low | Flat module structure with 20 submodules |
| MOD-03 | Organization | Low | Large console handler files |
| DOC-02 | Documentation | Low | No module-level documentation |
| TEST-03 | Test coverage | Low | No tests for nat.rs or trunk_register.rs |
| DEAD-02 | Dead code | Low | Commented-out code in multiple locations |
| DEAD-03 | Dead code | Low | Incorrect/redundant `#[allow(dead_code)]` |
| DEAD-04 | Dead code | Low | `println!` in fixtures instead of tracing |
| NAME-01 | Naming | Low | Misspelled variable `canceld` |
| NAME-02 | Naming | Low | Inconsistent method visibility |
| NAME-03 | Naming | Low | Inconsistent SDP variable naming |
| QUAL-01 | Quality | Low | Redundant platform-conditional branches |
| QUAL-02 | Quality | Low | Dummy object construction for failure reports |
| TEST-04 | Test coverage | Informational | Test:source ratio is 1:4.6 |

---

*End of report*
