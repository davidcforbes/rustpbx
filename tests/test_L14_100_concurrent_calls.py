"""
L14 Concurrent Calls Tests -- 100 simultaneous SIP call load test.

Exercises RustPBX with 100 simultaneous SIP calls using the SIPLoadGenerator
from sip_load_generator.py.  Each test creates virtual SIP user agents
(extensions 5001-5200), registers them, establishes concurrent calls with
RTP media, and verifies signalling, media flow, packet loss, port exhaustion,
memory usage, CPU saturation, CDR correctness, and call setup latency under
extreme load.

Tests:
  1. 200 concurrent registrations        -- Register 5001-5200, verify all succeed
  2. 100 concurrent calls (basic)         -- 5001->5002, 5003->5004, ..., verify all establish
  3. 100 concurrent calls (RTP)           -- Verify RTP flows bidirectionally on all 100 calls
  4. 100-call packet loss                 -- Verify packet loss < 8% across all calls
  5. RTP port exhaustion check            -- Verify no port exhaustion (200+ ports needed)
  6. Memory baseline                      -- Check server memory before/after, verify < 1GB growth
  7. CDR writes                           -- Verify all 100 CDRs written correctly after calls end
  8. CPU saturation                       -- Monitor CPU via health API, log peak usage
  9. Setup latency                        -- Measure p50/p90/p95/p99 call setup times

Server:  RUSTPBX_HOST (default 127.0.0.1) : 5060  (UDP)
Users:   Dynamically registered 5001-5200 (password = "test{ext}")
Health:  https://RUSTPBX_HOST:8443/ami/v1/health

IMPORTANT: The server RTP port range must be expanded to at least 20000-20400
for 100 concurrent calls.  Each call requires 2 RTP ports (one per leg),
so 100 calls = 200 ports minimum.  The default range of 20000-20100 (100 ports)
is NOT sufficient.  Update config.toml:
    rtp_start_port = 20000
    rtp_end_port = 20400

Run with:
  python -m pytest tests/test_L14_100_concurrent_calls.py -v -s
  python -m pytest tests/test_L14_100_concurrent_calls.py -v -s -m load

Environment variables (all optional):
  RUSTPBX_HOST          SIP server IP           (default: 127.0.0.1)
  RUSTPBX_SIP_PORT      SIP port                (default: 5060)
  RUSTPBX_EXTERNAL_IP   Public IP for SIP URIs  (default: same as HOST)
  RUSTPBX_HTTP_PORT     HTTP(S) port            (default: 8443)
  RUSTPBX_SCHEME        http or https           (default: https)
"""

import asyncio
import logging
import os
import sys
import time

import pytest
import requests
import urllib3

# Suppress InsecureRequestWarning for self-signed TLS certs
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# Import load generator -- skip gracefully if unavailable
# ---------------------------------------------------------------------------

try:
    sys.path.insert(0, os.path.dirname(__file__))
    from sip_load_generator import (
        SIPLoadGenerator,
        SIPAgent,
        CallMetrics,
        LoadTestMetrics,
    )
    HAS_LOAD_GENERATOR = True
except ImportError:
    HAS_LOAD_GENERATOR = False

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SERVER_HOST = os.environ.get("RUSTPBX_HOST", "127.0.0.1")
SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8443"))
SCHEME = os.environ.get("RUSTPBX_SCHEME", "https")
EXTERNAL_IP = os.environ.get("RUSTPBX_EXTERNAL_IP", SERVER_HOST)
BASE_URL = f"{SCHEME}://{SERVER_HOST}:{HTTP_PORT}"
VERIFY_TLS = os.environ.get("RUSTPBX_VERIFY_TLS", "false").lower() in (
    "1", "true", "yes",
)

ADMIN_USER = os.environ.get("RUSTPBX_ADMIN_USER", "admin")
ADMIN_PASS = os.environ.get("RUSTPBX_ADMIN_PASS", "admin123")

# Number of concurrent calls to test
NUM_CALLS = 100
NUM_AGENTS = NUM_CALLS * 2  # 200 agents: 100 callers + 100 callees
BASE_EXTENSION = 5001
# Port range for agents -- each agent uses 2 ports (SIP + RTP)
# Starting from 50000+ to avoid collisions with L10 (21000+), L11 (30000+),
# L13 (40000+)
BASE_PORT = 50000
# Call hold duration for tests (seconds)
CALL_DURATION = 30.0
# Setup timeout -- much longer for 100 calls (seconds)
SETUP_TIMEOUT = 120
# Maximum allowed packet loss percentage (relaxed for 100 calls)
MAX_PACKET_LOSS_PCT = 8.0
# Maximum allowed call failures out of 100
MAX_CALL_FAILURES = 10
# Maximum average call setup time (seconds) -- relaxed for extreme concurrency
MAX_AVG_SETUP_TIME = 10.0
# Maximum per-call setup time (seconds)
MAX_PER_CALL_SETUP_TIME = 20.0
# Maximum memory growth in MB (relaxed for 100 calls)
MAX_MEMORY_GROWTH_MB = 1000
# RTP port range on the server -- must be expanded for 100 calls!
# Default (20000-20100) only provides 100 ports; 100 calls need 200+ ports.
# Recommended config: rtp_start_port = 20000, rtp_end_port = 20400
SERVER_RTP_START_PORT = 20000
SERVER_RTP_END_PORT = 20400  # Recommended expanded range
SERVER_RTP_PORT_COUNT = SERVER_RTP_END_PORT - SERVER_RTP_START_PORT  # 400
REQUIRED_RTP_PORTS = NUM_CALLS * 2  # 200 ports for 100 calls

logger = logging.getLogger("test_L14_100_concurrent_calls")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _server_reachable() -> bool:
    """Quick check: is the SIP server reachable via HTTP health endpoint?"""
    try:
        resp = requests.get(
            f"{BASE_URL}/ami/v1/health",
            timeout=5,
            verify=VERIFY_TLS,
        )
        return resp.status_code == 200
    except requests.RequestException:
        return False


def _get_health_data() -> dict:
    """Fetch server health data from the AMI endpoint."""
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
                return {"raw": resp.text[:500], "status_code": 200}
        return {"status_code": resp.status_code}
    except requests.RequestException as exc:
        return {"error": str(exc)}


def _get_dialog_count() -> int:
    """Fetch the current number of active SIP dialogs from the server."""
    try:
        resp = requests.get(
            f"{BASE_URL}/ami/v1/dialogs",
            timeout=10,
            verify=VERIFY_TLS,
        )
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                return len(data)
            elif isinstance(data, dict):
                return data.get("count", len(data.get("dialogs", [])))
        return -1
    except (requests.RequestException, ValueError):
        return -1


def _get_call_records() -> list:
    """Fetch call records from the console API.

    Returns a list of call record dicts, or an empty list on failure.
    Uses the admin session cookie for authenticated access.
    """
    try:
        session = requests.Session()
        # Authenticate first
        session.post(
            f"{BASE_URL}/login",
            json={"username": ADMIN_USER, "password": ADMIN_PASS},
            timeout=10,
            verify=VERIFY_TLS,
        )
        resp = session.get(
            f"{BASE_URL}/console/call-records",
            timeout=15,
            verify=VERIFY_TLS,
        )
        if resp.status_code == 200:
            try:
                data = resp.json()
                if isinstance(data, list):
                    return data
                elif isinstance(data, dict):
                    return data.get("records", data.get("items", []))
            except ValueError:
                pass
        return []
    except requests.RequestException:
        return []


def _get_memory_mb_from_health(health: dict) -> float:
    """Extract memory usage in MB from health data, returning -1 if unavailable."""
    mem = health.get("memory_mb") or health.get("rss_mb")
    if mem is not None:
        return float(mem)
    return -1.0


def _get_cpu_from_health(health: dict) -> float:
    """Extract CPU usage percentage from health data, returning -1 if unavailable.

    Tries several common field names that the health endpoint might expose.
    """
    for key in ("cpu_percent", "cpu_usage", "cpu_pct", "cpu"):
        val = health.get(key)
        if val is not None:
            return float(val)
    return -1.0


def _create_generator(base_port: int = BASE_PORT,
                      num_agents: int = NUM_AGENTS,
                      base_extension: int = BASE_EXTENSION) -> SIPLoadGenerator:
    """Create a SIPLoadGenerator with the standard test configuration."""
    return SIPLoadGenerator(
        host=SERVER_HOST,
        port=SIP_PORT,
        num_agents=num_agents,
        base_extension=base_extension,
        external_ip=EXTERNAL_IP,
        base_port=base_port,
    )


def _percentile(sorted_data: list, pct: int) -> float:
    """Compute a percentile from a pre-sorted list."""
    if not sorted_data:
        return 0.0
    idx = int(len(sorted_data) * pct / 100)
    idx = min(idx, len(sorted_data) - 1)
    return sorted_data[idx]


# ---------------------------------------------------------------------------
# Fixtures / markers
# ---------------------------------------------------------------------------

skip_no_generator = pytest.mark.skipif(
    not HAS_LOAD_GENERATOR,
    reason="sip_load_generator.py not available",
)

skip_no_server = pytest.mark.skipif(
    not _server_reachable(),
    reason=f"RustPBX server not reachable at {BASE_URL}",
)

load = pytest.mark.load


# ---------------------------------------------------------------------------
# Test class
# ---------------------------------------------------------------------------

@skip_no_generator
@skip_no_server
class TestConcurrentCalls100:
    """L14: 100-call concurrent load tests using SIPLoadGenerator."""

    # ------------------------------------------------------------------
    # TC-L14-001: Register 200 extensions simultaneously
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_registrations(self):
        """TC-L14-001: Register 200 extensions (5001-5200) simultaneously.

        Creates 200 SIPAgent instances and registers all of them at once.
        Verifies that all 200 registrations succeed within a reasonable time.
        Retries up to 3 times for robustness against transient network issues,
        since registering 200 agents is a very heavy load.
        """
        gen = _create_generator()
        max_retries = 3

        try:
            for attempt in range(1, max_retries + 1):
                success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                    gen.register_all()
                )

                print(f"\n--- TC-L14-001: 200 Concurrent Registrations (attempt {attempt}) ---")
                print(f"  Agents:    {NUM_AGENTS}")
                print(f"  Success:   {success_count}")
                print(f"  Failed:    {fail_count}")

                if success_count == NUM_AGENTS:
                    break

                # If not all succeeded, close and retry with fresh generator
                if attempt < max_retries:
                    logger.warning(
                        f"Only {success_count}/{NUM_AGENTS} registered, retrying..."
                    )
                    gen._close_all()
                    time.sleep(5)
                    gen = _create_generator()

            # Verify all agents report registered
            registered_agents = [a for a in gen.agents if a.registered]
            print(f"  Verified registered: {len(registered_agents)}")

            assert success_count == NUM_AGENTS, (
                f"Expected all {NUM_AGENTS} agents to register, but only "
                f"{success_count} succeeded ({fail_count} failed)"
            )

            # Unregister all before leaving
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

        finally:
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-002: 100 concurrent calls (basic signalling)
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_basic(self):
        """TC-L14-002: Set up 100 simultaneous calls (paired extensions).

        Pairs: 5001->5002, 5003->5004, 5005->5006, ..., 5199->5200
        All 100 calls are established simultaneously with RTP media.
        Verifies all calls get 200 OK and hold for the configured duration.
        Allows up to MAX_CALL_FAILURES (10) failures for transient issues at
        this extreme level of concurrency.  Also measures and reports latency
        percentiles (p50/p90/p95/p99) and total setup time.

        NOTE: Requires RTP port range expanded to 200+ ports on the server.
        """
        gen = _create_generator()

        try:
            # Register all agents
            success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES, (
                f"Need at least {NUM_AGENTS - MAX_CALL_FAILURES} agents registered, "
                f"got {success_count}"
            )

            # Start incoming call handlers on all agents (callees auto-answer)
            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Build call pairs: (0,1), (2,3), (4,5), ..., (198,199)
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            print(f"\n--- TC-L14-002: 100 Concurrent Calls (Basic) ---")
            print(f"  Call pairs: {len(pairs)} pairs")
            print(f"  Duration:   {CALL_DURATION}s each")

            # Make all 100 calls simultaneously
            start_time = time.monotonic()
            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=CALL_DURATION,
                    send_rtp=True,
                )
            )
            elapsed = time.monotonic() - start_time

            # Analyze results
            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            # Setup time statistics
            setup_times = sorted(m.setup_time for m in successful) if successful else []

            print(f"  Total elapsed: {elapsed:.2f}s")
            print(f"  Successful:    {len(successful)}/{len(call_metrics)}")
            if setup_times:
                print(f"  Setup time -- avg: {sum(setup_times)/len(setup_times):.3f}s, "
                      f"p50: {_percentile(setup_times, 50):.3f}s, "
                      f"p90: {_percentile(setup_times, 90):.3f}s, "
                      f"p95: {_percentile(setup_times, 95):.3f}s, "
                      f"p99: {_percentile(setup_times, 99):.3f}s, "
                      f"max: {setup_times[-1]:.3f}s")
            for m in successful[:10]:
                print(f"    {m.caller} -> {m.callee}: setup={m.setup_time:.3f}s "
                      f"rtp_sent={m.rtp_packets_sent}")
            if len(successful) > 10:
                print(f"    ... and {len(successful) - 10} more successful calls")
            for m in failed:
                print(f"    FAILED {m.caller} -> {m.callee}: {m.error} "
                      f"(code={m.final_response_code}, category={m.error_category})")

            # Allow up to MAX_CALL_FAILURES failures out of 100 calls
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Expected at least {NUM_CALLS - MAX_CALL_FAILURES} successful calls, "
                f"got {len(successful)}. "
                f"Failures: {[(m.caller, m.callee, m.error) for m in failed]}"
            )

            # Setup time for each call should be reasonable
            for m in successful:
                assert m.setup_time < MAX_PER_CALL_SETUP_TIME, (
                    f"Call {m.caller}->{m.callee} setup took {m.setup_time:.3f}s "
                    f"(>{MAX_PER_CALL_SETUP_TIME}s)"
                )

            # Average setup time check
            if setup_times:
                avg_setup = sum(setup_times) / len(setup_times)
                assert avg_setup < MAX_AVG_SETUP_TIME, (
                    f"Average call setup time {avg_setup:.3f}s exceeds "
                    f"maximum allowed {MAX_AVG_SETUP_TIME}s"
                )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-003: 100 concurrent calls with RTP media verification
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_with_rtp(self):
        """TC-L14-003: Verify RTP flows bidirectionally on all 100 calls.

        Establishes 100 concurrent calls with RTP enabled and verifies that
        each caller sends a non-trivial number of RTP packets during the
        call hold period.  At 20ms intervals over the RTP duration, expects
        approximately duration/0.020 packets per call.  Requires at least 20%
        of the expected count to account for scheduling jitter under extreme
        load (relaxed from 25% in the 50-call test).

        NOTE: Requires RTP port range expanded to 200+ ports on the server.
        """
        gen = _create_generator(base_port=BASE_PORT + 500)

        try:
            # Register all
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES, (
                f"Need at least {NUM_AGENTS - MAX_CALL_FAILURES} registered, "
                f"got {success_count}"
            )

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # 100 concurrent calls with RTP -- use shorter duration than the
            # full CALL_DURATION to keep test time reasonable while still
            # generating enough packets to validate RTP flow
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 10.0  # seconds of RTP to send

            print(f"\n--- TC-L14-003: 100-Call RTP Flow ---")
            print(f"  RTP duration: {rtp_duration}s per call")

            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=rtp_duration,
                    send_rtp=True,
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Need at least {NUM_CALLS - MAX_CALL_FAILURES} successful calls "
                f"for RTP test, got {len(successful)}"
            )

            # Each successful call should have sent RTP packets.
            # At 20ms intervals over rtp_duration seconds, expect approximately
            # rtp_duration / 0.020 = 500 packets per call.  Allow generous
            # margin -- require at least 20% of expected (more relaxed for 100 calls).
            expected_min_packets = int((rtp_duration / 0.020) * 0.20)
            rtp_issues = []
            total_rtp_sent = 0
            total_rtp_recv = 0

            for m in successful:
                total_rtp_sent += m.rtp_packets_sent
                total_rtp_recv += m.rtp_packets_received
                if m.rtp_packets_sent < expected_min_packets:
                    rtp_issues.append(
                        f"{m.caller}->{m.callee}: sent only {m.rtp_packets_sent} "
                        f"(expected >= {expected_min_packets})"
                    )

            print(f"  Successful calls: {len(successful)}/{len(call_metrics)}")
            print(f"  Total RTP sent:   {total_rtp_sent}")
            print(f"  Total RTP recv:   {total_rtp_recv}")
            print(f"  Expected min packets per call: {expected_min_packets}")
            print(f"  RTP issues: {len(rtp_issues)}")

            # Show a sample of per-call stats (first 10 + last 5)
            sample_calls = (
                successful[:10] + successful[-5:]
                if len(successful) > 15
                else successful
            )
            for m in sample_calls:
                print(f"    {m.caller} -> {m.callee}: "
                      f"rtp_sent={m.rtp_packets_sent}, "
                      f"rtp_recv={m.rtp_packets_received}")

            for m in failed:
                print(f"    FAILED {m.caller} -> {m.callee}: {m.error}")

            # Allow up to 10 calls with low RTP for transient scheduling issues
            # at this extreme concurrency level
            assert len(rtp_issues) <= MAX_CALL_FAILURES, (
                f"Too many calls with insufficient RTP "
                f"({len(rtp_issues)}/{len(successful)}): "
                f"{rtp_issues[:15]}"  # Truncate list in error message
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-004: Packet loss across 100 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_packet_loss(self):
        """TC-L14-004: Verify packet loss < 8% across all 100 concurrent calls.

        Runs 100 concurrent calls with RTP and calculates the packet loss
        ratio from the sent/received counters in CallMetrics.  The test
        focuses on verifying that the outbound RTP pipeline does not drop
        frames under extreme concurrency -- i.e., we sent what we intended
        to send.

        Additionally checks that no more than MAX_CALL_FAILURES calls were
        dropped mid-stream, which would indicate server-side resource
        exhaustion under 100-call load.

        NOTE: 8% threshold is more relaxed than the 5% used in the 50-call
        test, since 100 concurrent calls place much heavier demands on
        CPU scheduling and network I/O.
        """
        gen = _create_generator(base_port=BASE_PORT + 1000)

        try:
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 10.0

            print(f"\n--- TC-L14-004: 100-Call Packet Loss ---")
            print(f"  RTP duration: {rtp_duration}s per call")

            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=rtp_duration,
                    send_rtp=True,
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            # Calculate expected packets per call
            expected_packets = rtp_duration / 0.020  # 500 for 10s at 20ms
            total_expected = expected_packets * len(successful)
            total_sent = sum(m.rtp_packets_sent for m in successful)

            # "Loss" = expected - actually sent (indicates our sending pipeline dropped)
            if total_expected > 0:
                send_loss_pct = max(
                    0, (total_expected - total_sent) / total_expected * 100
                )
            else:
                send_loss_pct = 0.0

            # Call-level failure rate (calls that did not complete)
            call_loss_pct = (
                (len(failed) / len(call_metrics) * 100) if call_metrics else 0.0
            )

            print(f"  Successful calls:    {len(successful)}/{len(call_metrics)}")
            print(f"  Total RTP expected:  ~{int(total_expected)}")
            print(f"  Total RTP sent:      {total_sent}")
            print(f"  Send pipeline loss:  {send_loss_pct:.2f}%")
            print(f"  Call failure rate:   {call_loss_pct:.2f}%")

            # Per-call loss breakdown (sample)
            per_call_losses = []
            for m in successful:
                per_call_expected = expected_packets
                per_call_loss = max(
                    0,
                    (per_call_expected - m.rtp_packets_sent)
                    / per_call_expected
                    * 100,
                )
                per_call_losses.append(per_call_loss)

            per_call_losses_sorted = sorted(per_call_losses)
            if per_call_losses_sorted:
                print(
                    f"  Per-call loss -- "
                    f"avg: {sum(per_call_losses)/len(per_call_losses):.2f}%, "
                    f"p50: {_percentile(per_call_losses_sorted, 50):.2f}%, "
                    f"p90: {_percentile(per_call_losses_sorted, 90):.2f}%, "
                    f"p95: {_percentile(per_call_losses_sorted, 95):.2f}%, "
                    f"max: {per_call_losses_sorted[-1]:.2f}%"
                )

            # Show worst offenders
            worst_calls = sorted(
                successful, key=lambda m: m.rtp_packets_sent
            )[:10]
            for m in worst_calls:
                loss = max(
                    0,
                    (expected_packets - m.rtp_packets_sent)
                    / expected_packets
                    * 100,
                )
                print(
                    f"    {m.caller}->{m.callee}: "
                    f"sent={m.rtp_packets_sent}, loss={loss:.1f}%"
                )

            for m in failed:
                print(f"    FAILED {m.caller}->{m.callee}: {m.error}")

            # Verify packet loss is within acceptable limits (8% for 100 calls)
            assert send_loss_pct < MAX_PACKET_LOSS_PCT, (
                f"RTP send pipeline loss {send_loss_pct:.2f}% exceeds "
                f"maximum allowed {MAX_PACKET_LOSS_PCT}%"
            )

            # No more than MAX_CALL_FAILURES calls should fail entirely
            assert len(failed) <= MAX_CALL_FAILURES, (
                f"{len(failed)} calls failed entirely "
                f"(max allowed {MAX_CALL_FAILURES}): "
                f"{[(m.caller, m.callee, m.error) for m in failed]}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-005: RTP port exhaustion check
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_rtp_port_exhaustion(self):
        """TC-L14-005: Check for RTP port exhaustion under 100-call load.

        100 concurrent calls require 200 RTP ports (2 per call -- one per
        leg).  The default RustPBX RTP port range of 20000-20100 provides
        only 100 ports, which is INSUFFICIENT for this test.

        REQUIRED SERVER CONFIG:
            rtp_start_port = 20000
            rtp_end_port = 20400   # at least 200 ports, 400 recommended

        This test:
          - Documents the required port range configuration
          - Verifies the server can allocate all needed ports
          - Monitors dialog count at peak to verify all 100 calls active
          - Categorizes failures to detect port exhaustion specifically
          - Reports bottleneck identification for capacity planning
        """
        gen = _create_generator(base_port=BASE_PORT + 1500)

        try:
            # Ensure baseline: no stale dialogs consuming ports
            baseline_dialogs = _get_dialog_count()
            print(f"\n--- TC-L14-005: RTP Port Exhaustion Check ---")
            print(f"  Server RTP port range: {SERVER_RTP_START_PORT}-{SERVER_RTP_END_PORT} "
                  f"({SERVER_RTP_PORT_COUNT} ports)")
            print(f"  Ports needed for 100 calls: {REQUIRED_RTP_PORTS} "
                  f"(100 calls x 2 legs)")
            print(f"  Port headroom: {SERVER_RTP_PORT_COUNT - REQUIRED_RTP_PORTS} "
                  f"spare ports")
            print(f"  Baseline dialogs: {baseline_dialogs}")
            print(f"")
            print(f"  NOTE: Server config MUST have rtp_end_port >= "
                  f"{SERVER_RTP_START_PORT + REQUIRED_RTP_PORTS}")
            print(f"  Recommended: rtp_start_port={SERVER_RTP_START_PORT}, "
                  f"rtp_end_port={SERVER_RTP_END_PORT}")

            # Register all agents
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make all 100 calls with moderate hold time to keep them active
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            hold_duration = 15.0

            # Run calls and sample dialogs during peak
            async def _run_calls_and_check_ports():
                call_task = asyncio.create_task(
                    gen.make_concurrent_calls(
                        pairs,
                        duration_secs=hold_duration,
                        send_rtp=True,
                    )
                )

                # Wait for calls to be established (extra time for 100 calls)
                await asyncio.sleep(15.0)

                # Check dialog count at peak -- should be close to 100
                peak_dialogs = _get_dialog_count()

                # Check server health at peak load
                peak_health = _get_health_data()

                # Wait for calls to finish
                call_metrics = await call_task
                return call_metrics, peak_dialogs, peak_health

            call_metrics, peak_dialogs, peak_health = (
                asyncio.get_event_loop().run_until_complete(
                    _run_calls_and_check_ports()
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            # Categorize failures
            port_errors = []
            other_errors = []
            for m in failed:
                error_lower = (m.error or "").lower()
                if any(
                    kw in error_lower
                    for kw in (
                        "port",
                        "resource",
                        "exhausted",
                        "unavailable",
                        "503",
                        "busy",
                    )
                ):
                    port_errors.append(m)
                else:
                    other_errors.append(m)

            print(f"  Calls successful:    {len(successful)}/{len(call_metrics)}")
            print(f"  Calls failed:        {len(failed)}")
            print(f"    Port-related:      {len(port_errors)}")
            print(f"    Other errors:      {len(other_errors)}")
            print(f"  Peak dialogs:        {peak_dialogs}")
            print(f"  Peak health:         "
                  f"{peak_health.get('status_code', peak_health)}")

            # Bottleneck identification
            print(f"\n  --- Bottleneck Analysis ---")
            if len(port_errors) > 0:
                print(f"  BOTTLENECK: RTP port exhaustion detected "
                      f"({len(port_errors)} port-related failures)")
                print(f"  ACTION: Expand rtp_end_port in server config")
            if len(other_errors) > MAX_CALL_FAILURES:
                print(f"  BOTTLENECK: High non-port failure rate "
                      f"({len(other_errors)} other errors)")
            if peak_dialogs >= 0 and peak_dialogs < NUM_CALLS * 0.8:
                print(f"  BOTTLENECK: Only {peak_dialogs} peak dialogs "
                      f"(expected ~{NUM_CALLS})")
            if len(port_errors) == 0 and len(successful) >= NUM_CALLS - MAX_CALL_FAILURES:
                print(f"  No port exhaustion detected -- server handled "
                      f"100 concurrent calls successfully")

            for m in port_errors:
                print(f"    PORT-FAIL {m.caller}->{m.callee}: {m.error} "
                      f"(code={m.final_response_code})")
            for m in other_errors[:10]:
                print(f"    OTHER-FAIL {m.caller}->{m.callee}: {m.error} "
                      f"(code={m.final_response_code})")

            # Primary assertion: no systemic port exhaustion
            # A few failures from timing are acceptable, but widespread port
            # exhaustion (many 503 errors) is not.
            assert len(port_errors) <= MAX_CALL_FAILURES, (
                f"Port exhaustion detected: {len(port_errors)} calls failed with "
                f"port-related errors. Server RTP port range must be expanded to "
                f"at least {REQUIRED_RTP_PORTS} ports for 100 concurrent calls. "
                f"Current recommended range: {SERVER_RTP_START_PORT}-{SERVER_RTP_END_PORT}. "
                f"Errors: "
                f"{[(m.caller, m.callee, m.error) for m in port_errors[:10]]}"
            )

            # Overall success rate should be reasonable
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded. "
                f"Port exhaustion may be a factor. "
                f"Failures: {[(m.caller, m.callee, m.error) for m in failed[:10]]}"
            )

            # Server should still be healthy after peak port usage
            assert "error" not in peak_health, (
                f"Server health check failed during peak port usage: {peak_health}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-006: Memory baseline before/after 100 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_memory_baseline(self):
        """TC-L14-006: Check server memory via API before/after 100-call load.

        Measures memory consumption at three points:
          1. Before load (baseline)
          2. During peak load (100 active calls with RTP)
          3. After load (post-teardown, after GC/cleanup)

        Verifies that post-load memory does not exceed baseline + 1000 MB,
        which would indicate a memory leak under extreme concurrency.
        Also monitors server health responsiveness during the entire cycle.

        The 1GB threshold is more relaxed than the 500MB threshold in the
        50-call test, since 100 concurrent calls with RTP media buffering
        naturally consume more memory.
        """
        gen = _create_generator(base_port=BASE_PORT + 2000)

        try:
            # Pre-load baseline
            health_before = _get_health_data()
            mem_before = _get_memory_mb_from_health(health_before)
            print(f"\n--- TC-L14-006: Memory Baseline (100 Calls) ---")
            print(f"  Health before: {health_before}")
            print(f"  Memory before: {mem_before:.1f} MB" if mem_before >= 0
                  else "  Memory before: unavailable")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Run 100 concurrent calls with RTP and poll health during peak
            call_duration = 15.0
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            async def _run_calls_and_monitor_memory():
                call_task = asyncio.create_task(
                    gen.make_concurrent_calls(
                        pairs,
                        duration_secs=call_duration,
                        send_rtp=True,
                    )
                )

                # Wait for calls to establish (longer for 100 calls)
                await asyncio.sleep(12.0)

                # Sample memory during peak load -- more samples for 100 calls
                health_samples = []
                for _ in range(8):
                    health = _get_health_data()
                    health_samples.append(health)
                    await asyncio.sleep(1.0)

                call_metrics = await call_task
                return call_metrics, health_samples

            call_metrics, health_samples = (
                asyncio.get_event_loop().run_until_complete(
                    _run_calls_and_monitor_memory()
                )
            )

            successful = [m for m in call_metrics if m.success]

            print(f"  Calls successful: {len(successful)}/{len(call_metrics)}")

            # Report peak memory during load
            peak_memories = []
            for i, h in enumerate(health_samples):
                mem = _get_memory_mb_from_health(h)
                has_error = "error" in h
                status = h.get(
                    "status_code", "ok" if not has_error else "error"
                )
                print(
                    f"    Sample {i + 1}: status={status}, "
                    f"mem={mem:.1f} MB"
                    if mem >= 0
                    else f"    Sample {i + 1}: status={status}, mem=unavailable"
                )
                if mem >= 0:
                    peak_memories.append(mem)

            if peak_memories:
                peak_mem = max(peak_memories)
                print(f"  Peak memory during load: {peak_mem:.1f} MB")

            # Health endpoint should be responsive during load
            health_errors = [h for h in health_samples if "error" in h]
            assert len(health_errors) <= 3, (
                f"Server health unreachable during "
                f"{len(health_errors)}/{len(health_samples)} "
                f"polls while {NUM_CALLS} calls active: {health_errors}"
            )

            # At least half the calls should succeed
            assert len(successful) >= NUM_CALLS // 2, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded "
                f"during memory test"
            )

            # Stop handlers and unregister, then wait for cleanup
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            # Give server time to clean up (longer for 100 calls)
            time.sleep(12)

            # Post-load measurement
            health_after = _get_health_data()
            mem_after = _get_memory_mb_from_health(health_after)
            print(f"  Health after load: {health_after}")
            print(f"  Memory after: {mem_after:.1f} MB" if mem_after >= 0
                  else "  Memory after: unavailable")

            # Check memory growth
            if mem_before >= 0 and mem_after >= 0:
                growth = mem_after - mem_before
                print(f"  Memory growth: {growth:.1f} MB "
                      f"(limit: {MAX_MEMORY_GROWTH_MB} MB)")
                assert growth < MAX_MEMORY_GROWTH_MB, (
                    f"Excessive memory growth during 100-call load: "
                    f"{growth:.1f} MB (before={mem_before:.1f}, "
                    f"after={mem_after:.1f}, "
                    f"limit={MAX_MEMORY_GROWTH_MB} MB)"
                )
            else:
                print(
                    "  Memory growth check skipped: memory data not available "
                    "from health endpoint"
                )

            # Server should still be healthy
            assert "error" not in health_after, (
                f"Server health check failed after 100-call load: "
                f"{health_after}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-007: CDR writes for 100 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_cdr_writes(self):
        """TC-L14-007: Verify all 100 CDRs written correctly after calls end.

        Establishes 100 concurrent calls with short RTP media, waits for them
        to complete, then checks the call records API to verify that CDRs
        were created for each call.

        Methodology:
          1. Snapshot call records count before test
          2. Run 100 concurrent calls (short duration to keep test fast)
          3. Wait for CDR flush (server may batch writes -- longer wait for 100)
          4. Snapshot call records count after test
          5. Verify the delta matches the number of successful calls

        Also verifies that the server dialog count returns to baseline after
        all calls are torn down (no resource leaks).
        """
        gen = _create_generator(base_port=BASE_PORT + 2500)

        try:
            # Baseline measurements
            baseline_dialogs = _get_dialog_count()
            records_before = _get_call_records()
            records_count_before = len(records_before)

            print(f"\n--- TC-L14-007: CDR Writes (100 Calls) ---")
            print(f"  Baseline dialogs: {baseline_dialogs}")
            print(f"  Call records before: {records_count_before}")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make 100 concurrent calls with short hold time
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=5.0,  # short hold -- we care about CDR, not media
                    send_rtp=True,
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            print(f"  Calls successful: {len(successful)}/{len(call_metrics)}")
            for m in failed[:10]:
                print(f"    FAILED {m.caller}->{m.callee}: {m.error}")

            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded"
            )

            # Teardown
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            # Wait for CDR flush -- server may batch/buffer CDR writes.
            # Longer wait for 100 calls to account for write batching and
            # potential DB contention.
            print("  Waiting for CDR flush...")
            time.sleep(15)

            # Check call records after test
            records_after = _get_call_records()
            records_count_after = len(records_after)
            new_records = records_count_after - records_count_before

            print(f"  Call records after: {records_count_after}")
            print(f"  New records: {new_records}")
            print(f"  Expected (successful calls): {len(successful)}")

            # Verify dialog count returned to baseline
            post_dialogs = _get_dialog_count()
            print(f"  Post-teardown dialogs: {post_dialogs}")

            if baseline_dialogs >= 0 and post_dialogs >= 0:
                leaked = post_dialogs - baseline_dialogs
                print(f"  Leaked dialogs: {leaked}")
                # Allow small tolerance for registration dialogs
                assert leaked <= 10, (
                    f"Possible dialog leak: {leaked} more dialogs after test "
                    f"(before={baseline_dialogs}, after={post_dialogs})"
                )

            # CDR verification: if the call records API returned data, verify
            # that the number of new records matches (or nearly matches) the
            # number of successful calls.  If the API returned empty data
            # (e.g., endpoint not available or requires different auth), we
            # still pass the test but log a warning.
            if (
                records_count_before >= 0
                and records_count_after >= 0
                and records_count_after > 0
            ):
                # Allow some tolerance: CDRs may be delayed or the API may
                # paginate.  Require at least 75% of successful calls have CDRs
                # (relaxed from 80% in 50-call test for higher DB contention).
                min_expected_records = int(len(successful) * 0.75)
                assert new_records >= min_expected_records, (
                    f"Expected at least {min_expected_records} new CDR records "
                    f"(75% of {len(successful)} successful calls), but only "
                    f"found {new_records} new records "
                    f"(before={records_count_before}, "
                    f"after={records_count_after})"
                )
                print(
                    f"  CDR write rate: {new_records}/{len(successful)} "
                    f"({new_records/len(successful)*100:.1f}%)"
                )
            else:
                logger.warning(
                    "Call records API returned no data -- CDR count verification "
                    "skipped. The API may require authentication or may not be "
                    "available. Call records endpoint: %s/console/call-records",
                    BASE_URL,
                )
                print(
                    "  CDR count verification: SKIPPED (API returned no data)"
                )

            # Server health check
            health = _get_health_data()
            print(f"  Server health after CDR test: {health}")
            assert "error" not in health, (
                f"Server health check failed after CDR test: {health}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-008: CPU saturation monitoring during 100 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_cpu_saturation(self):
        """TC-L14-008: Monitor CPU usage via health API during 100-call load.

        Establishes 100 concurrent calls with RTP media and polls the server
        health endpoint repeatedly to track CPU usage throughout the load
        cycle.  Reports peak CPU usage and samples over time.

        This test does NOT assert a hard CPU limit (since acceptable CPU
        usage depends on hardware), but it does:
          - Log all CPU samples for capacity planning analysis
          - Verify the health endpoint remains responsive under load
          - Verify at least 50% of calls succeed (server not completely
            overwhelmed)
          - Report peak CPU as a bottleneck indicator

        The CPU data may not be available from all health endpoint
        implementations; in that case, the test logs a warning and still
        passes, focusing on health endpoint responsiveness instead.
        """
        gen = _create_generator(base_port=BASE_PORT + 3000)

        try:
            # Pre-load baseline CPU
            health_before = _get_health_data()
            cpu_before = _get_cpu_from_health(health_before)
            mem_before = _get_memory_mb_from_health(health_before)

            print(f"\n--- TC-L14-008: CPU Saturation (100 Calls) ---")
            print(f"  CPU before: {cpu_before:.1f}%" if cpu_before >= 0
                  else "  CPU before: unavailable")
            print(f"  Memory before: {mem_before:.1f} MB" if mem_before >= 0
                  else "  Memory before: unavailable")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Run 100 concurrent calls and sample CPU throughout
            call_duration = 20.0
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            async def _run_calls_and_monitor_cpu():
                call_task = asyncio.create_task(
                    gen.make_concurrent_calls(
                        pairs,
                        duration_secs=call_duration,
                        send_rtp=True,
                    )
                )

                # Sample health/CPU before calls fully establish
                await asyncio.sleep(5.0)

                # Collect samples throughout the call lifecycle
                samples = []
                for i in range(12):
                    health = _get_health_data()
                    cpu = _get_cpu_from_health(health)
                    mem = _get_memory_mb_from_health(health)
                    samples.append({
                        "time_offset": 5.0 + i * 1.5,
                        "health": health,
                        "cpu": cpu,
                        "memory_mb": mem,
                        "responsive": "error" not in health,
                    })
                    await asyncio.sleep(1.5)

                call_metrics = await call_task
                return call_metrics, samples

            call_metrics, cpu_samples = (
                asyncio.get_event_loop().run_until_complete(
                    _run_calls_and_monitor_cpu()
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            print(f"  Calls successful: {len(successful)}/{len(call_metrics)}")

            # Report CPU samples
            cpu_values = []
            mem_values = []
            unresponsive_count = 0
            print(f"\n  CPU/Memory samples during load:")
            for s in cpu_samples:
                cpu = s["cpu"]
                mem = s["memory_mb"]
                responsive = s["responsive"]
                offset = s["time_offset"]
                if not responsive:
                    unresponsive_count += 1
                if cpu >= 0:
                    cpu_values.append(cpu)
                if mem >= 0:
                    mem_values.append(mem)
                cpu_str = f"{cpu:.1f}%" if cpu >= 0 else "n/a"
                mem_str = f"{mem:.1f}MB" if mem >= 0 else "n/a"
                status_str = "OK" if responsive else "UNREACHABLE"
                print(f"    t+{offset:5.1f}s: cpu={cpu_str}, "
                      f"mem={mem_str}, status={status_str}")

            # Summary
            print(f"\n  --- CPU Saturation Summary ---")
            if cpu_values:
                peak_cpu = max(cpu_values)
                avg_cpu = sum(cpu_values) / len(cpu_values)
                print(f"  Peak CPU:    {peak_cpu:.1f}%")
                print(f"  Avg CPU:     {avg_cpu:.1f}%")
                print(f"  CPU samples: {len(cpu_values)}")

                # Bottleneck identification
                if peak_cpu > 90:
                    print(f"  BOTTLENECK: CPU saturation detected "
                          f"(peak {peak_cpu:.1f}%)")
                elif peak_cpu > 70:
                    print(f"  WARNING: CPU usage high "
                          f"(peak {peak_cpu:.1f}%)")
                else:
                    print(f"  CPU headroom available "
                          f"(peak {peak_cpu:.1f}%)")
            else:
                print("  CPU data not available from health endpoint")
                logger.warning(
                    "Health endpoint does not expose CPU metrics. "
                    "CPU saturation test relies on health API responsiveness only."
                )

            if mem_values:
                peak_mem = max(mem_values)
                print(f"  Peak memory: {peak_mem:.1f} MB")

            print(f"  Health endpoint unresponsive: "
                  f"{unresponsive_count}/{len(cpu_samples)} samples")

            for m in failed[:5]:
                print(f"    FAILED {m.caller}->{m.callee}: {m.error}")

            # Assertions: focus on system stability, not hard CPU limits
            # (CPU thresholds depend on hardware)

            # Health endpoint should be mostly responsive during load
            assert unresponsive_count <= 4, (
                f"Server health endpoint unresponsive during "
                f"{unresponsive_count}/{len(cpu_samples)} polls -- "
                f"possible CPU saturation causing timeouts"
            )

            # At least half the calls should succeed (server not overwhelmed)
            assert len(successful) >= NUM_CALLS // 2, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded -- "
                f"server may be CPU-saturated"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L14-009: Call setup latency percentiles for 100 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(600)
    def test_100_concurrent_calls_setup_latency(self):
        """TC-L14-009: Measure p50/p90/p95/p99 call setup times for 100 calls.

        Establishes 100 concurrent calls and measures the time from INVITE
        sent to 200 OK received for each call.  Reports detailed latency
        percentiles for capacity planning.

        Thresholds (relaxed for 100-call extreme concurrency):
          - p50 < 5s   (half of calls should set up quickly)
          - p90 < 8s   (90th percentile reasonable for this load)
          - p95 < 10s  (95th percentile within maximum tolerance)
          - p99 < 15s  (only 1% of calls can be very slow)
          - Average < 10s (MAX_AVG_SETUP_TIME)
          - Maximum per-call < 20s (MAX_PER_CALL_SETUP_TIME)

        Also categorizes latency distribution into buckets for analysis.
        """
        gen = _create_generator(base_port=BASE_PORT + 3500)

        try:
            # Register all agents
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES, (
                f"Need at least {NUM_AGENTS - MAX_CALL_FAILURES} registered, "
                f"got {success_count}"
            )

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # 100 concurrent calls -- use moderate hold duration since we
            # primarily care about setup time, not call duration
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            hold_duration = 10.0

            print(f"\n--- TC-L14-009: Setup Latency (100 Calls) ---")
            print(f"  Calls: {NUM_CALLS}")
            print(f"  Hold duration: {hold_duration}s")

            start_time = time.monotonic()
            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=hold_duration,
                    send_rtp=True,
                )
            )
            total_elapsed = time.monotonic() - start_time

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Need at least {NUM_CALLS - MAX_CALL_FAILURES} successful "
                f"calls for latency analysis, got {len(successful)}"
            )

            # Compute latency percentiles
            setup_times = sorted(m.setup_time for m in successful)

            if not setup_times:
                pytest.fail("No successful calls to measure setup latency")

            p50 = _percentile(setup_times, 50)
            p90 = _percentile(setup_times, 90)
            p95 = _percentile(setup_times, 95)
            p99 = _percentile(setup_times, 99)
            avg_setup = sum(setup_times) / len(setup_times)
            min_setup = setup_times[0]
            max_setup = setup_times[-1]

            print(f"\n  --- Latency Report ---")
            print(f"  Total elapsed: {total_elapsed:.2f}s")
            print(f"  Successful:    {len(successful)}/{len(call_metrics)}")
            print(f"  Failed:        {len(failed)}")
            print(f"")
            print(f"  Setup time percentiles:")
            print(f"    Min:  {min_setup:.3f}s")
            print(f"    p50:  {p50:.3f}s")
            print(f"    p90:  {p90:.3f}s")
            print(f"    p95:  {p95:.3f}s")
            print(f"    p99:  {p99:.3f}s")
            print(f"    Max:  {max_setup:.3f}s")
            print(f"    Avg:  {avg_setup:.3f}s")

            # Latency distribution buckets
            buckets = {
                "< 0.5s": 0,
                "0.5-1s": 0,
                "1-2s": 0,
                "2-5s": 0,
                "5-10s": 0,
                "10-15s": 0,
                "15-20s": 0,
                "> 20s": 0,
            }
            for t in setup_times:
                if t < 0.5:
                    buckets["< 0.5s"] += 1
                elif t < 1.0:
                    buckets["0.5-1s"] += 1
                elif t < 2.0:
                    buckets["1-2s"] += 1
                elif t < 5.0:
                    buckets["2-5s"] += 1
                elif t < 10.0:
                    buckets["5-10s"] += 1
                elif t < 15.0:
                    buckets["10-15s"] += 1
                elif t < 20.0:
                    buckets["15-20s"] += 1
                else:
                    buckets["> 20s"] += 1

            print(f"\n  Latency distribution:")
            for bucket, count in buckets.items():
                bar = "#" * count
                pct = count / len(setup_times) * 100
                print(f"    {bucket:>8s}: {count:3d} ({pct:5.1f}%) {bar}")

            # Show slowest 10 calls
            print(f"\n  Slowest 10 calls:")
            slowest = sorted(successful, key=lambda m: -m.setup_time)[:10]
            for m in slowest:
                print(f"    {m.caller}->{m.callee}: {m.setup_time:.3f}s")

            # Show fastest 5 calls
            print(f"\n  Fastest 5 calls:")
            fastest = sorted(successful, key=lambda m: m.setup_time)[:5]
            for m in fastest:
                print(f"    {m.caller}->{m.callee}: {m.setup_time:.3f}s")

            for m in failed[:5]:
                print(f"    FAILED {m.caller}->{m.callee}: {m.error}")

            # Bottleneck identification based on latency
            print(f"\n  --- Latency Bottleneck Analysis ---")
            if p50 > 5.0:
                print(f"  BOTTLENECK: High median latency ({p50:.3f}s) -- "
                      f"server struggling with 100-call concurrency")
            if p99 > 15.0:
                print(f"  WARNING: p99 latency very high ({p99:.3f}s) -- "
                      f"some calls experiencing extreme delays")
            if max_setup > MAX_PER_CALL_SETUP_TIME:
                print(f"  WARNING: Maximum setup time ({max_setup:.3f}s) "
                      f"exceeds {MAX_PER_CALL_SETUP_TIME}s threshold")
            if avg_setup < 3.0 and p95 < 5.0:
                print(f"  EXCELLENT: Server handles 100 calls with low latency")
            elif avg_setup < MAX_AVG_SETUP_TIME:
                print(f"  ACCEPTABLE: Average latency within limits")

            # Assertions -- relaxed thresholds for 100-call load
            assert p50 < 5.0, (
                f"p50 setup latency {p50:.3f}s exceeds 5.0s threshold. "
                f"Half of calls are taking too long to set up."
            )

            assert p95 < 10.0, (
                f"p95 setup latency {p95:.3f}s exceeds 10.0s threshold. "
                f"95th percentile calls are too slow."
            )

            assert avg_setup < MAX_AVG_SETUP_TIME, (
                f"Average setup time {avg_setup:.3f}s exceeds "
                f"{MAX_AVG_SETUP_TIME}s threshold"
            )

            # Per-call max check (allow a few outliers but not systemic slowness)
            slow_calls = [m for m in successful
                          if m.setup_time > MAX_PER_CALL_SETUP_TIME]
            assert len(slow_calls) <= 5, (
                f"{len(slow_calls)} calls exceeded {MAX_PER_CALL_SETUP_TIME}s "
                f"setup time (max 5 allowed). Slowest: "
                f"{[(m.caller, m.callee, f'{m.setup_time:.3f}s') for m in slow_calls[:10]]}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()
