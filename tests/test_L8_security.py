"""
L8 Security Tests -- Authentication, input validation, and resilience to abuse.

These tests verify that RustPBX correctly rejects unauthenticated or malicious
requests and does not crash or leak information when presented with invalid
input.

Environment variables:
    SIPP_PATH          Path to the sipp binary       (default: sipp)
    RUSTPBX_HOST       Hostname / IP of the PBX      (default: rustpbx)
    RUSTPBX_SIP_PORT   SIP port                      (default: 5060)
    RUSTPBX_HTTP_PORT  HTTP/AMI port                  (default: 8080)
"""

import os
import socket
import subprocess
import time

import pytest
import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SIPP_PATH = os.environ.get("SIPP_PATH", "sipp")
RUSTPBX_HOST = os.environ.get("RUSTPBX_HOST", "rustpbx")
RUSTPBX_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
RUSTPBX_HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8080"))
SIPP_SCENARIOS = os.path.join(os.path.dirname(__file__), "sipp")

BASE_URL = f"http://{RUSTPBX_HOST}:{RUSTPBX_HTTP_PORT}"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def run_sipp(scenario, extra_args=None, timeout=30):
    """Run a SIPp scenario file and return the CompletedProcess."""
    cmd = [
        SIPP_PATH,
        f"{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT}",
        "-sf", os.path.join(SIPP_SCENARIOS, scenario),
        "-m", "1",
        "-timeout", str(timeout),
        "-timeout_error",
        "-max_retrans", "3",
    ]
    if extra_args:
        cmd.extend(extra_args)
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 10)


def _send_raw_udp(data: bytes, timeout=3):
    """Send raw bytes to the SIP UDP port and return any response (or None)."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(timeout)
    try:
        sock.sendto(data, (RUSTPBX_HOST, RUSTPBX_SIP_PORT))
        try:
            resp, _ = sock.recvfrom(4096)
            return resp
        except socket.timeout:
            return None
    finally:
        sock.close()


# ---------------------------------------------------------------------------
# L8 Test Class
# ---------------------------------------------------------------------------


class TestL8Security:
    """L8: Security, authentication, and input-validation tests."""

    # -- SIP auth -----------------------------------------------------------

    @pytest.mark.timeout(30)
    def test_L8_001_register_without_auth_rejected(self):
        """TC-L8-001: REGISTER without credentials is rejected with 401.

        Sends a bare REGISTER (no Authorization header).  The server MUST
        respond with 401 Unauthorized.  A 200 OK would indicate the auth
        module is disabled or misconfigured.
        """
        # Build a raw REGISTER request.
        branch = f"z9hG4bK-sec-{int(time.monotonic_ns())}"
        call_id = f"sec-{int(time.monotonic_ns())}@127.0.0.1"
        register = (
            f"REGISTER sip:{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:9999;branch={branch}\r\n"
            f"From: <sip:1001@{RUSTPBX_HOST}>;tag=sec001\r\n"
            f"To: <sip:1001@{RUSTPBX_HOST}>\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <sip:1001@127.0.0.1:9999>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(register, timeout=5)
        assert resp is not None, "No SIP response received for unauthenticated REGISTER"
        resp_str = resp.decode("utf-8", errors="replace")
        assert "401" in resp_str, (
            f"Expected 401 Unauthorized but got:\n{resp_str[:500]}"
        )

    # -- Console auth -------------------------------------------------------

    @pytest.mark.timeout(15)
    def test_L8_002_console_without_session_rejected(self):
        """TC-L8-002: Accessing /console without a session cookie redirects or returns 401.

        The console is configured at /console with allow_registration=false.
        Without valid auth, the server should redirect to login or return
        401/403.
        """
        resp = requests.get(
            f"{BASE_URL}/console",
            allow_redirects=False,
            timeout=5,
        )
        # Accept redirect (302/303) to login page, or 401/403 forbidden.
        assert resp.status_code in (301, 302, 303, 307, 401, 403), (
            f"Console returned {resp.status_code} without auth -- "
            f"expected redirect or 401/403"
        )

    # -- Malformed SIP ------------------------------------------------------

    @pytest.mark.timeout(30)
    def test_L8_003_malformed_sip_no_crash(self):
        """TC-L8-003: Sending garbage bytes to the SIP port does not crash the PBX.

        Sends several payloads of random/malformed data and then verifies the
        HTTP health endpoint is still responsive.
        """
        garbage_payloads = [
            b"\x00" * 1024,
            b"\xff" * 512,
            b"NOT A SIP MESSAGE\r\n\r\n",
            b"INVITE\r\n\r\n",
            b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
            b"SIP/2.0 200 OK\r\n\r\n",  # response sent as if we are a server
            b"\x80\x00\x00\x01" + b"\xde\xad\xbe\xef" * 100,
            # Oversized Via header.
            (b"OPTIONS sip:x SIP/2.0\r\n"
             b"Via: " + b"A" * 4000 + b"\r\n"
             b"Content-Length: 0\r\n\r\n"),
        ]

        for payload in garbage_payloads:
            _send_raw_udp(payload, timeout=1)

        # Give the PBX a moment to process the garbage.
        time.sleep(1)

        # Health check -- the PBX must still be alive.
        resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
        assert resp.status_code == 200, (
            f"PBX health check failed after malformed SIP input "
            f"(HTTP {resp.status_code})"
        )

    # -- SQL injection in login ---------------------------------------------

    @pytest.mark.timeout(15)
    def test_L8_004_sql_injection_in_login(self):
        """TC-L8-004: SQL injection payloads in login form do not bypass auth.

        Sends common SQL injection strings as username/password to the console
        login endpoint.  A successful bypass would return 200 or a redirect to
        a protected page; the correct behaviour is 400, 401, or 403.
        """
        sqli_payloads = [
            {"username": "' OR '1'='1' --", "password": "anything"},
            {"username": "admin", "password": "' OR '1'='1' --"},
            {"username": "1001'; DROP TABLE users; --", "password": "x"},
            {"username": "' UNION SELECT 1,2,3 --", "password": "x"},
        ]

        for payload in sqli_payloads:
            resp = requests.post(
                f"{BASE_URL}/console/login",
                data=payload,
                allow_redirects=False,
                timeout=5,
            )
            # Should NOT get a 200 with a valid session or a redirect to /console.
            if resp.status_code in (301, 302, 303, 307):
                location = resp.headers.get("Location", "")
                # Redirect to login page is fine; redirect to console is not.
                assert "/login" in location or "/console" not in location or "login" in location, (
                    f"Possible SQL injection bypass: redirected to {location} "
                    f"with payload {payload}"
                )
            else:
                # 400, 401, 403, 404, 422 are all acceptable rejections.
                assert resp.status_code in (400, 401, 403, 404, 422), (
                    f"Unexpected status {resp.status_code} for SQLi payload {payload}"
                )

    # -- Path traversal -----------------------------------------------------

    @pytest.mark.timeout(15)
    def test_L8_005_path_traversal_blocked(self):
        """TC-L8-005: Path traversal sequences in URLs are rejected.

        Attempts various path traversal attacks to read sensitive files.  The
        server must return 400 or 404, never the file contents.
        """
        traversal_paths = [
            "/../../../etc/passwd",
            "/console/../../../etc/passwd",
            "/%2e%2e/%2e%2e/%2e%2e/etc/passwd",
            "/..%252f..%252f..%252fetc/passwd",
            "/console/..%5c..%5c..%5cetc/passwd",
            "/static/../../../config.toml",
        ]

        for path in traversal_paths:
            resp = requests.get(
                f"{BASE_URL}{path}",
                allow_redirects=False,
                timeout=5,
            )
            body = resp.text.lower()
            # Must not contain Unix password file markers or TOML config.
            assert "root:" not in body, (
                f"Path traversal exposed /etc/passwd via {path}"
            )
            assert "database_url" not in body, (
                f"Path traversal exposed config file via {path}"
            )
            # Accept 400, 403, or 404.
            assert resp.status_code in (301, 302, 400, 403, 404), (
                f"Unexpected status {resp.status_code} for traversal path {path}"
            )

    # -- SIP invite without registration ------------------------------------

    @pytest.mark.timeout(30)
    def test_L8_006_invite_from_unregistered_rejected(self):
        """TC-L8-006: INVITE from an unregistered user is rejected.

        Sends an INVITE for a non-existent caller.  The PBX should respond
        with 401, 403, or 404 -- never route the call.
        """
        branch = f"z9hG4bK-sec-inv-{int(time.monotonic_ns())}"
        call_id = f"sec-inv-{int(time.monotonic_ns())}@127.0.0.1"
        invite = (
            f"INVITE sip:1001@{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:9999;branch={branch}\r\n"
            f"From: <sip:attacker@evil.com>;tag=attack1\r\n"
            f"To: <sip:1001@{RUSTPBX_HOST}>\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: 1 INVITE\r\n"
            f"Contact: <sip:attacker@127.0.0.1:9999>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(invite, timeout=5)
        if resp is not None:
            resp_str = resp.decode("utf-8", errors="replace")
            # Should be 401, 403, 404, or 407 -- not 100/180/200.
            first_line = resp_str.split("\r\n", 1)[0]
            assert any(code in first_line for code in ("401", "403", "404", "407", "480", "488")), (
                f"INVITE from unregistered user was not rejected: {first_line}"
            )

    # -- Oversized headers --------------------------------------------------

    @pytest.mark.timeout(15)
    def test_L8_007_oversized_header_no_crash(self):
        """TC-L8-007: SIP message with extremely long headers does not crash PBX."""
        branch = f"z9hG4bK-big-{int(time.monotonic_ns())}"
        call_id = f"big-{int(time.monotonic_ns())}@127.0.0.1"
        # Create an OPTIONS with a 10 KB Subject header.
        big_header = "X" * 10_000
        options = (
            f"OPTIONS sip:{RUSTPBX_HOST} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:9999;branch={branch}\r\n"
            f"From: <sip:test@127.0.0.1>;tag=big1\r\n"
            f"To: <sip:{RUSTPBX_HOST}>\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: 1 OPTIONS\r\n"
            f"Subject: {big_header}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        _send_raw_udp(options, timeout=2)
        time.sleep(1)

        # PBX must still be alive.
        resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
        assert resp.status_code == 200, (
            "PBX crashed or became unresponsive after oversized SIP header"
        )

    # -- HTTP method fuzzing ------------------------------------------------

    @pytest.mark.timeout(15)
    def test_L8_008_unexpected_http_methods(self):
        """TC-L8-008: Unusual HTTP methods on API endpoints return proper errors."""
        methods_to_test = ["DELETE", "PATCH", "PUT", "TRACE", "OPTIONS"]

        for method in methods_to_test:
            resp = requests.request(
                method,
                f"{BASE_URL}/ami/v1/health",
                timeout=5,
            )
            # Should not return 500 (server error).  405, 404, 200 are all fine.
            assert resp.status_code != 500, (
                f"HTTP {method} on /ami/v1/health returned 500 Internal Server Error"
            )
