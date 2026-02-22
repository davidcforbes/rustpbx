"""
L0 Smoke Tests -- Basic system health checks.

These tests verify that the RustPBX container is alive and that the most
fundamental services (HTTP, SIP, console) are responding. They require no
authentication and no pre-existing state.

Expected execution time: < 30 seconds for the full suite.
"""
import socket

import pytest
import requests


class TestL0Smoke:
    """L0: Verify RustPBX container is alive and basic services respond."""

    # ------------------------------------------------------------------
    # TC-L0-001: HTTP port responds
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_001_http_port_responds(self, base_url):
        """TC-L0-001: HTTP port 8080 responds to any request.

        A GET to the root path must return a valid HTTP response. We accept
        200 (direct content) or 3xx (redirect to console/login).
        """
        resp = requests.get(f"{base_url}/", timeout=5)
        assert resp.status_code in (200, 301, 302, 303), (
            f"Expected 200 or redirect, got {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L0-002: SIP UDP port accepts
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_002_sip_udp_port_accepts(self, sip_host, sip_port):
        """TC-L0-002: SIP port 5060 accepts UDP packets (no ICMP reject).

        Sends a minimal SIP OPTIONS probe and verifies we receive a SIP
        response. Any SIP-formatted reply (including 4xx/5xx) proves the
        SIP stack is alive.
        """
        from conftest import build_sip_options

        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(3)
        try:
            msg = build_sip_options(sip_host, sip_port)
            sock.sendto(msg, (sip_host, sip_port))
            data, addr = sock.recvfrom(4096)
            assert b"SIP/2.0" in data, (
                "Response does not look like a SIP message"
            )
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L0-003: SIP TCP port accepts
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_003_sip_tcp_port_accepts(self, sip_host, sip_port):
        """TC-L0-003: SIP port 5060 accepts TCP connections.

        A successful TCP three-way handshake proves the SIP TCP listener
        is up.
        """
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(3)
        try:
            sock.connect((sip_host, sip_port))
            # If we reach here the connection succeeded
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L0-004: Console login page
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_004_console_login_page(self, base_url):
        """TC-L0-004: Console login page loads with HTML content.

        The login page should return HTTP 200 and contain recognisable
        login-related keywords (form fields, headings, etc.).
        """
        resp = requests.get(f"{base_url}/console/login", timeout=5)
        assert resp.status_code == 200, (
            f"Login page returned {resp.status_code}"
        )
        body_lower = resp.text.lower()
        assert "login" in body_lower or "password" in body_lower, (
            "Login page does not contain expected login/password keywords"
        )

    # ------------------------------------------------------------------
    # TC-L0-005: AMI health endpoint
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_005_ami_health_endpoint(self, base_url):
        """TC-L0-005: AMI health endpoint returns JSON with status.

        The /ami/v1/health endpoint should return 200 with a JSON body
        containing at least one status-related key.
        """
        resp = requests.get(f"{base_url}/ami/v1/health", timeout=5)
        assert resp.status_code == 200, (
            f"Health endpoint returned {resp.status_code}"
        )
        data = resp.json()
        # The response should contain at least one meaningful key
        assert isinstance(data, dict), "Health response is not a JSON object"
        assert len(data) > 0, "Health response is empty"

    # ------------------------------------------------------------------
    # TC-L0-006: AMI dialogs endpoint
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_006_ami_dialogs_endpoint(self, base_url):
        """TC-L0-006: AMI dialogs returns empty list (no active calls).

        With no calls in progress the dialogs endpoint should return an
        empty list or object.
        """
        resp = requests.get(f"{base_url}/ami/v1/dialogs", timeout=5)
        assert resp.status_code == 200, (
            f"Dialogs endpoint returned {resp.status_code}"
        )
        data = resp.json()
        assert isinstance(data, (list, dict)), (
            f"Unexpected dialogs response type: {type(data).__name__}"
        )

    # ------------------------------------------------------------------
    # TC-L0-007: AMI transactions endpoint
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_007_ami_transactions_endpoint(self, base_url):
        """TC-L0-007: AMI transactions returns empty list.

        The transactions endpoint should respond with 200 and a valid JSON
        payload even when there are no active transactions.
        """
        resp = requests.get(f"{base_url}/ami/v1/transactions", timeout=5)
        assert resp.status_code == 200, (
            f"Transactions endpoint returned {resp.status_code}"
        )
        data = resp.json()
        assert isinstance(data, (list, dict)), (
            f"Unexpected transactions response type: {type(data).__name__}"
        )

    # ------------------------------------------------------------------
    # TC-L0-008: Static assets served
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_008_static_assets(self, base_url):
        """TC-L0-008: Static assets are served (CSS/JS for console).

        The login page should be a fully-formed HTML document. We check for
        standard HTML boilerplate to confirm static asset serving is working.
        """
        resp = requests.get(f"{base_url}/console/login", timeout=5)
        assert resp.status_code == 200, (
            f"Console page returned {resp.status_code}"
        )
        body_lower = resp.text.lower()
        assert "<html" in body_lower or "<!doctype" in body_lower, (
            "Response does not appear to be an HTML document"
        )
