"""
RustPBX Test Suite -- Shared Fixtures

Provides pytest fixtures and SIP message builders used across all test levels
(L0 smoke, L1 infrastructure, L2 API contracts).

Configuration is read from environment variables with sensible defaults that
match the docker-compose.test.yml setup.
"""
import os
import socket
import time
import uuid

import pytest
import requests

# ---------------------------------------------------------------------------
# Configuration from environment or defaults
# ---------------------------------------------------------------------------
RUSTPBX_HOST = os.environ.get("RUSTPBX_HOST", "rustpbx")
RUSTPBX_HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8080"))
RUSTPBX_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
RUSTPBX_BASE_URL = f"http://{RUSTPBX_HOST}:{RUSTPBX_HTTP_PORT}"
ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

# Pre-configured SIP users (must match test-config.toml)
TEST_USERS = [
    {"username": "1001", "password": "test1001"},
    {"username": "1002", "password": "test1002"},
    {"username": "1003", "password": "test1003"},
    {"username": "1004", "password": "test1004"},
]


# ---------------------------------------------------------------------------
# SIP message builders
# ---------------------------------------------------------------------------

def build_sip_options(target_host, target_port, from_user="testprobe", call_id=None):
    """Build a minimal SIP OPTIONS request.

    OPTIONS is the standard SIP "ping" -- it verifies the SIP stack is alive
    without triggering any call state.

    Args:
        target_host: SIP server hostname or IP.
        target_port: SIP server port.
        from_user:   User part of the From header.
        call_id:     Optional Call-ID; a random UUID is generated if omitted.

    Returns:
        bytes: The encoded SIP OPTIONS message ready to send over UDP/TCP.
    """
    cid = call_id or str(uuid.uuid4())
    branch = f"z9hG4bK-{uuid.uuid4().hex[:12]}"
    tag = uuid.uuid4().hex[:8]
    msg = (
        f"OPTIONS sip:{target_host}:{target_port} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {target_host}:{target_port};branch={branch}\r\n"
        f"From: <sip:{from_user}@{target_host}>;tag={tag}\r\n"
        f"To: <sip:{target_host}:{target_port}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: 1 OPTIONS\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n"
        f"\r\n"
    )
    return msg.encode()


def build_sip_register(target_host, target_port, username, domain=None,
                       call_id=None, expires=60):
    """Build a minimal SIP REGISTER request (unauthenticated).

    This intentionally omits the Authorization header so the server should
    respond with 401 Unauthorized and a WWW-Authenticate challenge.

    Args:
        target_host: SIP server hostname or IP.
        target_port: SIP server port.
        username:    SIP username (e.g. "1001").
        domain:      SIP domain; defaults to target_host.
        call_id:     Optional Call-ID; a random UUID is generated if omitted.
        expires:     Registration expiry in seconds.

    Returns:
        bytes: The encoded SIP REGISTER message ready to send over UDP/TCP.
    """
    domain = domain or target_host
    cid = call_id or str(uuid.uuid4())
    branch = f"z9hG4bK-{uuid.uuid4().hex[:12]}"
    tag = uuid.uuid4().hex[:8]
    msg = (
        f"REGISTER sip:{domain}:{target_port} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {target_host}:{target_port};branch={branch}\r\n"
        f"From: <sip:{username}@{domain}>;tag={tag}\r\n"
        f"To: <sip:{username}@{domain}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: 1 REGISTER\r\n"
        f"Contact: <sip:{username}@{target_host}:{target_port}>\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: {expires}\r\n"
        f"Content-Length: 0\r\n"
        f"\r\n"
    )
    return msg.encode()


# ---------------------------------------------------------------------------
# Session-scoped fixtures
# ---------------------------------------------------------------------------

@pytest.fixture(scope="session")
def base_url():
    """Base URL for HTTP/API requests."""
    return RUSTPBX_BASE_URL


@pytest.fixture(scope="session")
def sip_host():
    """SIP server hostname."""
    return RUSTPBX_HOST


@pytest.fixture(scope="session")
def sip_port():
    """SIP server port."""
    return RUSTPBX_SIP_PORT


@pytest.fixture(scope="session")
def api_session(base_url):
    """Authenticated requests.Session with console login cookie.

    Attempts to log in to the RustPBX console using the configured admin
    credentials. If login fails the entire session is skipped because most
    L1/L2 tests depend on an authenticated session.
    """
    session = requests.Session()
    login_data = {"identifier": ADMIN_USER, "password": ADMIN_PASS}
    try:
        resp = session.post(
            f"{base_url}/console/login",
            data=login_data,
            allow_redirects=False,
            timeout=10,
        )
    except requests.ConnectionError as exc:
        pytest.skip(f"Cannot connect to console for login: {exc}")
        return  # unreachable, but keeps type-checkers happy

    # 302/303 redirect means login succeeded and set a session cookie
    if resp.status_code not in (302, 303, 200):
        pytest.skip(f"Cannot login to console: HTTP {resp.status_code}")

    return session


@pytest.fixture(scope="session")
def ami_url(base_url):
    """AMI API base URL (e.g. http://rustpbx:8080/ami/v1)."""
    return f"{base_url}/ami/v1"


# ---------------------------------------------------------------------------
# Function-scoped fixtures
# ---------------------------------------------------------------------------

@pytest.fixture
def sip_socket(sip_host, sip_port):
    """Connected UDP socket for sending SIP messages.

    The socket is pre-connected to the SIP server so callers can simply
    ``sock.send(data)`` without specifying the address each time.
    Automatically closed after the test.
    """
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(5)
    try:
        sock.connect((sip_host, sip_port))
    except socket.error as exc:
        pytest.skip(f"Cannot connect UDP socket to SIP server: {exc}")
    yield sock
    sock.close()
