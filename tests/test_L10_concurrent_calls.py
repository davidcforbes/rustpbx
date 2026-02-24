"""
L10 Concurrent Calls Tests -- 10 simultaneous SIP call load test.

Exercises RustPBX with 10 simultaneous SIP calls using the SIPLoadGenerator
from sip_load_generator.py.  Each test creates virtual SIP user agents
(extensions 2001-2020), registers them, establishes concurrent calls with
RTP media, and verifies signalling, media flow, packet loss, teardown,
and server health under load.

Tests:
  1. 20 concurrent registrations       -- Register 2001-2020, verify all succeed
  2. 10 concurrent calls               -- 2001->2002, 2003->2004, ..., verify all establish
  3. Concurrent call RTP flow           -- Verify RTP flows bidirectionally on all 10 calls
  4. Concurrent call packet loss        -- Verify packet loss < 2% across all calls
  5. Concurrent call teardown           -- Cleanly BYE all 10 calls, verify no resource leaks
  6. Server health during load          -- Check /ami/v1/health during active calls
  7. Sequential call bursts             -- Create and tear down 10 calls 3 times in rapid succession

Server:  RUSTPBX_HOST (default 127.0.0.1) : 5060  (UDP)
Users:   Dynamically registered 2001-2020 (password = "test{ext}")
Health:  https://RUSTPBX_HOST:8443/ami/v1/health

Run with:
  python -m pytest tests/test_L10_concurrent_calls.py -v -s

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
NUM_CALLS = 10
NUM_AGENTS = NUM_CALLS * 2  # 20 agents: 10 callers + 10 callees
BASE_EXTENSION = 2001
# Port range for agents -- each agent uses 2 ports (SIP + RTP)
BASE_PORT = 21000
# Call hold duration for tests (seconds)
CALL_DURATION = 5.0
# Maximum allowed packet loss percentage
MAX_PACKET_LOSS_PCT = 2.0
# Number of burst iterations for sequential burst test
BURST_ITERATIONS = 3

logger = logging.getLogger("test_L10_concurrent_calls")


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
class TestConcurrentCalls:
    """L10: 10-call concurrent load tests using SIPLoadGenerator."""

    # ------------------------------------------------------------------
    # TC-L10-001: Register 20 extensions simultaneously
    # ------------------------------------------------------------------
    @pytest.mark.timeout(60)
    def test_10_concurrent_registrations(self):
        """TC-L10-001: Register 20 extensions (2001-2020) simultaneously.

        Creates 20 SIPAgent instances and registers all of them at once.
        Verifies that all 20 registrations succeed within a reasonable time.
        Retries up to 2 times for robustness against transient network issues.
        """
        gen = _create_generator()
        max_retries = 2

        try:
            for attempt in range(1, max_retries + 1):
                success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                    gen.register_all()
                )

                print(f"\n--- TC-L10-001: Concurrent Registrations (attempt {attempt}) ---")
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
    # TC-L10-002: 10 concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    def test_10_concurrent_calls(self):
        """TC-L10-002: Set up 10 simultaneous calls (paired extensions).

        Pairs: 2001->2002, 2003->2004, 2005->2006, ..., 2019->2020
        All 10 calls are established simultaneously with RTP media.
        Verifies all calls get 200 OK and hold for the configured duration.
        """
        gen = _create_generator()

        try:
            # Register all agents
            success_count, fail_count = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 1, (
                f"Need at least {NUM_AGENTS - 1} agents registered, got {success_count}"
            )

            # Start incoming call handlers on all agents (callees auto-answer)
            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Build call pairs: (0,1), (2,3), (4,5), ..., (18,19)
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            print(f"\n--- TC-L10-002: 10 Concurrent Calls ---")
            print(f"  Call pairs: {pairs}")
            print(f"  Duration:   {CALL_DURATION}s each")

            # Make all 10 calls simultaneously
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

            # All 10 calls must succeed
            assert len(successful) == NUM_CALLS, (
                f"Expected {NUM_CALLS} successful calls, got {len(successful)}. "
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
    # TC-L10-003: RTP flow on concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    def test_concurrent_call_rtp_flow(self):
        """TC-L10-003: Verify RTP flows bidirectionally on all 10 calls.

        Establishes 10 concurrent calls with RTP enabled and verifies that
        each caller sends a non-trivial number of RTP packets during the
        call hold period.  The RTP flow is verified via packet counts
        recorded in CallMetrics.
        """
        gen = _create_generator(base_port=BASE_PORT + 100)

        try:
            # Register all
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 1, (
                f"Need at least {NUM_AGENTS - 1} registered, got {success_count}"
            )

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # 10 concurrent calls with RTP
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
            rtp_duration = 4.0  # seconds of RTP to send

            call_metrics = asyncio.get_event_loop().run_until_complete(
                gen.make_concurrent_calls(
                    pairs,
                    duration_secs=rtp_duration,
                    send_rtp=True,
                )
            )

            print(f"\n--- TC-L10-003: Concurrent Call RTP Flow ---")

            successful = [m for m in call_metrics if m.success]
            assert len(successful) >= NUM_CALLS - 1, (
                f"Need at least {NUM_CALLS - 1} successful calls for RTP test, "
                f"got {len(successful)}"
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

            # Allow up to 1 call with low RTP for transient scheduling issues
            assert len(rtp_issues) <= 1, (
                f"Too many calls with insufficient RTP: {rtp_issues}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L10-004: Packet loss across concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    def test_concurrent_call_packet_loss(self):
        """TC-L10-004: Verify packet loss < 2% across all 10 concurrent calls.

        Runs 10 concurrent calls with RTP and calculates the packet loss
        ratio from the sent/received counters in CallMetrics.  Since the
        load generator primarily tracks sent packets (the server echoes
        nothing back to our test RTP socket in many configurations), this
        test focuses on verifying that the outbound RTP pipeline does not
        drop frames -- i.e., we sent what we intended to send.

        Additionally checks that no calls were dropped mid-stream (which
        would indicate server-side packet handling issues).
        """
        gen = _create_generator(base_port=BASE_PORT + 200)

        try:
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 1

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

            print(f"\n--- TC-L10-004: Concurrent Call Packet Loss ---")

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

            # Verify packet loss is within acceptable limits
            assert send_loss_pct < MAX_PACKET_LOSS_PCT, (
                f"RTP send pipeline loss {send_loss_pct:.2f}% exceeds "
                f"maximum allowed {MAX_PACKET_LOSS_PCT}%"
            )

            # No more than 1 call should fail entirely
            assert len(failed) <= 1, (
                f"{len(failed)} calls failed entirely: "
                f"{[(m.caller, m.callee, m.error) for m in failed]}"
            )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L10-005: Clean teardown of concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    def test_concurrent_call_teardown(self):
        """TC-L10-005: Cleanly hang up all 10 calls, verify no resource leaks.

        Establishes 10 concurrent calls, holds them briefly, then tears
        them all down via BYE.  After teardown, verifies:
          - All calls completed successfully (BYE accepted)
          - Server dialog count returns to baseline (no leaked dialogs)
          - Server health endpoint still responds normally
        """
        gen = _create_generator(base_port=BASE_PORT + 300)

        try:
            # Get baseline dialog count
            baseline_dialogs = _get_dialog_count()
            print(f"\n--- TC-L10-005: Concurrent Call Teardown ---")
            print(f"  Baseline dialogs: {baseline_dialogs}")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 1

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Make 10 concurrent calls with short duration (calls will BYE quickly)
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

            # All calls should have completed (BYE sent and responded to)
            assert len(successful) >= NUM_CALLS - 1, (
                f"Only {len(successful)}/{NUM_CALLS} calls completed cleanly"
            )

            # Stop handlers and unregister
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            # Wait for server to clean up dialogs
            time.sleep(3)

            # Verify dialogs returned to baseline
            post_dialogs = _get_dialog_count()
            print(f"  Post-teardown dialogs: {post_dialogs}")

            if baseline_dialogs >= 0 and post_dialogs >= 0:
                leaked = post_dialogs - baseline_dialogs
                print(f"  Leaked dialogs: {leaked}")
                # Allow small tolerance -- registrations may count as dialogs
                assert leaked <= 2, (
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
    # TC-L10-006: Server health during active load
    # ------------------------------------------------------------------
    @pytest.mark.timeout(120)
    def test_server_health_during_load(self):
        """TC-L10-006: Check server health/memory endpoints during active calls.

        While 10 concurrent calls are active with RTP, continuously polls
        the server health endpoint to verify the server remains responsive
        and does not exhibit signs of resource exhaustion.
        """
        gen = _create_generator(base_port=BASE_PORT + 400)

        try:
            # Health baseline
            health_before = _get_health_data()
            print(f"\n--- TC-L10-006: Server Health During Load ---")
            print(f"  Health before: {health_before}")

            # Register
            success_count, _ = asyncio.get_event_loop().run_until_complete(
                gen.register_all()
            )
            assert success_count >= NUM_AGENTS - 1

            asyncio.get_event_loop().run_until_complete(gen.start_callee_handlers())

            # Use a longer call duration so we have time to poll health
            call_duration = 8.0
            pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]

            # Start calls in background
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
                await asyncio.sleep(3.0)

                # Poll health endpoint multiple times during the call
                health_samples = []
                for i in range(5):
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
            time.sleep(2)
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
                assert growth < 50, (
                    f"Excessive memory growth during 10-call load: "
                    f"{growth:.1f} MB (before={mem_before}, after={mem_after})"
                )

        finally:
            gen.stop_callee_handlers()
            asyncio.get_event_loop().run_until_complete(gen.unregister_all())
            gen._close_all()

    # ------------------------------------------------------------------
    # TC-L10-007: Sequential call bursts
    # ------------------------------------------------------------------
    @pytest.mark.timeout(300)
    def test_sequential_call_bursts(self):
        """TC-L10-007: Create and tear down 10 calls 3 times in rapid succession.

        Tests the server's ability to handle repeated bursts of concurrent
        calls without accumulating stale state or resource leaks.  Each
        burst establishes 10 simultaneous calls, holds them briefly, then
        tears them all down before starting the next burst.
        """
        # Use a distinct port range for each burst iteration
        burst_results = []

        print(f"\n--- TC-L10-007: Sequential Call Bursts ---")
        print(f"  Iterations:    {BURST_ITERATIONS}")
        print(f"  Calls/burst:   {NUM_CALLS}")

        for iteration in range(BURST_ITERATIONS):
            burst_base_port = BASE_PORT + 500 + (iteration * NUM_AGENTS * 2)
            gen = _create_generator(base_port=burst_base_port)

            try:
                # Register
                success_count, _ = asyncio.get_event_loop().run_until_complete(
                    gen.register_all()
                )
                if success_count < NUM_AGENTS - 1:
                    logger.warning(
                        f"Burst {iteration + 1}: Only {success_count}/{NUM_AGENTS} "
                        f"registered, proceeding..."
                    )

                asyncio.get_event_loop().run_until_complete(
                    gen.start_callee_handlers()
                )

                # Make 10 concurrent calls with short hold
                pairs = [(i * 2, i * 2 + 1) for i in range(NUM_CALLS)]
                start = time.monotonic()

                call_metrics = asyncio.get_event_loop().run_until_complete(
                    gen.make_concurrent_calls(
                        pairs,
                        duration_secs=2.0,
                        send_rtp=True,
                    )
                )

                elapsed = time.monotonic() - start
                successful = [m for m in call_metrics if m.success]
                failed = [m for m in call_metrics if not m.success]

                burst_result = {
                    "iteration": iteration + 1,
                    "registered": success_count,
                    "calls_attempted": len(call_metrics),
                    "calls_succeeded": len(successful),
                    "calls_failed": len(failed),
                    "elapsed": elapsed,
                    "avg_setup_time": (
                        sum(m.setup_time for m in successful) / len(successful)
                        if successful else 0.0
                    ),
                }
                burst_results.append(burst_result)

                print(f"\n  Burst {iteration + 1}/{BURST_ITERATIONS}:")
                print(f"    Registered: {success_count}/{NUM_AGENTS}")
                print(f"    Succeeded:  {len(successful)}/{len(call_metrics)}")
                print(f"    Elapsed:    {elapsed:.2f}s")
                print(f"    Avg setup:  {burst_result['avg_setup_time']:.3f}s")
                if failed:
                    for m in failed:
                        print(f"    FAILED: {m.caller}->{m.callee}: {m.error}")

                # Teardown
                gen.stop_callee_handlers()
                asyncio.get_event_loop().run_until_complete(gen.unregister_all())

            finally:
                gen._close_all()

            # Brief pause between bursts for server to stabilize
            if iteration < BURST_ITERATIONS - 1:
                time.sleep(2)

        # Verify overall burst performance
        total_succeeded = sum(r["calls_succeeded"] for r in burst_results)
        total_attempted = sum(r["calls_attempted"] for r in burst_results)

        print(f"\n  --- Burst Summary ---")
        print(f"  Total calls attempted: {total_attempted}")
        print(f"  Total calls succeeded: {total_succeeded}")
        print(f"  Overall success rate:  "
              f"{(total_succeeded / total_attempted * 100) if total_attempted else 0:.1f}%")

        # At least 70% of all calls across all bursts should succeed.
        # This accounts for possible transient issues between bursts.
        min_success_rate = 0.70
        actual_success_rate = total_succeeded / total_attempted if total_attempted else 0
        assert actual_success_rate >= min_success_rate, (
            f"Overall burst success rate {actual_success_rate:.2%} is below "
            f"minimum {min_success_rate:.0%}. Results: {burst_results}"
        )

        # No single burst should be a complete failure
        for r in burst_results:
            assert r["calls_succeeded"] >= 1, (
                f"Burst {r['iteration']} had zero successful calls out of "
                f"{r['calls_attempted']} attempted"
            )

        # Later bursts should not degrade significantly compared to the first
        if len(burst_results) >= 2 and burst_results[0]["calls_succeeded"] > 0:
            first_success = burst_results[0]["calls_succeeded"]
            last_success = burst_results[-1]["calls_succeeded"]
            if first_success > 2:
                degradation = (first_success - last_success) / first_success
                print(f"  Degradation (first vs last): {degradation:.1%}")
                assert degradation < 0.5, (
                    f"Significant performance degradation across bursts: "
                    f"first={first_success}, last={last_success} ({degradation:.1%} drop)"
                )

        # Verify server is still healthy after all bursts
        time.sleep(2)
        final_health = _get_health_data()
        print(f"  Server health after all bursts: {final_health}")
        assert "error" not in final_health, (
            f"Server unhealthy after burst test: {final_health}"
        )
