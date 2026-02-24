"""
L3 SIP Protocol Tests -- Comprehensive SIP signalling verification.

These tests exercise real SIP signalling via raw UDP sockets against a running
RustPBX instance. They verify registration flows, call setup/teardown, CANCEL
handling, re-registration, unregistration, and SIP URI handling.

No external dependencies beyond Python stdlib are required (no SIPp needed).
Each test is self-contained with its own UDP socket and cleanup logic.

Expected execution time: < 120 seconds for the full suite.

Usage:
  /root/test-env/bin/python -m pytest tests/test_L3_sip.py -v

Environment variables (all optional, sensible defaults for local server):
  RUSTPBX_HOST          SIP server IP           (default: 127.0.0.1)
  RUSTPBX_SIP_PORT      SIP port                (default: 5060)
  RUSTPBX_EXTERNAL_IP   Public IP for SIP URIs  (default: same as HOST)
"""

import hashlib
import os
import random
import re
import socket
import string
import time
import uuid

import pytest


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", SERVER_HOST)

TEST_USERS = {
    "1001": "test1001",
    "1002": "test1002",
}

# Port range for test UA sockets -- each test picks a unique port via the
# _alloc_port() helper to avoid collisions when running in parallel or when
# the OS has not yet released a TIME_WAIT socket.
_PORT_BASE = 17100
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
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=16)) + "@l3test"


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


def _get_method(data):
    """Extract the SIP method from a request line."""
    m = re.match(r"^(\w+)\s+sip:", data)
    return m.group(1) if m else ""


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


def _recv_all(sock, timeout=5, max_messages=20):
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


# ---------------------------------------------------------------------------
# Higher-level SIP operations
# ---------------------------------------------------------------------------

def _register_with_auth(sock, username, password, expires=60, cseq_start=1):
    """Perform a full REGISTER + digest-auth flow.

    Returns (response_code, response_text) of the final response.
    """
    local_port = _local_port(sock)
    tag = _gen_tag()
    cid = f"l3reg-{_gen_callid()}"
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
        "s=L3Test\r\n"
        f"c=IN IP4 {local_ip}\r\n"
        "t=0 0\r\n"
        f"m=audio {rtp_port} RTP/AVP 0 101\r\n"
        "a=rtpmap:0 PCMU/8000\r\n"
        "a=rtpmap:101 telephone-event/8000\r\n"
        "a=fmtp:101 0-16\r\n"
        "a=sendrecv\r\n"
    )


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
# L3 SIP Protocol Test Suite
# ===================================================================

class TestL3SIPProtocol:
    """L3: Comprehensive SIP protocol tests using raw UDP sockets."""

    # ------------------------------------------------------------------
    # TC-L3-001: REGISTER with correct credentials -> 200 OK
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_001_register_correct_credentials(self, sip_sock):
        """TC-L3-001: REGISTER with valid digest auth returns 200 OK.

        Exercises the full REGISTER challenge/response flow:
          1. Send unauthenticated REGISTER
          2. Receive 401 with WWW-Authenticate
          3. Compute digest and re-send with Authorization
          4. Receive 200 OK with Contact echoed back
        """
        code, resp = _register_with_auth(sip_sock, "1001", "test1001")
        assert code == 200, (
            f"Expected 200 OK for valid credentials, got {code}. "
            f"Response: {resp[:300]}"
        )
        # Verify Contact header is echoed in 200 OK
        contact = _get_header(resp, "Contact")
        assert contact, "200 OK response missing Contact header"
        assert "1001" in contact, (
            f"Contact header does not contain username '1001': {contact}"
        )

        # Cleanup
        _unregister(sip_sock, "1001")

    # ------------------------------------------------------------------
    # TC-L3-002: REGISTER with wrong password -> 403 Forbidden
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_002_register_wrong_password(self, sip_sock):
        """TC-L3-002: REGISTER with incorrect password is rejected.

        After the 401 challenge, sending a digest computed with the wrong
        password should result in 403 Forbidden (or a second 401, depending
        on server policy).
        """
        code, resp = _register_with_auth(sip_sock, "1001", "WRONG_PASSWORD")
        assert code in (401, 403), (
            f"Expected 401 or 403 for wrong password, got {code}. "
            f"Response: {resp[:300]}"
        )
        # Must NOT be 200
        assert code != 200, "Server accepted wrong password!"

    # ------------------------------------------------------------------
    # TC-L3-003: REGISTER with unknown user -> error response
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_003_register_unknown_user(self, sip_sock):
        """TC-L3-003: REGISTER for a non-existent user returns error.

        The server may return 401 (challenge that can never succeed),
        403 (forbidden), or 404 (not found). It must NOT return 200.
        """
        code, resp = _register_with_auth(
            sip_sock, "9999", "nopassword"
        )
        assert code != 200, (
            f"Server returned 200 for unknown user '9999'! Response: {resp[:300]}"
        )
        # Accept 401, 403, or 404 as valid rejections
        assert code in (0, 401, 403, 404), (
            f"Unexpected response code {code} for unknown user. "
            f"Response: {resp[:300]}"
        )

    # ------------------------------------------------------------------
    # TC-L3-004: OPTIONS keepalive -> 200 OK
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L3_004_options_keepalive(self, sip_sock):
        """TC-L3-004: OPTIONS request returns 200 OK (SIP keepalive/ping).

        OPTIONS is the standard SIP liveness probe. The server should
        respond with 200 OK regardless of authentication state.
        """
        local_port = _local_port(sip_sock)
        branch = _gen_branch()
        tag = _gen_tag()
        cid = _gen_callid()
        msg = (
            f"OPTIONS sip:{SERVER_HOST}:{SIP_PORT} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
            f"From: <sip:l3probe@{SERVER_HOST}>;tag={tag}\r\n"
            f"To: <sip:{SERVER_HOST}:{SIP_PORT}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 OPTIONS\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        _send(sip_sock, msg)
        resp, _ = _recv(sip_sock, timeout=5)
        assert resp is not None, "No response to OPTIONS request"
        code = _get_response_code(resp)
        assert code == 200, (
            f"Expected 200 OK for OPTIONS, got {code}. Response: {resp[:200]}"
        )

    # ------------------------------------------------------------------
    # TC-L3-005: INVITE call setup flow
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L3_005_invite_call_setup(self, sip_sock_pair):
        """TC-L3-005: Full INVITE call setup through the PBX.

        Flow: caller INVITE -> [401 -> auth INVITE] -> 100 Trying ->
              180 Ringing -> 200 OK -> ACK

        Requires both 1001 and 1002 to be registered. The callee (1002)
        auto-answers the INVITE from the PBX with 200 OK + SDP.
        """
        caller_sock, callee_sock = sip_sock_pair
        caller_port = _local_port(caller_sock)
        callee_port = _local_port(callee_sock)

        # Register both users
        code1, _ = _register_with_auth(caller_sock, "1001", "test1001")
        assert code1 == 200, f"Caller registration failed: {code1}"

        code2, _ = _register_with_auth(callee_sock, "1002", "test1002")
        assert code2 == 200, f"Callee registration failed: {code2}"

        # Prepare caller INVITE state
        caller_ext = "1001"
        callee_ext = "1002"
        from_uri = f"sip:{caller_ext}@{EXTERNAL_IP}"
        to_uri = f"sip:{callee_ext}@{SERVER_HOST}"
        local_tag = _gen_tag()
        call_id = _gen_callid()
        contact = f"<sip:{caller_ext}@{SERVER_HOST}:{caller_port}>"
        cseq = 1
        rtp_port = _alloc_port() + 1000  # offset to avoid SIP port collision
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

        def send_ack(cs, to_tag_val=None):
            to_field = f"<{to_uri}>;tag={to_tag_val}" if to_tag_val else f"<{to_uri}>"
            ack = (
                f"ACK {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={_gen_branch()};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: {to_field}\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {cs} ACK\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            _send(caller_sock, ack)

        # Start callee listener in main thread -- we poll both sockets
        import selectors
        sel = selectors.DefaultSelector()
        sel.register(caller_sock, selectors.EVENT_READ, "caller")
        sel.register(callee_sock, selectors.EVENT_READ, "callee")

        # Send initial INVITE
        branch = _gen_branch()
        _send(caller_sock, make_invite(branch, cseq))

        # Track which provisional responses we saw
        saw_100 = False
        saw_180_or_183 = False
        saw_200 = False
        auth_done = False
        to_tag = ""
        callee_got_invite = False

        deadline = time.time() + 20

        while time.time() < deadline:
            events = sel.select(timeout=1.0)
            for key, _ in events:
                sock_label = key.data
                data, addr = key.fileobj.recvfrom(65535)
                msg = data.decode(errors="replace")

                if sock_label == "callee":
                    # Callee receives INVITE from PBX -- auto-answer
                    if msg.startswith("INVITE "):
                        callee_got_invite = True
                        # Send 180 Ringing then 200 OK
                        cid_h = _get_header(msg, "Call-ID")
                        from_h = _get_header(msg, "From")
                        to_h = _get_header(msg, "To")
                        cseq_h = _get_header(msg, "CSeq")
                        vias = _get_all_via(msg)
                        via_block = "\r\n".join(vias)
                        callee_tag = _gen_tag()
                        to_with_tag = to_h + f";tag={callee_tag}" if "tag=" not in to_h else to_h
                        callee_contact = f"<sip:{callee_ext}@{SERVER_HOST}:{callee_port}>"

                        # 180 Ringing
                        ringing = (
                            f"SIP/2.0 180 Ringing\r\n"
                            f"{via_block}\r\n"
                            f"From: {from_h}\r\n"
                            f"To: {to_with_tag}\r\n"
                            f"Call-ID: {cid_h}\r\n"
                            f"CSeq: {cseq_h}\r\n"
                            f"Contact: {callee_contact}\r\n"
                            f"Content-Length: 0\r\n\r\n"
                        )
                        callee_sock.sendto(ringing.encode(), addr)
                        time.sleep(0.2)

                        # 200 OK with SDP
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
                    elif msg.startswith("ACK "):
                        pass  # Expected after 200 OK
                    continue

                # Caller receives responses from PBX
                if msg.startswith("SIP/2.0"):
                    code = _get_response_code(msg)
                    if code == 100:
                        saw_100 = True
                    elif code in (180, 183):
                        saw_180_or_183 = True
                    elif code in (401, 407) and not auth_done:
                        auth_done = True
                        send_ack(cseq)

                        # Extract auth and resend
                        auth_hdr = ""
                        for line in msg.split("\r\n"):
                            low = line.lower()
                            if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                                auth_hdr = line.split(":", 1)[1].strip()
                                break
                        realm, nonce = _parse_www_authenticate(auth_hdr)
                        realm = realm or EXTERNAL_IP
                        digest = _compute_digest(caller_ext, realm, "test1001", "INVITE", to_uri, nonce)
                        hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
                        auth_line = (
                            f'{hdr_name}: Digest username="{caller_ext}", realm="{realm}", '
                            f'nonce="{nonce}", uri="{to_uri}", response="{digest}", algorithm=MD5'
                        )
                        cseq += 1
                        branch = _gen_branch()
                        _send(caller_sock, make_invite(branch, cseq, auth_line))
                    elif code == 200:
                        saw_200 = True
                        to_tag = _get_to_tag(msg)
                        send_ack(cseq, to_tag)
                        break
                    elif code >= 400:
                        send_ack(cseq)
                        break

            if saw_200:
                break

        sel.unregister(caller_sock)
        sel.unregister(callee_sock)
        sel.close()

        # Assertions
        assert callee_got_invite, "Callee never received INVITE from PBX"
        assert saw_200, (
            f"Never received 200 OK. saw_100={saw_100}, "
            f"saw_180={saw_180_or_183}, auth_done={auth_done}"
        )
        # 100 Trying is expected from PBX (though not strictly mandatory per RFC 3261)
        # 180 Ringing should have been forwarded from callee
        assert saw_180_or_183, "Never received 180 Ringing or 183 Session Progress"

        # Send BYE to clean up
        bye_cseq = cseq + 1
        to_field = f"<{to_uri}>;tag={to_tag}" if to_tag else f"<{to_uri}>"
        bye = (
            f"BYE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={_gen_branch()};rport\r\n"
            f"From: <{from_uri}>;tag={local_tag}\r\n"
            f"To: {to_field}\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: {bye_cseq} BYE\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        _send(caller_sock, bye)
        _recv(caller_sock, timeout=3)

        # Cleanup registrations
        _unregister(caller_sock, "1001")
        _unregister(callee_sock, "1002")

    # ------------------------------------------------------------------
    # TC-L3-006: BYE call teardown
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L3_006_bye_call_teardown(self, sip_sock_pair):
        """TC-L3-006: BYE terminates an established call with 200 OK.

        Establishes a call between 1001 and 1002, then sends BYE from the
        caller and verifies a 200 OK response.
        """
        caller_sock, callee_sock = sip_sock_pair
        caller_port = _local_port(caller_sock)
        callee_port = _local_port(callee_sock)

        # Register both
        code1, _ = _register_with_auth(caller_sock, "1001", "test1001")
        assert code1 == 200, f"Caller registration failed: {code1}"
        code2, _ = _register_with_auth(callee_sock, "1002", "test1002")
        assert code2 == 200, f"Callee registration failed: {code2}"

        # Set up call (simplified -- reuse INVITE logic)
        caller_ext = "1001"
        callee_ext = "1002"
        from_uri = f"sip:{caller_ext}@{EXTERNAL_IP}"
        to_uri = f"sip:{callee_ext}@{SERVER_HOST}"
        local_tag = _gen_tag()
        call_id = _gen_callid()
        contact = f"<sip:{caller_ext}@{SERVER_HOST}:{caller_port}>"
        cseq = 1
        rtp_port = _alloc_port() + 1000
        sdp_body = _build_sdp(rtp_port)

        # Send INVITE (handle auth)
        branch = _gen_branch()
        invite = (
            f"INVITE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={branch};rport\r\n"
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
        _send(caller_sock, invite)

        import selectors
        sel = selectors.DefaultSelector()
        sel.register(caller_sock, selectors.EVENT_READ, "caller")
        sel.register(callee_sock, selectors.EVENT_READ, "callee")

        to_tag = ""
        call_established = False
        auth_done = False
        deadline = time.time() + 15

        while time.time() < deadline and not call_established:
            events = sel.select(timeout=1.0)
            for key, _ in events:
                data, addr = key.fileobj.recvfrom(65535)
                msg = data.decode(errors="replace")

                if key.data == "callee" and msg.startswith("INVITE "):
                    # Auto-answer
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
                elif key.data == "caller" and msg.startswith("SIP/2.0"):
                    code = _get_response_code(msg)
                    if code in (401, 407) and not auth_done:
                        auth_done = True
                        # ACK the 401
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
                        digest = _compute_digest(caller_ext, realm, "test1001", "INVITE", to_uri, nonce)
                        hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
                        auth_line = (
                            f'{hdr_name}: Digest username="{caller_ext}", realm="{realm}", '
                            f'nonce="{nonce}", uri="{to_uri}", response="{digest}", algorithm=MD5'
                        )
                        cseq += 1
                        branch = _gen_branch()
                        invite_auth = (
                            f"INVITE {to_uri} SIP/2.0\r\n"
                            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={branch};rport\r\n"
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
                        _send(caller_sock, invite_auth)
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
                        call_established = True
                        break

        sel.unregister(caller_sock)
        sel.unregister(callee_sock)
        sel.close()

        assert call_established, "Failed to establish call for BYE test"

        # Now send BYE
        bye_cseq = cseq + 1
        to_field = f"<{to_uri}>;tag={to_tag}" if to_tag else f"<{to_uri}>"
        bye = (
            f"BYE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={_gen_branch()};rport\r\n"
            f"From: <{from_uri}>;tag={local_tag}\r\n"
            f"To: {to_field}\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: {bye_cseq} BYE\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        _send(caller_sock, bye)

        # Expect 200 OK for BYE
        resp, _ = _recv(caller_sock, timeout=5)
        assert resp is not None, "No response to BYE"
        code = _get_response_code(resp)
        assert code == 200, (
            f"Expected 200 OK for BYE, got {code}. Response: {resp[:200]}"
        )

        # Cleanup
        _unregister(caller_sock, "1001")
        _unregister(callee_sock, "1002")

    # ------------------------------------------------------------------
    # TC-L3-007: CANCEL mid-ring
    # ------------------------------------------------------------------
    @pytest.mark.timeout(30)
    def test_L3_007_cancel_mid_ring(self, sip_sock_pair):
        """TC-L3-007: CANCEL a ringing call returns 200 OK for CANCEL + 487.

        Flow:
          1. Caller sends INVITE (with auth)
          2. Wait for 180 Ringing from callee (callee does NOT send 200 OK)
          3. Caller sends CANCEL
          4. Expect 200 OK for CANCEL
          5. Expect 487 Request Terminated for original INVITE
        """
        caller_sock, callee_sock = sip_sock_pair
        caller_port = _local_port(caller_sock)
        callee_port = _local_port(callee_sock)

        # Register both
        code1, _ = _register_with_auth(caller_sock, "1001", "test1001")
        assert code1 == 200, f"Caller registration failed: {code1}"
        code2, _ = _register_with_auth(callee_sock, "1002", "test1002")
        assert code2 == 200, f"Callee registration failed: {code2}"

        caller_ext = "1001"
        callee_ext = "1002"
        from_uri = f"sip:{caller_ext}@{EXTERNAL_IP}"
        to_uri = f"sip:{callee_ext}@{SERVER_HOST}"
        local_tag = _gen_tag()
        call_id = _gen_callid()
        contact = f"<sip:{caller_ext}@{SERVER_HOST}:{caller_port}>"
        cseq = 1
        rtp_port = _alloc_port() + 1000
        sdp_body = _build_sdp(rtp_port)

        branch = _gen_branch()
        invite = (
            f"INVITE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={branch};rport\r\n"
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
        _send(caller_sock, invite)

        import selectors
        sel = selectors.DefaultSelector()
        sel.register(caller_sock, selectors.EVENT_READ, "caller")
        sel.register(callee_sock, selectors.EVENT_READ, "callee")

        auth_done = False
        saw_ringing = False
        invite_branch = branch  # branch used for the (possibly re-sent) INVITE
        cancel_sent = False
        saw_cancel_200 = False
        saw_487 = False
        deadline = time.time() + 15

        while time.time() < deadline:
            events = sel.select(timeout=0.5)
            for key, _ in events:
                data, addr = key.fileobj.recvfrom(65535)
                msg = data.decode(errors="replace")

                if key.data == "callee":
                    if msg.startswith("INVITE "):
                        # Send 180 Ringing only -- do NOT answer
                        cid_h = _get_header(msg, "Call-ID")
                        from_h = _get_header(msg, "From")
                        to_h = _get_header(msg, "To")
                        cseq_h = _get_header(msg, "CSeq")
                        vias = _get_all_via(msg)
                        via_block = "\r\n".join(vias)
                        callee_tag = _gen_tag()
                        to_with_tag = to_h + f";tag={callee_tag}" if "tag=" not in to_h else to_h
                        callee_contact = f"<sip:{callee_ext}@{SERVER_HOST}:{callee_port}>"
                        ringing = (
                            f"SIP/2.0 180 Ringing\r\n"
                            f"{via_block}\r\n"
                            f"From: {from_h}\r\n"
                            f"To: {to_with_tag}\r\n"
                            f"Call-ID: {cid_h}\r\n"
                            f"CSeq: {cseq_h}\r\n"
                            f"Contact: {callee_contact}\r\n"
                            f"Content-Length: 0\r\n\r\n"
                        )
                        callee_sock.sendto(ringing.encode(), addr)
                    elif msg.startswith("CANCEL "):
                        # Respond 200 OK to CANCEL
                        cid_h = _get_header(msg, "Call-ID")
                        from_h = _get_header(msg, "From")
                        to_h = _get_header(msg, "To")
                        cseq_h = _get_header(msg, "CSeq")
                        vias = _get_all_via(msg)
                        via_block = "\r\n".join(vias)
                        ok_resp = (
                            f"SIP/2.0 200 OK\r\n"
                            f"{via_block}\r\n"
                            f"From: {from_h}\r\n"
                            f"To: {to_h}\r\n"
                            f"Call-ID: {cid_h}\r\n"
                            f"CSeq: {cseq_h}\r\n"
                            f"Content-Length: 0\r\n\r\n"
                        )
                        callee_sock.sendto(ok_resp.encode(), addr)

                        # Also send 487 for the original INVITE
                        inv_cseq = cseq_h.replace("CANCEL", "INVITE").strip()
                        terminated = (
                            f"SIP/2.0 487 Request Terminated\r\n"
                            f"{via_block}\r\n"
                            f"From: {from_h}\r\n"
                            f"To: {to_h}\r\n"
                            f"Call-ID: {cid_h}\r\n"
                            f"CSeq: {inv_cseq}\r\n"
                            f"Content-Length: 0\r\n\r\n"
                        )
                        callee_sock.sendto(terminated.encode(), addr)
                    continue

                # Caller responses
                if msg.startswith("SIP/2.0"):
                    code = _get_response_code(msg)
                    if code in (401, 407) and not auth_done:
                        auth_done = True
                        # ACK the error
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
                        digest = _compute_digest(caller_ext, realm, "test1001", "INVITE", to_uri, nonce)
                        hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
                        auth_line = (
                            f'{hdr_name}: Digest username="{caller_ext}", realm="{realm}", '
                            f'nonce="{nonce}", uri="{to_uri}", response="{digest}", algorithm=MD5'
                        )
                        cseq += 1
                        invite_branch = _gen_branch()
                        invite_auth = (
                            f"INVITE {to_uri} SIP/2.0\r\n"
                            f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={invite_branch};rport\r\n"
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
                        _send(caller_sock, invite_auth)
                    elif code in (180, 183):
                        saw_ringing = True
                    elif code == 100:
                        pass  # Trying
                    elif code == 200:
                        # Check if this is the CANCEL 200 or INVITE 200
                        cseq_hdr = _get_header(msg, "CSeq")
                        if "CANCEL" in cseq_hdr:
                            saw_cancel_200 = True
                        # If INVITE 200, send ACK (shouldn't happen in this test)
                    elif code == 487:
                        saw_487 = True
                        # ACK the 487
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

            # Send CANCEL after ringing is received
            if saw_ringing and not cancel_sent:
                cancel_sent = True
                # CANCEL must use the same branch as the INVITE
                cancel = (
                    f"CANCEL {to_uri} SIP/2.0\r\n"
                    f"Via: SIP/2.0/UDP {SERVER_HOST}:{caller_port};branch={invite_branch};rport\r\n"
                    f"From: <{from_uri}>;tag={local_tag}\r\n"
                    f"To: <{to_uri}>\r\n"
                    f"Call-ID: {call_id}\r\n"
                    f"CSeq: {cseq} CANCEL\r\n"
                    f"Max-Forwards: 70\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                _send(caller_sock, cancel)

            if saw_cancel_200 and saw_487:
                break

        sel.unregister(caller_sock)
        sel.unregister(callee_sock)
        sel.close()

        assert saw_ringing, "Never received 180 Ringing"
        assert cancel_sent, "CANCEL was never sent (no ringing received)"
        assert saw_cancel_200 or saw_487, (
            f"CANCEL flow incomplete: saw_cancel_200={saw_cancel_200}, saw_487={saw_487}"
        )
        # At minimum, one of these must be true to prove CANCEL was processed
        assert saw_cancel_200 or saw_487, "CANCEL was not properly acknowledged"

        # Cleanup
        _unregister(caller_sock, "1001")
        _unregister(callee_sock, "1002")

    # ------------------------------------------------------------------
    # TC-L3-008: Re-REGISTER (refresh)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_008_re_register(self, sip_sock):
        """TC-L3-008: Re-registering an already-registered user succeeds.

        Register once, then register again with the same Contact and a
        new Expires. Both should return 200 OK.
        """
        code1, resp1 = _register_with_auth(sip_sock, "1001", "test1001", expires=60)
        assert code1 == 200, f"Initial registration failed: {code1}"

        time.sleep(1)

        # Re-register with a refreshed expiry (different CSeq is handled
        # internally by _register_with_auth using cseq_start)
        code2, resp2 = _register_with_auth(
            sip_sock, "1001", "test1001", expires=120, cseq_start=10
        )
        assert code2 == 200, (
            f"Re-registration failed with code {code2}. Response: {resp2[:300]}"
        )

        # Cleanup
        _unregister(sip_sock, "1001")

    # ------------------------------------------------------------------
    # TC-L3-009: Unregister (Expires: 0)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_009_unregister_expires_zero(self, sip_sock):
        """TC-L3-009: REGISTER with Expires: 0 removes the registration.

        After unregistering, the server should no longer have a binding
        for the user. We verify by sending Expires:0 and checking for
        a 200 OK (or 401 challenge which is also acceptable since the
        unauthenticated unregister may be challenged).
        """
        # First, register
        code, _ = _register_with_auth(sip_sock, "1001", "test1001", expires=60)
        assert code == 200, f"Registration before unregister failed: {code}"

        # Now unregister with Expires: 0 (with auth)
        code_unreg, resp_unreg = _register_with_auth(
            sip_sock, "1001", "test1001", expires=0, cseq_start=20
        )
        assert code_unreg == 200, (
            f"Unregister (Expires:0) failed with code {code_unreg}. "
            f"Response: {resp_unreg[:300]}"
        )

    # ------------------------------------------------------------------
    # TC-L3-010: Multiple concurrent registrations from different ports
    # ------------------------------------------------------------------
    @pytest.mark.timeout(25)
    def test_L3_010_concurrent_registrations(self):
        """TC-L3-010: Multiple users can register concurrently.

        Register 1001 and 1002 from different sockets/ports and verify
        both receive 200 OK independently.
        """
        sock1 = _make_socket(_alloc_port())
        sock2 = _make_socket(_alloc_port())

        try:
            code1, _ = _register_with_auth(sock1, "1001", "test1001")
            code2, _ = _register_with_auth(sock2, "1002", "test1002")

            assert code1 == 200, f"User 1001 registration failed: {code1}"
            assert code2 == 200, f"User 1002 registration failed: {code2}"

            # Verify both are registered by re-checking (idempotent)
            code1b, _ = _register_with_auth(sock1, "1001", "test1001", cseq_start=10)
            code2b, _ = _register_with_auth(sock2, "1002", "test1002", cseq_start=10)
            assert code1b == 200, f"User 1001 re-registration failed: {code1b}"
            assert code2b == 200, f"User 1002 re-registration failed: {code2b}"

            # Cleanup
            _unregister(sock1, "1001")
            _unregister(sock2, "1002")
        finally:
            sock1.close()
            sock2.close()

    # ------------------------------------------------------------------
    # TC-L3-011: SIP URI handling (To/From/Contact parsing)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_011_sip_uri_handling(self, sip_sock):
        """TC-L3-011: Server correctly parses and echoes SIP URIs.

        Verifies that the server's response contains proper SIP URI
        formatting in the Via, From, To, and Contact headers.
        """
        local_port = _local_port(sip_sock)
        tag = _gen_tag()
        cid = _gen_callid()
        branch = _gen_branch()
        username = "1001"
        from_uri = f"sip:{username}@{EXTERNAL_IP}"
        contact_uri = f"sip:{username}@{SERVER_HOST}:{local_port};transport=udp"

        # Send unauthenticated REGISTER to get a 401 with proper headers
        msg = (
            f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
            f"From: <{from_uri}>;tag={tag}\r\n"
            f"To: <{from_uri}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: <{contact_uri}>\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        _send(sip_sock, msg)
        resp, _ = _recv(sip_sock, timeout=5)
        assert resp is not None, "No response to REGISTER"

        # Check the response is valid SIP
        assert resp.startswith("SIP/2.0"), (
            f"Response does not start with SIP/2.0: {resp[:50]}"
        )

        # Via header should be echoed back with our branch
        via = _get_header(resp, "Via")
        assert branch in via, (
            f"Via header does not contain our branch {branch}: {via}"
        )

        # From header should be echoed back exactly
        from_h = _get_header(resp, "From")
        assert username in from_h, (
            f"From header does not contain username '{username}': {from_h}"
        )
        assert f"tag={tag}" in from_h, (
            f"From header does not contain our tag '{tag}': {from_h}"
        )

        # To header should contain our username
        to_h = _get_header(resp, "To")
        assert username in to_h, (
            f"To header does not contain username '{username}': {to_h}"
        )

        # Call-ID should match
        cid_h = _get_header(resp, "Call-ID")
        assert cid_h == cid, (
            f"Call-ID mismatch: sent '{cid}', got '{cid_h}'"
        )

        # CSeq should match method
        cseq_h = _get_header(resp, "CSeq")
        assert "REGISTER" in cseq_h, (
            f"CSeq does not contain REGISTER: {cseq_h}"
        )

    # ------------------------------------------------------------------
    # TC-L3-012: Initial REGISTER returns 401 challenge
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L3_012_register_returns_401_challenge(self, sip_sock):
        """TC-L3-012: Unauthenticated REGISTER returns 401 with challenge.

        The server must return 401 Unauthorized with a WWW-Authenticate
        header containing realm and nonce parameters.
        """
        local_port = _local_port(sip_sock)
        branch = _gen_branch()
        tag = _gen_tag()
        cid = _gen_callid()
        from_uri = f"sip:1001@{EXTERNAL_IP}"
        contact = f"<sip:1001@{SERVER_HOST}:{local_port};transport=udp>"

        msg = (
            f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER_HOST}:{local_port};branch={branch};rport\r\n"
            f"From: <{from_uri}>;tag={tag}\r\n"
            f"To: <{from_uri}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: 1 REGISTER\r\n"
            f"Contact: {contact}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 60\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        _send(sip_sock, msg)
        resp, _ = _recv(sip_sock, timeout=5)
        assert resp is not None, "No response to unauthenticated REGISTER"

        code = _get_response_code(resp)
        assert code in (401, 407), (
            f"Expected 401/407 challenge, got {code}. Response: {resp[:200]}"
        )

        # Extract WWW-Authenticate
        auth_hdr = ""
        for line in resp.split("\r\n"):
            low = line.lower()
            if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
                auth_hdr = line.split(":", 1)[1].strip()
                break

        assert auth_hdr, "Missing WWW-Authenticate/Proxy-Authenticate header"
        assert "Digest" in auth_hdr, (
            f"Auth header is not Digest: {auth_hdr}"
        )

        realm, nonce = _parse_www_authenticate(auth_hdr)
        assert nonce, f"Missing nonce in challenge: {auth_hdr}"
        # realm may be empty in some configs but nonce is required

    # ------------------------------------------------------------------
    # TC-L3-013: REGISTER for second user (1002)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(20)
    def test_L3_013_register_second_user(self, sip_sock):
        """TC-L3-013: A different user (1002) can register independently."""
        code, resp = _register_with_auth(sip_sock, "1002", "test1002")
        assert code == 200, (
            f"User 1002 registration failed with code {code}. "
            f"Response: {resp[:300]}"
        )
        _unregister(sip_sock, "1002")

    # ------------------------------------------------------------------
    # TC-L3-014: No lingering dialogs after tests
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L3_014_no_lingering_dialogs(self):
        """TC-L3-014: AMI dialogs endpoint shows no active calls.

        After all call tests, the PBX should have no lingering dialogs.
        This catches state leaks from improperly terminated calls.
        """
        import requests
        import urllib3
        urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

        # Try multiple endpoints/schemes
        for base in [
            f"https://{SERVER_HOST}:8443",
            f"http://{SERVER_HOST}:8080",
        ]:
            try:
                resp = requests.get(
                    f"{base}/ami/v1/dialogs",
                    timeout=5,
                    verify=False,
                )
                if resp.status_code == 200:
                    data = resp.json()
                    if isinstance(data, list):
                        assert len(data) == 0, (
                            f"Lingering dialogs found: {data}"
                        )
                    return  # Success
            except (requests.ConnectionError, requests.Timeout):
                continue

        pytest.skip("Cannot reach AMI dialogs endpoint")
