# L0 Smoke Tests -- RustPBX

**Level:** L0 -- Smoke
**Purpose:** Fastest possible validation that the system is alive and minimally functional. Every test must complete in under 5 seconds. If any L0 test fails, the build is broken and no higher-level tests should run.
**Execution Order:** Sequential (TC-L0-001 must pass before the rest are attempted).
**Total Test Cases:** 8

---

## Summary Table

| ID         | Title                                  | Priority | Tool          | Time Limit |
|------------|----------------------------------------|----------|---------------|------------|
| TC-L0-001  | Container starts and stays running     | P0       | docker CLI    | 5 s        |
| TC-L0-002  | HTTP port 8080 responds                | P0       | curl / pytest | 2 s        |
| TC-L0-003  | SIP port 5060 accepts UDP              | P0       | python socket | 2 s        |
| TC-L0-004  | SIP port 5060 accepts TCP              | P0       | python socket | 2 s        |
| TC-L0-005  | Database migrations applied            | P0       | curl / pytest | 3 s        |
| TC-L0-006  | Console login page loads               | P1       | curl / pytest | 2 s        |
| TC-L0-007  | Super user creation via CLI succeeds   | P1       | docker CLI    | 5 s        |
| TC-L0-008  | AMI health endpoint responds           | P0       | curl / pytest | 2 s        |

---

## Test Cases

### TC-L0-001: Container starts and stays running

**Priority:** P0
**Automated:** Yes
**Tool:** docker CLI / pytest (subprocess)

**Preconditions:**
- Docker engine is running on the test host.
- The RustPBX image has been built or pulled (tag known to the test harness).
- No conflicting container is using ports 5060, 8080, or the 20000-20100 range.

**Steps:**
1. Run `docker compose up -d` (or the equivalent single-container `docker run -d` command) targeting the RustPBX image with the standard port mappings (5060/udp, 5060/tcp, 8080/tcp, 20000-20100/udp).
2. Wait 3 seconds for the container to initialize.
3. Run `docker inspect --format='{{.State.Status}}' <container>` and capture the output.
4. Run `docker inspect --format='{{.State.ExitCode}}' <container>` and capture the output.
5. Verify the container has been running for at least 2 seconds without restarting by checking `docker inspect --format='{{.State.StartedAt}}' <container>` and confirming `RestartCount` is 0.

**Expected Result:**
- `Status` is exactly the string `running`.
- `ExitCode` is `0`.
- `RestartCount` is `0`.
- The container has been up for at least 2 continuous seconds (no crash loop).

**Pass Criteria:**
- All four assertions above are true within 5 seconds of issuing the `docker run` / `docker compose up` command.

**Fail Action:**
- **Severity:** BLOCKER. Abort the entire test suite.
- Capture `docker logs <container>` (last 200 lines) and attach to the test report.
- Check for port conflicts (`ss -tlnp` / `netstat -tlnp` on the host) and include in diagnostics.
- File as a P0 defect: "Container fails to start or crashes on launch."

---

### TC-L0-002: HTTP port 8080 responds

**Priority:** P0
**Automated:** Yes
**Tool:** curl / pytest (requests)

**Preconditions:**
- TC-L0-001 has passed (container is running).
- Port 8080 on the host is mapped to the container's 8080.

**Steps:**
1. Open a TCP connection to `127.0.0.1:8080` with a 2-second timeout.
2. Send a minimal HTTP request: `GET / HTTP/1.0\r\nHost: localhost\r\n\r\n`.
3. Read the response (at least the status line).

**Expected Result:**
- The TCP connection succeeds (no `ConnectionRefused`, no timeout).
- An HTTP response is received. The status line matches the pattern `HTTP/1.[01] <3-digit-code>`. Any valid HTTP status code (200, 301, 302, 404, etc.) is acceptable -- the point is that the HTTP server is listening and responding.

**Pass Criteria:**
- A syntactically valid HTTP response status line is received within 2 seconds.

**Fail Action:**
- **Severity:** BLOCKER. Abort suite.
- Verify the container is still running (re-check TC-L0-001 assertions).
- Capture `docker logs <container>` and `ss -tlnp | grep 8080`.
- File as P0: "HTTP listener on port 8080 is not responding."

---

### TC-L0-003: SIP port 5060 accepts UDP

**Priority:** P0
**Automated:** Yes
**Tool:** python socket (UDP)

**Preconditions:**
- TC-L0-001 has passed.
- Port 5060/udp on the host is mapped to the container.

**Steps:**
1. Create a UDP socket with a 2-second receive timeout.
2. Send a minimal SIP OPTIONS request to `127.0.0.1:5060`:
   ```
   OPTIONS sip:ping@127.0.0.1 SIP/2.0\r\n
   Via: SIP/2.0/UDP 127.0.0.1:15060;branch=z9hG4bK-smoke-001;rport\r\n
   From: <sip:smoke@127.0.0.1>;tag=smoke001\r\n
   To: <sip:ping@127.0.0.1>\r\n
   Call-ID: smoke-l0-003@127.0.0.1\r\n
   CSeq: 1 OPTIONS\r\n
   Max-Forwards: 70\r\n
   Content-Length: 0\r\n
   \r\n
   ```
3. Attempt to receive a response datagram (up to 2048 bytes).

**Expected Result:**
- Either: a SIP response is received (any response code: 200, 405, 501, etc.), OR
- No ICMP port-unreachable error is raised (i.e., the port is open and the packet was accepted even if no response arrives within the timeout).

**Pass Criteria:**
- The UDP send does not raise a `ConnectionRefused` exception (ICMP reject).
- Ideally, a SIP response starting with `SIP/2.0` is received. If the socket times out without error, the test still passes (UDP is fire-and-forget; the port is open).

**Fail Action:**
- **Severity:** BLOCKER. Abort suite.
- If `ConnectionRefused`: the SIP UDP listener is not bound. Check container logs for bind errors.
- File as P0: "SIP UDP port 5060 is not accepting packets."

---

### TC-L0-004: SIP port 5060 accepts TCP

**Priority:** P0
**Automated:** Yes
**Tool:** python socket (TCP)

**Preconditions:**
- TC-L0-001 has passed.
- Port 5060/tcp on the host is mapped to the container.

**Steps:**
1. Create a TCP socket with a 2-second connection timeout.
2. Attempt to connect to `127.0.0.1:5060`.
3. If the connection succeeds, immediately close the socket.

**Expected Result:**
- The TCP three-way handshake completes successfully (no `ConnectionRefused`, no timeout).

**Pass Criteria:**
- `socket.connect(('127.0.0.1', 5060))` returns without error within 2 seconds.

**Fail Action:**
- **Severity:** BLOCKER. Abort suite.
- Verify the container is still running.
- Check for SIP TCP listener errors in `docker logs`.
- File as P0: "SIP TCP port 5060 is not accepting connections."

---

### TC-L0-005: Database migrations applied

**Priority:** P0
**Automated:** Yes
**Tool:** curl / pytest (requests)

**Preconditions:**
- TC-L0-001 and TC-L0-002 have passed (container running, HTTP alive).
- The database backend (PostgreSQL or SQLite) is configured and reachable by the container.

**Steps:**
1. Send `GET http://127.0.0.1:8080/ami/v1/health` with a 3-second timeout and accept JSON.
2. Parse the JSON response body.
3. Look for a field indicating database status (e.g., `database`, `db_status`, `db`, or `migrations`).
4. Alternatively, if no health detail is available, run `docker exec <container> rustpbx --check-db` or inspect startup logs for migration success messages (e.g., "migrations applied", "database ready", "tables created").

**Expected Result:**
- The health endpoint (or logs) confirms the database is connected and migrations have been applied.
- At minimum, core tables exist: `users` (or `sip_users`), `cdrs` (or `call_records`), `recordings` (or equivalent).

**Pass Criteria:**
- One of the following is true:
  - The health endpoint returns a JSON object with a database-related field whose value indicates success (e.g., `"status": "ok"`, `"database": "connected"`, `"migrations": "applied"`), OR
  - Container logs contain a line matching the pattern `migration` or `database.*ready` (case-insensitive) without any subsequent error.

**Fail Action:**
- **Severity:** BLOCKER. Abort suite.
- Capture the full health endpoint response and the last 100 lines of container logs.
- Check database connectivity: is PostgreSQL running? Is the connection string correct?
- File as P0: "Database migrations not applied or database unreachable."

---

### TC-L0-006: Console login page loads

**Priority:** P1
**Automated:** Yes
**Tool:** curl / pytest (requests)

**Preconditions:**
- TC-L0-002 has passed (HTTP port is responding).

**Steps:**
1. Send `GET http://127.0.0.1:8080/console/login` with a 2-second timeout.
2. Capture the HTTP status code.
3. Capture the `Content-Type` response header.
4. Read the response body (first 4096 bytes are sufficient).

**Expected Result:**
- HTTP status code is `200`.
- `Content-Type` header contains `text/html`.
- The response body contains at least one HTML indicator: the string `<form` or `<input` or `login` (case-insensitive).

**Pass Criteria:**
- Status is 200, content type includes `text/html`, and the body contains recognizable login form markup.

**Fail Action:**
- **Severity:** HIGH. Continue remaining L0 tests, but flag.
- If 404: the console route is not registered. Check that the console feature is compiled in.
- If 500: server-side error. Capture response body and container logs.
- File as P1: "Console login page is not loading."

---

### TC-L0-007: Super user creation via CLI succeeds

**Priority:** P1
**Automated:** Yes
**Tool:** docker CLI / pytest (subprocess)

**Preconditions:**
- TC-L0-001 has passed (container is running).
- No admin user with the test username already exists (or the command is idempotent).

**Steps:**
1. Run the super-user creation command inside the container:
   ```
   docker exec <container> rustpbx create-superuser --username smoketest_admin --password smoketest_pass123
   ```
   (Adjust the exact CLI syntax to match the RustPBX binary's subcommand structure. Alternatives might include `rustpbx user create --admin` or a similar variant.)
2. Capture the exit code and stdout/stderr.
3. Optionally, verify the user exists by attempting a console login (POST to `/console/login` with the created credentials).

**Expected Result:**
- The command exits with code `0`.
- stdout or stderr contains a success indicator (e.g., "user created", "superuser created", "ok") and no error messages.

**Pass Criteria:**
- Exit code is `0`.
- No error keywords in output (`error`, `failed`, `panic`, `unable` -- case-insensitive).

**Fail Action:**
- **Severity:** HIGH. Continue remaining L0 tests, but flag.
- Capture the full command output.
- Check if the database is writable (migrations applied, disk not full for SQLite).
- File as P1: "Super user creation via CLI failed."

---

### TC-L0-008: AMI health endpoint responds

**Priority:** P0
**Automated:** Yes
**Tool:** curl / pytest (requests)

**Preconditions:**
- TC-L0-002 has passed (HTTP port is responding).

**Steps:**
1. Send `GET http://127.0.0.1:8080/ami/v1/health` with a 2-second timeout.
2. Set the `Accept: application/json` header.
3. Capture the HTTP status code.
4. Parse the response body as JSON.

**Expected Result:**
- HTTP status code is `200`.
- The response body is valid JSON (no parse errors).
- The JSON object contains a key `status` (case-insensitive key match acceptable).
- The value of `status` is a string (e.g., `"ok"`, `"healthy"`, `"running"`).

**Pass Criteria:**
- Status code is 200 AND the body is valid JSON AND a `status` field is present with a non-empty string value.

**Fail Action:**
- **Severity:** BLOCKER. Abort suite.
- If 404: the AMI health route is not registered. Verify AMI feature is compiled and route prefix is `/ami/v1/`.
- If the body is not valid JSON: log the raw body for inspection.
- File as P0: "AMI health endpoint not responding or returning invalid data."

---

## Execution Notes

### Ordering and Gating
Tests should execute in numerical order. TC-L0-001 is a hard gate: if the container is not running, all subsequent tests are meaningless and must be skipped (marked as `BLOCKED`). TC-L0-002 through TC-L0-004 are also gates for their respective protocol families.

### Environment Variables
The test harness should accept the following environment variables for flexibility:

| Variable              | Default           | Description                        |
|-----------------------|-------------------|------------------------------------|
| `RUSTPBX_HOST`        | `127.0.0.1`       | Host where the container is mapped |
| `RUSTPBX_HTTP_PORT`   | `8080`             | HTTP/console port                  |
| `RUSTPBX_SIP_PORT`    | `5060`             | SIP signaling port                 |
| `RUSTPBX_CONTAINER`   | `rustpbx`          | Container name or ID               |
| `RUSTPBX_IMAGE`       | `rustpbx:latest`   | Image to use if starting fresh     |

### Timeout Policy
Every individual test has a hard wall-clock timeout of 5 seconds. If a test exceeds this, it is marked `FAIL` with reason `TIMEOUT` and the fail action is executed.

### Cleanup
After the full L0 suite completes (pass or fail), capture `docker logs <container> --tail=500` and attach to the test report. If TC-L0-007 created a `smoketest_admin` user, that user should be deleted in teardown (or ignored if using an ephemeral container).
