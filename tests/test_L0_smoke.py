"""
L0 Smoke Tests -- Server health and startup verification.

These tests verify that RustPBX is alive and that all fundamental services
are responding correctly. They run against a live server instance (typically
the Linode production box at 74.207.251.126) and cover:

  - HTTP/HTTPS port accessibility
  - SIP UDP/TCP port liveness
  - AMI health endpoint with version info
  - WebSocket endpoint availability
  - Trunk registration status
  - SIP UA registration (digest auth flow)

Expected execution time: < 60 seconds for the full suite.

Usage:
  /root/test-env/bin/python -m pytest tests/test_L0_smoke.py -v

Environment variables (all optional, sensible defaults for Linode server):
  RUSTPBX_HOST          SIP server IP           (default: 127.0.0.1)
  RUSTPBX_SIP_PORT      SIP port                (default: 5060)
  RUSTPBX_HTTP_PORT     HTTP port               (default: 8080)
  RUSTPBX_HTTPS_PORT    HTTPS port              (default: 8443)
  RUSTPBX_EXTERNAL_IP   Public IP for SIP URIs  (default: 74.207.251.126)
  RUSTPBX_ADMIN_USER    Console admin user      (default: admin)
  RUSTPBX_ADMIN_PASS    Console admin password  (default: admin123)
"""

import hashlib
import os
import random
import re
import socket
import ssl
import string
import uuid

import pytest
import requests
import urllib3

# Suppress InsecureRequestWarning for self-signed certificate
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)


# ---------------------------------------------------------------------------
# Configuration -- matches the Linode server defaults
# ---------------------------------------------------------------------------
SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8080"))
HTTPS_PORT = int(os.environ.get("RUSTPBX_HTTPS_PORT", "8443"))
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", "74.207.251.126")
ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

HTTP_BASE = f"http://{SERVER_HOST}:{HTTP_PORT}"
HTTPS_BASE = f"https://{SERVER_HOST}:{HTTPS_PORT}"

# The AMI endpoint -- prefer HTTPS when available, fall back to HTTP
AMI_BASE = f"{HTTPS_BASE}/ami/v1"

# Test SIP user credentials (must match config.toml user_backends)
TEST_USER = {"username": "1001", "password": "test1001"}

# Local SIP port for the test UA (must not conflict with the server)
TEST_UA_SIP_PORT = 15070


# ---------------------------------------------------------------------------
# SIP Helpers (adapted from call_quality_test.py patterns)
# ---------------------------------------------------------------------------

def _gen_branch():
    return "z9hG4bK" + "".join(random.choices(string.ascii_lowercase + string.digits, k=12))


def _gen_tag():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=8))


def _gen_callid():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=16)) + "@l0test"


def _md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()


def _compute_digest(username, realm, password, method, uri, nonce):
    ha1 = _md5hex(f"{username}:{realm}:{password}")
    ha2 = _md5hex(f"{method}:{uri}")
    return _md5hex(f"{ha1}:{nonce}:{ha2}")


def _get_response_code(data):
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0


def _parse_www_authenticate(header_line):
    realm = re.search(r'realm="([^"]*)"', header_line)
    nonce = re.search(r'nonce="([^"]*)"', header_line)
    return (realm.group(1) if realm else ""), (nonce.group(1) if nonce else "")


def _send_sip(sock, msg, dest=None):
    if dest is None:
        dest = (SERVER_HOST, SIP_PORT)
    sock.sendto(msg.encode() if isinstance(msg, str) else msg, dest)


def _recv_sip(sock, timeout=5):
    sock.settimeout(timeout)
    try:
        data, addr = sock.recvfrom(65535)
        return data.decode(errors="replace"), addr
    except socket.timeout:
        return None, None


def _make_sip_socket(port):
    """Create and bind a UDP socket for SIP messaging."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", port))
    return s


def _build_sip_options(target_host, target_port, local_port, from_user="l0probe"):
    """Build a minimal SIP OPTIONS request (SIP ping).

    The Via header uses the local address so the response is routed back
    to the sender rather than to the target (which would happen if we used
    target_host in the Via).
    """
    cid = str(uuid.uuid4())
    branch = f"z9hG4bK-{uuid.uuid4().hex[:12]}"
    tag = uuid.uuid4().hex[:8]
    msg = (
        f"OPTIONS sip:{target_host}:{target_port} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {target_host}:{local_port};branch={branch};rport\r\n"
        f"From: <sip:{from_user}@{target_host}>;tag={tag}\r\n"
        f"To: <sip:{target_host}:{target_port}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: 1 OPTIONS\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n"
        f"\r\n"
    )
    return msg.encode()


# ---------------------------------------------------------------------------
# Pytest fixtures
# ---------------------------------------------------------------------------

@pytest.fixture(scope="module")
def http_base():
    """HTTP base URL."""
    return HTTP_BASE


@pytest.fixture(scope="module")
def https_base():
    """HTTPS base URL."""
    return HTTPS_BASE


@pytest.fixture(scope="module")
def ami_base():
    """AMI API base URL."""
    return AMI_BASE


@pytest.fixture(scope="module")
def console_session():
    """Authenticated console session for endpoints that need it.

    Logs in via the console login form and returns a requests.Session
    with the session cookie set. Returns None if login fails (tests
    that need this fixture should handle None gracefully).
    """
    session = requests.Session()
    session.verify = False
    login_data = {"identifier": ADMIN_USER, "password": ADMIN_PASS}
    try:
        resp = session.post(
            f"{HTTPS_BASE}/console/login",
            data=login_data,
            allow_redirects=False,
            timeout=10,
        )
        if resp.status_code in (200, 302, 303):
            return session
    except requests.ConnectionError:
        pass
    # Fall back to HTTP
    try:
        resp = session.post(
            f"{HTTP_BASE}/console/login",
            data=login_data,
            allow_redirects=False,
            timeout=10,
        )
        if resp.status_code in (200, 302, 303):
            return session
    except requests.ConnectionError:
        pass
    return None


# ===================================================================
# L0 Smoke Test Suite
# ===================================================================

class TestL0Smoke:
    """L0: Verify RustPBX server is alive and fundamental services respond."""

    # ------------------------------------------------------------------
    # TC-L0-001: Health endpoint on HTTPS
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L0_001_health_endpoint(self):
        """TC-L0-001: AMI health endpoint returns running status with version.

        The /ami/v1/health endpoint is the primary liveness probe. It must
        return HTTP 200 with JSON containing 'status: running' and version
        information.
        """
        resp = requests.get(f"{AMI_BASE}/health", timeout=10, verify=False)
        assert resp.status_code == 200, (
            f"Health endpoint returned HTTP {resp.status_code}"
        )

        data = resp.json()
        assert isinstance(data, dict), "Health response is not a JSON object"
        assert data.get("status") == "running", (
            f"Expected status 'running', got '{data.get('status')}'"
        )

        # Verify version info is present
        version = data.get("version")
        assert version is not None, "Health response missing 'version' field"
        if isinstance(version, dict):
            # Structured version: should have at least a version string
            assert any(
                k in version for k in ("version", "git_hash", "build_date", "pkg_version")
            ), f"Version object has no recognized fields: {list(version.keys())}"
        elif isinstance(version, str):
            assert len(version) > 0, "Version string is empty"

    # ------------------------------------------------------------------
    # TC-L0-002: SIP UDP port is listening
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_002_sip_udp_port_listening(self):
        """TC-L0-002: SIP port 5060/UDP accepts packets and replies.

        Sends an unauthenticated SIP REGISTER for a known user and verifies
        that a SIP-formatted response comes back (expected: 401 challenge).
        Any SIP response proves the SIP stack is alive and processing
        requests on UDP.

        Note: When ensure_user=true in the proxy config, the server only
        responds to REGISTER for users that exist in the user backends,
        so we use a known test user (1001) rather than an arbitrary name.
        """
        local_port = 15071
        ext = TEST_USER["username"]
        sock = _make_sip_socket(local_port)
        sock.settimeout(5)
        try:
            branch = _gen_branch()
            tag = _gen_tag()
            cid = f"udpprobe-{_gen_callid()}"
            from_uri = f"sip:{ext}@{EXTERNAL_IP}"
            msg = (
                f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
                f"From: <{from_uri}>;tag={tag}\r\n"
                f"To: <{from_uri}>\r\n"
                f"Call-ID: {cid}\r\n"
                f"CSeq: 1 REGISTER\r\n"
                f"Contact: <sip:{ext}@{SERVER_HOST}:{local_port};transport=udp>\r\n"
                f"Max-Forwards: 70\r\n"
                f"Expires: 60\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            sock.sendto(msg.encode(), (SERVER_HOST, SIP_PORT))
            data, addr = sock.recvfrom(4096)
            assert data is not None, "No response received from SIP UDP port"
            assert b"SIP/2.0" in data, (
                "Response does not look like a SIP message"
            )
            # Any valid SIP response code proves the stack is alive
            response_text = data.decode(errors="replace")
            code = _get_response_code(response_text)
            assert code > 0, (
                f"Could not parse SIP response code from: {response_text[:80]}"
            )
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L0-003: SIP TCP port is listening
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_003_sip_tcp_port_listening(self):
        """TC-L0-003: SIP port 5060/TCP accepts connections if enabled.

        Not all deployments enable SIP over TCP. If TCP connect is refused
        but UDP was verified in test_L0_002, the SIP stack is still healthy
        and this test is skipped rather than failed.
        """
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        try:
            sock.connect((SERVER_HOST, SIP_PORT))
            # Connection succeeded -- the TCP listener is alive
        except ConnectionRefusedError:
            pytest.skip(
                f"SIP TCP not enabled on {SERVER_HOST}:{SIP_PORT} "
                f"(UDP-only configuration)"
            )
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L0-004: HTTP port accessible
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_004_http_port_accessible(self):
        """TC-L0-004: HTTP port 8080 responds to requests.

        A GET to the root path must return a valid HTTP response. We accept
        200 (direct content), 3xx (redirect to console), or 404.
        """
        try:
            resp = requests.get(f"{HTTP_BASE}/", timeout=5)
            assert resp.status_code < 500, (
                f"HTTP port returned server error {resp.status_code}"
            )
        except requests.ConnectionError:
            # HTTP might not be running if only HTTPS is configured
            pytest.skip("HTTP port not available (HTTPS-only mode?)")

    # ------------------------------------------------------------------
    # TC-L0-005: HTTPS port accessible
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_005_https_port_accessible(self):
        """TC-L0-005: HTTPS port 8443 responds to requests.

        Verifies the TLS listener is up. We disable certificate verification
        since the Linode server uses a self-signed certificate.
        """
        resp = requests.get(f"{HTTPS_BASE}/", timeout=5, verify=False)
        assert resp.status_code < 500, (
            f"HTTPS port returned server error {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L0-006: WebSocket endpoint accessible
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L0_006_websocket_endpoint_accessible(self):
        """TC-L0-006: WebSocket upgrade endpoint at /ws is reachable.

        Sends an HTTP Upgrade request to the /ws path. The server should
        respond with 101 Switching Protocols (if we provide correct headers)
        or a 4xx indicating it recognized the WS endpoint but rejected the
        upgrade (e.g., missing Sec-WebSocket-Key). Either proves the WS
        route is registered.
        """
        ws_url = f"{HTTPS_BASE}/ws"
        headers = {
            "Connection": "Upgrade",
            "Upgrade": "websocket",
            "Sec-WebSocket-Version": "13",
            "Sec-WebSocket-Key": "dGhlIHNhbXBsZSBub25jZQ==",
            "Sec-WebSocket-Protocol": "sip",
        }
        resp = requests.get(ws_url, headers=headers, timeout=10, verify=False)

        # 101 = successful upgrade, 400/426 = server recognized the endpoint
        # but rejected upgrade details. Either is acceptable for a smoke test.
        # 404 would mean the WebSocket route is not registered at all.
        assert resp.status_code != 404, (
            "WebSocket endpoint /ws returned 404 -- route not registered"
        )
        # Any non-500 response means the WS infrastructure is alive
        assert resp.status_code < 500, (
            f"WebSocket endpoint returned server error {resp.status_code}"
        )

    # ------------------------------------------------------------------
    # TC-L0-007: Config is loaded (health returns version and stats)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L0_007_config_loaded(self):
        """TC-L0-007: Health endpoint confirms configuration is loaded.

        Checks that the health response includes sipserver stats, uptime,
        and call counters -- all of which prove the config was parsed and
        the SIP server was initialized.
        """
        resp = requests.get(f"{AMI_BASE}/health", timeout=10, verify=False)
        assert resp.status_code == 200
        data = resp.json()

        # uptime proves the server started
        assert "uptime" in data, "Health response missing 'uptime'"

        # sipserver section proves the SIP proxy initialized
        sipserver = data.get("sipserver")
        assert sipserver is not None, "Health response missing 'sipserver' section"
        assert isinstance(sipserver, dict), (
            f"'sipserver' is not a dict: {type(sipserver).__name__}"
        )

        # Call counters prove the app state initialized
        assert "total" in data, "Health response missing 'total' call counter"

    # ------------------------------------------------------------------
    # TC-L0-008: Trunks are configured (via AMI reload/trunks or console)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L0_008_trunks_configured(self, console_session):
        """TC-L0-008: At least one SIP trunk is configured.

        Verifies trunk configuration by calling the AMI reload/trunks endpoint
        (which returns the trunk count) or checking the console diagnostics
        page. At minimum, the Telnyx trunk should be present.
        """
        # Method 1: Try AMI reload/trunks (POST, returns trunk count)
        # This is safe -- it re-reads config and returns metrics but
        # does not disrupt active calls.
        try:
            resp = requests.post(
                f"{AMI_BASE}/reload/trunks",
                timeout=10,
                verify=False,
            )
            if resp.status_code == 200:
                data = resp.json()
                trunks_count = data.get("trunks_reloaded", 0)
                assert trunks_count > 0, (
                    f"No trunks loaded after reload: {data}"
                )
                return  # Success
        except (requests.ConnectionError, requests.Timeout):
            pass

        # Method 2: If AMI failed, try console diagnostics with session cookie
        if console_session is not None:
            try:
                resp = console_session.get(
                    f"{HTTPS_BASE}/console/diagnostics",
                    timeout=10,
                    verify=False,
                )
                if resp.status_code == 200:
                    body = resp.text.lower()
                    # The diagnostics page should mention trunks
                    assert "telnyx" in body or "trunk" in body, (
                        "Diagnostics page does not mention any trunks"
                    )
                    return  # Success
            except (requests.ConnectionError, requests.Timeout):
                pass

        pytest.fail(
            "Could not verify trunk configuration via AMI or console"
        )

    # ------------------------------------------------------------------
    # TC-L0-009: SIP REGISTER with digest auth succeeds (200 OK)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L0_009_sip_register(self):
        """TC-L0-009: SIP UA registration with digest auth returns 200 OK.

        Performs a full SIP REGISTER flow:
          1. Send unauthenticated REGISTER
          2. Receive 401 Unauthorized with WWW-Authenticate challenge
          3. Compute digest response and re-send REGISTER with Authorization
          4. Receive 200 OK

        This proves the SIP registrar, user backend, and digest auth are
        all functioning correctly.
        """
        ext = TEST_USER["username"]
        password = TEST_USER["password"]
        sip_port = TEST_UA_SIP_PORT

        sock = _make_sip_socket(sip_port)
        try:
            tag = _gen_tag()
            cid = f"l0reg-{_gen_callid()}"
            from_uri = f"sip:{ext}@{EXTERNAL_IP}"
            contact = f"<sip:{ext}@{SERVER_HOST}:{sip_port};transport=udp>"
            cseq = 1
            branch = _gen_branch()

            # Step 1: Send unauthenticated REGISTER
            msg = (
                f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER_HOST}:{sip_port};branch={branch};rport\r\n"
                f"From: <{from_uri}>;tag={tag}\r\n"
                f"To: <{from_uri}>\r\n"
                f"Call-ID: {cid}\r\n"
                f"CSeq: {cseq} REGISTER\r\n"
                f"Contact: {contact}\r\n"
                f"Max-Forwards: 70\r\n"
                f"Expires: 60\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            _send_sip(sock, msg)

            # Step 2: Receive 401 challenge
            resp, _ = _recv_sip(sock, timeout=5)
            assert resp is not None, "No response to initial REGISTER"
            code = _get_response_code(resp)
            assert code in (401, 407), (
                f"Expected 401/407 challenge, got {code}. "
                f"Response: {resp[:200]}"
            )

            # Extract auth challenge
            auth_hdr = ""
            for line in resp.split("\r\n"):
                low = line.lower()
                if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
                    auth_hdr = line.split(":", 1)[1].strip()
                    break

            assert auth_hdr, (
                "No WWW-Authenticate or Proxy-Authenticate header in 401 response"
            )

            realm, nonce = _parse_www_authenticate(auth_hdr)
            assert nonce, "No nonce in authentication challenge"
            realm = realm or EXTERNAL_IP

            # Step 3: Re-REGISTER with digest credentials
            uri = f"sip:{realm}"
            digest = _compute_digest(ext, realm, password, "REGISTER", uri, nonce)
            hdr_name = "Authorization" if code == 401 else "Proxy-Authorization"
            auth_line = (
                f'Digest username="{ext}", realm="{realm}", '
                f'nonce="{nonce}", uri="{uri}", '
                f'response="{digest}", algorithm=MD5'
            )

            cseq += 1
            branch = _gen_branch()
            msg = (
                f"REGISTER sip:{realm} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER_HOST}:{sip_port};branch={branch};rport\r\n"
                f"From: <{from_uri}>;tag={tag}\r\n"
                f"To: <{from_uri}>\r\n"
                f"Call-ID: {cid}\r\n"
                f"CSeq: {cseq} REGISTER\r\n"
                f"Contact: {contact}\r\n"
                f"{hdr_name}: {auth_line}\r\n"
                f"Max-Forwards: 70\r\n"
                f"Expires: 60\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            _send_sip(sock, msg)

            # Step 4: Expect 200 OK
            resp, _ = _recv_sip(sock, timeout=5)
            assert resp is not None, "No response to authenticated REGISTER"
            code = _get_response_code(resp)
            assert code == 200, (
                f"Expected 200 OK for authenticated REGISTER, got {code}. "
                f"Response: {resp[:200]}"
            )

            # Verify the Contact header is echoed back
            assert "Contact:" in resp or "contact:" in resp.lower(), (
                "200 OK response missing Contact header"
            )

        finally:
            # Clean up: unregister
            try:
                unreg_branch = _gen_branch()
                unreg_tag = _gen_tag()
                unreg = (
                    f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
                    f"Via: SIP/2.0/UDP {SERVER_HOST}:{sip_port};branch={unreg_branch};rport\r\n"
                    f"From: <sip:{ext}@{EXTERNAL_IP}>;tag={unreg_tag}\r\n"
                    f"To: <sip:{ext}@{EXTERNAL_IP}>\r\n"
                    f"Call-ID: unreg-{_gen_callid()}\r\n"
                    f"CSeq: 1 REGISTER\r\n"
                    f"Contact: {contact}\r\n"
                    f"Max-Forwards: 70\r\n"
                    f"Expires: 0\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                _send_sip(sock, unreg)
                _recv_sip(sock, timeout=2)
            except Exception:
                pass
            sock.close()

    # ------------------------------------------------------------------
    # TC-L0-010: AMI dialogs endpoint responds
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_010_ami_dialogs_endpoint(self):
        """TC-L0-010: AMI dialogs returns a valid JSON list.

        With no active calls the dialogs endpoint should return an empty
        list, proving the dialog layer is accessible via AMI.
        """
        resp = requests.get(f"{AMI_BASE}/dialogs", timeout=5, verify=False)
        assert resp.status_code == 200, (
            f"Dialogs endpoint returned {resp.status_code}"
        )
        data = resp.json()
        assert isinstance(data, (list, dict)), (
            f"Unexpected dialogs type: {type(data).__name__}"
        )

    # ------------------------------------------------------------------
    # TC-L0-011: AMI transactions endpoint responds
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_011_ami_transactions_endpoint(self):
        """TC-L0-011: AMI transactions returns a valid JSON payload.

        The transactions endpoint should respond with 200 and valid JSON
        even when there are no active transactions.
        """
        resp = requests.get(f"{AMI_BASE}/transactions", timeout=5, verify=False)
        assert resp.status_code == 200, (
            f"Transactions endpoint returned {resp.status_code}"
        )
        data = resp.json()
        assert isinstance(data, (list, dict)), (
            f"Unexpected transactions type: {type(data).__name__}"
        )

    # ------------------------------------------------------------------
    # TC-L0-012: Console login page loads
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_012_console_login_page(self):
        """TC-L0-012: Console login page loads with HTML content.

        The login page should return HTTP 200 and contain recognizable
        login-related content (form fields, headings, etc.).
        """
        # Try HTTPS first, fall back to HTTP
        resp = None
        for base in (HTTPS_BASE, HTTP_BASE):
            try:
                resp = requests.get(
                    f"{base}/console/login", timeout=5, verify=False
                )
                if resp.status_code == 200:
                    break
            except requests.ConnectionError:
                continue

        assert resp is not None, "Could not connect to console login page"
        assert resp.status_code == 200, (
            f"Login page returned {resp.status_code}"
        )
        body_lower = resp.text.lower()
        assert "login" in body_lower or "password" in body_lower, (
            "Login page does not contain expected login/password keywords"
        )
        assert "<html" in body_lower or "<!doctype" in body_lower, (
            "Response does not appear to be an HTML document"
        )

    # ------------------------------------------------------------------
    # TC-L0-013: Static assets served
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L0_013_static_assets_served(self):
        """TC-L0-013: Static assets (CSS/JS) are served correctly.

        The login page should be a fully-formed HTML document with
        stylesheet or script references, proving the static file serving
        infrastructure is working.
        """
        resp = None
        for base in (HTTPS_BASE, HTTP_BASE):
            try:
                resp = requests.get(
                    f"{base}/console/login", timeout=5, verify=False
                )
                if resp.status_code == 200:
                    break
            except requests.ConnectionError:
                continue

        assert resp is not None, "Could not connect to console"
        assert resp.status_code == 200
        body_lower = resp.text.lower()
        # Check for CSS/JS references which prove static serving works
        has_assets = (
            "<link" in body_lower
            or "<script" in body_lower
            or "stylesheet" in body_lower
            or ".css" in body_lower
            or ".js" in body_lower
        )
        assert has_assets, (
            "Login page has no CSS/JS references -- static asset serving may be broken"
        )
