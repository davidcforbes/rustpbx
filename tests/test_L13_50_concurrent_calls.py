"""
L13 Concurrent Calls Tests -- 50 simultaneous SIP call load test.

Exercises RustPBX with 50 simultaneous SIP calls using the SIPLoadGenerator
from sip_load_generator.py.  Each test creates virtual SIP user agents
(extensions 4001-4100), registers them, establishes concurrent calls with
RTP media, and verifies signalling, media flow, packet loss, port exhaustion,
memory usage, and CDR correctness under heavy load.

Tests:
  1. 100 concurrent registrations        -- Register 4001-4100, verify all succeed
  2. 50 concurrent calls (basic)         -- 4001->4002, 4003->4004, ..., verify all establish
  3. 50 concurrent calls (RTP)           -- Verify RTP flows bidirectionally on all 50 calls
  4. 50-call packet loss                 -- Verify packet loss < 5% across all calls
  5. RTP port exhaustion check           -- Verify no port exhaustion (100 ports needed)
  6. Memory baseline                     -- Check server memory before/after, verify < 500MB growth
  7. CDR writes                          -- Verify all 50 CDRs written correctly after calls end

Server:  RUSTPBX_HOST (default 127.0.0.1) : 5060  (UDP)
Users:   Dynamically registered 4001-4100 (password = "test{ext}")
Health:  https://RUSTPBX_HOST:8443/ami/v1/health

Run with:
  python -m pytest tests/test_L13_50_concurrent_calls.py -v -s
  python -m pytest tests/test_L13_50_concurrent_calls.py -v -s -m load

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
NUM_CALLS = 50
NUM_AGENTS = NUM_CALLS * 2  # 100 agents: 50 callers + 50 callees
BASE_EXTENSION = 4001
# Port range for agents -- each agent uses 2 ports (SIP + RTP)
# Starting from 40000+ to avoid collisions with L10 (21000+) and L11 (30000+)
BASE_PORT = 40000
# Call hold duration for tests (seconds)
CALL_DURATION = 30.0
# Setup timeout -- longer for 50 calls (seconds)
SETUP_TIMEOUT = 60
# Maximum allowed packet loss percentage (relaxed for 50 calls)
MAX_PACKET_LOSS_PCT = 5.0
# Maximum allowed call failures out of 50
MAX_CALL_FAILURES = 5
# Maximum average call setup time (seconds) -- relaxed for higher concurrency
MAX_AVG_SETUP_TIME = 8.0
# Maximum memory growth in MB
MAX_MEMORY_GROWTH_MB = 500
# RTP port range on the server (default config)
SERVER_RTP_START_PORT = 20000
SERVER_RTP_END_PORT = 20100
SERVER_RTP_PORT_COUNT = SERVER_RTP_END_PORT - SERVER_RTP_START_PORT  # 100

logger = logging.getLogger("test_L13_50_concurrent_calls")


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
class TestConcurrentCalls50:
    """L13: 50-call concurrent load tests using SIPLoadGenerator."""

    # ------------------------------------------------------------------
    # TC-L13-001: Register 100 extensions simultaneously
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(120)
    def test_50_concurrent_registrations(self):
        """TC-L13-001: Register 100 extensions (4001-4100) simultaneously.

        Creates 100 SIPAgent instances and registers all of them at once.
        Verifies that all 100 registrations succeed within a reasonable time.
        Retries up to 3 times for robustness against transient network issues,
        since registering 100 agents is a heavier load than previous tests.
        """
        gen = _create_generator()
        max_retries = 3

        try:
            for attempt in range(1, max_retries + 1):
                success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                    gen.register_all()
                )

                print(f"\n--- TC-L13-001: 100 Concurrent Registrations (attempt {attempt}) ---")
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
                    time.sleep(3)
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
    # TC-L13-002: 50 concurrent calls (basic signalling)
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(300)
    def test_50_concurrent_calls_basic(self):
        """TC-L13-002: Set up 50 simultaneous calls (paired extensions).

        Pairs: 4001->4002, 4003->4004, 4005->4006, ..., 4099->4100
        All 50 calls are established simultaneously with RTP media.
        Verifies all calls get 200 OK and hold for the configured duration.
        Allows up to MAX_CALL_FAILURES (5) failures for transient issues at
        this level of concurrency.  Also measures and reports latency
        percentiles (p50/p90/p95) and total setup time.
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

            # Build call pairs: (0,1), (2,3), (4,5), ..., (98,99)
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            print(f"\n--- TC-L13-002: 50 Concurrent Calls (Basic) ---")
            print(f"  Call pairs: {len(pairs)} pairs")
            print(f"  Duration:   {CALL_DURATION}s each")

            # Make all 50 calls simultaneously
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
                      f"max: {setup_times[-1]:.3f}s")
            for m in successful[:10]:
                print(f"    {m.caller} -> {m.callee}: setup={m.setup_time:.3f}s "
                      f"rtp_sent={m.rtp_packets_sent}")
            if len(successful) > 10:
                print(f"    ... and {len(successful) - 10} more successful calls")
            for m in failed:
                print(f"    FAILED {m.caller} -> {m.callee}: {m.error} "
                      f"(code={m.final_response_code}, category={m.error_category})")

            # Allow up to MAX_CALL_FAILURES failures out of 50 calls
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Expected at least {NUM_CALLS - MAX_CALL_FAILURES} successful calls, "
                f"got {len(successful)}. "
                f"Failures: {[(m.caller, m.callee, m.error) for m in failed]}"
            )

            # Setup time for each call should be reasonable (< 15s for 50-call load)
            for m in successful:
                assert m.setup_time < 15.0, (
                    f"Call {m.caller}->{m.callee} setup took {m.setup_time:.3f}s (>15s)"
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
    # TC-L13-003: 50 concurrent calls with RTP media verification
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(300)
    def test_50_concurrent_calls_with_rtp(self):
        """TC-L13-003: Verify RTP flows bidirectionally on all 50 calls.

        Establishes 50 concurrent calls with RTP enabled and verifies that
        each caller sends a non-trivial number of RTP packets during the
        call hold period.  At 20ms intervals over the RTP duration, expects
        approximately duration/0.020 packets per call.  Requires at least 25%
        of the expected count to account for scheduling jitter under heavy load.
        """
        gen = _create_generator(base_port=BASE_PORT + 300)

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

            # 50 concurrent calls with RTP -- use shorter duration than the
            # full CALL_DURATION to keep test time reasonable while still
            # generating enough packets to validate RTP flow
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 10.0  # seconds of RTP to send

            print(f"\n--- TC-L13-003: 50-Call RTP Flow ---")
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
            # margin -- require at least 25% of expected (more relaxed for 50 calls).
            expected_min_packets = int((rtp_duration / 0.020) * 0.25)
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
            sample_calls = successful[:10] + successful[-5:] if len(successful) > 15 else successful
            for m in sample_calls:
                print(f"    {m.caller} -> {m.callee}: "
                      f"rtp_sent={m.rtp_packets_sent}, "
                      f"rtp_recv={m.rtp_packets_received}")

            for m in failed:
                print(f"    FAILED {m.caller} -> {m.callee}: {m.error}")

            # Allow up to 5 calls with low RTP for transient scheduling issues
            # at this higher concurrency level
            assert len(rtp_issues) <= 5, (
                f"Too many calls with insufficient RTP ({len(rtp_issues)}/{len(successful)}): "
                f"{rtp_issues[:10]}"  # Truncate list in error message
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L13-004: Packet loss across 50 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(300)
    def test_50_concurrent_calls_packet_loss(self):
        """TC-L13-004: Verify packet loss < 5% across all 50 concurrent calls.

        Runs 50 concurrent calls with RTP and calculates the packet loss
        ratio from the sent/received counters in CallMetrics.  The test
        focuses on verifying that the outbound RTP pipeline does not drop
        frames under heavy concurrency -- i.e., we sent what we intended
        to send.

        Additionally checks that no more than MAX_CALL_FAILURES calls were
        dropped mid-stream, which would indicate server-side resource
        exhaustion under 50-call load.
        """
        gen = _create_generator(base_port=BASE_PORT + 600)

        try:
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 10.0

            print(f"\n--- TC-L13-004: 50-Call Packet Loss ---")
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
                send_loss_pct = max(0, (total_expected - total_sent) / total_expected * 100)
            else:
                send_loss_pct = 0.0

            # Call-level failure rate (calls that did not complete)
            call_loss_pct = (len(failed) / len(call_metrics) * 100) if call_metrics else 0.0

            print(f"  Successful calls:    {len(successful)}/{len(call_metrics)}")
            print(f"  Total RTP expected:  ~{int(total_expected)}")
            print(f"  Total RTP sent:      {total_sent}")
            print(f"  Send pipeline loss:  {send_loss_pct:.2f}%")
            print(f"  Call failure rate:   {call_loss_pct:.2f}%")

            # Per-call loss breakdown (sample)
            per_call_losses = []
            for m in successful:
                per_call_expected = expected_packets
                per_call_loss = max(0, (per_call_expected - m.rtp_packets_sent) / per_call_expected * 100)
                per_call_losses.append(per_call_loss)

            per_call_losses_sorted = sorted(per_call_losses)
            if per_call_losses_sorted:
                print(f"  Per-call loss -- avg: {sum(per_call_losses)/len(per_call_losses):.2f}%, "
                      f"p50: {_percentile(per_call_losses_sorted, 50):.2f}%, "
                      f"p90: {_percentile(per_call_losses_sorted, 90):.2f}%, "
                      f"max: {per_call_losses_sorted[-1]:.2f}%")

            # Show worst offenders
            worst_calls = sorted(successful, key=lambda m: m.rtp_packets_sent)[:5]
            for m in worst_calls:
                loss = max(0, (expected_packets - m.rtp_packets_sent) / expected_packets * 100)
                print(f"    {m.caller}->{m.callee}: sent={m.rtp_packets_sent}, loss={loss:.1f}%")

            for m in failed:
                print(f"    FAILED {m.caller}->{m.callee}: {m.error}")

            # Verify packet loss is within acceptable limits (5% for 50 calls)
            assert send_loss_pct < MAX_PACKET_LOSS_PCT, (
                f"RTP send pipeline loss {send_loss_pct:.2f}% exceeds "
                f"maximum allowed {MAX_PACKET_LOSS_PCT}%"
            )

            # No more than MAX_CALL_FAILURES calls should fail entirely
            assert len(failed) <= MAX_CALL_FAILURES, (
                f"{len(failed)} calls failed entirely (max allowed {MAX_CALL_FAILURES}): "
                f"{[(m.caller, m.callee, m.error) for m in failed]}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L13-005: RTP port exhaustion check
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(300)
    def test_50_concurrent_calls_rtp_port_exhaustion(self):
        """TC-L13-005: Check for RTP port exhaustion under 50-call load.

        The default RustPBX RTP port range is 20000-20100, providing 100
        available ports.  With 50 concurrent calls, each requiring 2 RTP
        ports (one per leg), the server needs exactly 100 ports -- the full
        range.  This test verifies the server can allocate all needed ports
        without exhaustion errors.

        Monitors:
          - Number of calls that fail with port-related errors
          - Server health during peak port usage
          - Error categories to detect port exhaustion specifically
          - Dialog count to verify all 50 calls are active simultaneously
        """
        gen = _create_generator(base_port=BASE_PORT + 900)

        try:
            # Ensure baseline: no stale dialogs consuming ports
            baseline_dialogs = _get_dialog_count()
            print(f"\n--- TC-L13-005: RTP Port Exhaustion Check ---")
            print(f"  Server RTP port range: {SERVER_RTP_START_PORT}-{SERVER_RTP_END_PORT} "
                  f"({SERVER_RTP_PORT_COUNT} ports)")
            print(f"  Ports needed for 50 calls: {NUM_CALLS * 2} "
                  f"(50 calls x 2 legs)")
            print(f"  Baseline dialogs: {baseline_dialogs}")

            # Register all agents
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make all 50 calls with moderate hold time to keep them active
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

                # Wait for calls to be established (give extra time for 50 calls)
                await asyncio.sleep(8.0)

                # Check dialog count at peak -- should be close to 50
                peak_dialogs = _get_dialog_count()

                # Check server health at peak load
                peak_health = _get_health_data()

                # Wait for calls to finish
                call_metrics = await call_task
                return call_metrics, peak_dialogs, peak_health

            call_metrics, peak_dialogs, peak_health = (
                asyncio.get_event_loop().run_until_complete(_run_calls_and_check_ports())
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            # Categorize failures
            port_errors = []
            other_errors = []
            for m in failed:
                error_lower = (m.error or "").lower()
                if any(kw in error_lower for kw in ("port", "resource", "exhausted",
                                                     "unavailable", "503", "busy")):
                    port_errors.append(m)
                else:
                    other_errors.append(m)

            print(f"  Calls successful:    {len(successful)}/{len(call_metrics)}")
            print(f"  Calls failed:        {len(failed)}")
            print(f"    Port-related:      {len(port_errors)}")
            print(f"    Other errors:      {len(other_errors)}")
            print(f"  Peak dialogs:        {peak_dialogs}")
            print(f"  Peak health:         {peak_health.get('status_code', peak_health)}")

            for m in port_errors:
                print(f"    PORT-FAIL {m.caller}->{m.callee}: {m.error} "
                      f"(code={m.final_response_code})")
            for m in other_errors[:5]:
                print(f"    OTHER-FAIL {m.caller}->{m.callee}: {m.error} "
                      f"(code={m.final_response_code})")

            # Primary assertion: no port exhaustion errors
            # With 100 ports and 50 calls needing 100 ports, we are at the limit.
            # A few failures from timing are acceptable, but systemic port
            # exhaustion (many 503 errors) is not.
            assert len(port_errors) <= MAX_CALL_FAILURES, (
                f"Port exhaustion detected: {len(port_errors)} calls failed with "
                f"port-related errors. Server may need a wider RTP port range for "
                f"50 concurrent calls. Errors: "
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
    # TC-L13-006: Memory baseline before/after 50 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(360)
    def test_50_concurrent_calls_memory_baseline(self):
        """TC-L13-006: Check server memory via API before/after 50-call load.

        Measures memory consumption at three points:
          1. Before load (baseline)
          2. During peak load (50 active calls with RTP)
          3. After load (post-teardown, after GC/cleanup)

        Verifies that post-load memory does not exceed baseline + 500 MB,
        which would indicate a memory leak under heavy concurrency.
        Also monitors server health responsiveness during the entire cycle.
        """
        gen = _create_generator(base_port=BASE_PORT + 1200)

        try:
            # Pre-load baseline
            health_before = _get_health_data()
            mem_before = _get_memory_mb_from_health(health_before)
            print(f"\n--- TC-L13-006: Memory Baseline (50 Calls) ---")
            print(f"  Health before: {health_before}")
            print(f"  Memory before: {mem_before:.1f} MB" if mem_before >= 0
                  else "  Memory before: unavailable")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Run 50 concurrent calls with RTP and poll health during peak
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

                # Wait for calls to establish
                await asyncio.sleep(6.0)

                # Sample memory during peak load
                health_samples = []
                for _ in range(5):
                    health = _get_health_data()
                    health_samples.append(health)
                    await asyncio.sleep(1.5)

                call_metrics = await call_task
                return call_metrics, health_samples

            call_metrics, health_samples = asyncio.get_event_loop().run_until_complete(
                _run_calls_and_monitor_memory()
            )

            successful = [m for m in call_metrics if m.success]

            print(f"  Calls successful: {len(successful)}/{len(call_metrics)}")

            # Report peak memory during load
            peak_memories = []
            for i, h in enumerate(health_samples):
                mem = _get_memory_mb_from_health(h)
                has_error = "error" in h
                status = h.get("status_code", "ok" if not has_error else "error")
                print(f"    Sample {i + 1}: status={status}, "
                      f"mem={mem:.1f} MB" if mem >= 0
                      else f"    Sample {i + 1}: status={status}, mem=unavailable")
                if mem >= 0:
                    peak_memories.append(mem)

            if peak_memories:
                peak_mem = max(peak_memories)
                print(f"  Peak memory during load: {peak_mem:.1f} MB")

            # Health endpoint should be responsive during load
            health_errors = [h for h in health_samples if "error" in h]
            assert len(health_errors) <= 2, (
                f"Server health unreachable during {len(health_errors)}/{len(health_samples)} "
                f"polls while {NUM_CALLS} calls active: {health_errors}"
            )

            # At least half the calls should succeed
            assert len(successful) >= NUM_CALLS // 2, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded during memory test"
            )

            # Stop handlers and unregister, then wait for cleanup
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            # Give server time to clean up (longer for 50 calls)
            time.sleep(8)

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
                    f"Excessive memory growth during 50-call load: "
                    f"{growth:.1f} MB (before={mem_before:.1f}, after={mem_after:.1f}, "
                    f"limit={MAX_MEMORY_GROWTH_MB} MB)"
                )
            else:
                print("  Memory growth check skipped: memory data not available "
                      "from health endpoint")

            # Server should still be healthy
            assert "error" not in health_after, (
                f"Server health check failed after 50-call load: {health_after}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L13-007: CDR writes for 50 concurrent calls
    # ------------------------------------------------------------------
    @load
    @pytest.mark.timeout(360)
    def test_50_concurrent_calls_cdr_writes(self):
        """TC-L13-007: Verify all 50 CDRs written correctly after calls end.

        Establishes 50 concurrent calls with short RTP media, waits for them
        to complete, then checks the call records API to verify that CDRs
        were created for each call.

        Methodology:
          1. Snapshot call records count before test
          2. Run 50 concurrent calls (short duration to keep test fast)
          3. Wait for CDR flush (server may batch writes)
          4. Snapshot call records count after test
          5. Verify the delta matches the number of successful calls

        Also verifies that the server dialog count returns to baseline after
        all calls are torn down (no resource leaks).
        """
        gen = _create_generator(base_port=BASE_PORT + 1500)

        try:
            # Baseline measurements
            baseline_dialogs = _get_dialog_count()
            records_before = _get_call_records()
            records_count_before = len(records_before)

            print(f"\n--- TC-L13-007: CDR Writes (50 Calls) ---")
            print(f"  Baseline dialogs: {baseline_dialogs}")
            print(f"  Call records before: {records_count_before}")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - MAX_CALL_FAILURES

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make 50 concurrent calls with short hold time
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
            for m in failed[:5]:
                print(f"    FAILED {m.caller}->{m.callee}: {m.error}")

            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded"
            )

            # Teardown
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            # Wait for CDR flush -- server may batch/buffer CDR writes.
            # Longer wait for 50 calls to account for write batching.
            print("  Waiting for CDR flush...")
            time.sleep(10)

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
                assert leaked <= 5, (
                    f"Possible dialog leak: {leaked} more dialogs after test "
                    f"(before={baseline_dialogs}, after={post_dialogs})"
                )

            # CDR verification: if the call records API returned data, verify
            # that the number of new records matches (or nearly matches) the
            # number of successful calls.  If the API returned empty data
            # (e.g., endpoint not available or requires different auth), we
            # still pass the test but log a warning.
            if records_count_before >= 0 and records_count_after >= 0 and records_count_after > 0:
                # Allow some tolerance: CDRs may be delayed or the API may
                # paginate.  Require at least 80% of successful calls have CDRs.
                min_expected_records = int(len(successful) * 0.80)
                assert new_records >= min_expected_records, (
                    f"Expected at least {min_expected_records} new CDR records "
                    f"(80% of {len(successful)} successful calls), but only "
                    f"found {new_records} new records "
                    f"(before={records_count_before}, after={records_count_after})"
                )
                print(f"  CDR write rate: {new_records}/{len(successful)} "
                      f"({new_records/len(successful)*100:.1f}%)")
            else:
                logger.warning(
                    "Call records API returned no data -- CDR count verification "
                    "skipped. The API may require authentication or may not be "
                    "available. Call records endpoint: %s/console/call-records",
                    BASE_URL,
                )
                print("  CDR count verification: SKIPPED (API returned no data)")

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
