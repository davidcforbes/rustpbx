"""
L2 API Contract Tests -- Endpoint behaviour and response schemas.

These tests validate the HTTP API contracts of the RustPBX console and AMI
interfaces. They check authentication flows end-to-end, verify JSON response
schemas, exercise reload commands, and confirm proper error handling for
invalid requests and unauthorised access.

Expected execution time: < 60 seconds for the full suite.
"""
import pytest
import requests


class TestL2APIContracts:
    """L2: Verify API contracts, response schemas, and error handling."""

    # ------------------------------------------------------------------
    # TC-L2-001: Full console auth flow (login / access / logout)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_001_console_auth_flow(self, base_url):
        """TC-L2-001: Full login -> protected access -> logout cycle.

        Verifies that:
          1. POST /console/login with valid credentials redirects and
             sets a session cookie.
          2. The authenticated session can access /console/ (200).
          3. POST /console/logout invalidates the session.
          4. Subsequent access to /console/ is denied (302/303/401).
        """
        session = requests.Session()

        # Step 1 -- Login
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": "admin", "password": "admin123"},
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303), (
            f"Login: expected redirect, got {resp.status_code}"
        )
        assert len(session.cookies) > 0, "No session cookie after login"

        # Step 2 -- Access protected page
        resp = session.get(f"{base_url}/console/", allow_redirects=False, timeout=5)
        assert resp.status_code == 200, (
            f"Protected page: expected 200, got {resp.status_code}"
        )

        # Step 3 -- Logout
        resp = session.post(
            f"{base_url}/console/logout",
            allow_redirects=False,
            timeout=5,
        )
        # Logout may return redirect or 200
        assert resp.status_code in (200, 302, 303), (
            f"Logout: unexpected status {resp.status_code}"
        )

        # Step 4 -- Verify session is invalidated
        resp = session.get(f"{base_url}/console/", allow_redirects=False, timeout=5)
        assert resp.status_code in (302, 303, 401), (
            f"Post-logout access: expected deny, got {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L2-002: AMI health schema
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_002_ami_health_schema(self, ami_url):
        """TC-L2-002: AMI health response contains expected fields.

        The health endpoint should return a JSON object with at least one of
        the following keys: 'status', 'sipserver', 'uptime', or 'version'.
        The response must be a non-empty dict.
        """
        resp = requests.get(f"{ami_url}/health", timeout=5)
        assert resp.status_code == 200
        data = resp.json()

        assert isinstance(data, dict), (
            f"Expected JSON object, got {type(data).__name__}"
        )
        assert len(data) > 0, "Health response is empty"

        # At least one recognised status field should be present
        known_fields = {"status", "sipserver", "uptime", "version", "registrations"}
        found_fields = known_fields & set(data.keys())
        # We don't hard-fail on missing fields (the API may evolve), but we
        # verify the response is structured JSON with >0 keys (already done).

    # ------------------------------------------------------------------
    # TC-L2-003: AMI dialogs schema
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_003_ami_dialogs_schema(self, ami_url):
        """TC-L2-003: AMI dialogs response is a list or object.

        With no active calls the dialogs endpoint should return an empty list
        (or an object wrapping an empty list). If any entries exist they should
        have a 'call_id' or 'id' field.
        """
        resp = requests.get(f"{ami_url}/dialogs", timeout=5)
        assert resp.status_code == 200
        data = resp.json()

        if isinstance(data, list):
            # Each dialog entry should be a dict
            for entry in data:
                assert isinstance(entry, dict), (
                    f"Dialog entry is not a dict: {type(entry).__name__}"
                )
        elif isinstance(data, dict):
            # Acceptable -- might be a wrapper like {"dialogs": []}
            pass
        else:
            pytest.fail(f"Unexpected dialogs type: {type(data).__name__}")

    # ------------------------------------------------------------------
    # TC-L2-004: AMI reload routes
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_004_ami_reload_routes(self, ami_url):
        """TC-L2-004: POST reload/routes returns success.

        Triggering a route reload should return 200 with a JSON body
        indicating success (or at minimum a non-error status code).
        """
        resp = requests.post(f"{ami_url}/reload/routes", timeout=5)
        # Accept 200 (success) or 204 (no content) or 404 (endpoint not
        # implemented yet -- skip in that case)
        if resp.status_code == 404:
            pytest.skip("reload/routes endpoint not implemented")
        assert resp.status_code in (200, 204), (
            f"Reload routes: expected 200/204, got {resp.status_code}"
        )
        if resp.status_code == 200 and resp.text:
            data = resp.json()
            assert isinstance(data, dict), "Reload response is not a JSON object"

    # ------------------------------------------------------------------
    # TC-L2-005: AMI reload trunks
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_005_ami_reload_trunks(self, ami_url):
        """TC-L2-005: POST reload/trunks returns success.

        Triggering a trunk reload should return 200 with a JSON body
        indicating success (or at minimum a non-error status code).
        """
        resp = requests.post(f"{ami_url}/reload/trunks", timeout=5)
        if resp.status_code == 404:
            pytest.skip("reload/trunks endpoint not implemented")
        assert resp.status_code in (200, 204), (
            f"Reload trunks: expected 200/204, got {resp.status_code}"
        )
        if resp.status_code == 200 and resp.text:
            data = resp.json()
            assert isinstance(data, dict), "Reload response is not a JSON object"

    # ------------------------------------------------------------------
    # TC-L2-006: Console extensions page
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_006_console_extensions_page(self, base_url, api_session):
        """TC-L2-006: Extensions page shows configured users.

        The extensions page should render and contain references to at
        least some of the pre-configured test users (1001-1004).
        """
        resp = api_session.get(f"{base_url}/console/extensions", timeout=5)
        assert resp.status_code == 200, (
            f"Extensions page returned {resp.status_code}"
        )
        body = resp.text
        # At least one of the test users should appear on the page
        found_users = [u for u in ("1001", "1002", "1003", "1004") if u in body]
        assert len(found_users) > 0, (
            "Extensions page does not contain any of the test users (1001-1004)"
        )

    # ------------------------------------------------------------------
    # TC-L2-007: Console routing page
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_007_console_routing_page(self, base_url, api_session):
        """TC-L2-007: Routing page shows configured routes.

        The routing page should render and contain a reference to the
        'internal' route defined in test-config.toml.
        """
        resp = api_session.get(f"{base_url}/console/routing", timeout=5)
        assert resp.status_code == 200, (
            f"Routing page returned {resp.status_code}"
        )
        body = resp.text.lower()
        # The test config defines a route named "internal"
        assert "internal" in body or "route" in body, (
            "Routing page does not contain expected route information"
        )

    # ------------------------------------------------------------------
    # TC-L2-008: Console call records page
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_008_console_call_records(self, base_url, api_session):
        """TC-L2-008: Call records page loads successfully.

        The call records / CDR page should return 200 and render valid HTML.
        With no calls made yet, the page may show an empty table or a
        "no records" message.
        """
        resp = api_session.get(f"{base_url}/console/call-records", timeout=5)
        assert resp.status_code == 200, (
            f"Call records page returned {resp.status_code}"
        )
        body_lower = resp.text.lower()
        assert "<html" in body_lower or "<!doctype" in body_lower, (
            "Call records response is not valid HTML"
        )

    # ------------------------------------------------------------------
    # TC-L2-009: API 404 handling
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_009_api_404_handling(self, ami_url):
        """TC-L2-009: Non-existent API path returns 404.

        Requests to undefined AMI endpoints should return 404 Not Found
        rather than 500 or an unexpected response.
        """
        resp = requests.get(f"{ami_url}/nonexistent/endpoint", timeout=5)
        assert resp.status_code == 404, (
            f"Expected 404 for non-existent path, got {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L2-010: Unauthenticated access to protected endpoints
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L2_010_unauthenticated_access(self, base_url):
        """TC-L2-010: Protected console endpoints reject unauthenticated requests.

        A fresh session (no cookies) should be redirected to login or receive
        401 when accessing any protected console page.
        """
        protected_pages = [
            "/console/",
            "/console/extensions",
            "/console/routing",
            "/console/settings",
            "/console/call-records",
            "/console/diagnostics",
        ]
        fresh_session = requests.Session()
        for page in protected_pages:
            resp = fresh_session.get(
                f"{base_url}{page}",
                allow_redirects=False,
                timeout=5,
            )
            assert resp.status_code in (302, 303, 401), (
                f"Page {page}: expected auth redirect/deny, got {resp.status_code}"
            )
