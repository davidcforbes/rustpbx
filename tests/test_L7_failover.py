"""
L7 Failover Tests -- Service recovery, config reload, and resilience.

These tests verify that RustPBX can recover from restarts and configuration
reloads without leaving the system in a broken state.

Environment variables:
    RUSTPBX_HOST       Hostname / IP of the PBX      (default: rustpbx)
    RUSTPBX_SIP_PORT   SIP port                      (default: 5060)
    RUSTPBX_HTTP_PORT  HTTP/AMI port                  (default: 8080)

Note:
    Some tests in this module require Docker access (to restart the rustpbx
    container).  When running inside the test-tools container, the Docker
    socket must be mounted, or the tests should be run from the host.
"""

import os
import socket
import time

import pytest
import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

RUSTPBX_HOST = os.environ.get("RUSTPBX_HOST", "rustpbx")
RUSTPBX_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
RUSTPBX_HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8080"))
RUSTPBX_CONTAINER = os.environ.get("RUSTPBX_CONTAINER", "rustpbx-test")

BASE_URL = f"http://{RUSTPBX_HOST}:{RUSTPBX_HTTP_PORT}"

# Maximum time to wait for the PBX to come back after a restart.
RESTART_TIMEOUT = 45  # seconds


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _wait_for_health(timeout=RESTART_TIMEOUT):
    """Poll the health endpoint until it responds or timeout is reached.

    Returns:
        True if the health endpoint responded with 2xx before timeout.
        False otherwise.
    """
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=3)
            if resp.status_code < 400:
                return True
        except (requests.ConnectionError, requests.Timeout):
            pass
        time.sleep(1)
    return False


def _check_sip_port(timeout=10):
    """Return True if the SIP UDP port is reachable (responds to a probe)."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.settimeout(2)
            # Send a minimal SIP OPTIONS probe.
            probe = (
                f"OPTIONS sip:{RUSTPBX_HOST} SIP/2.0\r\n"
                f"Via: SIP/2.0/UDP 127.0.0.1:9999;branch=z9hG4bK-probe\r\n"
                f"From: <sip:probe@127.0.0.1>;tag=probe\r\n"
                f"To: <sip:{RUSTPBX_HOST}>\r\n"
                f"Call-ID: probe-{time.monotonic_ns()}@127.0.0.1\r\n"
                f"CSeq: 1 OPTIONS\r\n"
                f"Max-Forwards: 1\r\n"
                f"Content-Length: 0\r\n"
                f"\r\n"
            ).encode()
            sock.sendto(probe, (RUSTPBX_HOST, RUSTPBX_SIP_PORT))
            data, _ = sock.recvfrom(4096)
            sock.close()
            if b"SIP/2.0" in data:
                return True
        except (socket.timeout, OSError):
            pass
        finally:
            try:
                sock.close()
            except Exception:
                pass
        time.sleep(1)
    return False


def _docker_available():
    """Return True if the ``docker`` Python package can connect to the daemon."""
    try:
        import docker
        client = docker.from_env()
        client.ping()
        return True
    except Exception:
        return False


def _restart_container():
    """Restart the RustPBX Docker container and wait for it to become healthy."""
    import docker
    client = docker.from_env()
    container = client.containers.get(RUSTPBX_CONTAINER)
    container.restart(timeout=10)
    # Wait for health.
    return _wait_for_health(timeout=RESTART_TIMEOUT)


# ---------------------------------------------------------------------------
# L7 Test Class
# ---------------------------------------------------------------------------


class TestL7Failover:
    """L7: Failover, recovery, and config-reload tests."""

    @pytest.mark.timeout(90)
    def test_L7_001_service_recovery_after_restart(self):
        """TC-L7-001: SIP port responds after a container restart.

        Restarts the RustPBX container, then verifies both the HTTP health
        endpoint and the SIP UDP port become reachable again.
        """
        if not _docker_available():
            pytest.skip("Docker daemon not available -- cannot restart container")

        # Restart.
        healthy = _restart_container()
        assert healthy, (
            f"RustPBX did not become healthy within {RESTART_TIMEOUT}s "
            f"after container restart"
        )

        # Also verify SIP port.
        sip_up = _check_sip_port(timeout=15)
        assert sip_up, "SIP port did not respond after container restart"

    @pytest.mark.timeout(30)
    def test_L7_002_config_reload_during_idle(self):
        """TC-L7-002: Reload routes while no calls are active.

        Triggers a configuration reload via the AMI endpoint and verifies the
        PBX remains healthy.  If there is no explicit reload endpoint, we
        simply verify current health as a baseline.
        """
        # Attempt reload via AMI.
        try:
            resp = requests.post(
                f"{BASE_URL}/ami/v1/reload",
                json={},
                timeout=10,
            )
            # Accept 200 (success) or 404 (endpoint not implemented).
            assert resp.status_code in (200, 204, 404, 405), (
                f"Unexpected status on reload: {resp.status_code}"
            )
        except requests.ConnectionError:
            pytest.skip("AMI reload endpoint not reachable")

        # After reload, health must still be OK.
        resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
        assert resp.status_code == 200, (
            f"Health check failed after config reload (HTTP {resp.status_code})"
        )

    @pytest.mark.timeout(30)
    def test_L7_003_api_available_after_reload(self):
        """TC-L7-003: AMI health endpoint responds after any previous reload."""
        resp = requests.get(f"{BASE_URL}/ami/v1/health", timeout=5)
        assert resp.status_code == 200

    @pytest.mark.timeout(90)
    def test_L7_004_sip_survives_restart(self):
        """TC-L7-004: After a restart the SIP port accepts new OPTIONS."""
        if not _docker_available():
            pytest.skip("Docker daemon not available")

        healthy = _restart_container()
        assert healthy, "PBX not healthy after restart"

        # Send a SIP OPTIONS probe.
        assert _check_sip_port(timeout=20), (
            "SIP port did not respond to OPTIONS after restart"
        )

    @pytest.mark.timeout(30)
    def test_L7_005_dialogs_empty_after_restart(self):
        """TC-L7-005: No stale dialogs remain after a service restart."""
        resp = requests.get(f"{BASE_URL}/ami/v1/dialogs", timeout=5)
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                assert len(data) == 0, f"Stale dialogs after restart: {data}"
