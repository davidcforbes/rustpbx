# L2 API Contract Tests

**Level:** L2 -- HTTP API Contract Testing
**Scope:** Console authentication, CRUD operations for extensions/routes/trunks,
AMI API endpoints, WebSocket events, recording downloads, and error handling.
**Created:** 2026-02-21
**RustPBX Version:** 0.3.18
**Base URL:** `http://localhost:8080`

---

## Summary Table

| ID | Title | Priority | Tool | Automated |
|----|-------|----------|------|-----------|
| TC-L2-001 | Console authentication (login/logout/session) | P0 | pytest + requests | Yes |
| TC-L2-002 | Extension CRUD | P0 | pytest + requests | Yes |
| TC-L2-003 | Route CRUD and pattern matching | P0 | pytest + requests | Yes |
| TC-L2-004 | Trunk CRUD and health status | P1 | pytest + requests | Yes |
| TC-L2-005 | Call records listing and filtering | P1 | pytest + requests | Yes |
| TC-L2-006 | AMI API authentication (IP restriction) | P1 | pytest + requests | Partial |
| TC-L2-007 | AMI reload endpoints | P0 | pytest + requests + curl | Yes |
| TC-L2-008 | WebSocket event subscription | P1 | pytest + websockets | Yes |
| TC-L2-009 | Recording download endpoint | P2 | pytest + requests | Partial |
| TC-L2-010 | API error handling (400, 401, 404, 500) | P1 | pytest + requests | Yes |

---

## Prerequisites (All Tests)

- RustPBX Docker container running and healthy (`docker ps --filter name=rustpbx` shows `Up`).
- HTTP port 8080 accessible from the test runner host.
- Admin super-user created: `admin` / `admin123`.
- Memory-backend test users configured: `1001`/`test1001`, `1002`/`test1002`.
- SQLite database initialized (migrations applied).
- AMI allows wildcard (`allows = ["*"]`) unless TC-L2-006 overrides it.
- Console base path: `/console`.

---

## Test Cases

### TC-L2-001: Console Authentication (Login/Logout/Session)

**Priority:** P0
**Automated:** Yes
**Tool:** pytest + requests

**Preconditions:**
- RustPBX container running with console enabled at `/console`.
- Admin super-user exists: `admin` / `admin123`.
- No active session (clean cookie jar).

**Steps:**
1. Send `GET /console/login` and verify the login page loads (HTTP 200, body contains a login form with `identifier` and `password` fields).
2. Send `POST /console/login` with form body `identifier=admin&password=admin123`. Do NOT follow redirects automatically.
3. Capture the response: verify HTTP 303 redirect with `Location` header pointing to `/console/` (or `/console`).
4. Extract the `Set-Cookie` header from the response. Verify a session cookie is present (name and value are non-empty).
5. Send `GET /console/` with the session cookie attached. Verify HTTP 200 and body contains dashboard content (e.g., the string "Dashboard" or "Extensions" in the HTML).
6. Send `GET /console/extensions` with the session cookie. Verify HTTP 200.
7. Send `GET /console/routing` with the session cookie. Verify HTTP 200.
8. Send `GET /console/settings` with the session cookie. Verify HTTP 200.
9. Send `GET /console/call-records` with the session cookie. Verify HTTP 200.
10. Send `GET /console/diagnostics` with the session cookie. Verify HTTP 200.
11. Send `POST /console/login` with invalid credentials `identifier=admin&password=wrong`. Verify the response is NOT a 303 redirect -- it should return HTTP 200 with an error message in the body, or HTTP 401.
12. Send `GET /console/` WITHOUT a session cookie. Verify the response redirects to `/console/login` (HTTP 302 or 303) or returns HTTP 401.

**Expected Result:**
- Valid login produces a 303 redirect with a session cookie.
- The session cookie grants access to all protected console pages.
- Invalid credentials do not produce a session cookie or redirect to the dashboard.
- Unauthenticated requests to protected pages are redirected to login.

**Pass Criteria:**
- Steps 1-10 return the expected HTTP status codes.
- Step 11 does NOT issue a session cookie.
- Step 12 does NOT return HTTP 200 with dashboard content.
- All assertions pass in a single pytest run.

**Fail Action:**
- **Severity: Critical.** Authentication is the foundation of console security. If login bypass is possible, file a P0 security defect. If session management is broken (cookie not set, or cookie not validated), file a P0 functional defect. Block all other L2 tests that depend on authenticated sessions.

---

### TC-L2-002: Extension CRUD (Create, Read, Update, Delete)

**Priority:** P0
**Automated:** Yes
**Tool:** pytest + requests

**Preconditions:**
- Authenticated console session (TC-L2-001 passed).
- Session cookie available for requests.
- Extension `9901` does NOT already exist (clean state).

**Steps:**
1. **CREATE** -- Send a POST request to the extension creation endpoint (e.g., `POST /console/extensions` or the API equivalent) with payload:
   - `username`: `9901`
   - `password`: `test9901`
   - `display_name`: `Test Extension 9901`
   Verify HTTP 200 or 201 response, or a 303 redirect indicating success.
2. **READ (list)** -- Send `GET /console/extensions` with the session cookie. Parse the response body and verify that extension `9901` appears in the extension list.
3. **READ (detail)** -- If the API supports a detail endpoint (e.g., `GET /console/extensions/9901`), send the request and verify the response contains `username=9901` and `display_name=Test Extension 9901`.
4. **UPDATE** -- Send a POST/PUT request to update extension `9901`:
   - `display_name`: `Updated Extension 9901`
   Verify the response indicates success.
5. **READ (verify update)** -- Re-read the extension and confirm `display_name` is now `Updated Extension 9901`.
6. **DELETE** -- Send a DELETE or POST request to remove extension `9901`. Verify the response indicates success.
7. **READ (verify delete)** -- Re-read the extension list and confirm `9901` is no longer present.
8. **SIP registration test** -- Before deleting, optionally verify the created extension can register via SIP (send a SIP REGISTER for `9901`/`test9901` and confirm 200 OK). This validates the extension backend integration.

**Expected Result:**
- Extension `9901` is created, readable, updatable, and deletable through the console/API.
- After creation, the extension appears in the extension list.
- After update, the modified fields reflect the new values.
- After deletion, the extension no longer appears.

**Pass Criteria:**
- All CRUD operations return success status codes.
- Read operations confirm the expected state after each mutation.
- Extension list count increases by 1 after create, decreases by 1 after delete.

**Fail Action:**
- **Severity: High.** Extension CRUD is core admin functionality. If creation fails, agents cannot be provisioned. If deletion fails, orphaned extensions may allow unauthorized access. File a P0 defect for create/delete failures, P1 for update or read failures.

---

### TC-L2-003: Route CRUD and Pattern Matching

**Priority:** P0
**Automated:** Yes
**Tool:** pytest + requests

**Preconditions:**
- Authenticated console session (TC-L2-001 passed).
- Route named `test-route-l2` does NOT already exist.

**Steps:**
1. **CREATE** -- Send a POST request to create a new route with:
   - `name`: `test-route-l2`
   - `priority`: `200`
   - `direction`: `any`
   - `match` pattern: `"to.user" = "^(990[0-9])$"`
   - `action` type: `local`
   - `rewrite` rule: `"to.host" = "127.0.0.1"`
   Verify the response indicates success.
2. **READ (list)** -- Send `GET /console/routing` with the session cookie. Verify `test-route-l2` appears in the route list.
3. **READ (detail)** -- If a detail endpoint exists, verify the route fields match what was submitted (name, priority, direction, match pattern, action, rewrite).
4. **UPDATE (priority)** -- Send a request to update the route's priority from `200` to `150`. Verify the response indicates success.
5. **READ (verify update)** -- Re-read the route and confirm the priority is now `150`.
6. **UPDATE (pattern)** -- Change the match pattern to `"to.user" = "^(880[0-9])$"`. Verify success.
7. **RELOAD** -- Send `POST /ami/v1/reload/routes` to hot-reload routes. Verify the reload response indicates success.
8. **DELETE** -- Remove the route `test-route-l2`. Verify success.
9. **READ (verify delete)** -- Confirm the route no longer appears in the route list.
10. **RELOAD (post-delete)** -- Send `POST /ami/v1/reload/routes` again. Verify the deleted route is no longer active.

**Expected Result:**
- Route CRUD operations complete successfully.
- Priority and pattern updates persist after reload.
- Deleted routes are removed from both the console listing and the active routing engine.

**Pass Criteria:**
- All CRUD operations succeed.
- Post-reload, the route engine reflects the current state (no stale routes).
- Route list count changes appropriately with create/delete.

**Fail Action:**
- **Severity: High.** Broken route management means calls cannot be routed correctly. If route creation fails, new call paths cannot be configured. If deletion fails, unwanted routes may misroute calls. File P0 for create/delete failures. File P1 for update or reload failures.

---

### TC-L2-004: Trunk CRUD and Health Status

**Priority:** P1
**Automated:** Yes
**Tool:** pytest + requests

**Preconditions:**
- Authenticated console session (TC-L2-001 passed).
- The existing `telnyx` trunk is present in configuration.
- Trunk named `test-trunk-l2` does NOT already exist.

**Steps:**
1. **CREATE** -- Create a new trunk configuration via the console or API:
   - `name`: `test-trunk-l2`
   - `dest`: `sip:test-sip.example.com:5060`
   - `transport`: `udp`
   - `username`: `testuser`
   - `password`: `testpass`
   - `direction`: `outbound`
   - `codec`: `["PCMU", "PCMA"]`
   - `max_calls`: `10`
   - `max_cps`: `2`
   Verify success.
2. **READ (list)** -- Verify `test-trunk-l2` appears in the trunk list (via console settings or API).
3. **READ (detail)** -- Verify trunk fields match submitted values. Confirm the password is NOT returned in plaintext (or is masked).
4. **UPDATE** -- Change `max_calls` from `10` to `20` and `direction` from `outbound` to `bidirectional`. Verify success.
5. **READ (verify update)** -- Confirm the updated fields reflect new values.
6. **RELOAD** -- Send `POST /ami/v1/reload/trunks`. Verify the reload response.
7. **HEALTH CHECK** -- Send `GET /ami/v1/health`. Verify the health response includes trunk information or at minimum does not report errors related to the test trunk.
8. **DELETE** -- Remove trunk `test-trunk-l2`. Verify success.
9. **RELOAD (post-delete)** -- Send `POST /ami/v1/reload/trunks`. Verify the deleted trunk is no longer loaded.
10. **VERIFY EXISTING** -- Confirm the pre-existing `telnyx` trunk is still present and unaffected by test trunk operations.

**Expected Result:**
- Trunk CRUD operations complete without affecting existing trunks.
- Reload picks up trunk changes.
- Health endpoint remains stable through trunk modifications.

**Pass Criteria:**
- All CRUD operations succeed.
- Existing `telnyx` trunk is unaffected (still present after test trunk creation and deletion).
- Reload succeeds without errors.
- Password is not exposed in plaintext in read responses.

**Fail Action:**
- **Severity: High.** Trunk misconfiguration can break all external calling. If trunk operations corrupt the existing telnyx trunk, file a P0 defect. If CRUD partially fails, file P1. If password leaks in API responses, file P0 security defect.

---

### TC-L2-005: Call Records Listing and Filtering

**Priority:** P1
**Automated:** Yes
**Tool:** pytest + requests

**Preconditions:**
- Authenticated console session (TC-L2-001 passed).
- At least two completed calls exist in the CDR database (one internal, one external if possible). If no calls exist, generate them first by completing SIP calls (depends on L3 tests or manual pre-seeding).
- CDR storage is configured (`callrecord.type = "local"`, `callrecord.root = "./config/cdr"`).

**Steps:**
1. **LIST ALL** -- Send `GET /console/call-records` with the session cookie. Verify HTTP 200 and the response body contains a list or table of call records.
2. **VERIFY FIELDS** -- For each call record in the list, verify the presence of these fields:
   - `call_id` (non-empty string)
   - `direction` (one of: `inbound`, `outbound`, `internal`)
   - `from` (caller identifier)
   - `to` (callee identifier)
   - `duration` (numeric, >= 0)
   - `status` (e.g., `completed`, `no-answer`, `busy`, `failed`)
   - `start_time` or `created_at` (ISO 8601 or Unix timestamp)
3. **FILTER BY DIRECTION** -- If the UI or API supports filtering, request only `direction=inbound` records. Verify all returned records have direction `inbound`.
4. **FILTER BY DATE RANGE** -- If supported, filter by a date range that includes the test calls. Verify results fall within the specified range.
5. **FILTER BY STATUS** -- If supported, filter by `status=completed`. Verify all returned records have status `completed`.
6. **EMPTY FILTER** -- Apply a date range filter for a period with no calls (e.g., far future date). Verify the result set is empty.
7. **PAGINATION** -- If the endpoint supports pagination, verify that requesting page 1 with a small page size returns the correct number of records, and subsequent pages return different records.
8. **SORT ORDER** -- Verify records are sorted by timestamp (most recent first, or as specified by the API).

**Expected Result:**
- Call records are retrievable and contain all required fields.
- Filtering narrows the result set appropriately.
- Empty filters return empty results without errors.

**Pass Criteria:**
- All required fields are present in every returned record.
- Direction filter returns only records matching the specified direction.
- Date filter returns only records within the specified range.
- No HTTP errors (4xx/5xx) from valid filter requests.

**Fail Action:**
- **Severity: Medium.** CDR listing is an operational monitoring feature. If records are missing fields, file P1. If filtering returns incorrect results, file P1. If the endpoint returns errors, file P0.

---

### TC-L2-006: AMI API Authentication (IP Restriction)

**Priority:** P1
**Automated:** Partial
**Tool:** pytest + requests

**Preconditions:**
- RustPBX container running.
- Current AMI configuration: `allows = ["*"]` (wildcard, all IPs allowed).
- Access to modify `config.toml` and reload the container or AMI config.
- Test runner IP address is known (e.g., `172.17.0.1` for Docker host).

**Steps:**
1. **BASELINE (wildcard)** -- With `allows = ["*"]`, send `GET /ami/v1/health` from the test runner. Verify HTTP 200 with JSON body containing `"status":"running"`.
2. **BASELINE (dialogs)** -- Send `GET /ami/v1/dialogs`. Verify HTTP 200 with a JSON array response.
3. **BASELINE (transactions)** -- Send `GET /ami/v1/transactions`. Verify HTTP 200 with a JSON array response.
4. **RESTRICT TO SPECIFIC IP** -- Modify the AMI configuration to `allows = ["192.0.2.1"]` (an IP that is NOT the test runner's IP). Reload the configuration via `POST /ami/v1/reload/acl` or `POST /ami/v1/reload/app`, or restart the container.
5. **VERIFY DENIAL** -- Send `GET /ami/v1/health` from the test runner. Verify the request is denied (HTTP 403 Forbidden, or connection refused, or a non-200 status).
6. **VERIFY DENIAL (dialogs)** -- Send `GET /ami/v1/dialogs`. Verify denial.
7. **RESTORE WILDCARD** -- Change AMI configuration back to `allows = ["*"]`. Reload.
8. **VERIFY RESTORED ACCESS** -- Send `GET /ami/v1/health`. Verify HTTP 200 is returned again.
9. **RESTRICT TO TEST RUNNER IP** -- Set `allows = ["<test_runner_ip>"]` (the actual IP of the test machine, e.g., `172.17.0.1`). Reload.
10. **VERIFY ALLOWED IP** -- Send `GET /ami/v1/health` from the test runner. Verify HTTP 200.

**Expected Result:**
- Wildcard allows all requests.
- Restricting to a different IP blocks the test runner.
- Restricting to the test runner's own IP allows access.

**Pass Criteria:**
- Steps 1-3 return HTTP 200.
- Steps 5-6 return a non-200 response (403, 401, or connection refusal).
- Steps 8 and 10 return HTTP 200.

**Fail Action:**
- **Severity: High.** If IP restriction does not actually restrict access, the AMI API is unprotected. File a P0 security defect. If the wildcard mode blocks valid requests, file P0 functional defect. Note: This test is marked "Partial" because steps 4-6 require config modification and container interaction which may need manual setup depending on the test environment.

---

### TC-L2-007: AMI Reload Endpoints

**Priority:** P0
**Automated:** Yes
**Tool:** pytest + requests + curl

**Preconditions:**
- RustPBX container running and accessible.
- AMI allows wildcard access (`allows = ["*"]`).
- A known route exists (e.g., the internal route `telnyx-inbound` from `config/routes/telnyx.toml`).
- The routes file directory is writable (or a test route file can be mounted).

**Steps:**
1. **RELOAD ROUTES** -- Send `POST /ami/v1/reload/routes`. Verify:
   - HTTP 200 response.
   - Response body contains a success message or JSON confirmation.
   - Container logs show `routes reloaded` message.
2. **VERIFY ROUTES LOADED** -- After reload, check that existing routes are still active. If a test SIP call can be placed, verify routing works. Alternatively, check logs for the route count.
3. **RELOAD TRUNKS** -- Send `POST /ami/v1/reload/trunks`. Verify:
   - HTTP 200 response.
   - Container logs show `trunks reloaded` with the correct trunk count.
4. **VERIFY TRUNK LOADED** -- After reload, send `GET /ami/v1/health`. Verify the health response does not indicate trunk errors.
5. **RELOAD ACL** -- Send `POST /ami/v1/reload/acl`. Verify:
   - HTTP 200 response.
   - Container logs show ACL reload confirmation.
6. **RELOAD APP (full)** -- Send `POST /ami/v1/reload/app`. Verify:
   - HTTP 200 response.
   - Container logs show a full reload sequence (routes, trunks, ACL, and other modules).
7. **POST-RELOAD HEALTH** -- After full reload, send `GET /ami/v1/health`. Verify the system is still running and healthy.
8. **RAPID RELOAD** -- Send 5 consecutive `POST /ami/v1/reload/routes` requests in quick succession (within 1 second). Verify:
   - All requests return HTTP 200 (no race condition crashes).
   - The system remains healthy after the burst.
9. **INVALID RELOAD ENDPOINT** -- Send `POST /ami/v1/reload/nonexistent`. Verify the response is HTTP 404 (not found) and not a 500 server error.
10. **RELOAD WITH WRONG METHOD** -- Send `GET /ami/v1/reload/routes` (GET instead of POST). Verify the response is HTTP 405 Method Not Allowed or 404, not a reload.

**Expected Result:**
- All reload endpoints accept POST and return success.
- The system remains healthy after each reload.
- Invalid endpoints return appropriate error codes.
- Rapid reloads do not crash the system.

**Pass Criteria:**
- Steps 1, 3, 5, 6 return HTTP 200.
- Step 7 confirms system health post-reload.
- Step 8: all 5 requests return 200 and post-burst health check passes.
- Steps 9-10 return 4xx error codes (not 5xx).

**Fail Action:**
- **Severity: Critical.** Reload endpoints are essential for zero-downtime configuration changes. If reload causes a crash or corrupts state, file a P0 defect. If reload silently fails (returns 200 but doesn't actually reload), file P0. If rapid reloads cause a race condition, file P0.

---

### TC-L2-008: WebSocket Event Subscription

**Priority:** P1
**Automated:** Yes
**Tool:** pytest + websockets (Python library)

**Preconditions:**
- RustPBX container running with WebSocket handler at `/ws`.
- WebSocket port accessible at `ws://localhost:8080/ws`.
- At least one registered SIP extension (1001 or 1002) for generating events (optional, for event verification).

**Steps:**
1. **CONNECT** -- Open a WebSocket connection to `ws://localhost:8080/ws`. Verify:
   - The connection upgrades successfully (HTTP 101 Switching Protocols).
   - No immediate error frame or disconnect.
2. **VERIFY CONNECTION LOGGED** -- Check container logs for `created WebSocket channel connection` or equivalent.
3. **SEND SIP REGISTER** -- Over the WebSocket, send a valid SIP REGISTER message for extension 1001 (formatted as a SIP-over-WebSocket frame per RFC 7118). Verify:
   - The server responds with a SIP 401 Unauthorized (challenge) or 200 OK.
   - The response is received as a valid SIP message over the WebSocket.
4. **EVENT FORMAT** -- If the WebSocket supports event subscription (e.g., subscribe to registration events or call state events), send the appropriate subscription message. Verify:
   - The subscription is acknowledged.
   - Events are received in a documented format (JSON or SIP NOTIFY).
5. **MULTIPLE CONNECTIONS** -- Open a second WebSocket connection to `ws://localhost:8080/ws`. Verify:
   - Both connections remain active simultaneously.
   - Messages on one connection do not leak to the other.
6. **DISCONNECT** -- Close the first WebSocket connection gracefully (send close frame). Verify:
   - The server acknowledges the close.
   - Container logs show `WebSocket connection handler exiting` or equivalent.
   - The second connection remains active.
7. **RAPID RECONNECT** -- Disconnect and immediately reconnect 3 times in sequence. Verify:
   - Each connection/disconnection cycle completes without errors.
   - No resource leaks (server remains healthy).
8. **INVALID DATA** -- Send a non-SIP, non-JSON binary blob over the WebSocket. Verify:
   - The server handles it gracefully (ignores, sends error, or closes the connection -- but does NOT crash).

**Expected Result:**
- WebSocket connections are established and maintained correctly.
- SIP-over-WebSocket messages are processed.
- Multiple concurrent connections are supported.
- Graceful disconnect is handled properly.
- Invalid data does not crash the server.

**Pass Criteria:**
- Step 1: connection establishes (101 upgrade).
- Step 3: valid SIP response received.
- Step 5: both connections remain open.
- Step 6: clean disconnect, second connection unaffected.
- Step 8: server remains healthy after invalid data.

**Fail Action:**
- **Severity: High.** WebSocket is the transport for browser-based softphones (WebRTC/SIP.js). If connections fail, browser clients cannot function. File P0 for connection failures. File P1 for event format issues. File P0 for crash-on-invalid-data.

---

### TC-L2-009: Recording Download Endpoint

**Priority:** P2
**Automated:** Partial
**Tool:** pytest + requests

**Preconditions:**
- RustPBX container running with recording enabled (`recording.enabled = true`, `recording.auto_start = true`).
- At least one completed call with a recording exists. This may require running an L3 SIP test first, or pre-seeding a recording file in `./config/cdr/`.
- Authenticated console session available.
- CDR storage configured at `./config/cdr`.

**Steps:**
1. **IDENTIFY RECORDING** -- Send `GET /console/call-records` and identify a call record that has an associated recording. Note the `call_id` and any recording file reference (path or URL).
2. **DOWNLOAD RECORDING** -- Send a GET request to the recording download URL (the exact endpoint depends on implementation; possibilities include `/console/call-records/{call_id}/recording`, `/ami/v1/recording/{filename}`, or a direct file serving path). Verify:
   - HTTP 200 response.
   - `Content-Type` header is `audio/wav`, `audio/x-wav`, or `application/octet-stream`.
   - `Content-Length` header is present and greater than 0.
3. **VERIFY FILE SIZE** -- The downloaded file should be non-trivial in size:
   - For a 10-second call: expect at least 80,000 bytes (8 kHz mono PCM16 = ~16 KB/sec).
   - File size should be greater than 1,000 bytes (reject empty or corrupt files).
4. **VERIFY WAV HEADER** -- Read the first 44 bytes of the downloaded file. Verify:
   - Bytes 0-3: `RIFF` (ASCII `0x52494646`).
   - Bytes 8-11: `WAVE` (ASCII `0x57415645`).
   - This confirms the file is a valid WAV container.
5. **DOWNLOAD NON-EXISTENT RECORDING** -- Request a recording with a fabricated call ID (e.g., `nonexistent-call-id-12345`). Verify:
   - HTTP 404 Not Found (not a 500 error or a 200 with empty body).
6. **DOWNLOAD WITHOUT AUTH** -- If the recording endpoint requires authentication, send the request without a session cookie. Verify:
   - HTTP 401 or 302 redirect to login (not a 200 with the file).

**Expected Result:**
- Recordings are downloadable via HTTP.
- Downloaded files are valid WAV containers with non-trivial audio data.
- Missing recordings return 404.
- Unauthenticated requests are rejected.

**Pass Criteria:**
- Step 2: HTTP 200 with audio content type.
- Step 3: file size > 1,000 bytes.
- Step 4: valid RIFF/WAVE header bytes.
- Step 5: HTTP 404.
- Step 6: non-200 response.

**Fail Action:**
- **Severity: Medium.** Recording access is an operational feature. If downloads fail entirely, file P1. If recordings are corrupt (invalid WAV header), file P1. If unauthenticated users can download recordings, file P0 security defect. This test is marked "Partial" because it depends on a completed call with a recording, which may require manual or L3-level setup.

---

### TC-L2-010: API Error Handling (400, 401, 404, 500)

**Priority:** P1
**Automated:** Yes
**Tool:** pytest + requests

**Preconditions:**
- RustPBX container running.
- AMI allows wildcard access.
- Console login credentials available.

**Steps:**

**400 Bad Request:**
1. Send `POST /console/login` with an empty body (no `identifier` or `password` fields). Verify the response is HTTP 400 Bad Request, or HTTP 200 with an error message (depending on implementation). The response must NOT be a 303 redirect with a session cookie.
2. Send `POST /console/extensions` (or the extension creation endpoint) with malformed data (e.g., missing required `username` field, or `username` as an empty string). Verify the response indicates a validation error (HTTP 400 or 422).
3. If the AMI API accepts JSON, send `POST /ami/v1/reload/routes` with a malformed JSON body (e.g., `{broken`). Verify the response is HTTP 400 (not 500).

**401 Unauthorized / 403 Forbidden:**
4. Send `GET /console/` without any session cookie. Verify the response is HTTP 302/303 redirect to `/console/login`, or HTTP 401.
5. Send `GET /console/extensions` without a session cookie. Verify the same redirect/denial behavior.
6. Send `GET /console/settings` without a session cookie. Verify the same redirect/denial behavior.
7. Send `POST /console/login` with `identifier=admin&password=wrongpassword`. Verify the response is HTTP 200 with error text or HTTP 401 (not a 303 redirect).

**404 Not Found:**
8. Send `GET /console/nonexistent-page`. Verify HTTP 404.
9. Send `GET /ami/v1/nonexistent-endpoint`. Verify HTTP 404.
10. Send `GET /ami/v1/hangup/nonexistent-call-id-12345`. Verify the response is HTTP 404 or a JSON error indicating the call was not found (not a 500 crash).
11. Send `GET /completely-unknown-path`. Verify HTTP 404.

**500 Internal Server Error (negative testing):**
12. Send a request with an excessively long URL path (e.g., `/ami/v1/` followed by 10,000 random characters). Verify the server returns HTTP 414 URI Too Long or HTTP 404, but NOT a 500 crash.
13. Send a request with an excessively large `Content-Length` header but minimal body. Verify the server handles it gracefully (rejects or ignores, but does not crash).
14. Send `POST /console/login` with an extremely long `identifier` value (10,000 characters). Verify the server rejects it without crashing.

**Response Format Consistency:**
15. For every error response, verify the body is either HTML (for console pages) or JSON (for AMI API endpoints). Verify error responses do not expose stack traces, internal file paths, or debug information.

**Expected Result:**
- Each error scenario returns the appropriate HTTP status code.
- The server never crashes (returns 5xx) for client-side errors.
- Error responses are consistent in format and do not leak internal details.

**Pass Criteria:**
- Steps 1-3: no session cookie issued for bad input.
- Steps 4-7: protected pages reject unauthenticated access.
- Steps 8-11: unknown paths return 404 (not 500).
- Steps 12-14: extreme input handled gracefully (no crash).
- Step 15: no stack traces or internal paths in error responses.

**Fail Action:**
- **Severity: High.** If the server crashes (5xx) on bad input, file P0. If authentication can be bypassed via malformed requests, file P0 security defect. If 404 pages leak internal paths, file P1 security defect. If error format is inconsistent, file P2.

---

## Appendix A: API Endpoint Reference

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/console/login` | None | Login page |
| POST | `/console/login` | None | Submit credentials |
| GET | `/console/` | Session | Dashboard |
| GET | `/console/extensions` | Session | Extension list |
| POST | `/console/extensions` | Session | Create/update extension |
| GET | `/console/routing` | Session | Route list |
| GET | `/console/settings` | Session | System settings |
| GET | `/console/call-records` | Session | CDR listing |
| GET | `/console/diagnostics` | Session | System diagnostics |
| GET | `/ami/v1/health` | IP ACL | System health |
| GET | `/ami/v1/dialogs` | IP ACL | Active calls |
| GET | `/ami/v1/transactions` | IP ACL | SIP transactions |
| GET | `/ami/v1/hangup/{id}` | IP ACL | Terminate call |
| POST | `/ami/v1/reload/routes` | IP ACL | Hot-reload routes |
| POST | `/ami/v1/reload/trunks` | IP ACL | Hot-reload trunks |
| POST | `/ami/v1/reload/acl` | IP ACL | Hot-reload ACL |
| POST | `/ami/v1/reload/app` | IP ACL | Full reload |
| WS | `/ws` | None | SIP over WebSocket |

## Appendix B: Test Data

| Item | Value |
|------|-------|
| Admin user | `admin` / `admin123` |
| Test extension 1 | `1001` / `test1001` (memory backend) |
| Test extension 2 | `1002` / `test1002` (memory backend) |
| CRUD test extension | `9901` / `test9901` (created/deleted during tests) |
| CRUD test route | `test-route-l2` (created/deleted during tests) |
| CRUD test trunk | `test-trunk-l2` (created/deleted during tests) |
| Base URL | `http://localhost:8080` |
| WebSocket URL | `ws://localhost:8080/ws` |

---

*Document: `C:\Development\RustPBX\docs\tests\L2_API_CONTRACT_TESTS.md`*
*Last updated: 2026-02-21*
