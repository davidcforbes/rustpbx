#!/usr/bin/env python3
"""
RustPBX Performance Dashboard Generator

Generates a self-contained HTML report visualizing load test results from the
RustPBX test suite (L10, L11, L13, L14 concurrent call tests).

Data sources:
  1. JSON result files from pytest-json-report or custom JSON output
  2. pytest JUnit XML output parsed into structured data
  3. Built-in sample data for demonstration when no results are available

Usage:
  python tests/performance_dashboard.py [--output report.html] [--results-dir tests/results/]

  If no --results-dir is given or the directory is empty, generates a sample
  report with realistic placeholder data for layout verification.

The generated HTML report is fully self-contained with embedded CSS and
inline JavaScript using Chart.js from CDN.  No local dependencies are needed
to view the report in a browser.
"""

import argparse
import json
import os
import sys
import glob
import statistics
from dataclasses import dataclass, field, asdict
from datetime import datetime
from typing import Any, Dict, List, Optional


# ---------------------------------------------------------------------------
# Data structures for dashboard metrics
# ---------------------------------------------------------------------------

@dataclass
class TierResult:
    """Results for a single concurrency tier (e.g. 10, 25, 50, 100 calls)."""
    tier: int = 0                         # number of concurrent calls
    total_calls: int = 0
    successful_calls: int = 0
    failed_calls: int = 0
    success_rate: float = 0.0             # 0.0 - 1.0
    avg_setup_time: float = 0.0           # seconds
    p50_setup_time: float = 0.0
    p90_setup_time: float = 0.0
    p95_setup_time: float = 0.0
    p99_setup_time: float = 0.0
    min_setup_time: float = 0.0
    max_setup_time: float = 0.0
    packet_loss_pct: float = 0.0
    rtp_packets_sent: int = 0
    rtp_packets_expected: int = 0
    memory_before_mb: float = -1.0
    memory_peak_mb: float = -1.0
    memory_after_mb: float = -1.0
    cpu_peak_pct: float = -1.0
    cpu_avg_pct: float = -1.0
    health_responsive: int = 0            # number of responsive health polls
    health_total: int = 0                 # total health poll attempts
    test_cases: List[Dict[str, Any]] = field(default_factory=list)
    # test_cases: list of {"name": str, "passed": bool, "duration": float, "error": str}


@dataclass
class ResourceSample:
    """A single CPU/memory sample taken during load testing."""
    time_offset: float = 0.0             # seconds from test start
    cpu_pct: float = -1.0
    memory_mb: float = -1.0
    concurrent_calls: int = 0
    tier: int = 0


@dataclass
class DashboardData:
    """All data needed to render the performance dashboard."""
    generated_at: str = ""
    server_host: str = ""
    server_info: str = ""
    tiers: List[TierResult] = field(default_factory=list)
    resource_samples: List[ResourceSample] = field(default_factory=list)
    mos_scores: List[float] = field(default_factory=list)
    latency_buckets: Dict[str, int] = field(default_factory=dict)

    def to_dict(self) -> dict:
        """Serialize to a plain dict for JSON embedding in HTML."""
        return {
            "generated_at": self.generated_at,
            "server_host": self.server_host,
            "server_info": self.server_info,
            "tiers": [asdict(t) for t in self.tiers],
            "resource_samples": [asdict(s) for s in self.resource_samples],
            "mos_scores": self.mos_scores,
            "latency_buckets": self.latency_buckets,
        }


# ---------------------------------------------------------------------------
# Percentile helper (matching test suite implementation)
# ---------------------------------------------------------------------------

def _percentile(sorted_data: list, pct: int) -> float:
    """Compute a percentile from a pre-sorted list."""
    if not sorted_data:
        return 0.0
    idx = int(len(sorted_data) * pct / 100)
    idx = min(idx, len(sorted_data) - 1)
    return sorted_data[idx]


# ---------------------------------------------------------------------------
# Parse JSON result files
# ---------------------------------------------------------------------------

def load_results_from_directory(results_dir: str) -> Optional[DashboardData]:
    """Load and parse JSON result files from a directory.

    Supports two JSON formats:

    1. Per-tier result files named like:
         tier_10.json, tier_25.json, tier_50.json, tier_100.json
       Each containing call-level metrics.

    2. A single combined result file named:
         load_test_results.json
       Containing a "tiers" array with all tier data.

    3. pytest-json-report output (report.json) with test outcomes.

    Returns None if no parseable files are found.
    """
    if not os.path.isdir(results_dir):
        return None

    dashboard = DashboardData(
        generated_at=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        server_host=os.environ.get("RUSTPBX_HOST", "unknown"),
    )

    # Try combined result file first
    combined_path = os.path.join(results_dir, "load_test_results.json")
    if os.path.isfile(combined_path):
        with open(combined_path, "r") as f:
            data = json.load(f)
        return _parse_combined_results(data, dashboard)

    # Try per-tier files
    tier_files = sorted(glob.glob(os.path.join(results_dir, "tier_*.json")))
    if tier_files:
        for tf in tier_files:
            with open(tf, "r") as f:
                tier_data = json.load(f)
            tier_result = _parse_tier_json(tier_data)
            if tier_result:
                dashboard.tiers.append(tier_result)
        if dashboard.tiers:
            _compute_derived_data(dashboard)
            return dashboard

    # Try pytest-json-report format
    report_path = os.path.join(results_dir, "report.json")
    if os.path.isfile(report_path):
        with open(report_path, "r") as f:
            data = json.load(f)
        return _parse_pytest_report(data, dashboard)

    # Try any .json files in the directory
    json_files = sorted(glob.glob(os.path.join(results_dir, "*.json")))
    for jf in json_files:
        try:
            with open(jf, "r") as f:
                data = json.load(f)
            if isinstance(data, dict) and "tiers" in data:
                return _parse_combined_results(data, dashboard)
            if isinstance(data, dict) and "tier" in data:
                tier_result = _parse_tier_json(data)
                if tier_result:
                    dashboard.tiers.append(tier_result)
        except (json.JSONDecodeError, KeyError):
            continue

    if dashboard.tiers:
        _compute_derived_data(dashboard)
        return dashboard

    return None


def _parse_combined_results(data: dict, dashboard: DashboardData) -> DashboardData:
    """Parse a combined load_test_results.json file."""
    dashboard.server_host = data.get("server_host", dashboard.server_host)
    dashboard.server_info = data.get("server_info", "")

    for tier_data in data.get("tiers", []):
        tier_result = _parse_tier_json(tier_data)
        if tier_result:
            dashboard.tiers.append(tier_result)

    for sample_data in data.get("resource_samples", []):
        dashboard.resource_samples.append(ResourceSample(
            time_offset=sample_data.get("time_offset", 0.0),
            cpu_pct=sample_data.get("cpu_pct", -1.0),
            memory_mb=sample_data.get("memory_mb", -1.0),
            concurrent_calls=sample_data.get("concurrent_calls", 0),
            tier=sample_data.get("tier", 0),
        ))

    dashboard.mos_scores = data.get("mos_scores", [])
    dashboard.latency_buckets = data.get("latency_buckets", {})

    _compute_derived_data(dashboard)
    return dashboard


def _parse_tier_json(data: dict) -> Optional[TierResult]:
    """Parse a single tier result from JSON."""
    tier = data.get("tier", data.get("concurrent_calls", 0))
    if not tier:
        return None

    total = data.get("total_calls", 0)
    successful = data.get("successful_calls", 0)
    failed = data.get("failed_calls", total - successful)

    # Setup time stats
    setup_stats = data.get("setup_time_stats", {}) or {}
    setup_times_raw = data.get("setup_times", [])
    if setup_times_raw:
        sorted_times = sorted(setup_times_raw)
        setup_stats.setdefault("mean", statistics.mean(sorted_times))
        setup_stats.setdefault("median", _percentile(sorted_times, 50))
        setup_stats.setdefault("p90", _percentile(sorted_times, 90))
        setup_stats.setdefault("p95", _percentile(sorted_times, 95))
        setup_stats.setdefault("p99", _percentile(sorted_times, 99))
        setup_stats.setdefault("min", sorted_times[0])
        setup_stats.setdefault("max", sorted_times[-1])

    return TierResult(
        tier=tier,
        total_calls=total,
        successful_calls=successful,
        failed_calls=failed,
        success_rate=data.get("success_rate", successful / total if total else 0.0),
        avg_setup_time=setup_stats.get("mean", 0.0),
        p50_setup_time=setup_stats.get("median", setup_stats.get("p50", 0.0)),
        p90_setup_time=setup_stats.get("p90", 0.0),
        p95_setup_time=setup_stats.get("p95", 0.0),
        p99_setup_time=setup_stats.get("p99", 0.0),
        min_setup_time=setup_stats.get("min", 0.0),
        max_setup_time=setup_stats.get("max", 0.0),
        packet_loss_pct=data.get("packet_loss_pct", 0.0),
        rtp_packets_sent=data.get("rtp_packets_sent", 0),
        rtp_packets_expected=data.get("rtp_packets_expected", 0),
        memory_before_mb=data.get("memory_before_mb", -1.0),
        memory_peak_mb=data.get("memory_peak_mb", -1.0),
        memory_after_mb=data.get("memory_after_mb", -1.0),
        cpu_peak_pct=data.get("cpu_peak_pct", -1.0),
        cpu_avg_pct=data.get("cpu_avg_pct", -1.0),
        health_responsive=data.get("health_responsive", 0),
        health_total=data.get("health_total", 0),
        test_cases=data.get("test_cases", []),
    )


def _parse_pytest_report(data: dict, dashboard: DashboardData) -> DashboardData:
    """Parse pytest-json-report format into dashboard data.

    This format contains test outcomes and durations, but not the detailed
    per-call metrics.  We extract what we can and fill in placeholders.
    """
    tests = data.get("tests", [])

    # Group tests by tier
    tier_tests: Dict[int, List[dict]] = {}
    tier_map = {
        "L10": 10, "L11": 25, "L13": 50, "L14": 100,
    }

    for test in tests:
        nodeid = test.get("nodeid", "")
        for prefix, tier_num in tier_map.items():
            if f"test_{prefix}" in nodeid:
                tier_tests.setdefault(tier_num, []).append(test)
                break

    for tier_num in sorted(tier_tests.keys()):
        tests_in_tier = tier_tests[tier_num]
        passed = sum(1 for t in tests_in_tier if t.get("outcome") == "passed")
        failed = sum(1 for t in tests_in_tier if t.get("outcome") == "failed")
        skipped = sum(1 for t in tests_in_tier if t.get("outcome") == "skipped")

        test_cases = []
        for t in tests_in_tier:
            name = t.get("nodeid", "").split("::")[-1] if "::" in t.get("nodeid", "") else t.get("nodeid", "")
            test_cases.append({
                "name": name,
                "passed": t.get("outcome") == "passed",
                "duration": t.get("duration", 0.0),
                "error": t.get("longrepr", "") if t.get("outcome") == "failed" else "",
            })

        dashboard.tiers.append(TierResult(
            tier=tier_num,
            total_calls=tier_num,
            successful_calls=tier_num if passed > 0 else 0,
            failed_calls=0 if passed > 0 else tier_num,
            success_rate=1.0 if passed == len(tests_in_tier) else passed / max(len(tests_in_tier), 1),
            test_cases=test_cases,
        ))

    _compute_derived_data(dashboard)
    return dashboard


def _compute_derived_data(dashboard: DashboardData):
    """Fill in derived fields and latency buckets from tier data."""
    # Build latency buckets from all tiers
    all_setup_times = []
    for tier in dashboard.tiers:
        # Synthesize approximate setup times from percentiles for bucketing
        if tier.avg_setup_time > 0:
            # Generate approximate distribution based on percentiles
            count = tier.successful_calls if tier.successful_calls > 0 else tier.total_calls
            for _ in range(max(1, count)):
                # Use percentile-informed random distribution for bucketing
                all_setup_times.append(tier.avg_setup_time)

    if all_setup_times and not dashboard.latency_buckets:
        buckets = {
            "< 0.5s": 0, "0.5-1s": 0, "1-2s": 0, "2-5s": 0,
            "5-10s": 0, "10-15s": 0, "15-20s": 0, "> 20s": 0,
        }
        for t in all_setup_times:
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
        dashboard.latency_buckets = buckets


# ---------------------------------------------------------------------------
# Sample data generator (for demo/layout verification)
# ---------------------------------------------------------------------------

def generate_sample_data() -> DashboardData:
    """Generate realistic sample data for dashboard demonstration.

    The sample data models a typical RustPBX load test progression from
    10 to 100 concurrent calls, showing realistic degradation patterns
    at higher concurrency levels.
    """
    dashboard = DashboardData(
        generated_at=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        server_host="74.207.251.126",
        server_info="RustPBX v0.3.18 | Linode 4GB | Ubuntu 22.04 | 2 vCPU | Rust 1.76",
    )

    # Tier 10: 10 concurrent calls -- all healthy
    dashboard.tiers.append(TierResult(
        tier=10,
        total_calls=10,
        successful_calls=10,
        failed_calls=0,
        success_rate=1.0,
        avg_setup_time=0.342,
        p50_setup_time=0.298,
        p90_setup_time=0.487,
        p95_setup_time=0.521,
        p99_setup_time=0.589,
        min_setup_time=0.187,
        max_setup_time=0.601,
        packet_loss_pct=0.12,
        rtp_packets_sent=2480,
        rtp_packets_expected=2500,
        memory_before_mb=42.3,
        memory_peak_mb=58.7,
        memory_after_mb=44.1,
        cpu_peak_pct=12.4,
        cpu_avg_pct=8.2,
        health_responsive=5,
        health_total=5,
        test_cases=[
            {"name": "test_10_concurrent_registrations", "passed": True, "duration": 3.2, "error": ""},
            {"name": "test_10_concurrent_calls", "passed": True, "duration": 12.8, "error": ""},
            {"name": "test_concurrent_call_rtp_flow", "passed": True, "duration": 15.1, "error": ""},
            {"name": "test_concurrent_call_packet_loss", "passed": True, "duration": 14.7, "error": ""},
            {"name": "test_concurrent_call_teardown", "passed": True, "duration": 9.3, "error": ""},
            {"name": "test_server_health_during_load", "passed": True, "duration": 18.4, "error": ""},
            {"name": "test_sequential_call_bursts", "passed": True, "duration": 45.2, "error": ""},
        ],
    ))

    # Tier 25: 25 concurrent calls -- very good
    dashboard.tiers.append(TierResult(
        tier=25,
        total_calls=25,
        successful_calls=25,
        failed_calls=0,
        success_rate=1.0,
        avg_setup_time=0.687,
        p50_setup_time=0.612,
        p90_setup_time=0.934,
        p95_setup_time=1.102,
        p99_setup_time=1.287,
        min_setup_time=0.289,
        max_setup_time=1.342,
        packet_loss_pct=0.34,
        rtp_packets_sent=6180,
        rtp_packets_expected=6250,
        memory_before_mb=44.1,
        memory_peak_mb=82.3,
        memory_after_mb=48.6,
        cpu_peak_pct=28.7,
        cpu_avg_pct=19.3,
        health_responsive=5,
        health_total=5,
        test_cases=[
            {"name": "test_25_concurrent_registrations", "passed": True, "duration": 5.1, "error": ""},
            {"name": "test_25_concurrent_calls", "passed": True, "duration": 18.4, "error": ""},
            {"name": "test_25_concurrent_call_rtp_flow", "passed": True, "duration": 22.1, "error": ""},
            {"name": "test_25_concurrent_call_packet_loss", "passed": True, "duration": 20.7, "error": ""},
            {"name": "test_25_concurrent_call_teardown", "passed": True, "duration": 14.3, "error": ""},
            {"name": "test_server_resource_usage_25", "passed": True, "duration": 24.6, "error": ""},
            {"name": "test_25_call_setup_latency", "passed": True, "duration": 16.2, "error": ""},
        ],
    ))

    # Tier 50: 50 concurrent calls -- some strain
    dashboard.tiers.append(TierResult(
        tier=50,
        total_calls=50,
        successful_calls=48,
        failed_calls=2,
        success_rate=0.96,
        avg_setup_time=1.523,
        p50_setup_time=1.287,
        p90_setup_time=2.341,
        p95_setup_time=2.876,
        p99_setup_time=3.412,
        min_setup_time=0.412,
        max_setup_time=3.687,
        packet_loss_pct=1.82,
        rtp_packets_sent=23640,
        rtp_packets_expected=25000,
        memory_before_mb=48.6,
        memory_peak_mb=156.2,
        memory_after_mb=58.3,
        cpu_peak_pct=62.4,
        cpu_avg_pct=45.8,
        health_responsive=5,
        health_total=5,
        test_cases=[
            {"name": "test_50_concurrent_registrations", "passed": True, "duration": 8.4, "error": ""},
            {"name": "test_50_concurrent_calls_basic", "passed": True, "duration": 42.1, "error": ""},
            {"name": "test_50_concurrent_calls_with_rtp", "passed": True, "duration": 38.7, "error": ""},
            {"name": "test_50_concurrent_calls_packet_loss", "passed": True, "duration": 35.2, "error": ""},
            {"name": "test_50_concurrent_calls_rtp_port_exhaustion", "passed": True, "duration": 28.9, "error": ""},
            {"name": "test_50_concurrent_calls_memory_baseline", "passed": True, "duration": 52.3, "error": ""},
            {"name": "test_50_concurrent_calls_cdr_writes", "passed": True, "duration": 48.1, "error": ""},
        ],
    ))

    # Tier 100: 100 concurrent calls -- under stress
    dashboard.tiers.append(TierResult(
        tier=100,
        total_calls=100,
        successful_calls=93,
        failed_calls=7,
        success_rate=0.93,
        avg_setup_time=3.187,
        p50_setup_time=2.654,
        p90_setup_time=4.987,
        p95_setup_time=6.234,
        p99_setup_time=8.412,
        min_setup_time=0.687,
        max_setup_time=11.234,
        packet_loss_pct=4.21,
        rtp_packets_sent=45280,
        rtp_packets_expected=50000,
        memory_before_mb=58.3,
        memory_peak_mb=312.7,
        memory_after_mb=78.4,
        cpu_peak_pct=87.3,
        cpu_avg_pct=72.1,
        health_responsive=10,
        health_total=12,
        test_cases=[
            {"name": "test_100_concurrent_registrations", "passed": True, "duration": 14.2, "error": ""},
            {"name": "test_100_concurrent_calls_basic", "passed": True, "duration": 68.4, "error": ""},
            {"name": "test_100_concurrent_calls_with_rtp", "passed": True, "duration": 72.1, "error": ""},
            {"name": "test_100_concurrent_calls_packet_loss", "passed": True, "duration": 65.8, "error": ""},
            {"name": "test_100_concurrent_calls_rtp_port_exhaustion", "passed": True, "duration": 52.3, "error": ""},
            {"name": "test_100_concurrent_calls_memory_baseline", "passed": True, "duration": 84.7, "error": ""},
            {"name": "test_100_concurrent_calls_cdr_writes", "passed": True, "duration": 78.2, "error": ""},
            {"name": "test_100_concurrent_calls_cpu_saturation", "passed": True, "duration": 92.4, "error": ""},
            {"name": "test_100_concurrent_calls_setup_latency", "passed": False, "duration": 88.1,
             "error": "p95 setup latency 6.234s exceeds 5.0s threshold"},
        ],
    ))

    # Resource samples (simulating health endpoint polling across all tiers)
    sample_idx = 0
    for tier_num, cpu_base, mem_base, num_samples in [
        (10, 6.0, 45.0, 5),
        (25, 15.0, 60.0, 5),
        (50, 35.0, 100.0, 8),
        (100, 55.0, 180.0, 12),
    ]:
        import random
        random.seed(42 + tier_num)  # Reproducible
        for i in range(num_samples):
            # CPU ramps up, peaks, then comes down
            progress = i / max(num_samples - 1, 1)
            ramp = 1.0 - abs(2 * progress - 1)  # triangle wave, peaks in middle
            cpu = cpu_base + ramp * tier_num * 0.4 + random.gauss(0, 2)
            mem = mem_base + ramp * tier_num * 1.2 + random.gauss(0, 3)
            dashboard.resource_samples.append(ResourceSample(
                time_offset=sample_idx * 1.5,
                cpu_pct=max(0, min(100, cpu)),
                memory_mb=max(0, mem),
                concurrent_calls=int(tier_num * ramp),
                tier=tier_num,
            ))
            sample_idx += 1

    # MOS scores distribution (simulated)
    import random
    random.seed(42)
    mos_scores = []
    for _ in range(200):
        # Most calls have good MOS (4.0-4.4), with a tail of degraded calls
        base = random.gauss(4.2, 0.3)
        # 10% chance of degraded quality
        if random.random() < 0.10:
            base -= random.uniform(0.5, 1.5)
        mos_scores.append(max(1.0, min(5.0, round(base, 2))))
    dashboard.mos_scores = mos_scores

    # Latency buckets (aggregated from all tiers)
    dashboard.latency_buckets = {
        "< 0.5s": 42,
        "0.5-1s": 68,
        "1-2s": 51,
        "2-5s": 32,
        "5-10s": 12,
        "10-15s": 3,
        "15-20s": 1,
        "> 20s": 0,
    }

    return dashboard


# ---------------------------------------------------------------------------
# pytest conftest.py fixture content for JSON metrics collection
# ---------------------------------------------------------------------------

CONFTEST_FIXTURE_CODE = '''
# --- Performance metrics collection fixture ---
# Add this to your conftest.py to save JSON metrics after each load test.
# Or save as tests/perf_conftest.py and add:  pytest_plugins = ["perf_conftest"]

import json
import os
import time
from datetime import datetime

import pytest


_perf_results = {}


@pytest.fixture(autouse=True, scope="session")
def _perf_session_setup(request):
    """Initialize performance results collection for the session."""
    _perf_results.clear()
    _perf_results["session_start"] = time.time()
    _perf_results["tiers"] = {}
    yield
    # Save results at session end
    _perf_results["session_end"] = time.time()
    results_dir = os.environ.get("PERF_RESULTS_DIR",
                                  os.path.join(os.path.dirname(__file__), "results"))
    os.makedirs(results_dir, exist_ok=True)
    output_path = os.path.join(results_dir, "load_test_results.json")
    with open(output_path, "w") as f:
        json.dump({
            "generated_at": datetime.now().isoformat(),
            "server_host": os.environ.get("RUSTPBX_HOST", "unknown"),
            "tiers": list(_perf_results["tiers"].values()),
            "resource_samples": _perf_results.get("resource_samples", []),
        }, f, indent=2, default=str)
    print(f"\\nPerformance results saved to: {output_path}")


def save_tier_metrics(tier: int, **kwargs):
    """Call from a test to save metrics for a concurrency tier.

    Example:
        save_tier_metrics(
            tier=10,
            total_calls=10,
            successful_calls=10,
            setup_times=[0.3, 0.4, ...],
            packet_loss_pct=0.5,
            memory_peak_mb=58.7,
            cpu_peak_pct=12.4,
        )
    """
    _perf_results.setdefault("tiers", {})
    _perf_results["tiers"][tier] = {"tier": tier, **kwargs}


def save_resource_sample(tier: int, time_offset: float,
                         cpu_pct: float = -1, memory_mb: float = -1,
                         concurrent_calls: int = 0):
    """Call from a test to save a CPU/memory sample."""
    _perf_results.setdefault("resource_samples", [])
    _perf_results["resource_samples"].append({
        "tier": tier,
        "time_offset": time_offset,
        "cpu_pct": cpu_pct,
        "memory_mb": memory_mb,
        "concurrent_calls": concurrent_calls,
    })
'''


# ---------------------------------------------------------------------------
# HTML report generation
# ---------------------------------------------------------------------------

def generate_html_report(data: DashboardData) -> str:
    """Generate a self-contained HTML performance dashboard.

    The report includes:
      - Summary cards (total calls, max concurrent, avg latency, peak resources)
      - Concurrent capacity chart (calls vs success rate)
      - Latency distribution (p50/p90/p95/p99 bar chart)
      - Resource usage (dual-axis CPU% and Memory MB)
      - Packet loss comparison across tiers
      - MOS/quality score histogram
      - Test results table (pass/fail for each test case)

    All charts use Chart.js loaded from CDN.  CSS is fully embedded.
    """
    data_json = json.dumps(data.to_dict(), indent=2)

    # Compute summary values
    total_calls_tested = sum(t.total_calls for t in data.tiers)
    max_concurrent = max((t.tier for t in data.tiers), default=0)
    all_avg_setups = [t.avg_setup_time for t in data.tiers if t.avg_setup_time > 0]
    overall_avg_setup = statistics.mean(all_avg_setups) if all_avg_setups else 0.0
    peak_memory = max((t.memory_peak_mb for t in data.tiers if t.memory_peak_mb >= 0), default=0.0)
    peak_cpu = max((t.cpu_peak_pct for t in data.tiers if t.cpu_peak_pct >= 0), default=0.0)
    total_passed = sum(
        sum(1 for tc in t.test_cases if tc.get("passed"))
        for t in data.tiers
    )
    total_tests = sum(len(t.test_cases) for t in data.tiers)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>RustPBX Performance Dashboard</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
<style>
  :root {{
    --bg-primary: #0f172a;
    --bg-secondary: #1e293b;
    --bg-card: #1e293b;
    --bg-card-hover: #334155;
    --text-primary: #f1f5f9;
    --text-secondary: #94a3b8;
    --text-muted: #64748b;
    --accent-blue: #3b82f6;
    --accent-green: #22c55e;
    --accent-yellow: #eab308;
    --accent-red: #ef4444;
    --accent-purple: #a855f7;
    --accent-cyan: #06b6d4;
    --border: #334155;
    --shadow: 0 4px 6px -1px rgba(0,0,0,0.3), 0 2px 4px -2px rgba(0,0,0,0.2);
  }}

  * {{ margin: 0; padding: 0; box-sizing: border-box; }}

  body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    line-height: 1.6;
    min-height: 100vh;
  }}

  /* Header */
  .header {{
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
    border-bottom: 1px solid var(--border);
    padding: 24px 32px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }}
  .header h1 {{
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 10px;
  }}
  .header h1 .logo {{
    width: 32px;
    height: 32px;
    background: var(--accent-blue);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.9rem;
    font-weight: 800;
    color: white;
  }}
  .header-meta {{
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-align: right;
  }}
  .header-meta .server {{
    font-weight: 600;
    color: var(--text-primary);
  }}

  /* Main container */
  .container {{
    max-width: 1440px;
    margin: 0 auto;
    padding: 24px;
  }}

  /* Summary cards */
  .summary-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }}
  .summary-card {{
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    box-shadow: var(--shadow);
    transition: transform 0.15s ease, border-color 0.15s ease;
  }}
  .summary-card:hover {{
    transform: translateY(-2px);
    border-color: var(--accent-blue);
  }}
  .summary-card .label {{
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: 6px;
  }}
  .summary-card .value {{
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1.2;
  }}
  .summary-card .detail {{
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-top: 4px;
  }}
  .summary-card.success .value {{ color: var(--accent-green); }}
  .summary-card.warning .value {{ color: var(--accent-yellow); }}
  .summary-card.danger .value {{ color: var(--accent-red); }}
  .summary-card.info .value {{ color: var(--accent-cyan); }}

  /* Chart sections */
  .chart-grid {{
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 20px;
    margin-bottom: 24px;
  }}
  @media (max-width: 1024px) {{
    .chart-grid {{ grid-template-columns: 1fr; }}
  }}
  .chart-card {{
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    box-shadow: var(--shadow);
  }}
  .chart-card h3 {{
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }}
  .chart-card canvas {{
    max-height: 320px;
  }}

  /* Full-width chart section */
  .chart-full {{
    grid-column: 1 / -1;
  }}

  /* Results table */
  .table-section {{
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    box-shadow: var(--shadow);
    margin-bottom: 24px;
    overflow-x: auto;
  }}
  .table-section h3 {{
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }}
  table {{
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }}
  thead th {{
    text-align: left;
    padding: 10px 12px;
    font-weight: 600;
    color: var(--text-secondary);
    border-bottom: 2px solid var(--border);
    white-space: nowrap;
  }}
  tbody td {{
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
  }}
  tbody tr:hover {{
    background: var(--bg-card-hover);
  }}
  .badge {{
    display: inline-block;
    padding: 2px 8px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 600;
  }}
  .badge-pass {{
    background: rgba(34, 197, 94, 0.15);
    color: var(--accent-green);
  }}
  .badge-fail {{
    background: rgba(239, 68, 68, 0.15);
    color: var(--accent-red);
  }}
  .badge-skip {{
    background: rgba(234, 179, 8, 0.15);
    color: var(--accent-yellow);
  }}

  /* Tier header in table */
  .tier-header {{
    background: var(--bg-primary);
    font-weight: 700;
    font-size: 0.9rem;
  }}
  .tier-header td {{
    color: var(--accent-blue);
    padding: 12px;
    border-bottom: 2px solid var(--accent-blue);
  }}

  /* Footer */
  .footer {{
    text-align: center;
    padding: 24px;
    color: var(--text-muted);
    font-size: 0.8rem;
    border-top: 1px solid var(--border);
    margin-top: 24px;
  }}

  /* Tooltip override for Chart.js */
  .chartjs-tooltip {{
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    border-radius: 8px !important;
    color: var(--text-primary) !important;
  }}
</style>
</head>
<body>

<!-- Header -->
<div class="header">
  <h1>
    <span class="logo">PBX</span>
    RustPBX Performance Dashboard
  </h1>
  <div class="header-meta">
    <div class="server">{data.server_host or 'Local'}</div>
    <div>{data.server_info or 'RustPBX Load Test Results'}</div>
    <div>Generated: {data.generated_at}</div>
  </div>
</div>

<div class="container">

  <!-- Summary Cards -->
  <div class="summary-grid">
    <div class="summary-card info">
      <div class="label">Total Calls Tested</div>
      <div class="value">{total_calls_tested}</div>
      <div class="detail">Across {len(data.tiers)} concurrency tiers</div>
    </div>
    <div class="summary-card">
      <div class="label">Max Concurrent</div>
      <div class="value">{max_concurrent}</div>
      <div class="detail">Peak simultaneous calls</div>
    </div>
    <div class="summary-card {'success' if overall_avg_setup < 2.0 else 'warning' if overall_avg_setup < 5.0 else 'danger'}">
      <div class="label">Avg Setup Latency</div>
      <div class="value">{overall_avg_setup:.2f}s</div>
      <div class="detail">Mean across all tiers</div>
    </div>
    <div class="summary-card {'success' if peak_memory < 200 else 'warning' if peak_memory < 500 else 'danger'}">
      <div class="label">Peak Memory</div>
      <div class="value">{peak_memory:.0f} MB</div>
      <div class="detail">Maximum during load</div>
    </div>
    <div class="summary-card {'success' if peak_cpu < 50 else 'warning' if peak_cpu < 80 else 'danger'}">
      <div class="label">Peak CPU</div>
      <div class="value">{peak_cpu:.0f}%</div>
      <div class="detail">Maximum utilization</div>
    </div>
    <div class="summary-card {'success' if total_passed == total_tests else 'warning' if total_passed > total_tests * 0.8 else 'danger'}">
      <div class="label">Tests Passed</div>
      <div class="value">{total_passed}/{total_tests}</div>
      <div class="detail">{'All passing' if total_passed == total_tests else f'{total_tests - total_passed} failing'}</div>
    </div>
  </div>

  <!-- Charts -->
  <div class="chart-grid">

    <!-- 1. Concurrent Capacity -->
    <div class="chart-card">
      <h3>Concurrent Capacity -- Calls vs Success Rate</h3>
      <canvas id="capacityChart"></canvas>
    </div>

    <!-- 2. Latency Percentiles -->
    <div class="chart-card">
      <h3>Call Setup Latency Percentiles</h3>
      <canvas id="latencyChart"></canvas>
    </div>

    <!-- 3. Resource Usage -->
    <div class="chart-card chart-full">
      <h3>Resource Usage Over Test Progression</h3>
      <canvas id="resourceChart"></canvas>
    </div>

    <!-- 4. Packet Loss -->
    <div class="chart-card">
      <h3>Packet Loss by Concurrency Tier</h3>
      <canvas id="packetLossChart"></canvas>
    </div>

    <!-- 5. MOS / Quality Distribution -->
    <div class="chart-card">
      <h3>Quality Score (MOS) Distribution</h3>
      <canvas id="mosChart"></canvas>
    </div>

  </div>

  <!-- Test Results Table -->
  <div class="table-section">
    <h3>Test Results by Concurrency Tier</h3>
    <table>
      <thead>
        <tr>
          <th>Test Case</th>
          <th>Status</th>
          <th>Duration</th>
          <th>Details</th>
        </tr>
      </thead>
      <tbody id="resultsTableBody">
      </tbody>
    </table>
  </div>

</div>

<div class="footer">
  RustPBX Performance Dashboard | Generated {data.generated_at} | Chart.js v4.4.1
</div>

<script>
// Embedded dashboard data
const DASHBOARD_DATA = {data_json};

// Chart.js global defaults
Chart.defaults.color = '#94a3b8';
Chart.defaults.borderColor = '#334155';
Chart.defaults.font.family = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

const COLORS = {{
  blue: '#3b82f6',
  green: '#22c55e',
  yellow: '#eab308',
  red: '#ef4444',
  purple: '#a855f7',
  cyan: '#06b6d4',
  orange: '#f97316',
  blueBg: 'rgba(59, 130, 246, 0.2)',
  greenBg: 'rgba(34, 197, 94, 0.2)',
  yellowBg: 'rgba(234, 179, 8, 0.2)',
  redBg: 'rgba(239, 68, 68, 0.2)',
  purpleBg: 'rgba(168, 85, 247, 0.2)',
  cyanBg: 'rgba(6, 182, 212, 0.2)',
}};

// ---------------------------------------------------------------------------
// 1. Concurrent Capacity Chart
// ---------------------------------------------------------------------------
(function() {{
  const tiers = DASHBOARD_DATA.tiers;
  if (!tiers.length) return;

  const labels = tiers.map(t => t.tier + ' calls');
  const successRates = tiers.map(t => (t.success_rate * 100).toFixed(1));
  const successCounts = tiers.map(t => t.successful_calls);
  const failCounts = tiers.map(t => t.failed_calls);

  new Chart(document.getElementById('capacityChart'), {{
    type: 'bar',
    data: {{
      labels: labels,
      datasets: [
        {{
          label: 'Successful',
          data: successCounts,
          backgroundColor: COLORS.greenBg,
          borderColor: COLORS.green,
          borderWidth: 2,
          borderRadius: 4,
          yAxisID: 'y',
        }},
        {{
          label: 'Failed',
          data: failCounts,
          backgroundColor: COLORS.redBg,
          borderColor: COLORS.red,
          borderWidth: 2,
          borderRadius: 4,
          yAxisID: 'y',
        }},
        {{
          label: 'Success Rate %',
          data: successRates,
          type: 'line',
          borderColor: COLORS.cyan,
          backgroundColor: COLORS.cyanBg,
          borderWidth: 3,
          pointRadius: 5,
          pointBackgroundColor: COLORS.cyan,
          tension: 0.3,
          yAxisID: 'y1',
        }},
      ],
    }},
    options: {{
      responsive: true,
      maintainAspectRatio: true,
      interaction: {{ mode: 'index', intersect: false }},
      scales: {{
        y: {{
          type: 'linear',
          position: 'left',
          title: {{ display: true, text: 'Call Count' }},
          beginAtZero: true,
          grid: {{ color: '#1e293b' }},
        }},
        y1: {{
          type: 'linear',
          position: 'right',
          title: {{ display: true, text: 'Success Rate (%)' }},
          min: 0,
          max: 105,
          grid: {{ drawOnChartArea: false }},
        }},
        x: {{
          grid: {{ color: '#1e293b' }},
        }},
      }},
      plugins: {{
        legend: {{ position: 'bottom' }},
      }},
    }},
  }});
}})();

// ---------------------------------------------------------------------------
// 2. Latency Percentiles Chart
// ---------------------------------------------------------------------------
(function() {{
  const tiers = DASHBOARD_DATA.tiers.filter(t => t.avg_setup_time > 0);
  if (!tiers.length) return;

  const labels = tiers.map(t => t.tier + ' calls');

  new Chart(document.getElementById('latencyChart'), {{
    type: 'bar',
    data: {{
      labels: labels,
      datasets: [
        {{
          label: 'p50',
          data: tiers.map(t => t.p50_setup_time.toFixed(3)),
          backgroundColor: COLORS.greenBg,
          borderColor: COLORS.green,
          borderWidth: 2,
          borderRadius: 4,
        }},
        {{
          label: 'p90',
          data: tiers.map(t => t.p90_setup_time.toFixed(3)),
          backgroundColor: COLORS.blueBg,
          borderColor: COLORS.blue,
          borderWidth: 2,
          borderRadius: 4,
        }},
        {{
          label: 'p95',
          data: tiers.map(t => t.p95_setup_time.toFixed(3)),
          backgroundColor: COLORS.yellowBg,
          borderColor: COLORS.yellow,
          borderWidth: 2,
          borderRadius: 4,
        }},
        {{
          label: 'p99',
          data: tiers.map(t => t.p99_setup_time.toFixed(3)),
          backgroundColor: COLORS.redBg,
          borderColor: COLORS.red,
          borderWidth: 2,
          borderRadius: 4,
        }},
      ],
    }},
    options: {{
      responsive: true,
      maintainAspectRatio: true,
      scales: {{
        y: {{
          title: {{ display: true, text: 'Setup Time (seconds)' }},
          beginAtZero: true,
          grid: {{ color: '#1e293b' }},
        }},
        x: {{
          grid: {{ color: '#1e293b' }},
        }},
      }},
      plugins: {{
        legend: {{ position: 'bottom' }},
      }},
    }},
  }});
}})();

// ---------------------------------------------------------------------------
// 3. Resource Usage Chart (dual axis: CPU + Memory over time)
// ---------------------------------------------------------------------------
(function() {{
  const samples = DASHBOARD_DATA.resource_samples;
  if (!samples.length) return;

  // Use sample index as x-axis, annotate with tier info
  const labels = samples.map((s, i) => {{
    if (i === 0 || samples[i-1].tier !== s.tier) {{
      return s.tier + '-call (t+' + s.time_offset.toFixed(0) + 's)';
    }}
    return 't+' + s.time_offset.toFixed(0) + 's';
  }});

  const cpuData = samples.map(s => s.cpu_pct >= 0 ? s.cpu_pct.toFixed(1) : null);
  const memData = samples.map(s => s.memory_mb >= 0 ? s.memory_mb.toFixed(1) : null);

  new Chart(document.getElementById('resourceChart'), {{
    type: 'line',
    data: {{
      labels: labels,
      datasets: [
        {{
          label: 'CPU %',
          data: cpuData,
          borderColor: COLORS.red,
          backgroundColor: COLORS.redBg,
          borderWidth: 2,
          pointRadius: 3,
          tension: 0.3,
          fill: true,
          yAxisID: 'y',
          spanGaps: true,
        }},
        {{
          label: 'Memory (MB)',
          data: memData,
          borderColor: COLORS.blue,
          backgroundColor: COLORS.blueBg,
          borderWidth: 2,
          pointRadius: 3,
          tension: 0.3,
          fill: true,
          yAxisID: 'y1',
          spanGaps: true,
        }},
      ],
    }},
    options: {{
      responsive: true,
      maintainAspectRatio: true,
      interaction: {{ mode: 'index', intersect: false }},
      scales: {{
        y: {{
          type: 'linear',
          position: 'left',
          title: {{ display: true, text: 'CPU (%)' }},
          min: 0,
          max: 100,
          grid: {{ color: '#1e293b' }},
        }},
        y1: {{
          type: 'linear',
          position: 'right',
          title: {{ display: true, text: 'Memory (MB)' }},
          beginAtZero: true,
          grid: {{ drawOnChartArea: false }},
        }},
        x: {{
          grid: {{ color: '#1e293b' }},
          ticks: {{
            maxRotation: 45,
            font: {{ size: 10 }},
          }},
        }},
      }},
      plugins: {{
        legend: {{ position: 'bottom' }},
      }},
    }},
  }});
}})();

// ---------------------------------------------------------------------------
// 4. Packet Loss Chart
// ---------------------------------------------------------------------------
(function() {{
  const tiers = DASHBOARD_DATA.tiers;
  if (!tiers.length) return;

  const labels = tiers.map(t => t.tier + ' calls');
  const pktLoss = tiers.map(t => t.packet_loss_pct.toFixed(2));

  // Color code by severity
  const bgColors = tiers.map(t => {{
    if (t.packet_loss_pct < 1) return COLORS.greenBg;
    if (t.packet_loss_pct < 3) return COLORS.yellowBg;
    if (t.packet_loss_pct < 5) return COLORS.redBg;
    return 'rgba(239, 68, 68, 0.4)';
  }});
  const borderColors = tiers.map(t => {{
    if (t.packet_loss_pct < 1) return COLORS.green;
    if (t.packet_loss_pct < 3) return COLORS.yellow;
    if (t.packet_loss_pct < 5) return COLORS.red;
    return COLORS.red;
  }});

  // Threshold lines
  const thresholds = tiers.map(() => 2);
  const thresholds5 = tiers.map(() => 5);

  new Chart(document.getElementById('packetLossChart'), {{
    type: 'bar',
    data: {{
      labels: labels,
      datasets: [
        {{
          label: 'Packet Loss %',
          data: pktLoss,
          backgroundColor: bgColors,
          borderColor: borderColors,
          borderWidth: 2,
          borderRadius: 4,
        }},
        {{
          label: '2% Threshold (10-call limit)',
          data: thresholds,
          type: 'line',
          borderColor: COLORS.yellow,
          borderDash: [6, 4],
          borderWidth: 1.5,
          pointRadius: 0,
          fill: false,
        }},
        {{
          label: '5% Threshold (50-call limit)',
          data: thresholds5,
          type: 'line',
          borderColor: COLORS.red,
          borderDash: [6, 4],
          borderWidth: 1.5,
          pointRadius: 0,
          fill: false,
        }},
      ],
    }},
    options: {{
      responsive: true,
      maintainAspectRatio: true,
      scales: {{
        y: {{
          title: {{ display: true, text: 'Packet Loss (%)' }},
          beginAtZero: true,
          grid: {{ color: '#1e293b' }},
        }},
        x: {{
          grid: {{ color: '#1e293b' }},
        }},
      }},
      plugins: {{
        legend: {{ position: 'bottom' }},
      }},
    }},
  }});
}})();

// ---------------------------------------------------------------------------
// 5. MOS Score Distribution
// ---------------------------------------------------------------------------
(function() {{
  const mosScores = DASHBOARD_DATA.mos_scores;
  if (!mosScores || !mosScores.length) {{
    // If no MOS data, show latency bucket distribution instead
    const buckets = DASHBOARD_DATA.latency_buckets;
    if (!buckets || !Object.keys(buckets).length) return;

    const labels = Object.keys(buckets);
    const values = Object.values(buckets);

    new Chart(document.getElementById('mosChart'), {{
      type: 'bar',
      data: {{
        labels: labels,
        datasets: [{{
          label: 'Call Count',
          data: values,
          backgroundColor: labels.map((_, i) => {{
            const fraction = i / Math.max(labels.length - 1, 1);
            if (fraction < 0.3) return COLORS.greenBg;
            if (fraction < 0.6) return COLORS.yellowBg;
            return COLORS.redBg;
          }}),
          borderColor: labels.map((_, i) => {{
            const fraction = i / Math.max(labels.length - 1, 1);
            if (fraction < 0.3) return COLORS.green;
            if (fraction < 0.6) return COLORS.yellow;
            return COLORS.red;
          }}),
          borderWidth: 2,
          borderRadius: 4,
        }}],
      }},
      options: {{
        responsive: true,
        maintainAspectRatio: true,
        scales: {{
          y: {{
            title: {{ display: true, text: 'Number of Calls' }},
            beginAtZero: true,
            grid: {{ color: '#1e293b' }},
          }},
          x: {{
            title: {{ display: true, text: 'Setup Latency Bucket' }},
            grid: {{ color: '#1e293b' }},
          }},
        }},
        plugins: {{
          legend: {{ display: false }},
          title: {{ display: true, text: 'Setup Latency Distribution (all tiers)', color: '#94a3b8' }},
        }},
      }},
    }});
    return;
  }}

  // Build histogram buckets for MOS scores
  const bucketLabels = ['1.0-1.5', '1.5-2.0', '2.0-2.5', '2.5-3.0', '3.0-3.5', '3.5-4.0', '4.0-4.5', '4.5-5.0'];
  const bucketCounts = new Array(8).fill(0);

  mosScores.forEach(score => {{
    let idx = Math.floor((score - 1.0) / 0.5);
    idx = Math.max(0, Math.min(7, idx));
    bucketCounts[idx]++;
  }});

  // Color: red for poor, yellow for fair, green for good
  const bgColors = bucketCounts.map((_, i) => {{
    if (i < 2) return COLORS.redBg;
    if (i < 4) return COLORS.yellowBg;
    return COLORS.greenBg;
  }});
  const borderClrs = bucketCounts.map((_, i) => {{
    if (i < 2) return COLORS.red;
    if (i < 4) return COLORS.yellow;
    return COLORS.green;
  }});

  new Chart(document.getElementById('mosChart'), {{
    type: 'bar',
    data: {{
      labels: bucketLabels,
      datasets: [{{
        label: 'Calls',
        data: bucketCounts,
        backgroundColor: bgColors,
        borderColor: borderClrs,
        borderWidth: 2,
        borderRadius: 4,
      }}],
    }},
    options: {{
      responsive: true,
      maintainAspectRatio: true,
      scales: {{
        y: {{
          title: {{ display: true, text: 'Number of Calls' }},
          beginAtZero: true,
          grid: {{ color: '#1e293b' }},
        }},
        x: {{
          title: {{ display: true, text: 'MOS Score Range' }},
          grid: {{ color: '#1e293b' }},
        }},
      }},
      plugins: {{
        legend: {{ display: false }},
      }},
    }},
  }});
}})();

// ---------------------------------------------------------------------------
// Test Results Table
// ---------------------------------------------------------------------------
(function() {{
  const tbody = document.getElementById('resultsTableBody');
  if (!tbody) return;

  const tiers = DASHBOARD_DATA.tiers;
  tiers.forEach(tier => {{
    // Tier header row
    const headerRow = document.createElement('tr');
    headerRow.className = 'tier-header';
    const passed = tier.test_cases.filter(tc => tc.passed).length;
    const total = tier.test_cases.length;
    headerRow.innerHTML = '<td colspan="4">' +
      tier.tier + '-Call Tier (' + passed + '/' + total + ' passed) ' +
      '| Success Rate: ' + (tier.success_rate * 100).toFixed(1) + '% ' +
      '| Avg Setup: ' + tier.avg_setup_time.toFixed(3) + 's ' +
      '| Pkt Loss: ' + tier.packet_loss_pct.toFixed(2) + '%' +
      '</td>';
    tbody.appendChild(headerRow);

    // Test case rows
    tier.test_cases.forEach(tc => {{
      const row = document.createElement('tr');
      const statusBadge = tc.passed
        ? '<span class="badge badge-pass">PASS</span>'
        : '<span class="badge badge-fail">FAIL</span>';
      const durationStr = tc.duration > 0 ? tc.duration.toFixed(1) + 's' : '-';
      const errorStr = tc.error ? tc.error.substring(0, 120) + (tc.error.length > 120 ? '...' : '') : '-';
      row.innerHTML =
        '<td>' + tc.name + '</td>' +
        '<td>' + statusBadge + '</td>' +
        '<td>' + durationStr + '</td>' +
        '<td style="color: ' + (tc.error ? '#ef4444' : '#64748b') + '; font-size: 0.8rem;">' + errorStr + '</td>';
      tbody.appendChild(row);
    }});
  }});
}})();
</script>
</body>
</html>"""

    return html


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="Generate an HTML performance dashboard from RustPBX load test results.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate a sample report (no test results needed)
  python tests/performance_dashboard.py

  # Generate from JSON results directory
  python tests/performance_dashboard.py --results-dir tests/results/

  # Specify custom output path
  python tests/performance_dashboard.py --output ~/rustpbx-perf.html

  # Print the conftest fixture code for integrating with pytest
  python tests/performance_dashboard.py --print-fixture
""",
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Output HTML file path (default: tests/performance_report.html)",
    )
    parser.add_argument(
        "--results-dir", "-r",
        default=None,
        help="Directory containing JSON result files from load tests",
    )
    parser.add_argument(
        "--print-fixture",
        action="store_true",
        help="Print the pytest conftest.py fixture code for metrics collection and exit",
    )
    parser.add_argument(
        "--json-input", "-j",
        default=None,
        help="Path to a single JSON file with combined results",
    )

    args = parser.parse_args()

    if args.print_fixture:
        print(CONFTEST_FIXTURE_CODE)
        return

    # Determine output path
    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_path = args.output or os.path.join(script_dir, "performance_report.html")

    # Try to load real data
    data = None

    if args.json_input and os.path.isfile(args.json_input):
        with open(args.json_input, "r") as f:
            raw = json.load(f)
        data = DashboardData(
            generated_at=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        )
        data = _parse_combined_results(raw, data)
        print(f"Loaded results from: {args.json_input}")

    if data is None and args.results_dir:
        data = load_results_from_directory(args.results_dir)
        if data:
            print(f"Loaded results from: {args.results_dir}")

    if data is None:
        # Try default results directory
        default_results = os.path.join(script_dir, "results")
        if os.path.isdir(default_results):
            data = load_results_from_directory(default_results)
            if data:
                print(f"Loaded results from: {default_results}")

    if data is None:
        print("No result files found. Generating sample report with placeholder data.")
        data = generate_sample_data()

    # Generate HTML
    html = generate_html_report(data)

    # Write output
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(html)

    print(f"Performance dashboard written to: {output_path}")
    print(f"  Tiers: {', '.join(str(t.tier) for t in data.tiers)}")
    print(f"  Total calls: {sum(t.total_calls for t in data.tiers)}")
    print(f"  Resource samples: {len(data.resource_samples)}")
    if data.mos_scores:
        print(f"  MOS scores: {len(data.mos_scores)}")

    return output_path


if __name__ == "__main__":
    main()
