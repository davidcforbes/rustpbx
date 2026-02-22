# RustPBX Test Suite — Master Test Runner (Windows PowerShell)
# Usage: pwsh tests/run_all.ps1 [level]
#   level: L0, L1, L2, L3, L5, L6, L7, L8, all (default: all)
#
# Environment variables:
#   RUSTPBX_IMAGE     — Docker image (default: ghcr.io/restsend/rustpbx:latest)
#   TELNYX_SIP_USER   — Telnyx credentials (for L4 only)
#   TELNYX_SIP_PASS
#   SKIP_BUILD        — Skip building test-tools image (set to 1)

param(
    [string]$Level = "all"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$ComposeFile = Join-Path $ScriptDir "docker-compose.test.yml"

function Log($msg)  { Write-Host "[test] $msg" -ForegroundColor Cyan }
function Pass($msg) { Write-Host "[PASS] $msg" -ForegroundColor Green }
function Fail($msg) { Write-Host "[FAIL] $msg" -ForegroundColor Red }
function Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }

# Cleanup on exit
$cleanupBlock = {
    Log "Tearing down test environment..."
    & docker compose -f $ComposeFile down -v --remove-orphans 2>$null
}

try {
    Log "=== RustPBX Test Suite ==="
    Log "Level: $Level"
    Log "Compose: $ComposeFile"

    # Build
    if ($env:SKIP_BUILD -ne "1") {
        Log "Building test-tools image..."
        & docker compose -f $ComposeFile build test-tools
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    }

    # Start services
    Log "Starting test environment..."
    & docker compose -f $ComposeFile up -d rustpbx postgres
    if ($LASTEXITCODE -ne 0) { throw "Failed to start services" }

    # Wait for readiness
    Log "Waiting for services to be ready..."
    $ready = $false
    for ($i = 1; $i -le 30; $i++) {
        try {
            $null = & docker compose -f $ComposeFile exec -T rustpbx curl -sf http://localhost:8080/ami/v1/health 2>$null
            if ($LASTEXITCODE -eq 0) {
                Log "RustPBX is ready (attempt $i)"
                $ready = $true
                break
            }
        } catch {}
        Start-Sleep -Seconds 2
    }
    if (-not $ready) {
        Fail "RustPBX did not become ready within 60 seconds"
        & docker compose -f $ComposeFile logs rustpbx
        exit 1
    }

    # Create admin user
    Log "Creating admin user..."
    & docker compose -f $ComposeFile exec -T rustpbx /app/rustpbx --conf /app/config.toml --super-username admin --super-password admin123 --super-email admin@test.local 2>$null

    # Determine test files
    $ResultsDir = Join-Path $ScriptDir "results"
    New-Item -ItemType Directory -Force -Path $ResultsDir | Out-Null

    $pytestArgs = @("-v", "--tb=short", "--timeout=60")

    switch ($Level.ToUpper()) {
        "L0" { $pytestArgs += "/tests/test_L0_smoke.py" }
        "L1" { $pytestArgs += "/tests/test_L0_smoke.py", "/tests/test_L1_infra.py" }
        "L2" { $pytestArgs += "/tests/test_L0_smoke.py", "/tests/test_L1_infra.py", "/tests/test_L2_api.py" }
        "L3" { $pytestArgs += "/tests/test_L0_smoke.py", "/tests/test_L1_infra.py", "/tests/test_L2_api.py", "/tests/test_L3_sip.py" }
        "L4" {
            if (-not $env:TELNYX_SIP_USER) {
                Fail "L4 tests require TELNYX_SIP_USER and TELNYX_SIP_PASS environment variables"
                exit 1
            }
            $pytestArgs += "/tests/test_L4_integration.py"
        }
        "L5" { $pytestArgs += "/tests/test_L5_media.py" }
        "L6" { $pytestArgs += "/tests/test_L6_load.py" }
        "L7" { $pytestArgs += "/tests/test_L7_failover.py" }
        "L8" { $pytestArgs += "/tests/test_L8_security.py" }
        "ALL" {
            $pytestArgs += "/tests/test_L0_smoke.py", "/tests/test_L1_infra.py", "/tests/test_L2_api.py",
                           "/tests/test_L3_sip.py", "/tests/test_L5_media.py",
                           "/tests/test_L7_failover.py", "/tests/test_L8_security.py"
        }
        default {
            Fail "Unknown level: $Level"
            Write-Host "Usage: run_all.ps1 [L0|L1|L2|L3|L4|L5|L6|L7|L8|all]"
            exit 1
        }
    }

    $pytestArgs += "--junitxml=/tests/results/junit.xml"

    # Run tests
    Log "Running tests..."
    $extraEnv = @()
    if ($env:TELNYX_SIP_USER) {
        $extraEnv = @("-e", "TELNYX_SIP_USER", "-e", "TELNYX_SIP_PASS")
    }

    $dockerArgs = @("compose", "-f", $ComposeFile, "run", "--rm") + $extraEnv + @("test-tools", "pytest") + $pytestArgs
    & docker @dockerArgs
    $exitCode = $LASTEXITCODE

    # Report
    Write-Host ""
    if ($exitCode -eq 0) {
        Pass "=== ALL TESTS PASSED ==="
    } else {
        Fail "=== TESTS FAILED (exit code: $exitCode) ==="
    }

    exit $exitCode

} finally {
    & $cleanupBlock
}
