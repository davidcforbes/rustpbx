"""
L12 Long-Duration Stability Tests -- Extended runtime tests for RustPBX.

Exercises RustPBX over sustained periods to detect memory leaks, resource
exhaustion, registration leaks, and degradation under continuous load.
Each test runs for a configurable duration (default 5 minutes for CI) and
produces a summary report with min/max/avg/p95 metrics.

Tests:
  1. Continuous call cycling       -- Create/hold/teardown calls in a loop
  2. Memory stability              -- Sample server memory during sustained load
  3. Registration churn            -- Continuously register/unregister extensions
  4. Sustained concurrent calls    -- Maintain N concurrent calls for full duration
  5. API reliability under load    -- Poll health endpoints during call load

Server:  RUSTPBX_HOST (default 127.0.0.1) : 5060  (UDP)
Users:   Dynamically registered 3001+ (password = "test{ext}")
Health:  https://RUSTPBX_HOST:8443/ami/v1/health

Run with:
  python -m pytest tests/test_L12_stability.py -v -s -m slow

Environment variables:
  STABILITY_DURATION_SECS    Total test duration      (default: 300 = 5 min)
  STABILITY_CALL_COUNT       Concurrent call target   (default: 5)
  STABILITY_CYCLE_HOLD_SECS  Hold time per call cycle (default: 30)
  RUSTPBX_HOST               SIP server IP            (default: 127.0.0.1)
  RUSTPBX_SIP_PORT           SIP port                 (default: 5060)
  RUSTPBX_EXTERNAL_IP        Public IP for SIP URIs   (default: same as HOST)
  RUSTPBX_HTTP_PORT          HTTP(S) port             (default: 8443)
  RUSTPBX_SCHEME             http or https            (default: https)
"""

import asyncio
import logging
import os
import statistics
import sys
import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

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

# Stability test tunables
STABILITY_DURATION_SECS = int(os.environ.get("STABILITY_DURATION_SECS", "300"))
STABILITY_CALL_COUNT = int(os.environ.get("STABILITY_CALL_COUNT", "5"))
STABILITY_CYCLE_HOLD_SECS = int(os.environ.get("STABILITY_CYCLE_HOLD_SECS", "30"))

# Agent pool sizing -- need 2 agents per concurrent call
NUM_AGENTS = STABILITY_CALL_COUNT * 2
BASE_EXTENSION = 3001
# Port range -- distinct from L10 tests to avoid conflicts
BASE_PORT = 23000

# Memory growth threshold (percentage above baseline)
MEMORY_GROWTH_THRESHOLD_PCT = 50

logger = logging.getLogger("test_L12_stability")


# ---------------------------------------------------------------------------
# Metric collection helpers
# ---------------------------------------------------------------------------

@dataclass
class TimeSample:
    """A timestamped metric sample."""
    timestamp: float  # monotonic time
    wall_time: float  # time.time() for human-readable logs
    value: float


@dataclass
class StabilityReport:
    """Aggregated metrics report for a stability test."""
    test_name: str
    duration_secs: float = 0.0
    total_iterations: int = 0
    successful_iterations: int = 0
    failed_iterations: int = 0
    samples: Dict[str, List[float]] = field(default_factory=dict)
    errors: List[str] = field(default_factory=list)

    def add_sample(self, metric_name: str, value: float):
        if metric_name not in self.samples:
            self.samples[metric_name] = []
        self.samples[metric_name].append(value)

    def stats(self, metric_name: str) -> Dict[str, Any]:
        """Compute min/max/avg/p95/p99 for a named metric."""
        values = self.samples.get(metric_name, [])
        if not values:
            return {"count": 0}
        sorted_vals = sorted(values)
        n = len(sorted_vals)
        return {
            "count": n,
            "min": round(sorted_vals[0], 4),
            "max": round(sorted_vals[-1], 4),
            "avg": round(statistics.mean(values), 4),
            "median": round(statistics.median(values), 4),
            "p95": round(sorted_vals[min(int(n * 0.95), n - 1)], 4),
            "p99": round(sorted_vals[min(int(n * 0.99), n - 1)], 4),
            "stdev": round(statistics.stdev(values), 4) if n > 1 else 0.0,
        }

    def print_report(self):
        """Print a formatted summary to stdout."""
        success_rate = (
            self.successful_iterations / self.total_iterations * 100
            if self.total_iterations > 0 else 0.0
        )

        print(f"\n{'=' * 72}")
        print(f"  Stability Report: {self.test_name}")
        print(f"{'=' * 72}")
        print(f"  Duration:         {self.duration_secs:.1f}s")
        print(f"  Iterations:       {self.total_iterations}")
        print(f"  Successful:       {self.successful_iterations}")
        print(f"  Failed:           {self.failed_iterations}")
        print(f"  Success rate:     {success_rate:.1f}%")

        for metric_name in sorted(self.samples.keys()):
            st = self.stats(metric_name)
            if st["count"] == 0:
                continue
            print(f"\n  {metric_name} ({st['count']} samples):")
            print(f"    Min:    {st['min']}")
            print(f"    Avg:    {st['avg']}")
            print(f"    Median: {st['median']}")
            print(f"    p95:    {st['p95']}")
            print(f"    p99:    {st['p99']}")
            print(f"    Max:    {st['max']}")
            if st["stdev"] > 0:
                print(f"    StdDev: {st['stdev']}")

        if self.errors:
            print(f"\n  Errors ({len(self.errors)} total):")
            # Show first 10 unique errors
            unique_errors = list(dict.fromkeys(self.errors))[:10]
            for err in unique_errors:
                count = self.errors.count(err)
                print(f"    [{count}x] {err}")

        print(f"{'=' * 72}")


# ---------------------------------------------------------------------------
# Server helpers
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


def _get_memory_mb() -> Optional[float]:
    """Extract server memory usage in MB from health endpoint."""
    data = _get_health_data()
    if "error" in data:
        return None
    # Try common field names
    for key in ("memory_mb", "rss_mb", "mem_mb", "memory"):
        val = data.get(key)
        if val is not None:
            try:
                return float(val)
            except (TypeError, ValueError):
                pass
    return None


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


def _get_registration_count() -> int:
    """Fetch the current number of active registrations from the server."""
    try:
        resp = requests.get(
            f"{BASE_URL}/ami/v1/registrations",
            timeout=10,
            verify=VERIFY_TLS,
        )
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                return len(data)
            elif isinstance(data, dict):
                return data.get("count", len(data.get("registrations", [])))
        return -1
    except (requests.RequestException, ValueError):
        return -1


def _timed_health_request(url: str) -> tuple:
    """Make a health request and return (status_code, response_time_ms, error).

    Returns:
        Tuple of (status_code, response_time_ms, error_string_or_None).
    """
    start = time.monotonic()
    try:
        resp = requests.get(url, timeout=10, verify=VERIFY_TLS)
        elapsed_ms = (time.monotonic() - start) * 1000
        return resp.status_code, elapsed_ms, None
    except requests.RequestException as exc:
        elapsed_ms = (time.monotonic() - start) * 1000
        return 0, elapsed_ms, str(exc)


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
# Async event loop helper
# ---------------------------------------------------------------------------

def _run(coro):
    """Run an async coroutine in the current or a new event loop."""
    try:
        loop = asyncio.get_event_loop()
        if loop.is_closed():
            raise RuntimeError("closed")
    except RuntimeError:
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
    return loop.run_until_complete(coro)


# ---------------------------------------------------------------------------
# Skip markers
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
@pytest.mark.slow
class TestLongDurationStability:
    """L12: Long-duration stability tests for RustPBX.

    These tests run for an extended, configurable duration to detect:
    - Memory leaks and resource exhaustion
    - Registration state leaks
    - Call setup degradation over time
    - API responsiveness under sustained load

    Duration is controlled by STABILITY_DURATION_SECS (default 300s = 5 min).
    Mark with @pytest.mark.slow so they can be excluded from fast CI runs.
    """

    # ------------------------------------------------------------------
    # TC-L12-001: Continuous call cycling
    # ------------------------------------------------------------------
    @pytest.mark.timeout(STABILITY_DURATION_SECS + 120)
    def test_continuous_call_cycling(self):
        """TC-L12-001: Create, hold, tear down calls in a repeating cycle.

        Runs for STABILITY_DURATION_SECS. Each cycle:
        1. Create STABILITY_CALL_COUNT concurrent calls
        2. Hold for STABILITY_CYCLE_HOLD_SECS
        3. Tear down all calls
        4. Record setup times, success/failure, and cycle duration

        Produces a summary report with per-cycle and aggregate metrics.
        Fails if success rate drops below 60% or final cycles degrade
        significantly compared to early cycles.
        """
        report = StabilityReport(test_name="Continuous Call Cycling")

        # Use a port offset to avoid conflict with other tests
        port_offset = 0
        gen = _create_generator(base_port=BASE_PORT + port_offset)

        print(f"\n--- TC-L12-001: Continuous Call Cycling ---")
        print(f"  Duration:    {STABILITY_DURATION_SECS}s")
        print(f"  Calls/cycle: {STABILITY_CALL_COUNT}")
        print(f"  Hold time:   {STABILITY_CYCLE_HOLD_SECS}s")

        try:
            # Register all agents once at the start
            success_count, fail_count = _run(gen.register_all())
            assert success_count >= NUM_AGENTS - 1, (
                f"Need at least {NUM_AGENTS - 1} agents registered, "
                f"got {success_count}"
            )

            _run(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(STABILITY_CALL_COUNT)]
            test_start = time.monotonic()
            test_deadline = test_start + STABILITY_DURATION_SECS
            cycle = 0

            while time.monotonic() < test_deadline:
                cycle += 1
                cycle_start = time.monotonic()
                report.total_iterations += 1

                try:
                    # Make concurrent calls
                    call_metrics = _run(
                        gen.make_concurrent_calls(
                            pairs,
                            duration_secs=STABILITY_CYCLE_HOLD_SECS,
                            send_rtp=True,
                        )
                    )

                    successful = [m for m in call_metrics if m.success]
                    failed = [m for m in call_metrics if not m.success]
                    cycle_elapsed = time.monotonic() - cycle_start

                    if len(successful) > 0:
                        report.successful_iterations += 1
                    else:
                        report.failed_iterations += 1

                    # Record metrics for this cycle
                    report.add_sample("cycle_duration_secs", cycle_elapsed)
                    report.add_sample("calls_succeeded", len(successful))
                    report.add_sample("calls_failed", len(failed))

                    for m in successful:
                        report.add_sample("setup_time_secs", m.setup_time)
                        if m.rtp_packets_sent > 0:
                            report.add_sample("rtp_packets_sent", m.rtp_packets_sent)

                    for m in failed:
                        err_msg = f"{m.caller}->{m.callee}: {m.error}"
                        report.errors.append(err_msg)

                    # Progress logging every few cycles
                    elapsed_total = time.monotonic() - test_start
                    remaining = test_deadline - time.monotonic()
                    print(
                        f"  Cycle {cycle}: "
                        f"{len(successful)}/{len(call_metrics)} ok, "
                        f"cycle={cycle_elapsed:.1f}s, "
                        f"elapsed={elapsed_total:.0f}s, "
                        f"remaining={remaining:.0f}s"
                    )

                except Exception as exc:
                    report.failed_iterations += 1
                    report.errors.append(f"Cycle {cycle} exception: {exc}")
                    logger.warning(f"Cycle {cycle} failed: {exc}")

                # Brief pause between cycles to let server clean up
                time.sleep(2)

            report.duration_secs = time.monotonic() - test_start

        finally:
            gen.stop_callee_handlers()
            _run(gen.unregister_all())
            gen._close_all()

        # Print summary
        report.print_report()

        # Assertions
        assert report.total_iterations > 0, "No cycles completed"

        success_rate = (
            report.successful_iterations / report.total_iterations
        )
        assert success_rate >= 0.60, (
            f"Success rate {success_rate:.1%} below 60% threshold "
            f"({report.successful_iterations}/{report.total_iterations})"
        )

        # Check for degradation: compare first quarter vs last quarter
        setup_times = report.samples.get("setup_time_secs", [])
        if len(setup_times) >= 8:
            quarter = len(setup_times) // 4
            first_q_avg = statistics.mean(setup_times[:quarter])
            last_q_avg = statistics.mean(setup_times[-quarter:])
            if first_q_avg > 0:
                degradation_ratio = last_q_avg / first_q_avg
                print(
                    f"\n  Setup time degradation: "
                    f"first_q_avg={first_q_avg:.4f}s, "
                    f"last_q_avg={last_q_avg:.4f}s, "
                    f"ratio={degradation_ratio:.2f}x"
                )
                assert degradation_ratio < 3.0, (
                    f"Call setup time degraded {degradation_ratio:.1f}x over "
                    f"the test duration (first_q={first_q_avg:.4f}s, "
                    f"last_q={last_q_avg:.4f}s)"
                )

    # ------------------------------------------------------------------
    # TC-L12-002: Memory stability
    # ------------------------------------------------------------------
    @pytest.mark.timeout(STABILITY_DURATION_SECS + 120)
    def test_memory_stability(self):
        """TC-L12-002: Monitor server memory during sustained call load.

        Samples server memory at regular intervals while maintaining a
        constant call load. After an initial ramp period, fails if memory
        grows more than MEMORY_GROWTH_THRESHOLD_PCT (50%) above the
        post-ramp baseline.
        """
        report = StabilityReport(test_name="Memory Stability")

        # Sample interval -- check memory every 10 seconds
        sample_interval = 10

        print(f"\n--- TC-L12-002: Memory Stability ---")
        print(f"  Duration:        {STABILITY_DURATION_SECS}s")
        print(f"  Sample interval: {sample_interval}s")
        print(f"  Growth limit:    {MEMORY_GROWTH_THRESHOLD_PCT}%")

        gen = _create_generator(base_port=BASE_PORT + 200)
        memory_samples: List[TimeSample] = []

        try:
            # Baseline memory before any load
            baseline_mem = _get_memory_mb()
            print(f"  Pre-load memory: {baseline_mem} MB")

            # Register and start call handlers
            success_count, _ = _run(gen.register_all())
            assert success_count >= NUM_AGENTS - 1, (
                f"Need at least {NUM_AGENTS - 1} agents registered, "
                f"got {success_count}"
            )
            _run(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(STABILITY_CALL_COUNT)]
            test_start = time.monotonic()
            test_deadline = test_start + STABILITY_DURATION_SECS

            # Ramp period -- first 10% of duration or 30s, whichever is less
            ramp_duration = min(STABILITY_DURATION_SECS * 0.1, 30)
            ramp_end = test_start + ramp_duration
            post_ramp_baseline = None

            cycle = 0
            last_sample_time = 0.0

            while time.monotonic() < test_deadline:
                cycle += 1
                report.total_iterations += 1

                # Hold time for this cycle -- shorter than full hold to
                # allow frequent memory sampling
                hold_secs = min(STABILITY_CYCLE_HOLD_SECS, sample_interval)

                try:
                    call_metrics = _run(
                        gen.make_concurrent_calls(
                            pairs,
                            duration_secs=hold_secs,
                            send_rtp=True,
                        )
                    )

                    successful = [m for m in call_metrics if m.success]
                    if len(successful) > 0:
                        report.successful_iterations += 1
                    else:
                        report.failed_iterations += 1

                    for m in successful:
                        report.add_sample("setup_time_secs", m.setup_time)

                except Exception as exc:
                    report.failed_iterations += 1
                    report.errors.append(f"Cycle {cycle}: {exc}")

                # Sample memory
                now = time.monotonic()
                if now - last_sample_time >= sample_interval:
                    mem = _get_memory_mb()
                    if mem is not None:
                        sample = TimeSample(
                            timestamp=now,
                            wall_time=time.time(),
                            value=mem,
                        )
                        memory_samples.append(sample)
                        report.add_sample("memory_mb", mem)

                        # Capture post-ramp baseline
                        if post_ramp_baseline is None and now >= ramp_end:
                            post_ramp_baseline = mem
                            print(
                                f"  Post-ramp baseline: {post_ramp_baseline:.1f} MB "
                                f"(at {now - test_start:.0f}s)"
                            )

                    last_sample_time = now

                # Progress
                elapsed = now - test_start
                remaining = test_deadline - now
                mem_str = f"{memory_samples[-1].value:.1f}" if memory_samples else "?"
                print(
                    f"  Cycle {cycle}: "
                    f"mem={mem_str}MB, "
                    f"elapsed={elapsed:.0f}s, "
                    f"remaining={remaining:.0f}s"
                )

                time.sleep(1)

            report.duration_secs = time.monotonic() - test_start

        finally:
            gen.stop_callee_handlers()
            _run(gen.unregister_all())
            gen._close_all()

        # Print summary
        report.print_report()

        # Analysis -- check memory growth
        if not memory_samples:
            print("  WARNING: No memory samples collected (endpoint may not "
                  "report memory). Skipping memory growth assertion.")
            return

        if post_ramp_baseline is None and memory_samples:
            # If ramp period covered entire test, use first sample
            post_ramp_baseline = memory_samples[0].value

        if post_ramp_baseline is not None and post_ramp_baseline > 0:
            peak_mem = max(s.value for s in memory_samples)
            final_mem = memory_samples[-1].value
            growth_pct = ((peak_mem - post_ramp_baseline) / post_ramp_baseline) * 100

            print(f"\n  Memory Analysis:")
            print(f"    Post-ramp baseline: {post_ramp_baseline:.1f} MB")
            print(f"    Peak memory:        {peak_mem:.1f} MB")
            print(f"    Final memory:       {final_mem:.1f} MB")
            print(f"    Peak growth:        {growth_pct:.1f}%")

            assert growth_pct < MEMORY_GROWTH_THRESHOLD_PCT, (
                f"Memory grew {growth_pct:.1f}% above post-ramp baseline "
                f"(limit: {MEMORY_GROWTH_THRESHOLD_PCT}%). "
                f"Baseline={post_ramp_baseline:.1f}MB, "
                f"Peak={peak_mem:.1f}MB"
            )

    # ------------------------------------------------------------------
    # TC-L12-003: Registration churn
    # ------------------------------------------------------------------
    @pytest.mark.timeout(STABILITY_DURATION_SECS + 120)
    def test_registration_churn(self):
        """TC-L12-003: Continuously register and unregister extensions.

        Creates a pool of agents and repeatedly registers/unregisters them
        for the full test duration. After each cycle, verifies the server
        registration count returns to baseline (no leaked registrations).
        """
        report = StabilityReport(test_name="Registration Churn")

        # Use a separate extension range and port range
        churn_base_ext = 4001
        churn_num_agents = 10
        churn_base_port = BASE_PORT + 400

        print(f"\n--- TC-L12-003: Registration Churn ---")
        print(f"  Duration:   {STABILITY_DURATION_SECS}s")
        print(f"  Agents:     {churn_num_agents}")
        print(f"  Extensions: {churn_base_ext}-{churn_base_ext + churn_num_agents - 1}")

        test_start = time.monotonic()
        test_deadline = test_start + STABILITY_DURATION_SECS

        # Get baseline registration count
        baseline_regs = _get_registration_count()
        print(f"  Baseline registrations: {baseline_regs}")

        cycle = 0
        while time.monotonic() < test_deadline:
            cycle += 1
            report.total_iterations += 1
            cycle_start = time.monotonic()

            gen = _create_generator(
                base_port=churn_base_port,
                num_agents=churn_num_agents,
                base_extension=churn_base_ext,
            )

            try:
                # Register all agents
                reg_start = time.monotonic()
                success_count, fail_count = _run(gen.register_all())
                reg_elapsed = time.monotonic() - reg_start
                report.add_sample("register_time_secs", reg_elapsed)
                report.add_sample("register_success_count", success_count)

                if success_count < churn_num_agents:
                    report.add_sample("register_fail_count", fail_count)

                # Brief hold to let registrations settle
                time.sleep(1)

                # Verify registrations are active
                current_regs = _get_registration_count()
                if current_regs >= 0 and baseline_regs >= 0:
                    new_regs = current_regs - baseline_regs
                    report.add_sample("active_registrations_above_baseline", new_regs)

                # Unregister all
                unreg_start = time.monotonic()
                _run(gen.unregister_all())
                unreg_elapsed = time.monotonic() - unreg_start
                report.add_sample("unregister_time_secs", unreg_elapsed)

                # Wait for server to process unregistrations
                time.sleep(1)

                # Check for registration leaks
                post_unreg_regs = _get_registration_count()
                if post_unreg_regs >= 0 and baseline_regs >= 0:
                    leaked = post_unreg_regs - baseline_regs
                    report.add_sample("leaked_registrations", leaked)
                    if leaked > 2:
                        report.errors.append(
                            f"Cycle {cycle}: {leaked} leaked registrations "
                            f"(baseline={baseline_regs}, "
                            f"after_unreg={post_unreg_regs})"
                        )

                cycle_elapsed = time.monotonic() - cycle_start
                report.add_sample("cycle_duration_secs", cycle_elapsed)
                report.successful_iterations += 1

                # Progress
                elapsed = time.monotonic() - test_start
                remaining = test_deadline - time.monotonic()
                print(
                    f"  Cycle {cycle}: "
                    f"reg={success_count}/{churn_num_agents}, "
                    f"reg_time={reg_elapsed:.2f}s, "
                    f"unreg_time={unreg_elapsed:.2f}s, "
                    f"elapsed={elapsed:.0f}s, "
                    f"remaining={remaining:.0f}s"
                )

            except Exception as exc:
                report.failed_iterations += 1
                report.errors.append(f"Cycle {cycle}: {exc}")
                logger.warning(f"Registration churn cycle {cycle} failed: {exc}")

            finally:
                gen._close_all()

            # Brief pause between cycles
            time.sleep(1)

        report.duration_secs = time.monotonic() - test_start
        report.print_report()

        # Assertions
        assert report.total_iterations > 0, "No churn cycles completed"

        success_rate = (
            report.successful_iterations / report.total_iterations
        )
        assert success_rate >= 0.80, (
            f"Registration churn success rate {success_rate:.1%} below 80% "
            f"({report.successful_iterations}/{report.total_iterations})"
        )

        # Check for persistent registration leaks
        leaked_samples = report.samples.get("leaked_registrations", [])
        if leaked_samples:
            # The last few samples should show no significant leaks
            tail_samples = leaked_samples[-min(5, len(leaked_samples)):]
            max_leaked = max(tail_samples) if tail_samples else 0
            print(f"\n  Final leak check: max leaked in last 5 cycles = {max_leaked}")
            assert max_leaked <= 3, (
                f"Persistent registration leak detected: {max_leaked} "
                f"registrations above baseline in final cycles"
            )

        # Check for degradation in registration time
        reg_times = report.samples.get("register_time_secs", [])
        if len(reg_times) >= 6:
            third = len(reg_times) // 3
            first_third = statistics.mean(reg_times[:third])
            last_third = statistics.mean(reg_times[-third:])
            if first_third > 0:
                ratio = last_third / first_third
                print(
                    f"  Registration time ratio (last/first third): "
                    f"{ratio:.2f}x "
                    f"(first={first_third:.3f}s, last={last_third:.3f}s)"
                )
                assert ratio < 5.0, (
                    f"Registration time degraded {ratio:.1f}x "
                    f"(first_third={first_third:.3f}s, "
                    f"last_third={last_third:.3f}s)"
                )

    # ------------------------------------------------------------------
    # TC-L12-004: Sustained concurrent calls
    # ------------------------------------------------------------------
    @pytest.mark.timeout(STABILITY_DURATION_SECS + 120)
    def test_sustained_concurrent_calls(self):
        """TC-L12-004: Maintain STABILITY_CALL_COUNT concurrent calls continuously.

        Keeps N calls active for the full test duration, replacing each call
        as it finishes its hold period. Tracks setup time and success rate
        per replacement cycle. Fails if setup time degrades significantly
        or success rate drops below threshold.
        """
        report = StabilityReport(test_name="Sustained Concurrent Calls")

        print(f"\n--- TC-L12-004: Sustained Concurrent Calls ---")
        print(f"  Duration:         {STABILITY_DURATION_SECS}s")
        print(f"  Target concurrent: {STABILITY_CALL_COUNT}")
        print(f"  Hold per cycle:   {STABILITY_CYCLE_HOLD_SECS}s")

        gen = _create_generator(base_port=BASE_PORT + 600)

        try:
            # Register all agents
            success_count, _ = _run(gen.register_all())
            assert success_count >= NUM_AGENTS - 1, (
                f"Need at least {NUM_AGENTS - 1} agents registered, "
                f"got {success_count}"
            )
            _run(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(STABILITY_CALL_COUNT)]
            test_start = time.monotonic()
            test_deadline = test_start + STABILITY_DURATION_SECS
            cycle = 0

            while time.monotonic() < test_deadline:
                cycle += 1
                report.total_iterations += 1
                cycle_start = time.monotonic()

                # Determine hold time -- shorten if near deadline
                remaining = test_deadline - time.monotonic()
                hold_secs = min(STABILITY_CYCLE_HOLD_SECS, max(remaining - 5, 2))
                if hold_secs < 2:
                    break  # Not enough time for another cycle

                try:
                    call_metrics = _run(
                        gen.make_concurrent_calls(
                            pairs,
                            duration_secs=hold_secs,
                            send_rtp=True,
                        )
                    )

                    successful = [m for m in call_metrics if m.success]
                    failed = [m for m in call_metrics if not m.success]
                    cycle_elapsed = time.monotonic() - cycle_start

                    report.add_sample("cycle_duration_secs", cycle_elapsed)
                    report.add_sample("calls_succeeded", len(successful))
                    report.add_sample("calls_failed", len(failed))
                    report.add_sample(
                        "success_rate_pct",
                        len(successful) / len(call_metrics) * 100
                        if call_metrics else 0,
                    )

                    for m in successful:
                        report.add_sample("setup_time_secs", m.setup_time)
                        report.add_sample("rtp_packets_sent", m.rtp_packets_sent)

                    for m in failed:
                        report.errors.append(
                            f"Cycle {cycle}: {m.caller}->{m.callee}: {m.error}"
                        )

                    if len(successful) > 0:
                        report.successful_iterations += 1
                    else:
                        report.failed_iterations += 1

                    # Progress
                    elapsed = time.monotonic() - test_start
                    rem = test_deadline - time.monotonic()
                    avg_setup = (
                        statistics.mean([m.setup_time for m in successful])
                        if successful else 0
                    )
                    print(
                        f"  Cycle {cycle}: "
                        f"{len(successful)}/{STABILITY_CALL_COUNT} ok, "
                        f"avg_setup={avg_setup:.3f}s, "
                        f"elapsed={elapsed:.0f}s, "
                        f"remaining={rem:.0f}s"
                    )

                except Exception as exc:
                    report.failed_iterations += 1
                    report.errors.append(f"Cycle {cycle}: {exc}")
                    logger.warning(f"Sustained call cycle {cycle} failed: {exc}")

                # Brief pause between replacements
                time.sleep(2)

            report.duration_secs = time.monotonic() - test_start

        finally:
            gen.stop_callee_handlers()
            _run(gen.unregister_all())
            gen._close_all()

        report.print_report()

        # Assertions
        assert report.total_iterations > 0, "No cycles completed"

        success_rate = (
            report.successful_iterations / report.total_iterations
        )
        assert success_rate >= 0.60, (
            f"Sustained call success rate {success_rate:.1%} below 60% "
            f"({report.successful_iterations}/{report.total_iterations})"
        )

        # Check setup time degradation
        setup_times = report.samples.get("setup_time_secs", [])
        if len(setup_times) >= 8:
            quarter = len(setup_times) // 4
            first_q = setup_times[:quarter]
            last_q = setup_times[-quarter:]
            first_avg = statistics.mean(first_q)
            last_avg = statistics.mean(last_q)
            if first_avg > 0:
                degradation = last_avg / first_avg
                print(
                    f"\n  Setup time degradation: {degradation:.2f}x "
                    f"(first_q={first_avg:.4f}s, last_q={last_avg:.4f}s)"
                )
                assert degradation < 3.0, (
                    f"Setup time degraded {degradation:.1f}x over duration "
                    f"(first_q={first_avg:.4f}s, last_q={last_avg:.4f}s)"
                )

        # Check per-cycle success rate does not trend downward
        per_cycle_rates = report.samples.get("success_rate_pct", [])
        if len(per_cycle_rates) >= 6:
            third = len(per_cycle_rates) // 3
            first_rate = statistics.mean(per_cycle_rates[:third])
            last_rate = statistics.mean(per_cycle_rates[-third:])
            if first_rate > 0:
                drop = first_rate - last_rate
                print(
                    f"  Success rate trend: "
                    f"first_third={first_rate:.1f}%, "
                    f"last_third={last_rate:.1f}%, "
                    f"drop={drop:.1f}pp"
                )
                assert drop < 30, (
                    f"Success rate dropped {drop:.1f} percentage points "
                    f"(first_third={first_rate:.1f}%, "
                    f"last_third={last_rate:.1f}%)"
                )

    # ------------------------------------------------------------------
    # TC-L12-005: API reliability under load
    # ------------------------------------------------------------------
    @pytest.mark.timeout(STABILITY_DURATION_SECS + 120)
    def test_api_reliability_under_load(self):
        """TC-L12-005: Poll health endpoints during sustained call load.

        While maintaining continuous call load, hits /health and
        /ami/v1/health endpoints every 5 seconds. Tracks response times
        and failure counts. Fails if more than 10% of health checks fail
        or if response time p95 exceeds 5 seconds.
        """
        report = StabilityReport(test_name="API Reliability Under Load")

        health_poll_interval = 5  # seconds between health checks

        print(f"\n--- TC-L12-005: API Reliability Under Load ---")
        print(f"  Duration:       {STABILITY_DURATION_SECS}s")
        print(f"  Poll interval:  {health_poll_interval}s")
        print(f"  Call load:      {STABILITY_CALL_COUNT} concurrent")

        gen = _create_generator(base_port=BASE_PORT + 800)

        health_results: List[Dict[str, Any]] = []

        try:
            # Register and start call handlers
            success_count, _ = _run(gen.register_all())
            assert success_count >= NUM_AGENTS - 1, (
                f"Need at least {NUM_AGENTS - 1} agents registered, "
                f"got {success_count}"
            )
            _run(gen.start_callee_handlers())

            pairs = [(i * 2, i * 2 + 1) for i in range(STABILITY_CALL_COUNT)]
            test_start = time.monotonic()
            test_deadline = test_start + STABILITY_DURATION_SECS

            # Strategy: run call cycles in the outer loop, poll health
            # between and during cycles using a combined async approach
            cycle = 0
            last_health_poll = 0.0

            while time.monotonic() < test_deadline:
                cycle += 1
                report.total_iterations += 1

                # Determine hold time
                remaining = test_deadline - time.monotonic()
                hold_secs = min(STABILITY_CYCLE_HOLD_SECS, max(remaining - 5, 2))
                if hold_secs < 2:
                    break

                # Poll health before starting calls
                now = time.monotonic()
                if now - last_health_poll >= health_poll_interval:
                    self._poll_health_endpoints(report, health_results)
                    last_health_poll = now

                try:
                    # Run calls with integrated health polling
                    call_metrics = _run(
                        self._calls_with_health_polling(
                            gen, pairs, hold_secs, report,
                            health_results, health_poll_interval,
                        )
                    )

                    successful = [m for m in call_metrics if m.success]
                    failed = [m for m in call_metrics if not m.success]

                    if len(successful) > 0:
                        report.successful_iterations += 1
                    else:
                        report.failed_iterations += 1

                    for m in successful:
                        report.add_sample("call_setup_time_secs", m.setup_time)

                    for m in failed:
                        report.errors.append(
                            f"Call {m.caller}->{m.callee}: {m.error}"
                        )

                except Exception as exc:
                    report.failed_iterations += 1
                    report.errors.append(f"Cycle {cycle}: {exc}")

                # Progress
                elapsed = time.monotonic() - test_start
                rem = test_deadline - time.monotonic()
                api_checks = len(health_results)
                api_ok = sum(1 for r in health_results if r.get("ok"))
                print(
                    f"  Cycle {cycle}: "
                    f"api_checks={api_checks} ({api_ok} ok), "
                    f"elapsed={elapsed:.0f}s, "
                    f"remaining={rem:.0f}s"
                )

                time.sleep(2)

            # Final health poll after load
            self._poll_health_endpoints(report, health_results)

            report.duration_secs = time.monotonic() - test_start

        finally:
            gen.stop_callee_handlers()
            _run(gen.unregister_all())
            gen._close_all()

        report.print_report()

        # Analyse health check results
        total_checks = len(health_results)
        ok_checks = sum(1 for r in health_results if r.get("ok"))
        failed_checks = total_checks - ok_checks

        print(f"\n  Health Check Summary:")
        print(f"    Total checks:  {total_checks}")
        print(f"    Successful:    {ok_checks}")
        print(f"    Failed:        {failed_checks}")

        if total_checks > 0:
            failure_rate = failed_checks / total_checks
            print(f"    Failure rate:  {failure_rate:.1%}")

            assert failure_rate <= 0.10, (
                f"Health endpoint failure rate {failure_rate:.1%} exceeds "
                f"10% threshold ({failed_checks}/{total_checks} failed)"
            )

        # Check response time p95
        resp_times = report.samples.get("health_response_time_ms", [])
        if resp_times:
            sorted_rt = sorted(resp_times)
            p95_idx = min(int(len(sorted_rt) * 0.95), len(sorted_rt) - 1)
            p95_ms = sorted_rt[p95_idx]
            avg_ms = statistics.mean(resp_times)
            print(f"    Avg response:  {avg_ms:.1f}ms")
            print(f"    p95 response:  {p95_ms:.1f}ms")

            assert p95_ms < 5000, (
                f"Health endpoint p95 response time {p95_ms:.1f}ms exceeds "
                f"5000ms threshold"
            )

    # ------------------------------------------------------------------
    # Helpers for TC-L12-005
    # ------------------------------------------------------------------

    def _poll_health_endpoints(self, report: StabilityReport,
                               results: List[Dict[str, Any]]):
        """Poll both /health and /ami/v1/health endpoints."""
        endpoints = [
            ("health", f"{BASE_URL}/health"),
            ("ami_health", f"{BASE_URL}/ami/v1/health"),
        ]
        for name, url in endpoints:
            status, elapsed_ms, error = _timed_health_request(url)
            ok = (status == 200 and error is None)
            results.append({
                "endpoint": name,
                "status": status,
                "response_time_ms": elapsed_ms,
                "error": error,
                "ok": ok,
                "timestamp": time.monotonic(),
            })
            report.add_sample("health_response_time_ms", elapsed_ms)
            if not ok:
                # Non-200 is not necessarily an error for /health (may not exist),
                # only track actual failures
                if error or status >= 500:
                    report.add_sample("health_failures", 1)
                    report.errors.append(
                        f"Health check {name}: status={status}, error={error}"
                    )

    async def _calls_with_health_polling(
        self, gen: SIPLoadGenerator,
        pairs: list, hold_secs: float,
        report: StabilityReport,
        health_results: List[Dict[str, Any]],
        poll_interval: float,
    ) -> List[CallMetrics]:
        """Run concurrent calls while polling health endpoints in parallel."""
        # Start call task
        call_task = asyncio.create_task(
            gen.make_concurrent_calls(
                pairs,
                duration_secs=hold_secs,
                send_rtp=True,
            )
        )

        # Poll health while calls are running
        poll_deadline = asyncio.get_running_loop().time() + hold_secs
        while asyncio.get_running_loop().time() < poll_deadline:
            if call_task.done():
                break
            await asyncio.sleep(poll_interval)
            # Health polling is synchronous (requests library), run in executor
            loop = asyncio.get_running_loop()
            await loop.run_in_executor(
                None,
                lambda: self._poll_health_endpoints(report, health_results),
            )

        # Wait for calls to complete
        call_metrics = await call_task
        return call_metrics
