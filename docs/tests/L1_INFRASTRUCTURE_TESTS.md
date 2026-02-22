# L1 Infrastructure Tests -- RustPBX

**Level:** L1 -- Infrastructure
**Purpose:** Deeper connectivity and protocol-level validation. These tests verify that each subsystem (SIP, WebSocket, TLS, RTP, database, console) is not just alive but functionally correct at the protocol layer. L1 tests assume all L0 smoke tests have passed.
**Execution Order:** Sequential within each group. All tests are independent unless noted.
**Total Test Cases:** 9

---

## Summary Table

| ID         | Title                                      | Priority | Tool             | Time Limit |
|------------|--------------------------------------------|----------|------------------|------------|
| TC-L1-001  | SIP OPTIONS ping returns 200 OK            | P0       | SIPp / pytest    | 5 s        |
| TC-L1-002  | SIP REGISTER with valid credentials        | P0       | PJSUA / SIPp     | 10 s       |
| TC-L1-003  | SIP REGISTER with invalid credentials      | P0       | SIPp / pytest    | 5 s        |
| TC-L1-004  | WebSocket upgrade at /ws succeeds          | P0       | pytest (websockets) | 5 s     |
| TC-L1-005  | TLS certificate validation                 | P1       | openssl / pytest | 5 s        |
| TC-L1-006  | RTP port range accessible                  | P1       | python socket    | 10 s       |
| TC-L1-007  | Database connectivity                      | P0       | curl / pytest    | 5 s        |
| TC-L1-008  | Console session management                 | P1       | pytest (requests)| 10 s       |
| TC-L1-009  | DNS resolution from container              | P2       | docker exec      | 5 s        |

---

## Test Cases

### TC-L1-001: SIP OPTIONS ping returns 200 OK

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp / pytest (socket)

**Preconditions:**
- All L0 tests have passed.
- SIP port 5060 is accepting both UDP and TCP (TC-L0-003 and TC-L0-004 passed).

**Steps:**
1. Construct a syntactically valid SIP OPTIONS request:
   ```
   OPTIONS sip:ping@127.0.0.1:5060 SIP/2.0
   Via: SIP/2.0/UDP 127.0.0.1:15060;branch=z9hG4bK-infra-001;rport
   From: "L1 Test" <sip:l1test@127.0.0.1>;tag=l1001
   To: <sip:ping@127.0.0.1>
   Call-ID: l1-001-options@127.0.0.1
   CSeq: 1 OPTIONS
   Max-Forwards: 70
   User-Agent: RustPBX-L1-Test/1.0
   Accept: application/sdp
   Content-Length: 0
   ```
2. Send via UDP to `127.0.0.1:5060`.
3. Wait up to 3 seconds for a response.
4. Parse the SIP response status line.
5. Repeat the same test over TCP: open a TCP connection to port 5060, send the same request (with `Via: SIP/2.0/TCP`), read the response.

**Expected Result:**
- A SIP response is received on both UDP and TCP.
- The response status line is `SIP/2.0 200 OK`.
- The response contains valid `Via`, `From`, `To`, `Call-ID`, and `CSeq` headers that match or echo the request.
- Optionally, an `Allow` header listing supported SIP methods is present.

**Pass Criteria:**
- Both UDP and TCP responses have status code `200`.
- The `CSeq` in the response matches `1 OPTIONS`.

**Fail Action:**
- **Severity:** CRITICAL.
- If no response on UDP: verify firewall rules, check if SIP process is bound (container logs).
- If a non-200 response (e.g., 405 Method Not Allowed): the SIP stack may not support OPTIONS. Log the response code and headers. This is still a concern but may not be a blocker for call testing.
- File as P0: "SIP OPTIONS does not return 200 OK."

---

### TC-L1-002: SIP REGISTER with valid credentials

**Priority:** P0
**Automated:** Yes
**Tool:** PJSUA / SIPp / pytest

**Preconditions:**
- All L0 tests have passed.
- Test user `1001` exists in the memory backend with password `test1001`.
- SIP port 5060 is reachable.

**Steps:**
1. Send an unauthenticated SIP REGISTER request:
   ```
   REGISTER sip:127.0.0.1:5060 SIP/2.0
   Via: SIP/2.0/UDP 127.0.0.1:15060;branch=z9hG4bK-reg-001;rport
   From: <sip:1001@127.0.0.1>;tag=reg1001
   To: <sip:1001@127.0.0.1>
   Call-ID: l1-002-register@127.0.0.1
   CSeq: 1 REGISTER
   Contact: <sip:1001@127.0.0.1:15060;transport=udp>
   Max-Forwards: 70
   Expires: 3600
   Content-Length: 0
   ```
2. Receive the response. Expect `401 Unauthorized` with a `WWW-Authenticate` header.
3. Parse the `WWW-Authenticate` header to extract: `realm`, `nonce`, and optionally `algorithm` and `qop`.
4. Compute the Digest authentication response:
   - `HA1 = MD5(username:realm:password)` = `MD5(1001:<realm>:test1001)`
   - `HA2 = MD5(REGISTER:sip:127.0.0.1:5060)`
   - `response = MD5(HA1:nonce:HA2)`
   - (If `qop=auth`, include `nc`, `cnonce` per RFC 2617.)
5. Send a second REGISTER with the `Authorization` header containing the computed digest:
   ```
   REGISTER sip:127.0.0.1:5060 SIP/2.0
   Via: SIP/2.0/UDP 127.0.0.1:15060;branch=z9hG4bK-reg-002;rport
   From: <sip:1001@127.0.0.1>;tag=reg1001
   To: <sip:1001@127.0.0.1>
   Call-ID: l1-002-register@127.0.0.1
   CSeq: 2 REGISTER
   Contact: <sip:1001@127.0.0.1:15060;transport=udp>
   Max-Forwards: 70
   Expires: 3600
   Authorization: Digest username="1001", realm="<realm>", nonce="<nonce>", uri="sip:127.0.0.1:5060", response="<computed>", algorithm=MD5
   Content-Length: 0
   ```
6. Receive the response.

**Expected Result:**
- Step 2: The server responds with `SIP/2.0 401 Unauthorized` and a `WWW-Authenticate` header containing at least `realm` and `nonce`.
- Step 6: The server responds with `SIP/2.0 200 OK`.
- The 200 OK response contains a `Contact` header reflecting the registered binding.
- The 200 OK response contains an `Expires` header or a `Contact` parameter with an expires value.

**Pass Criteria:**
- First REGISTER produces a `401` with `WWW-Authenticate`.
- Second REGISTER (with valid digest) produces a `200`.
- The entire exchange completes in under 10 seconds.

**Fail Action:**
- **Severity:** CRITICAL.
- If first REGISTER returns something other than 401 (e.g., 403, 500): the authentication challenge mechanism is broken.
- If second REGISTER returns 401 again: the digest computation may be wrong (check realm, algorithm) or the user credentials in the memory backend are not matching.
- If second REGISTER returns 403: the server rejected valid credentials. Check the user store.
- Capture the full SIP message exchange (request + response for both rounds) in the test report.
- File as P0: "SIP REGISTER with valid credentials does not succeed."

---

### TC-L1-003: SIP REGISTER with invalid credentials returns 403

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp / pytest

**Preconditions:**
- All L0 tests have passed.
- SIP port 5060 is reachable.
- The username `baduser` does NOT exist in the system, OR the password `wrongpassword` is known to be incorrect for user `1001`.

**Steps:**
1. Send an unauthenticated SIP REGISTER for user `1001`:
   ```
   REGISTER sip:127.0.0.1:5060 SIP/2.0
   Via: SIP/2.0/UDP 127.0.0.1:15060;branch=z9hG4bK-badreg-001;rport
   From: <sip:1001@127.0.0.1>;tag=badreg1001
   To: <sip:1001@127.0.0.1>
   Call-ID: l1-003-badreg@127.0.0.1
   CSeq: 1 REGISTER
   Contact: <sip:1001@127.0.0.1:15060;transport=udp>
   Max-Forwards: 70
   Expires: 3600
   Content-Length: 0
   ```
2. Receive the `401 Unauthorized` challenge. Parse `WWW-Authenticate`.
3. Compute a Digest response using the WRONG password (`wrongpassword` instead of `test1001`).
4. Send the second REGISTER with the incorrect Authorization digest.
5. Receive the response.

**Expected Result:**
- Step 2: `401 Unauthorized` with `WWW-Authenticate` (same as TC-L1-002).
- Step 5: The server responds with `SIP/2.0 403 Forbidden` (or `401 Unauthorized` with a fresh nonce -- either is acceptable as a rejection).
- The server does NOT respond with `200 OK`.

**Pass Criteria:**
- The final response status code is `403` or `401` (NOT `200`).
- No registration binding is created for the user (verifiable via a subsequent query or via AMI API if available).

**Fail Action:**
- **Severity:** CRITICAL -- this is a security test.
- If the server returns `200 OK`: AUTHENTICATION IS BROKEN. This is a security vulnerability.
- Immediately halt testing and file as P0/SECURITY: "Server accepts invalid SIP credentials."
- Capture the full message exchange.

---

### TC-L1-004: WebSocket upgrade at /ws succeeds

**Priority:** P0
**Automated:** Yes
**Tool:** pytest (websockets library)

**Preconditions:**
- TC-L0-002 has passed (HTTP port is responding).
- The WebSocket endpoint is configured at `/ws` on port 8080.

**Steps:**
1. Initiate a WebSocket handshake to `ws://127.0.0.1:8080/ws` using the `sip` subprotocol:
   ```python
   import websockets
   ws = await websockets.connect(
       'ws://127.0.0.1:8080/ws',
       subprotocols=['sip'],
       open_timeout=5
   )
   ```
2. Verify the connection is open (the handshake completed).
3. Check the negotiated subprotocol (should be `sip`).
4. Send a minimal SIP OPTIONS message over the WebSocket:
   ```
   OPTIONS sip:ping@127.0.0.1 SIP/2.0
   Via: SIP/2.0/WS 127.0.0.1;branch=z9hG4bK-ws-001;rport
   From: <sip:wstest@127.0.0.1>;tag=ws001
   To: <sip:ping@127.0.0.1>
   Call-ID: l1-004-ws@127.0.0.1
   CSeq: 1 OPTIONS
   Max-Forwards: 70
   Content-Length: 0
   ```
5. Wait up to 3 seconds for a response message on the WebSocket.
6. Close the WebSocket connection cleanly.

**Expected Result:**
- The WebSocket handshake completes with HTTP `101 Switching Protocols`.
- The negotiated subprotocol is `sip`.
- Optionally, a SIP response is received over the WebSocket (verifying bidirectional communication). If no response is received within the timeout, the test still passes on the handshake alone.

**Pass Criteria:**
- WebSocket connection is established without exception.
- The `Upgrade: websocket` and `Connection: Upgrade` headers were received (implicit in a successful `websockets.connect`).
- The subprotocol is `sip`.

**Fail Action:**
- **Severity:** CRITICAL.
- If connection refused: the WebSocket endpoint is not configured or the HTTP server does not handle upgrades.
- If 404: the `/ws` path is not registered.
- If the subprotocol is rejected: verify the server supports the `sip` subprotocol.
- File as P0: "WebSocket upgrade at /ws failed."

---

### TC-L1-005: TLS certificate validation

**Priority:** P1
**Automated:** Partial
**Tool:** openssl s_client / pytest (ssl)

**Preconditions:**
- TLS is enabled in the RustPBX configuration (this test is skipped if TLS is not configured).
- The TLS port is known (typically 5061 for SIP-TLS, or 8443 for HTTPS).
- The expected certificate hostname/CN is known.

**Steps:**
1. Check if TLS is enabled by inspecting the container's configuration or environment variables. If TLS is not enabled, mark this test as `SKIPPED` (not `FAIL`).
2. If TLS is enabled on port 5061 (SIP-TLS):
   ```bash
   echo | openssl s_client -connect 127.0.0.1:5061 -servername <hostname> 2>/dev/null
   ```
3. If TLS is enabled on port 8443 (HTTPS):
   ```bash
   echo | openssl s_client -connect 127.0.0.1:8443 -servername <hostname> 2>/dev/null
   ```
4. Capture the certificate chain output.
5. Verify:
   - The handshake completes (no "handshake failure" in output).
   - The certificate subject or SAN matches the expected hostname.
   - The certificate is not expired (check `notBefore` and `notAfter`).
   - The certificate chain depth is at least 1 (self-signed) or ideally > 1 (CA-signed).

**Expected Result:**
- TLS handshake completes successfully.
- The server presents a valid certificate.
- The certificate's validity period includes the current date.
- The certificate's CN or SAN matches the configured hostname.

**Pass Criteria:**
- `openssl s_client` output contains `Verify return code: 0 (ok)` OR, for self-signed certificates in test environments, the handshake completes and the certificate dates are valid.
- No `handshake failure`, `connection refused`, or `certificate expired` errors.

**Fail Action:**
- **Severity:** HIGH.
- If TLS is enabled but handshake fails: check the certificate file paths in the configuration, verify the files exist and are readable.
- If the certificate is expired: flag for immediate renewal.
- If the CN/SAN does not match: flag configuration mismatch.
- File as P1: "TLS certificate validation failed."

---

### TC-L1-006: RTP port range accessible

**Priority:** P1
**Automated:** Yes
**Tool:** python socket (UDP)

**Preconditions:**
- TC-L0-001 has passed (container is running).
- Ports 20000-20100/udp on the host are mapped to the container.

**Steps:**
1. Select a sample of ports from the RTP range to probe. Testing all 101 ports is not required. Sample: `[20000, 20010, 20025, 20050, 20075, 20099, 20100]` (7 ports).
2. For each sampled port:
   a. Create a UDP socket with a 1-second timeout.
   b. Send a small probe packet (4 bytes: `0x00 0x00 0x00 0x00`) to `127.0.0.1:<port>`.
   c. Attempt to receive a response (expect timeout, not error).
   d. Record whether the send succeeded without `ConnectionRefused` (ICMP reject).
3. Tally results.

**Expected Result:**
- For all sampled ports, the UDP send succeeds without raising a `ConnectionRefused` exception.
- No response is expected (RTP ports only respond during active media sessions), but the port must be open (no ICMP port-unreachable).

**Pass Criteria:**
- At least 6 out of 7 sampled ports accept the UDP probe without ICMP reject (allowing for 1 transient failure).
- Zero ports raise `ConnectionRefused`.

**Fail Action:**
- **Severity:** HIGH.
- If ports are rejected: verify the Docker port mapping (`-p 20000-20100:20000-20100/udp`) and check the container's RTP configuration.
- If all ports are rejected: the RTP listener may not be bound, or Docker port mapping is missing entirely.
- File as P1: "RTP port range 20000-20100 is not accessible."

---

### TC-L1-007: Database connectivity

**Priority:** P0
**Automated:** Yes
**Tool:** curl / pytest (requests)

**Preconditions:**
- TC-L0-002 and TC-L0-008 have passed (HTTP and AMI health are responding).

**Steps:**
1. Send `GET http://127.0.0.1:8080/ami/v1/health` and parse the JSON response.
2. Inspect the response for database-specific status fields. Look for any of these patterns:
   - `"database": "connected"` or `"database": { "status": "ok" }`
   - `"db_status": "ok"` or `"db": "up"`
   - `"components": { "database": { ... } }`
3. If the health endpoint does not expose database details, fall back to a functional test:
   a. Send `GET http://127.0.0.1:8080/ami/v1/users` (or any API that queries the database).
   b. Verify the response is `200` with a JSON array (even if empty).
4. As a second fallback, check container logs:
   ```bash
   docker logs <container> 2>&1 | grep -i 'database\|postgres\|sqlite\|migration'
   ```
   Verify no error patterns are present.

**Expected Result:**
- The database is confirmed reachable and functional via at least one method.
- No database error messages (e.g., "connection refused", "no such table", "migration failed") appear in logs.

**Pass Criteria:**
- At least one of the three verification methods (health endpoint, API query, log inspection) confirms database connectivity without errors.

**Fail Action:**
- **Severity:** CRITICAL.
- If using PostgreSQL: verify the PostgreSQL container is running, the connection string is correct, and network connectivity exists between containers.
- If using SQLite: verify the database file path is writable and not corrupted.
- Capture the health response, the API response, and the last 50 log lines related to database activity.
- File as P0: "Database connectivity check failed."

---

### TC-L1-008: Console session management

**Priority:** P1
**Automated:** Yes
**Tool:** pytest (requests)

**Preconditions:**
- TC-L0-006 has passed (console login page loads).
- Admin credentials are known: username `admin`, password `admin123`.

**Steps:**
1. **Login:** Send a POST request to `http://127.0.0.1:8080/console/login` with form data:
   ```
   username=admin&password=admin123
   ```
   Use `Content-Type: application/x-www-form-urlencoded`.
   Follow redirects (or capture the `302` and its `Location` header).
   Capture the `Set-Cookie` response header.
2. **Verify Cookie:** Confirm that a session cookie was set. The cookie name might be `session`, `sid`, `token`, or similar. Extract the cookie value.
3. **Access Protected Page:** Send a GET request to `http://127.0.0.1:8080/console/` (or `/console/dashboard`, `/console/index`) including the session cookie.
   Verify the response is `200 OK` and contains HTML with console content (not a redirect to login).
4. **Logout:** Send a POST (or GET) request to `http://127.0.0.1:8080/console/logout` with the session cookie.
   Capture the response.
5. **Verify Session Invalidated:** Send a GET request to the same protected page from step 3, using the same session cookie.
   Verify the response is either a `302` redirect to `/console/login` or a `401 Unauthorized`.

**Expected Result:**
- Step 1: Login succeeds with a `200` or `302` response and a `Set-Cookie` header.
- Step 2: A non-empty session cookie is present.
- Step 3: The protected page returns `200` with HTML content (not a login redirect).
- Step 4: Logout returns `200` or `302` (redirect to login).
- Step 5: The old session cookie is no longer valid; the server redirects to login or returns `401`.

**Pass Criteria:**
- All five steps produce the expected results.
- The full login-access-logout-verify cycle completes in under 10 seconds.

**Fail Action:**
- **Severity:** HIGH.
- If login fails (no cookie, 401, 403): verify the admin credentials. Check if TC-L0-007 (super user creation) was needed first.
- If the protected page is accessible without a cookie: SESSION SECURITY IS BROKEN. File as P0/SECURITY.
- If logout does not invalidate the session: the session store is not clearing entries. File as P1.
- Capture all HTTP requests and responses (headers + body) for the full cycle.
- File as P1: "Console session management is not working correctly."

---

### TC-L1-009: DNS resolution from container

**Priority:** P2
**Automated:** Yes
**Tool:** docker exec

**Preconditions:**
- TC-L0-001 has passed (container is running).
- The container has network access (not running in `--network=none` mode).

**Steps:**
1. Execute a DNS lookup inside the container for a well-known external hostname:
   ```bash
   docker exec <container> nslookup google.com
   ```
   If `nslookup` is not available in the container, try alternatives:
   ```bash
   docker exec <container> getent hosts google.com
   ```
   Or:
   ```bash
   docker exec <container> ping -c 1 -W 2 google.com
   ```
   (We only need DNS resolution, not actual connectivity; even a failed ping with a resolved IP is fine.)
2. Capture the output and exit code.
3. Verify that an IP address was resolved.

**Expected Result:**
- The DNS query resolves `google.com` to at least one IP address.
- The command exit code is `0` (for `nslookup` or `getent`) or the output contains a resolved IP (for `ping`).

**Pass Criteria:**
- An IPv4 or IPv6 address is present in the output (matches pattern `\d+\.\d+\.\d+\.\d+` or contains `:`).
- The command does not fail with "server can't find", "Name or service not known", or similar DNS failure messages.

**Fail Action:**
- **Severity:** MEDIUM.
- This test failing means the container cannot reach external services (SIP trunks, STUN/TURN servers, external APIs).
- Check Docker network configuration: is the container using a custom network with no DNS? Is the host's DNS resolver accessible?
- Check `/etc/resolv.conf` inside the container.
- File as P2: "Container cannot resolve external DNS."

---

## Execution Notes

### Dependency on L0
The entire L1 suite must be gated behind a full L0 pass. If any L0 test is in `FAIL` status, the L1 suite should not execute and all L1 tests should be marked `BLOCKED`.

### Parallel Execution
The following tests are fully independent and may run in parallel to reduce total execution time:
- Group A: TC-L1-001, TC-L1-003, TC-L1-004, TC-L1-005, TC-L1-006, TC-L1-009
- Group B (sequential): TC-L1-002 (must complete before any test that depends on a registered user)
- Group C (sequential): TC-L1-008 (login -> access -> logout must be sequential within the test)

TC-L1-007 may run in parallel with any other test.

### Environment Variables

| Variable              | Default           | Description                              |
|-----------------------|-------------------|------------------------------------------|
| `RUSTPBX_HOST`        | `127.0.0.1`       | Host where the container is mapped       |
| `RUSTPBX_HTTP_PORT`   | `8080`             | HTTP/console port                        |
| `RUSTPBX_SIP_PORT`    | `5060`             | SIP signaling port                       |
| `RUSTPBX_WS_PATH`     | `/ws`              | WebSocket endpoint path                  |
| `RUSTPBX_RTP_START`   | `20000`            | First port in the RTP range              |
| `RUSTPBX_RTP_END`     | `20100`            | Last port in the RTP range               |
| `RUSTPBX_CONTAINER`   | `rustpbx`          | Container name or ID                     |
| `RUSTPBX_TLS_ENABLED` | `false`            | Whether TLS tests should run             |
| `RUSTPBX_TLS_PORT`    | `5061`             | TLS port for SIP-TLS                     |
| `RUSTPBX_SIP_USER`    | `1001`             | Test SIP username                        |
| `RUSTPBX_SIP_PASS`    | `test1001`         | Test SIP password                        |
| `RUSTPBX_ADMIN_USER`  | `admin`            | Console admin username                   |
| `RUSTPBX_ADMIN_PASS`  | `admin123`         | Console admin password                   |

### Timeout Policy
Individual test timeouts are listed in the summary table. The total L1 suite should complete within 60 seconds. If any individual test exceeds its time limit, it is marked `FAIL` with reason `TIMEOUT`.

### SIP Message Logging
For all SIP-related tests (TC-L1-001 through TC-L1-003), the full SIP message exchange (sent and received) must be captured and included in the test report, regardless of pass/fail status. This aids in debugging protocol-level issues.

### Cleanup
- Any SIP registrations created during TC-L1-002 should be unregistered in teardown (send REGISTER with `Expires: 0`).
- Any console sessions created during TC-L1-008 should be logged out in teardown.
- If using SIPp, capture scenario logs and include them in the test report.
