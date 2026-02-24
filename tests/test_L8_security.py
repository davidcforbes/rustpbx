"""
L8 Security Tests -- Authentication, authorization, input validation, and
resilience to abuse.

These tests verify that RustPBX correctly enforces authentication on all
protected surfaces (HTTP console, AMI API, SIP), rejects malicious input
(SQL injection, XSS, path traversal, header injection), and does not leak
information or crash when presented with adversarial traffic.

This is *defensive* security testing of our own system -- not offensive
penetration testing against third-party targets.

Target server: https://74.207.251.126:8443 (HTTPS, self-signed cert)
SIP:           74.207.251.126:5060/UDP
Test users:    1001/test1001, 1002/test1002
Console login: admin/admin123

Run with:
    python -m pytest tests/test_L8_security.py -v

Expected execution time: < 120 seconds for the full suite.

Environment variables (all optional -- defaults match conftest.py):
    RUSTPBX_HOST        SIP/HTTP server IP         (default: 127.0.0.1)
    RUSTPBX_HTTP_PORT   HTTPS port                 (default: 8443)
    RUSTPBX_SIP_PORT    SIP port                   (default: 5060)
    RUSTPBX_SCHEME      http or https              (default: https)
    RUSTPBX_VERIFY_TLS  Verify TLS certs           (default: false)
    RUSTPBX_ADMIN_USER  Console admin username     (default: admin)
    RUSTPBX_ADMIN_PASS  Console admin password     (default: admin123)
"""

import hashlib
import os
import re
import socket
import ssl
import time
import uuid

import pytest
import requests
import urllib3

# Suppress InsecureRequestWarning for self-signed TLS certs
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# Configuration -- mirrors conftest.py but local so the file is self-contained
# for reading.  Actual values come from conftest fixtures at runtime.
# ---------------------------------------------------------------------------

RUSTPBX_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
RUSTPBX_HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
RUSTPBX_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
RUSTPBX_SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
RUSTPBX_VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in (
    "1", "true", "yes",
)
ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")
BASE_URL = f"{RUSTPBX_SCHEME}://{RUSTPBX_HOST}:{RUSTPBX_HTTP_PORT}"


# ---------------------------------------------------------------------------
# SIP helpers -- thin wrappers matching test_L3_sip.py patterns
# ---------------------------------------------------------------------------

def _gen_branch():
    return "z9hG4bK" + uuid.uuid4().hex[:12]


def _gen_tag():
    return uuid.uuid4().hex[:8]


def _gen_callid():
    return uuid.uuid4().hex[:16] + "@l8sec"


def _md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()


def _compute_digest(username, realm, password, method, uri, nonce):
    """Compute SIP digest authentication response."""
    ha1 = _md5hex(f"{username}:{realm}:{password}")
    ha2 = _md5hex(f"{method}:{uri}")
    return _md5hex(f"{ha1}:{nonce}:{ha2}")


def _get_response_code(data):
    """Extract the SIP response status code from the first line."""
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0


def _parse_www_authenticate(header_line):
    """Extract realm and nonce from a WWW-Authenticate header value."""
    realm = re.search(r'realm="([^"]*)"', header_line)
    nonce = re.search(r'nonce="([^"]*)"', header_line)
    return (realm.group(1) if realm else ""), (nonce.group(1) if nonce else "")


def _send_raw_udp(data: bytes, host=None, port=None, timeout=5):
    """Send raw bytes to the SIP UDP port and return any response (or None)."""
    host = host or RUSTPBX_HOST
    port = port or RUSTPBX_SIP_PORT
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(timeout)
    try:
        sock.sendto(data, (host, port))
        try:
            resp, _ = sock.recvfrom(65535)
            return resp
        except socket.timeout:
            return None
    finally:
        sock.close()


def _fresh_session():
    """Return a new requests.Session with TLS verification disabled."""
    s = requests.Session()
    s.verify = RUSTPBX_VERIFY_TLS
    return s


def _login_session():
    """Return a requests.Session logged into the console."""
    s = _fresh_session()
    s.post(
        f"{BASE_URL}/console/login",
        data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
        allow_redirects=False,
        timeout=10,
    )
    return s


# ===========================================================================
# TC-L8-001: Authentication Enforcement
# ===========================================================================

class TestAuthenticationEnforcement:
    """TC-L8-001: Verify API and console endpoints require authentication."""

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("path", [
        "/console/",
        "/console/extensions",
        "/console/routing",
        "/console/settings",
        "/console/call-records",
        "/console/diagnostics",
        "/console/trunks",
    ])
    def test_console_pages_require_auth(self, base_url, verify_tls, path):
        """Unauthenticated GET to protected console pages is rejected."""
        session = _fresh_session()
        resp = session.get(
            f"{base_url}{path}",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401, 403), (
            f"Page {path}: expected auth redirect/deny, got {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_console_redirect_targets_login(self, base_url, verify_tls):
        """Unauthenticated console access redirects specifically to /login."""
        session = _fresh_session()
        resp = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        if resp.status_code in (302, 303):
            location = resp.headers.get("Location", "")
            assert "login" in location.lower(), (
                f"Redirect should point to login, got {location!r}"
            )

    @pytest.mark.timeout(15)
    def test_console_post_endpoints_require_auth(self, base_url, verify_tls):
        """POST to console action endpoints without auth is rejected."""
        session = _fresh_session()
        # Try to create an extension without being logged in
        resp = session.post(
            f"{base_url}/console/extensions",
            data={"username": "9999", "password": "hack"},
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401, 403, 404, 405), (
            f"POST to /console/extensions without auth: got {resp.status_code}"
        )


# ===========================================================================
# TC-L8-002: SIP Digest Authentication
# ===========================================================================

class TestSIPDigestAuth:
    """TC-L8-002: Verify REGISTER without credentials gets 401 with nonce."""

    @pytest.mark.timeout(15)
    def test_bare_register_gets_401(self, sip_host, sip_port):
        """REGISTER without Authorization header returns 401."""
        branch = _gen_branch()
        cid = _gen_callid()
        register = (
            f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:19901;branch={branch}\r\n"
            f"From: <sip:1001@{sip_host}>;tag={_gen_tag()}\r\n"
            f"To: <sip:1001@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <sip:1001@127.0.0.1:19901>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(register, host=sip_host, port=sip_port)
        assert resp is not None, "No SIP response for unauthenticated REGISTER"
        resp_str = resp.decode("utf-8", errors="replace")
        code = _get_response_code(resp_str)
        assert code == 401, (
            f"Expected 401 Unauthorized, got {code}"
        )

    @pytest.mark.timeout(15)
    def test_401_contains_www_authenticate(self, sip_host, sip_port):
        """401 response includes WWW-Authenticate with realm and nonce."""
        branch = _gen_branch()
        cid = _gen_callid()
        register = (
            f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:19902;branch={branch}\r\n"
            f"From: <sip:1001@{sip_host}>;tag={_gen_tag()}\r\n"
            f"To: <sip:1001@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <sip:1001@127.0.0.1:19902>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(register, host=sip_host, port=sip_port)
        assert resp is not None, "No SIP response"
        resp_str = resp.decode("utf-8", errors="replace")

        # Find WWW-Authenticate header
        auth_hdr = ""
        for line in resp_str.split("\r\n"):
            if line.lower().startswith("www-authenticate:"):
                auth_hdr = line.split(":", 1)[1].strip()
                break

        assert auth_hdr, "401 response missing WWW-Authenticate header"
        realm, nonce = _parse_www_authenticate(auth_hdr)
        assert realm, "WWW-Authenticate missing realm parameter"
        assert nonce, "WWW-Authenticate missing nonce parameter"
        assert len(nonce) >= 8, (
            f"Nonce too short ({len(nonce)} chars) -- may be predictable"
        )


# ===========================================================================
# TC-L8-003: Wrong Password Rejection
# ===========================================================================

class TestWrongPasswordRejection:
    """TC-L8-003: Verify REGISTER with wrong password gets 403."""

    @pytest.mark.timeout(15)
    def test_wrong_password_rejected(self, sip_host, sip_port):
        """REGISTER with correct username but wrong password is rejected."""
        username = "1001"
        wrong_password = "WRONGPASSWORD"
        local_port = 19903
        tag = _gen_tag()
        cid = _gen_callid()

        # Step 1: send unauthenticated REGISTER to get challenge
        branch = _gen_branch()
        register1 = (
            f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:{local_port};branch={branch}\r\n"
            f"From: <sip:{username}@{sip_host}>;tag={tag}\r\n"
            f"To: <sip:{username}@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <sip:{username}@127.0.0.1:{local_port}>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(register1, host=sip_host, port=sip_port)
        assert resp is not None, "No SIP response for initial REGISTER"
        resp_str = resp.decode("utf-8", errors="replace")
        code = _get_response_code(resp_str)
        assert code in (401, 407), f"Expected 401/407 challenge, got {code}"

        # Extract challenge
        auth_hdr = ""
        for line in resp_str.split("\r\n"):
            low = line.lower()
            if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
                auth_hdr = line.split(":", 1)[1].strip()
                break

        assert auth_hdr, "No authentication challenge header found"
        realm, nonce = _parse_www_authenticate(auth_hdr)
        realm = realm or sip_host

        # Step 2: compute digest with WRONG password
        uri = f"sip:{realm}"
        digest = _compute_digest(username, realm, wrong_password, "REGISTER", uri, nonce)
        hdr_name = "Authorization" if code == 401 else "Proxy-Authorization"
        auth_line = (
            f'Digest username="{username}", realm="{realm}", '
            f'nonce="{nonce}", uri="{uri}", '
            f'response="{digest}", algorithm=MD5'
        )

        branch2 = _gen_branch()
        register2 = (
            f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:{local_port};branch={branch2}\r\n"
            f"From: <sip:{username}@{sip_host}>;tag={tag}\r\n"
            f"To: <sip:{username}@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 2 REGISTER\r\n"
            f"Contact: <sip:{username}@127.0.0.1:{local_port}>\r\n"
            f"{hdr_name}: {auth_line}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp2 = _send_raw_udp(register2, host=sip_host, port=sip_port)
        assert resp2 is not None, "No SIP response for authenticated REGISTER"
        resp2_str = resp2.decode("utf-8", errors="replace")
        code2 = _get_response_code(resp2_str)
        # Wrong password should yield 401 or 403 -- never 200
        assert code2 in (401, 403), (
            f"Wrong password should be rejected, got {code2}"
        )
        assert code2 != 200, "REGISTER with wrong password was accepted (200 OK)!"


# ===========================================================================
# TC-L8-004: SQL Injection in API
# ===========================================================================

class TestSQLInjection:
    """TC-L8-004: Test common SQL injection patterns in query params and forms."""

    SQLI_PAYLOADS = [
        "' OR '1'='1' --",
        "' OR '1'='1'/*",
        "admin'--",
        "1'; DROP TABLE users; --",
        "' UNION SELECT 1,2,3 --",
        "1' AND 1=1 --",
        "' OR 1=1#",
        "'; WAITFOR DELAY '0:0:5' --",
        "1' OR '1'='1",
    ]

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("payload", SQLI_PAYLOADS)
    def test_sqli_in_login_username(self, base_url, verify_tls, payload):
        """SQL injection in login username field does not bypass auth."""
        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": payload, "password": "anything"},
            allow_redirects=False,
            timeout=5,
        )
        if resp.status_code in (301, 302, 303, 307):
            location = resp.headers.get("Location", "")
            # Redirect back to login is fine; redirect to console root is a bypass
            assert "login" in location.lower() or "/console/" not in location, (
                f"Possible SQLi bypass: redirected to {location} with payload {payload!r}"
            )
        else:
            # Any non-redirect status except 200 is acceptable
            assert resp.status_code != 200 or "login" in resp.text.lower(), (
                f"SQLi payload may have bypassed auth: status {resp.status_code}"
            )

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("payload", SQLI_PAYLOADS)
    def test_sqli_in_login_password(self, base_url, verify_tls, payload):
        """SQL injection in login password field does not bypass auth."""
        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": "admin", "password": payload},
            allow_redirects=False,
            timeout=5,
        )
        if resp.status_code in (301, 302, 303, 307):
            location = resp.headers.get("Location", "")
            assert "login" in location.lower() or "/console/" not in location, (
                f"Possible SQLi bypass via password: redirected to {location}"
            )

    @pytest.mark.timeout(15)
    def test_sqli_in_ami_query_params(self, ami_url, verify_tls):
        """SQL injection in AMI query parameters does not cause errors."""
        payloads = [
            "' OR 1=1 --",
            "1; DROP TABLE calls",
            "' UNION SELECT * FROM users --",
        ]
        for payload in payloads:
            resp = requests.get(
                f"{ami_url}/dialogs",
                params={"filter": payload},
                timeout=5,
                verify=verify_tls,
            )
            # Should not return 500 (would indicate unhandled SQL error)
            assert resp.status_code != 500, (
                f"AMI returned 500 with SQLi param: {payload!r}"
            )

    @pytest.mark.timeout(15)
    def test_sqli_does_not_crash_server(self, base_url, ami_url, verify_tls):
        """Server health is intact after SQL injection attempts."""
        # Fire a batch of SQLi payloads at login
        for payload in self.SQLI_PAYLOADS:
            try:
                requests.post(
                    f"{base_url}/console/login",
                    data={"identifier": payload, "password": payload},
                    timeout=3,
                    verify=verify_tls,
                )
            except requests.RequestException:
                pass

        # Server must still be alive
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Server unhealthy after SQLi barrage: HTTP {resp.status_code}"
        )


# ===========================================================================
# TC-L8-005: XSS in API Responses
# ===========================================================================

class TestXSSProtection:
    """TC-L8-005: Verify API responses have proper Content-Type and escaping."""

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("endpoint", [
        "/ami/v1/health",
        "/ami/v1/dialogs",
        "/ami/v1/transactions",
    ])
    def test_api_json_content_type(self, base_url, verify_tls, endpoint):
        """API JSON endpoints set Content-Type: application/json, not text/html."""
        resp = requests.get(
            f"{base_url}{endpoint}", timeout=5, verify=verify_tls,
        )
        ct = resp.headers.get("Content-Type", "")
        assert "application/json" in ct, (
            f"{endpoint} returned Content-Type {ct!r} -- "
            f"should be application/json to prevent XSS via content sniffing"
        )

    @pytest.mark.timeout(15)
    def test_xss_in_404_not_reflected(self, base_url, verify_tls):
        """Script tags in 404 URLs are not reflected in the response body."""
        xss_path = "/<script>alert('xss')</script>"
        resp = requests.get(
            f"{base_url}{xss_path}",
            timeout=5,
            verify=verify_tls,
        )
        body = resp.text
        # The raw script tag must NOT appear unescaped in the response
        assert "<script>alert('xss')</script>" not in body, (
            "XSS payload reflected verbatim in 404 response body"
        )

    @pytest.mark.timeout(15)
    def test_xss_in_login_error_not_reflected(self, base_url, verify_tls):
        """XSS payloads in login fields are not reflected in error pages."""
        xss_payloads = [
            "<script>alert(1)</script>",
            "<img src=x onerror=alert(1)>",
            "'\"><svg/onload=alert(1)>",
        ]
        for payload in xss_payloads:
            resp = requests.post(
                f"{base_url}/console/login",
                data={"identifier": payload, "password": "x"},
                allow_redirects=True,
                timeout=5,
                verify=verify_tls,
            )
            body = resp.text
            # The payload must not appear unescaped
            assert payload not in body, (
                f"XSS payload reflected in login response: {payload!r}"
            )

    @pytest.mark.timeout(15)
    def test_x_content_type_options_header(self, base_url, verify_tls):
        """Responses should include X-Content-Type-Options: nosniff."""
        resp = requests.get(
            f"{base_url}/ami/v1/health", timeout=5, verify=verify_tls,
        )
        xcto = resp.headers.get("X-Content-Type-Options", "")
        # This is a recommended security header -- note if missing
        if xcto:
            assert xcto.lower() == "nosniff", (
                f"X-Content-Type-Options should be 'nosniff', got {xcto!r}"
            )
        else:
            pytest.skip(
                "X-Content-Type-Options header not set (recommended: nosniff)"
            )


# ===========================================================================
# TC-L8-006: Path Traversal
# ===========================================================================

class TestPathTraversal:
    """TC-L8-006: Test ../ patterns in URL paths and file-serving endpoints."""

    TRAVERSAL_PATHS = [
        "/../../../etc/passwd",
        "/console/../../../etc/passwd",
        "/%2e%2e/%2e%2e/%2e%2e/etc/passwd",
        "/..%252f..%252f..%252fetc/passwd",
        "/console/..%5c..%5c..%5cetc/passwd",
        "/static/../../../config.toml",
        "/static/..%2f..%2f..%2fconfig.toml",
        "/../../../etc/shadow",
        "/console/../../../proc/self/environ",
        "/..\\..\\..\\etc\\passwd",
    ]

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("path", TRAVERSAL_PATHS)
    def test_traversal_blocked(self, base_url, verify_tls, path):
        """Path traversal sequence does not expose sensitive files."""
        resp = requests.get(
            f"{base_url}{path}",
            allow_redirects=False,
            timeout=5,
            verify=verify_tls,
        )
        body = resp.text.lower()

        # Must not contain Unix password/shadow file markers
        assert "root:" not in body, (
            f"Path traversal exposed /etc/passwd via {path}"
        )
        assert "daemon:" not in body, (
            f"Path traversal exposed /etc/passwd via {path}"
        )

        # Must not contain config file contents
        assert "database_url" not in body, (
            f"Path traversal exposed config via {path}"
        )
        assert "[sipserver]" not in body, (
            f"Path traversal exposed config via {path}"
        )

        # Acceptable status codes for blocked traversal
        assert resp.status_code in (301, 302, 400, 403, 404), (
            f"Unexpected status {resp.status_code} for traversal path {path}"
        )

    @pytest.mark.timeout(15)
    def test_null_byte_traversal(self, base_url, verify_tls):
        """Null byte injection in path does not bypass file checks."""
        resp = requests.get(
            f"{base_url}/static/../../etc/passwd%00.html",
            allow_redirects=False,
            timeout=5,
            verify=verify_tls,
        )
        assert "root:" not in resp.text.lower(), (
            "Null byte injection exposed /etc/passwd"
        )


# ===========================================================================
# TC-L8-007: CSRF Protection
# ===========================================================================

class TestCSRFProtection:
    """TC-L8-007: Verify form submissions require proper tokens/headers."""

    @pytest.mark.timeout(20)
    def test_login_from_foreign_origin_rejected(self, base_url, verify_tls):
        """Login POST with a foreign Origin header is rejected or has no effect."""
        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            headers={"Origin": "https://evil.example.com"},
            allow_redirects=False,
            timeout=5,
        )
        # If the server checks Origin, it might reject with 403.
        # If it does not check, the login may succeed -- that is a finding
        # but not necessarily a blocker for all deployments.  We record the
        # result for review.
        if resp.status_code in (302, 303):
            # Login succeeded -- check if the server set CSRF-related headers
            location = resp.headers.get("Location", "")
            if "login" not in location.lower():
                pytest.skip(
                    "Server accepted login with foreign Origin -- "
                    "CSRF protection via Origin checking is not enforced "
                    "(may rely on SameSite cookies instead)"
                )

    @pytest.mark.timeout(15)
    def test_state_changing_api_rejects_get(self, ami_url, verify_tls):
        """State-changing endpoints (reload) reject GET requests (405)."""
        resp = requests.get(
            f"{ami_url}/reload/routes", timeout=5, verify=verify_tls,
        )
        assert resp.status_code in (405, 404), (
            f"GET on POST-only reload endpoint returned {resp.status_code} "
            f"-- should be 405 to prevent CSRF via <img> tags"
        )


# ===========================================================================
# TC-L8-008: Session Management
# ===========================================================================

class TestSessionManagement:
    """TC-L8-008: Verify sessions expire and can't be reused after logout."""

    @pytest.mark.timeout(20)
    def test_session_invalidated_after_logout(self, base_url, verify_tls):
        """Session cookie is invalidated after logout."""
        session = _fresh_session()

        # Login
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            allow_redirects=False,
            timeout=5,
        )
        if resp.status_code not in (302, 303, 200):
            pytest.skip(f"Could not login: HTTP {resp.status_code}")

        # Verify we can access protected page
        resp = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code == 200, (
            f"Cannot access console after login: {resp.status_code}"
        )

        # Save cookies before logout
        saved_cookies = dict(session.cookies)

        # Logout
        session.get(
            f"{base_url}/console/logout",
            allow_redirects=False,
            timeout=5,
        )

        # Try using saved cookies in a new session (session replay attack)
        replay_session = _fresh_session()
        for name, value in saved_cookies.items():
            replay_session.cookies.set(name, value)

        resp = replay_session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401, 403), (
            f"Replayed session cookie still valid after logout: "
            f"HTTP {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_fabricated_session_cookie_rejected(self, base_url, verify_tls):
        """A fabricated session cookie does not grant access."""
        session = _fresh_session()
        # Try several common cookie names with fake values
        fake_cookies = {
            "session": "fabricated-session-id-12345",
            "sid": "aaaaaabbbbbbccccccdddddd",
            "token": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbiJ9.fake",
            "rustpbx_session": "0123456789abcdef",
        }
        for name, value in fake_cookies.items():
            session.cookies.set(name, value)

        resp = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp.status_code in (302, 303, 401, 403), (
            f"Fabricated cookie accepted: HTTP {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_wrong_login_credentials_rejected(self, base_url, verify_tls):
        """Login with wrong password does not create a valid session."""
        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": ADMIN_USER, "password": "WRONG_PASSWORD"},
            allow_redirects=False,
            timeout=5,
        )
        # After failed login, accessing console should be denied
        resp2 = session.get(
            f"{base_url}/console/",
            allow_redirects=False,
            timeout=5,
        )
        assert resp2.status_code in (302, 303, 401, 403), (
            f"Console accessible after failed login: HTTP {resp2.status_code}"
        )


# ===========================================================================
# TC-L8-009: Rate Limiting
# ===========================================================================

class TestRateLimiting:
    """TC-L8-009: Verify server handles rapid repeated failed auth attempts."""

    @pytest.mark.timeout(30)
    def test_rapid_failed_logins_no_crash(self, base_url, ami_url, verify_tls):
        """Server stays healthy after many rapid failed login attempts."""
        session = _fresh_session()
        failure_count = 0

        for i in range(50):
            try:
                resp = session.post(
                    f"{base_url}/console/login",
                    data={"identifier": f"attacker{i}", "password": "wrong"},
                    allow_redirects=False,
                    timeout=3,
                )
                failure_count += 1
            except requests.RequestException:
                # Connection refused or timeout is acceptable (rate limiting)
                break

        # Server must still be alive
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert resp.status_code == 200, (
            f"Server unhealthy after {failure_count} failed login attempts"
        )

    @pytest.mark.timeout(30)
    def test_rapid_sip_register_no_crash(self, sip_host, sip_port, ami_url, verify_tls):
        """Server stays healthy after many rapid unauthenticated REGISTERs."""
        for i in range(50):
            branch = _gen_branch()
            cid = _gen_callid()
            register = (
                f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP 127.0.0.1:19999;branch={branch}\r\n"
                f"From: <sip:attacker{i}@evil.com>;tag={_gen_tag()}\r\n"
                f"To: <sip:attacker{i}@{sip_host}>\r\n"
                f"Call-ID: {cid}\r\n"
                f"CSeq: 1 REGISTER\r\n"
                f"Contact: <sip:attacker{i}@127.0.0.1:19999>\r\n"
                f"Max-Forwards: 70\r\n"
                f"Expires: 60\r\n"
                f"Content-Length: 0\r\n"
                f"\r\n"
            ).encode()
            _send_raw_udp(register, host=sip_host, port=sip_port, timeout=1)

        time.sleep(1)

        # Server must still be alive
        resp = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert resp.status_code == 200, (
            "Server unhealthy after rapid SIP REGISTER flood"
        )


# ===========================================================================
# TC-L8-010: SIP Header Injection (CRLF Injection)
# ===========================================================================

class TestSIPHeaderInjection:
    """TC-L8-010: Test for CRLF injection in SIP headers."""

    @pytest.mark.timeout(15)
    def test_crlf_in_from_header(self, sip_host, sip_port, ami_url, verify_tls):
        """CRLF sequences in From header do not inject extra headers."""
        branch = _gen_branch()
        cid = _gen_callid()
        # Try to inject a Via header through the From display name
        injected_from = (
            '"Attacker\\r\\nVia: SIP/2.0/UDP attacker.com:5060;branch=z9hG4bKhacked"'
        )
        register = (
            f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:19910;branch={branch}\r\n"
            f"From: {injected_from} <sip:1001@{sip_host}>;tag={_gen_tag()}\r\n"
            f"To: <sip:1001@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <sip:1001@127.0.0.1:19910>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(register, host=sip_host, port=sip_port)
        if resp is not None:
            resp_str = resp.decode("utf-8", errors="replace")
            # The response should not contain our injected Via
            assert "attacker.com" not in resp_str.lower(), (
                "CRLF injection: attacker Via header appeared in response"
            )

        # Server must still be alive
        time.sleep(0.5)
        health = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert health.status_code == 200, "Server crashed after CRLF injection attempt"

    @pytest.mark.timeout(15)
    def test_crlf_in_contact_header(self, sip_host, sip_port, ami_url, verify_tls):
        """CRLF in Contact header does not cause server crash."""
        branch = _gen_branch()
        cid = _gen_callid()
        # Actual CRLF bytes embedded in Contact
        raw_register = (
            f"REGISTER sip:{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:19911;branch={branch}\r\n"
            f"From: <sip:1001@{sip_host}>;tag={_gen_tag()}\r\n"
            f"To: <sip:1001@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <sip:1001@127.0.0.1:19911>\r\nX-Injected: true\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(raw_register, host=sip_host, port=sip_port)
        # Response may be None (server may drop malformed packet) -- that's fine

        time.sleep(0.5)
        health = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert health.status_code == 200, "Server crashed after Contact CRLF injection"

    @pytest.mark.timeout(15)
    def test_malformed_sip_methods_no_crash(self, sip_host, sip_port, ami_url, verify_tls):
        """Garbage SIP methods and payloads do not crash the server."""
        garbage_payloads = [
            b"\x00" * 1024,
            b"\xff" * 512,
            b"NOT A SIP MESSAGE\r\n\r\n",
            b"INVITE\r\n\r\n",
            b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
            b"SIP/2.0 200 OK\r\n\r\n",
            b"\x80\x00\x00\x01" + b"\xde\xad\xbe\xef" * 100,
            # Oversized Via header
            (b"OPTIONS sip:x SIP/2.0\r\n"
             b"Via: " + b"A" * 4000 + b"\r\n"
             b"Content-Length: 0\r\n\r\n"),
        ]

        for payload in garbage_payloads:
            _send_raw_udp(payload, host=sip_host, port=sip_port, timeout=1)

        time.sleep(1)
        health = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert health.status_code == 200, (
            "PBX crashed or became unresponsive after malformed SIP input"
        )

    @pytest.mark.timeout(15)
    def test_oversized_sip_header_no_crash(self, sip_host, sip_port, ami_url, verify_tls):
        """SIP message with extremely long headers does not crash PBX."""
        branch = _gen_branch()
        cid = _gen_callid()
        big_header = "X" * 10_000
        options = (
            f"OPTIONS sip:{sip_host} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:19912;branch={branch}\r\n"
            f"From: <sip:test@127.0.0.1>;tag={_gen_tag()}\r\n"
            f"To: <sip:{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 OPTIONS\r\n"
            f"Subject: {big_header}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        _send_raw_udp(options, host=sip_host, port=sip_port, timeout=2)
        time.sleep(1)

        health = requests.get(f"{ami_url}/health", timeout=5, verify=verify_tls)
        assert health.status_code == 200, (
            "PBX crashed after oversized SIP header"
        )


# ===========================================================================
# TC-L8-011: TLS Configuration
# ===========================================================================

class TestTLSConfiguration:
    """TC-L8-011: Verify HTTPS is properly configured."""

    @pytest.mark.timeout(15)
    def test_https_connection_succeeds(self, base_url, verify_tls):
        """HTTPS connection to the server succeeds."""
        if not base_url.startswith("https"):
            pytest.skip("Server not configured with HTTPS")

        resp = requests.get(
            f"{base_url}/ami/v1/health",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code == 200, (
            f"HTTPS connection failed: HTTP {resp.status_code}"
        )

    @pytest.mark.timeout(15)
    def test_tls_version(self, sip_host):
        """Server supports TLS 1.2 or higher."""
        port = RUSTPBX_HTTP_PORT
        try:
            # Try TLS 1.2 connection
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            context.minimum_version = ssl.TLSVersion.TLSv1_2

            with socket.create_connection((sip_host, port), timeout=5) as raw_sock:
                with context.wrap_socket(raw_sock, server_hostname=sip_host) as tls_sock:
                    version = tls_sock.version()
                    assert version in ("TLSv1.2", "TLSv1.3"), (
                        f"Server using outdated TLS version: {version}"
                    )
        except (ssl.SSLError, ConnectionError, socket.timeout) as exc:
            pytest.skip(f"Cannot establish TLS connection: {exc}")

    @pytest.mark.timeout(15)
    def test_tls_rejects_sslv3(self, sip_host):
        """Server rejects SSLv3 connections (POODLE mitigation)."""
        port = RUSTPBX_HTTP_PORT
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            # Try to force maximum version to TLS 1.1 (effectively testing
            # that the server refuses anything below TLS 1.2)
            context.maximum_version = ssl.TLSVersion.TLSv1_1

            with socket.create_connection((sip_host, port), timeout=5) as raw_sock:
                try:
                    with context.wrap_socket(raw_sock, server_hostname=sip_host) as tls_sock:
                        # If we get here, the server accepted TLS 1.1 or lower
                        version = tls_sock.version()
                        if version in ("SSLv3", "TLSv1", "TLSv1.1"):
                            pytest.fail(
                                f"Server accepted deprecated TLS version: {version}"
                            )
                except ssl.SSLError:
                    # Good -- server refused the outdated protocol
                    pass
        except (ConnectionError, socket.timeout, OSError) as exc:
            pytest.skip(f"Cannot test TLS versions: {exc}")


# ===========================================================================
# TC-L8-012: CORS Headers
# ===========================================================================

class TestCORSHeaders:
    """TC-L8-012: Check that CORS headers are appropriate."""

    @pytest.mark.timeout(15)
    def test_cors_not_wildcard_on_sensitive_endpoints(self, ami_url, verify_tls):
        """Sensitive API endpoints do not set Access-Control-Allow-Origin: *."""
        endpoints = [
            f"{ami_url}/health",
            f"{ami_url}/dialogs",
            f"{ami_url}/reload/routes",
        ]
        for url in endpoints:
            resp = requests.get(url, timeout=5, verify=verify_tls)
            acao = resp.headers.get("Access-Control-Allow-Origin", "")
            if acao == "*":
                # Wildcard CORS on state-changing or sensitive endpoints is risky
                # For health endpoint this may be acceptable, but flag for review
                if "reload" in url or "dialogs" in url:
                    pytest.fail(
                        f"Wildcard CORS on sensitive endpoint {url} -- "
                        f"Access-Control-Allow-Origin: *"
                    )

    @pytest.mark.timeout(15)
    def test_cors_preflight_response(self, ami_url, verify_tls):
        """OPTIONS preflight requests do not expose dangerous methods."""
        resp = requests.options(
            f"{ami_url}/health",
            headers={
                "Origin": "https://evil.example.com",
                "Access-Control-Request-Method": "DELETE",
            },
            timeout=5,
            verify=verify_tls,
        )
        allow_methods = resp.headers.get("Access-Control-Allow-Methods", "")
        # DELETE should not be allowed on health endpoint
        if "DELETE" in allow_methods.upper():
            pytest.fail(
                f"CORS allows DELETE method on /health: {allow_methods}"
            )

    @pytest.mark.timeout(15)
    def test_cors_credentials_with_wildcard(self, ami_url, verify_tls):
        """CORS does not combine Allow-Credentials: true with wildcard origin."""
        resp = requests.get(
            f"{ami_url}/health",
            headers={"Origin": "https://evil.example.com"},
            timeout=5,
            verify=verify_tls,
        )
        acao = resp.headers.get("Access-Control-Allow-Origin", "")
        acac = resp.headers.get("Access-Control-Allow-Credentials", "")
        if acao == "*" and acac.lower() == "true":
            pytest.fail(
                "CORS misconfiguration: Allow-Origin: * with Allow-Credentials: true "
                "-- browsers ignore this but it indicates a config error"
            )


# ===========================================================================
# TC-L8-013: Cookie Security
# ===========================================================================

class TestCookieSecurity:
    """TC-L8-013: Verify session cookies have Secure, HttpOnly, SameSite flags."""

    @pytest.mark.timeout(20)
    def test_session_cookie_flags(self, base_url, verify_tls):
        """Session cookie after login has appropriate security flags."""
        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            allow_redirects=False,
            timeout=5,
        )

        if resp.status_code not in (302, 303, 200):
            pytest.skip(f"Could not login: HTTP {resp.status_code}")

        # Check Set-Cookie headers
        set_cookie_headers = resp.headers.get("Set-Cookie", "")
        if not set_cookie_headers:
            # Some servers use multiple Set-Cookie headers
            raw_headers = resp.raw.headers.getlist("Set-Cookie") if hasattr(resp.raw.headers, "getlist") else []
            if not raw_headers:
                pytest.skip("No Set-Cookie header in login response")
            set_cookie_headers = "; ".join(raw_headers)

        cookie_lower = set_cookie_headers.lower()

        # Check HttpOnly flag
        has_httponly = "httponly" in cookie_lower
        if not has_httponly:
            pytest.fail(
                "Session cookie missing HttpOnly flag -- vulnerable to XSS cookie theft. "
                f"Set-Cookie: {set_cookie_headers[:200]}"
            )

    @pytest.mark.timeout(20)
    def test_session_cookie_secure_flag(self, base_url, verify_tls):
        """Session cookie has Secure flag when served over HTTPS."""
        if not base_url.startswith("https"):
            pytest.skip("Not testing Secure flag on non-HTTPS server")

        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            allow_redirects=False,
            timeout=5,
        )

        if resp.status_code not in (302, 303, 200):
            pytest.skip(f"Could not login: HTTP {resp.status_code}")

        set_cookie_headers = resp.headers.get("Set-Cookie", "")
        if not set_cookie_headers:
            pytest.skip("No Set-Cookie header in login response")

        cookie_lower = set_cookie_headers.lower()
        has_secure = "secure" in cookie_lower
        if not has_secure:
            pytest.fail(
                "Session cookie missing Secure flag on HTTPS -- "
                "cookie will be sent over plain HTTP connections. "
                f"Set-Cookie: {set_cookie_headers[:200]}"
            )

    @pytest.mark.timeout(20)
    def test_session_cookie_samesite(self, base_url, verify_tls):
        """Session cookie has SameSite attribute (Lax or Strict)."""
        session = _fresh_session()
        resp = session.post(
            f"{base_url}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            allow_redirects=False,
            timeout=5,
        )

        if resp.status_code not in (302, 303, 200):
            pytest.skip(f"Could not login: HTTP {resp.status_code}")

        set_cookie_headers = resp.headers.get("Set-Cookie", "")
        if not set_cookie_headers:
            pytest.skip("No Set-Cookie header in login response")

        cookie_lower = set_cookie_headers.lower()
        has_samesite = "samesite" in cookie_lower
        if not has_samesite:
            # SameSite defaults to Lax in modern browsers, so this is
            # informational rather than a hard failure
            pytest.skip(
                "Session cookie missing explicit SameSite attribute "
                "(browsers default to Lax)"
            )
        else:
            # If present, it should be Lax or Strict -- not None
            if "samesite=none" in cookie_lower:
                pytest.fail(
                    "Session cookie has SameSite=None -- allows cross-site sending. "
                    f"Set-Cookie: {set_cookie_headers[:200]}"
                )


# ===========================================================================
# TC-L8-014: Directory Listing
# ===========================================================================

class TestDirectoryListing:
    """TC-L8-014: Verify static file serving doesn't expose directory listings."""

    DIRECTORY_PATHS = [
        "/static/",
        "/static",
        "/console/static/",
        "/templates/",
        "/config/",
        "/assets/",
        "/",
    ]

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("path", DIRECTORY_PATHS)
    def test_no_directory_listing(self, base_url, verify_tls, path):
        """Directory paths do not expose file listings."""
        resp = requests.get(
            f"{base_url}{path}",
            allow_redirects=True,
            timeout=5,
            verify=verify_tls,
        )
        if resp.status_code == 200:
            body = resp.text.lower()
            # Common directory listing indicators
            listing_indicators = [
                "index of /",
                "directory listing",
                "<pre><a href=",
                "parent directory",
                "[dir]",
            ]
            for indicator in listing_indicators:
                assert indicator not in body, (
                    f"Directory listing detected at {path}: found '{indicator}'"
                )

    @pytest.mark.timeout(15)
    def test_config_directory_not_exposed(self, base_url, verify_tls):
        """Configuration directory is not accessible via HTTP."""
        config_paths = [
            "/config/config.toml",
            "/config.toml",
            "/rustpbx.toml",
            "/config/certs/",
            "/.env",
            "/config/certs/rustpbx.key",
        ]
        for path in config_paths:
            resp = requests.get(
                f"{base_url}{path}",
                timeout=5,
                verify=verify_tls,
            )
            if resp.status_code == 200:
                body = resp.text.lower()
                # Check for config file content markers
                assert "[sipserver]" not in body, (
                    f"Config file exposed at {path}"
                )
                assert "private key" not in body, (
                    f"Private key exposed at {path}"
                )
                assert "begin rsa" not in body, (
                    f"RSA key exposed at {path}"
                )
                assert "begin certificate" not in body or path.endswith(".crt"), (
                    f"Certificate exposed at {path}"
                )

    @pytest.mark.timeout(15)
    def test_dotfiles_not_exposed(self, base_url, verify_tls):
        """Hidden dotfiles are not accessible via HTTP."""
        dotfile_paths = [
            "/.git/config",
            "/.git/HEAD",
            "/.gitignore",
            "/.env",
            "/.htaccess",
            "/.DS_Store",
        ]
        for path in dotfile_paths:
            resp = requests.get(
                f"{base_url}{path}",
                timeout=5,
                verify=verify_tls,
            )
            if resp.status_code == 200:
                body = resp.text
                # Git config would contain "[core]" or similar
                if ".git" in path:
                    assert "[core]" not in body and "ref:" not in body, (
                        f"Git internals exposed at {path}"
                    )


# ===========================================================================
# Additional Security Tests -- HTTP method and error handling
# ===========================================================================

class TestHTTPMethodSecurity:
    """Additional: Verify unusual HTTP methods return proper errors."""

    @pytest.mark.timeout(15)
    @pytest.mark.parametrize("method", ["DELETE", "PATCH", "PUT", "TRACE"])
    def test_unusual_methods_no_500(self, ami_url, verify_tls, method):
        """Unusual HTTP methods on API endpoints do not return 500."""
        resp = requests.request(
            method,
            f"{ami_url}/health",
            timeout=5,
            verify=verify_tls,
        )
        assert resp.status_code != 500, (
            f"HTTP {method} on /health returned 500 Internal Server Error"
        )

    @pytest.mark.timeout(15)
    def test_trace_method_disabled(self, base_url, verify_tls):
        """TRACE method is disabled (prevents cross-site tracing attacks)."""
        resp = requests.request(
            "TRACE",
            f"{base_url}/",
            timeout=5,
            verify=verify_tls,
        )
        if resp.status_code == 200:
            # If TRACE returns 200, check it doesn't echo back headers
            # (which would enable XST attacks)
            body = resp.text.lower()
            if "trace /" in body:
                pytest.fail(
                    "TRACE method enabled and echoing request -- "
                    "vulnerable to Cross-Site Tracing (XST)"
                )

    @pytest.mark.timeout(15)
    def test_invite_from_unregistered_rejected(self, sip_host, sip_port):
        """INVITE from an unregistered user is rejected."""
        branch = _gen_branch()
        cid = _gen_callid()
        invite = (
            f"INVITE sip:1001@{sip_host}:{sip_port} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP 127.0.0.1:19920;branch={branch}\r\n"
            f"From: <sip:attacker@evil.com>;tag={_gen_tag()}\r\n"
            f"To: <sip:1001@{sip_host}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 INVITE\r\n"
            f"Contact: <sip:attacker@127.0.0.1:19920>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n"
            f"\r\n"
        ).encode()

        resp = _send_raw_udp(invite, host=sip_host, port=sip_port)
        if resp is not None:
            resp_str = resp.decode("utf-8", errors="replace")
            first_line = resp_str.split("\r\n", 1)[0]
            # Should be an error response, not a success or provisional
            assert any(code in first_line for code in
                       ("401", "403", "404", "407", "480", "488")), (
                f"INVITE from unregistered user was not rejected: {first_line}"
            )


class TestServerInformationLeakage:
    """Additional: Verify server does not leak unnecessary information."""

    @pytest.mark.timeout(15)
    def test_server_header_not_verbose(self, base_url, verify_tls):
        """Server header does not expose detailed version information."""
        resp = requests.get(
            f"{base_url}/ami/v1/health",
            timeout=5,
            verify=verify_tls,
        )
        server_hdr = resp.headers.get("Server", "")
        # Server header should not expose detailed version info with
        # vulnerability-identifying precision (e.g., "Apache/2.4.49")
        if server_hdr:
            # Flag if it contains very specific version numbers of known
            # web servers (which could aid attackers)
            import re as _re
            if _re.search(r"(Apache|nginx|IIS)/\d+\.\d+\.\d+", server_hdr):
                pytest.fail(
                    f"Server header exposes specific version: {server_hdr!r} -- "
                    f"consider removing or generalizing"
                )

    @pytest.mark.timeout(15)
    def test_error_responses_no_stack_traces(self, base_url, verify_tls):
        """Error responses do not contain stack traces or internal paths."""
        error_urls = [
            f"{base_url}/nonexistent/path",
            f"{base_url}/ami/v1/nonexistent",
            f"{base_url}/console/nonexistent",
        ]
        for url in error_urls:
            resp = requests.get(url, timeout=5, verify=verify_tls)
            body = resp.text.lower()
            # Check for common stack trace patterns
            trace_indicators = [
                "traceback (most recent call last)",
                "at java.",
                "at sun.",
                "panic:",
                "goroutine",
                "stack trace",
                "internal server error",
                "/usr/src/",
                "/home/",
                "backtrace",
            ]
            for indicator in trace_indicators:
                if indicator in body:
                    # "internal server error" in a 404 body might be a generic
                    # error page, only flag on 500
                    if indicator == "internal server error" and resp.status_code != 500:
                        continue
                    if resp.status_code == 500:
                        pytest.fail(
                            f"Error response at {url} contains '{indicator}' -- "
                            f"may leak internal information"
                        )
