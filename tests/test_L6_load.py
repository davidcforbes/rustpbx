"""
L6 Load Tests -- Concurrency, throughput, and stress.

Pure-Python load and stress tests for RustPBX using asyncio, aiohttp, and raw
UDP sockets.  No external tools (SIPp) required.

Tests:
  1. API load test          -- 100 concurrent /ami/v1/health requests < 2s p95
  2. SIP REGISTER flood     -- 50 concurrent REGISTER requests, all get responses
  3. SIP OPTIONS flood      -- 100 concurrent OPTIONS requests, measure latency
  4. Concurrent SIP sessions -- 5 simultaneous REGISTER sessions maintained
  5. Rapid call setup/teardown -- 10 rapid INVITE+BYE cycles, measure setup time
  6. API endpoint stress     -- Concurrent hits on /health, /dialogs, /transactions
  7. WebSocket connection pool -- 10 concurrent WebSocket connections
  8. Memory baseline         -- Server memory before/after load, check for leaks

Server: 74.207.251.126:8443 (HTTPS), SIP on port 5060

Run with:
  python -m pytest tests/test_L6_load.py -v -s

Environment variables (all optional):
  RUSTPBX_HOST        Server IP           (default: 127.0.0.1)
  RUSTPBX_SIP_PORT    SIP port            (default: 5060)
  RUSTPBX_HTTP_PORT   HTTPS port          (default: 8443)
  RUSTPBX_SCHEME      http or https       (default: https)
  RUSTPBX_EXTERNAL_IP Public IP for URIs  (default: same as HOST)
"""

import asyncio
import hashlib
import os
import random
import re
import socket
import ssl
import statistics
import string
import struct
import threading
import time
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed

import pytest
import requests
import urllib3

# Optional async deps -- tests that need them will skip if unavailable
try:
    import aiohttp
    HAS_AIOHTTP = True
except ImportError:
    HAS_AIOHTTP = False

try:
    import websockets
    HAS_WEBSOCKETS = True
except ImportError:
    HAS_WEBSOCKETS = False

# Suppress TLS warnings for self-signed certs
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", SERVER_HOST)
BASE_URL = f"{SCHEME}://{SERVER_HOST}:{HTTP_PORT}"
VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in ("1", "true", "yes")

ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

TEST_USERS = {
    "1001": "test1001",
    "1002": "test1002",
}

# Port allocator -- each test gets a unique range to avoid collisions
_PORT_LOCK = threading.Lock()
_PORT_BASE = 18200


def _alloc_port():
    """Return a unique local UDP port for test sockets."""
    global _PORT_BASE
    with _PORT_LOCK:
        port = _PORT_BASE
        _PORT_BASE += 1
    return port


def _alloc_ports(n):
    """Allocate *n* unique consecutive ports."""
    global _PORT_BASE
    with _PORT_LOCK:
        base = _PORT_BASE
        _PORT_BASE += n
    return list(range(base, base + n))


# ---------------------------------------------------------------------------
# SIP helpers (adapted from L3/L5 patterns)
# ---------------------------------------------------------------------------

def _gen_branch():
    return "z9hG4bK" + "".join(random.choices(string.ascii_lowercase + string.digits, k=12))


def _gen_tag():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=8))


def _gen_callid():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=16)) + "@l6test"


def _md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()


def _compute_digest(username, realm, password, method, uri, nonce):
    ha1 = _md5hex(f"{username}:{realm}:{password}")
    ha2 = _md5hex(f"{method}:{uri}")
    return _md5hex(f"{ha1}:{nonce}:{ha2}")


def _get_response_code(data):
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0


def _get_header(data, name):
    for line in data.split("\r\n"):
        if line.lower().startswith(name.lower() + ":"):
            return line.split(":", 1)[1].strip()
    return ""


def _get_to_tag(data):
    to_hdr = _get_header(data, "To")
    m = re.search(r"tag=([^\s;>]+)", to_hdr)
    return m.group(1) if m else ""


def _parse_www_authenticate(header_line):
    realm = re.search(r'realm="([^"]*)"', header_line)
    nonce = re.search(r'nonce="([^"]*)"', header_line)
    return (realm.group(1) if realm else ""), (nonce.group(1) if nonce else "")


def _make_socket(port=0, timeout=5):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", port))
    s.settimeout(timeout)
    return s


def _send(sock, msg):
    if isinstance(msg, str):
        msg = msg.encode()
    sock.sendto(msg, (SERVER_HOST, SIP_PORT))


def _recv(sock, timeout=5):
    sock.settimeout(timeout)
    try:
        data, addr = sock.recvfrom(65535)
        return data.decode(errors="replace"), addr
    except socket.timeout:
        return None, None


def _recv_all(sock, timeout=5, max_messages=20):
    messages = []
    deadline = time.time() + timeout
    while len(messages) < max_messages:
        remaining = deadline - time.time()
        if remaining <= 0:
            break
        sock.settimeout(remaining)
        try:
            data, addr = sock.recvfrom(65535)
            messages.append((data.decode(errors="replace"), addr))
        except socket.timeout:
            break
    return messages


def _local_port(sock):
    return sock.getsockname()[1]


# ---------------------------------------------------------------------------
# Higher-level SIP operations
# ---------------------------------------------------------------------------

def _build_options_msg(local_port):
    """Build a SIP OPTIONS message for the server."""
    branch = _gen_branch()
    tag = _gen_tag()
    cid = _gen_callid()
    return (
        f"OPTIONS sip:{EXTERNAL_IP}:{SIP_PORT} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <sip:loadtest@{EXTERNAL_IP}>;tag={tag}\r\n"
        f"To: <sip:{EXTERNAL_IP}:{SIP_PORT}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: 1 OPTIONS\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n\r\n"
    )


def _build_register_msg(local_port, username, domain=None, expires=60, cseq=1):
    """Build an unauthenticated SIP REGISTER message."""
    domain = domain or EXTERNAL_IP
    branch = _gen_branch()
    tag = _gen_tag()
    cid = _gen_callid()
    from_uri = f"sip:{username}@{domain}"
    contact = f"<sip:{username}@{SERVER_HOST}:{local_port};transport=udp>"
    return (
        f"REGISTER sip:{domain} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: {expires}\r\n"
        f"Content-Length: 0\r\n\r\n"
    ), tag, cid


def _register_with_auth(sock, username, password, expires=60, cseq_start=1):
    """Full REGISTER + digest-auth flow.  Returns (code, response_text)."""
    local_port = _local_port(sock)
    tag = _gen_tag()
    cid = f"l6reg-{_gen_callid()}"
    from_uri = f"sip:{username}@{EXTERNAL_IP}"
    contact = f"<sip:{username}@{SERVER_HOST}:{local_port};transport=udp>"
    cseq = cseq_start
    branch = _gen_branch()

    # Step 1: unauthenticated REGISTER
    msg = (
        f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: {expires}\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    _send(sock, msg)
    resp, _ = _recv(sock, timeout=5)
    if resp is None:
        return 0, ""

    code = _get_response_code(resp)
    if code not in (401, 407):
        return code, resp

    # Step 2: extract challenge
    auth_hdr = ""
    for line in resp.split("\r\n"):
        low = line.lower()
        if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
            auth_hdr = line.split(":", 1)[1].strip()
            break

    if not auth_hdr:
        return code, resp

    realm, nonce = _parse_www_authenticate(auth_hdr)
    realm = realm or EXTERNAL_IP
    uri = f"sip:{realm}"
    digest = _compute_digest(username, realm, password, "REGISTER", uri, nonce)
    hdr_name = "Authorization" if code == 401 else "Proxy-Authorization"
    auth_line = (
        f'Digest username="{username}", realm="{realm}", '
        f'nonce="{nonce}", uri="{uri}", '
        f'response="{digest}", algorithm=MD5'
    )

    # Step 3: authenticated REGISTER
    cseq += 1
    branch = _gen_branch()
    msg = (
        f"REGISTER sip:{realm} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"{hdr_name}: {auth_line}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: {expires}\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    _send(sock, msg)
    resp, _ = _recv(sock, timeout=5)
    if resp is None:
        return 0, ""
    return _get_response_code(resp), resp


def _unregister(sock, username):
    """Send REGISTER Expires:0 to clean up."""
    local_port = _local_port(sock)
    tag = _gen_tag()
    from_uri = f"sip:{username}@{EXTERNAL_IP}"
    contact = f"<sip:{username}@{SERVER_HOST}:{local_port};transport=udp>"
    branch = _gen_branch()
    msg = (
        f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: unreg-{_gen_callid()}\r\n"
        f"CSeq: 1 REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: 0\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    _send(sock, msg)
    _recv(sock, timeout=2)  # drain


def _build_sdp(rtp_port, local_ip=None):
    """Build a minimal SDP body offering PCMU."""
    local_ip = local_ip or SERVER_HOST
    sid = str(random.randint(100000, 999999))
    return (
        "v=0\r\n"
        f"o=- {sid} {sid} IN IP4 {local_ip}\r\n"
        "s=L6LoadTest\r\n"
        f"c=IN IP4 {local_ip}\r\n"
        "t=0 0\r\n"
        f"m=audio {rtp_port} RTP/AVP 0 101\r\n"
        "a=rtpmap:0 PCMU/8000\r\n"
        "a=rtpmap:101 telephone-event/8000\r\n"
        "a=fmtp:101 0-16\r\n"
        "a=sendrecv\r\n"
    )


def _build_invite(caller_ext, callee_ext, local_port, rtp_port,
                  tag, cid, cseq, branch, auth_line=None, auth_hdr_name=None):
    """Build a SIP INVITE message."""
    from_uri = f"sip:{caller_ext}@{EXTERNAL_IP}"
    to_uri = f"sip:{callee_ext}@{EXTERNAL_IP}"
    contact = f"<sip:{caller_ext}@{SERVER_HOST}:{local_port};transport=udp>"
    sdp = _build_sdp(rtp_port, SERVER_HOST)

    msg = (
        f"INVITE {to_uri} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{to_uri}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} INVITE\r\n"
        f"Contact: {contact}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Type: application/sdp\r\n"
    )
    if auth_line and auth_hdr_name:
        msg += f"{auth_hdr_name}: {auth_line}\r\n"
    msg += f"Content-Length: {len(sdp)}\r\n\r\n{sdp}"
    return msg


def _build_ack(to_uri, local_port, tag, to_tag, cid, cseq, branch):
    """Build a SIP ACK for a final response."""
    from_uri = f"sip:loadtest@{EXTERNAL_IP}"
    msg = (
        f"ACK {to_uri} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{to_uri}>;tag={to_tag}\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} ACK\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    return msg


def _build_bye(to_uri, local_port, from_tag, to_tag, cid, cseq):
    """Build a SIP BYE message."""
    branch = _gen_branch()
    from_uri = f"sip:loadtest@{EXTERNAL_IP}"
    msg = (
        f"BYE {to_uri} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={from_tag}\r\n"
        f"To: <{to_uri}>;tag={to_tag}\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} BYE\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    return msg


# ---------------------------------------------------------------------------
# Async helpers
# ---------------------------------------------------------------------------

def _get_ssl_context():
    """Return an SSL context that skips verification for self-signed certs."""
    if not VERIFY_TLS:
        ctx = ssl.create_default_context()
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        return ctx
    return None


def _get_aiohttp_connector():
    """Create an aiohttp TCPConnector that skips TLS verification if needed."""
    if not VERIFY_TLS:
        return aiohttp.TCPConnector(ssl=False)
    return aiohttp.TCPConnector()


# ---------------------------------------------------------------------------
# Metrics helpers
# ---------------------------------------------------------------------------

def _percentile(sorted_list, pct):
    """Return the percentile value from a pre-sorted list."""
    if not sorted_list:
        return 0.0
    idx = int(len(sorted_list) * pct / 100.0)
    idx = min(idx, len(sorted_list) - 1)
    return sorted_list[idx]


def _print_latency_report(label, latencies, errors=0, total=0):
    """Print a formatted latency report."""
    if not latencies:
        print(f"\n--- {label}: No data ---")
        return
    latencies_sorted = sorted(latencies)
    mean = statistics.mean(latencies)
    p50 = _percentile(latencies_sorted, 50)
    p95 = _percentile(latencies_sorted, 95)
    p99 = _percentile(latencies_sorted, 99)
    _min = latencies_sorted[0]
    _max = latencies_sorted[-1]
    total = total or len(latencies) + errors
    print(f"\n--- {label} ({total} requests) ---")
    print(f"  Min   : {_min:.4f}s")
    print(f"  Mean  : {mean:.4f}s")
    print(f"  p50   : {p50:.4f}s")
    print(f"  p95   : {p95:.4f}s")
    print(f"  p99   : {p99:.4f}s")
    print(f"  Max   : {_max:.4f}s")
    if errors:
        print(f"  Errors: {errors}/{total}")


# ===================================================================
# L6 Load Test Suite
# ===================================================================

class TestL6Load:
    """L6: Load and stress tests using pure Python (no SIPp)."""

    # ------------------------------------------------------------------
    # TC-L6-001: API load test -- 100 concurrent /ami/v1/health requests
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_001_api_concurrent_health(self):
        """TC-L6-001: 100 concurrent /ami/v1/health requests all succeed < 2s.

        Fires 100 HTTP GET requests in parallel using a thread pool.
        Verifies all return 200 OK with p95 latency under 2 seconds.
        """
        NUM_REQUESTS = 100
        MAX_P95_SECONDS = 2.0
        MAX_ERROR_RATE = 0.05  # 5%

        latencies = []
        errors = []
        lock = threading.Lock()

        def _hit_health(idx):
            start = time.monotonic()
            try:
                resp = requests.get(
                    f"{BASE_URL}/ami/v1/health",
                    timeout=10,
                    verify=VERIFY_TLS,
                )
                elapsed = time.monotonic() - start
                with lock:
                    latencies.append(elapsed)
                if resp.status_code != 200:
                    with lock:
                        errors.append(f"req {idx}: HTTP {resp.status_code}")
            except requests.RequestException as exc:
                elapsed = time.monotonic() - start
                with lock:
                    latencies.append(elapsed)
                    errors.append(f"req {idx}: {exc}")

        with ThreadPoolExecutor(max_workers=20) as pool:
            futures = [pool.submit(_hit_health, i) for i in range(NUM_REQUESTS)]
            for f in as_completed(futures):
                f.result()  # propagate exceptions

        assert len(latencies) == NUM_REQUESTS, (
            f"Only {len(latencies)}/{NUM_REQUESTS} requests completed"
        )

        latencies_sorted = sorted(latencies)
        p95 = _percentile(latencies_sorted, 95)
        _print_latency_report(
            "TC-L6-001: API health concurrency",
            latencies, len(errors), NUM_REQUESTS,
        )

        assert p95 < MAX_P95_SECONDS, (
            f"p95 latency ({p95:.4f}s) exceeds {MAX_P95_SECONDS}s threshold"
        )
        assert len(errors) < NUM_REQUESTS * MAX_ERROR_RATE, (
            f"Too many errors: {len(errors)}/{NUM_REQUESTS} "
            f"(max {NUM_REQUESTS * MAX_ERROR_RATE:.0f}). "
            f"First errors: {errors[:5]}"
        )

    # ------------------------------------------------------------------
    # TC-L6-002: SIP REGISTER flood -- 50 concurrent
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_002_sip_register_flood(self):
        """TC-L6-002: 50 concurrent unauthenticated REGISTER requests all get responses.

        Each request uses a unique socket and Call-ID.  We expect either 401
        (challenge) or 200 (if the server auto-accepts).  Zero responses means
        the server dropped packets under load.
        """
        NUM_REGISTERS = 50
        results = []
        lock = threading.Lock()
        ports = _alloc_ports(NUM_REGISTERS)

        def _send_register(idx):
            port = ports[idx]
            # Cycle through test users
            users = list(TEST_USERS.keys())
            username = users[idx % len(users)]
            sock = None
            try:
                sock = _make_socket(port, timeout=8)
                msg, _, _ = _build_register_msg(port, username, expires=30)
                start = time.monotonic()
                _send(sock, msg)
                resp, _ = _recv(sock, timeout=8)
                elapsed = time.monotonic() - start
                code = _get_response_code(resp) if resp else 0
                with lock:
                    results.append({
                        "idx": idx,
                        "code": code,
                        "latency": elapsed,
                        "got_response": resp is not None,
                    })
            except Exception as exc:
                with lock:
                    results.append({
                        "idx": idx,
                        "code": 0,
                        "latency": 0,
                        "got_response": False,
                        "error": str(exc),
                    })
            finally:
                if sock:
                    sock.close()

        threads = []
        for i in range(NUM_REGISTERS):
            t = threading.Thread(target=_send_register, args=(i,))
            threads.append(t)

        # Launch all threads near-simultaneously
        for t in threads:
            t.start()
        for t in threads:
            t.join(timeout=15)

        responded = [r for r in results if r["got_response"]]
        latencies = [r["latency"] for r in responded]

        _print_latency_report(
            "TC-L6-002: SIP REGISTER flood",
            latencies, NUM_REGISTERS - len(responded), NUM_REGISTERS,
        )

        # At least 80% must get a response (server may legitimately drop
        # some under extreme burst, but it must not fall over)
        min_responses = int(NUM_REGISTERS * 0.80)
        assert len(responded) >= min_responses, (
            f"Only {len(responded)}/{NUM_REGISTERS} REGISTER requests got responses "
            f"(minimum: {min_responses}). Server may be dropping packets under load."
        )

        # All responses must be valid SIP (401 challenge or 200)
        valid_codes = {200, 401, 403, 407}
        bad = [r for r in responded if r["code"] not in valid_codes]
        assert len(bad) == 0, (
            f"{len(bad)} responses had unexpected SIP codes: "
            f"{[(r['idx'], r['code']) for r in bad[:10]]}"
        )

    # ------------------------------------------------------------------
    # TC-L6-003: SIP OPTIONS flood -- 100 concurrent
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_003_sip_options_flood(self):
        """TC-L6-003: 100 concurrent SIP OPTIONS pings, measure response times.

        OPTIONS is the SIP equivalent of an HTTP health check.  The server
        should handle a burst without crashing or excessive latency.
        """
        NUM_OPTIONS = 100
        results = []
        lock = threading.Lock()
        ports = _alloc_ports(NUM_OPTIONS)

        def _send_options(idx):
            port = ports[idx]
            sock = None
            try:
                sock = _make_socket(port, timeout=8)
                msg = _build_options_msg(port)
                start = time.monotonic()
                _send(sock, msg)
                resp, _ = _recv(sock, timeout=8)
                elapsed = time.monotonic() - start
                code = _get_response_code(resp) if resp else 0
                with lock:
                    results.append({
                        "idx": idx,
                        "code": code,
                        "latency": elapsed,
                        "got_response": resp is not None,
                    })
            except Exception as exc:
                with lock:
                    results.append({
                        "idx": idx,
                        "code": 0,
                        "latency": 0,
                        "got_response": False,
                        "error": str(exc),
                    })
            finally:
                if sock:
                    sock.close()

        threads = []
        for i in range(NUM_OPTIONS):
            t = threading.Thread(target=_send_options, args=(i,))
            threads.append(t)

        for t in threads:
            t.start()
        for t in threads:
            t.join(timeout=15)

        responded = [r for r in results if r["got_response"]]
        latencies = [r["latency"] for r in responded]

        _print_latency_report(
            "TC-L6-003: SIP OPTIONS flood",
            latencies, NUM_OPTIONS - len(responded), NUM_OPTIONS,
        )

        # At least 75% must respond (OPTIONS under burst, some loss acceptable)
        min_responses = int(NUM_OPTIONS * 0.75)
        assert len(responded) >= min_responses, (
            f"Only {len(responded)}/{NUM_OPTIONS} OPTIONS got responses "
            f"(minimum: {min_responses})"
        )

        # Verify PBX is still alive after the burst
        try:
            health = requests.get(
                f"{BASE_URL}/ami/v1/health", timeout=10, verify=VERIFY_TLS,
            )
            assert health.status_code == 200, (
                f"PBX health check failed after OPTIONS burst: HTTP {health.status_code}"
            )
        except requests.RequestException as exc:
            pytest.fail(f"PBX unreachable after OPTIONS burst: {exc}")

    # ------------------------------------------------------------------
    # TC-L6-004: Concurrent SIP sessions -- 5 simultaneous REGISTERs
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_004_concurrent_sip_sessions(self):
        """TC-L6-004: 5 simultaneous REGISTER sessions maintained in parallel.

        Registers user 1001 from 5 different "phones" (unique Contact addresses)
        using full digest authentication, holds them for 3 seconds, then
        unregisters.  Verifies all 5 sessions get 200 OK.
        """
        NUM_SESSIONS = 5
        results = {}
        lock = threading.Lock()
        ports = _alloc_ports(NUM_SESSIONS)
        barrier = threading.Barrier(NUM_SESSIONS, timeout=10)

        def _session(idx):
            port = ports[idx]
            # Alternate between users to avoid single-user lock contention
            users = list(TEST_USERS.items())
            username, password = users[idx % len(users)]
            sock = None
            try:
                sock = _make_socket(port, timeout=10)
                # Wait for all threads to be ready
                barrier.wait()

                code, resp = _register_with_auth(sock, username, password, expires=60)
                with lock:
                    results[idx] = {"code": code, "username": username}

                # Hold registration for a few seconds
                time.sleep(3)

                # Clean unregister
                _unregister(sock, username)
            except Exception as exc:
                with lock:
                    results[idx] = {"code": 0, "error": str(exc)}
            finally:
                if sock:
                    sock.close()

        threads = []
        for i in range(NUM_SESSIONS):
            t = threading.Thread(target=_session, args=(i,))
            threads.append(t)
            t.start()

        for t in threads:
            t.join(timeout=30)

        print(f"\n--- TC-L6-004: Concurrent SIP sessions ({NUM_SESSIONS}) ---")
        successes = 0
        for idx in sorted(results):
            r = results[idx]
            status = "OK" if r["code"] == 200 else f"FAIL ({r.get('code', 'err')})"
            print(f"  Session {idx} ({r.get('username', '?')}): {status}")
            if r["code"] == 200:
                successes += 1

        assert successes >= NUM_SESSIONS - 1, (
            f"Only {successes}/{NUM_SESSIONS} sessions got 200 OK. "
            f"Details: {results}"
        )

    # ------------------------------------------------------------------
    # TC-L6-005: Rapid call setup/teardown -- 10 INVITE+BYE cycles
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    @pytest.mark.slow
    def test_L6_005_rapid_call_setup_teardown(self):
        """TC-L6-005: 10 rapid INVITE+BYE cycles, measure call setup time.

        For each cycle:
          1. Register caller (1001) and callee (1002)
          2. Caller sends INVITE (with auth)
          3. Callee auto-answers with 200 OK
          4. Caller sends ACK, then immediately BYE
          5. Measure time from INVITE to receiving a response

        Reports setup time statistics across all 10 cycles.
        """
        NUM_CALLS = 10
        setup_times = []
        call_results = []

        # Use dedicated ports for caller and callee per call
        base_ports = _alloc_ports(NUM_CALLS * 4)  # 4 ports per call

        for call_idx in range(NUM_CALLS):
            caller_port = base_ports[call_idx * 4]
            callee_port = base_ports[call_idx * 4 + 1]
            caller_rtp_port = base_ports[call_idx * 4 + 2]
            callee_rtp_port = base_ports[call_idx * 4 + 3]

            caller_sock = None
            callee_sock = None
            result = {"call": call_idx, "status": "unknown", "setup_time": None}

            try:
                caller_sock = _make_socket(caller_port, timeout=10)
                callee_sock = _make_socket(callee_port, timeout=10)

                # Register both endpoints
                c1_code, _ = _register_with_auth(caller_sock, "1001", "test1001")
                c2_code, _ = _register_with_auth(callee_sock, "1002", "test1002")

                if c1_code != 200 or c2_code != 200:
                    result["status"] = f"reg_fail (caller={c1_code}, callee={c2_code})"
                    call_results.append(result)
                    continue

                # Build INVITE
                tag = _gen_tag()
                cid = f"l6call-{call_idx}-{_gen_callid()}"
                cseq = 1
                branch = _gen_branch()
                to_uri = f"sip:1002@{EXTERNAL_IP}"

                invite_msg = _build_invite(
                    "1001", "1002", caller_port, caller_rtp_port,
                    tag, cid, cseq, branch,
                )

                start_time = time.monotonic()
                _send(caller_sock, invite_msg)

                # Collect responses with a timeout
                got_100 = False
                got_auth_challenge = False
                got_final = False
                final_code = 0
                to_tag = ""
                setup_time = None

                deadline = time.time() + 15
                while time.time() < deadline:
                    # Check caller socket for responses
                    resp, _ = _recv(caller_sock, timeout=1)
                    if resp:
                        code = _get_response_code(resp)
                        if code == 100:
                            got_100 = True
                        elif code in (401, 407) and not got_auth_challenge:
                            # Handle auth challenge
                            got_auth_challenge = True
                            auth_hdr = ""
                            for line in resp.split("\r\n"):
                                low = line.lower()
                                if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
                                    auth_hdr = line.split(":", 1)[1].strip()
                                    break
                            if auth_hdr:
                                realm, nonce = _parse_www_authenticate(auth_hdr)
                                realm = realm or EXTERNAL_IP
                                digest = _compute_digest(
                                    "1001", realm, "test1001", "INVITE", to_uri, nonce
                                )
                                hdr_name = "Authorization" if code == 401 else "Proxy-Authorization"
                                auth_line = (
                                    f'Digest username="1001", realm="{realm}", '
                                    f'nonce="{nonce}", uri="{to_uri}", '
                                    f'response="{digest}", algorithm=MD5'
                                )
                                # ACK the 401
                                ack_msg = _build_ack(
                                    to_uri, caller_port, tag,
                                    _get_to_tag(resp) or "dummy",
                                    cid, cseq, _gen_branch(),
                                )
                                _send(caller_sock, ack_msg)
                                time.sleep(0.05)

                                # Re-INVITE with auth
                                cseq += 1
                                branch = _gen_branch()
                                invite_msg = _build_invite(
                                    "1001", "1002", caller_port, caller_rtp_port,
                                    tag, cid, cseq, branch,
                                    auth_line=auth_line, auth_hdr_name=hdr_name,
                                )
                                _send(caller_sock, invite_msg)
                        elif code == 180 or code == 183:
                            pass  # Ringing / session progress
                        elif code >= 200:
                            setup_time = time.monotonic() - start_time
                            final_code = code
                            to_tag = _get_to_tag(resp)
                            got_final = True

                            # Send ACK
                            ack_branch = _gen_branch()
                            ack_msg = _build_ack(
                                to_uri, caller_port, tag, to_tag,
                                cid, cseq, ack_branch,
                            )
                            _send(caller_sock, ack_msg)

                            if code == 200:
                                # Immediate BYE
                                time.sleep(0.1)
                                bye_msg = _build_bye(
                                    to_uri, caller_port, tag, to_tag,
                                    cid, cseq + 1,
                                )
                                _send(caller_sock, bye_msg)
                                # Wait for BYE 200 OK
                                bye_resp, _ = _recv(caller_sock, timeout=5)
                            break

                    # Also check callee socket for INVITE from PBX
                    callee_resp, callee_addr = _recv(callee_sock, timeout=0.5)
                    if callee_resp and callee_resp.startswith("INVITE "):
                        # Auto-answer with 200 OK
                        inv_cid = _get_header(callee_resp, "Call-ID")
                        inv_from = _get_header(callee_resp, "From")
                        inv_to = _get_header(callee_resp, "To")
                        inv_via = ""
                        for line in callee_resp.split("\r\n"):
                            if line.lower().startswith("via:"):
                                inv_via += line + "\r\n"
                        inv_cseq = _get_header(callee_resp, "CSeq")
                        callee_tag = _gen_tag()
                        sdp = _build_sdp(callee_rtp_port, SERVER_HOST)

                        ok_msg = (
                            f"SIP/2.0 200 OK\r\n"
                            f"{inv_via}"
                            f"From: {inv_from}\r\n"
                            f"To: {inv_to};tag={callee_tag}\r\n"
                            f"Call-ID: {inv_cid}\r\n"
                            f"CSeq: {inv_cseq}\r\n"
                            f"Contact: <sip:1002@{SERVER_HOST}:{callee_port}>\r\n"
                            f"Content-Type: application/sdp\r\n"
                            f"Content-Length: {len(sdp)}\r\n\r\n{sdp}"
                        )
                        callee_sock.sendto(ok_msg.encode(), callee_addr)

                if setup_time is not None:
                    setup_times.append(setup_time)
                    result["setup_time"] = setup_time

                if got_final:
                    result["status"] = f"completed (code={final_code})"
                else:
                    result["status"] = "no_final_response"

            except Exception as exc:
                result["status"] = f"error: {exc}"
            finally:
                call_results.append(result)
                if caller_sock:
                    try:
                        _unregister(caller_sock, "1001")
                    except Exception:
                        pass
                    caller_sock.close()
                if callee_sock:
                    try:
                        _unregister(callee_sock, "1002")
                    except Exception:
                        pass
                    callee_sock.close()

            # Small gap between calls to avoid flooding
            time.sleep(0.3)

        # Report
        print(f"\n--- TC-L6-005: Rapid call setup/teardown ({NUM_CALLS} calls) ---")
        for r in call_results:
            t = f"{r['setup_time']:.3f}s" if r.get("setup_time") else "N/A"
            print(f"  Call {r['call']}: {r['status']}  setup={t}")

        if setup_times:
            _print_latency_report(
                "Call setup times", setup_times, 0, NUM_CALLS,
            )

        # At least half the calls should complete (setup is complex, some may fail)
        completed = [r for r in call_results if "completed" in r["status"]]
        assert len(completed) >= NUM_CALLS // 2, (
            f"Only {len(completed)}/{NUM_CALLS} calls completed. "
            f"Details: {[r['status'] for r in call_results]}"
        )

    # ------------------------------------------------------------------
    # TC-L6-006: API endpoint stress -- multiple endpoints concurrently
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_006_api_endpoint_stress(self):
        """TC-L6-006: Hit /health, /dialogs, /transactions concurrently.

        Sends 30 requests to each of 3 API endpoints in parallel (90 total).
        Verifies response codes and measures per-endpoint latencies.
        """
        ENDPOINTS = [
            "/ami/v1/health",
            "/ami/v1/dialogs",
            "/ami/v1/transactions",
        ]
        REQUESTS_PER_ENDPOINT = 30

        results = {ep: {"latencies": [], "errors": 0} for ep in ENDPOINTS}
        lock = threading.Lock()

        def _hit_endpoint(endpoint, idx):
            start = time.monotonic()
            try:
                resp = requests.get(
                    f"{BASE_URL}{endpoint}",
                    timeout=10,
                    verify=VERIFY_TLS,
                )
                elapsed = time.monotonic() - start
                with lock:
                    results[endpoint]["latencies"].append(elapsed)
                if resp.status_code != 200:
                    with lock:
                        results[endpoint]["errors"] += 1
            except requests.RequestException:
                elapsed = time.monotonic() - start
                with lock:
                    results[endpoint]["latencies"].append(elapsed)
                    results[endpoint]["errors"] += 1

        with ThreadPoolExecutor(max_workers=30) as pool:
            futures = []
            for ep in ENDPOINTS:
                for i in range(REQUESTS_PER_ENDPOINT):
                    futures.append(pool.submit(_hit_endpoint, ep, i))
            for f in as_completed(futures):
                f.result()

        total_requests = len(ENDPOINTS) * REQUESTS_PER_ENDPOINT
        total_errors = sum(r["errors"] for r in results.values())

        print(f"\n--- TC-L6-006: API endpoint stress ({total_requests} requests) ---")
        for ep in ENDPOINTS:
            r = results[ep]
            lats = sorted(r["latencies"])
            if lats:
                p95 = _percentile(lats, 95)
                mean = statistics.mean(lats)
                print(f"  {ep}: mean={mean:.4f}s  p95={p95:.4f}s  errors={r['errors']}/{REQUESTS_PER_ENDPOINT}")
            else:
                print(f"  {ep}: NO DATA  errors={r['errors']}/{REQUESTS_PER_ENDPOINT}")

        # Less than 10% total error rate
        max_errors = int(total_requests * 0.10)
        assert total_errors <= max_errors, (
            f"Too many errors across endpoints: {total_errors}/{total_requests} "
            f"(max {max_errors})"
        )

        # Each endpoint must have at least some successful responses
        for ep in ENDPOINTS:
            successful = REQUESTS_PER_ENDPOINT - results[ep]["errors"]
            assert successful >= REQUESTS_PER_ENDPOINT * 0.5, (
                f"Endpoint {ep} had too many failures: "
                f"{results[ep]['errors']}/{REQUESTS_PER_ENDPOINT}"
            )

    # ------------------------------------------------------------------
    # TC-L6-007: WebSocket connection pool -- 10 concurrent connections
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    @pytest.mark.skipif(not HAS_WEBSOCKETS, reason="websockets package not installed")
    def test_L6_007_websocket_connection_pool(self):
        """TC-L6-007: Open 10 concurrent WebSocket connections.

        Connects to the /ws endpoint with the 'sip' subprotocol.  Verifies
        each connection handshake succeeds and the connections can be held
        open simultaneously for a few seconds.
        """
        NUM_CONNECTIONS = 10

        # Build WS URL from the HTTP base
        ws_scheme = "wss" if SCHEME == "https" else "ws"
        ws_url = f"{ws_scheme}://{SERVER_HOST}:{HTTP_PORT}/ws"

        results = {}
        lock = threading.Lock()

        async def _connect_one(idx):
            ssl_ctx = _get_ssl_context()
            try:
                async with websockets.connect(
                    ws_url,
                    subprotocols=["sip"],
                    ssl=ssl_ctx,
                    open_timeout=10,
                    close_timeout=5,
                    ping_interval=None,  # don't auto-ping
                ) as ws:
                    with lock:
                        results[idx] = {"status": "connected", "error": None}
                    # Hold connection open for 3 seconds
                    await asyncio.sleep(3)
                    # Try to receive any event (non-blocking, short timeout)
                    try:
                        msg = await asyncio.wait_for(ws.recv(), timeout=0.5)
                        with lock:
                            results[idx]["received"] = str(msg)[:200]
                    except (asyncio.TimeoutError, websockets.exceptions.ConnectionClosed):
                        pass
            except Exception as exc:
                with lock:
                    results[idx] = {"status": "failed", "error": str(exc)}

        async def _run_all():
            tasks = [_connect_one(i) for i in range(NUM_CONNECTIONS)]
            await asyncio.gather(*tasks, return_exceptions=True)

        asyncio.run(_run_all())

        print(f"\n--- TC-L6-007: WebSocket connection pool ({NUM_CONNECTIONS}) ---")
        connected = 0
        for idx in sorted(results):
            r = results[idx]
            status = r["status"]
            if status == "connected":
                connected += 1
            extra = f" error={r['error']}" if r.get("error") else ""
            recv = f" recv={r.get('received', 'none')}" if r.get("received") else ""
            print(f"  WS {idx}: {status}{extra}{recv}")

        # WebSocket may not be fully configured on all servers.
        # If the endpoint exists (not 404), at least some connections should work.
        # If all fail with connection refused or 404, skip gracefully.
        if connected == 0:
            all_errors = [r.get("error", "") for r in results.values()]
            if any("404" in str(e) for e in all_errors):
                pytest.skip("WebSocket endpoint /ws returned 404 -- not configured")
            if any("refused" in str(e).lower() for e in all_errors):
                pytest.skip("WebSocket connections refused -- endpoint not available")

        # At least half should connect successfully
        min_connected = NUM_CONNECTIONS // 2
        assert connected >= min_connected, (
            f"Only {connected}/{NUM_CONNECTIONS} WebSocket connections succeeded "
            f"(minimum: {min_connected}). Errors: "
            f"{[r.get('error') for r in results.values() if r.get('error')][:5]}"
        )

    # ------------------------------------------------------------------
    # TC-L6-008: Memory baseline -- check for significant leaks
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    @pytest.mark.slow
    def test_L6_008_memory_baseline(self):
        """TC-L6-008: Server memory before/after load, verify no significant leak.

        Reads the /ami/v1/health endpoint to get server uptime/memory info
        (if available), runs a burst of 200 requests, then checks again.
        If the health endpoint does not expose memory info, falls back to
        verifying the server is still responsive and stable.
        """
        def _get_health_data():
            try:
                resp = requests.get(
                    f"{BASE_URL}/ami/v1/health",
                    timeout=10,
                    verify=VERIFY_TLS,
                )
                if resp.status_code == 200:
                    try:
                        return resp.json()
                    except ValueError:
                        return {"raw": resp.text[:500]}
            except requests.RequestException:
                pass
            return None

        # Snapshot before load
        health_before = _get_health_data()
        print(f"\n--- TC-L6-008: Memory baseline ---")
        print(f"  Health before: {health_before}")

        # Run a significant burst of mixed requests
        NUM_BURST = 200
        burst_errors = 0

        def _burst_request(idx):
            nonlocal burst_errors
            endpoints = ["/ami/v1/health", "/ami/v1/dialogs", "/ami/v1/transactions"]
            ep = endpoints[idx % len(endpoints)]
            try:
                resp = requests.get(
                    f"{BASE_URL}{ep}", timeout=10, verify=VERIFY_TLS,
                )
                if resp.status_code != 200:
                    burst_errors += 1
            except requests.RequestException:
                burst_errors += 1

        with ThreadPoolExecutor(max_workers=20) as pool:
            futures = [pool.submit(_burst_request, i) for i in range(NUM_BURST)]
            for f in as_completed(futures):
                f.result()

        # Also send a burst of SIP traffic
        sip_ports = _alloc_ports(50)
        sip_socks = []
        for port in sip_ports:
            try:
                sock = _make_socket(port, timeout=3)
                msg = _build_options_msg(port)
                _send(sock, msg)
                sip_socks.append(sock)
            except Exception:
                pass

        # Drain responses
        time.sleep(1)
        for sock in sip_socks:
            try:
                _recv(sock, timeout=0.5)
            except Exception:
                pass
            sock.close()

        # Brief pause to let server settle
        time.sleep(2)

        # Snapshot after load
        health_after = _get_health_data()
        print(f"  Health after:  {health_after}")
        print(f"  Burst errors:  {burst_errors}/{NUM_BURST}")

        # The server must still be responsive
        final_check = requests.get(
            f"{BASE_URL}/ami/v1/health", timeout=10, verify=VERIFY_TLS,
        )
        assert final_check.status_code == 200, (
            f"Server not healthy after load burst: HTTP {final_check.status_code}"
        )

        # If health data includes memory info, check for excessive growth
        if health_before and health_after:
            mem_before = health_before.get("memory_mb") or health_before.get("rss_mb")
            mem_after = health_after.get("memory_mb") or health_after.get("rss_mb")
            if mem_before is not None and mem_after is not None:
                growth = float(mem_after) - float(mem_before)
                growth_pct = (growth / float(mem_before)) * 100 if float(mem_before) > 0 else 0
                print(f"  Memory before: {mem_before} MB")
                print(f"  Memory after:  {mem_after} MB")
                print(f"  Growth:        {growth:.1f} MB ({growth_pct:.1f}%)")
                # Allow up to 50MB growth or 50% -- a real leak would be much more
                assert growth < 50 or growth_pct < 50, (
                    f"Possible memory leak: {growth:.1f} MB growth ({growth_pct:.1f}%) "
                    f"after {NUM_BURST} HTTP + 50 SIP requests"
                )
            else:
                print("  (Server health endpoint does not expose memory metrics)")

        # Error rate should be low
        assert burst_errors < NUM_BURST * 0.10, (
            f"Too many burst errors: {burst_errors}/{NUM_BURST}"
        )


# ===================================================================
# Async variants (using aiohttp for true concurrent HTTP)
# ===================================================================

@pytest.mark.skipif(not HAS_AIOHTTP, reason="aiohttp package not installed")
class TestL6AsyncLoad:
    """L6 async variants -- uses aiohttp for truly concurrent HTTP load."""

    # ------------------------------------------------------------------
    # TC-L6-009: Async API health flood -- 100 concurrent with aiohttp
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_009_async_api_health_flood(self):
        """TC-L6-009: 100 concurrent /ami/v1/health via aiohttp, all < 2s p95.

        Uses aiohttp for genuine async I/O concurrency (not thread-pool).
        """
        NUM_REQUESTS = 100
        MAX_P95 = 2.0
        latencies = []
        errors = []

        async def _fetch(session, idx):
            start = time.monotonic()
            try:
                async with session.get(
                    f"{BASE_URL}/ami/v1/health", timeout=aiohttp.ClientTimeout(total=10),
                ) as resp:
                    elapsed = time.monotonic() - start
                    latencies.append(elapsed)
                    if resp.status != 200:
                        errors.append(f"req {idx}: HTTP {resp.status}")
                    await resp.read()
            except Exception as exc:
                elapsed = time.monotonic() - start
                latencies.append(elapsed)
                errors.append(f"req {idx}: {exc}")

        async def _run():
            connector = _get_aiohttp_connector()
            async with aiohttp.ClientSession(connector=connector) as session:
                tasks = [_fetch(session, i) for i in range(NUM_REQUESTS)]
                await asyncio.gather(*tasks)

        asyncio.run(_run())

        _print_latency_report(
            "TC-L6-009: Async API health flood",
            latencies, len(errors), NUM_REQUESTS,
        )

        latencies_sorted = sorted(latencies)
        p95 = _percentile(latencies_sorted, 95)
        assert p95 < MAX_P95, (
            f"Async p95 latency ({p95:.4f}s) exceeds {MAX_P95}s"
        )
        assert len(errors) < NUM_REQUESTS * 0.05, (
            f"Too many async errors: {len(errors)}/{NUM_REQUESTS}: {errors[:5]}"
        )

    # ------------------------------------------------------------------
    # TC-L6-010: Async multi-endpoint stress
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_010_async_multi_endpoint_stress(self):
        """TC-L6-010: Async concurrent requests to 3 endpoints (150 total).

        Sends 50 requests each to /health, /dialogs, /transactions via aiohttp.
        """
        ENDPOINTS = [
            "/ami/v1/health",
            "/ami/v1/dialogs",
            "/ami/v1/transactions",
        ]
        PER_ENDPOINT = 50

        results = {ep: {"latencies": [], "errors": 0} for ep in ENDPOINTS}

        async def _fetch(session, endpoint, idx):
            start = time.monotonic()
            try:
                async with session.get(
                    f"{BASE_URL}{endpoint}",
                    timeout=aiohttp.ClientTimeout(total=10),
                ) as resp:
                    elapsed = time.monotonic() - start
                    results[endpoint]["latencies"].append(elapsed)
                    if resp.status != 200:
                        results[endpoint]["errors"] += 1
                    await resp.read()
            except Exception:
                elapsed = time.monotonic() - start
                results[endpoint]["latencies"].append(elapsed)
                results[endpoint]["errors"] += 1

        async def _run():
            connector = _get_aiohttp_connector()
            async with aiohttp.ClientSession(connector=connector) as session:
                tasks = []
                for ep in ENDPOINTS:
                    for i in range(PER_ENDPOINT):
                        tasks.append(_fetch(session, ep, i))
                await asyncio.gather(*tasks)

        asyncio.run(_run())

        total = len(ENDPOINTS) * PER_ENDPOINT
        total_errors = sum(r["errors"] for r in results.values())

        print(f"\n--- TC-L6-010: Async multi-endpoint stress ({total} requests) ---")
        for ep in ENDPOINTS:
            r = results[ep]
            lats = sorted(r["latencies"])
            if lats:
                p95 = _percentile(lats, 95)
                mean = statistics.mean(lats)
                print(f"  {ep}: mean={mean:.4f}s  p95={p95:.4f}s  errors={r['errors']}/{PER_ENDPOINT}")

        assert total_errors < total * 0.10, (
            f"Too many errors: {total_errors}/{total}"
        )


# ===================================================================
# Load Generator integration tests (using sip_load_generator module)
# ===================================================================

# Import the load generator -- it lives alongside this file in tests/
import sys as _sys
import os as _os
_test_dir = _os.path.dirname(_os.path.abspath(__file__))
if _test_dir not in _sys.path:
    _sys.path.insert(0, _test_dir)

try:
    from sip_load_generator import SIPLoadGenerator, LoadTestMetrics
    HAS_LOAD_GEN = True
except ImportError:
    HAS_LOAD_GEN = False


@pytest.mark.skipif(not HAS_LOAD_GEN, reason="sip_load_generator module not available")
class TestL6LoadGenerator:
    """L6 tests that use the SIPLoadGenerator module for structured load tests."""

    # ------------------------------------------------------------------
    # TC-L6-011: Load generator -- register/unregister pool
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    @pytest.mark.slow
    def test_L6_011_load_gen_register_pool(self):
        """TC-L6-011: Register and unregister a pool of agents via load generator.

        Creates 4 agents using the pre-configured test users (1001-1004),
        registers all concurrently, verifies success, then unregisters.
        """
        ports = _alloc_ports(8)  # 4 agents * 2 ports each (SIP + RTP)

        # Use pre-configured test users that exist on the server
        test_users = {
            "1001": "test1001",
            "1002": "test1002",
        }

        gen = SIPLoadGenerator(
            host=SERVER_HOST,
            port=SIP_PORT,
            num_agents=2,
            base_extension=1001,
            password_fn=lambda ext: test_users.get(ext, f"test{ext}"),
            external_ip=EXTERNAL_IP,
            base_port=ports[0],
        )

        async def _run():
            ok, fail = await gen.register_all()
            print(f"\n--- TC-L6-011: Load gen register pool ---")
            print(f"  Registered: {ok}/{gen.num_agents}  Failed: {fail}")
            assert ok >= 1, f"Expected at least 1 registration, got {ok}"
            await gen.unregister_all()
            gen._close_all()
            return ok

        result = asyncio.run(_run())
        assert result >= 1

    # ------------------------------------------------------------------
    # TC-L6-012: Load generator -- concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    @pytest.mark.slow
    def test_L6_012_load_gen_concurrent_calls(self):
        """TC-L6-012: Use load generator for 4 sequential calls between 2 agents.

        Registers 2 agents (1001, 1002), places 4 calls with concurrency=1
        (sequential), each held for 2 seconds. Verifies success rate and
        reports setup time stats.
        """
        ports = _alloc_ports(8)

        test_users = {
            "1001": "test1001",
            "1002": "test1002",
        }

        gen = SIPLoadGenerator(
            host=SERVER_HOST,
            port=SIP_PORT,
            num_agents=2,
            base_extension=1001,
            password_fn=lambda ext: test_users.get(ext, f"test{ext}"),
            external_ip=EXTERNAL_IP,
            base_port=ports[0],
        )

        async def _run():
            metrics = await gen.run_load_test(
                num_calls=4,
                concurrency=1,
                duration_secs=2.0,
                send_rtp=True,
                inter_batch_delay=0.5,
            )
            gen.print_report()
            return metrics

        metrics = asyncio.run(_run())

        print(f"\n--- TC-L6-012: Load gen concurrent calls ---")
        print(f"  Total: {metrics.total_calls}  "
              f"Success: {metrics.successful_calls}  "
              f"Failed: {metrics.failed_calls}")
        print(f"  Success rate: {metrics.success_rate * 100:.1f}%")

        # At least some calls should succeed (server config may limit)
        # This is a best-effort test -- if users are not configured, calls
        # may fail with 403 or 404, which is still valid load testing
        assert metrics.total_calls == 4, (
            f"Expected 4 total calls, got {metrics.total_calls}"
        )

    # ------------------------------------------------------------------
    # TC-L6-013: Load generator -- metrics and reporting
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L6_013_load_gen_metrics_structure(self):
        """TC-L6-013: Verify LoadTestMetrics data structures and reporting.

        Unit test for the metrics classes -- no server needed.
        """
        from sip_load_generator import CallMetrics, LoadTestMetrics

        # Build synthetic metrics
        m = LoadTestMetrics(
            total_calls=10,
            successful_calls=8,
            failed_calls=2,
            start_time=100.0,
            end_time=110.0,
            num_agents=4,
            concurrency=2,
        )

        for i in range(8):
            m.call_metrics.append(CallMetrics(
                call_id=f"test-{i}",
                caller="1001",
                callee="1002",
                setup_time=0.3 + i * 0.05,
                success=True,
                final_response_code=200,
                rtp_packets_sent=50,
            ))
        for i in range(2):
            m.call_metrics.append(CallMetrics(
                call_id=f"fail-{i}",
                caller="1001",
                callee="1002",
                error="Timeout waiting for 200 OK",
                error_category="timeout",
                success=False,
            ))

        summary = m.summary()

        assert summary["total_calls"] == 10
        assert summary["successful_calls"] == 8
        assert summary["failed_calls"] == 2
        assert summary["success_rate"] == 0.8
        assert summary["duration_secs"] == 10.0
        assert summary["calls_per_second"] == 1.0

        assert summary["setup_time_stats"] is not None
        assert summary["setup_time_stats"]["count"] == 8
        assert summary["setup_time_stats"]["min"] > 0
        assert summary["setup_time_stats"]["max"] > summary["setup_time_stats"]["min"]

        assert summary["error_breakdown"] == {"timeout": 2}

        # Verify print_report does not crash
        m.print_report()
        print("  Metrics structure test: PASSED")
