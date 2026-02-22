#!/usr/bin/env python3
"""
wait_for_services.py

Waits for RustPBX and PostgreSQL to become healthy before tests execute.
Intended to be run inside the test-tools container where 'rustpbx' and
'postgres' resolve via Docker's internal DNS.

Usage:
    python wait_for_services.py

Exit codes:
    0  - All services are ready
    1  - Timeout reached before all services became ready
"""

import socket
import sys
import time

import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
RUSTPBX_HEALTH_URL = "http://rustpbx:8080/ami/v1/health"
POSTGRES_HOST = "postgres"
POSTGRES_PORT = 5432

POLL_INTERVAL_SECONDS = 2
TIMEOUT_SECONDS = 60


# ---------------------------------------------------------------------------
# Health-check helpers
# ---------------------------------------------------------------------------

def check_rustpbx() -> bool:
    """Return True if the RustPBX HTTP health endpoint responds with 2xx."""
    try:
        resp = requests.get(RUSTPBX_HEALTH_URL, timeout=3)
        return resp.status_code < 400
    except (requests.ConnectionError, requests.Timeout, requests.RequestException):
        return False


def check_postgres() -> bool:
    """Return True if a TCP connection to PostgreSQL succeeds."""
    try:
        sock = socket.create_connection((POSTGRES_HOST, POSTGRES_PORT), timeout=3)
        sock.close()
        return True
    except (socket.timeout, socket.error, OSError):
        return False


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

def main() -> int:
    start = time.time()
    postgres_ready = False
    rustpbx_ready = False

    print(f"Waiting up to {TIMEOUT_SECONDS}s for services to become ready...")
    print(f"  RustPBX health endpoint : {RUSTPBX_HEALTH_URL}")
    print(f"  PostgreSQL              : {POSTGRES_HOST}:{POSTGRES_PORT}")
    print()

    while True:
        elapsed = time.time() - start

        if not postgres_ready:
            postgres_ready = check_postgres()
            if postgres_ready:
                print(f"  [+] PostgreSQL is ready  ({elapsed:.1f}s)")

        if not rustpbx_ready:
            rustpbx_ready = check_rustpbx()
            if rustpbx_ready:
                print(f"  [+] RustPBX is ready     ({elapsed:.1f}s)")

        if postgres_ready and rustpbx_ready:
            print(f"\nAll services ready in {elapsed:.1f}s.")
            return 0

        if elapsed >= TIMEOUT_SECONDS:
            print(f"\nTimeout after {TIMEOUT_SECONDS}s waiting for services.")
            if not postgres_ready:
                print("  [-] PostgreSQL is NOT ready")
            if not rustpbx_ready:
                print("  [-] RustPBX is NOT ready")
            return 1

        time.sleep(POLL_INTERVAL_SECONDS)


if __name__ == "__main__":
    sys.exit(main())
