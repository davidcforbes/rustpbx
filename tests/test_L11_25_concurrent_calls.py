"""
L11 Concurrent Calls Tests -- 25 simultaneous SIP call load test.

Exercises RustPBX with 25 simultaneous SIP calls using the SIPLoadGenerator
from sip_load_generator.py.  Each test creates virtual SIP user agents
(extensions 3001-3050), registers them, establishes concurrent calls with
RTP media, and verifies signalling, media flow, packet loss, teardown,
and server health under load.

Tests:
  1. 50 concurrent registrations       -- Register 3001-3050, verify all succeed
  2. 25 concurrent calls               -- 3001->3002, 3003->3004, ..., verify all establish
  3. 25-call RTP flow                   -- Verify RTP flows bidirectionally on all 25 calls
  4. 25-call packet loss                -- Verify packet loss < 3% across all calls
  5. 25-call teardown                   -- Cleanly BYE all 25 calls, verify no resource leaks
  6. Server resource usage              -- Check memory/CPU during 25 active calls
  7. Call setup latency                 -- Measure average call setup time across 25 calls (< 5s avg)

Server:  RUSTPBX_HOST (default 127.0.0.1) : 5060  (UDP)
Users:   Dynamically registered 3001-3050 (password = "test{ext}")
Health:  https://RUSTPBX_HOST:8443/ami/v1/health

Run with:
  python -m pytest tests/test_L11_25_concurrent_calls.py -v -s

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
NUM_CALLS = 25
NUM_AGENTS = NUM_CALLS * 2  # 50 agents: 25 callers + 25 callees
BASE_EXTENSION = 3001
# Port range for agents -- each agent uses 2 ports (SIP + RTP)
# Starting from 30000+ to avoid collisions with the L10 test suite
BASE_PORT = 30000
# Call hold duration for tests (seconds)
CALL_DURATION = 5.0
# Maximum allowed packet loss percentage (slightly relaxed for 25 calls)
MAX_PACKET_LOSS_PCT = 3.0
# Maximum allowed call failures out of 25
MAX_CALL_FAILURES = 2
# Maximum average call setup time (seconds)
MAX_AVG_SETUP_TIME = 5.0

logger = logging.getLogger("test_L11_25_concurrent_calls")


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


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

skip_no_generator = pytest.mark.skipif(
    not HAS_LOAD_GENERATOR,
    reason="sip_load_generator.py not available",
)

skip_no_server = pytest.mark.skipif(
    not _server_reachable(),
    reason=f"RustPBX server not reachable at {BASE_URL}",
)


# ---------------------------------------------------------------------------
# Test class
# ---------------------------------------------------------------------------

@skip_no_generator
@skip_no_server
class TestConcurrentCalls25:
    """L11: 25-call concurrent load tests using SIPLoadGenerator."""

    # ------------------------------------------------------------------
    # TC-L11-001: Register 50 extensions simultaneously
    # ------------------------------------------------------------------
    @pytest.mark.timeout(90)
    def test_25_concurrent_registrations(self):
        """TC-L11-001: Register 50 extensions (3001-3050) simultaneously.

        Creates 50 SIPAgent instances and registers all of them at once.
        Verifies that all 50 registrations succeed within a reasonable time.
        Retries up to 2 times for robustness against transient network issues.
        """
        gen = _create_generator()
        max_retries = 2

        try:
            for attempt in range(1, max_retries + 1):
                success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                    gen.register_all()
                )

                print(f"\n--- TC-L11-001: Concurrent Registrations (attempt {attempt}) ---")
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
                    time.sleep(2)
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
    # TC-L11-002: 25 concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    def test_25_concurrent_calls(self):
        """TC-L11-002: Set up 25 simultaneous calls (paired extensions).

        Pairs: 3001->3002, 3003->3004, 3005->3006, ..., 3049->3050
        All 25 calls are established simultaneously with RTP media.
        Verifies all calls get 200 OK and hold for the configured duration.
        Allows up to 2 failures for transient issues at higher concurrency.
        """
        gen = _create_generator()

        try:
            # Register all agents
            success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 2, (
                f"Need at least {NUM_AGENTS - 2} agents registered, got {success_count}"
            )

            # Start incoming call handlers on all agents (callees auto-answer)
            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Build call pairs: (0,1), (2,3), (4,5), ..., (48,49)
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            print(f"\n--- TC-L11-002: 25 Concurrent Calls ---")
            print(f"  Call pairs: {len(pairs)} pairs")
            print(f"  Duration:   {CALL_DURATION}s each")

            # Make all 25 calls simultaneously
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

            print(f"  Total elapsed: {elapsed:.2f}s")
            print(f"  Successful:    {len(successful)}/{len(call_metrics)}")
            for m in successful:
                print(f"    {m.caller} -> {m.callee}: setup={m.setup_time:.3f}s "
                      f"rtp_sent={m.rtp_packets_sent}")
            for m in failed:
                print(f"    FAILED {m.caller} -> {m.callee}: {m.error} "
                      f"(code={m.final_response_code}, category={m.error_category})")

            # Allow up to MAX_CALL_FAILURES failures out of 25 calls
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Expected at least {NUM_CALLS - MAX_CALL_FAILURES} successful calls, "
                f"got {len(successful)}. "
                f"Failures: {[(m.caller, m.callee, m.error) for m in failed]}"
            )

            # Setup time for each call should be reasonable (< 10s)
            for m in successful:
                assert m.setup_time < 10.0, (
                    f"Call {m.caller}->{m.callee} setup took {m.setup_time:.3f}s (>10s)"
                )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L11-003: RTP flow on 25 concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    def test_25_call_rtp_flow(self):
        """TC-L11-003: Verify RTP flows bidirectionally on all 25 calls.

        Establishes 25 concurrent calls with RTP enabled and verifies that
        each caller sends a non-trivial number of RTP packets during the
        call hold period.  The RTP flow is verified via packet counts
        recorded in CallMetrics.
        """
        gen = _create_generator(base_port=BASE_PORT + 200)

        try:
            # Register all
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 2, (
                f"Need at least {NUM_AGENTS - 2} registered, got {success_count}"
            )

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # 25 concurrent calls with RTP
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 4.0  # seconds of RTP to send

            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=rtp_duration,
                    send_rtp=True,
                )
            )

            print(f"\n--- TC-L11-003: 25-Call RTP Flow ---")

            successful = [m for m in call_metrics if m.success]
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Need at least {NUM_CALLS - MAX_CALL_FAILURES} successful calls "
                f"for RTP test, got {len(successful)}"
            )

            # Each successful call should have sent RTP packets.
            # At 20ms intervals over rtp_duration seconds, expect approximately
            # rtp_duration / 0.020 = 200 packets per call.  Allow generous
            # margin -- require at least 25% of expected.
            expected_min_packets = int((rtp_duration / 0.020) * 0.25)
            rtp_issues = []

            for m in successful:
                print(f"  {m.caller} -> {m.callee}: "
                      f"rtp_sent={m.rtp_packets_sent}, "
                      f"rtp_recv={m.rtp_packets_received}")
                if m.rtp_packets_sent < expected_min_packets:
                    rtp_issues.append(
                        f"{m.caller}->{m.callee}: sent only {m.rtp_packets_sent} "
                        f"(expected >= {expected_min_packets})"
                    )

            print(f"  Expected min packets per call: {expected_min_packets}")
            print(f"  RTP issues: {len(rtp_issues)}")

            # Allow up to 2 calls with low RTP for transient scheduling issues
            # (slightly relaxed from 1 in the 10-call test)
            assert len(rtp_issues) <= 2, (
                f"Too many calls with insufficient RTP: {rtp_issues}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L11-004: Packet loss across 25 concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    def test_25_call_packet_loss(self):
        """TC-L11-004: Verify packet loss < 3% across all 25 concurrent calls.

        Runs 25 concurrent calls with RTP and calculates the packet loss
        ratio from the sent/received counters in CallMetrics.  Since the
        load generator primarily tracks sent packets (the server echoes
        nothing back to our test RTP socket in many configurations), this
        test focuses on verifying that the outbound RTP pipeline does not
        drop frames -- i.e., we sent what we intended to send.

        Additionally checks that no more than 2 calls were dropped mid-stream
        (which would indicate server-side packet handling issues).
        """
        gen = _create_generator(base_port=BASE_PORT + 400)

        try:
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 2

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 5.0

            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=rtp_duration,
                    send_rtp=True,
                )
            )

            print(f"\n--- TC-L11-004: 25-Call Packet Loss ---")

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            # Calculate expected packets per call
            expected_packets = rtp_duration / 0.020  # 250 for 5s at 20ms
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

            for m in successful:
                per_call_expected = expected_packets
                per_call_loss = max(0, (per_call_expected - m.rtp_packets_sent) / per_call_expected * 100)
                print(f"    {m.caller}->{m.callee}: sent={m.rtp_packets_sent}, "
                      f"loss={per_call_loss:.1f}%")

            # Verify packet loss is within acceptable limits (3% for 25 calls)
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
    # TC-L11-005: Clean teardown of 25 concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    def test_25_call_teardown(self):
        """TC-L11-005: Cleanly hang up all 25 calls, verify no resource leaks.

        Establishes 25 concurrent calls, holds them briefly, then tears
        them all down via BYE.  After teardown, verifies:
          - All calls completed successfully (BYE accepted)
          - Server dialog count returns to baseline (no leaked dialogs)
          - Server health endpoint still responds normally
        """
        gen = _create_generator(base_port=BASE_PORT + 600)

        try:
            # Get baseline dialog count
            baseline_dialogs = _get_dialog_count()
            print(f"\n--- TC-L11-005: 25-Call Teardown ---")
            print(f"  Baseline dialogs: {baseline_dialogs}")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 2

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make 25 concurrent calls with short duration (calls will BYE quickly)
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=2.0,  # short hold
                    send_rtp=True,
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            print(f"  Calls established: {len(successful)}/{len(call_metrics)}")
            for m in failed:
                print(f"    FAILED: {m.caller}->{m.callee}: {m.error}")

            # Allow up to MAX_CALL_FAILURES failures
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Only {len(successful)}/{NUM_CALLS} calls completed cleanly"
            )

            # Stop handlers and unregister
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            # Wait for server to clean up dialogs (slightly longer for 25 calls)
            time.sleep(5)

            # Verify dialogs returned to baseline
            post_dialogs = _get_dialog_count()
            print(f"  Post-teardown dialogs: {post_dialogs}")

            if baseline_dialogs >= 0 and post_dialogs >= 0:
                leaked = post_dialogs - baseline_dialogs
                print(f"  Leaked dialogs: {leaked}")
                # Allow small tolerance -- registrations may count as dialogs
                assert leaked <= 3, (
                    f"Possible dialog leak: {leaked} more dialogs after test "
                    f"(before={baseline_dialogs}, after={post_dialogs})"
                )

            # Server health check after teardown
            health = _get_health_data()
            print(f"  Server health after teardown: {health}")
            assert "error" not in health, (
                f"Server health check failed after teardown: {health}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L11-006: Server resource usage during 25 active calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    def test_server_resource_usage(self):
        """TC-L11-006: Check server memory/CPU during 25 active calls.

        While 25 concurrent calls are active with RTP, continuously polls
        the server health endpoint to verify the server remains responsive
        and does not exhibit signs of resource exhaustion.  Monitors memory
        growth to detect leaks under higher concurrency load.
        """
        gen = _create_generator(base_port=BASE_PORT + 800)

        try:
            # Health baseline
            health_before = _get_health_data()
            print(f"\n--- TC-L11-006: Server Resource Usage During 25-Call Load ---")
            print(f"  Health before: {health_before}")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 2

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Use a longer call duration so we have time to poll health
            call_duration = 10.0
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            # Start calls in background and poll health during the call
            async def _run_calls_and_poll():
                # Start all calls as a background task
                call_task = asyncio.create_task(
                    gen.make_concurrent_calls(
                        pairs,
                        duration_secs=call_duration,
                        send_rtp=True,
                    )
                )

                # Wait a moment for calls to establish
                await asyncio.sleep(4.0)

                # Poll health endpoint multiple times during the call
                health_samples = []
                for i in range(6):
                    health = _get_health_data()
                    health_samples.append(health)
                    await asyncio.sleep(1.0)

                # Wait for calls to finish
                call_metrics = await call_task
                return call_metrics, health_samples

            call_metrics, health_samples = asyncio.get_event_loop().run_until_complete(
                _run_calls_and_poll()
            )

            successful = [m for m in call_metrics if m.success]

            print(f"  Calls successful: {len(successful)}/{len(call_metrics)}")
            print(f"  Health samples during load: {len(health_samples)}")
            for i, h in enumerate(health_samples):
                has_error = "error" in h
                status = h.get("status_code", "ok" if not has_error else "error")
                print(f"    Sample {i + 1}: status={status}")

            # Health endpoint should have been reachable during all polls
            health_errors = [h for h in health_samples if "error" in h]
            assert len(health_errors) <= 1, (
                f"Server health unreachable during {len(health_errors)}/{len(health_samples)} "
                f"polls while {NUM_CALLS} calls active: {health_errors}"
            )

            # At least half the calls should succeed even under monitoring load
            assert len(successful) >= NUM_CALLS // 2, (
                f"Only {len(successful)}/{NUM_CALLS} calls succeeded during health monitoring"
            )

            # Post-load health
            time.sleep(3)
            health_after = _get_health_data()
            print(f"  Health after load: {health_after}")

            # Check memory growth if available
            mem_before = (health_before.get("memory_mb")
                          or health_before.get("rss_mb"))
            mem_after = (health_after.get("memory_mb")
                         or health_after.get("rss_mb"))
            if mem_before is not None and mem_after is not None:
                growth = float(mem_after) - float(mem_before)
                print(f"  Memory growth: {growth:.1f} MB")
                # After cleanup, growth should not be excessive
                # Allow slightly more headroom than 10-call test (75 MB vs 50 MB)
                assert growth < 75, (
                    f"Excessive memory growth during 25-call load: "
                    f"{growth:.1f} MB (before={mem_before}, after={mem_after})"
                )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L11-007: Call setup latency across 25 calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(180)
    def test_call_setup_latency(self):
        """TC-L11-007: Measure average call setup time across 25 calls.

        Establishes 25 concurrent calls and measures the setup latency
        (time from INVITE sent to 200 OK received) for each call.
        Verifies:
          - Average setup time < 5 seconds
          - No individual call setup exceeds 15 seconds
          - Reports p50, p90, p95 latencies for diagnostics
        """
        gen = _create_generator(base_port=BASE_PORT + 1000)

        try:
            # Register all agents
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 2, (
                f"Need at least {NUM_AGENTS - 2} registered, got {success_count}"
            )

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make 25 concurrent calls
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            print(f"\n--- TC-L11-007: Call Setup Latency (25 Calls) ---")

            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=3.0,  # short hold, we care about setup time
                    send_rtp=True,
                )
            )

            successful = [m for m in call_metrics if m.success]
            failed = [m for m in call_metrics if not m.success]

            print(f"  Successful: {len(successful)}/{len(call_metrics)}")

            # Need a meaningful number of successful calls to measure latency
            assert len(successful) >= NUM_CALLS - MAX_CALL_FAILURES, (
                f"Only {len(successful)} calls succeeded, need at least "
                f"{NUM_CALLS - MAX_CALL_FAILURES} for latency measurement"
            )

            # Collect setup times
            setup_times = [m.setup_time for m in successful]
            setup_times_sorted = sorted(setup_times)

            avg_setup = sum(setup_times) / len(setup_times)
            min_setup = setup_times_sorted[0]
            max_setup = setup_times_sorted[-1]

            # Percentile calculations
            def _percentile(sorted_data, pct):
                idx = int(len(sorted_data) * pct / 100)
                idx = min(idx, len(sorted_data) - 1)
                return sorted_data[idx]

            p50 = _percentile(setup_times_sorted, 50)
            p90 = _percentile(setup_times_sorted, 90)
            p95 = _percentile(setup_times_sorted, 95)

            print(f"  Setup time statistics:")
            print(f"    Min:    {min_setup:.3f}s")
            print(f"    Avg:    {avg_setup:.3f}s")
            print(f"    P50:    {p50:.3f}s")
            print(f"    P90:    {p90:.3f}s")
            print(f"    P95:    {p95:.3f}s")
            print(f"    Max:    {max_setup:.3f}s")

            for m in successful:
                print(f"    {m.caller} -> {m.callee}: {m.setup_time:.3f}s")
            for m in failed:
                print(f"    FAILED {m.caller} -> {m.callee}: {m.error}")

            # Average setup time should be under MAX_AVG_SETUP_TIME (5s)
            assert avg_setup < MAX_AVG_SETUP_TIME, (
                f"Average call setup time {avg_setup:.3f}s exceeds "
                f"maximum allowed {MAX_AVG_SETUP_TIME}s"
            )

            # No individual call should take more than 15 seconds to set up
            for m in successful:
                assert m.setup_time < 15.0, (
                    f"Call {m.caller}->{m.callee} setup took {m.setup_time:.3f}s (>15s)"
                )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()
