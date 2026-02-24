#!/usr/bin/env python3
"""
SIP Load Generator -- Reusable load testing tool for RustPBX.

Creates a pool of virtual SIP user agents using raw UDP sockets and asyncio.
Each agent has its own socket, Call-ID space, and CSeq counter.  Supports
concurrent REGISTER, INVITE/BYE flows with digest authentication, and
optional RTP media exchange (silence frames).

No external SIP libraries required -- pure Python stdlib + asyncio.

Usage as CLI:
  python tests/sip_load_generator.py --host 127.0.0.1 --port 5060 \\
         --agents 20 --calls 50 --concurrency 10 --duration 5 \\
         --report-file load_report.json

Usage as module (from test_L6_load.py or elsewhere):
  from sip_load_generator import SIPLoadGenerator
  gen = SIPLoadGenerator("127.0.0.1", 5060, num_agents=20)
  asyncio.run(gen.run_load_test(num_calls=50, concurrency=10, duration_secs=5))
  print(gen.summary())

Environment variables (all optional):
  RUSTPBX_HOST          Server IP           (default: 127.0.0.1)
  RUSTPBX_SIP_PORT      SIP port            (default: 5060)
  RUSTPBX_EXTERNAL_IP   Public IP for URIs  (default: same as HOST)
"""

import argparse
import asyncio
import hashlib
import json
import logging
import os
import random
import re
import socket
import statistics
import string
import struct
import sys
import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple

# ---------------------------------------------------------------------------
# Configuration defaults
# ---------------------------------------------------------------------------

DEFAULT_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
DEFAULT_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
DEFAULT_EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", DEFAULT_HOST)

logger = logging.getLogger("sip_load_generator")


# ---------------------------------------------------------------------------
# SIP protocol helpers (adapted from test_L3_sip.py / test_L5_media.py)
# ---------------------------------------------------------------------------

def _gen_branch():
    return "z9hG4bK" + "".join(random.choices(string.ascii_lowercase + string.digits, k=12))


def _gen_tag():
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=8))


def _gen_callid(prefix="lg"):
    return prefix + "-" + "".join(random.choices(string.ascii_lowercase + string.digits, k=16))


def _md5hex(s):
    return hashlib.md5(s.encode()).hexdigest()


def _compute_digest(username, realm, password, method, uri, nonce):
    ha1 = _md5hex(f"{username}:{realm}:{password}")
    ha2 = _md5hex(f"{method}:{uri}")
    return _md5hex(f"{ha1}:{nonce}:{ha2}")


def _get_response_code(data: str) -> int:
    m = re.match(r"SIP/2\.0 (\d+)", data)
    return int(m.group(1)) if m else 0


def _get_method(data: str) -> str:
    m = re.match(r"^(\w+)\s+sip:", data)
    return m.group(1) if m else ""


def _get_header(data: str, name: str) -> str:
    for line in data.split("\r\n"):
        if line.lower().startswith(name.lower() + ":"):
            return line.split(":", 1)[1].strip()
    return ""


def _get_to_tag(data: str) -> str:
    to_hdr = _get_header(data, "To")
    m = re.search(r"tag=([^\s;>]+)", to_hdr)
    return m.group(1) if m else ""


def _get_all_via(data: str) -> List[str]:
    vias = []
    for line in data.split("\r\n"):
        if line.lower().startswith("via:"):
            vias.append(line)
    return vias


def _parse_www_authenticate(header_line: str) -> Tuple[str, str]:
    realm = re.search(r'realm="([^"]*)"', header_line)
    nonce = re.search(r'nonce="([^"]*)"', header_line)
    return (realm.group(1) if realm else ""), (nonce.group(1) if nonce else "")


def _parse_sdp(sdp_text: str) -> Tuple[Optional[str], Optional[int], List[Tuple[int, str]]]:
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


def _build_sdp(rtp_port: int, local_ip: str) -> str:
    """Build a minimal SDP body offering PCMU + telephone-event."""
    sid = str(random.randint(100000, 999999))
    return (
        "v=0\r\n"
        f"o=- {sid} {sid} IN IP4 {local_ip}\r\n"
        "s=SIPLoadGen\r\n"
        f"c=IN IP4 {local_ip}\r\n"
        "t=0 0\r\n"
        f"m=audio {rtp_port} RTP/AVP 0 101\r\n"
        "a=rtpmap:0 PCMU/8000\r\n"
        "a=rtpmap:101 telephone-event/8000\r\n"
        "a=fmtp:101 0-16\r\n"
        "a=sendrecv\r\n"
    )


def _build_rtp(pt: int, seq: int, timestamp: int, ssrc: int,
               payload: bytes, marker: bool = False) -> bytes:
    """Build a single RTP packet."""
    b0 = 0x80
    b1 = (0x80 if marker else 0x00) | (pt & 0x7f)
    hdr = struct.pack("!BBHII", b0, b1, seq & 0xffff, timestamp & 0xffffffff, ssrc)
    return hdr + payload


# 160 bytes of PCMU silence (mu-law 0xFF = digital silence)
PCMU_SILENCE_FRAME = b"\xff" * 160


# ---------------------------------------------------------------------------
# Metrics data classes
# ---------------------------------------------------------------------------

@dataclass
class CallMetrics:
    """Metrics for a single call attempt."""
    call_id: str = ""
    caller: str = ""
    callee: str = ""
    start_time: float = 0.0
    invite_sent_time: float = 0.0
    first_response_time: float = 0.0
    ringing_time: float = 0.0
    answer_time: float = 0.0
    bye_time: float = 0.0
    end_time: float = 0.0
    setup_time: float = 0.0          # INVITE sent -> 200 OK received
    rtp_packets_sent: int = 0
    rtp_packets_received: int = 0
    final_response_code: int = 0
    success: bool = False
    error: str = ""
    error_category: str = ""         # timeout, auth_fail, rejected, network, internal


@dataclass
class LoadTestMetrics:
    """Aggregate metrics for a load test run."""
    total_calls: int = 0
    successful_calls: int = 0
    failed_calls: int = 0
    call_metrics: List[CallMetrics] = field(default_factory=list)
    start_time: float = 0.0
    end_time: float = 0.0
    num_agents: int = 0
    concurrency: int = 0

    @property
    def duration(self) -> float:
        return self.end_time - self.start_time if self.end_time > 0 else 0.0

    @property
    def success_rate(self) -> float:
        return self.successful_calls / self.total_calls if self.total_calls > 0 else 0.0

    @property
    def calls_per_second(self) -> float:
        return self.total_calls / self.duration if self.duration > 0 else 0.0

    @property
    def setup_times(self) -> List[float]:
        return [m.setup_time for m in self.call_metrics if m.success and m.setup_time > 0]

    @property
    def error_breakdown(self) -> Dict[str, int]:
        breakdown: Dict[str, int] = {}
        for m in self.call_metrics:
            if not m.success and m.error_category:
                breakdown[m.error_category] = breakdown.get(m.error_category, 0) + 1
        return breakdown

    def summary(self) -> Dict[str, Any]:
        """Generate a summary report dict."""
        setup = self.setup_times
        setup_sorted = sorted(setup) if setup else []
        summary: Dict[str, Any] = {
            "total_calls": self.total_calls,
            "successful_calls": self.successful_calls,
            "failed_calls": self.failed_calls,
            "success_rate": round(self.success_rate, 4),
            "duration_secs": round(self.duration, 2),
            "calls_per_second": round(self.calls_per_second, 2),
            "num_agents": self.num_agents,
            "concurrency": self.concurrency,
            "error_breakdown": self.error_breakdown,
        }
        if setup_sorted:
            summary["setup_time_stats"] = {
                "min": round(setup_sorted[0], 4),
                "max": round(setup_sorted[-1], 4),
                "mean": round(statistics.mean(setup), 4),
                "median": round(statistics.median(setup), 4),
                "p95": round(setup_sorted[int(len(setup_sorted) * 0.95)], 4)
                       if len(setup_sorted) > 1 else round(setup_sorted[0], 4),
                "p99": round(setup_sorted[min(int(len(setup_sorted) * 0.99), len(setup_sorted) - 1)], 4),
                "stdev": round(statistics.stdev(setup), 4) if len(setup) > 1 else 0.0,
                "count": len(setup),
            }
        else:
            summary["setup_time_stats"] = None
        return summary

    def print_report(self):
        """Print a formatted summary to stdout."""
        s = self.summary()
        print("\n" + "=" * 70)
        print("  SIP Load Test Report")
        print("=" * 70)
        print(f"  Total calls:      {s['total_calls']}")
        print(f"  Successful:       {s['successful_calls']}")
        print(f"  Failed:           {s['failed_calls']}")
        print(f"  Success rate:     {s['success_rate'] * 100:.1f}%")
        print(f"  Duration:         {s['duration_secs']:.2f}s")
        print(f"  Throughput:       {s['calls_per_second']:.2f} calls/sec")
        print(f"  Agents:           {s['num_agents']}")
        print(f"  Concurrency:      {s['concurrency']}")

        if s["setup_time_stats"]:
            st = s["setup_time_stats"]
            print(f"\n  Call Setup Times ({st['count']} samples):")
            print(f"    Min:    {st['min']:.4f}s")
            print(f"    Mean:   {st['mean']:.4f}s")
            print(f"    Median: {st['median']:.4f}s")
            print(f"    p95:    {st['p95']:.4f}s")
            print(f"    p99:    {st['p99']:.4f}s")
            print(f"    Max:    {st['max']:.4f}s")
            if st["stdev"] > 0:
                print(f"    StdDev: {st['stdev']:.4f}s")

        if s["error_breakdown"]:
            print(f"\n  Error Breakdown:")
            for cat, count in sorted(s["error_breakdown"].items(), key=lambda x: -x[1]):
                print(f"    {cat}: {count}")

        print("=" * 70)


# ---------------------------------------------------------------------------
# SIPAgent -- a single virtual SIP user agent
# ---------------------------------------------------------------------------

class SIPAgent:
    """A virtual SIP user agent with its own UDP socket and SIP state."""

    def __init__(self, extension: str, password: str,
                 server_host: str, server_port: int,
                 local_ip: str, local_port: int,
                 external_ip: str):
        self.extension = extension
        self.password = password
        self.server_host = server_host
        self.server_port = server_port
        self.local_ip = local_ip
        self.local_port = local_port
        self.external_ip = external_ip

        self.sock: Optional[socket.socket] = None
        self.rtp_sock: Optional[socket.socket] = None
        self.rtp_port: int = local_port + 1  # RTP port = SIP port + 1
        self.cseq: int = 0
        self.registered: bool = False
        self._call_id_counter: int = 0

        # For callee mode
        self._incoming_handler_task: Optional[asyncio.Task] = None
        self._stop_incoming = asyncio.Event()

    def _next_callid(self) -> str:
        self._call_id_counter += 1
        return f"lg-{self.extension}-{self._call_id_counter}-{_gen_callid()}"

    def _next_cseq(self) -> int:
        self.cseq += 1
        return self.cseq

    def open(self):
        """Create and bind the SIP and RTP UDP sockets."""
        if self.sock is not None:
            return
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind(("0.0.0.0", self.local_port))
        self.sock.setblocking(False)

        self.rtp_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.rtp_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.rtp_sock.bind(("0.0.0.0", self.rtp_port))
        self.rtp_sock.setblocking(False)

    def close(self):
        """Close all sockets."""
        self._stop_incoming.set()
        if self._incoming_handler_task and not self._incoming_handler_task.done():
            self._incoming_handler_task.cancel()
        if self.sock:
            try:
                self.sock.close()
            except OSError:
                pass
            self.sock = None
        if self.rtp_sock:
            try:
                self.rtp_sock.close()
            except OSError:
                pass
            self.rtp_sock = None
        self.registered = False

    async def _send(self, msg: str):
        """Send a SIP message to the server via the event loop."""
        loop = asyncio.get_running_loop()
        data = msg.encode() if isinstance(msg, str) else msg
        await loop.run_in_executor(
            None, lambda: self.sock.sendto(data, (self.server_host, self.server_port))
        )

    async def _recv(self, timeout: float = 5.0) -> Optional[str]:
        """Receive a SIP message with timeout."""
        loop = asyncio.get_running_loop()
        deadline = loop.time() + timeout
        while True:
            remaining = deadline - loop.time()
            if remaining <= 0:
                return None
            try:
                data = await asyncio.wait_for(
                    loop.run_in_executor(None, lambda: self.sock.recv(65535)),
                    timeout=min(remaining, 0.5),
                )
                return data.decode(errors="replace")
            except (asyncio.TimeoutError, BlockingIOError, OSError):
                remaining = deadline - loop.time()
                if remaining <= 0:
                    return None
                await asyncio.sleep(0.01)

    async def _recv_sip(self, sock: socket.socket, timeout: float = 5.0) -> Tuple[Optional[str], Optional[tuple]]:
        """Receive a SIP message from a specific socket with timeout."""
        loop = asyncio.get_running_loop()
        deadline = loop.time() + timeout
        while True:
            remaining = deadline - loop.time()
            if remaining <= 0:
                return None, None
            try:
                data, addr = await asyncio.wait_for(
                    loop.run_in_executor(None, lambda: sock.recvfrom(65535)),
                    timeout=min(remaining, 0.5),
                )
                return data.decode(errors="replace"), addr
            except (asyncio.TimeoutError, BlockingIOError, OSError):
                remaining = deadline - loop.time()
                if remaining <= 0:
                    return None, None
                await asyncio.sleep(0.01)

    async def register(self) -> bool:
        """Perform REGISTER with digest auth. Returns True on 200 OK."""
        tag = _gen_tag()
        cid = self._next_callid()
        from_uri = f"sip:{self.extension}@{self.external_ip}"
        contact = f"<sip:{self.extension}@{self.server_host}:{self.local_port};transport=udp>"
        cseq = self._next_cseq()
        branch = _gen_branch()

        # Step 1: unauthenticated REGISTER
        msg = (
            f"REGISTER sip:{self.external_ip} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {self.server_host}:{self.local_port};branch={branch};rport\r\n"
            f"From: <{from_uri}>;tag={tag}\r\n"
            f"To: <{from_uri}>\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq} REGISTER\r\n"
            f"Contact: {contact}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 3600\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        await self._send(msg)
        resp = await self._recv(timeout=5)
        if resp is None:
            logger.warning(f"Agent {self.extension}: No response to REGISTER")
            return False

        code = _get_response_code(resp)
        if code == 200:
            self.registered = True
            return True
        if code not in (401, 407):
            logger.warning(f"Agent {self.extension}: Unexpected REGISTER response: {code}")
            return False

        # Step 2: extract challenge and compute digest
        auth_hdr = ""
        for line in resp.split("\r\n"):
            low = line.lower()
            if low.startswith("www-authenticate:") or low.startswith("proxy-authenticate:"):
                auth_hdr = line.split(":", 1)[1].strip()
                break

        if not auth_hdr:
            logger.warning(f"Agent {self.extension}: No auth header in {code} response")
            return False

        realm, nonce = _parse_www_authenticate(auth_hdr)
        realm = realm or self.external_ip
        uri = f"sip:{realm}"
        digest = _compute_digest(self.extension, realm, self.password, "REGISTER", uri, nonce)
        hdr_name = "Authorization" if code == 401 else "Proxy-Authorization"
        auth_line = (
            f'Digest username="{self.extension}", realm="{realm}", '
            f'nonce="{nonce}", uri="{uri}", '
            f'response="{digest}", algorithm=MD5'
        )

        # Step 3: authenticated REGISTER
        cseq = self._next_cseq()
        branch = _gen_branch()
        msg = (
            f"REGISTER sip:{realm} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {self.server_host}:{self.local_port};branch={branch};rport\r\n"
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
        await self._send(msg)
        resp = await self._recv(timeout=5)
        if resp is None:
            logger.warning(f"Agent {self.extension}: No response to authenticated REGISTER")
            return False

        code = _get_response_code(resp)
        if code == 200:
            self.registered = True
            logger.debug(f"Agent {self.extension}: Registered successfully")
            return True

        logger.warning(f"Agent {self.extension}: REGISTER auth failed: {code}")
        return False

    async def unregister(self) -> bool:
        """Send REGISTER Expires:0 to unregister."""
        tag = _gen_tag()
        from_uri = f"sip:{self.extension}@{self.external_ip}"
        contact = f"<sip:{self.extension}@{self.server_host}:{self.local_port};transport=udp>"
        branch = _gen_branch()
        cseq = self._next_cseq()
        msg = (
            f"REGISTER sip:{self.external_ip} SIP/2.0\r\n"
            f"Via: SIP/2.0/UDP {self.server_host}:{self.local_port};branch={branch};rport\r\n"
            f"From: <{from_uri}>;tag={tag}\r\n"
            f"To: <{from_uri}>\r\n"
            f"Call-ID: unreg-{self._next_callid()}\r\n"
            f"CSeq: {cseq} REGISTER\r\n"
            f"Contact: {contact}\r\n"
            f"Max-Forwards: 70\r\n"
            f"Expires: 0\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        await self._send(msg)
        resp = await self._recv(timeout=3)
        self.registered = False
        return resp is not None

    async def make_call_to(self, callee_agent: "SIPAgent",
                           duration_secs: float = 3.0,
                           send_rtp: bool = True) -> CallMetrics:
        """Place a call to another agent, hold for duration, then BYE.

        The callee_agent must have start_incoming_handler() running.
        Returns CallMetrics with timing and success info.
        """
        metrics = CallMetrics(
            caller=self.extension,
            callee=callee_agent.extension,
            start_time=time.monotonic(),
        )

        from_uri = f"sip:{self.extension}@{self.external_ip}"
        to_uri = f"sip:{callee_agent.extension}@{self.server_host}"
        local_tag = _gen_tag()
        call_id = self._next_callid()
        metrics.call_id = call_id
        contact = f"<sip:{self.extension}@{self.server_host}:{self.local_port}>"
        cseq = self._next_cseq()
        sdp_body = _build_sdp(self.rtp_port, self.server_host)

        def _make_invite(br, cs, auth_hdr=None):
            auth = f"{auth_hdr}\r\n" if auth_hdr else ""
            return (
                f"INVITE {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {self.server_host}:{self.local_port};branch={br};rport\r\n"
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

        async def _send_ack(cs, to_tag_val=None):
            to_field = f"<{to_uri}>;tag={to_tag_val}" if to_tag_val else f"<{to_uri}>"
            ack = (
                f"ACK {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {self.server_host}:{self.local_port};branch={_gen_branch()};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: {to_field}\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {cs} ACK\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            await self._send(ack)

        # Send INVITE
        branch = _gen_branch()
        metrics.invite_sent_time = time.monotonic()
        await self._send(_make_invite(branch, cseq))

        auth_done = False
        rtp_target = None
        to_tag = ""
        answered = False

        try:
            deadline = asyncio.get_running_loop().time() + 30
            while asyncio.get_running_loop().time() < deadline:
                resp = await self._recv(timeout=2)
                if resp is None:
                    continue

                # Handle SIP request forwarded to us (e.g. BYE from server)
                if not resp.startswith("SIP/2.0"):
                    method = _get_method(resp)
                    if method == "BYE":
                        # Server sent us a BYE, respond with 200 OK
                        vias = _get_all_via(resp)
                        via_block = "\r\n".join(vias)
                        ok_resp = (
                            f"SIP/2.0 200 OK\r\n"
                            f"{via_block}\r\n"
                            f"From: {_get_header(resp, 'From')}\r\n"
                            f"To: {_get_header(resp, 'To')}\r\n"
                            f"Call-ID: {_get_header(resp, 'Call-ID')}\r\n"
                            f"CSeq: {_get_header(resp, 'CSeq')}\r\n"
                            f"Content-Length: 0\r\n\r\n"
                        )
                        await self._send(ok_resp)
                    continue

                code = _get_response_code(resp)

                if metrics.first_response_time == 0.0:
                    metrics.first_response_time = time.monotonic()

                if code == 100:
                    continue
                elif code in (180, 183):
                    metrics.ringing_time = time.monotonic()
                    continue
                elif code in (401, 407) and not auth_done:
                    auth_done = True
                    await _send_ack(cseq)

                    auth_hdr_val = ""
                    for line in resp.split("\r\n"):
                        low = line.lower()
                        if low.startswith("proxy-authenticate:") or low.startswith("www-authenticate:"):
                            auth_hdr_val = line.split(":", 1)[1].strip()
                            break

                    realm, nonce = _parse_www_authenticate(auth_hdr_val)
                    realm = realm or self.external_ip
                    digest = _compute_digest(
                        self.extension, realm, self.password,
                        "INVITE", to_uri, nonce,
                    )
                    hdr_name = "Proxy-Authorization" if code == 407 else "Authorization"
                    auth_line = (
                        f'{hdr_name}: Digest username="{self.extension}", realm="{realm}", '
                        f'nonce="{nonce}", uri="{to_uri}", response="{digest}", algorithm=MD5'
                    )
                    cseq = self._next_cseq()
                    branch = _gen_branch()
                    await self._send(_make_invite(branch, cseq, auth_line))
                    continue

                elif code == 200:
                    metrics.answer_time = time.monotonic()
                    metrics.setup_time = metrics.answer_time - metrics.invite_sent_time
                    to_tag = _get_to_tag(resp)
                    await _send_ack(cseq, to_tag)
                    answered = True

                    # Parse remote SDP for RTP target
                    sdp_start = resp.find("\r\n\r\n")
                    if sdp_start >= 0:
                        ip, port, codecs = _parse_sdp(resp[sdp_start + 4:])
                        if ip and port:
                            rtp_target = (ip, port)

                    break

                elif code >= 400:
                    metrics.final_response_code = code
                    metrics.error = f"INVITE rejected with {code}"
                    metrics.error_category = "rejected"
                    await _send_ack(cseq, _get_to_tag(resp) or "")
                    metrics.end_time = time.monotonic()
                    return metrics

            if not answered:
                metrics.error = "Timeout waiting for 200 OK"
                metrics.error_category = "timeout"
                metrics.end_time = time.monotonic()
                return metrics

            metrics.final_response_code = 200

            # Hold call: send RTP silence and wait
            rtp_task = None
            if send_rtp and rtp_target:
                rtp_task = asyncio.create_task(
                    self._send_rtp_silence(rtp_target, duration_secs, metrics)
                )

            await asyncio.sleep(duration_secs)

            if rtp_task:
                rtp_task.cancel()
                try:
                    await rtp_task
                except asyncio.CancelledError:
                    pass

            # Send BYE
            bye_cseq = self._next_cseq()
            to_field = f"<{to_uri}>;tag={to_tag}" if to_tag else f"<{to_uri}>"
            bye = (
                f"BYE {to_uri} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP {self.server_host}:{self.local_port};branch={_gen_branch()};rport\r\n"
                f"From: <{from_uri}>;tag={local_tag}\r\n"
                f"To: {to_field}\r\n"
                f"Call-ID: {call_id}\r\n"
                f"CSeq: {bye_cseq} BYE\r\n"
                f"Max-Forwards: 70\r\n"
                f"Content-Length: 0\r\n\r\n"
            )
            metrics.bye_time = time.monotonic()
            await self._send(bye)
            bye_resp = await self._recv(timeout=5)
            # Drain any extra messages
            if bye_resp:
                pass

            metrics.success = True
            metrics.end_time = time.monotonic()
            return metrics

        except Exception as exc:
            metrics.error = str(exc)
            metrics.error_category = "internal"
            metrics.end_time = time.monotonic()
            return metrics

    async def _send_rtp_silence(self, target: Tuple[str, int],
                                duration_secs: float,
                                metrics: CallMetrics):
        """Send PCMU silence frames at 20ms pacing."""
        loop = asyncio.get_running_loop()
        ssrc = random.randint(1, 0xFFFFFFFF)
        seq = random.randint(0, 0xFFFF)
        ts = random.randint(0, 0xFFFFFFFF)
        interval = 0.020  # 20ms
        end_time = loop.time() + duration_secs

        try:
            while loop.time() < end_time:
                pkt = _build_rtp(0, seq, ts, ssrc, PCMU_SILENCE_FRAME, marker=(seq == 0))
                try:
                    self.rtp_sock.sendto(pkt, target)
                    metrics.rtp_packets_sent += 1
                except OSError:
                    break
                seq = (seq + 1) & 0xFFFF
                ts = (ts + 160) & 0xFFFFFFFF
                await asyncio.sleep(interval)
        except asyncio.CancelledError:
            pass

    async def start_incoming_handler(self):
        """Start background task that auto-answers incoming INVITEs."""
        self._stop_incoming.clear()
        self._incoming_handler_task = asyncio.create_task(
            self._incoming_handler_loop()
        )

    async def _incoming_handler_loop(self):
        """Listen for and auto-answer incoming INVITE requests."""
        while not self._stop_incoming.is_set():
            try:
                msg, addr = await self._recv_sip(self.sock, timeout=1.0)
                if msg is None:
                    continue

                if msg.startswith("INVITE "):
                    await self._handle_invite(msg, addr)
                elif msg.startswith("ACK "):
                    pass  # Expected after our 200 OK
                elif msg.startswith("BYE "):
                    await self._reply_200(msg, addr)
                elif msg.startswith("CANCEL "):
                    await self._reply_200(msg, addr)
                # Ignore responses (they are for calls we initiated, handled elsewhere)
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug(f"Agent {self.extension} incoming handler error: {exc}")

    async def _handle_invite(self, msg: str, addr: tuple):
        """Auto-answer an incoming INVITE with 180 Ringing + 200 OK."""
        cid = _get_header(msg, "Call-ID")
        from_h = _get_header(msg, "From")
        to_h = _get_header(msg, "To")
        cseq_h = _get_header(msg, "CSeq")
        vias = _get_all_via(msg)
        via_block = "\r\n".join(vias)

        to_tag = _gen_tag()
        to_with_tag = to_h + f";tag={to_tag}" if "tag=" not in to_h else to_h
        contact = f"<sip:{self.extension}@{self.server_host}:{self.local_port}>"

        # 180 Ringing
        ringing = (
            f"SIP/2.0 180 Ringing\r\n"
            f"{via_block}\r\n"
            f"From: {from_h}\r\n"
            f"To: {to_with_tag}\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq_h}\r\n"
            f"Contact: {contact}\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(
            None, lambda: self.sock.sendto(ringing.encode(), addr)
        )

        await asyncio.sleep(0.2)

        # 200 OK with SDP
        sdp_body = _build_sdp(self.rtp_port, self.server_host)
        ok = (
            f"SIP/2.0 200 OK\r\n"
            f"{via_block}\r\n"
            f"From: {from_h}\r\n"
            f"To: {to_with_tag}\r\n"
            f"Call-ID: {cid}\r\n"
            f"CSeq: {cseq_h}\r\n"
            f"Contact: {contact}\r\n"
            f"Content-Type: application/sdp\r\n"
            f"Content-Length: {len(sdp_body)}\r\n\r\n"
            f"{sdp_body}"
        )
        await loop.run_in_executor(
            None, lambda: self.sock.sendto(ok.encode(), addr)
        )

    async def _reply_200(self, msg: str, addr: tuple):
        """Send a generic 200 OK response to a request."""
        vias = _get_all_via(msg)
        via_block = "\r\n".join(vias)
        ok = (
            f"SIP/2.0 200 OK\r\n"
            f"{via_block}\r\n"
            f"From: {_get_header(msg, 'From')}\r\n"
            f"To: {_get_header(msg, 'To')}\r\n"
            f"Call-ID: {_get_header(msg, 'Call-ID')}\r\n"
            f"CSeq: {_get_header(msg, 'CSeq')}\r\n"
            f"Content-Length: 0\r\n\r\n"
        )
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(
            None, lambda: self.sock.sendto(ok.encode(), addr)
        )

    def stop_incoming_handler(self):
        """Signal the incoming handler to stop."""
        self._stop_incoming.set()
        if self._incoming_handler_task and not self._incoming_handler_task.done():
            self._incoming_handler_task.cancel()


# ---------------------------------------------------------------------------
# SIPLoadGenerator -- the main orchestrator
# ---------------------------------------------------------------------------

class SIPLoadGenerator:
    """Orchestrates a pool of SIPAgent instances for load testing.

    Args:
        host:           SIP server hostname or IP.
        port:           SIP server port.
        num_agents:     Number of virtual user agents to create.
        base_extension: Starting extension number (agents get sequential numbers).
        password_fn:    Optional function(extension) -> password. Defaults to
                        "test{extension}" (e.g. extension 2001 -> password "test2001").
        external_ip:    External IP for SIP URIs (defaults to host).
        base_port:      Starting local UDP port (agents get sequential even ports).
    """

    def __init__(self, host: str = DEFAULT_HOST,
                 port: int = DEFAULT_SIP_PORT,
                 num_agents: int = 10,
                 base_extension: int = 2000,
                 password_fn=None,
                 external_ip: str = None,
                 base_port: int = 19000):
        self.host = host
        self.port = port
        self.num_agents = num_agents
        self.base_extension = base_extension
        self.external_ip = external_ip or DEFAULT_EXTERNAL_IP
        self.base_port = base_port
        self.password_fn = password_fn or (lambda ext: f"test{ext}")

        self.agents: List[SIPAgent] = []
        self.metrics = LoadTestMetrics(num_agents=num_agents)
        self._created = False

    def _create_agents(self):
        """Instantiate all SIPAgent objects (idempotent)."""
        if self._created:
            return
        for i in range(self.num_agents):
            ext = str(self.base_extension + i)
            password = self.password_fn(ext)
            local_port = self.base_port + (i * 2)  # even ports for SIP, odd for RTP
            agent = SIPAgent(
                extension=ext,
                password=password,
                server_host=self.host,
                server_port=self.port,
                local_ip=self.host,
                local_port=local_port,
                external_ip=self.external_ip,
            )
            self.agents.append(agent)
        self._created = True

    def _open_all(self):
        """Open all agent sockets."""
        for agent in self.agents:
            agent.open()

    def _close_all(self):
        """Close all agent sockets."""
        for agent in self.agents:
            agent.close()

    async def register_all(self) -> Tuple[int, int]:
        """Register all agents concurrently.

        Returns (success_count, fail_count).
        """
        self._create_agents()
        self._open_all()
        tasks = [agent.register() for agent in self.agents]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        successes = sum(1 for r in results if r is True)
        failures = len(results) - successes
        logger.info(f"Registered {successes}/{len(self.agents)} agents ({failures} failures)")
        return successes, failures

    async def unregister_all(self) -> None:
        """Unregister all agents concurrently."""
        tasks = []
        for agent in self.agents:
            if agent.registered:
                tasks.append(agent.unregister())
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
        logger.info(f"Unregistered {len(tasks)} agents")

    async def start_callee_handlers(self):
        """Start incoming call handlers on all agents."""
        for agent in self.agents:
            await agent.start_incoming_handler()

    def stop_callee_handlers(self):
        """Stop incoming call handlers on all agents."""
        for agent in self.agents:
            agent.stop_incoming_handler()

    async def make_call(self, caller_idx: int, callee_idx: int,
                        duration_secs: float = 3.0,
                        send_rtp: bool = True) -> CallMetrics:
        """Make a call between two agents by index.

        The callee must have its incoming handler running.
        """
        if caller_idx < 0 or caller_idx >= len(self.agents):
            raise ValueError(f"Invalid caller_idx: {caller_idx}")
        if callee_idx < 0 or callee_idx >= len(self.agents):
            raise ValueError(f"Invalid callee_idx: {callee_idx}")
        if caller_idx == callee_idx:
            raise ValueError("Caller and callee must be different agents")

        caller = self.agents[caller_idx]
        callee = self.agents[callee_idx]
        return await caller.make_call_to(callee, duration_secs, send_rtp)

    async def make_concurrent_calls(self, pairs: List[Tuple[int, int]],
                                    duration_secs: float = 3.0,
                                    send_rtp: bool = True) -> List[CallMetrics]:
        """Make multiple calls simultaneously.

        Args:
            pairs: List of (caller_idx, callee_idx) tuples.
            duration_secs: How long to hold each call.
            send_rtp: Whether to send RTP silence during the call.

        Returns:
            List of CallMetrics for each call.
        """
        tasks = []
        for caller_idx, callee_idx in pairs:
            tasks.append(self.make_call(caller_idx, callee_idx, duration_secs, send_rtp))
        results = await asyncio.gather(*tasks, return_exceptions=True)

        metrics_list = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                m = CallMetrics(
                    caller=str(self.base_extension + pairs[i][0]),
                    callee=str(self.base_extension + pairs[i][1]),
                    error=str(result),
                    error_category="internal",
                    end_time=time.monotonic(),
                )
                metrics_list.append(m)
            else:
                metrics_list.append(result)
        return metrics_list

    async def run_load_test(self, num_calls: int = 50,
                            concurrency: int = 5,
                            duration_secs: float = 3.0,
                            send_rtp: bool = True,
                            inter_batch_delay: float = 0.5) -> LoadTestMetrics:
        """Run a load test: N total calls, M concurrent at a time.

        Agents are divided into caller/callee pairs.  Calls are made in
        batches of `concurrency` until `num_calls` total have been placed.

        Args:
            num_calls:         Total number of calls to make.
            concurrency:       Maximum simultaneous calls.
            duration_secs:     How long to hold each call.
            send_rtp:          Whether to send RTP silence during calls.
            inter_batch_delay: Seconds to wait between batches.

        Returns:
            LoadTestMetrics with all results.
        """
        self.metrics = LoadTestMetrics(
            total_calls=num_calls,
            num_agents=self.num_agents,
            concurrency=concurrency,
        )

        # Need at least 2 agents to make calls
        if self.num_agents < 2:
            raise ValueError("Need at least 2 agents for call testing")

        self._create_agents()
        self._open_all()

        logger.info(f"Starting load test: {num_calls} calls, "
                    f"{concurrency} concurrent, {duration_secs}s duration, "
                    f"{self.num_agents} agents")

        # Register all agents
        reg_ok, reg_fail = await self.register_all()
        if reg_ok < 2:
            logger.error(f"Cannot run load test: only {reg_ok} agents registered")
            self._close_all()
            return self.metrics

        # Start callee handlers on all agents
        await self.start_callee_handlers()

        # Build call pairs -- round-robin agents
        # Use first half as callers, second half as callees (or interleave)
        registered_indices = [i for i, a in enumerate(self.agents) if a.registered]
        if len(registered_indices) < 2:
            logger.error("Not enough registered agents for calls")
            self._close_all()
            return self.metrics

        self.metrics.start_time = time.monotonic()

        calls_made = 0
        while calls_made < num_calls:
            batch_size = min(concurrency, num_calls - calls_made)

            # Generate pairs for this batch
            pairs = []
            for b in range(batch_size):
                # Pick caller and callee from registered agents
                caller_idx = registered_indices[(calls_made + b) % len(registered_indices)]
                callee_idx = registered_indices[
                    (calls_made + b + 1) % len(registered_indices)
                ]
                # Make sure they are different
                if caller_idx == callee_idx:
                    callee_idx = registered_indices[
                        (calls_made + b + 2) % len(registered_indices)
                    ]
                pairs.append((caller_idx, callee_idx))

            # Execute batch
            batch_results = await self.make_concurrent_calls(
                pairs, duration_secs, send_rtp
            )

            for m in batch_results:
                self.metrics.call_metrics.append(m)
                if m.success:
                    self.metrics.successful_calls += 1
                else:
                    self.metrics.failed_calls += 1

            calls_made += batch_size

            if calls_made < num_calls and inter_batch_delay > 0:
                await asyncio.sleep(inter_batch_delay)

        self.metrics.end_time = time.monotonic()

        # Cleanup
        self.stop_callee_handlers()
        await self.unregister_all()
        self._close_all()

        logger.info(
            f"Load test complete: {self.metrics.successful_calls}/{num_calls} "
            f"calls succeeded in {self.metrics.duration:.2f}s"
        )

        return self.metrics

    def summary(self) -> Dict[str, Any]:
        """Return a summary dict of the last load test run."""
        return self.metrics.summary()

    def print_report(self):
        """Print a formatted report of the last load test run."""
        self.metrics.print_report()

    def write_report(self, filepath: str):
        """Write a JSON report of the last load test run."""
        report = self.metrics.summary()
        report["individual_calls"] = []
        for m in self.metrics.call_metrics:
            report["individual_calls"].append({
                "call_id": m.call_id,
                "caller": m.caller,
                "callee": m.callee,
                "success": m.success,
                "setup_time": round(m.setup_time, 4) if m.setup_time > 0 else None,
                "final_response_code": m.final_response_code,
                "rtp_packets_sent": m.rtp_packets_sent,
                "rtp_packets_received": m.rtp_packets_received,
                "error": m.error or None,
                "error_category": m.error_category or None,
            })
        with open(filepath, "w") as f:
            json.dump(report, f, indent=2)
        logger.info(f"Report written to {filepath}")


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="SIP Load Generator for RustPBX",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Quick test with 4 agents, 10 calls, 2 concurrent
  python tests/sip_load_generator.py --agents 4 --calls 10 --concurrency 2

  # Stress test against remote server with JSON report
  python tests/sip_load_generator.py --host 74.207.251.126 --agents 20 \\
         --calls 100 --concurrency 10 --duration 5 --report-file report.json

  # Register-only test (no calls)
  python tests/sip_load_generator.py --agents 50 --calls 0 --register-only
""",
    )
    parser.add_argument("--host", default=DEFAULT_HOST,
                        help=f"SIP server host (default: {DEFAULT_HOST})")
    parser.add_argument("--port", type=int, default=DEFAULT_SIP_PORT,
                        help=f"SIP server port (default: {DEFAULT_SIP_PORT})")
    parser.add_argument("--external-ip", default=None,
                        help="External IP for SIP URIs (default: same as host)")
    parser.add_argument("--agents", type=int, default=10,
                        help="Number of virtual SIP agents (default: 10)")
    parser.add_argument("--base-extension", type=int, default=2000,
                        help="Starting extension number (default: 2000)")
    parser.add_argument("--base-port", type=int, default=19000,
                        help="Starting local UDP port (default: 19000)")
    parser.add_argument("--calls", type=int, default=20,
                        help="Total number of calls to make (default: 20)")
    parser.add_argument("--concurrency", type=int, default=5,
                        help="Max simultaneous calls (default: 5)")
    parser.add_argument("--duration", type=float, default=3.0,
                        help="Call hold duration in seconds (default: 3.0)")
    parser.add_argument("--no-rtp", action="store_true",
                        help="Skip RTP silence during calls")
    parser.add_argument("--register-only", action="store_true",
                        help="Only test registration (no calls)")
    parser.add_argument("--report-file", default=None,
                        help="Path for JSON report output")
    parser.add_argument("--inter-batch-delay", type=float, default=0.5,
                        help="Delay between call batches in seconds (default: 0.5)")
    parser.add_argument("-v", "--verbose", action="store_true",
                        help="Enable verbose logging")
    return parser.parse_args(argv)


async def _async_main(args):
    gen = SIPLoadGenerator(
        host=args.host,
        port=args.port,
        num_agents=args.agents,
        base_extension=args.base_extension,
        external_ip=args.external_ip,
        base_port=args.base_port,
    )

    if args.register_only:
        # Just test registration throughput
        gen._create_agents()
        gen._open_all()
        start = time.monotonic()
        ok, fail = await gen.register_all()
        elapsed = time.monotonic() - start
        print(f"\nRegistration test: {ok} OK, {fail} failed in {elapsed:.3f}s")
        print(f"  Rate: {ok / elapsed:.1f} registrations/sec" if elapsed > 0 else "")
        await gen.unregister_all()
        gen._close_all()
        return

    metrics = await gen.run_load_test(
        num_calls=args.calls,
        concurrency=args.concurrency,
        duration_secs=args.duration,
        send_rtp=not args.no_rtp,
        inter_batch_delay=args.inter_batch_delay,
    )

    gen.print_report()

    if args.report_file:
        gen.write_report(args.report_file)


def main(argv=None):
    args = parse_args(argv)

    level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )

    asyncio.run(_async_main(args))


if __name__ == "__main__":
    main()
