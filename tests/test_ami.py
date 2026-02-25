"""
AMI (Administrative Management Interface) Integration Tests

Tests the /ami/v1/ HTTP API endpoints:
  - GET    /health              System health status
  - GET    /dialogs             List active SIP dialogs
  - GET    /transactions        List active SIP transactions
  - GET    /frequency_limits    List frequency/rate limits
  - DELETE /frequency_limits    Clear frequency limits
  - GET    /backup/status       Backup status
  - GET    /backup/health       Backup health check
  - GET    /backup/history      Backup history

Requires a running RustPBX server with AMI auth (skips gracefully if unavailable).
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
ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

BASE_URL = f"{SCHEME}://{SERVER_HOST}:{HTTP_PORT}"
AMI_URL = f"{BASE_URL}/ami/v1"


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
# Session fixture: authenticated console session for AMI auth
# ---------------------------------------------------------------------------
@pytest.fixture(scope="module")
def ami_session():
    """Create an authenticated session for AMI endpoints."""
    session = requests.Session()
    session.verify = VERIFY_TLS
    try:
        resp = session.post(
            f"{BASE_URL}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            allow_redirects=False,
            timeout=10,
        )
    except requests.ConnectionError:
        pytest.skip("Cannot connect for login")
        return
    if resp.status_code not in (302, 303, 200):
        pytest.skip(f"Cannot login: HTTP {resp.status_code}")
    return session


# ---------------------------------------------------------------------------
# Helper
# ---------------------------------------------------------------------------
def _ami_get(session, path: str) -> requests.Response:
    return session.get(f"{AMI_URL}{path}", timeout=10)


def _ami_post(session, path: str, json=None) -> requests.Response:
    return session.post(f"{AMI_URL}{path}", json=json, timeout=10)


def _ami_delete(session, path: str) -> requests.Response:
    return session.delete(f"{AMI_URL}{path}", timeout=10)


# ---------------------------------------------------------------------------
# Tests: GET /health
# ---------------------------------------------------------------------------

@skip_no_server
class TestHealth:
    """Tests for GET /ami/v1/health."""

    def test_health_returns_200(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        assert resp.status_code == 200

    def test_health_returns_json(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        assert "application/json" in resp.headers.get("content-type", "")

    def test_health_has_status_field(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        data = resp.json()
        assert "status" in data
        assert data["status"] == "running"

    def test_health_has_version_info(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        data = resp.json()
        assert "version" in data

    def test_health_has_uptime(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        data = resp.json()
        assert "uptime" in data

    def test_health_has_call_counters(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        data = resp.json()
        assert "total" in data
        assert "failed" in data
        assert isinstance(data["total"], int)
        assert isinstance(data["failed"], int)

    def test_health_has_sipserver_stats(self, ami_session):
        resp = _ami_get(ami_session, "/health")
        data = resp.json()
        assert "sipserver" in data
        sip = data["sipserver"]
        assert "transactions" in sip
        assert "dialogs" in sip


# ---------------------------------------------------------------------------
# Tests: GET /dialogs
# ---------------------------------------------------------------------------

@skip_no_server
class TestDialogs:
    """Tests for GET /ami/v1/dialogs."""

    def test_list_dialogs_returns_200(self, ami_session):
        resp = _ami_get(ami_session, "/dialogs")
        assert resp.status_code == 200

    def test_list_dialogs_returns_array(self, ami_session):
        resp = _ami_get(ami_session, "/dialogs")
        data = resp.json()
        assert isinstance(data, list)


# ---------------------------------------------------------------------------
# Tests: GET /transactions
# ---------------------------------------------------------------------------

@skip_no_server
class TestTransactions:
    """Tests for GET /ami/v1/transactions."""

    def test_list_transactions_returns_200(self, ami_session):
        resp = _ami_get(ami_session, "/transactions")
        assert resp.status_code == 200

    def test_list_transactions_returns_array(self, ami_session):
        resp = _ami_get(ami_session, "/transactions")
        data = resp.json()
        assert isinstance(data, list)


# ---------------------------------------------------------------------------
# Tests: GET /hangup/{id} (non-existent dialog)
# ---------------------------------------------------------------------------

@skip_no_server
class TestHangupDialog:
    """Tests for GET /ami/v1/hangup/{id}."""

    def test_hangup_nonexistent_returns_404(self, ami_session):
        fake_id = str(uuid.uuid4())
        resp = _ami_get(ami_session, f"/hangup/{fake_id}")
        assert resp.status_code == 404

    def test_hangup_nonexistent_returns_error_json(self, ami_session):
        fake_id = str(uuid.uuid4())
        resp = _ami_get(ami_session, f"/hangup/{fake_id}")
        data = resp.json()
        assert data["status"] == "error"
        assert "not found" in data["message"].lower()


# ---------------------------------------------------------------------------
# Tests: GET /frequency_limits
# ---------------------------------------------------------------------------

@skip_no_server
class TestFrequencyLimits:
    """Tests for GET/DELETE /ami/v1/frequency_limits."""

    def test_list_frequency_limits_responds(self, ami_session):
        resp = _ami_get(ami_session, "/frequency_limits")
        # 200 if frequency limits configured, 501 if not implemented
        assert resp.status_code in (200, 501)

    def test_clear_frequency_limits_responds(self, ami_session):
        resp = _ami_delete(ami_session, "/frequency_limits")
        assert resp.status_code in (200, 501)


# ---------------------------------------------------------------------------
# Tests: Backup endpoints
# ---------------------------------------------------------------------------

@skip_no_server
class TestBackup:
    """Tests for /ami/v1/backup/* endpoints."""

    def test_backup_status_responds(self, ami_session):
        resp = _ami_get(ami_session, "/backup/status")
        # 200 if backup configured, 501 if not implemented
        assert resp.status_code in (200, 501)

    def test_backup_health_responds(self, ami_session):
        resp = _ami_get(ami_session, "/backup/health")
        assert resp.status_code in (200, 501)

    def test_backup_history_responds(self, ami_session):
        resp = _ami_get(ami_session, "/backup/history")
        assert resp.status_code in (200, 501)


# ---------------------------------------------------------------------------
# Tests: AMI auth enforcement
# ---------------------------------------------------------------------------

@skip_no_server
class TestAMIAuth:
    """Verify AMI endpoints respond to unauthenticated requests.

    When AMI auth is configured, endpoints return 401.
    When not configured, endpoints are open (return 200).
    """

    def test_health_unauthenticated_responds(self):
        resp = requests.get(
            f"{AMI_URL}/health",
            timeout=10,
            verify=VERIFY_TLS,
        )
        # 200 if no AMI auth configured, 401 if auth required
        assert resp.status_code in (200, 401)

    def test_dialogs_unauthenticated_responds(self):
        resp = requests.get(
            f"{AMI_URL}/dialogs",
            timeout=10,
            verify=VERIFY_TLS,
        )
        assert resp.status_code in (200, 401)

    def test_transactions_unauthenticated_responds(self):
        resp = requests.get(
            f"{AMI_URL}/transactions",
            timeout=10,
            verify=VERIFY_TLS,
        )
        assert resp.status_code in (200, 401)


# ---------------------------------------------------------------------------
# Tests: API contract validation
# ---------------------------------------------------------------------------

@skip_no_server
class TestAMIContract:
    """Verify API response format and content-type headers."""

    def test_dialogs_returns_json_content_type(self, ami_session):
        resp = _ami_get(ami_session, "/dialogs")
        assert "application/json" in resp.headers.get("content-type", "")

    def test_transactions_returns_json_content_type(self, ami_session):
        resp = _ami_get(ami_session, "/transactions")
        assert "application/json" in resp.headers.get("content-type", "")

    def test_nonexistent_endpoint_returns_404(self, ami_session):
        resp = _ami_get(ami_session, "/nonexistent")
        assert resp.status_code == 404
