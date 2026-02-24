"""
Call Monitoring API Tests -- Verify monitoring endpoints for active calls.

These tests validate the HTTP API contracts for the RustPBX call monitoring
feature.  The monitoring API allows a supervisor to silently listen, whisper,
or barge into an active call via three endpoints:

  POST /ami/v1/calls/{session_id}/monitor/start   -- start monitoring
  POST /ami/v1/calls/{session_id}/monitor/stop    -- stop monitoring
  POST /ami/v1/calls/{session_id}/monitor/mode    -- change monitor mode

Tests cover:
  1. Starting a monitor on a session (happy path with active call)
  2. Stopping a monitor on a session
  3. Changing monitor mode on a session
  4. Attempting to monitor a non-existent session (expect 404)
  5. Verifying all three mode values are accepted
  6. Full lifecycle: start -> change mode -> stop

Server:  RUSTPBX_HOST (default 127.0.0.1) : RUSTPBX_HTTP_PORT (default 8443)
Scheme:  RUSTPBX_SCHEME (default https)

Run with:
  python -m pytest tests/test_call_monitoring.py -v -s

Environment variables (all optional):
  RUSTPBX_HOST          HTTP server IP          (default: 127.0.0.1)
  RUSTPBX_HTTP_PORT     HTTP(S) port            (default: 8443)
  RUSTPBX_SCHEME        http or https           (default: https)
  RUSTPBX_VERIFY_TLS    Verify TLS certs        (default: false)
  RUSTPBX_ADMIN_USER    Admin username           (default: admin)
  RUSTPBX_ADMIN_PASS    Admin password           (default: admin123)
"""

import logging
import os
import uuid

import pytest
import requests
import urllib3

# Suppress InsecureRequestWarning for self-signed TLS certs
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in (
    "1", "true", "yes",
)
ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

BASE_URL = f"{SCHEME}://{SERVER_HOST}:{HTTP_PORT}"
AMI_URL = f"{BASE_URL}/ami/v1"

logger = logging.getLogger("test_call_monitoring")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _server_reachable() -> bool:
    """Quick check: is the server reachable via the HTTP health endpoint?"""
    try:
        resp = requests.get(
            f"{AMI_URL}/health",
            timeout=5,
            verify=VERIFY_TLS,
        )
        return resp.status_code == 200
    except requests.RequestException:
        return False


def _get_active_session_id() -> str | None:
    """Try to get an active call session ID from the server.

    Returns the first active call's session_id, or None if there are no
    active calls.  This is used by tests that need a real session to
    interact with.
    """
    try:
        resp = requests.get(
            f"{AMI_URL}/calls",
            timeout=5,
            verify=VERIFY_TLS,
        )
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list) and len(data) > 0:
                # Return the session_id (or call_id) of the first call
                first = data[0]
                return first.get("session_id") or first.get("call_id") or first.get("id")
            elif isinstance(data, dict):
                calls = data.get("calls", data.get("sessions", []))
                if calls and len(calls) > 0:
                    first = calls[0]
                    return first.get("session_id") or first.get("call_id") or first.get("id")
    except (requests.RequestException, ValueError, KeyError):
        pass
    return None


def _make_fake_session_id() -> str:
    """Generate a fake session ID that is guaranteed to not exist on the server."""
    return f"nonexistent-{uuid.uuid4().hex[:12]}"


def _monitor_start(session_id: str, mode: str = "silent_listen") -> requests.Response:
    """POST to /ami/v1/calls/{session_id}/monitor/start with the given mode."""
    return requests.post(
        f"{AMI_URL}/calls/{session_id}/monitor/start",
        json={"mode": mode},
        timeout=10,
        verify=VERIFY_TLS,
    )


def _monitor_stop(session_id: str) -> requests.Response:
    """POST to /ami/v1/calls/{session_id}/monitor/stop."""
    return requests.post(
        f"{AMI_URL}/calls/{session_id}/monitor/stop",
        timeout=10,
        verify=VERIFY_TLS,
    )


def _monitor_set_mode(session_id: str, mode: str) -> requests.Response:
    """POST to /ami/v1/calls/{session_id}/monitor/mode with the given mode."""
    return requests.post(
        f"{AMI_URL}/calls/{session_id}/monitor/mode",
        json={"mode": mode},
        timeout=10,
        verify=VERIFY_TLS,
    )


# ---------------------------------------------------------------------------
# Skip conditions
# ---------------------------------------------------------------------------

skip_no_server = pytest.mark.skipif(
    not _server_reachable(),
    reason=f"RustPBX server not reachable at {BASE_URL}",
)


# ---------------------------------------------------------------------------
# Test class: Monitor API with invalid/non-existent sessions
# ---------------------------------------------------------------------------

@skip_no_server
class TestMonitorInvalidSession:
    """Tests that verify correct error responses when the session does not exist."""

    @pytest.mark.timeout(15)
    def test_monitor_start_invalid_session_returns_404(self):
        """TC-MON-001: POST monitor/start on a non-existent session returns 404.

        The server should respond with HTTP 404 and a JSON error body when
        attempting to start monitoring on a session that does not exist.
        """
        fake_id = _make_fake_session_id()
        resp = _monitor_start(fake_id, "silent_listen")

        assert resp.status_code == 404, (
            f"Expected 404 for non-existent session, got {resp.status_code}: "
            f"{resp.text[:200]}"
        )

        # Response body should be JSON with an error message
        data = resp.json()
        assert data.get("status") == "error", (
            f"Expected status 'error', got {data}"
        )
        assert "not found" in data.get("message", "").lower(), (
            f"Expected 'not found' in error message, got: {data.get('message')}"
        )

    @pytest.mark.timeout(15)
    def test_monitor_stop_invalid_session_returns_404(self):
        """TC-MON-002: POST monitor/stop on a non-existent session returns 404."""
        fake_id = _make_fake_session_id()
        resp = _monitor_stop(fake_id)

        assert resp.status_code == 404, (
            f"Expected 404 for non-existent session, got {resp.status_code}: "
            f"{resp.text[:200]}"
        )

        data = resp.json()
        assert data.get("status") == "error"

    @pytest.mark.timeout(15)
    def test_monitor_mode_change_invalid_session_returns_404(self):
        """TC-MON-003: POST monitor/mode on a non-existent session returns 404."""
        fake_id = _make_fake_session_id()
        resp = _monitor_set_mode(fake_id, "whisper")

        assert resp.status_code == 404, (
            f"Expected 404 for non-existent session, got {resp.status_code}: "
            f"{resp.text[:200]}"
        )

        data = resp.json()
        assert data.get("status") == "error"


# ---------------------------------------------------------------------------
# Test class: Monitor mode values
# ---------------------------------------------------------------------------

@skip_no_server
class TestMonitorModeValues:
    """Tests that verify all three monitoring modes are accepted by the API.

    These tests use a non-existent session ID, but the mode validation happens
    in the JSON payload parsing before the session lookup.  If the server
    returns 404 (session not found) rather than 400/422 (bad mode), the mode
    value was accepted as valid.
    """

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("mode", ["silent_listen", "whisper", "barge"])
    def test_monitor_start_accepts_mode(self, mode):
        """TC-MON-004: Monitor start endpoint accepts all three mode values.

        Sends a monitor/start request with each of the three valid mode
        values.  Since no real call session exists, the expected response
        is 404 (not a 400/422 validation error), confirming the mode was
        parsed correctly.
        """
        fake_id = _make_fake_session_id()
        resp = _monitor_start(fake_id, mode)

        # 404 = mode was accepted but session not found (expected)
        # 400/422 = mode was rejected as invalid (unexpected)
        assert resp.status_code == 404, (
            f"Expected 404 (mode '{mode}' accepted but session not found), "
            f"got {resp.status_code}: {resp.text[:200]}"
        )

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("mode", ["silent_listen", "whisper", "barge"])
    def test_monitor_set_mode_accepts_mode(self, mode):
        """TC-MON-005: Monitor set-mode endpoint accepts all three mode values.

        Same validation approach as test_monitor_start_accepts_mode, but
        for the mode-change endpoint.
        """
        fake_id = _make_fake_session_id()
        resp = _monitor_set_mode(fake_id, mode)

        assert resp.status_code == 404, (
            f"Expected 404 (mode '{mode}' accepted but session not found), "
            f"got {resp.status_code}: {resp.text[:200]}"
        )

    @pytest.mark.timeout(15)
    def test_monitor_start_rejects_invalid_mode(self):
        """TC-MON-006: Monitor start endpoint rejects an invalid mode value.

        Sends a request with an invalid mode string.  The server should
        respond with a 400-level error (400 or 422) indicating the mode
        is not recognized.
        """
        fake_id = _make_fake_session_id()
        resp = requests.post(
            f"{AMI_URL}/calls/{fake_id}/monitor/start",
            json={"mode": "invalid_mode_value"},
            timeout=10,
            verify=VERIFY_TLS,
        )

        # The server should reject the invalid mode before looking up the session.
        # Acceptable error codes: 400 (Bad Request), 422 (Unprocessable Entity),
        # or any 4xx that is NOT 404.
        assert resp.status_code != 404, (
            f"Server returned 404 instead of rejecting invalid mode 'invalid_mode_value'. "
            f"This suggests the mode was accepted without validation."
        )
        assert 400 <= resp.status_code < 500, (
            f"Expected a 4xx client error for invalid mode, got {resp.status_code}: "
            f"{resp.text[:200]}"
        )


# ---------------------------------------------------------------------------
# Test class: Monitor endpoint structure
# ---------------------------------------------------------------------------

@skip_no_server
class TestMonitorEndpointStructure:
    """Tests that verify the monitor API endpoints exist and return proper
    JSON responses with expected fields."""

    @pytest.mark.timeout(15)
    def test_monitor_start_response_has_status_field(self):
        """TC-MON-007: Monitor start response body contains a 'status' field."""
        fake_id = _make_fake_session_id()
        resp = _monitor_start(fake_id, "silent_listen")

        data = resp.json()
        assert "status" in data, (
            f"Response JSON missing 'status' field. Keys: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_monitor_stop_response_has_status_field(self):
        """TC-MON-008: Monitor stop response body contains a 'status' field."""
        fake_id = _make_fake_session_id()
        resp = _monitor_stop(fake_id)

        data = resp.json()
        assert "status" in data, (
            f"Response JSON missing 'status' field. Keys: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_monitor_mode_response_has_status_field(self):
        """TC-MON-009: Monitor mode-change response body contains a 'status' field."""
        fake_id = _make_fake_session_id()
        resp = _monitor_set_mode(fake_id, "barge")

        data = resp.json()
        assert "status" in data, (
            f"Response JSON missing 'status' field. Keys: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_monitor_start_error_has_message_field(self):
        """TC-MON-010: Error response from monitor/start contains a 'message' field."""
        fake_id = _make_fake_session_id()
        resp = _monitor_start(fake_id, "silent_listen")

        # Should be 404 error
        assert resp.status_code == 404
        data = resp.json()
        assert "message" in data, (
            f"Error response missing 'message' field. Keys: {list(data.keys())}"
        )
        assert isinstance(data["message"], str) and len(data["message"]) > 0, (
            f"Error 'message' should be a non-empty string, got: {data['message']!r}"
        )


# ---------------------------------------------------------------------------
# Test class: Monitor lifecycle with active call (requires active session)
# ---------------------------------------------------------------------------

@skip_no_server
class TestMonitorLifecycleWithActiveCall:
    """Tests that exercise the full monitor lifecycle on an active call session.

    These tests require at least one active call on the server. If no active
    calls exist, the tests are skipped. To run these tests, establish a call
    before running the test suite (e.g., using a softphone).
    """

    @pytest.fixture(autouse=True)
    def _require_active_call(self):
        """Skip if there are no active call sessions on the server."""
        session_id = _get_active_session_id()
        if session_id is None:
            pytest.skip(
                "No active call sessions found on the server. "
                "Establish a call before running lifecycle tests."
            )
        self.session_id = session_id

    @pytest.mark.timeout(30)
    def test_monitor_start_on_active_call(self):
        """TC-MON-011: Start monitoring on an active call session.

        POSTs to monitor/start with mode 'silent_listen' and verifies
        the server returns 200 with status 'ok'.
        """
        resp = _monitor_start(self.session_id, "silent_listen")

        print(f"\n--- TC-MON-011: Monitor Start ---")
        print(f"  Session:  {self.session_id}")
        print(f"  Status:   {resp.status_code}")
        print(f"  Body:     {resp.text[:300]}")

        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}: {resp.text[:200]}"
        )
        data = resp.json()
        assert data.get("status") == "ok", (
            f"Expected status 'ok', got: {data}"
        )

        # Clean up: stop the monitor
        _monitor_stop(self.session_id)

    @pytest.mark.timeout(30)
    def test_monitor_stop_on_active_call(self):
        """TC-MON-012: Stop monitoring on an active call session.

        First starts monitoring, then stops it, verifying the stop
        response returns 200 with status 'ok'.
        """
        # Start monitoring first
        start_resp = _monitor_start(self.session_id, "silent_listen")
        if start_resp.status_code != 200:
            pytest.skip(f"Could not start monitor: {start_resp.status_code}")

        resp = _monitor_stop(self.session_id)

        print(f"\n--- TC-MON-012: Monitor Stop ---")
        print(f"  Session:  {self.session_id}")
        print(f"  Status:   {resp.status_code}")
        print(f"  Body:     {resp.text[:300]}")

        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}: {resp.text[:200]}"
        )
        data = resp.json()
        assert data.get("status") == "ok", (
            f"Expected status 'ok', got: {data}"
        )

    @pytest.mark.timeout(30)
    def test_monitor_mode_change_on_active_call(self):
        """TC-MON-013: Change monitor mode on an active call session.

        Starts monitoring in 'silent_listen' mode, then changes to 'whisper',
        verifying the mode-change response returns 200 with status 'ok'.
        """
        # Start monitoring
        start_resp = _monitor_start(self.session_id, "silent_listen")
        if start_resp.status_code != 200:
            pytest.skip(f"Could not start monitor: {start_resp.status_code}")

        try:
            resp = _monitor_set_mode(self.session_id, "whisper")

            print(f"\n--- TC-MON-013: Monitor Mode Change ---")
            print(f"  Session:  {self.session_id}")
            print(f"  Status:   {resp.status_code}")
            print(f"  Body:     {resp.text[:300]}")

            assert resp.status_code == 200, (
                f"Expected 200, got {resp.status_code}: {resp.text[:200]}"
            )
            data = resp.json()
            assert data.get("status") == "ok", (
                f"Expected status 'ok', got: {data}"
            )
        finally:
            _monitor_stop(self.session_id)

    @pytest.mark.timeout(60)
    def test_monitor_full_lifecycle_on_active_call(self):
        """TC-MON-014: Full monitor lifecycle: start -> mode changes -> stop.

        Exercises the complete monitoring lifecycle on an active call:
          1. Start monitoring in silent_listen mode
          2. Change mode to whisper
          3. Change mode to barge
          4. Change mode back to silent_listen
          5. Stop monitoring

        Each step is verified for a 200 response with status 'ok'.
        """
        session_id = self.session_id
        print(f"\n--- TC-MON-014: Full Monitor Lifecycle ---")
        print(f"  Session: {session_id}")

        # Step 1: Start in silent_listen
        resp = _monitor_start(session_id, "silent_listen")
        assert resp.status_code == 200, (
            f"Step 1 (start): Expected 200, got {resp.status_code}"
        )
        data = resp.json()
        assert data.get("status") == "ok"
        print(f"  Step 1 (start silent_listen): OK")

        try:
            # Step 2: Change to whisper
            resp = _monitor_set_mode(session_id, "whisper")
            assert resp.status_code == 200, (
                f"Step 2 (whisper): Expected 200, got {resp.status_code}"
            )
            data = resp.json()
            assert data.get("status") == "ok"
            print(f"  Step 2 (mode -> whisper): OK")

            # Step 3: Change to barge
            resp = _monitor_set_mode(session_id, "barge")
            assert resp.status_code == 200, (
                f"Step 3 (barge): Expected 200, got {resp.status_code}"
            )
            data = resp.json()
            assert data.get("status") == "ok"
            print(f"  Step 3 (mode -> barge): OK")

            # Step 4: Back to silent_listen
            resp = _monitor_set_mode(session_id, "silent_listen")
            assert resp.status_code == 200, (
                f"Step 4 (silent_listen): Expected 200, got {resp.status_code}"
            )
            data = resp.json()
            assert data.get("status") == "ok"
            print(f"  Step 4 (mode -> silent_listen): OK")

        finally:
            # Step 5: Stop monitoring
            resp = _monitor_stop(session_id)
            assert resp.status_code == 200, (
                f"Step 5 (stop): Expected 200, got {resp.status_code}"
            )
            data = resp.json()
            assert data.get("status") == "ok"
            print(f"  Step 5 (stop): OK")
