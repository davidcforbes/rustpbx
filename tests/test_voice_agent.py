"""
Voice Agent API Integration Tests

Tests the /voice-agent/v1/ HTTP API endpoints:
  - POST   /calls              Create a new voice agent call
  - GET    /calls              List active voice agent calls
  - DELETE /calls/{session_id} Hangup a call
  - POST   /calls/{session_id}/command  Send command to active call

Requires a running RustPBX server (skips gracefully if unavailable).
"""
import os
import uuid

import pytest
import requests
import urllib3

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in ("1", "true", "yes")

BASE_URL = f"{SCHEME}://{SERVER_HOST}:{HTTP_PORT}"
VA_URL = f"{BASE_URL}/voice-agent/v1"


def _server_reachable() -> bool:
    """Check if the RustPBX server is reachable."""
    try:
        resp = requests.get(f"{BASE_URL}/", timeout=5, verify=VERIFY_TLS)
        return resp.status_code == 200
    except (requests.ConnectionError, requests.Timeout):
        return False


skip_no_server = pytest.mark.skipif(
    not _server_reachable(),
    reason=f"RustPBX server not reachable at {BASE_URL}",
)


# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

def _create_call(playbook: str | None = None) -> requests.Response:
    """POST /voice-agent/v1/calls"""
    payload = {}
    if playbook is not None:
        payload["playbook"] = playbook
    return requests.post(
        f"{VA_URL}/calls",
        json=payload,
        timeout=10,
        verify=VERIFY_TLS,
    )


def _list_calls() -> requests.Response:
    """GET /voice-agent/v1/calls"""
    return requests.get(
        f"{VA_URL}/calls",
        timeout=10,
        verify=VERIFY_TLS,
    )


def _hangup_call(session_id: str) -> requests.Response:
    """DELETE /voice-agent/v1/calls/{session_id}"""
    return requests.delete(
        f"{VA_URL}/calls/{session_id}",
        timeout=10,
        verify=VERIFY_TLS,
    )


def _send_command(session_id: str, command: dict) -> requests.Response:
    """POST /voice-agent/v1/calls/{session_id}/command"""
    return requests.post(
        f"{VA_URL}/calls/{session_id}/command",
        json=command,
        timeout=10,
        verify=VERIFY_TLS,
    )


# ---------------------------------------------------------------------------
# Tests: POST /calls (create call)
# ---------------------------------------------------------------------------

@skip_no_server
class TestCreateCall:
    """Tests for POST /voice-agent/v1/calls."""

    def test_create_call_returns_201(self):
        resp = _create_call()
        assert resp.status_code == 201

    def test_create_call_returns_session_id(self):
        resp = _create_call()
        data = resp.json()
        assert "sessionId" in data
        assert isinstance(data["sessionId"], str)
        assert len(data["sessionId"]) > 0

    def test_create_call_returns_created_status(self):
        resp = _create_call()
        data = resp.json()
        assert data["status"] == "created"

    def test_create_call_with_playbook(self):
        resp = _create_call(playbook="greeting")
        assert resp.status_code == 201
        data = resp.json()
        assert data["status"] == "created"

    def test_create_call_unique_session_ids(self):
        resp1 = _create_call()
        resp2 = _create_call()
        id1 = resp1.json()["sessionId"]
        id2 = resp2.json()["sessionId"]
        assert id1 != id2

    def test_create_call_session_id_is_valid_uuid(self):
        resp = _create_call()
        session_id = resp.json()["sessionId"]
        # Should be a valid UUID v4
        parsed = uuid.UUID(session_id, version=4)
        assert str(parsed) == session_id


# ---------------------------------------------------------------------------
# Tests: GET /calls (list calls)
# ---------------------------------------------------------------------------

@skip_no_server
class TestListCalls:
    """Tests for GET /voice-agent/v1/calls."""

    def test_list_calls_returns_200(self):
        resp = _list_calls()
        assert resp.status_code == 200

    def test_list_calls_returns_array(self):
        resp = _list_calls()
        data = resp.json()
        assert isinstance(data, list)

    def test_list_calls_items_have_session_id_and_status(self):
        resp = _list_calls()
        data = resp.json()
        for item in data:
            assert "sessionId" in item
            assert "status" in item


# ---------------------------------------------------------------------------
# Tests: DELETE /calls/{session_id} (hangup)
# ---------------------------------------------------------------------------

@skip_no_server
class TestHangupCall:
    """Tests for DELETE /voice-agent/v1/calls/{session_id}."""

    def test_hangup_nonexistent_returns_404(self):
        fake_id = str(uuid.uuid4())
        resp = _hangup_call(fake_id)
        assert resp.status_code == 404

    def test_hangup_empty_session_id_returns_404(self):
        # URL will be /calls/ which should not match the route
        resp = requests.delete(
            f"{VA_URL}/calls/",
            timeout=10,
            verify=VERIFY_TLS,
        )
        # Either 404 or 405 is acceptable for a missing path segment
        assert resp.status_code in (404, 405)


# ---------------------------------------------------------------------------
# Tests: POST /calls/{session_id}/command
# ---------------------------------------------------------------------------

@skip_no_server
class TestSendCommand:
    """Tests for POST /voice-agent/v1/calls/{session_id}/command."""

    def test_send_command_returns_202(self):
        resp = _send_command("any-session", {"action": "play", "text": "hello"})
        assert resp.status_code == 202

    def test_send_command_accepts_arbitrary_json(self):
        resp = _send_command("test-session", {
            "action": "tts",
            "text": "Buenos dias",
            "language": "es",
        })
        assert resp.status_code == 202

    def test_send_command_with_empty_body(self):
        resp = _send_command("test-session", {})
        assert resp.status_code == 202


# ---------------------------------------------------------------------------
# Tests: API contract validation
# ---------------------------------------------------------------------------

@skip_no_server
class TestAPIContract:
    """Verify API response format and content-type headers."""

    def test_create_call_returns_json_content_type(self):
        resp = _create_call()
        assert "application/json" in resp.headers.get("content-type", "")

    def test_list_calls_returns_json_content_type(self):
        resp = _list_calls()
        assert "application/json" in resp.headers.get("content-type", "")

    def test_invalid_method_on_calls_returns_405(self):
        """PATCH is not a supported method on /calls."""
        resp = requests.patch(
            f"{VA_URL}/calls",
            json={},
            timeout=10,
            verify=VERIFY_TLS,
        )
        assert resp.status_code == 405

    def test_create_call_invalid_json_returns_error(self):
        """Sending non-JSON body should return 4xx."""
        resp = requests.post(
            f"{VA_URL}/calls",
            data="not json",
            headers={"Content-Type": "application/json"},
            timeout=10,
            verify=VERIFY_TLS,
        )
        assert resp.status_code in (400, 422)
