#!/usr/bin/env python3
"""
RustPBX Automated Call Quality Test Suite

Runs three test modes against a live RustPBX instance:
  smoke         - Opus negotiation, RTP bidirectional flow, 60s call stability
  fidelity      - Audio quality: SNR, PESQ, ViSQOL via 1kHz sine wave
  transcription - English/Spanish speech through call, verify transcription
  all           - Run all tests sequentially

Requires: opuslib, numpy, scipy, pesq, pyvisqol, gtts (in /root/test-env)
Server:   RustPBX running on localhost:5060 with users 1001/1002

Usage:
  /root/test-env/bin/python tests/call_quality_test.py [smoke|fidelity|transcription|all]
"""

import argparse
import hashlib
import json
import os
import random
import re
import socket
import string
import struct
import subprocess
import sys
import threading
import time
import wave

import numpy as np

try:
    import opuslib
    HAS_OPUS = True
except ImportError:
    HAS_OPUS = False

try:
    from pesq import pesq as pesq_fn
    HAS_PESQ = True
except ImportError:
    HAS_PESQ = False

try:
    from pyvisqol import Visqol
    HAS_VISQOL = True
except ImportError:
    HAS_VISQOL = False

# ============================================================
# Configuration
# ============================================================
SERVER = "127.0.0.1"
SERVER_PORT = 5060
EXTERNAL_IP = "74.207.251.126"
API_BASE = "https://127.0.0.1:8443"

CALLER = {"ext": "1001", "password": "test1001", "sip_port": 15060, "rtp_port": 30010}
CALLEE = {"ext": "1002", "password": "test1002", "sip_port": 15062, "rtp_port": 30012}

OPUS_PT = 111
PCMU_PT = 0
DTMF_PT = 101
SAMPLE_RATE = 48000
FRAME_MS = 20
FRAME_SAMPLES = SAMPLE_RATE * FRAME_MS // 1000  # 960

SPEECH_DIR = "/tmp/rustpbx_test_speech"
RECORDER_DIR = os.path.expanduser("~/rustpbx/config/recorders")

# ============================================================
# SIP Helpers
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

def build_sdp(rtp_port, local_ip="127.0.0.1"):
    sid = str(random.randint(100000, 999999))
    return (
        "v=0\r\n"
        f"o=- {sid} {sid} IN IP4 {local_ip}\r\n"
        "s=RustPBX Test\r\n"
        f"c=IN IP4 {local_ip}\r\n"
        "t=0 0\r\n"
        f"m=audio {rtp_port} RTP/AVP {OPUS_PT} {PCMU_PT} {DTMF_PT}\r\n"
        f"a=rtpmap:{OPUS_PT} opus/48000/2\r\n"
        f"a=fmtp:{OPUS_PT} minptime=10;useinbandfec=1;stereo=1;sprop-stereo=1;maxaveragebitrate=128000\r\n"
        f"a=rtpmap:{PCMU_PT} PCMU/8000\r\n"
        f"a=rtpmap:{DTMF_PT} telephone-event/8000\r\n"
        f"a=fmtp:{DTMF_PT} 0-16\r\n"
        "a=sendrecv\r\n"
    )

def send_sip(sock, msg, dest=(SERVER, SERVER_PORT)):
    sock.sendto(msg.encode(), dest)

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
    pt = b1 & 0x7f
    marker = bool(b1 & 0x80)
    cc = b0 & 0x0f
    offset = 12 + cc * 4
    if b0 & 0x10 and len(data) >= offset + 4:
        ext_len = struct.unpack("!H", data[offset + 2:offset + 4])[0]
        offset += 4 + ext_len * 4
    return {"pt": pt, "seq": seq, "ts": ts, "ssrc": ssrc, "payload": data[offset:], "marker": marker}

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

def generate_speech_wav(text, lang, output_path):
    """Generate speech WAV via gtts+ffmpeg, fallback to espeak-ng."""
    try:
        from gtts import gTTS
        mp3_path = output_path + ".mp3"
        gTTS(text, lang=lang).save(mp3_path)
        subprocess.run(
            ["ffmpeg", "-y", "-i", mp3_path, "-ar", str(SAMPLE_RATE), "-ac", "1", output_path],
            capture_output=True, check=True,
        )
        os.unlink(mp3_path)
        return True
    except Exception:
        pass
    try:
        # espeak-ng outputs WAV (usually 22050 Hz) - convert with ffmpeg
        tmp = output_path + ".espeak.wav"
        subprocess.run(["espeak-ng", "-v", lang, text, "-w", tmp], capture_output=True, check=True)
        subprocess.run(
            ["ffmpeg", "-y", "-i", tmp, "-ar", str(SAMPLE_RATE), "-ac", "1", output_path],
            capture_output=True, check=True,
        )
        os.unlink(tmp)
        return True
    except Exception:
        return False

def load_wav_48k(path):
    """Load a WAV file as 48kHz mono int16 PCM."""
    try:
        import librosa
        audio, _ = librosa.load(path, sr=SAMPLE_RATE, mono=True)
        return (audio * 32767).astype(np.int16)
    except ImportError:
        pass
    with wave.open(path, "r") as wf:
        raw = wf.readframes(wf.getnframes())
        samples = np.frombuffer(raw, dtype=np.int16)
        if wf.getnchannels() == 2:
            samples = samples[::2]
        return samples

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
    auth_line = f'Digest username="{ext}", realm="{realm}", nonce="{nonce}", uri="{uri}", response="{digest}", algorithm=MD5'

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
# Callee Auto-Answer
# ============================================================

class CalleeListener:
    def __init__(self, sock, ext, rtp_port):
        self.sock = sock
        self.ext = ext
        self.rtp_port = rtp_port
        self.rtp_target = None
        self.remote_codecs = []
        self.call_established = threading.Event()
        self.invite_received = threading.Event()
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

        # Parse SDP
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

def caller_invite(sock, caller, callee_ext):
    """Send INVITE, handle auth challenge, return (rtp_target, codecs, call_state)."""
    ext = caller["ext"]
    pw = caller["password"]
    sip_port = caller["sip_port"]
    rtp_port = caller["rtp_port"]
    from_uri = f"sip:{ext}@{EXTERNAL_IP}"
    to_uri = f"sip:{callee_ext}@{SERVER}"
    local_tag = gen_tag()
    call_id = gen_callid()
    contact = f"<sip:{ext}@{SERVER}:{sip_port}>"
    cseq = 1
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

            auth_hdr = ""
            for line in resp.split("\r\n"):
                low = line.lower()
                if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                    auth_hdr = line.split(":", 1)[1].strip()
                    break
            realm, nonce = parse_www_authenticate(auth_hdr)
            realm = realm or EXTERNAL_IP
            digest = compute_digest(ext, realm, pw, "INVITE", to_uri, nonce)
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
# RTP Sender / Receiver
# ============================================================

def rtp_sender(sock, target, opus_frames, stop_event):
    ssrc = random.randint(1, 0xFFFFFFFF)
    seq = random.randint(0, 0xFFFF)
    ts = random.randint(0, 0xFFFFFFFF)
    idx = 0
    interval = FRAME_MS / 1000.0
    next_send = time.time()

    while not stop_event.is_set():
        if idx >= len(opus_frames):
            idx = 0
        pkt = build_rtp(OPUS_PT, seq, ts, ssrc, opus_frames[idx], marker=(idx == 0))
        try:
            sock.sendto(pkt, target)
        except OSError:
            break
        seq = (seq + 1) & 0xFFFF
        ts = (ts + FRAME_SAMPLES) & 0xFFFFFFFF
        idx += 1
        next_send += interval
        delay = next_send - time.time()
        if delay > 0:
            time.sleep(delay)
        elif delay < -0.5:
            next_send = time.time()

def rtp_receiver(sock, stop_event):
    """Collect received Opus RTP packets. Returns list of (seq, ts, payload)."""
    packets = []
    sock.settimeout(0.2)
    while not stop_event.is_set():
        try:
            data, _ = sock.recvfrom(65535)
            parsed = parse_rtp(data)
            if parsed and parsed["pt"] == OPUS_PT:
                packets.append((parsed["seq"], parsed["ts"], bytes(parsed["payload"])))
        except socket.timeout:
            continue
        except OSError:
            break
    return packets

# ============================================================
# Quality Analysis
# ============================================================

def decode_opus_packets(packets):
    if not HAS_OPUS:
        return None
    dec = opuslib.Decoder(SAMPLE_RATE, 1)
    parts = []
    for _, _, payload in sorted(packets, key=lambda p: p[0]):
        try:
            pcm = dec.decode(payload, FRAME_SAMPLES)
            parts.append(np.frombuffer(pcm, dtype=np.int16))
        except Exception:
            parts.append(np.zeros(FRAME_SAMPLES, dtype=np.int16))
    return np.concatenate(parts) if parts else np.array([], dtype=np.int16)

def compute_snr(reference, received):
    min_len = min(len(reference), len(received))
    ref = reference[:min_len].astype(np.float64)
    rec = received[:min_len].astype(np.float64)
    # Normalize RMS amplitudes before comparison (Opus codec changes gain)
    ref_rms = np.sqrt(np.mean(ref ** 2))
    rec_rms = np.sqrt(np.mean(rec ** 2))
    if ref_rms > 0 and rec_rms > 0:
        rec = rec * (ref_rms / rec_rms)
    noise = rec - ref
    sig_power = np.mean(ref ** 2)
    noise_power = np.mean(noise ** 2)
    if noise_power < 1e-10:
        return 100.0
    return 10 * np.log10(sig_power / max(noise_power, 1e-10))

def align_signals(reference, received):
    """Cross-correlate to find best alignment offset, then trim."""
    chunk_len = min(SAMPLE_RATE, len(reference), len(received))
    ref_chunk = reference[:chunk_len].astype(np.float64)
    rec_search = received[:chunk_len * 2].astype(np.float64) if len(received) > chunk_len else received.astype(np.float64)
    corr = np.correlate(rec_search, ref_chunk, mode="valid")
    offset = int(np.argmax(np.abs(corr)))
    rec_aligned = received[offset:]
    min_len = min(len(reference), len(rec_aligned))
    return reference[:min_len], rec_aligned[:min_len]

def run_pesq(reference, received):
    if not HAS_PESQ:
        return None
    try:
        from scipy.signal import resample as scipy_resample
        # PESQ needs 16kHz for wideband
        r16 = scipy_resample(reference.astype(np.float64), int(len(reference) * 16000 / SAMPLE_RATE))
        d16 = scipy_resample(received.astype(np.float64), int(len(received) * 16000 / SAMPLE_RATE))
        min_len = min(len(r16), len(d16))
        r16 = r16[:min_len].astype(np.float32)
        d16 = d16[:min_len].astype(np.float32)
        mx = max(np.max(np.abs(r16)), np.max(np.abs(d16)), 1.0)
        return pesq_fn(16000, r16 / mx, d16 / mx, "wb")
    except Exception as e:
        print(f"  [WARN] PESQ error: {e}")
        return None

def run_visqol(reference, received):
    if not HAS_VISQOL:
        return None
    try:
        import soundfile as sf
        from scipy.signal import resample as scipy_resample
        # ViSQOL needs 16kHz
        r16 = scipy_resample(reference.astype(np.float64), int(len(reference) * 16000 / SAMPLE_RATE)).astype(np.float32)
        d16 = scipy_resample(received.astype(np.float64), int(len(received) * 16000 / SAMPLE_RATE)).astype(np.float32)
        min_len = min(len(r16), len(d16))
        r16 = r16[:min_len] / max(np.max(np.abs(r16[:min_len])), 1.0)
        d16 = d16[:min_len] / max(np.max(np.abs(d16[:min_len])), 1.0)
        ref_path, deg_path = "/tmp/vq_ref.wav", "/tmp/vq_deg.wav"
        sf.write(ref_path, r16, 16000)
        sf.write(deg_path, d16, 16000)
        v = Visqol()
        score = v.measure(ref_path, deg_path)
        os.unlink(ref_path)
        os.unlink(deg_path)
        return score
    except Exception as e:
        print(f"  [WARN] ViSQOL error: {e}")
        return None

# ============================================================
# Helpers shared by all tests
# ============================================================

class TestResult:
    def __init__(self):
        self.passed = True
        self.checks = []

    def check(self, name, ok, detail=""):
        tag = "PASS" if ok else "FAIL"
        self.checks.append((name, ok, detail))
        if not ok:
            self.passed = False
        msg = f"  [{tag}] {name}"
        if detail:
            msg += f"  ({detail})"
        print(msg)

def setup_call():
    """Register both UAs, start callee listener, send INVITE. Returns context dict or None."""
    ctx = {}
    ctx["caller_sip"] = make_socket(CALLER["sip_port"])
    ctx["callee_sip"] = make_socket(CALLEE["sip_port"])
    ctx["caller_rtp"] = make_socket(CALLER["rtp_port"])
    ctx["callee_rtp"] = make_socket(CALLEE["rtp_port"])

    print("  Registering 1001...")
    ok1 = sip_register(ctx["caller_sip"], CALLER["ext"], CALLER["password"], CALLER["sip_port"])
    print("  Registering 1002...")
    ok2 = sip_register(ctx["callee_sip"], CALLEE["ext"], CALLEE["password"], CALLEE["sip_port"])
    if not (ok1 and ok2):
        return None, "Registration failed"

    listener = CalleeListener(ctx["callee_sip"], CALLEE["ext"], CALLEE["rtp_port"])
    listener.start()
    ctx["listener"] = listener

    print(f"  Calling {CALLEE['ext']}...")
    rtp_target, codecs, call_state = caller_invite(ctx["caller_sip"], CALLER, CALLEE["ext"])
    if not call_state:
        return None, "INVITE failed"

    listener.wait_for_call(timeout=10)
    ctx["caller_rtp_target"] = rtp_target
    ctx["caller_codecs"] = codecs
    ctx["call_state"] = call_state
    return ctx, None

def teardown_call(ctx):
    if not ctx:
        return
    if "call_state" in ctx:
        try:
            send_bye(ctx["call_state"])
        except Exception:
            pass
    if "listener" in ctx:
        ctx["listener"].stop()
    for key in ("caller_sip", "callee_sip"):
        if key in ctx:
            try:
                sip_unregister(ctx[key], CALLER["ext"] if "caller" in key else CALLEE["ext"],
                               CALLER["sip_port"] if "caller" in key else CALLEE["sip_port"])
            except Exception:
                pass
    time.sleep(0.5)
    for key in ("caller_sip", "callee_sip", "caller_rtp", "callee_rtp"):
        if key in ctx:
            try:
                ctx[key].close()
            except Exception:
                pass

# ============================================================
# Test: Smoke
# ============================================================

def test_smoke(duration=60):
    print("\n" + "=" * 60)
    print("  SMOKE TEST")
    print("=" * 60)
    r = TestResult()

    ctx, err = setup_call()
    if err:
        r.check("Call setup", False, err)
        teardown_call(ctx)
        return r

    r.check("Registration", True)
    r.check("Call established", True)

    # Check Opus negotiation
    caller_has_opus = any("opus" in c[1].lower() for c in ctx["caller_codecs"])
    callee_has_opus = any("opus" in c[1].lower() for c in ctx["listener"].remote_codecs)
    r.check("Opus in caller answer SDP", caller_has_opus,
            f"codecs: {[c[1] for c in ctx['caller_codecs']]}")
    r.check("Opus in callee offer SDP", callee_has_opus,
            f"codecs: {[c[1] for c in ctx['listener'].remote_codecs]}")

    if not HAS_OPUS:
        r.check("Opus encoder", False, "opuslib not installed")
        teardown_call(ctx)
        return r

    # Encode silence
    enc = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
    frames = pcm_to_opus_frames(generate_silence_pcm(duration + 1), enc)

    stop = threading.Event()
    caller_pkts = []
    callee_pkts = []

    def recv_caller():
        nonlocal caller_pkts
        caller_pkts = rtp_receiver(ctx["caller_rtp"], stop)

    def recv_callee():
        nonlocal callee_pkts
        callee_pkts = rtp_receiver(ctx["callee_rtp"], stop)

    threads = [
        threading.Thread(target=recv_caller, daemon=True),
        threading.Thread(target=recv_callee, daemon=True),
        threading.Thread(target=rtp_sender, args=(ctx["caller_rtp"], ctx["caller_rtp_target"], frames, stop), daemon=True),
    ]
    if ctx["listener"].rtp_target:
        threads.append(threading.Thread(
            target=rtp_sender, args=(ctx["callee_rtp"], ctx["listener"].rtp_target, frames, stop), daemon=True
        ))

    for t in threads:
        t.start()

    print(f"  Holding call for {duration}s...")
    call_dropped = False
    for i in range(duration):
        time.sleep(1)
        if (i + 1) % 15 == 0:
            print(f"  ... {i+1}s")
        # Check for unexpected BYE
        ctx["caller_sip"].settimeout(0.01)
        try:
            data, _ = ctx["caller_sip"].recvfrom(65535)
            msg = data.decode(errors="replace")
            if msg.startswith("BYE "):
                r.check(f"Call stable for {duration}s", False, f"BYE at {i+1}s")
                call_dropped = True
                break
        except (socket.timeout, OSError):
            pass

    stop.set()
    time.sleep(1)

    expected = duration * (1000 // FRAME_MS)
    r.check(f"RTP 1001→1002", len(callee_pkts) > expected * 0.8,
            f"{len(callee_pkts)}/{expected} packets")
    r.check(f"RTP 1002→1001", len(caller_pkts) > expected * 0.8,
            f"{len(caller_pkts)}/{expected} packets")

    if callee_pkts:
        loss = max(0, (1 - len(callee_pkts) / expected) * 100)
        r.check("Packet loss < 2%", loss < 2, f"{loss:.1f}%")

    if not call_dropped:
        r.check(f"Call stable for {duration}s", True)

    teardown_call(ctx)
    return r

# ============================================================
# Test: Fidelity
# ============================================================

def test_fidelity(duration=30):
    print("\n" + "=" * 60)
    print("  FIDELITY TEST")
    print("=" * 60)
    r = TestResult()

    if not HAS_OPUS:
        r.check("Opus available", False, "opuslib not installed")
        return r

    ctx, err = setup_call()
    if err:
        r.check("Call setup", False, err)
        teardown_call(ctx)
        return r
    r.check("Call setup", True)

    first_codec = ctx["caller_codecs"][0][1] if ctx["caller_codecs"] else "none"
    r.check("Opus primary codec", "opus" in first_codec.lower(), f"first: {first_codec}")

    # Generate reference audio
    print(f"  Generating 1kHz sine ({duration}s)...")
    ref_pcm = generate_sine_pcm(1000, duration + 1)
    callee_ref = generate_sine_pcm(440, duration + 1)

    enc1 = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
    enc2 = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
    caller_frames = pcm_to_opus_frames(ref_pcm, enc1)
    callee_frames = pcm_to_opus_frames(callee_ref, enc2)

    stop = threading.Event()
    caller_pkts = []
    callee_pkts = []

    def recv_caller():
        nonlocal caller_pkts
        caller_pkts = rtp_receiver(ctx["caller_rtp"], stop)

    def recv_callee():
        nonlocal callee_pkts
        callee_pkts = rtp_receiver(ctx["callee_rtp"], stop)

    threads = [
        threading.Thread(target=recv_caller, daemon=True),
        threading.Thread(target=recv_callee, daemon=True),
        threading.Thread(target=rtp_sender, args=(ctx["caller_rtp"], ctx["caller_rtp_target"], caller_frames, stop), daemon=True),
    ]
    if ctx["listener"].rtp_target:
        threads.append(threading.Thread(
            target=rtp_sender, args=(ctx["callee_rtp"], ctx["listener"].rtp_target, callee_frames, stop), daemon=True
        ))
    for t in threads:
        t.start()

    print(f"  Streaming for {duration}s...")
    time.sleep(duration)
    stop.set()
    time.sleep(1)

    r.check("Callee received packets", len(callee_pkts) > 0, f"{len(callee_pkts)} pkts")
    r.check("Caller received packets", len(caller_pkts) > 0, f"{len(caller_pkts)} pkts")

    # Decode and analyze callee's received audio (1kHz from caller)
    if callee_pkts:
        received = decode_opus_packets(callee_pkts)
        if received is not None and len(received) > SAMPLE_RATE * 2:
            # Skip first/last 1s for stabilization
            skip = SAMPLE_RATE
            ref_trim = ref_pcm[skip:]
            rec_trim = received[skip:]
            ref_a, rec_a = align_signals(ref_trim, rec_trim)

            if len(ref_a) > SAMPLE_RATE:
                snr = compute_snr(ref_a, rec_a)
                r.check("SNR > 20 dB", snr > 20, f"{snr:.1f} dB")

                pesq_mos = run_pesq(ref_a, rec_a)
                if pesq_mos is not None:
                    r.check("PESQ MOS > 3.0", pesq_mos > 3.0, f"{pesq_mos:.2f}")
                else:
                    r.check("PESQ", False, "unavailable")

                visqol_mos = run_visqol(ref_a, rec_a)
                if visqol_mos is not None:
                    r.check("ViSQOL MOS > 3.0", visqol_mos > 3.0, f"{visqol_mos:.2f}")
                else:
                    r.check("ViSQOL", False, "unavailable")
            else:
                r.check("Enough audio for analysis", False, f"{len(ref_a)} samples")
        else:
            r.check("Audio decode", False, "insufficient data")

    teardown_call(ctx)
    return r

# ============================================================
# Test: Transcription
# ============================================================

def test_transcription(duration=45):
    print("\n" + "=" * 60)
    print("  TRANSCRIPTION TEST")
    print("=" * 60)
    r = TestResult()

    if not HAS_OPUS:
        r.check("Opus available", False)
        return r

    os.makedirs(SPEECH_DIR, exist_ok=True)
    en_wav = os.path.join(SPEECH_DIR, "english.wav")
    es_wav = os.path.join(SPEECH_DIR, "spanish.wav")
    en_text = "The quick brown fox jumps over the lazy dog"
    es_text = "buenos dias como estas hoy es un buen dia para trabajar"

    if not os.path.exists(en_wav):
        print("  Generating English speech...")
        if not generate_speech_wav(en_text, "en", en_wav):
            r.check("Generate English speech", False, "gtts/espeak unavailable")
            return r
    if not os.path.exists(es_wav):
        print("  Generating Spanish speech...")
        if not generate_speech_wav(es_text, "es", es_wav):
            r.check("Generate Spanish speech", False, "gtts/espeak unavailable")
            return r
    r.check("Speech files ready", True)

    en_pcm = load_wav_48k(en_wav)
    es_pcm = load_wav_48k(es_wav)

    # Pad: repeat speech with gaps to fill duration
    def pad_audio(pcm, total_s):
        total = SAMPLE_RATE * total_s
        out = np.zeros(total, dtype=np.int16)
        pos = SAMPLE_RATE * 2
        while pos + len(pcm) < total:
            out[pos:pos + len(pcm)] = pcm
            pos += len(pcm) + SAMPLE_RATE * 3
        return out

    en_padded = pad_audio(en_pcm, duration)
    es_padded = pad_audio(es_pcm, duration)

    enc1 = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
    enc2 = opuslib.Encoder(SAMPLE_RATE, 1, opuslib.APPLICATION_AUDIO)
    en_frames = pcm_to_opus_frames(en_padded, enc1)
    es_frames = pcm_to_opus_frames(es_padded, enc2)
    r.check("Speech encoded", len(en_frames) > 0 and len(es_frames) > 0,
            f"EN:{len(en_frames)} ES:{len(es_frames)} frames")

    # Note timestamps of recordings before call
    pre_recordings = set()
    if os.path.isdir(RECORDER_DIR):
        pre_recordings = set(os.listdir(RECORDER_DIR))

    ctx, err = setup_call()
    if err:
        r.check("Call setup", False, err)
        teardown_call(ctx)
        return r
    r.check("Call setup", True)

    stop = threading.Event()
    threads = [
        threading.Thread(target=rtp_sender, args=(ctx["caller_rtp"], ctx["caller_rtp_target"], en_frames, stop), daemon=True),
    ]
    if ctx["listener"].rtp_target:
        threads.append(threading.Thread(
            target=rtp_sender, args=(ctx["callee_rtp"], ctx["listener"].rtp_target, es_frames, stop), daemon=True
        ))
    for t in threads:
        t.start()

    print(f"  Streaming speech for {duration}s...")
    time.sleep(duration)
    stop.set()
    time.sleep(1)

    teardown_call(ctx)

    # Wait for transcription
    print("  Waiting 60s for transcription...")
    time.sleep(60)

    # Check for new recordings
    en_match = False
    es_match = False
    transcript_found = False

    if os.path.isdir(RECORDER_DIR):
        post_files = set(os.listdir(RECORDER_DIR))
        new_files = post_files - pre_recordings
        print(f"  New recording files: {len(new_files)}")

        for fname in sorted(new_files):
            fpath = os.path.join(RECORDER_DIR, fname)
            # Check associated transcript files
            base = os.path.splitext(fpath)[0]
            for ext in (".txt", ".json", ".transcript"):
                tpath = base + ext
                if os.path.exists(tpath):
                    with open(tpath) as f:
                        text = f.read().lower()
                    if text.strip():
                        transcript_found = True
                        if any(w in text for w in ["fox", "quick", "brown", "lazy", "dog"]):
                            en_match = True
                        if any(w in text for w in ["buenos", "dias", "como", "estas", "trabajar", "buen"]):
                            es_match = True

    # Also check via API
    if not transcript_found:
        try:
            import urllib.request
            import ssl
            ssl_ctx = ssl.create_default_context()
            ssl_ctx.check_hostname = False
            ssl_ctx.verify_mode = ssl.CERT_NONE
            req = urllib.request.Request(f"{API_BASE}/api/callrecords")
            with urllib.request.urlopen(req, context=ssl_ctx, timeout=10) as resp:
                records = json.loads(resp.read())
                if isinstance(records, list):
                    for rec in records[:5]:
                        tx = str(rec.get("transcript", "") or rec.get("transcription", "")).lower()
                        if tx:
                            transcript_found = True
                            if any(w in tx for w in ["fox", "quick", "brown"]):
                                en_match = True
                            if any(w in tx for w in ["buenos", "dias", "trabajar"]):
                                es_match = True
        except Exception as e:
            print(f"  [WARN] API check: {e}")

    # Check server log as last resort
    if not transcript_found:
        try:
            log_path = os.path.expanduser("~/rustpbx.log")
            with open(log_path) as f:
                lines = f.readlines()[-1000:]
            text = "\n".join(lines).lower()
            if "transcript" in text:
                transcript_found = True
                if any(w in text for w in ["fox", "quick", "brown"]):
                    en_match = True
                if any(w in text for w in ["buenos", "dias", "trabajar"]):
                    es_match = True
        except Exception:
            pass

    r.check("Transcription produced", transcript_found)
    r.check("English recognized", en_match, f"expected: fox, quick, brown, lazy, dog")
    r.check("Spanish recognized", es_match, f"expected: buenos, dias, trabajar")

    return r

# ============================================================
# Main
# ============================================================

def main():
    parser = argparse.ArgumentParser(description="RustPBX Call Quality Test Suite")
    parser.add_argument("test", nargs="?", default="all",
                        choices=["smoke", "fidelity", "transcription", "all"],
                        help="Test to run (default: all)")
    parser.add_argument("--duration", type=int, help="Override duration (seconds)")
    args = parser.parse_args()

    print("=" * 60)
    print("  RustPBX Call Quality Test Suite")
    print(f"  Server: {SERVER}:{SERVER_PORT}")
    print(f"  Caller: {CALLER['ext']} (SIP:{CALLER['sip_port']} RTP:{CALLER['rtp_port']})")
    print(f"  Callee: {CALLEE['ext']} (SIP:{CALLEE['sip_port']} RTP:{CALLEE['rtp_port']})")
    print(f"  Opus: {'yes' if HAS_OPUS else 'NO'}")
    print(f"  PESQ: {'yes' if HAS_PESQ else 'NO'}")
    print(f"  ViSQOL: {'yes' if HAS_VISQOL else 'NO'}")
    print("=" * 60)

    results = []

    if args.test in ("smoke", "all"):
        results.append(("SMOKE", test_smoke(args.duration or 60)))
        time.sleep(3)

    if args.test in ("fidelity", "all"):
        results.append(("FIDELITY", test_fidelity(args.duration or 30)))
        time.sleep(3)

    if args.test in ("transcription", "all"):
        results.append(("TRANSCRIPTION", test_transcription(args.duration or 45)))

    # Summary
    print("\n" + "=" * 60)
    print("  SUMMARY")
    print("=" * 60)
    all_pass = True
    for name, r in results:
        tag = "PASS" if r.passed else "FAIL"
        ok = sum(1 for _, p, _ in r.checks if p)
        total = len(r.checks)
        print(f"  [{tag}] {name}: {ok}/{total} checks passed")
        if not r.passed:
            all_pass = False
            for cname, cpassed, cdetail in r.checks:
                if not cpassed:
                    print(f"         FAIL: {cname}" + (f" ({cdetail})" if cdetail else ""))
    print("=" * 60)

    sys.exit(0 if all_pass else 1)


if __name__ == "__main__":
    main()
