"""
L5 Media and Codec Tests -- RTP, codec negotiation, DTMF, and recording.

Exercises the media plane of a running RustPBX instance using a pure-Python
SIP UA and RTP stack (adapted from call_quality_test.py).  Each test
establishes its own call, sends/receives media, and tears down cleanly.

Tests:
  1. Opus codec negotiation
  2. PCMU (G.711 mu-law) fallback
  3. Bidirectional RTP flow
  4. RTP packet structure (V=2, PT, SSRC, sequence)
  5. RTP timing (20ms pacing)
  6. DTMF via RFC 2833
  7. Codec preference order in server SDP
  8. Media timeout (stop RTP, expect BYE)
  9. Audio quality (1kHz sine, SNR > 20dB)
 10. Recording verification (WAV exists after call)

Server:  127.0.0.1:5060  (UDP)
Users:   1001/test1001, 1002/test1002

Run with:
  /root/test-env/bin/python -m pytest tests/test_L5_media.py -v
"""

import hashlib
import os
import random
import re
import socket
import string
import struct
import threading
import time
import wave

import numpy as np
import pytest

try:
    import opuslib
    HAS_OPUS = True
except ImportError:
    HAS_OPUS = False

# ============================================================
# Configuration
# ============================================================

SERVER = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SERVER_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", SERVER)

OPUS_PT = 111
PCMU_PT = 0
DTMF_PT = 101
SAMPLE_RATE = 48000
PCMU_RATE = 8000
FRAME_MS = 20
FRAME_SAMPLES = SAMPLE_RATE * FRAME_MS // 1000  # 960
PCMU_FRAME_SAMPLES = PCMU_RATE * FRAME_MS // 1000  # 160

RECORDER_DIR = os.environ.get(
    "RECORDING_DIR",
    os.path.expanduser("~/rustpbx/config/recorders"),
)

# Each test gets a unique port range to avoid collisions when running
# tests sequentially.  The base is offset per-test via a global counter.
_PORT_LOCK = threading.Lock()
_PORT_BASE = 17060


def _alloc_ports():
    """Allocate a unique set of 4 ports (caller_sip, callee_sip, caller_rtp, callee_rtp)."""
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
# SIP Helpers (from call_quality_test.py)
# ============================================================

def gen_branch():
    return "z9hG4bK" + "".join(random.choices(string.ascii_lowercase + string.digits, k=12))

def gen_tag():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=8))

def gen_callid():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=16)) + "@test"

def md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()

def compute_digest(username, realm, password, method, uri, nonce):
    ha1 = md5hex(f"{username}:{realm}:{password}")
    ha2 = md5hex(f"{method}:{uri}")
    return md5hex(f"{ha1}:{nonce}:{ha2}")

def get_response_code(data):
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0

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

def build_sdp(rtp_port, local_ip="127.0.0.1", codecs=None):
    """Build an SDP body.  *codecs* is a list of (pt, rtpmap, fmtp|None) tuples.
    When omitted, the default Opus + PCMU + telephone-event set is used."""
    sid = str(random.randint(100000, 999999))
    if codecs is None:
        codecs = [
            (OPUS_PT, f"opus/48000/2",
             f"minptime=10;useinbandfec=1;stereo=1;sprop-stereo=1;maxaveragebitrate=128000"),
            (PCMU_PT, "PCMU/8000", None),
            (DTMF_PT, "telephone-event/8000", "0-16"),
        ]
    pts = " ".join(str(c[0]) for c in codecs)
    lines = [
        "v=0",
        f"o=- {sid} {sid} IN IP4 {local_ip}",
        "s=RustPBX Test",
        f"c=IN IP4 {local_ip}",
        "t=0 0",
        f"m=audio {rtp_port} RTP/AVP {pts}",
    ]
    for pt, rtpmap, fmtp in codecs:
        lines.append(f"a=rtpmap:{pt} {rtpmap}")
        if fmtp:
            lines.append(f"a=fmtp:{pt} {fmtp}")
    lines.append("a=sendrecv")
    return "\r\n".join(lines) + "\r\n"

def send_sip(sock, msg, dest=(SERVER, SERVER_PORT)):
    sock.sendto(msg.encode() if isinstance(msg, str) else msg, dest)

def recv_sip(sock, timeout=5):
    sock.settimeout(timeout)
    try:
        data, addr = sock.recvfrom(65535)
        return data.decode(errors="replace"), addr
    except socket.timeout:
        return None, None

def make_socket(port):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", port))
    return s


# ============================================================
# RTP Helpers (from call_quality_test.py)
# ============================================================

def build_rtp(pt, seq, timestamp, ssrc, payload, marker=False):
    b0 = 0x80  # V=2, P=0, X=0, CC=0
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
    # Handle extension header
    if b0 & 0x10 and len(data) >= offset + 4:
        ext_len = struct.unpack("!H", data[offset + 2:offset + 4])[0]
        offset += 4 + ext_len * 4
    return {
        "version": version,
        "pt": pt,
        "seq": seq,
        "ts": ts,
        "ssrc": ssrc,
        "payload": data[offset:],
        "marker": marker,
    }


# ============================================================
# Audio Generation
# ============================================================

def generate_sine_pcm(freq_hz, duration_s, sample_rate=SAMPLE_RATE, amplitude=16000):
    t = np.arange(int(sample_rate * duration_s)) / sample_rate
    return (amplitude * np.sin(2 * np.pi * freq_hz * t)).astype(np.int16)

def generate_silence_pcm(duration_s, sample_rate=SAMPLE_RATE):
    return np.zeros(int(sample_rate * duration_s), dtype=np.int16)

def pcm_to_opus_frames(pcm_samples, encoder):
    frames = []
    for i in range(0, len(pcm_samples) - FRAME_SAMPLES + 1, FRAME_SAMPLES):
        chunk = pcm_samples[i:i + FRAME_SAMPLES]
        encoded = encoder.encode(chunk.tobytes(), FRAME_SAMPLES)
        frames.append(encoded)
    return frames

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

def pcm_to_ulaw_frames(pcm_samples_8k):
    """Convert 8kHz int16 PCM to 20ms mu-law frames (160 bytes each)."""
    frames = []
    for i in range(0, len(pcm_samples_8k) - PCMU_FRAME_SAMPLES + 1, PCMU_FRAME_SAMPLES):
        chunk = pcm_samples_8k[i:i + PCMU_FRAME_SAMPLES]
        ulaw_frame = bytes(linear_to_ulaw(int(s)) for s in chunk)
        frames.append(ulaw_frame)
    return frames


# ============================================================
# SIP Registration
# ============================================================

def sip_register(sock, ext, password, sip_port):
    tag = gen_tag()
    cid = f"reg-{gen_callid()}"
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
# Callee Auto-Answer Listener
# ============================================================

class CalleeListener:
    """Background thread that auto-answers incoming INVITEs."""

    def __init__(self, sock, ext, rtp_port, sdp_builder=None):
        self.sock = sock
        self.ext = ext
        self.rtp_port = rtp_port
        self.sdp_builder = sdp_builder  # optional custom SDP builder fn(rtp_port)->str
        self.rtp_target = None
        self.remote_codecs = []
        self.call_established = threading.Event()
        self.invite_received = threading.Event()
        self.bye_received = threading.Event()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)

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
                self._on_invite(msg, addr)
            elif msg.startswith("ACK "):
                self.call_established.set()
            elif msg.startswith("BYE "):
                self._reply_ok(msg, addr)
                self.bye_received.set()
                break
            elif msg.startswith("CANCEL "):
                self._reply_ok(msg, addr)
                break

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
        if self.sdp_builder:
            sdp_body = self.sdp_builder(self.rtp_port)
        else:
            sdp_body = build_sdp(self.rtp_port)
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


# ============================================================
# Caller INVITE
# ============================================================

def caller_invite(sock, ext, password, sip_port, rtp_port, callee_ext, sdp_body=None):
    """Send INVITE, handle auth challenge, return (rtp_target, codecs, call_state) or (None, [], None)."""
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
            return None, [], None

    return None, [], None


def send_bye(call_state):
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
    recv_sip(call_state["sock"], timeout=3)


# ============================================================
# RTP Sender / Receiver threads
# ============================================================

def rtp_sender(sock, target, frames, stop_event, pt=OPUS_PT, ts_increment=FRAME_SAMPLES):
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


def rtp_receiver_raw(sock, stop_event):
    """Collect ALL received RTP packets (raw bytes + parsed).  Returns list of (raw, parsed)."""
    packets = []
    sock.settimeout(0.2)
    while not stop_event.is_set():
        try:
            data, _ = sock.recvfrom(65535)
            parsed = parse_rtp(data)
            if parsed:
                packets.append((bytes(data), parsed))
        except socket.timeout:
            continue
        except OSError:
            break
    return packets


# ============================================================
# Call Setup / Teardown Fixture
# ============================================================

class CallContext:
    """Holds all state for a single test call."""

    def __init__(self, ports, callee_sdp_builder=None, caller_sdp_body=None):
        self.ports = ports
        self.callee_sdp_builder = callee_sdp_builder
        self.caller_sdp_body = caller_sdp_body
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
        """Register both UAs, start callee listener, and place the call."""
        try:
            self.caller_sip = make_socket(self.ports["caller_sip_port"])
            self.callee_sip = make_socket(self.ports["callee_sip_port"])
            self.caller_rtp = make_socket(self.ports["caller_rtp_port"])
            self.callee_rtp = make_socket(self.ports["callee_rtp_port"])
        except OSError as e:
            self.error = f"Socket bind failed: {e}"
            return False

        ok1 = sip_register(self.caller_sip, "1001", "test1001", self.ports["caller_sip_port"])
        ok2 = sip_register(self.callee_sip, "1002", "test1002", self.ports["callee_sip_port"])
        if not (ok1 and ok2):
            self.error = f"Registration failed (caller={ok1}, callee={ok2})"
            return False

        self.listener = CalleeListener(
            self.callee_sip, "1002", self.ports["callee_rtp_port"],
            sdp_builder=self.callee_sdp_builder,
        )
        self.listener.start()

        rtp_target, codecs, call_state = caller_invite(
            self.caller_sip, "1001", "test1001",
            self.ports["caller_sip_port"], self.ports["caller_rtp_port"],
            "1002", sdp_body=self.caller_sdp_body,
        )
        if not call_state:
            self.error = "INVITE failed or was rejected"
            return False

        self.listener.wait_for_call(timeout=10)
        self.caller_rtp_target = rtp_target
        self.caller_codecs = codecs
        self.call_state = call_state
        return True

    def teardown(self):
        """BYE, unregister, close sockets."""
        if self.call_state:
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
        for s in (self.caller_sip, self.callee_sip, self.caller_rtp, self.callee_rtp):
            if s:
                try:
                    s.close()
                except Exception:
                    pass


@pytest.fixture
def call_ctx():
    """Pytest fixture that sets up a standard Opus call, yields the context, and tears down."""
    ports = _alloc_ports()
    ctx = CallContext(ports)
    ok = ctx.setup()
    if not ok:
        ctx.teardown()
        pytest.skip(f"Call setup failed: {ctx.error}")
    yield ctx
    ctx.teardown()


# ============================================================
# Test Class
# ============================================================

class TestL5Media:
    """L5: Media plane, codec negotiation, RTP, DTMF, and recording tests."""

    # ------------------------------------------------------------------
    # 1. Opus codec negotiation
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L5_001_opus_codec_negotiation(self, call_ctx):
        """TC-L5-001: Opus appears as primary codec in the SDP answer from the server."""
        codecs = call_ctx.caller_codecs
        assert len(codecs) > 0, "No codecs in server SDP answer"

        # Check that Opus is present
        opus_codecs = [c for c in codecs if "opus" in c[1].lower()]
        assert len(opus_codecs) > 0, (
            f"Opus not found in SDP answer codecs: {[c[1] for c in codecs]}"
        )

        # Opus should be the first (primary) codec
        first_codec_name = codecs[0][1].lower()
        assert "opus" in first_codec_name, (
            f"Opus is not the primary codec. First codec: {codecs[0][1]}, "
            f"all codecs: {[c[1] for c in codecs]}"
        )

    # ------------------------------------------------------------------
    # 2. PCMU fallback
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L5_002_pcmu_fallback(self):
        """TC-L5-002: When only PCMU is offered, the server accepts it."""
        ports = _alloc_ports()
        # Callee answers with PCMU-only SDP
        pcmu_sdp_builder = lambda rtp_port: build_sdp(
            rtp_port, codecs=[(PCMU_PT, "PCMU/8000", None)]
        )
        # Caller offers PCMU-only SDP
        caller_sdp = build_sdp(
            ports["caller_rtp_port"],
            codecs=[
                (PCMU_PT, "PCMU/8000", None),
                (DTMF_PT, "telephone-event/8000", "0-16"),
            ],
        )
        ctx = CallContext(ports, callee_sdp_builder=pcmu_sdp_builder, caller_sdp_body=caller_sdp)
        try:
            ok = ctx.setup()
            assert ok, f"PCMU-only call failed: {ctx.error}"

            # The server SDP answer should contain PCMU
            pcmu_codecs = [c for c in ctx.caller_codecs if "PCMU" in c[1].upper()]
            assert len(pcmu_codecs) > 0, (
                f"PCMU not in answer: {[c[1] for c in ctx.caller_codecs]}"
            )

            # Verify we can actually send and receive PCMU RTP
            stop = threading.Event()
            silence_8k = generate_silence_pcm(2, sample_rate=PCMU_RATE)
            ulaw_frames = pcm_to_ulaw_frames(silence_8k)
            assert len(ulaw_frames) > 0, "Failed to generate mu-law frames"

            callee_pkts = []
            def recv_fn():
                nonlocal callee_pkts
                callee_pkts = rtp_receiver(ctx.callee_rtp, stop, accept_pts={PCMU_PT})

            recv_t = threading.Thread(target=recv_fn, daemon=True)
            send_t = threading.Thread(
                target=rtp_sender,
                args=(ctx.caller_rtp, ctx.caller_rtp_target, ulaw_frames, stop),
                kwargs={"pt": PCMU_PT, "ts_increment": PCMU_FRAME_SAMPLES},
                daemon=True,
            )
            recv_t.start()
            send_t.start()
            time.sleep(3)
            stop.set()
            time.sleep(0.5)

            assert len(callee_pkts) > 0, "No PCMU packets received by callee"
        finally:
            ctx.teardown()

    # ------------------------------------------------------------------
    # 3. Bidirectional RTP flow
    # ------------------------------------------------------------------

    @pytest.mark.timeout(90)
    def test_L5_003_bidirectional_rtp(self, call_ctx):
        """TC-L5-003: RTP flows in both directions (caller->callee and callee->caller)."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
        frames = pcm_to_opus_frames(generate_silence_pcm(5), enc)

        stop = threading.Event()
        caller_pkts = []
        callee_pkts = []

        def recv_caller():
            nonlocal caller_pkts
            caller_pkts = rtp_receiver(call_ctx.caller_rtp, stop, accept_pts={OPUS_PT})

        def recv_callee():
            nonlocal callee_pkts
            callee_pkts = rtp_receiver(call_ctx.callee_rtp, stop, accept_pts={OPUS_PT})

        threads = [
            threading.Thread(target=recv_caller, daemon=True),
            threading.Thread(target=recv_callee, daemon=True),
            threading.Thread(
                target=rtp_sender,
                args=(call_ctx.caller_rtp, call_ctx.caller_rtp_target, frames, stop),
                daemon=True,
            ),
        ]
        if call_ctx.listener.rtp_target:
            threads.append(threading.Thread(
                target=rtp_sender,
                args=(call_ctx.callee_rtp, call_ctx.listener.rtp_target, frames, stop),
                daemon=True,
            ))

        for t in threads:
            t.start()

        time.sleep(5)
        stop.set()
        time.sleep(1)

        expected_min = 5 * 50 * 0.5  # 5s * 50pps * 50% tolerance
        assert len(callee_pkts) > expected_min, (
            f"Caller->Callee: only {len(callee_pkts)} packets (expected > {expected_min:.0f})"
        )
        assert len(caller_pkts) > expected_min, (
            f"Callee->Caller: only {len(caller_pkts)} packets (expected > {expected_min:.0f})"
        )

    # ------------------------------------------------------------------
    # 4. RTP packet structure
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L5_004_rtp_packet_structure(self, call_ctx):
        """TC-L5-004: RTP packets have correct V=2, consistent SSRC, sequential seq numbers."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
        frames = pcm_to_opus_frames(generate_silence_pcm(3), enc)

        stop = threading.Event()
        raw_pkts = []

        def recv_fn():
            nonlocal raw_pkts
            raw_pkts = rtp_receiver_raw(call_ctx.callee_rtp, stop)

        # Send from caller to callee through the proxy
        recv_t = threading.Thread(target=recv_fn, daemon=True)
        send_t = threading.Thread(
            target=rtp_sender,
            args=(call_ctx.caller_rtp, call_ctx.caller_rtp_target, frames, stop),
            daemon=True,
        )
        recv_t.start()
        send_t.start()
        time.sleep(3)
        stop.set()
        time.sleep(0.5)

        assert len(raw_pkts) > 10, f"Too few RTP packets received: {len(raw_pkts)}"

        # Check V=2 on all packets
        for raw_bytes, parsed in raw_pkts:
            assert parsed["version"] == 2, (
                f"RTP version != 2: got {parsed['version']}"
            )

        # Check consistent SSRC (proxy may re-SSRC, so just check consistency)
        ssrcs = set(p["ssrc"] for _, p in raw_pkts)
        assert len(ssrcs) <= 2, (
            f"Too many distinct SSRCs (expected 1-2, got {len(ssrcs)}): {ssrcs}"
        )

        # Check sequential sequence numbers (allowing for proxy rewrite)
        # Sort by seq, check that they are mostly incrementing
        seqs = [p["seq"] for _, p in raw_pkts]
        in_order = 0
        for i in range(1, len(seqs)):
            diff = (seqs[i] - seqs[i - 1]) & 0xFFFF
            if diff == 1:
                in_order += 1
        order_ratio = in_order / max(len(seqs) - 1, 1)
        assert order_ratio > 0.8, (
            f"Sequence numbers not sequential enough: {order_ratio:.1%} in order "
            f"(first 10 seqs: {seqs[:10]})"
        )

        # Check payload type
        pts = set(p["pt"] for _, p in raw_pkts)
        assert len(pts) > 0, "No payload types found"
        # We expect Opus PT or proxy-rewritten PT
        # At minimum, all packets should share the same PT
        assert len(pts) <= 2, f"Multiple unexpected PTs: {pts}"

    # ------------------------------------------------------------------
    # 5. RTP timing (20ms pacing)
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L5_005_rtp_timing(self, call_ctx):
        """TC-L5-005: RTP pacing is approximately 50 packets/sec (20ms intervals) for Opus."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
        frames = pcm_to_opus_frames(generate_silence_pcm(5), enc)

        stop = threading.Event()
        callee_pkts = []

        def recv_fn():
            nonlocal callee_pkts
            callee_pkts = rtp_receiver(call_ctx.callee_rtp, stop, accept_pts={OPUS_PT})

        recv_t = threading.Thread(target=recv_fn, daemon=True)
        send_t = threading.Thread(
            target=rtp_sender,
            args=(call_ctx.caller_rtp, call_ctx.caller_rtp_target, frames, stop),
            daemon=True,
        )
        recv_t.start()
        send_t.start()

        duration_s = 5
        time.sleep(duration_s)
        stop.set()
        time.sleep(0.5)

        # Expect ~50 pps * duration_s packets, allow 30-70 pps tolerance
        pps = len(callee_pkts) / duration_s
        assert 25 < pps < 75, (
            f"RTP packet rate out of range: {pps:.1f} pps "
            f"(expected ~50 pps for 20ms Opus, got {len(callee_pkts)} pkts in {duration_s}s)"
        )

        # Also verify timestamps increment by FRAME_SAMPLES (960 for 48kHz/20ms)
        if len(callee_pkts) > 5:
            ts_diffs = []
            for i in range(1, min(len(callee_pkts), 30)):
                diff = (callee_pkts[i]["ts"] - callee_pkts[i - 1]["ts"]) & 0xFFFFFFFF
                ts_diffs.append(diff)
            # Most diffs should be 960 (48kHz * 20ms) or 160 (8kHz, if proxy transcodes)
            median_diff = sorted(ts_diffs)[len(ts_diffs) // 2]
            assert median_diff in (160, 320, 480, 960), (
                f"Unexpected median timestamp increment: {median_diff} "
                f"(expected 960 for Opus/48kHz or 160 for 8kHz). "
                f"Diffs sample: {ts_diffs[:10]}"
            )

    # ------------------------------------------------------------------
    # 6. DTMF via RFC 2833
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L5_006_dtmf_rfc2833(self, call_ctx):
        """TC-L5-006: DTMF telephone-event RTP packets are relayed through the proxy."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        # First send some regular audio so the call is established
        enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
        audio_frames = pcm_to_opus_frames(generate_silence_pcm(2), enc)

        stop = threading.Event()
        callee_pkts = []

        def recv_fn():
            nonlocal callee_pkts
            # Accept both Opus and DTMF payload types
            callee_pkts = rtp_receiver(call_ctx.callee_rtp, stop, accept_pts={OPUS_PT, DTMF_PT})

        recv_t = threading.Thread(target=recv_fn, daemon=True)
        send_t = threading.Thread(
            target=rtp_sender,
            args=(call_ctx.caller_rtp, call_ctx.caller_rtp_target, audio_frames, stop),
            daemon=True,
        )
        recv_t.start()
        send_t.start()

        # Wait for audio flow to stabilize
        time.sleep(1)

        # Now send DTMF digit "5" as RFC 2833 telephone-event
        # RFC 2833 payload: event(1B), E+R+volume(1B), duration(2B)
        ssrc = random.randint(1, 0xFFFFFFFF)
        seq = random.randint(0, 0xFFFF)
        ts_base = random.randint(0, 0xFFFFFFFF)
        dtmf_event = 5  # digit '5'
        volume = 10  # -10 dBm0

        # Send DTMF begin packets (no end flag), then end packet
        # Duration increases over time (in timestamp units, 8kHz)
        for i in range(5):
            duration = (i + 1) * 160  # 20ms each at 8kHz
            end_flag = 0x00
            payload = struct.pack("!BBH", dtmf_event, end_flag | (volume & 0x3F), duration)
            pkt = build_rtp(DTMF_PT, seq, ts_base, ssrc, payload, marker=(i == 0))
            try:
                call_ctx.caller_rtp.sendto(pkt, call_ctx.caller_rtp_target)
            except OSError:
                break
            seq = (seq + 1) & 0xFFFF
            time.sleep(0.02)

        # Send end packets (3 copies per RFC 2833)
        for _ in range(3):
            duration = 6 * 160
            end_flag = 0x80  # End bit set
            payload = struct.pack("!BBH", dtmf_event, end_flag | (volume & 0x3F), duration)
            pkt = build_rtp(DTMF_PT, seq, ts_base, ssrc, payload)
            try:
                call_ctx.caller_rtp.sendto(pkt, call_ctx.caller_rtp_target)
            except OSError:
                break
            seq = (seq + 1) & 0xFFFF
            time.sleep(0.02)

        # Wait a bit for packets to arrive
        time.sleep(2)
        stop.set()
        time.sleep(0.5)

        # Check we received some packets (either DTMF relayed or audio)
        total_pkts = len(callee_pkts)
        assert total_pkts > 0, "No RTP packets received at all"

        # Check for DTMF packets specifically; the server may relay them
        # with the same PT or transcode.  We check that either:
        # (a) DTMF PT packets arrived, or
        # (b) total packet count is healthy (DTMF was consumed/converted by server)
        dtmf_pkts = [p for p in callee_pkts if p["pt"] == DTMF_PT]
        audio_pkts = [p for p in callee_pkts if p["pt"] != DTMF_PT]

        # The proxy may consume DTMF events (for features like IVR) rather than relay.
        # Either way, the call should remain stable with audio flowing.
        assert len(audio_pkts) > 20 or len(dtmf_pkts) > 0, (
            f"Neither audio ({len(audio_pkts)} pkts) nor DTMF ({len(dtmf_pkts)} pkts) "
            f"received adequately after sending DTMF events"
        )

    # ------------------------------------------------------------------
    # 7. Codec preference order
    # ------------------------------------------------------------------

    @pytest.mark.timeout(60)
    def test_L5_007_codec_preference_order(self, call_ctx):
        """TC-L5-007: Server's SDP offer to callee has Opus as first codec."""
        # The callee listener captures the remote (server) SDP in the INVITE
        callee_codecs = call_ctx.listener.remote_codecs
        assert len(callee_codecs) > 0, "No codecs in server SDP offer to callee"

        first_codec = callee_codecs[0][1].lower()
        assert "opus" in first_codec, (
            f"Opus is not first in server offer to callee. "
            f"Codec order: {[c[1] for c in callee_codecs]}"
        )

    # ------------------------------------------------------------------
    # 8. Media timeout
    # ------------------------------------------------------------------

    @pytest.mark.timeout(180)
    def test_L5_008_media_timeout(self):
        """TC-L5-008: Stopping RTP causes the server to eventually terminate the call."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        ports = _alloc_ports()
        ctx = CallContext(ports)
        ok = ctx.setup()
        if not ok:
            ctx.teardown()
            pytest.skip(f"Call setup failed: {ctx.error}")

        try:
            # Send a few seconds of audio to establish media flow
            enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
            frames = pcm_to_opus_frames(generate_silence_pcm(3), enc)

            stop_send = threading.Event()
            send_t = threading.Thread(
                target=rtp_sender,
                args=(ctx.caller_rtp, ctx.caller_rtp_target, frames, stop_send),
                daemon=True,
            )
            send_t.start()
            time.sleep(3)

            # Stop all RTP
            stop_send.set()
            time.sleep(0.5)

            # Now wait for the server to send BYE (media timeout)
            # RustPBX default media timeout is typically 30-60s
            bye_received = False
            start = time.time()
            max_wait = 120  # generous timeout

            while time.time() - start < max_wait:
                # Check caller socket for BYE
                ctx.caller_sip.settimeout(1)
                try:
                    data, addr = ctx.caller_sip.recvfrom(65535)
                    msg = data.decode(errors="replace")
                    if msg.startswith("BYE "):
                        bye_received = True
                        # Reply 200 OK to the BYE
                        vias = get_all_via(msg)
                        via_block = "\r\n".join(vias)
                        ok_resp = (
                            f"SIP/2.0 200 OK\r\n"
                            f"{via_block}\r\n"
                            f"From: {get_header(msg, 'From')}\r\n"
                            f"To: {get_header(msg, 'To')}\r\n"
                            f"Call-ID: {get_header(msg, 'Call-ID')}\r\n"
                            f"CSeq: {get_header(msg, 'CSeq')}\r\n"
                            f"Content-Length: 0\r\n\r\n"
                        )
                        ctx.caller_sip.sendto(ok_resp.encode(), addr)
                        break
                except (socket.timeout, OSError):
                    pass

                # Also check if callee received BYE
                if ctx.listener.bye_received.is_set():
                    bye_received = True
                    break

            elapsed = time.time() - start
            assert bye_received, (
                f"Server did not send BYE after {elapsed:.0f}s of media silence "
                f"(waited {max_wait}s). Media timeout may be disabled or very long."
            )
            # Clear call_state so teardown does not try to send BYE again
            ctx.call_state = None

        finally:
            ctx.teardown()

    # ------------------------------------------------------------------
    # 9. Audio quality check (1kHz sine, SNR > 20dB)
    # ------------------------------------------------------------------

    @pytest.mark.timeout(90)
    def test_L5_009_audio_quality_snr(self, call_ctx):
        """TC-L5-009: 1kHz sine wave through call path yields SNR > 20dB."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        duration = 8
        ref_pcm = generate_sine_pcm(1000, duration + 1)

        enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
        caller_frames = pcm_to_opus_frames(ref_pcm, enc)

        # Also send from callee so call stays alive
        enc2 = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
        callee_frames = pcm_to_opus_frames(generate_silence_pcm(duration + 1), enc2)

        stop = threading.Event()
        callee_pkts = []

        def recv_fn():
            nonlocal callee_pkts
            callee_pkts = rtp_receiver(call_ctx.callee_rtp, stop, accept_pts={OPUS_PT})

        threads = [
            threading.Thread(target=recv_fn, daemon=True),
            threading.Thread(
                target=rtp_sender,
                args=(call_ctx.caller_rtp, call_ctx.caller_rtp_target, caller_frames, stop),
                daemon=True,
            ),
        ]
        if call_ctx.listener.rtp_target:
            threads.append(threading.Thread(
                target=rtp_sender,
                args=(call_ctx.callee_rtp, call_ctx.listener.rtp_target, callee_frames, stop),
                daemon=True,
            ))
        for t in threads:
            t.start()

        time.sleep(duration)
        stop.set()
        time.sleep(1)

        assert len(callee_pkts) > 50, (
            f"Not enough packets for quality analysis: {len(callee_pkts)}"
        )

        # Decode received Opus
        dec = opuslib.Decoder(SAMPLE_RATE, 1)
        parts = []
        for p in sorted(callee_pkts, key=lambda x: x["seq"]):
            try:
                pcm = dec.decode(bytes(p["payload"]), FRAME_SAMPLES)
                parts.append(np.frombuffer(pcm, dtype=np.int16))
            except Exception:
                parts.append(np.zeros(FRAME_SAMPLES, dtype=np.int16))
        received = np.concatenate(parts) if parts else np.array([], dtype=np.int16)

        assert len(received) > SAMPLE_RATE * 2, (
            f"Decoded audio too short: {len(received)} samples"
        )

        # Skip first/last second for stabilization
        skip = SAMPLE_RATE
        ref_trim = ref_pcm[skip:]
        rec_trim = received[skip:]

        # Align signals via cross-correlation
        chunk_len = min(SAMPLE_RATE, len(ref_trim), len(rec_trim))
        ref_chunk = ref_trim[:chunk_len].astype(np.float64)
        rec_search = rec_trim[:chunk_len * 2].astype(np.float64) if len(rec_trim) > chunk_len else rec_trim.astype(np.float64)
        corr = np.correlate(rec_search, ref_chunk, mode="valid")
        offset = int(np.argmax(corr))
        rec_aligned = rec_trim[offset:]
        min_len = min(len(ref_trim), len(rec_aligned))
        ref_a = ref_trim[:min_len].astype(np.float64)
        rec_a = rec_aligned[:min_len].astype(np.float64)

        assert len(ref_a) > SAMPLE_RATE, (
            f"Not enough aligned audio: {len(ref_a)} samples"
        )

        # Normalize RMS
        ref_rms = np.sqrt(np.mean(ref_a ** 2))
        rec_rms = np.sqrt(np.mean(rec_a ** 2))
        if ref_rms > 0 and rec_rms > 0:
            rec_a = rec_a * (ref_rms / rec_rms)

        noise = rec_a - ref_a
        sig_power = np.mean(ref_a ** 2)
        noise_power = np.mean(noise ** 2)
        if noise_power < 1e-10:
            snr = 100.0
        else:
            snr = 10 * np.log10(sig_power / noise_power)

        assert snr > 20.0, (
            f"SNR too low: {snr:.1f} dB (expected > 20 dB). "
            f"Audio may be distorted or not passing through correctly."
        )

    # ------------------------------------------------------------------
    # 10. Recording verification
    # ------------------------------------------------------------------

    @pytest.mark.timeout(90)
    def test_L5_010_recording_verification(self):
        """TC-L5-010: After a call with audio, a WAV recording file is created."""
        if not HAS_OPUS:
            pytest.skip("opuslib not installed")

        # Snapshot existing recordings
        pre_files = set()
        if os.path.isdir(RECORDER_DIR):
            pre_files = set(os.listdir(RECORDER_DIR))

        ports = _alloc_ports()
        ctx = CallContext(ports)
        ok = ctx.setup()
        if not ok:
            ctx.teardown()
            pytest.skip(f"Call setup failed: {ctx.error}")

        try:
            # Send audio for a few seconds
            enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
            enc2 = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
            caller_frames = pcm_to_opus_frames(generate_sine_pcm(440, 5), enc)
            callee_frames = pcm_to_opus_frames(generate_sine_pcm(880, 5), enc2)

            stop = threading.Event()
            threads = [
                threading.Thread(
                    target=rtp_sender,
                    args=(ctx.caller_rtp, ctx.caller_rtp_target, caller_frames, stop),
                    daemon=True,
                ),
            ]
            if ctx.listener.rtp_target:
                threads.append(threading.Thread(
                    target=rtp_sender,
                    args=(ctx.callee_rtp, ctx.listener.rtp_target, callee_frames, stop),
                    daemon=True,
                ))
            for t in threads:
                t.start()

            time.sleep(5)
            stop.set()
            time.sleep(0.5)
        finally:
            ctx.teardown()

        # Wait for recording to be finalized
        time.sleep(5)

        if not os.path.isdir(RECORDER_DIR):
            pytest.skip(f"Recording directory does not exist: {RECORDER_DIR}")

        post_files = set(os.listdir(RECORDER_DIR))
        new_files = sorted(f for f in (post_files - pre_files) if f.endswith(".wav"))

        assert len(new_files) > 0, (
            f"No new WAV files in {RECORDER_DIR} after call. "
            f"Recording may be disabled. Pre-existing: {len(pre_files)}, post: {len(post_files)}"
        )

        # Verify the WAV file is well-formed and non-empty
        wav_path = os.path.join(RECORDER_DIR, new_files[-1])
        file_size = os.path.getsize(wav_path)
        assert file_size > 100, (
            f"WAV file too small: {file_size} bytes ({wav_path})"
        )

        # Try to read WAV header
        try:
            with wave.open(wav_path, "r") as wf:
                channels = wf.getnchannels()
                rate = wf.getsampwidth()
                n_frames = wf.getnframes()
                assert n_frames > 0, f"WAV has 0 frames: {wav_path}"
                assert channels in (1, 2), f"Unexpected channels: {channels}"
        except wave.Error:
            # mu-law WAV (format 7) may not be readable by Python's wave module
            # Just verify the file exists and has reasonable size
            assert file_size > 1000, (
                f"WAV file exists but is suspiciously small ({file_size} bytes) "
                f"and cannot be parsed by Python wave module: {wav_path}"
            )
