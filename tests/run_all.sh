#!/usr/bin/env bash
# RustPBX Test Suite — Master Test Runner
# Usage: ./tests/run_all.sh [level]
#   level: L0, L1, L2, L3, L5, L6, L7, L8, all (default: all)
#
# Environment variables:
#   RUSTPBX_IMAGE     — Docker image (default: ghcr.io/restsend/rustpbx:latest)
#   TELNYX_SIP_USER   — Telnyx credentials (for L4 only)
#   TELNYX_SIP_PASS
#   SKIP_BUILD        — Skip building test-tools image (set to 1)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.test.yml"
LEVEL="${1:-all}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[test]${NC} $*"; }
pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }

cleanup() {
    log "Tearing down test environment..."
    docker compose -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
}

trap cleanup EXIT

# --- Build & Start ---
log "=== RustPBX Test Suite ==="
log "Level: $LEVEL"
log "Compose: $COMPOSE_FILE"

if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
    log "Building test-tools image..."
    docker compose -f "$COMPOSE_FILE" build test-tools
fi

log "Starting test environment..."
docker compose -f "$COMPOSE_FILE" up -d rustpbx postgres

log "Waiting for services to be ready..."
# Wait for RustPBX HTTP
for i in $(seq 1 30); do
    if docker compose -f "$COMPOSE_FILE" exec -T rustpbx \
        curl -sf http://localhost:8080/ami/v1/health > /dev/null 2>&1; then
        log "RustPBX is ready (attempt $i)"
        break
    fi
    if [[ $i -eq 30 ]]; then
        fail "RustPBX did not become ready within 60 seconds"
        docker compose -f "$COMPOSE_FILE" logs rustpbx
        exit 1
    fi
    sleep 2
done

# Create super user
log "Creating admin user..."
docker compose -f "$COMPOSE_FILE" exec -T rustpbx \
    /app/rustpbx --conf /app/config.toml \
    --super-username admin --super-password admin123 \
    --super-email admin@test.local 2>/dev/null || true

# --- Determine which tests to run ---
PYTEST_ARGS="-v --tb=short --timeout=60"
RESULTS_DIR="$SCRIPT_DIR/results"
mkdir -p "$RESULTS_DIR"

case "$LEVEL" in
    L0|l0)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L0_smoke.py"
        ;;
    L1|l1)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L0_smoke.py /tests/test_L1_infra.py"
        ;;
    L2|l2)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L0_smoke.py /tests/test_L1_infra.py /tests/test_L2_api.py"
        ;;
    L3|l3)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L0_smoke.py /tests/test_L1_infra.py /tests/test_L2_api.py /tests/test_L3_sip.py"
        ;;
    L4|l4)
        if [[ -z "${TELNYX_SIP_USER:-}" ]]; then
            fail "L4 tests require TELNYX_SIP_USER and TELNYX_SIP_PASS environment variables"
            exit 1
        fi
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L4_integration.py"
        ;;
    L5|l5)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L5_media.py"
        ;;
    L6|l6)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L6_load.py"
        ;;
    L7|l7)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L7_failover.py"
        ;;
    L8|l8)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L8_security.py"
        ;;
    all)
        PYTEST_ARGS="$PYTEST_ARGS /tests/test_L0_smoke.py /tests/test_L1_infra.py /tests/test_L2_api.py /tests/test_L3_sip.py /tests/test_L5_media.py /tests/test_L7_failover.py /tests/test_L8_security.py"
        ;;
    *)
        fail "Unknown level: $LEVEL"
        echo "Usage: $0 [L0|L1|L2|L3|L4|L5|L6|L7|L8|all]"
        exit 1
        ;;
esac

PYTEST_ARGS="$PYTEST_ARGS --junitxml=/tests/results/junit.xml"

# --- Run Tests ---
log "Running tests..."
EXTRA_ENV=""
if [[ -n "${TELNYX_SIP_USER:-}" ]]; then
    EXTRA_ENV="-e TELNYX_SIP_USER -e TELNYX_SIP_PASS"
fi

docker compose -f "$COMPOSE_FILE" run --rm \
    $EXTRA_ENV \
    test-tools pytest $PYTEST_ARGS
EXIT_CODE=$?

# --- Report ---
echo ""
if [[ $EXIT_CODE -eq 0 ]]; then
    pass "=== ALL TESTS PASSED ==="
else
    fail "=== TESTS FAILED (exit code: $EXIT_CODE) ==="
fi

if [[ -f "$RESULTS_DIR/junit.xml" ]]; then
    log "JUnit XML report: $RESULTS_DIR/junit.xml"
fi

exit $EXIT_CODE
