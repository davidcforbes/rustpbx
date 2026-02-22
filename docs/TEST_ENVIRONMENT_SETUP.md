# RustPBX Test Environment Setup

This document describes how to set up, run, and extend the Docker-based integration test environment for RustPBX.

---

## Prerequisites

| Requirement | Minimum Version | Notes |
|---|---|---|
| Docker Engine | 24.0+ | `docker --version` |
| Docker Compose | v2.20+ (plugin) | `docker compose version` |
| Python | 3.12+ | Only needed for running tests outside Docker |
| Git | Any recent | To clone the repository |

> **Tip:** On Linux, ensure your user is in the `docker` group so you can run commands without `sudo`.

---

## Quick Start

All commands are run from the repository root.

### 1. Start the test environment

```bash
docker compose -f tests/docker-compose.test.yml up -d --build
```

This brings up three containers:

- **rustpbx** -- the system under test
- **postgres** -- backing database
- **test-tools** -- Python + SIPp test runner (idles with `sleep infinity`)

### 2. Wait for services to be healthy

```bash
docker compose -f tests/docker-compose.test.yml exec test-tools python wait_for_services.py
```

### 3. Run the full test suite

```bash
docker compose -f tests/docker-compose.test.yml exec test-tools \
    pytest --timeout=120 -v --html=/tests/report.html --self-contained-html
```

Or run a single test level:

```bash
# L1 - Unit / smoke tests
docker compose -f tests/docker-compose.test.yml exec test-tools pytest -v -m "l1"

# L2 - SIP integration tests
docker compose -f tests/docker-compose.test.yml exec test-tools pytest -v -m "l2"

# L3 - Performance / load tests
docker compose -f tests/docker-compose.test.yml exec test-tools pytest -v -m "l3"
```

### 4. Tear down

```bash
docker compose -f tests/docker-compose.test.yml down -v
```

The `-v` flag removes the ephemeral PostgreSQL volume.

---

## Architecture

```
+------------------+       +------------------+       +------------------+
|                  |       |                  |       |                  |
|    test-tools    |  SIP  |     rustpbx      |  SQL  |    postgres      |
|  (pytest + SIPp) +------>+  (system under   +------>+  (PostgreSQL     |
|                  |  HTTP  |    test)         |       |   16-alpine)     |
|                  +------>+                  |       |                  |
+------------------+       +--------+---------+       +------------------+
                                    |
                           ports exposed to host:
                           5060/udp, 5060/tcp,
                           8080/tcp, 20000-20100/udp

All containers share the 'rustpbx-test' bridge network.
```

### Container Details

| Container | Image | Purpose |
|---|---|---|
| `rustpbx-test` | `ghcr.io/restsend/rustpbx:latest` | The RustPBX server configured with 4 test SIP users (1001-1004) |
| `rustpbx-test-postgres` | `postgres:16-alpine` | Database backend for call records, user data |
| `rustpbx-test-tools` | Built from `Dockerfile.test-tools` | Test runner with pytest, SIPp, and network tools |

### Pre-configured SIP Users

| Username | Password | Domain |
|---|---|---|
| 1001 | test1001 | rustpbx |
| 1002 | test1002 | rustpbx |
| 1003 | test1003 | rustpbx |
| 1004 | test1004 | rustpbx |

---

## Test Levels

The test suite is organized into four levels:

### L1 -- Smoke / Health Tests

Basic connectivity and service health checks.

```bash
pytest -v -m "l1"
```

### L2 -- SIP Integration Tests

SIP REGISTER, INVITE, BYE, CANCEL, and call-flow scenarios using SIPp.

```bash
pytest -v -m "l2"
```

### L3 -- Performance / Load Tests

Sustained call load, concurrent registrations, and media throughput.

```bash
pytest -v -m "l3"
```

### L4 -- Telnyx External Integration Tests

These tests require a live Telnyx account and are **not** run by default.

```bash
# Set required environment variables
export TELNYX_API_KEY="your-api-key"
export TELNYX_SIP_DOMAIN="your-domain.sip.telnyx.com"
export TELNYX_DID="+15551234567"

# Run L4 tests
docker compose -f tests/docker-compose.test.yml exec \
    -e TELNYX_API_KEY="$TELNYX_API_KEY" \
    -e TELNYX_SIP_DOMAIN="$TELNYX_SIP_DOMAIN" \
    -e TELNYX_DID="$TELNYX_DID" \
    test-tools pytest -v -m "l4"
```

> **Warning:** L4 tests make real phone calls and may incur charges on your Telnyx account.

---

## How to Add New Test Cases

1. Create a new Python file under `tests/` following the naming convention `test_<level>_<description>.py` (e.g., `test_l2_call_transfer.py`).

2. Mark the test with the appropriate level marker:

```python
import pytest

@pytest.mark.l2
class TestCallTransfer:
    def test_blind_transfer(self):
        """Test SIP blind transfer (REFER) between two parties."""
        ...
```

3. If the test requires SIPp scenarios, place XML scenario files under `tests/sipp/` and reference them from your test.

4. Rebuild the test-tools container to include the new files:

```bash
docker compose -f tests/docker-compose.test.yml build test-tools
```

5. Run your new test:

```bash
docker compose -f tests/docker-compose.test.yml exec test-tools \
    pytest -v tests/test_l2_call_transfer.py
```

---

## How to Add SIPp Scenarios

SIPp XML scenarios live in `tests/sipp/`. To create a new scenario:

1. Write the SIPp XML file (e.g., `tests/sipp/uac_register.xml`).
2. Call SIPp from your Python test using `subprocess`:

```python
import subprocess

def run_sipp(scenario_file, target="rustpbx:5060", **kwargs):
    cmd = [
        "sipp", target,
        "-sf", f"/tests/sipp/{scenario_file}",
        "-m", str(kwargs.get("calls", 1)),
        "-r", str(kwargs.get("rate", 1)),
        "-timeout", str(kwargs.get("timeout", 30)),
        "-trace_err",
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
    return result
```

---

## Troubleshooting

### Container fails to start

**Symptom:** `rustpbx-test` exits immediately or restarts in a loop.

```bash
# Check the logs
docker compose -f tests/docker-compose.test.yml logs rustpbx

# Verify the config file is mounted correctly
docker compose -f tests/docker-compose.test.yml exec rustpbx cat /app/config.toml
```

**Common causes:**
- Invalid TOML syntax in `tests/config/test-config.toml`
- PostgreSQL not ready before RustPBX starts (check `depends_on` health conditions)
- Port 5060 or 8080 already in use on the host

### PostgreSQL connection refused

**Symptom:** RustPBX logs show `connection refused` to `postgres:5432`.

```bash
# Check postgres is running and healthy
docker compose -f tests/docker-compose.test.yml ps
docker compose -f tests/docker-compose.test.yml logs postgres
```

### SIPp tests fail with "Connection refused"

**Symptom:** SIPp cannot connect to `rustpbx:5060`.

```bash
# From the test-tools container, verify connectivity
docker compose -f tests/docker-compose.test.yml exec test-tools \
    nc -zv rustpbx 5060

# Check RustPBX is listening
docker compose -f tests/docker-compose.test.yml exec rustpbx \
    ss -tlnp | grep 5060
```

### Tests time out waiting for services

**Symptom:** `wait_for_services.py` exits with code 1.

```bash
# Increase the timeout
docker compose -f tests/docker-compose.test.yml exec test-tools \
    python -c "
import wait_for_services
wait_for_services.TIMEOUT_SECONDS = 120
import sys; sys.exit(wait_for_services.main())
"
```

### Port conflicts on the host

If ports 5060 or 8080 are already in use:

```bash
# Find what is using the port
ss -tlnp | grep :5060
ss -tlnp | grep :8080

# Or change the host port mapping in docker-compose.test.yml
# e.g., "15060:5060/udp" instead of "5060:5060/udp"
```

### Cleaning up stale state

```bash
# Full cleanup: containers, volumes, networks, and images
docker compose -f tests/docker-compose.test.yml down -v --rmi local --remove-orphans
```

---

## Viewing Test Reports

After a test run, the HTML report is available at `tests/report.html`. To copy it out of the container:

```bash
docker cp rustpbx-test-tools:/tests/report.html ./tests/report.html
```

Open `tests/report.html` in a browser to see detailed results with pass/fail status, durations, and captured output.

---

## CI Integration

To run the test suite in a CI pipeline (e.g., GitHub Actions):

```yaml
jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Start test environment
        run: docker compose -f tests/docker-compose.test.yml up -d --build

      - name: Wait for services
        run: |
          docker compose -f tests/docker-compose.test.yml exec -T test-tools \
              python wait_for_services.py

      - name: Run tests
        run: |
          docker compose -f tests/docker-compose.test.yml exec -T test-tools \
              pytest --timeout=120 -v --html=/tests/report.html --self-contained-html

      - name: Collect report
        if: always()
        run: docker cp rustpbx-test-tools:/tests/report.html ./report.html

      - name: Upload report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-report
          path: report.html

      - name: Tear down
        if: always()
        run: docker compose -f tests/docker-compose.test.yml down -v
```
