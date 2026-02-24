"""
L7 Failover & Resilience Tests -- Service stability under adverse conditions.

These tests verify that RustPBX remains stable and recovers gracefully when
subjected to load bursts, malformed input, abrupt disconnections, and
concurrent authentication storms.  No Docker access is required -- all tests
exercise the live SIP and HTTP/API interfaces directly.

Each test is self-contained with its own UDP socket(s) and cleanup logic.

Expected execution time: < 180 seconds for the full suite.

Usage:
  python -m pytest tests/test_L7_failover.py -v

Environment variables (all optional, sensible defaults for Linode server):
  RUSTPBX_HOST          SIP / API server IP     (default: 127.0.0.1)
  RUSTPBX_SIP_PORT      SIP port                (default: 5060)
  RUSTPBX_HTTP_PORT     HTTPS port              (default: 8443)
  RUSTPBX_SCHEME        http or https           (default: https)
  RUSTPBX_EXTERNAL_IP   Public IP for SIP URIs  (default: same as HOST)
"""

import hashlib
import os
import random
import re
import selectors
import socket
import string
import threading
import time
import uuid

import pytest
import requests
import urllib3

# Suppress InsecureRequestWarning for self-signed TLS certs
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", SERVER_HOST)
VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in ("1", "true", "yes")

BASE_URL = f"{SCHEME}://{SERVER_HOST}:{HTTP_PORT}"
AMI_BASE = f"{BASE_URL}/ami/v1"

TEST_USERS = {
    "1001": "test1001",
    "1002": "test1002",
}

# Port range for test UA sockets -- each test picks a unique port to avoid
# collisions when running in parallel or when the OS has TIME_WAIT sockets.
_PORT_BASE = 18100
_port_counter = 0


def _alloc_port():
    """Return a unique local port for each test socket."""
    global _port_counter
    _port_counter += 1
    return _PORT_BASE + _port_counter


# ---------------------------------------------------------------------------
# SIP helpers -- thin wrappers over raw UDP
# ---------------------------------------------------------------------------

def _gen_branch():
    return "z9hG4bK" + "".join(random.choices(string.ascii_lowercase + string.digits, k=12))


def _gen_tag():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=8))


def _gen_callid():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=16)) + "@l7test"


def _md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()


def _compute_digest(username, realm, password, method, uri, nonce):
    ha1 = _md5hex(f"{username}:{realm}:{password}")
    ha2 = _md5hex(f"{method}:{uri}")
    return _md5hex(f"{ha1}:{nonce}:{ha2}")


def _get_response_code(data):
    """Extract the SIP response status code from the first line."""
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0


def _get_header(data, name):
    """Return the value of the first occurrence of the named header."""
    for line in data.split("\r\n"):
        if line.lower().startswith(name.lower() + ":"):
            return line.split(":", 1)[1].strip()
    return ""


def _get_to_tag(data):
    to_hdr = _get_header(data, "To")
    m = re.search(r"tag=([^\s;>]+)", to_hdr)
    return m.group(1) if m else ""


def _get_all_via(data):
    vias = []
    for line in data.split("\r\n"):
        if line.lower().startswith("via:"):
            vias.append(line)
    return vias


def _parse_www_authenticate(header_line):
    realm = re.search(r'realm="([^"]*)"', header_line)
    nonce = re.search(r'nonce="([^"]*)"', header_line)
    return (realm.group(1) if realm else ""), (nonce.group(1) if nonce else "")


def _make_socket(port=0):
    """Create a bound UDP socket.  Port 0 = OS-assigned ephemeral port."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", port))
    s.settimeout(5)
    return s


def _send(sock, msg):
    """Send a SIP message (str or bytes) to the server."""
    if isinstance(msg, str):
        msg = msg.encode()
    sock.sendto(msg, (SERVER_HOST, SIP_PORT))


def _recv(sock, timeout=5):
    """Receive a single SIP message.  Returns (str, addr) or (None, None)."""
    sock.settimeout(timeout)
    try:
        data, addr = sock.recvfrom(65535)
        return data.decode(errors="replace"), addr
    except socket.timeout:
        return None, None


def _recv_all(sock, timeout=5, max_messages=50):
    """Receive all SIP messages until timeout.  Returns list of (str, addr)."""
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
    """Return the local port of an already-bound socket."""
    return sock.getsockname()[1]


def _build_options(sock, from_user="l7probe", call_id=None):
    """Build a SIP OPTIONS message using the socket's local port."""
    local_port = _local_port(sock)
    cid = call_id or _gen_callid()
    branch = _gen_branch()
    tag = _gen_tag()
    return (
        f"OPTIONS sip:{SERVER_HOST}:{SIP_PORT} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
        f"From: <sip:{from_user}@{SERVER_HOST}>;tag={tag}\r\n"
        f"To: <sip:{SERVER_HOST}:{SIP_PORT}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: 1 OPTIONS\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n\r\n"
    )


def _build_register(sock, username, expires=60, call_id=None, cseq=1):
    """Build an unauthenticated SIP REGISTER."""
    local_port = _local_port(sock)
    cid = call_id or _gen_callid()
    branch = _gen_branch()
    tag = _gen_tag()
    from_uri = f"sip:{username}@{EXTERNAL_IP}"
    contact = f"<sip:{username}@{SERVER_HOST}:{local_port};transport=udp>"
    return (
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


def _register_with_auth(sock, username, password, expires=60, cseq_start=1):
    """Perform a full REGISTER + digest-auth flow.

    Returns (response_code, response_text) of the final response.
    """
    local_port = _local_port(sock)
    tag = _gen_tag()
    cid = f"l7reg-{_gen_callid()}"
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
    """Send an unauthenticated REGISTER with Expires: 0 (best-effort cleanup)."""
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
        "s=L7Test\r\n"
        f"c=IN IP4 {local_ip}\r\n"
        "t=0 0\r\n"
        f"m=audio {rtp_port} RTP/AVP 0 101\r\n"
        "a=rtpmap:0 PCMU/8000\r\n"
        "a=rtpmap:101 telephone-event/8000\r\n"
        "a=fmtp:101 0-16\r\n"
        "a=sendrecv\r\n"
    )


def _establish_call(caller_sock, callee_sock, caller_ext="1001", callee_ext="1002"):
    """Register both users, set up a call via INVITE, and return call state.

    Returns a dict with keys:
        call_id, from_uri, to_uri, local_tag, to_tag, cseq, established
    or None if the call could not be established.
    """
    caller_port = _local_port(caller_sock)
    callee_port = _local_port(callee_sock)
    caller_pw = TEST_USERS[caller_ext]
    callee_pw = TEST_USERS[callee_ext]

    # Register both
    c1, _ = _register_with_auth(caller_sock, caller_ext, caller_pw)
    if c1 != 200:
        return None
    c2, _ = _register_with_auth(callee_sock, callee_ext, callee_pw)
    if c2 != 200:
        return None

    from_uri = f"sip:{caller_ext}@{EXTERNAL_IP}"
    to_uri = f"sip:{callee_ext}@{SERVER_HOST}"
    local_tag = _gen_tag()
    call_id = _gen_callid()
    contact = f"<sip:{caller_ext}@{SERVER_HOST}:{caller_port}>"
    cseq = 1
    rtp_port = _alloc_port() + 2000
    sdp_body = _build_sdp(rtp_port)

    def make_invite(br, cs, auth_hdr=None):
        auth = f"{auth_hdr}\r\n" if auth_hdr else ""
        return (
            f"INVITE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={br};rport\r\n"
            f"From: <{from_uri}>;tag={local_tag}\r\n"
            f"To: <{to_uri}>\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: {cs} INVITE\r\n"
            f"Contact: {contact}\r\n"
            f"{auth}"
            f"Max-Forwards: 70\r\n"
            f"Content-Type: application/sdp\r\n"
            f"Content-Length: {len(sdp_body)}\r\n\r\n"
            f"{sdp_body}"
        )

    # Send initial INVITE
    branch = _gen_branch()
    _send(caller_sock, make_invite(branch, cseq))

    sel = selectors.DefaultSelector()
    sel.register(caller_sock, selectors.EVENT_READ, "caller")
    sel.register(callee_sock, selectors.EVENT_READ, "callee")

    to_tag = ""
    auth_done = False
    established = False
    deadline = time.time() + 15

    while time.time() < deadline and not established:
        events = sel.select(timeout=1.0)
        for key, _ in events:
            data, addr = key.fileobj.recvfrom(65535)
            msg = data.decode(errors="replace")

            if key.data == "callee":
                if msg.startswith("INVITE "):
                    # Auto-answer: 200 OK with SDP
                    cid_h = _get_header(msg, "Call-ID")
                    from_h = _get_header(msg, "From")
                    to_h = _get_header(msg, "To")
                    cseq_h = _get_header(msg, "CSeq")
                    vias = _get_all_via(msg)
                    via_block = "\r\n".join(vias)
                    callee_tag = _gen_tag()
                    to_with_tag = to_h + f";tag={callee_tag}" if "tag=" not in to_h else to_h
                    callee_contact = f"<sip:{callee_ext}@{SERVER_HOST}:{callee_port}>"
                    callee_sdp = _build_sdp(rtp_port + 2)
                    ok = (
                        f"SIP/2.0 200 OK\r\n"
                        f"{via_block}\r\n"
                        f"From: {from_h}\r\n"
                        f"To: {to_with_tag}\r\n"
                        f"Call-ID: {cid_h}\r\n"
                        f"CSeq: {cseq_h}\r\n"
                        f"Contact: {callee_contact}\r\n"
                        f"Content-Type: application/sdp\r\n"
                        f"Content-Length: {len(callee_sdp)}\r\n\r\n"
                        f"{callee_sdp}"
                    )
                    callee_sock.sendto(ok.encode(), addr)
                continue

            # Caller responses
            if msg.startswith("SIP/2.0"):
                code = _get_response_code(msg)
                if code in (401, 407) and not auth_done:
                    auth_done = True
                    # ACK the challenge
                    ack = (
                        f"ACK {to_uri} SIP/2.0\r\n"
                        f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={_gen_branch()};rport\r\n"
                        f"From: <{from_uri}>;tag={local_tag}\r\n"
                        f"To: <{to_uri}>\r\n"
                        f"Call-ID: {call_id}\r\n"
                        f"CSeq: {cseq} ACK\r\n"
                        f"Max-Forwards: 70\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    _send(caller_sock, ack)

                    auth_hdr = ""
                    for line in msg.split("\r\n"):
                        low = line.lower()
                        if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                            auth_hdr = line.split(":", 1)[1].strip()
                            break
                    realm, nonce = _parse_www_authenticate(auth_hdr)
                    realm = realm or EXTERNAL_IP
                    digest_resp = _compute_digest(
                        caller_ext, realm, caller_pw, "INVITE", to_uri, nonce
                    )
                    hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
                    auth_line = (
                        f'{hdr_name}: Digest username="{caller_ext}", realm="{realm}", '
                        f'nonce="{nonce}", uri="{to_uri}", response="{digest_resp}", algorithm=MD5'
                    )
                    cseq += 1
                    branch = _gen_branch()
                    _send(caller_sock, make_invite(branch, cseq, auth_line))
                elif code == 200:
                    to_tag = _get_to_tag(msg)
                    # Send ACK
                    to_field = f"<{to_uri}>;tag={to_tag}" if to_tag else f"<{to_uri}>"
                    ack = (
                        f"ACK {to_uri} SIP/2.0\r\n"
                        f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={_gen_branch()};rport\r\n"
                        f"From: <{from_uri}>;tag={local_tag}\r\n"
                        f"To: {to_field}\r\n"
                        f"Call-ID: {call_id}\r\n"
                        f"CSeq: {cseq} ACK\r\n"
                        f"Max-Forwards: 70\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    _send(caller_sock, ack)
                    established = True
                    break
                elif code >= 400:
                    # ACK error response
                    ack = (
                        f"ACK {to_uri} SIP/2.0\r\n"
                        f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={_gen_branch()};rport\r\n"
                        f"From: <{from_uri}>;tag={local_tag}\r\n"
                        f"To: <{to_uri}>\r\n"
                        f"Call-ID: {call_id}\r\n"
                        f"CSeq: {cseq} ACK\r\n"
                        f"Max-Forwards: 70\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    _send(caller_sock, ack)
                    break

    sel.unregister(caller_sock)
    sel.unregister(callee_sock)
    sel.close()

    if not established:
        return None

    return {
        "call_id": call_id,
        "from_uri": from_uri,
        "to_uri": to_uri,
        "local_tag": local_tag,
        "to_tag": to_tag,
        "cseq": cseq,
        "caller_port": caller_port,
        "callee_port": callee_port,
        "established": True,
    }


def _send_bye(sock, call_state):
    """Send BYE for an established call and return the response code."""
    to_field = (
        f"<{call_state['to_uri']}>;tag={call_state['to_tag']}"
        if call_state['to_tag']
        else f"<{call_state['to_uri']}>"
    )
    bye_cseq = call_state["cseq"] + 1
    bye = (
        f"BYE {call_state['to_uri']} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER_HOST}:{call_state['caller_port']};branch={_gen_branch()};rport\r\n"
        f"From: <{call_state['from_uri']}>;tag={call_state['local_tag']}\r\n"
        f"To: {to_field}\r\n"
        f"Call-ID: {call_state['call_id']}\r\n"
        f"CSeq: {bye_cseq} BYE\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    _send(sock, bye)
    resp, _ = _recv(sock, timeout=5)
    if resp is None:
        return 0
    return _get_response_code(resp)


# ---------------------------------------------------------------------------
# Pytest fixtures
# ---------------------------------------------------------------------------

@pytest.fixture()
def sip_sock():
    """Yield a fresh UDP socket bound to a unique port; closes after test."""
    port = _alloc_port()
    sock = _make_socket(port)
    yield sock
    sock.close()


@pytest.fixture()
def sip_sock_pair():
    """Yield two fresh UDP sockets (for caller + callee scenarios)."""
    s1 = _make_socket(_alloc_port())
    s2 = _make_socket(_alloc_port())
    yield s1, s2
    s1.close()
    s2.close()


# ===================================================================
# L7 Failover & Resilience Test Suite
# ===================================================================

class TestL7Failover:
    """L7: Failover, resilience, and recovery tests."""

    # ------------------------------------------------------------------
    # TC-L7-001: Server health after load burst
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    def test_L7_001_health_after_options_burst(self, sip_sock):
        """TC-L7-001: Server remains healthy after 50 rapid OPTIONS requests.

        Sends 50 SIP OPTIONS messages in rapid succession over UDP, then
        verifies the HTTP health endpoint still returns 200 OK.  This
        checks that a burst of stateless SIP traffic does not degrade the
        server or exhaust its resources.
        """
        NUM_OPTIONS = 50
        responses_received = 0

        # Send 50 OPTIONS as fast as possible
        for i in range(NUM_OPTIONS):
            msg = _build_options(sip_sock, from_user=f"burst{i}")
            _send(sip_sock, msg)

        # Drain all responses (don't care about individual results)
        all_msgs = _recv_all(sip_sock, timeout=10, max_messages=NUM_OPTIONS)
        for msg_text, _ in all_msgs:
            if _get_response_code(msg_text) > 0:
                responses_received += 1

        # At least some OPTIONS should have gotten responses
        assert responses_received > 0, (
            f"No responses received for {NUM_OPTIONS} OPTIONS requests"
        )

        # The critical check: HTTP health must still be OK
        time.sleep(1)  # brief settling time
        try:
            resp = requests.get(
                f"{AMI_BASE}/health", timeout=10, verify=VERIFY_TLS
            )
            assert resp.status_code == 200, (
                f"Health endpoint returned {resp.status_code} after "
                f"OPTIONS burst ({responses_received}/{NUM_OPTIONS} SIP responses)"
            )
        except requests.ConnectionError as exc:
            pytest.fail(
                f"Health endpoint unreachable after OPTIONS burst: {exc}"
            )

    # ------------------------------------------------------------------
    # TC-L7-002: Graceful call cleanup after abrupt drop
    # ------------------------------------------------------------------
    @pytest.mark.timeout(45)
    def test_L7_002_graceful_call_cleanup_after_drop(self, sip_sock_pair):
        """TC-L7-002: Server cleans up dialog when one side drops abruptly.

        Establishes a call between 1001 and 1002, then closes the callee
        socket without sending BYE.  Verifies the server eventually
        cleans up the orphaned dialog (checks /ami/v1/dialogs).
        """
        caller_sock, callee_sock = sip_sock_pair

        call_state = _establish_call(caller_sock, callee_sock)
        if call_state is None:
            pytest.skip("Could not establish call for cleanup test")

        # Abruptly close the callee socket (simulating a crash)
        callee_sock.close()

        # Wait a moment for the server to notice the missing endpoint
        time.sleep(2)

        # Now send BYE from caller to trigger cleanup
        bye_code = _send_bye(caller_sock, call_state)
        # Accept 200 (normal), 481 (dialog already gone), or 408 (timeout)
        assert bye_code in (200, 408, 481, 0), (
            f"Unexpected BYE response code: {bye_code}"
        )

        # Give the server time to process cleanup
        time.sleep(3)

        # Verify the dialog is no longer listed
        try:
            resp = requests.get(
                f"{AMI_BASE}/dialogs", timeout=10, verify=VERIFY_TLS
            )
            if resp.status_code == 200:
                dialogs = resp.json()
                if isinstance(dialogs, list):
                    stale = [
                        d for d in dialogs
                        if isinstance(d, dict)
                        and d.get("call_id") == call_state["call_id"]
                    ]
                    assert len(stale) == 0, (
                        f"Orphaned dialog still present after BYE + drop: {stale}"
                    )
        except requests.ConnectionError:
            pass  # API might not be available; SIP cleanup is the primary test

        # Cleanup caller registration
        _unregister(caller_sock, "1001")

    # ------------------------------------------------------------------
    # TC-L7-003: Invalid SIP message handling
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L7_003_invalid_sip_messages(self, sip_sock):
        """TC-L7-003: Server handles malformed SIP messages without crashing.

        Sends various malformed SIP messages and verifies the server
        is still responsive afterwards.  A well-behaved SIP stack should
        silently discard unparseable messages (RFC 3261 section 7.3.1).
        """
        malformed_messages = [
            # Completely invalid (not SIP at all)
            b"This is not a SIP message at all\r\n\r\n",
            # Missing request-line method
            b"SIP/2.0\r\nVia: SIP/2.0/UDP 127.0.0.1\r\n\r\n",
            # Truncated headers
            b"INVITE sip:foo@bar SIP/2.0\r\nVia: \r\n\r\n",
            # Missing CRLF line endings (just LF)
            b"OPTIONS sip:test SIP/2.0\nVia: SIP/2.0/UDP 127.0.0.1\n\n",
            # Empty message
            b"",
            # Just CRLF
            b"\r\n\r\n",
            # Binary garbage
            bytes(range(256)),
            # Valid request line but no headers at all
            b"REGISTER sip:example.com SIP/2.0\r\n\r\n",
            # Duplicate Content-Length headers with conflicting values
            (
                b"OPTIONS sip:test SIP/2.0\r\n"
                b"Via: SIP/2.0/UDP 127.0.0.1;branch=z9hG4bK-bad\r\n"
                b"Content-Length: 100\r\n"
                b"Content-Length: 0\r\n"
                b"\r\n"
            ),
            # Invalid SIP version
            b"OPTIONS sip:test SIP/3.0\r\nVia: SIP/2.0/UDP 127.0.0.1\r\n\r\n",
        ]

        for i, bad_msg in enumerate(malformed_messages):
            if len(bad_msg) > 0:
                _send(sip_sock, bad_msg)
            # Small delay to avoid flooding
            time.sleep(0.05)

        # Drain any responses the server might have sent
        _recv_all(sip_sock, timeout=2)

        # The critical check: server must still respond to a valid OPTIONS
        valid_options = _build_options(sip_sock, from_user="postmalform")
        _send(sip_sock, valid_options)
        resp, _ = _recv(sip_sock, timeout=5)
        assert resp is not None, (
            "Server stopped responding after receiving malformed SIP messages"
        )
        code = _get_response_code(resp)
        assert code == 200, (
            f"Expected 200 OK for valid OPTIONS after malformed burst, got {code}"
        )

    # ------------------------------------------------------------------
    # TC-L7-004: Oversized SIP message handling
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L7_004_oversized_sip_message(self, sip_sock):
        """TC-L7-004: Server handles an oversized SIP message (>64KB) gracefully.

        Sends a SIP message with a body exceeding 64KB.  The server should
        either reject it (e.g., 413 Request Entity Too Large) or silently
        discard it, but must not crash.  UDP datagrams larger than ~65KB
        will be truncated by the OS, so we test at the UDP MTU boundary.
        """
        local_port = _local_port(sip_sock)
        branch = _gen_branch()
        tag = _gen_tag()
        cid = _gen_callid()

        # Build a message with a very large body (padded SDP)
        # UDP max is ~65507 bytes; the OS may truncate but the server
        # must handle whatever arrives.
        large_body = "v=0\r\n" + ("a=padding:" + "X" * 1000 + "\r\n") * 60
        # This creates a body of roughly 60KB

        oversized_msg = (
            f"OPTIONS sip:{SERVER_HOST}:{SIP_PORT} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
            f"From: <sip:oversized@{SERVER_HOST}>;tag={tag}\r\n"
            f"To: <sip:{SERVER_HOST}:{SIP_PORT}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 OPTIONS\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Type: application/sdp\r\n"
            f"Content-Length: {len(large_body)}\r\n\r\n"
            f"{large_body}"
        )

        try:
            _send(sip_sock, oversized_msg)
        except OSError:
            # OS may refuse to send a datagram this large -- that's fine
            pass

        # Wait briefly for any response
        _recv_all(sip_sock, timeout=2)

        # Critical check: server must still respond to a normal OPTIONS
        normal_options = _build_options(sip_sock, from_user="postoversize")
        _send(sip_sock, normal_options)
        resp, _ = _recv(sip_sock, timeout=5)
        assert resp is not None, (
            "Server stopped responding after receiving oversized SIP message"
        )
        code = _get_response_code(resp)
        assert code == 200, (
            f"Expected 200 OK after oversized message test, got {code}"
        )

    # ------------------------------------------------------------------
    # TC-L7-005: Rapid register/unregister cycles
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    def test_L7_005_rapid_register_unregister_cycles(self, sip_sock):
        """TC-L7-005: 20 rapid register/unregister cycles remain stable.

        Rapidly registers and unregisters user 1001 twenty times.
        Verifies that the final registration attempt still succeeds
        and the server is not confused by the rapid state changes.
        """
        NUM_CYCLES = 20
        success_count = 0
        last_code = 0

        for i in range(NUM_CYCLES):
            # Register with auth
            code, _ = _register_with_auth(
                sip_sock, "1001", "test1001", expires=30
            )
            if code == 200:
                success_count += 1
            last_code = code

            # Immediately unregister (best-effort, no auth)
            _unregister(sip_sock, "1001")

            # Tiny delay between cycles to avoid overwhelming the socket buffer
            time.sleep(0.1)

        # At least half the cycles should have succeeded (network jitter allowed)
        assert success_count >= NUM_CYCLES // 2, (
            f"Only {success_count}/{NUM_CYCLES} register cycles succeeded"
        )

        # Final verification: register one more time and ensure it works
        time.sleep(1)
        final_code, _ = _register_with_auth(
            sip_sock, "1001", "test1001", expires=60
        )
        assert final_code == 200, (
            f"Final registration after {NUM_CYCLES} cycles failed with {final_code}"
        )

        # Cleanup
        _unregister(sip_sock, "1001")

    # ------------------------------------------------------------------
    # TC-L7-006: Connection recovery after socket close/reopen
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L7_006_connection_recovery_after_socket_reopen(self):
        """TC-L7-006: Re-registration works after closing and reopening the UDP socket.

        Registers user 1001, closes the socket entirely, opens a new socket
        on a different port, and re-registers.  Verifies the server accepts
        the new binding without issues.
        """
        # First socket: register
        port1 = _alloc_port()
        sock1 = _make_socket(port1)
        try:
            code1, _ = _register_with_auth(sock1, "1001", "test1001", expires=60)
            assert code1 == 200, (
                f"Initial registration failed with {code1}"
            )
        finally:
            sock1.close()

        # Brief pause to simulate network interruption
        time.sleep(1)

        # Second socket on a different port: re-register
        port2 = _alloc_port()
        sock2 = _make_socket(port2)
        try:
            code2, _ = _register_with_auth(sock2, "1001", "test1001", expires=60)
            assert code2 == 200, (
                f"Re-registration on new socket failed with {code2}. "
                f"Server may have stale binding from port {port1}."
            )

            # Verify the new binding works by sending OPTIONS
            options = _build_options(sock2, from_user="1001")
            _send(sock2, options)
            resp, _ = _recv(sock2, timeout=5)
            assert resp is not None, (
                "Server did not respond to OPTIONS after re-registration"
            )
        finally:
            _unregister(sock2, "1001")
            sock2.close()

    # ------------------------------------------------------------------
    # TC-L7-007: Concurrent auth attempts with wrong password
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L7_007_concurrent_bad_auth_attempts(self):
        """TC-L7-007: 10 simultaneous REGISTER with wrong password get proper 401s.

        Sends 10 REGISTER requests concurrently, each with an incorrect
        password.  All should receive 401 or 403 (never 200).  The server
        must not leak auth state or crash under concurrent auth pressure.
        """
        NUM_ATTEMPTS = 10
        results = [None] * NUM_ATTEMPTS
        sockets = []

        def attempt_bad_register(index):
            """Thread worker: register with wrong password, record result."""
            port = _alloc_port()
            sock = _make_socket(port)
            sockets.append(sock)
            try:
                code, _ = _register_with_auth(
                    sock, "1001", "WRONG_PASSWORD_" + str(index)
                )
                results[index] = code
            except Exception as exc:
                results[index] = -1

        threads = []
        for i in range(NUM_ATTEMPTS):
            t = threading.Thread(target=attempt_bad_register, args=(i,))
            threads.append(t)
            t.start()

        # Wait for all threads to finish
        for t in threads:
            t.join(timeout=15)

        # Close all sockets
        for s in sockets:
            try:
                s.close()
            except Exception:
                pass

        # Analyze results
        completed = [r for r in results if r is not None]
        accepted = [r for r in completed if r == 200]
        rejected = [r for r in completed if r in (401, 403)]

        assert len(accepted) == 0, (
            f"Server accepted {len(accepted)} registrations with wrong password! "
            f"Results: {results}"
        )
        assert len(rejected) > 0, (
            f"No proper 401/403 rejections received. Results: {results}"
        )
        # At least half should have gotten explicit rejections
        # (some may timeout due to concurrency)
        assert len(rejected) >= NUM_ATTEMPTS // 2, (
            f"Only {len(rejected)}/{NUM_ATTEMPTS} got proper 401/403 rejections. "
            f"Results: {results}"
        )

    # ------------------------------------------------------------------
    # TC-L7-008: API error recovery
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L7_008_api_error_recovery(self):
        """TC-L7-008: Invalid API endpoints don't break valid ones.

        Hits several non-existent or invalid API endpoints, then verifies
        that the valid /ami/v1/health endpoint still responds correctly.
        This checks that error handling in the HTTP layer does not corrupt
        shared state.
        """
        invalid_endpoints = [
            f"{AMI_BASE}/nonexistent",
            f"{AMI_BASE}/../../etc/passwd",
            f"{AMI_BASE}/" + "A" * 1000,
            f"{BASE_URL}/console/fakepath",
            f"{AMI_BASE}/dialogs/999999999",
            f"{BASE_URL}/%00%01%02",  # null bytes in URL
        ]

        invalid_methods = [
            ("DELETE", f"{AMI_BASE}/health"),
            ("PATCH", f"{AMI_BASE}/dialogs"),
            ("PUT", f"{AMI_BASE}/health"),
        ]

        session = requests.Session()
        session.verify = VERIFY_TLS

        # Hit all invalid endpoints
        for url in invalid_endpoints:
            try:
                resp = session.get(url, timeout=5)
                # We don't care what code we get, just that it doesn't crash
                assert resp.status_code >= 100, (
                    f"Invalid HTTP response for {url}"
                )
            except (requests.ConnectionError, requests.Timeout):
                pass  # Connection refused or timeout is acceptable

        # Hit with invalid methods
        for method, url in invalid_methods:
            try:
                resp = session.request(method, url, timeout=5)
                assert resp.status_code >= 100
            except (requests.ConnectionError, requests.Timeout):
                pass

        # The critical check: valid endpoint must still work
        try:
            resp = session.get(f"{AMI_BASE}/health", timeout=10)
            assert resp.status_code == 200, (
                f"Health endpoint returned {resp.status_code} after "
                f"invalid API requests"
            )
        except requests.ConnectionError as exc:
            pytest.fail(
                f"Health endpoint unreachable after invalid API requests: {exc}"
            )

        # Also verify dialogs endpoint still works
        try:
            resp = session.get(f"{AMI_BASE}/dialogs", timeout=10)
            assert resp.status_code == 200, (
                f"Dialogs endpoint returned {resp.status_code} after "
                f"invalid API requests"
            )
        except requests.ConnectionError as exc:
            pytest.fail(
                f"Dialogs endpoint unreachable after invalid API requests: {exc}"
            )

    # ------------------------------------------------------------------
    # TC-L7-009: Dialog leak detection
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    def test_L7_009_dialog_leak_detection(self):
        """TC-L7-009: 5 calls with proper BYE leave no orphaned dialogs.

        Makes 5 sequential calls between 1001 and 1002, each properly
        terminated with BYE.  After all calls complete, verifies that
        /ami/v1/dialogs does not contain any stale entries for those
        Call-IDs.  This detects dialog memory leaks.
        """
        NUM_CALLS = 5
        call_ids_used = []

        for i in range(NUM_CALLS):
            caller_sock = _make_socket(_alloc_port())
            callee_sock = _make_socket(_alloc_port())

            try:
                call_state = _establish_call(caller_sock, callee_sock)
                if call_state is None:
                    # If the call didn't establish (e.g., server busy from
                    # previous tests), skip this iteration but continue
                    continue

                call_ids_used.append(call_state["call_id"])

                # Hold the call briefly
                time.sleep(0.5)

                # Send BYE
                bye_code = _send_bye(caller_sock, call_state)
                assert bye_code in (200, 481), (
                    f"Call {i+1}: BYE returned unexpected code {bye_code}"
                )

                # Drain any BYE forwarded to callee
                _recv(callee_sock, timeout=2)
            finally:
                _unregister(caller_sock, "1001")
                _unregister(callee_sock, "1002")
                caller_sock.close()
                callee_sock.close()

            # Brief pause between calls
            time.sleep(0.5)

        # Must have completed at least some calls for the test to be meaningful
        assert len(call_ids_used) >= 3, (
            f"Only {len(call_ids_used)}/{NUM_CALLS} calls established -- "
            f"cannot reliably test for leaks"
        )

        # Wait for server to process all cleanups
        time.sleep(3)

        # Check /ami/v1/dialogs for stale entries
        try:
            resp = requests.get(
                f"{AMI_BASE}/dialogs", timeout=10, verify=VERIFY_TLS
            )
            if resp.status_code == 200:
                dialogs = resp.json()
                if isinstance(dialogs, list):
                    stale = [
                        d for d in dialogs
                        if isinstance(d, dict)
                        and d.get("call_id") in call_ids_used
                    ]
                    assert len(stale) == 0, (
                        f"Found {len(stale)} stale dialogs after {len(call_ids_used)} "
                        f"calls with proper BYE: "
                        f"{[d.get('call_id') for d in stale]}"
                    )
        except requests.ConnectionError:
            pytest.skip("Dialogs API not reachable for leak verification")

    # ------------------------------------------------------------------
    # TC-L7-010: Transaction timeout on abandoned INVITE
    # ------------------------------------------------------------------
    @pytest.mark.timeout(90)
    def test_L7_010_transaction_timeout_abandoned_invite(self, sip_sock):
        """TC-L7-010: Abandoned INVITE transaction is cleaned up by the server.

        Sends an INVITE (with auth), receives 100 Trying but never follows
        up (no ACK, no CANCEL).  The server's transaction timer should
        eventually clean up the pending transaction.  We verify cleanup
        by checking that the server still accepts new requests afterwards
        and that /ami/v1/dialogs does not show a stale entry.
        """
        local_port = _local_port(sip_sock)

        # Register caller
        code, _ = _register_with_auth(sip_sock, "1001", "test1001")
        assert code == 200, f"Registration failed with {code}"

        # Build INVITE to 1002 (who may or may not be registered -- doesn't matter)
        caller_ext = "1001"
        callee_ext = "1002"
        from_uri = f"sip:{caller_ext}@{EXTERNAL_IP}"
        to_uri = f"sip:{callee_ext}@{SERVER_HOST}"
        local_tag = _gen_tag()
        call_id = _gen_callid()
        contact = f"<sip:{caller_ext}@{SERVER_HOST}:{local_port}>"
        cseq = 1
        rtp_port = _alloc_port() + 3000
        sdp_body = _build_sdp(rtp_port)

        branch = _gen_branch()
        invite = (
            f"INVITE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
            f"From: <{from_uri}>;tag={local_tag}\r\n"
            f"To: <{to_uri}>\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: {cseq} INVITE\r\n"
            f"Contact: {contact}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Type: application/sdp\r\n"
            f"Content-Length: {len(sdp_body)}\r\n\r\n"
            f"{sdp_body}"
        )
        _send(sip_sock, invite)

        # Handle the auth challenge if the server sends one
        auth_done = False
        got_provisional = False
        deadline = time.time() + 10

        while time.time() < deadline:
            resp, _ = _recv(sip_sock, timeout=2)
            if resp is None:
                break

            code = _get_response_code(resp)
            if code in (401, 407) and not auth_done:
                auth_done = True
                # ACK the challenge
                ack = (
                    f"ACK {to_uri} SIP/2.0\r\n"
                    f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={_gen_branch()};rport\r\n"
                    f"From: <{from_uri}>;tag={local_tag}\r\n"
                    f"To: <{to_uri}>\r\n"
                    f"Call-ID: {call_id}\r\n"
                    f"CSeq: {cseq} ACK\r\n"
                    f"Max-Forwards: 70\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                _send(sip_sock, ack)

                # Resend INVITE with auth
                auth_hdr = ""
                for line in resp.split("\r\n"):
                    low = line.lower()
                    if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                        auth_hdr = line.split(":", 1)[1].strip()
                        break
                realm, nonce = _parse_www_authenticate(auth_hdr)
                realm = realm or EXTERNAL_IP
                digest_resp = _compute_digest(
                    caller_ext, realm, "test1001", "INVITE", to_uri, nonce
                )
                hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
                auth_line = (
                    f'{hdr_name}: Digest username="{caller_ext}", realm="{realm}", '
                    f'nonce="{nonce}", uri="{to_uri}", response="{digest_resp}", algorithm=MD5'
                )
                cseq += 1
                branch = _gen_branch()
                invite_auth = (
                    f"INVITE {to_uri} SIP/2.0\r\n"
                    f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
                    f"From: <{from_uri}>;tag={local_tag}\r\n"
                    f"To: <{to_uri}>\r\n"
                    f"Call-ID: {call_id}\r\n"
                    f"CSeq: {cseq} INVITE\r\n"
                    f"Contact: {contact}\r\n"
                    f"{auth_line}\r\n"
                    f"Max-Forwards: 70\r\n"
                    f"Content-Type: application/sdp\r\n"
                    f"Content-Length: {len(sdp_body)}\r\n\r\n"
                    f"{sdp_body}"
                )
                _send(sip_sock, invite_auth)
            elif code == 100:
                got_provisional = True
                # Intentionally do NOT respond -- abandon the transaction
                break
            elif code in (180, 183):
                got_provisional = True
                break
            elif code >= 200:
                # Got a final response (e.g., 404 callee not found) -- ACK it
                ack = (
                    f"ACK {to_uri} SIP/2.0\r\n"
                    f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={_gen_branch()};rport\r\n"
                    f"From: <{from_uri}>;tag={local_tag}\r\n"
                    f"To: <{to_uri}>\r\n"
                    f"Call-ID: {call_id}\r\n"
                    f"CSeq: {cseq} ACK\r\n"
                    f"Max-Forwards: 70\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                _send(sip_sock, ack)
                got_provisional = True
                break

        # Now we intentionally abandon the transaction (no CANCEL, no ACK).
        # Wait for the server's transaction timer to expire.
        # RFC 3261 Timer B is 32 seconds for INVITE transactions.
        # We drain any retransmissions the server sends during this time.
        drain_deadline = time.time() + 35
        retransmit_count = 0
        while time.time() < drain_deadline:
            resp, _ = _recv(sip_sock, timeout=5)
            if resp is None:
                break
            retransmit_count += 1
            # If we get a final error response (408, 487), ACK it
            code = _get_response_code(resp)
            if code >= 400:
                to_tag = _get_to_tag(resp)
                to_field = (
                    f"<{to_uri}>;tag={to_tag}" if to_tag else f"<{to_uri}>"
                )
                ack = (
                    f"ACK {to_uri} SIP/2.0\r\n"
                    f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={_gen_branch()};rport\r\n"
                    f"From: <{from_uri}>;tag={local_tag}\r\n"
                    f"To: {to_field}\r\n"
                    f"Call-ID: {call_id}\r\n"
                    f"CSeq: {cseq} ACK\r\n"
                    f"Max-Forwards: 70\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                _send(sip_sock, ack)
                break

        # Critical check: server must still be responsive
        time.sleep(2)
        options = _build_options(sip_sock, from_user="posttimeout")
        _send(sip_sock, options)
        resp, _ = _recv(sip_sock, timeout=5)
        assert resp is not None, (
            "Server stopped responding after abandoned INVITE transaction"
        )
        code = _get_response_code(resp)
        assert code == 200, (
            f"Expected 200 OK for OPTIONS after timeout cleanup, got {code}"
        )

        # Verify no stale dialog for this call
        time.sleep(1)
        try:
            resp = requests.get(
                f"{AMI_BASE}/dialogs", timeout=10, verify=VERIFY_TLS
            )
            if resp.status_code == 200:
                dialogs = resp.json()
                if isinstance(dialogs, list):
                    stale = [
                        d for d in dialogs
                        if isinstance(d, dict)
                        and d.get("call_id") == call_id
                    ]
                    assert len(stale) == 0, (
                        f"Stale dialog found after abandoned INVITE: {stale}"
                    )
        except requests.ConnectionError:
            pass  # API check is best-effort

        # Cleanup
        _unregister(sip_sock, "1001")
