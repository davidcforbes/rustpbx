"""
L2 API Contract Tests -- Endpoint behaviour and response schemas.

These tests validate the HTTP API contracts of the RustPBX AMI and console
interfaces. They check JSON response schemas, exercise reload commands, verify
authentication enforcement, and confirm proper error handling for invalid
requests and unauthorised access.

Target server: https://127.0.0.1:8443 (self-signed cert, TLS verification
disabled by default via conftest.py).

Run with:
    /root/test-env/bin/python -m pytest tests/test_L2_api.py -v

Expected execution time: < 60 seconds for the full suite.
"""
import pytest
import requests


# ---------------------------------------------------------------------------
# Helper: build a fresh session with TLS verification disabled
# ---------------------------------------------------------------------------
def _fresh_session(verify_tls):
    """Return a new requests.Session that honours the TLS verification flag."""
    s = requests.Session()
    s.verify = verify_tls
    return s


# ===========================================================================
# AMI Endpoint Tests
# ===========================================================================

class TestAMIHealth:
    """TC-L2-001: GET /ami/v1/health returns structured health data."""

    @pytest.mark.timeout(15)
    def test_health_returns_200(self, ami_url, verify_tls):
        """Health endpoint returns HTTP 200."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_health_is_json_object(self, ami_url, verify_tls):
        """Health response body is a non-empty JSON object."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        data = resp.json()
        assert isinstance(data, dict), (
            f"Expected JSON object, got {type(data).__name__}"
        )
        assert len(data) > 0, "Health response is empty"

    @pytest.mark.timeout(15)
    def test_health_contains_uptime(self, ami_url, verify_tls):
        """Health response contains an 'uptime' field."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        data = resp.json()
        assert "uptime" in data, (
            f"Missing 'uptime' field. Keys present: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_health_contains_version(self, ami_url, verify_tls):
        """Health response contains a 'version' field."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        data = resp.json()
        assert "version" in data, (
            f"Missing 'version' field. Keys present: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_health_contains_status_running(self, ami_url, verify_tls):
        """Health response has status == 'running'."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        data = resp.json()
        assert data.get("status") == "running", (
            f"Expected status 'running', got {data.get('status')!r}"
        )

    @pytest.mark.timeout(15)
    def test_health_contains_sipserver_stats(self, ami_url, verify_tls):
        """Health response contains a 'sipserver' block with dialog/call counts."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        data = resp.json()
        assert "sipserver" in data, (
            f"Missing 'sipserver' field. Keys present: {list(data.keys())}"
        )
        sip = data["sipserver"]
        assert isinstance(sip, dict), "sipserver should be a JSON object"
        # Should have at least dialogs and calls counts
        for key in ("dialogs", "calls"):
            assert key in sip, (
                f"Missing sipserver.{key}. Keys present: {list(sip.keys())}"
            )


class TestAMIDialogs:
    """TC-L2-002: GET /ami/v1/dialogs returns dialog list."""

    @pytest.mark.timeout(15)
    def test_dialogs_returns_200(self, ami_url, verify_tls):
        """Dialogs endpoint returns HTTP 200."""
        resp = requests.get(f"{ami_url}/dialogs", timeout=5, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_dialogs_returns_array(self, ami_url, verify_tls):
        """Dialogs response body is a JSON array (may be empty with no active calls)."""
        resp = requests.get(f"{ami_url}/dialogs", timeout=5, verify=verify_tls)
        data = resp.json()
        assert isinstance(data, list), (
            f"Expected JSON array, got {type(data).__name__}"
        )

    @pytest.mark.timeout(15)
    def test_dialogs_entries_are_dicts(self, ami_url, verify_tls):
        """If any dialogs exist, each entry should be a JSON object."""
        resp = requests.get(f"{ami_url}/dialogs", timeout=5, verify=verify_tls)
        data = resp.json()
        for i, entry in enumerate(data):
            assert isinstance(entry, dict), (
                f"Dialog entry [{i}] is not a dict: {type(entry).__name__}"
            )


class TestAMITransactions:
    """TC-L2-003: GET /ami/v1/transactions returns transaction list."""

    @pytest.mark.timeout(15)
    def test_transactions_returns_200(self, ami_url, verify_tls):
        """Transactions endpoint returns HTTP 200."""
        resp = requests.get(f"{ami_url}/transactions", timeout=5, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_transactions_returns_array(self, ami_url, verify_tls):
        """Transactions response body is a JSON array."""
        resp = requests.get(f"{ami_url}/transactions", timeout=5, verify=verify_tls)
        data = resp.json()
        assert isinstance(data, list), (
            f"Expected JSON array, got {type(data).__name__}"
        )

    @pytest.mark.timeout(15)
    def test_transactions_entries_are_strings(self, ami_url, verify_tls):
        """Each transaction entry should be a string (transaction key)."""
        resp = requests.get(f"{ami_url}/transactions", timeout=5, verify=verify_tls)
        data = resp.json()
        for i, entry in enumerate(data):
            assert isinstance(entry, str), (
                f"Transaction entry [{i}] is not a string: {type(entry).__name__}"
            )


class TestAMIReloadRoutes:
    """TC-L2-004: POST /ami/v1/reload/routes triggers route reload."""

    @pytest.mark.timeout(15)
    def test_reload_routes_returns_200(self, ami_url, verify_tls):
        """Reload routes endpoint returns HTTP 200."""
        resp = requests.post(f"{ami_url}/reload/routes", timeout=10, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_reload_routes_returns_json(self, ami_url, verify_tls):
        """Reload routes returns a JSON object with status field."""
        resp = requests.post(f"{ami_url}/reload/routes", timeout=10, verify=verify_tls)
        data = resp.json()
        assert isinstance(data, dict), (
            f"Expected JSON object, got {type(data).__name__}"
        )
        assert "status" in data, (
            f"Missing 'status' field. Keys present: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_reload_routes_status_ok(self, ami_url, verify_tls):
        """Reload routes returns status 'ok'."""
        resp = requests.post(f"{ami_url}/reload/routes", timeout=10, verify=verify_tls)
        data = resp.json()
        assert data.get("status") == "ok", (
            f"Expected status 'ok', got {data.get('status')!r}"
        )


class TestAMIReloadTrunks:
    """TC-L2-005: POST /ami/v1/reload/trunks triggers trunk reload."""

    @pytest.mark.timeout(15)
    def test_reload_trunks_returns_200(self, ami_url, verify_tls):
        """Reload trunks endpoint returns HTTP 200."""
        resp = requests.post(f"{ami_url}/reload/trunks", timeout=10, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_reload_trunks_returns_json(self, ami_url, verify_tls):
        """Reload trunks returns a JSON object with status field."""
        resp = requests.post(f"{ami_url}/reload/trunks", timeout=10, verify=verify_tls)
        data = resp.json()
        assert isinstance(data, dict), (
            f"Expected JSON object, got {type(data).__name__}"
        )
        assert "status" in data, (
            f"Missing 'status' field. Keys present: {list(data.keys())}"
        )

    @pytest.mark.timeout(15)
    def test_reload_trunks_status_ok(self, ami_url, verify_tls):
        """Reload trunks returns status 'ok'."""
        resp = requests.post(f"{ami_url}/reload/trunks", timeout=10, verify=verify_tls)
        data = resp.json()
        assert data.get("status") == "ok", (
            f"Expected status 'ok', got {data.get('status')!r}"
        )


class TestAMIFrequencyLimits:
    """TC-L2-006: GET /ami/v1/frequency_limits returns limit data or 501."""

    @pytest.mark.timeout(15)
    def test_frequency_limits_returns_valid_status(self, ami_url, verify_tls):
        """Frequency limits endpoint returns 200 (configured) or 501 (not configured)."""
        resp = requests.get(f"{ami_url}/frequency_limits", timeout=5, verify=verify_tls)
        assert resp.status_code in (200, 501), (
            f"Expected 200 or 501, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_frequency_limits_returns_json(self, ami_url, verify_tls):
        """Frequency limits response is valid JSON regardless of config status."""
        resp = requests.get(f"{ami_url}/frequency_limits", timeout=5, verify=verify_tls)
        data = resp.json()
        # 200 -> could be a list or object; 501 -> object with 'status'
        if resp.status_code == 501:
            assert isinstance(data, dict), "501 response should be a JSON object"
            assert data.get("status") == "unavailable", (
                f"Expected status 'unavailable', got {data.get('status')!r}"
            )
        else:
            # 200 -- could be a list of limits or an object
            assert isinstance(data, (list, dict)), (
                f"Expected JSON array or object, got {type(data).__name__}"
            )


# ===========================================================================
# Console Endpoint Tests
# ===========================================================================

class TestConsoleLoginPage:
    """TC-L2-007: Console login page is publicly accessible."""

    @pytest.mark.timeout(15)
    def test_login_page_returns_200(self, base_url, verify_tls):
        """GET /console/login returns HTTP 200."""
        resp = requests.get(
            f"{base_url}/console/login", timeout=5, verify=verify_tls
        )
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_login_page_is_html(self, base_url, verify_tls):
        """Login page response contains HTML content."""
        resp = requests.get(
            f"{base_url}/console/login", timeout=5, verify=verify_tls
        )
        content_type = resp.headers.get("Content-Type", "")
        assert "text/html" in content_type, (
            f"Expected text/html, got {content_type!r}"
        )

    @pytest.mark.timeout(15)
    def test_login_page_has_form(self, base_url, verify_tls):
        """Login page contains a login form with password input."""
        resp = requests.get(
            f"{base_url}/console/login", timeout=5, verify=verify_tls
        )
        body = resp.text.lower()
        assert "password" in body, "Login page should contain a password field"


# ===========================================================================
# Authentication / Authorization Tests
# ===========================================================================

class TestUnauthorizedAccess:
    """TC-L2-008: Protected endpoints reject unauthenticated requests."""

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("path", [
        "/console/",
        "/console/extensions",
        "/console/routing",
        "/console/settings",
        "/console/call-records",
        "/console/diagnostics",
    ])
    def test_console_pages_require_auth(self, base_url, verify_tls, path):
        """Unauthenticated access to console pages is redirected or denied."""
        session = _fresh_session(verify_tls)
        resp = session.get(
            f"{base_url}{path}",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401, 403), (
            f"Page {path}: expected auth redirect/deny, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_console_auth_redirects_to_login(self, base_url, verify_tls):
        """Unauthenticated console access redirects to the login page."""
        session = _fresh_session(verify_tls)
        resp = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        if resp.status_code in (302, 303):
            location = resp.headers.get("Location", "")
            assert "login" in location.lower(), (
                f"Redirect target should contain 'login', got {location!r}"
            )


class TestConsoleAuthFlow:
    """TC-L2-008b: Full console authentication lifecycle."""

    @pytest.mark.timeout(20)
    def test_login_logout_cycle(self, base_url, verify_tls):
        """Login -> access protected page -> logout -> verify session invalidated."""
        session = _fresh_session(verify_tls)

        # Step 1: Login
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": "admin", "password": "admin123"},
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303), (
            f"Login: expected redirect, got {resp.status_code}"
        )
        assert len(session.cookies) > 0, "No session cookie set after login"

        # Step 2: Access protected page
        resp = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code == 200, (
            f"Protected page after login: expected 200, got {resp.status_code}"
        )

        # Step 3: Logout
        resp = session.get(
            f"{base_url}/console/logout",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (200, 302, 303), (
            f"Logout: unexpected status {resp.status_code}"
        )

        # Step 4: Verify session is invalidated
        resp = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401), (
            f"Post-logout: expected auth redirect/deny, got {resp.status_code}"
        )


# ===========================================================================
# Error Handling Tests
# ===========================================================================

class TestNotFoundHandling:
    """TC-L2-009: Invalid endpoints return 404."""

    @pytest.mark.timeout(15)
    def test_ami_nonexistent_endpoint_returns_404(self, ami_url, verify_tls):
        """GET to a non-existent AMI path returns 404."""
        resp = requests.get(
            f"{ami_url}/nonexistent/endpoint",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 404, (
            f"Expected 404, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_ami_nonexistent_post_returns_404(self, ami_url, verify_tls):
        """POST to a non-existent AMI path returns 404 or 405."""
        resp = requests.post(
            f"{ami_url}/nonexistent/action",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code in (404, 405), (
            f"Expected 404 or 405, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_root_nonexistent_path_returns_404(self, base_url, verify_tls):
        """GET to a completely unknown root path returns 404."""
        resp = requests.get(
            f"{base_url}/this/path/does/not/exist",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 404, (
            f"Expected 404, got {resp.status_code}"
        )


class TestMethodNotAllowed:
    """TC-L2-009b: Wrong HTTP methods return 405."""

    @pytest.mark.timeout(15)
    def test_get_on_reload_routes_returns_405(self, ami_url, verify_tls):
        """GET on a POST-only endpoint returns 405 Method Not Allowed."""
        resp = requests.get(
            f"{ami_url}/reload/routes",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 405, (
            f"Expected 405, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_post_on_health_returns_405(self, ami_url, verify_tls):
        """POST on a GET-only endpoint returns 405 Method Not Allowed."""
        resp = requests.post(
            f"{ami_url}/health",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 405, (
            f"Expected 405, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_post_on_dialogs_returns_405(self, ami_url, verify_tls):
        """POST on GET-only dialogs endpoint returns 405."""
        resp = requests.post(
            f"{ami_url}/dialogs",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 405, (
            f"Expected 405, got {resp.status_code}"
        )


# ===========================================================================
# Additional AMI Endpoint Tests
# ===========================================================================

class TestAMIReloadACL:
    """TC-L2-010: POST /ami/v1/reload/acl triggers ACL reload."""

    @pytest.mark.timeout(15)
    def test_reload_acl_returns_200(self, ami_url, verify_tls):
        """Reload ACL endpoint returns HTTP 200."""
        resp = requests.post(f"{ami_url}/reload/acl", timeout=10, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Expected 200, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_reload_acl_returns_json_with_status(self, ami_url, verify_tls):
        """Reload ACL returns a JSON object with status 'ok'."""
        resp = requests.post(f"{ami_url}/reload/acl", timeout=10, verify=verify_tls)
        data = resp.json()
        assert isinstance(data, dict), (
            f"Expected JSON object, got {type(data).__name__}"
        )
        assert data.get("status") == "ok", (
            f"Expected status 'ok', got {data.get('status')!r}"
        )


class TestAMIHangupNonexistent:
    """TC-L2-011: GET /ami/v1/hangup/{id} for nonexistent dialog returns 404."""

    @pytest.mark.timeout(15)
    def test_hangup_nonexistent_returns_404(self, ami_url, verify_tls):
        """Hangup with a fake dialog ID returns 404 Not Found."""
        resp = requests.get(
            f"{ami_url}/hangup/nonexistent-dialog-id-12345",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 404, (
            f"Expected 404, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_hangup_nonexistent_returns_error_json(self, ami_url, verify_tls):
        """Hangup 404 response contains JSON with error status."""
        resp = requests.get(
            f"{ami_url}/hangup/nonexistent-dialog-id-12345",
            timeout=5,
            verify=verify_tls,
        )
        data = resp.json()
        assert isinstance(data, dict), "Expected JSON object"
        assert data.get("status") == "error", (
            f"Expected status 'error', got {data.get('status')!r}"
        )


class TestAMIResponseHeaders:
    """TC-L2-012: AMI endpoints return proper content-type headers."""

    @pytest.mark.timeout(15)
    def test_health_content_type_json(self, ami_url, verify_tls):
        """Health endpoint returns application/json content type."""
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        content_type = resp.headers.get("Content-Type", "")
        assert "application/json" in content_type, (
            f"Expected application/json, got {content_type!r}"
        )

    @pytest.mark.timeout(15)
    def test_dialogs_content_type_json(self, ami_url, verify_tls):
        """Dialogs endpoint returns application/json content type."""
        resp = requests.get(f"{ami_url}/dialogs", timeout=5, verify=verify_tls)
        content_type = resp.headers.get("Content-Type", "")
        assert "application/json" in content_type, (
            f"Expected application/json, got {content_type!r}"
        )

    @pytest.mark.timeout(15)
    def test_transactions_content_type_json(self, ami_url, verify_tls):
        """Transactions endpoint returns application/json content type."""
        resp = requests.get(f"{ami_url}/transactions", timeout=5, verify=verify_tls)
        content_type = resp.headers.get("Content-Type", "")
        assert "application/json" in content_type, (
            f"Expected application/json, got {content_type!r}"
        )
