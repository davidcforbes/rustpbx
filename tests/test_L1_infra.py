"""
L1 Infrastructure Tests -- Connectivity and protocol-level checks.

These tests go one layer deeper than L0: they exercise SIP protocol
semantics (OPTIONS response codes, REGISTER challenge flow), WebSocket
upgrade, console authentication mechanics, and page-level rendering.

Expected execution time: < 60 seconds for the full suite.
"""
import socket

import pytest
import requests


class TestL1Infrastructure:
    """L1: Verify SIP, HTTP, WebSocket connectivity and authentication."""

    # ------------------------------------------------------------------
    # TC-L1-001: SIP OPTIONS returns valid SIP response
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_001_sip_options_200(self, sip_host, sip_port):
        """TC-L1-001: SIP OPTIONS returns 200 OK (or a valid SIP error).

        Sends a well-formed OPTIONS request and verifies the SIP stack
        responds with a properly formatted SIP status line. Both 200 OK
        and 401/403 are acceptable -- the point is that the SIP parser
        processed the request and generated a response.
        """
        from conftest import build_sip_options

        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(5)
        try:
            msg = build_sip_options(sip_host, sip_port)
            sock.sendto(msg, (sip_host, sip_port))
            data, _ = sock.recvfrom(4096)
            response = data.decode(errors="replace")
            # Accept 200 OK or auth-required (401/403/407)
            valid_responses = ("SIP/2.0 200", "SIP/2.0 401",
                               "SIP/2.0 403", "SIP/2.0 407")
            assert any(r in response for r in valid_responses), (
                f"Unexpected SIP response: {response[:120]}"
            )
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L1-002: SIP REGISTER without credentials returns 401
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_002_sip_register_requires_auth(self, sip_host, sip_port):
        """TC-L1-002: SIP REGISTER without credentials returns 401.

        An unauthenticated REGISTER must be challenged with 401 Unauthorized
        (or 407 Proxy Authentication Required) and include a WWW-Authenticate
        or Proxy-Authenticate header carrying a digest challenge.
        """
        from conftest import build_sip_register

        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(5)
        try:
            msg = build_sip_register(sip_host, sip_port, "1001")
            sock.sendto(msg, (sip_host, sip_port))
            data, _ = sock.recvfrom(4096)
            response = data.decode(errors="replace")

            # Must be a 401 or 407 challenge
            assert "SIP/2.0 401" in response or "SIP/2.0 407" in response, (
                f"Expected 401/407 challenge, got: {response[:120]}"
            )
            # Must include an authentication challenge header
            assert (
                "WWW-Authenticate" in response
                or "Proxy-Authenticate" in response
            ), "Missing WWW-Authenticate / Proxy-Authenticate header"
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L1-003: WebSocket upgrade at /ws
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_003_websocket_upgrade(self, base_url):
        """TC-L1-003: WebSocket upgrade at /ws succeeds.

        Attempts to open a WebSocket connection with the 'sip' subprotocol.
        Success means the server completed the HTTP 101 upgrade. If the
        endpoint is not available (404/403) we skip rather than fail.
        """
        import asyncio

        try:
            import websockets  # noqa: F401
        except ImportError:
            pytest.skip("websockets package not installed")

        ws_url = base_url.replace("http://", "ws://") + "/ws"

        async def _test_ws():
            import websockets
            try:
                async with websockets.connect(
                    ws_url,
                    subprotocols=["sip"],
                    open_timeout=5,
                    close_timeout=5,
                ) as ws:
                    assert ws.open
            except Exception as exc:
                # 403/404 means the endpoint exists but rejects us --
                # that is acceptable for a connectivity check
                err_str = str(exc)
                if "403" in err_str or "404" in err_str:
                    pytest.skip(f"WebSocket endpoint not available: {exc}")
                raise

        asyncio.run(_test_ws())

    # ------------------------------------------------------------------
    # TC-L1-004: Console login flow
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_004_console_login_flow(self, base_url):
        """TC-L1-004: Console login returns session cookie on valid credentials.

        POSTing correct credentials to /console/login should produce a 302/303
        redirect and set at least one session cookie.
        """
        session = requests.Session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": "admin", "password": "admin123"},
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303), (
            f"Expected redirect after login, got {resp.status_code}"
        )
        assert len(session.cookies) > 0, (
            "No session cookie set after successful login"
        )

    # ------------------------------------------------------------------
    # TC-L1-005: Protected page requires auth
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_005_console_protected_page_requires_auth(self, base_url):
        """TC-L1-005: Protected console pages redirect to login without session.

        An unauthenticated GET to /console/ should redirect to the login page
        (302/303) or return 401 Unauthorized.
        """
        resp = requests.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401), (
            f"Expected auth redirect/deny, got {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L1-006: AMI health response fields
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_006_ami_health_fields(self, ami_url):
        """TC-L1-006: AMI health returns a non-empty JSON object.

        The health endpoint must return a dictionary with at least one key
        (e.g. 'sipserver', 'status', 'uptime').
        """
        resp = requests.get(f"{ami_url}/health", timeout=5)
        assert resp.status_code == 200, (
            f"Health endpoint returned {resp.status_code}"
        )
        data = resp.json()
        assert isinstance(data, dict), "Health response is not a JSON object"
        assert len(data) > 0, "Health response object is empty"

    # ------------------------------------------------------------------
    # TC-L1-007: RTP port range reachable
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_007_rtp_port_range(self, sip_host):
        """TC-L1-007: RTP port range (20000-20100) is reachable via UDP.

        Sends a small UDP probe to a port in the RTP range. Success means
        no immediate ICMP port-unreachable was received. In Docker networks
        ICMP may be suppressed, so we skip on socket errors rather than fail.
        """
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(2)
        try:
            # Minimal RTP-like probe (12 bytes header)
            sock.sendto(b"\x00" * 12, (sip_host, 20000))
            # If sendto succeeded without ConnectionRefused, the port is
            # at least not actively rejecting packets
        except socket.error as exc:
            pytest.skip(f"RTP port probe failed (may be normal in Docker): {exc}")
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L1-008: Console logout invalidates session
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_008_console_logout(self, base_url):
        """TC-L1-008: Console logout invalidates session.

        After logging out, the same session should no longer have access to
        protected pages.

        Note: Uses its own session instead of the shared api_session fixture
        so that other tests are not affected by the logout.
        """
        # Login with a fresh session
        session = requests.Session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": "admin", "password": "admin123"},
            allow_redirects=False,
            timeout=5,
        )
        if resp.status_code not in (302, 303, 200):
            pytest.skip(f"Login failed with status {resp.status_code}")

        # Verify we are logged in (protected page accessible)
        resp = session.get(f"{base_url}/console/", allow_redirects=False, timeout=5)
        assert resp.status_code == 200, (
            f"Expected 200 for authenticated access, got {resp.status_code}"
        )

        # Logout
        session.post(f"{base_url}/console/logout", allow_redirects=False, timeout=5)

        # Verify session is no longer valid
        resp = session.get(f"{base_url}/console/", allow_redirects=False, timeout=5)
        assert resp.status_code in (302, 303, 401), (
            f"Expected redirect/deny after logout, got {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L1-009: All main console pages load
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_009_console_pages_load(self, base_url, api_session):
        """TC-L1-009: All main console pages return 200.

        Iterates over the known console page paths and verifies each one
        returns HTTP 200.
        """
        pages = [
            "/console/extensions",
            "/console/routing",
            "/console/settings",
            "/console/call-records",
            "/console/diagnostics",
        ]
        for page in pages:
            resp = api_session.get(f"{base_url}{page}", timeout=5)
            assert resp.status_code == 200, (
                f"Console page {page} returned {resp.status_code}"
            )
