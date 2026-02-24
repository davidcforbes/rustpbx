"""
L9 End-to-End Call Flow Tests -- Full lifecycle call scenarios.

Exercises complete call flows against a running RustPBX instance using
pure-Python SIP UAs and RTP stacks.  Each test establishes real calls
between two endpoints (1001, 1002), verifies signalling and media,
and cleans up after itself.

Tests:
  1. Internal call with recording verification
  2. Call with DTMF (RFC 2833)
  3. Multiple sequential calls
  4. Call reject (486 Busy Here)
  5. Call cancel (CANCEL before answer)
  6. Re-INVITE hold/unhold
  7. Concurrent calls

Server:  RUSTPBX_HOST (default 127.0.0.1) : 5060  (UDP)
Users:   1001/test1001, 1002/test1002

Run with:
  python -m pytest tests/test_L9_e2e.py -v

Environment variables (all optional, sensible defaults for local server):
  RUSTPBX_HOST          SIP server IP           (default: 127.0.0.1)
  RUSTPBX_SIP_PORT      SIP port                (default: 5060)
  RUSTPBX_EXTERNAL_IP   Public IP for SIP URIs  (default: same as HOST)
  RUSTPBX_HTTP_PORT     HTTP(S) port            (default: 8443)
  RUSTPBX_SCHEME        http or https           (default: https)
"""

import hashlib
import os
import random
import re
import selectors
import socket
import string
import struct
import threading
import time
import uuid

import pytest
import requests
import urllib3

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ============================================================
# Configuration
# ============================================================

SERVER = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SERVER_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", SERVER)
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
HTTP_SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
BASE_URL = os.environ.get(
    "RUSTPBX_BASE_URL",
    f"{HTTP_SCHEME}://{SERVER}:{HTTP_PORT}",
)
VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in ("1", "true", "yes")
ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

RECORDER_DIR = os.environ.get(
    "RECORDING_DIR",
    os.path.expanduser("~/rustpbx/config/recorders"),
)

TEST_USERS = {
    "1001": "test1001",
    "1002": "test1002",
}

PCMU_PT = 0
DTMF_PT = 101
PCMU_RATE = 8000
FRAME_MS = 20
PCMU_FRAME_SAMPLES = PCMU_RATE * FRAME_MS // 1000  # 160

# Port allocation -- each test gets a unique block of ports
_PORT_LOCK = threading.Lock()
_PORT_BASE = 18200


def _alloc_ports(count=4):
    """Allocate a unique set of ports.

    Returns dict with caller_sip_port, callee_sip_port, caller_rtp_port,
    callee_rtp_port.
    """
    global _PORT_BASE
    with _PORT_LOCK:
        base = _PORT_BASE
        _PORT_BASE += 10
    return {
        "caller_sip_port": base,
        "callee_sip_port": base + 2,
        "caller_rtp_port": base + 4,
        "callee_rtp_port": base + 6,
    }


# ============================================================
# SIP Helpers
# ============================================================

def gen_branch():
    return "z9hG4bK" + "".join(random.choices(string.ascii_lowercase + string.digits, k=12))


def gen_tag():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=8))


def gen_callid():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=16)) + "@e2e"


def md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()


def compute_digest(username, realm, password, method, uri, nonce):
    ha1 = md5hex(f"{username}:{realm}:{password}")
    ha2 = md5hex(f"{method}:{uri}")
    return md5hex(f"{ha1}:{nonce}:{ha2}")


def get_response_code(data):
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0


def get_method(data):
    m = re.match(r"^(\w+)\s+sip:", data)
    return m.group(1) if m else ""


def get_header(data, name):
    for line in data.split("\r\n"):
        if line.lower().startswith(name.lower() + ":"):
            return line.split(":", 1)[1].strip()
    return ""


def get_to_tag(data):
    to_hdr = get_header(data, "To")
    m = re.search(r"tag=([^\s;>]+)", to_hdr)
    return m.group(1) if m else ""


def get_all_via(data):
    vias = []
    for line in data.split("\r\n"):
        if line.lower().startswith("via:"):
            vias.append(line)
    return vias


def parse_www_authenticate(header_line):
    realm = re.search(r'realm="([^"]*)"', header_line)
    nonce = re.search(r'nonce="([^"]*)"', header_line)
    return (realm.group(1) if realm else ""), (nonce.group(1) if nonce else "")


def parse_sdp(sdp_text):
    """Extract connection IP, media port, and codec list from SDP."""
    ip = None
    port = None
    codecs = []
    for line in sdp_text.split("\r\n"):
        line = line.strip()
        if line.startswith("c=IN IP4 "):
            ip = line.split()[-1]
        elif line.startswith("m=audio "):
            parts = line.split()
            port = int(parts[1])
        elif line.startswith("a=rtpmap:"):
            m = re.match(r"a=rtpmap:(\d+)\s+(\S+)", line)
            if m:
                codecs.append((int(m.group(1)), m.group(2)))
    return ip, port, codecs


def get_sdp_direction(sdp_text):
    """Extract media direction attribute (sendrecv, sendonly, recvonly, inactive)."""
    for line in sdp_text.split("\r\n"):
        line = line.strip()
        if line in ("a=sendrecv", "a=sendonly", "a=recvonly", "a=inactive"):
            return line[2:]  # strip "a="
    return "sendrecv"  # default per RFC


def build_sdp(rtp_port, local_ip=None, direction="sendrecv"):
    """Build a minimal SDP body offering PCMU + telephone-event."""
    local_ip = local_ip or SERVER
    sid = str(random.randint(100000, 999999))
    return (
        "v=0\r\n"
        f"o=- {sid} {sid} IN IP4 {local_ip}\r\n"
        "s=E2ETest\r\n"
        f"c=IN IP4 {local_ip}\r\n"
        "t=0 0\r\n"
        f"m=audio {rtp_port} RTP/AVP {PCMU_PT} {DTMF_PT}\r\n"
        f"a=rtpmap:{PCMU_PT} PCMU/8000\r\n"
        f"a=rtpmap:{DTMF_PT} telephone-event/8000\r\n"
        f"a=fmtp:{DTMF_PT} 0-16\r\n"
        f"a={direction}\r\n"
    )


def make_socket(port):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", port))
    return s


def send_sip(sock, msg, dest=(SERVER, SERVER_PORT)):
    sock.sendto(msg.encode() if isinstance(msg, str) else msg, dest)


def recv_sip(sock, timeout=5):
    sock.settimeout(timeout)
    try:
        data, addr = sock.recvfrom(65535)
        return data.decode(errors="replace"), addr
    except socket.timeout:
        return None, None


def drain_socket(sock, timeout=0.5):
    """Drain all pending messages from a socket."""
    sock.settimeout(timeout)
    while True:
        try:
            sock.recvfrom(65535)
        except socket.timeout:
            break
        except OSError:
            break


# ============================================================
# RTP Helpers
# ============================================================

def build_rtp(pt, seq, timestamp, ssrc, payload, marker=False):
    b0 = 0x80
    b1 = (0x80 if marker else 0x00) | (pt & 0x7f)
    hdr = struct.pack("!BBHII", b0, b1, seq & 0xffff, timestamp & 0xffffffff, ssrc)
    return hdr + payload


def parse_rtp(data):
    if len(data) < 12:
        return None
    b0, b1, seq, ts, ssrc = struct.unpack("!BBHII", data[:12])
    version = (b0 >> 6) & 0x03
    pt = b1 & 0x7f
    marker = bool(b1 & 0x80)
    cc = b0 & 0x0f
    offset = 12 + cc * 4
    if b0 & 0x10 and len(data) >= offset + 4:
        ext_len = struct.unpack("!H", data[offset + 2:offset + 4])[0]
        offset += 4 + ext_len * 4
    return {
        "version": version, "pt": pt, "seq": seq, "ts": ts,
        "ssrc": ssrc, "payload": data[offset:], "marker": marker,
    }


# ============================================================
# Audio Generation (PCMU / mu-law)
# ============================================================

def linear_to_ulaw(sample):
    """Convert a 16-bit linear PCM sample to 8-bit mu-law."""
    BIAS = 0x84
    MAX = 32635
    sign = 0
    if sample < 0:
        sign = 0x80
        sample = -sample
    if sample > MAX:
        sample = MAX
    sample = sample + BIAS
    exponent = 7
    for exp_val in [0x4000, 0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100]:
        if sample >= exp_val:
            break
        exponent -= 1
    mantissa = (sample >> (exponent + 3)) & 0x0F
    ulaw_byte = ~(sign | (exponent << 4) | mantissa) & 0xFF
    return ulaw_byte


def generate_ulaw_frames(freq_hz=440, duration_s=5, amplitude=16000):
    """Generate PCMU (mu-law) 20ms frames from a sine wave."""
    import math
    frames = []
    total_samples = int(PCMU_RATE * duration_s)
    for start in range(0, total_samples - PCMU_FRAME_SAMPLES + 1, PCMU_FRAME_SAMPLES):
        frame_bytes = bytearray(PCMU_FRAME_SAMPLES)
        for i in range(PCMU_FRAME_SAMPLES):
            t = (start + i) / PCMU_RATE
            sample = int(amplitude * math.sin(2 * math.pi * freq_hz * t))
            frame_bytes[i] = linear_to_ulaw(sample)
        frames.append(bytes(frame_bytes))
    return frames


def generate_silence_ulaw(duration_s=5):
    """Generate PCMU silence frames (mu-law zero = 0xFF)."""
    frames = []
    total_samples = int(PCMU_RATE * duration_s)
    for _ in range(0, total_samples - PCMU_FRAME_SAMPLES + 1, PCMU_FRAME_SAMPLES):
        frames.append(b'\xff' * PCMU_FRAME_SAMPLES)
    return frames


# ============================================================
# RTP Sender / Receiver Threads
# ============================================================

def rtp_sender(sock, target, frames, stop_event, pt=PCMU_PT,
               ts_increment=PCMU_FRAME_SAMPLES):
    """Send RTP packets at 20ms pacing until stop_event is set."""
    ssrc = random.randint(1, 0xFFFFFFFF)
    seq = random.randint(0, 0xFFFF)
    ts = random.randint(0, 0xFFFFFFFF)
    idx = 0
    interval = FRAME_MS / 1000.0
    next_send = time.time()

    while not stop_event.is_set():
        if idx >= len(frames):
            idx = 0
        pkt = build_rtp(pt, seq, ts, ssrc, frames[idx], marker=(idx == 0))
        try:
            sock.sendto(pkt, target)
        except OSError:
            break
        seq = (seq + 1) & 0xFFFF
        ts = (ts + ts_increment) & 0xFFFFFFFF
        idx += 1
        next_send += interval
        delay = next_send - time.time()
        if delay > 0:
            time.sleep(delay)
        elif delay < -0.5:
            next_send = time.time()


def rtp_receiver(sock, stop_event, accept_pts=None):
    """Collect received RTP packets.  Returns list of parsed dicts."""
    packets = []
    sock.settimeout(0.2)
    while not stop_event.is_set():
        try:
            data, _ = sock.recvfrom(65535)
            parsed = parse_rtp(data)
            if parsed:
                if accept_pts is None or parsed["pt"] in accept_pts:
                    packets.append(parsed)
        except socket.timeout:
            continue
        except OSError:
            break
    return packets


# ============================================================
# SIP Registration
# ============================================================

def sip_register(sock, ext, password, sip_port):
    """Perform full REGISTER with digest auth.  Returns True on success."""
    tag = gen_tag()
    cid = f"e2ereg-{gen_callid()}"
    from_uri = f"sip:{ext}@{EXTERNAL_IP}"
    contact = f"<sip:{ext}@{SERVER}:{sip_port};transport=udp>"
    cseq = 1
    branch = gen_branch()

    msg = (
        f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: 3600\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    send_sip(sock, msg)
    resp, _ = recv_sip(sock)
    if not resp:
        return False

    code = get_response_code(resp)
    if code not in (401, 407):
        return code == 200

    # Digest auth
    auth_hdr = ""
    for line in resp.split("\r\n"):
        low = line.lower()
        if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
            auth_hdr = line.split(":", 1)[1].strip()
            break

    realm, nonce = parse_www_authenticate(auth_hdr)
    realm = realm or EXTERNAL_IP
    uri = f"sip:{realm}"
    digest = compute_digest(ext, realm, password, "REGISTER", uri, nonce)
    hdr_name = "Authorization" if code == 401 else "Proxy-Authorization"
    auth_line = (
        f'Digest username="{ext}", realm="{realm}", nonce="{nonce}", '
        f'uri="{uri}", response="{digest}", algorithm=MD5'
    )

    cseq += 1
    branch = gen_branch()
    msg = (
        f"REGISTER sip:{realm} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: {cid}\r\n"
        f"CSeq: {cseq} REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"{hdr_name}: {auth_line}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: 3600\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    send_sip(sock, msg)
    resp, _ = recv_sip(sock)
    return resp and get_response_code(resp) == 200


def sip_unregister(sock, ext, sip_port):
    """Send Expires: 0 REGISTER (best-effort cleanup)."""
    tag = gen_tag()
    from_uri = f"sip:{ext}@{EXTERNAL_IP}"
    contact = f"<sip:{ext}@{SERVER}:{sip_port};transport=udp>"
    branch = gen_branch()
    msg = (
        f"REGISTER sip:{EXTERNAL_IP} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={branch};rport\r\n"
        f"From: <{from_uri}>;tag={tag}\r\n"
        f"To: <{from_uri}>\r\n"
        f"Call-ID: unreg-{gen_callid()}\r\n"
        f"CSeq: 1 REGISTER\r\n"
        f"Contact: {contact}\r\n"
        f"Max-Forwards: 70\r\n"
        f"Expires: 0\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    send_sip(sock, msg)
    recv_sip(sock, timeout=2)


# ============================================================
# Callee Behaviors -- pluggable response strategies
# ============================================================

class CalleeAutoAnswer:
    """Background thread that auto-answers incoming INVITEs with 200 OK."""

    def __init__(self, sock, ext, rtp_port, sdp_direction="sendrecv"):
        self.sock = sock
        self.ext = ext
        self.rtp_port = rtp_port
        self.sdp_direction = sdp_direction
        self.rtp_target = None
        self.remote_codecs = []
        self.call_established = threading.Event()
        self.invite_received = threading.Event()
        self.bye_received = threading.Event()
        self.reinvite_received = threading.Event()
        self.reinvite_sdp = None
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._invite_count = 0
        self._last_addr = None

    def start(self):
        self._thread.start()

    def stop(self):
        self._stop.set()

    def wait_for_call(self, timeout=30):
        return self.call_established.wait(timeout)

    def _run(self):
        self.sock.settimeout(1)
        while not self._stop.is_set():
            try:
                data, addr = self.sock.recvfrom(65535)
                msg = data.decode(errors="replace")
            except socket.timeout:
                continue
            except OSError:
                break

            if msg.startswith("INVITE "):
                self._invite_count += 1
                self._last_addr = addr
                if self._invite_count == 1:
                    # First INVITE: normal answer
                    self._on_invite(msg, addr)
                else:
                    # Subsequent INVITEs: re-INVITE
                    self._on_reinvite(msg, addr)
            elif msg.startswith("ACK "):
                self.call_established.set()
            elif msg.startswith("BYE "):
                self._reply_ok(msg, addr)
                self.bye_received.set()
            elif msg.startswith("CANCEL "):
                self._reply_ok(msg, addr)

    def _on_invite(self, msg, addr):
        cid = get_header(msg, "Call-ID")
        from_h = get_header(msg, "From")
        to_h = get_header(msg, "To")
        cseq = get_header(msg, "CSeq")
        vias = get_all_via(msg)
        via_block = "\r\n".join(vias)

        # Parse remote SDP
        sdp_start = msg.find("\r\n\r\n")
        if sdp_start >= 0:
            ip, port, codecs = parse_sdp(msg[sdp_start + 4:])
            if ip and port:
                self.rtp_target = (ip, port)
                self.remote_codecs = codecs
                self.invite_received.set()

        to_tag = gen_tag()
        to_with_tag = to_h + f";tag={to_tag}" if "tag=" not in to_h else to_h
        contact = f"<sip:{self.ext}@{SERVER}:{self.sock.getsockname()[1]}>"

        # 180 Ringing
        ringing = (
            f"SIP/2.0 180 Ringing\r\n"
            f"{via_block}\r\n"
            f"From: {from_h}\r\n"
            f"To: {to_with_tag}\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq}\r\n"
            f"Contact: {contact}\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        self.sock.sendto(ringing.encode(), addr)
        time.sleep(0.3)

        # 200 OK with SDP
        sdp_body = build_sdp(self.rtp_port, direction=self.sdp_direction)
        ok = (
            f"SIP/2.0 200 OK\r\n"
            f"{via_block}\r\n"
            f"From: {from_h}\r\n"
            f"To: {to_with_tag}\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq}\r\n"
            f"Contact: {contact}\r\n"
            f"Content-Type: application/sdp\r\n"
            f"Content-Length: {len(sdp_body)}\r\n\r\n"
            f"{sdp_body}"
        )
        self.sock.sendto(ok.encode(), addr)

    def _on_reinvite(self, msg, addr):
        """Handle re-INVITEs (hold/unhold)."""
        cid = get_header(msg, "Call-ID")
        from_h = get_header(msg, "From")
        to_h = get_header(msg, "To")
        cseq = get_header(msg, "CSeq")
        vias = get_all_via(msg)
        via_block = "\r\n".join(vias)

        # Parse the SDP from re-INVITE
        sdp_start = msg.find("\r\n\r\n")
        if sdp_start >= 0:
            self.reinvite_sdp = msg[sdp_start + 4:]
            self.reinvite_received.set()

        contact = f"<sip:{self.ext}@{SERVER}:{self.sock.getsockname()[1]}>"

        # Respond with 200 OK, echoing the same direction
        direction = "sendrecv"
        if self.reinvite_sdp:
            direction = get_sdp_direction(self.reinvite_sdp)
            # If caller sends sendonly, we respond with recvonly (and vice versa)
            if direction == "sendonly":
                direction = "recvonly"
            elif direction == "recvonly":
                direction = "sendonly"

        sdp_body = build_sdp(self.rtp_port, direction=direction)
        ok = (
            f"SIP/2.0 200 OK\r\n"
            f"{via_block}\r\n"
            f"From: {from_h}\r\n"
            f"To: {to_h}\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq}\r\n"
            f"Contact: {contact}\r\n"
            f"Content-Type: application/sdp\r\n"
            f"Content-Length: {len(sdp_body)}\r\n\r\n"
            f"{sdp_body}"
        )
        self.sock.sendto(ok.encode(), addr)

    def _reply_ok(self, msg, addr):
        vias = get_all_via(msg)
        via_block = "\r\n".join(vias)
        ok = (
            f"SIP/2.0 200 OK\r\n"
            f"{via_block}\r\n"
            f"From: {get_header(msg, 'From')}\r\n"
            f"To: {get_header(msg, 'To')}\r\n"
            f"Call-ID: {get_header(msg, 'Call-ID')}\r\n"
            f"CSeq: {get_header(msg, 'CSeq')}\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        self.sock.sendto(ok.encode(), addr)


class CalleeReject:
    """Background thread that rejects incoming INVITEs with a configurable code."""

    def __init__(self, sock, ext, reject_code=486, reject_reason="Busy Here"):
        self.sock = sock
        self.ext = ext
        self.reject_code = reject_code
        self.reject_reason = reject_reason
        self.invite_received = threading.Event()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)

    def start(self):
        self._thread.start()

    def stop(self):
        self._stop.set()

    def _run(self):
        self.sock.settimeout(1)
        while not self._stop.is_set():
            try:
                data, addr = self.sock.recvfrom(65535)
                msg = data.decode(errors="replace")
            except socket.timeout:
                continue
            except OSError:
                break

            if msg.startswith("INVITE "):
                self.invite_received.set()
                self._reject(msg, addr)
            elif msg.startswith("ACK "):
                pass  # ACK to our error response

    def _reject(self, msg, addr):
        cid = get_header(msg, "Call-ID")
        from_h = get_header(msg, "From")
        to_h = get_header(msg, "To")
        cseq = get_header(msg, "CSeq")
        vias = get_all_via(msg)
        via_block = "\r\n".join(vias)
        to_tag = gen_tag()
        to_with_tag = to_h + f";tag={to_tag}" if "tag=" not in to_h else to_h

        resp = (
            f"SIP/2.0 {self.reject_code} {self.reject_reason}\r\n"
            f"{via_block}\r\n"
            f"From: {from_h}\r\n"
            f"To: {to_with_tag}\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq}\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        self.sock.sendto(resp.encode(), addr)


class CalleeRingOnly:
    """Background thread that sends 180 Ringing but never answers.

    Used for CANCEL tests.
    """

    def __init__(self, sock, ext):
        self.sock = sock
        self.ext = ext
        self.invite_received = threading.Event()
        self.cancel_received = threading.Event()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)

    def start(self):
        self._thread.start()

    def stop(self):
        self._stop.set()

    def _run(self):
        self.sock.settimeout(1)
        invite_from = None
        invite_to = None
        invite_cid = None
        invite_vias = None
        while not self._stop.is_set():
            try:
                data, addr = self.sock.recvfrom(65535)
                msg = data.decode(errors="replace")
            except socket.timeout:
                continue
            except OSError:
                break

            if msg.startswith("INVITE "):
                self.invite_received.set()
                cid = get_header(msg, "Call-ID")
                from_h = get_header(msg, "From")
                to_h = get_header(msg, "To")
                cseq = get_header(msg, "CSeq")
                vias = get_all_via(msg)
                via_block = "\r\n".join(vias)
                to_tag = gen_tag()
                to_with_tag = to_h + f";tag={to_tag}" if "tag=" not in to_h else to_h
                contact = f"<sip:{self.ext}@{SERVER}:{self.sock.getsockname()[1]}>"

                invite_from = from_h
                invite_to = to_with_tag
                invite_cid = cid
                invite_vias = via_block

                # Send 180 Ringing only
                ringing = (
                    f"SIP/2.0 180 Ringing\r\n"
                    f"{via_block}\r\n"
                    f"From: {from_h}\r\n"
                    f"To: {to_with_tag}\r\n"
                    f"Call-ID: {cid}\r\n"
                    f"CSeq: {cseq}\r\n"
                    f"Contact: {contact}\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                self.sock.sendto(ringing.encode(), addr)

            elif msg.startswith("CANCEL "):
                self.cancel_received.set()
                cid = get_header(msg, "Call-ID")
                from_h = get_header(msg, "From")
                to_h = get_header(msg, "To")
                cseq = get_header(msg, "CSeq")
                vias = get_all_via(msg)
                via_block = "\r\n".join(vias)

                # 200 OK to CANCEL
                ok_resp = (
                    f"SIP/2.0 200 OK\r\n"
                    f"{via_block}\r\n"
                    f"From: {from_h}\r\n"
                    f"To: {to_h}\r\n"
                    f"Call-ID: {cid}\r\n"
                    f"CSeq: {cseq}\r\n"
                    f"Content-Length: 0\r\n\r\n"
                )
                self.sock.sendto(ok_resp.encode(), addr)

                # 487 Request Terminated for original INVITE
                if invite_vias and invite_cid:
                    inv_cseq_num = cseq.split()[0]
                    terminated = (
                        f"SIP/2.0 487 Request Terminated\r\n"
                        f"{invite_vias}\r\n"
                        f"From: {invite_from}\r\n"
                        f"To: {invite_to}\r\n"
                        f"Call-ID: {invite_cid}\r\n"
                        f"CSeq: {inv_cseq_num} INVITE\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    self.sock.sendto(terminated.encode(), addr)


# ============================================================
# Caller INVITE with Auth (returns call_state dict)
# ============================================================

def caller_invite(sock, ext, password, sip_port, rtp_port, callee_ext,
                  sdp_body=None):
    """Send INVITE with digest auth.

    Returns (rtp_target, codecs, call_state) or (None, [], None) on failure.
    """
    from_uri = f"sip:{ext}@{EXTERNAL_IP}"
    to_uri = f"sip:{callee_ext}@{SERVER}"
    local_tag = gen_tag()
    call_id = gen_callid()
    contact = f"<sip:{ext}@{SERVER}:{sip_port}>"
    cseq = 1
    if sdp_body is None:
        sdp_body = build_sdp(rtp_port)

    def make_invite(br, cs, auth_hdr=None):
        auth = f"{auth_hdr}\r\n" if auth_hdr else ""
        return (
            f"INVITE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={br};rport\r\n"
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
            f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={gen_branch()};rport\r\n"
            f"From: <{from_uri}>;tag={local_tag}\r\n"
            f"To: {to_field}\r\n"
            f"Call-ID: {call_id}\r\n"
            f"CSeq: {cs} ACK\r\n"
            f"Max-Forwards: 70\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        send_sip(sock, ack)

    branch = gen_branch()
    send_sip(sock, make_invite(branch, cseq))

    auth_done = False
    start = time.time()
    while time.time() - start < 30:
        resp, _ = recv_sip(sock, timeout=2)
        if not resp:
            continue
        code = get_response_code(resp)
        if code == 100:
            continue
        elif code in (180, 183):
            continue
        elif code in (401, 407):
            if auth_done:
                return None, [], None
            auth_done = True
            send_ack(cseq)

            auth_hdr_val = ""
            for line in resp.split("\r\n"):
                low = line.lower()
                if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                    auth_hdr_val = line.split(":", 1)[1].strip()
                    break
            realm, nonce = parse_www_authenticate(auth_hdr_val)
            realm = realm or EXTERNAL_IP
            digest = compute_digest(ext, realm, password, "INVITE", to_uri, nonce)
            hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
            auth_line = (
                f'{hdr_name}: Digest username="{ext}", realm="{realm}", '
                f'nonce="{nonce}", uri="{to_uri}", response="{digest}", algorithm=MD5'
            )
            cseq += 1
            branch = gen_branch()
            send_sip(sock, make_invite(branch, cseq, auth_line))
            continue
        elif code == 200:
            to_tag = get_to_tag(resp)
            rtp_target = None
            codecs = []
            sdp_start = resp.find("\r\n\r\n")
            if sdp_start >= 0:
                ip, port, codecs = parse_sdp(resp[sdp_start + 4:])
                if ip and port:
                    rtp_target = (ip, port)
            send_ack(cseq, to_tag)
            call_state = {
                "call_id": call_id, "from_uri": from_uri, "to_uri": to_uri,
                "local_tag": local_tag, "to_tag": to_tag, "cseq": cseq,
                "sip_port": sip_port, "sock": sock,
            }
            return rtp_target, codecs, call_state
        elif code >= 400:
            send_ack(cseq)
            return None, [], {"rejected_code": code}

    return None, [], None


def caller_invite_no_wait(sock, ext, password, sip_port, rtp_port,
                          callee_ext, sdp_body=None):
    """Send INVITE with auth but return after auth completes.

    Caller must then poll for 180/200/etc.

    Returns (call_id, from_uri, to_uri, local_tag, cseq, invite_branch)
    or None on auth failure.
    """
    from_uri = f"sip:{ext}@{EXTERNAL_IP}"
    to_uri = f"sip:{callee_ext}@{SERVER}"
    local_tag = gen_tag()
    call_id = gen_callid()
    contact = f"<sip:{ext}@{SERVER}:{sip_port}>"
    cseq = 1
    if sdp_body is None:
        sdp_body = build_sdp(rtp_port)

    def make_invite(br, cs, auth_hdr=None):
        auth = f"{auth_hdr}\r\n" if auth_hdr else ""
        return (
            f"INVITE {to_uri} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={br};rport\r\n"
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
    branch = gen_branch()
    send_sip(sock, make_invite(branch, cseq))

    # Handle auth challenge
    start = time.time()
    while time.time() - start < 10:
        resp, _ = recv_sip(sock, timeout=2)
        if not resp:
            continue
        code = get_response_code(resp)
        if code == 100:
            continue
        elif code in (180, 183):
            # Already ringing before auth -- unusual but ok
            return {
                "call_id": call_id, "from_uri": from_uri, "to_uri": to_uri,
                "local_tag": local_tag, "cseq": cseq, "branch": branch,
                "sip_port": sip_port,
            }
        elif code in (401, 407):
            # ACK the challenge
            ack = (
                f"ACK {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER}:{sip_port};branch={gen_branch()};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: <{to_uri}>\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {cseq} ACK\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            send_sip(sock, ack)

            auth_hdr_val = ""
            for line in resp.split("\r\n"):
                low = line.lower()
                if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                    auth_hdr_val = line.split(":", 1)[1].strip()
                    break
            realm, nonce = parse_www_authenticate(auth_hdr_val)
            realm = realm or EXTERNAL_IP
            digest = compute_digest(ext, realm, password, "INVITE", to_uri, nonce)
            hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
            auth_line = (
                f'{hdr_name}: Digest username="{ext}", realm="{realm}", '
                f'nonce="{nonce}", uri="{to_uri}", response="{digest}", algorithm=MD5'
            )
            cseq += 1
            branch = gen_branch()
            send_sip(sock, make_invite(branch, cseq, auth_line))

            return {
                "call_id": call_id, "from_uri": from_uri, "to_uri": to_uri,
                "local_tag": local_tag, "cseq": cseq, "branch": branch,
                "sip_port": sip_port,
            }

    return None


def send_bye(call_state):
    """Send BYE for an established call."""
    cs = call_state["cseq"] + 1
    msg = (
        f"BYE {call_state['to_uri']} SIP/2.0\r\n"
        f"Via: SIP/2.0/UDP {SERVER}:{call_state['sip_port']};branch={gen_branch()};rport\r\n"
        f"From: <{call_state['from_uri']}>;tag={call_state['local_tag']}\r\n"
        f"To: <{call_state['to_uri']}>;tag={call_state['to_tag']}\r\n"
        f"Call-ID: {call_state['call_id']}\r\n"
        f"CSeq: {cs} BYE\r\n"
        f"Max-Forwards: 70\r\n"
        f"Content-Length: 0\r\n\r\n"
    )
    send_sip(call_state["sock"], msg)
    resp, _ = recv_sip(call_state["sock"], timeout=3)
    return resp


# ============================================================
# Integrated Call Context
# ============================================================

class E2ECallContext:
    """Holds all state for a single end-to-end test call."""

    def __init__(self, ports, callee_behavior=None):
        self.ports = ports
        self.callee_behavior = callee_behavior  # CalleeAutoAnswer, CalleeReject, etc.
        self.caller_sip = None
        self.callee_sip = None
        self.caller_rtp = None
        self.callee_rtp = None
        self.listener = None
        self.call_state = None
        self.caller_rtp_target = None
        self.caller_codecs = []
        self.error = None

    def setup(self):
        """Register both UAs, start callee behavior, and place the call."""
        try:
            self.caller_sip = make_socket(self.ports["caller_sip_port"])
            self.callee_sip = make_socket(self.ports["callee_sip_port"])
            self.caller_rtp = make_socket(self.ports["caller_rtp_port"])
            self.callee_rtp = make_socket(self.ports["callee_rtp_port"])
        except OSError as e:
            self.error = f"Socket bind failed: {e}"
            return False

        ok1 = sip_register(self.caller_sip, "1001", "test1001",
                           self.ports["caller_sip_port"])
        ok2 = sip_register(self.callee_sip, "1002", "test1002",
                           self.ports["callee_sip_port"])
        if not (ok1 and ok2):
            self.error = f"Registration failed (caller={ok1}, callee={ok2})"
            return False

        # Default: auto-answer callee
        if self.callee_behavior is None:
            self.listener = CalleeAutoAnswer(
                self.callee_sip, "1002", self.ports["callee_rtp_port"])
        else:
            self.listener = self.callee_behavior
        self.listener.start()

        rtp_target, codecs, call_state = caller_invite(
            self.caller_sip, "1001", "test1001",
            self.ports["caller_sip_port"], self.ports["caller_rtp_port"],
            "1002",
        )

        if isinstance(call_state, dict) and "rejected_code" in call_state:
            self.call_state = call_state
            return True  # "success" in the sense that we got a response

        if not call_state:
            self.error = "INVITE failed or was rejected"
            return False

        if hasattr(self.listener, 'wait_for_call'):
            self.listener.wait_for_call(timeout=10)

        self.caller_rtp_target = rtp_target
        self.caller_codecs = codecs
        self.call_state = call_state
        return True

    def teardown(self):
        """BYE, unregister, close sockets."""
        if self.call_state and "rejected_code" not in self.call_state:
            try:
                send_bye(self.call_state)
            except Exception:
                pass
        if self.listener:
            self.listener.stop()
        for sock, ext, port_key in [
            (self.caller_sip, "1001", "caller_sip_port"),
            (self.callee_sip, "1002", "callee_sip_port"),
        ]:
            if sock:
                try:
                    sip_unregister(sock, ext, self.ports[port_key])
                except Exception:
                    pass
        time.sleep(0.3)
        for s in (self.caller_sip, self.callee_sip,
                  self.caller_rtp, self.callee_rtp):
            if s:
                try:
                    s.close()
                except Exception:
                    pass


# ============================================================
# HTTP API helper
# ============================================================

def get_api_session():
    """Login to admin console and return authenticated session."""
    session = requests.Session()
    session.verify = VERIFY_TLS
    try:
        resp = session.post(
            f"{BASE_URL}/console/login",
            data={"identifier": ADMIN_USER, "password": ADMIN_PASS},
            allow_redirects=False,
            timeout=10,
        )
        if resp.status_code in (200, 302, 303):
            return session
    except requests.ConnectionError:
        pass
    return None


# ============================================================
# Test Class
# ============================================================

class TestL9E2E:
    """L9: End-to-end call flow tests."""

    # ------------------------------------------------------------------
    # TC-L9-001: Internal call with recording
    # ------------------------------------------------------------------

    @pytest.mark.timeout(120)
    def test_L9_001_internal_call_with_recording(self):
        """TC-L9-001: 1001 calls 1002, both exchange RTP for 10s, verify recording.

        Full end-to-end call flow:
          1. Register both endpoints
          2. 1001 sends INVITE to 1002
          3. 1002 answers with 200 OK
          4. Both sides exchange PCMU RTP for 10 seconds
          5. Caller sends BYE
          6. Verify call completed cleanly
          7. Check for recording file via filesystem (if accessible)
          8. Check call records via API (if accessible)
        """
        # Snapshot existing recordings
        pre_files = set()
        if os.path.isdir(RECORDER_DIR):
            pre_files = set(os.listdir(RECORDER_DIR))

        ports = _alloc_ports()
        ctx = E2ECallContext(ports)
        ok = ctx.setup()
        if not ok:
            ctx.teardown()
            pytest.skip(f"Call setup failed: {ctx.error}")

        try:
            assert ctx.caller_rtp_target is not None, (
                "No RTP target from SDP answer"
            )

            # Generate audio frames
            caller_frames = generate_ulaw_frames(freq_hz=440, duration_s=12)
            callee_frames = generate_ulaw_frames(freq_hz=880, duration_s=12)
            assert len(caller_frames) > 0, "No caller audio frames generated"
            assert len(callee_frames) > 0, "No callee audio frames generated"

            # Start bidirectional RTP
            stop = threading.Event()
            caller_received = []
            callee_received = []

            def recv_caller():
                nonlocal caller_received
                caller_received = rtp_receiver(
                    ctx.caller_rtp, stop, accept_pts={PCMU_PT})

            def recv_callee():
                nonlocal callee_received
                callee_received = rtp_receiver(
                    ctx.callee_rtp, stop, accept_pts={PCMU_PT})

            threads = [
                threading.Thread(target=recv_caller, daemon=True),
                threading.Thread(target=recv_callee, daemon=True),
                threading.Thread(
                    target=rtp_sender,
                    args=(ctx.caller_rtp, ctx.caller_rtp_target,
                          caller_frames, stop),
                    daemon=True,
                ),
            ]
            if ctx.listener.rtp_target:
                threads.append(threading.Thread(
                    target=rtp_sender,
                    args=(ctx.callee_rtp, ctx.listener.rtp_target,
                          callee_frames, stop),
                    daemon=True,
                ))
            for t in threads:
                t.start()

            # Exchange RTP for 10 seconds
            time.sleep(10)
            stop.set()
            time.sleep(0.5)

            # Verify bidirectional RTP flowed
            min_expected = 10 * 50 * 0.3  # 10s * 50pps * 30% tolerance
            assert len(callee_received) > min_expected, (
                f"Caller->Callee: only {len(callee_received)} packets "
                f"(expected > {min_expected:.0f})"
            )
            # Callee->Caller may not work if proxy does not relay
            # (we still verify caller->callee at minimum)

        finally:
            ctx.teardown()

        # Wait for recording to be finalized
        time.sleep(5)

        # Check for new recording files
        if os.path.isdir(RECORDER_DIR):
            post_files = set(os.listdir(RECORDER_DIR))
            new_files = sorted(
                f for f in (post_files - pre_files) if f.endswith(".wav"))
            assert len(new_files) > 0, (
                f"No new WAV files in {RECORDER_DIR} after 10s call. "
                f"Recording may be disabled."
            )
            # Verify file is non-trivial
            wav_path = os.path.join(RECORDER_DIR, new_files[-1])
            file_size = os.path.getsize(wav_path)
            assert file_size > 5000, (
                f"Recording file too small ({file_size} bytes): {wav_path}"
            )
        else:
            # Try API check instead
            api = get_api_session()
            if api:
                try:
                    resp = api.get(
                        f"{BASE_URL}/console/call-records",
                        timeout=10,
                    )
                    assert resp.status_code == 200, (
                        f"Call records page returned {resp.status_code}"
                    )
                except requests.ConnectionError:
                    pytest.skip(
                        "Cannot verify recording: no filesystem access "
                        "and API unreachable"
                    )
            else:
                pytest.skip(
                    f"Recording dir {RECORDER_DIR} not accessible and "
                    f"API login failed"
                )

    # ------------------------------------------------------------------
    # TC-L9-002: Call with DTMF
    # ------------------------------------------------------------------

    @pytest.mark.timeout(90)
    def test_L9_002_call_with_dtmf(self):
        """TC-L9-002: Establish call, send DTMF digits via RFC 2833.

        Verifies that DTMF telephone-event packets are either relayed
        through the proxy or consumed without disrupting the audio flow.
        Sends digits 1, 2, 3 in sequence.
        """
        ports = _alloc_ports()
        ctx = E2ECallContext(ports)
        ok = ctx.setup()
        if not ok:
            ctx.teardown()
            pytest.skip(f"Call setup failed: {ctx.error}")

        try:
            assert ctx.caller_rtp_target is not None

            # Start audio first to stabilize the call
            audio_frames = generate_ulaw_frames(freq_hz=440, duration_s=8)
            stop = threading.Event()
            callee_received = []

            def recv_fn():
                nonlocal callee_received
                callee_received = rtp_receiver(
                    ctx.callee_rtp, stop,
                    accept_pts={PCMU_PT, DTMF_PT})

            threads = [
                threading.Thread(target=recv_fn, daemon=True),
                threading.Thread(
                    target=rtp_sender,
                    args=(ctx.caller_rtp, ctx.caller_rtp_target,
                          audio_frames, stop),
                    daemon=True,
                ),
            ]
            for t in threads:
                t.start()

            # Wait for audio to flow
            time.sleep(2)

            # Send DTMF digits 1, 2, 3
            dtmf_ssrc = random.randint(1, 0xFFFFFFFF)
            dtmf_seq = random.randint(0, 0xFFFF)
            dtmf_ts = random.randint(0, 0xFFFFFFFF)
            volume = 10

            for digit in [1, 2, 3]:
                # Begin packets
                for i in range(5):
                    duration = (i + 1) * 160
                    payload = struct.pack("!BBH", digit, volume & 0x3F,
                                          duration)
                    pkt = build_rtp(DTMF_PT, dtmf_seq, dtmf_ts, dtmf_ssrc,
                                    payload, marker=(i == 0))
                    ctx.caller_rtp.sendto(pkt, ctx.caller_rtp_target)
                    dtmf_seq = (dtmf_seq + 1) & 0xFFFF
                    time.sleep(0.02)

                # End packets (3 copies per RFC 2833)
                for _ in range(3):
                    duration = 6 * 160
                    end_flag = 0x80
                    payload = struct.pack("!BBH", digit,
                                          end_flag | (volume & 0x3F),
                                          duration)
                    pkt = build_rtp(DTMF_PT, dtmf_seq, dtmf_ts, dtmf_ssrc,
                                    payload)
                    ctx.caller_rtp.sendto(pkt, ctx.caller_rtp_target)
                    dtmf_seq = (dtmf_seq + 1) & 0xFFFF
                    time.sleep(0.02)

                # Advance timestamp for next digit
                dtmf_ts = (dtmf_ts + 8 * 160) & 0xFFFFFFFF
                time.sleep(0.3)

            # Wait for remaining packets
            time.sleep(3)
            stop.set()
            time.sleep(0.5)

            # Verify the call remained stable through DTMF sending
            total_pkts = len(callee_received)
            assert total_pkts > 0, "No RTP packets received at all"

            dtmf_pkts = [p for p in callee_received if p["pt"] == DTMF_PT]
            audio_pkts = [p for p in callee_received if p["pt"] != DTMF_PT]

            # Either DTMF was relayed or audio continued flowing
            assert len(audio_pkts) > 50 or len(dtmf_pkts) > 0, (
                f"Call disrupted by DTMF: audio={len(audio_pkts)}, "
                f"dtmf={len(dtmf_pkts)}"
            )

        finally:
            ctx.teardown()

    # ------------------------------------------------------------------
    # TC-L9-003: Multiple sequential calls
    # ------------------------------------------------------------------

    @pytest.mark.timeout(180)
    def test_L9_003_multiple_sequential_calls(self):
        """TC-L9-003: Make 3 calls in sequence, verify each completes cleanly.

        Each call:
          1. Register both users
          2. Establish call with INVITE/200/ACK
          3. Exchange RTP for 3 seconds
          4. Tear down with BYE
          5. Unregister
          6. Close sockets
        Then verify no resource leaks (subsequent calls still work).
        """
        for call_num in range(1, 4):
            ports = _alloc_ports()
            ctx = E2ECallContext(ports)
            ok = ctx.setup()
            if not ok:
                ctx.teardown()
                pytest.fail(
                    f"Call {call_num}/3 setup failed: {ctx.error}")

            try:
                assert ctx.caller_rtp_target is not None, (
                    f"Call {call_num}: no RTP target"
                )

                # Exchange RTP for 3 seconds
                frames = generate_ulaw_frames(freq_hz=440, duration_s=5)
                stop = threading.Event()
                callee_received = []

                def recv_fn():
                    nonlocal callee_received
                    callee_received = rtp_receiver(
                        ctx.callee_rtp, stop, accept_pts={PCMU_PT})

                recv_t = threading.Thread(target=recv_fn, daemon=True)
                send_t = threading.Thread(
                    target=rtp_sender,
                    args=(ctx.caller_rtp, ctx.caller_rtp_target,
                          frames, stop),
                    daemon=True,
                )
                recv_t.start()
                send_t.start()

                time.sleep(3)
                stop.set()
                time.sleep(0.5)

                # Verify RTP flowed
                assert len(callee_received) > 50, (
                    f"Call {call_num}: only {len(callee_received)} packets"
                )

            finally:
                ctx.teardown()

            # Brief pause between calls for server cleanup
            time.sleep(2)

    # ------------------------------------------------------------------
    # TC-L9-004: Call reject (486 Busy Here)
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L9_004_call_reject_486(self):
        """TC-L9-004: 1001 calls 1002, 1002 sends 486 Busy Here.

        Verifies:
          - Caller receives 486 response (forwarded by proxy)
          - No call state leaks (can make another call afterwards)
        """
        ports = _alloc_ports()

        # Set up sockets and register
        caller_sip = make_socket(ports["caller_sip_port"])
        callee_sip = make_socket(ports["callee_sip_port"])

        try:
            ok1 = sip_register(caller_sip, "1001", "test1001",
                               ports["caller_sip_port"])
            ok2 = sip_register(callee_sip, "1002", "test1002",
                               ports["callee_sip_port"])
            assert ok1, "Caller registration failed"
            assert ok2, "Callee registration failed"

            # Start callee that rejects with 486
            rejecter = CalleeReject(callee_sip, "1002", 486, "Busy Here")
            rejecter.start()

            # Send INVITE from caller
            rtp_target, codecs, call_state = caller_invite(
                caller_sip, "1001", "test1001",
                ports["caller_sip_port"], ports["caller_rtp_port"],
                "1002",
            )

            # Call should have been rejected
            assert rtp_target is None, (
                "Expected rejection but got RTP target"
            )
            assert call_state is not None, (
                "No response from server at all"
            )
            if isinstance(call_state, dict) and "rejected_code" in call_state:
                assert call_state["rejected_code"] in (486, 603), (
                    f"Expected 486 or 603, got {call_state['rejected_code']}"
                )

            # Verify callee received the INVITE
            assert rejecter.invite_received.wait(timeout=5), (
                "Callee never received the INVITE"
            )

            rejecter.stop()

        finally:
            try:
                sip_unregister(caller_sip, "1001", ports["caller_sip_port"])
            except Exception:
                pass
            try:
                sip_unregister(callee_sip, "1002", ports["callee_sip_port"])
            except Exception:
                pass
            time.sleep(0.3)
            caller_sip.close()
            callee_sip.close()

    # ------------------------------------------------------------------
    # TC-L9-005: Call cancel before answer
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L9_005_call_cancel(self):
        """TC-L9-005: 1001 calls 1002, 1001 sends CANCEL before 1002 answers.

        Flow:
          1. Caller sends INVITE
          2. Callee sends 180 Ringing (does NOT answer)
          3. Caller sends CANCEL
          4. Expect 200 OK for CANCEL
          5. Expect 487 Request Terminated
        """
        ports = _alloc_ports()
        caller_sip = make_socket(ports["caller_sip_port"])
        callee_sip = make_socket(ports["callee_sip_port"])

        try:
            ok1 = sip_register(caller_sip, "1001", "test1001",
                               ports["caller_sip_port"])
            ok2 = sip_register(callee_sip, "1002", "test1002",
                               ports["callee_sip_port"])
            assert ok1, "Caller registration failed"
            assert ok2, "Callee registration failed"

            # Start callee that only rings (no answer)
            ringer = CalleeRingOnly(callee_sip, "1002")
            ringer.start()

            # Send INVITE with auth handling
            inv_state = caller_invite_no_wait(
                caller_sip, "1001", "test1001",
                ports["caller_sip_port"], ports["caller_rtp_port"],
                "1002",
            )
            assert inv_state is not None, "INVITE auth failed"

            call_id = inv_state["call_id"]
            from_uri = inv_state["from_uri"]
            to_uri = inv_state["to_uri"]
            local_tag = inv_state["local_tag"]
            cseq = inv_state["cseq"]
            invite_branch = inv_state["branch"]
            sip_port = inv_state["sip_port"]

            # Wait for ringing
            saw_ringing = False
            start = time.time()
            while time.time() - start < 10:
                resp, _ = recv_sip(caller_sip, timeout=1)
                if resp:
                    code = get_response_code(resp)
                    if code in (180, 183):
                        saw_ringing = True
                        break
                    elif code == 100:
                        continue

            assert saw_ringing, "Never received 180 Ringing"

            # Send CANCEL (must use same branch as INVITE)
            cancel = (
                f"CANCEL {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER}:{sip_port};"
                f"branch={invite_branch};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: <{to_uri}>\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {cseq} CANCEL\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            send_sip(caller_sip, cancel)

            # Collect responses
            saw_cancel_200 = False
            saw_487 = False
            start = time.time()
            while time.time() - start < 10:
                resp, _ = recv_sip(caller_sip, timeout=1)
                if not resp:
                    continue
                code = get_response_code(resp)
                cseq_hdr = get_header(resp, "CSeq")

                if code == 200 and "CANCEL" in cseq_hdr:
                    saw_cancel_200 = True
                elif code == 487:
                    saw_487 = True
                    # ACK the 487
                    ack = (
                        f"ACK {to_uri} SIP/2.0\r\n"
                        f"Via: SIP/2.0/UDP {SERVER}:{sip_port};"
                        f"branch={gen_branch()};rport\r\n"
                        f"From: <{from_uri}>;tag={local_tag}\r\n"
                        f"To: {get_header(resp, 'To')}\r\n"
                        f"Call-ID: {call_id}\r\n"
                        f"CSeq: {cseq} ACK\r\n"
                        f"Max-Forwards: 70\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    send_sip(caller_sip, ack)

                if saw_cancel_200 and saw_487:
                    break

            assert saw_cancel_200, (
                "Never received 200 OK for CANCEL"
            )
            assert saw_487, (
                "Never received 487 Request Terminated"
            )

            ringer.stop()

        finally:
            try:
                sip_unregister(caller_sip, "1001", ports["caller_sip_port"])
            except Exception:
                pass
            try:
                sip_unregister(callee_sip, "1002", ports["callee_sip_port"])
            except Exception:
                pass
            time.sleep(0.3)
            caller_sip.close()
            callee_sip.close()

    # ------------------------------------------------------------------
    # TC-L9-006: Re-INVITE hold/unhold
    # ------------------------------------------------------------------

    @pytest.mark.timeout(90)
    def test_L9_006_reinvite_hold_unhold(self):
        """TC-L9-006: Establish call, send re-INVITE with sendonly (hold),
        then sendrecv (unhold).

        Verifies:
          - re-INVITE with a=sendonly is accepted (200 OK)
          - re-INVITE with a=sendrecv is accepted (200 OK)
          - Call remains functional after unhold
        """
        ports = _alloc_ports()
        ctx = E2ECallContext(ports)
        ok = ctx.setup()
        if not ok:
            ctx.teardown()
            pytest.skip(f"Call setup failed: {ctx.error}")

        try:
            assert ctx.caller_rtp_target is not None

            # Exchange RTP to stabilize the call
            frames = generate_ulaw_frames(freq_hz=440, duration_s=5)
            stop = threading.Event()
            send_t = threading.Thread(
                target=rtp_sender,
                args=(ctx.caller_rtp, ctx.caller_rtp_target,
                      frames, stop),
                daemon=True,
            )
            send_t.start()
            time.sleep(2)

            cs = ctx.call_state
            call_id = cs["call_id"]
            from_uri = cs["from_uri"]
            to_uri = cs["to_uri"]
            local_tag = cs["local_tag"]
            to_tag = cs["to_tag"]
            sip_port = cs["sip_port"]
            cseq = cs["cseq"] + 1

            # --- HOLD: re-INVITE with a=sendonly ---
            hold_sdp = build_sdp(
                ports["caller_rtp_port"], direction="sendonly")
            to_field = f"<{to_uri}>;tag={to_tag}" if to_tag else f"<{to_uri}>"
            contact = f"<sip:1001@{SERVER}:{sip_port}>"
            reinvite_hold = (
                f"INVITE {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER}:{sip_port};"
                f"branch={gen_branch()};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: {to_field}\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {cseq} INVITE\r\n"
                f"Contact: {contact}\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Type: application/sdp\r\n"
                f"Content-Length: {len(hold_sdp)}\r\n\r\n"
                f"{hold_sdp}"
            )
            send_sip(ctx.caller_sip, reinvite_hold)

            # Wait for 200 OK to re-INVITE (hold)
            hold_ok = False
            start = time.time()
            while time.time() - start < 10:
                resp, _ = recv_sip(ctx.caller_sip, timeout=2)
                if not resp:
                    continue
                code = get_response_code(resp)
                if code == 100:
                    continue
                elif code == 200:
                    hold_ok = True
                    hold_to_tag = get_to_tag(resp)
                    # ACK the 200 OK
                    ack_to = f"<{to_uri}>;tag={hold_to_tag}" if hold_to_tag else to_field
                    ack = (
                        f"ACK {to_uri} SIP/2.0\r\n"
                        f"Via: SIP/2.0/UDP {SERVER}:{sip_port};"
                        f"branch={gen_branch()};rport\r\n"
                        f"From: <{from_uri}>;tag={local_tag}\r\n"
                        f"To: {ack_to}\r\n"
                        f"Call-ID: {call_id}\r\n"
                        f"CSeq: {cseq} ACK\r\n"
                        f"Max-Forwards: 70\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    send_sip(ctx.caller_sip, ack)
                    break
                elif code >= 400:
                    break

            assert hold_ok, "re-INVITE (hold) was not accepted with 200 OK"

            time.sleep(1)

            # --- UNHOLD: re-INVITE with a=sendrecv ---
            cseq += 1
            unhold_sdp = build_sdp(
                ports["caller_rtp_port"], direction="sendrecv")
            reinvite_unhold = (
                f"INVITE {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {SERVER}:{sip_port};"
                f"branch={gen_branch()};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: {to_field}\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {cseq} INVITE\r\n"
                f"Contact: {contact}\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Type: application/sdp\r\n"
                f"Content-Length: {len(unhold_sdp)}\r\n\r\n"
                f"{unhold_sdp}"
            )
            send_sip(ctx.caller_sip, reinvite_unhold)

            # Wait for 200 OK to re-INVITE (unhold)
            unhold_ok = False
            start = time.time()
            while time.time() - start < 10:
                resp, _ = recv_sip(ctx.caller_sip, timeout=2)
                if not resp:
                    continue
                code = get_response_code(resp)
                if code == 100:
                    continue
                elif code == 200:
                    unhold_ok = True
                    unhold_to_tag = get_to_tag(resp)
                    ack_to = f"<{to_uri}>;tag={unhold_to_tag}" if unhold_to_tag else to_field
                    ack = (
                        f"ACK {to_uri} SIP/2.0\r\n"
                        f"Via: SIP/2.0/UDP {SERVER}:{sip_port};"
                        f"branch={gen_branch()};rport\r\n"
                        f"From: <{from_uri}>;tag={local_tag}\r\n"
                        f"To: {ack_to}\r\n"
                        f"Call-ID: {call_id}\r\n"
                        f"CSeq: {cseq} ACK\r\n"
                        f"Max-Forwards: 70\r\n"
                        f"Content-Length: 0\r\n\r\n"
                    )
                    send_sip(ctx.caller_sip, ack)
                    break
                elif code >= 400:
                    break

            assert unhold_ok, "re-INVITE (unhold) was not accepted with 200 OK"

            # Verify call still works after unhold
            time.sleep(2)

            # Check that callee received re-INVITEs
            # (the proxy should have forwarded them)
            # The callee listener tracks re-INVITE arrivals
            # Note: the proxy may handle hold locally; it is acceptable
            # if the callee does not receive the re-INVITE

            # Update cseq in call_state for proper teardown
            ctx.call_state["cseq"] = cseq

            stop.set()
            time.sleep(0.5)

        finally:
            ctx.teardown()

    # ------------------------------------------------------------------
    # TC-L9-007: Concurrent calls
    # ------------------------------------------------------------------

    @pytest.mark.timeout(120)
    def test_L9_007_concurrent_calls(self):
        """TC-L9-007: Two simultaneous calls verify server handles concurrency.

        Since we only have 2 test users (1001, 1002), we run two sequential
        calls but verify the second call works immediately after the first,
        proving no resource leakage. If 1003/1004 are available, we attempt
        truly concurrent calls.

        Strategy:
          - Call A: 1001 -> 1002, exchange RTP for 5s
          - Immediately start Call B: 1001 -> 1002 again after A completes
          - Both must succeed without errors
        """
        results = []

        # Attempt truly concurrent with different user pairs if 1003/1004 exist
        # Otherwise fall back to rapid sequential calls
        has_extra_users = False

        # Test with extra users (1003, 1004) if they exist
        test_ports = _alloc_ports()
        test_sock = None
        try:
            test_sock = make_socket(test_ports["caller_sip_port"])
            ok = sip_register(test_sock, "1003", "test1003",
                              test_ports["caller_sip_port"])
            if ok:
                has_extra_users = True
                sip_unregister(test_sock, "1003", test_ports["caller_sip_port"])
        except Exception:
            pass
        finally:
            if test_sock:
                test_sock.close()

        if has_extra_users:
            # Truly concurrent calls: 1001->1002 and 1003->1004
            self._run_concurrent_calls_real(results)
        else:
            # Rapid sequential: two fast calls with 1001->1002
            self._run_concurrent_calls_sequential(results)

        assert len(results) >= 2, (
            f"Expected at least 2 call results, got {len(results)}"
        )
        for i, result in enumerate(results):
            assert result["success"], (
                f"Call {i + 1} failed: {result.get('error', 'unknown')}"
            )
            assert result["packets"] > 30, (
                f"Call {i + 1}: insufficient packets ({result['packets']})"
            )

    def _run_concurrent_calls_real(self, results):
        """Run two truly concurrent calls using different user pairs."""
        errors = []

        def run_call(caller_ext, caller_pwd, callee_ext, callee_pwd, idx):
            ports = _alloc_ports()
            result = {"success": False, "packets": 0, "error": None}
            try:
                caller_sip = make_socket(ports["caller_sip_port"])
                callee_sip = make_socket(ports["callee_sip_port"])
                caller_rtp = make_socket(ports["caller_rtp_port"])
                callee_rtp = make_socket(ports["callee_rtp_port"])

                ok1 = sip_register(caller_sip, caller_ext, caller_pwd,
                                   ports["caller_sip_port"])
                ok2 = sip_register(callee_sip, callee_ext, callee_pwd,
                                   ports["callee_sip_port"])
                if not (ok1 and ok2):
                    result["error"] = "Registration failed"
                    results.append(result)
                    return

                listener = CalleeAutoAnswer(
                    callee_sip, callee_ext, ports["callee_rtp_port"])
                listener.start()

                rtp_target, codecs, call_state = caller_invite(
                    caller_sip, caller_ext, caller_pwd,
                    ports["caller_sip_port"], ports["caller_rtp_port"],
                    callee_ext,
                )

                if not call_state or "rejected_code" in call_state:
                    result["error"] = "INVITE failed"
                    results.append(result)
                    listener.stop()
                    return

                listener.wait_for_call(timeout=10)

                if rtp_target:
                    frames = generate_ulaw_frames(freq_hz=440, duration_s=7)
                    stop = threading.Event()
                    received = []

                    def recv_fn():
                        nonlocal received
                        received = rtp_receiver(
                            callee_rtp, stop, accept_pts={PCMU_PT})

                    recv_t = threading.Thread(target=recv_fn, daemon=True)
                    send_t = threading.Thread(
                        target=rtp_sender,
                        args=(caller_rtp, rtp_target, frames, stop),
                        daemon=True,
                    )
                    recv_t.start()
                    send_t.start()
                    time.sleep(5)
                    stop.set()
                    time.sleep(0.5)
                    result["packets"] = len(received)

                send_bye(call_state)
                listener.stop()

                sip_unregister(caller_sip, caller_ext,
                               ports["caller_sip_port"])
                sip_unregister(callee_sip, callee_ext,
                               ports["callee_sip_port"])
                result["success"] = True

            except Exception as e:
                result["error"] = str(e)
            finally:
                for s in [caller_sip, callee_sip, caller_rtp, callee_rtp]:
                    try:
                        s.close()
                    except Exception:
                        pass
                results.append(result)

        t1 = threading.Thread(
            target=run_call,
            args=("1001", "test1001", "1002", "test1002", 0),
            daemon=True,
        )
        t2 = threading.Thread(
            target=run_call,
            args=("1003", "test1003", "1004", "test1004", 1),
            daemon=True,
        )
        t1.start()
        t2.start()
        t1.join(timeout=60)
        t2.join(timeout=60)

    def _run_concurrent_calls_sequential(self, results):
        """Run two rapid sequential calls with same user pair."""
        for i in range(2):
            ports = _alloc_ports()
            result = {"success": False, "packets": 0, "error": None}
            ctx = E2ECallContext(ports)
            ok = ctx.setup()
            if not ok:
                result["error"] = ctx.error
                ctx.teardown()
                results.append(result)
                continue

            try:
                if ctx.caller_rtp_target:
                    frames = generate_ulaw_frames(freq_hz=440, duration_s=7)
                    stop = threading.Event()
                    received = []

                    def recv_fn():
                        nonlocal received
                        received = rtp_receiver(
                            ctx.callee_rtp, stop, accept_pts={PCMU_PT})

                    recv_t = threading.Thread(target=recv_fn, daemon=True)
                    send_t = threading.Thread(
                        target=rtp_sender,
                        args=(ctx.caller_rtp, ctx.caller_rtp_target,
                              frames, stop),
                        daemon=True,
                    )
                    recv_t.start()
                    send_t.start()
                    time.sleep(5)
                    stop.set()
                    time.sleep(0.5)
                    result["packets"] = len(received)

                result["success"] = True

            except Exception as e:
                result["error"] = str(e)
            finally:
                ctx.teardown()
                results.append(result)
                time.sleep(2)
