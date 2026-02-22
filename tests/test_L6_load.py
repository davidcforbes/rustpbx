"""
L6 Load Tests -- Concurrency, throughput, and stress.

These tests put the PBX under heavier-than-normal load to verify it remains
stable and responsive.

Environment variables:
    SIPP_PATH          Path to the sipp binary       (default: sipp)
    RUSTPBX_HOST       Hostname / IP of the PBX      (default: rustpbx)
    RUSTPBX_SIP_PORT   SIP port                      (default: 5060)
    RUSTPBX_HTTP_PORT  HTTP/AMI port                  (default: 8080)
"""

import os
import subprocess
import statistics
import time

import pytest
import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SIPP_PATH = os.environ.get("SIPP_PATH", "sipp")
RUSTPBX_HOST = os.environ.get("RUSTPBX_HOST", "rustpbx")
RUSTPBX_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
RUSTPBX_HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8080"))
SIPP_SCENARIOS = os.path.join(os.path.dirname(__file__), "sipp")

BASE_URL = f"http://{RUSTPBX_HOST}:{RUSTPBX_HTTP_PORT}"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _create_users_csv(path, count=50):
    """Create a SIPp CSV injection file with ``count`` users.

    The test config provides users 1001-1004.  For load tests we cycle through
    these users to generate many concurrent registrations.
    """
    with open(path, "w") as f:
        f.write("SEQUENTIAL\n")
        for i in range(count):
            idx = (i % 4) + 1  # 1..4
            user = f"100{idx}"
            pwd = f"test100{idx}"
            f.write(f"{user};{pwd}\n")


# ---------------------------------------------------------------------------
# L6 Test Class
# ---------------------------------------------------------------------------


class TestL6Load:
    """L6: Load and stress tests."""

    @pytest.mark.timeout(120)
    def test_L6_001_concurrent_registrations(self, tmp_path):
        """TC-L6-001: 50 concurrent REGISTER flows at 10/s complete without errors.

        Uses the load_register.xml scenario with a CSV of user credentials.
        """
        csv_file = str(tmp_path / "users.csv")
        _create_users_csv(csv_file, count=50)

        cmd = [
            SIPP_PATH,
            f"{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT}",
            "-sf", os.path.join(SIPP_SCENARIOS, "load_register.xml"),
            "-inf", csv_file,
            "-r", "10",              # 10 registrations per second
            "-l", "50",              # up to 50 concurrent
            "-m", "50",              # total 50 registrations
            "-timeout", "60",
            "-timeout_error",
            "-max_retrans", "5",
            "-trace_err",
        ]
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=90,
        )
        assert result.returncode == 0, (
            f"SIPp load register failed (rc={result.returncode}).\n"
            f"stderr:\n{result.stderr}"
        )

    @pytest.mark.timeout(60)
    def test_L6_002_api_throughput(self):
        """TC-L6-002: 100 rapid AMI health requests complete with acceptable latency.

        Measures p50, p95, and p99 response times.  Fails if p95 exceeds 2 seconds.
        """
        NUM_REQUESTS = 100
        MAX_P95_SECONDS = 2.0

        latencies = []
        errors = 0

        for _ in range(NUM_REQUESTS):
            start = time.monotonic()
            try:
                resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
                elapsed = time.monotonic() - start
                latencies.append(elapsed)
                if resp.status_code != 200:
                    errors += 1
            except requests.RequestException:
                elapsed = time.monotonic() - start
                latencies.append(elapsed)
                errors += 1

        assert len(latencies) > 0, "No requests completed"

        latencies.sort()
        p50 = latencies[int(len(latencies) * 0.50)]
        p95 = latencies[int(len(latencies) * 0.95)]
        p99 = latencies[int(len(latencies) * 0.99)]
        mean = statistics.mean(latencies)

        print(f"\n--- API throughput results ({NUM_REQUESTS} requests) ---")
        print(f"  Mean  : {mean:.4f}s")
        print(f"  p50   : {p50:.4f}s")
        print(f"  p95   : {p95:.4f}s")
        print(f"  p99   : {p99:.4f}s")
        print(f"  Errors: {errors}/{NUM_REQUESTS}")

        assert p95 < MAX_P95_SECONDS, (
            f"p95 latency ({p95:.4f}s) exceeds threshold ({MAX_P95_SECONDS}s)"
        )
        assert errors < NUM_REQUESTS * 0.05, (
            f"Too many errors: {errors}/{NUM_REQUESTS} "
            f"(>{NUM_REQUESTS * 0.05:.0f} allowed)"
        )

    @pytest.mark.timeout(60)
    def test_L6_003_rapid_options(self):
        """TC-L6-003: 100 SIP OPTIONS in rapid succession do not crash the PBX.

        Sends OPTIONS pings at a high rate using SIPp and then checks the PBX
        health endpoint is still responsive.
        """
        cmd = [
            SIPP_PATH,
            f"{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT}",
            "-sf", os.path.join(SIPP_SCENARIOS, "options_ping.xml"),
            "-r", "50",              # 50 per second
            "-m", "100",             # 100 total
            "-timeout", "30",
            "-timeout_error",
            "-max_retrans", "2",
        ]
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=45,
        )
        # SIPp may report failures if OPTIONS gets 401, but the PBX must not crash.
        # Verify by hitting the health endpoint.
        resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
        assert resp.status_code == 200, (
            f"PBX health check failed after rapid OPTIONS burst "
            f"(HTTP {resp.status_code})"
        )

    @pytest.mark.timeout(120)
    def test_L6_004_sustained_registration_rate(self, tmp_path):
        """TC-L6-004: Sustained registration rate of 5/s for 20 seconds."""
        csv_file = str(tmp_path / "users_sustained.csv")
        _create_users_csv(csv_file, count=100)

        cmd = [
            SIPP_PATH,
            f"{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT}",
            "-sf", os.path.join(SIPP_SCENARIOS, "load_register.xml"),
            "-inf", csv_file,
            "-r", "5",               # 5 per second
            "-l", "20",              # 20 concurrent max
            "-m", "100",             # 100 total
            "-timeout", "90",
            "-timeout_error",
            "-max_retrans", "5",
        ]
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=110,
        )
        assert result.returncode == 0, (
            f"Sustained registration test failed.\nstderr:\n{result.stderr}"
        )

    @pytest.mark.timeout(30)
    def test_L6_005_health_after_load(self):
        """TC-L6-005: PBX is healthy after all load tests."""
        resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
        assert resp.status_code == 200
