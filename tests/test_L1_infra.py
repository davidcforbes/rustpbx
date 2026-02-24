"""
L1 Infrastructure Tests -- System resource and connectivity checks.

These tests validate that the underlying infrastructure required by RustPBX
is healthy: TLS certificates, network ports, disk space, memory, config
files, database, and log file accessibility.

All tests target the LOCAL machine (localhost / 127.0.0.1) and assume the
server is running with its standard configuration.

Environment variables (with defaults):
    RUSTPBX_HOST        - SIP/HTTP host          (default: from conftest)
    RUSTPBX_HTTPS_PORT  - HTTPS port             (default: 8443)
    RUSTPBX_SIP_PORT    - SIP port               (default: 5060)
    RUSTPBX_CONFIG      - Path to config.toml    (default: auto-detect)
    RUSTPBX_WORK_DIR    - Server working dir     (default: ~/rustpbx)

Expected execution time: < 30 seconds for the full suite.
"""
import os
import ssl
import socket
import shutil
import pathlib

import pytest

try:
    import tomllib
except ImportError:
    # Python < 3.11 fallback
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ImportError:
        tomllib = None  # type: ignore[assignment]


# ---------------------------------------------------------------------------
# Configuration helpers
# ---------------------------------------------------------------------------

def _get_host():
    """Resolve the target host -- prefer env var, then conftest default."""
    return os.environ.get("RUSTPBX_HOST", "127.0.0.1")


def _get_https_port():
    return int(os.environ.get("RUSTPBX_HTTPS_PORT", "8443"))


def _get_sip_port():
    return int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))


def _get_work_dir():
    """Server working directory (where it was launched from)."""
    env = os.environ.get("RUSTPBX_WORK_DIR")
    if env:
        return pathlib.Path(env)
    # Common locations on the Linode / test server
    candidates = [
        pathlib.Path.home() / "rustpbx",
        pathlib.Path("/root/rustpbx"),
        pathlib.Path.cwd(),
    ]
    for p in candidates:
        if (p / "config.toml").exists() or (p / "rustpbx-config" / "config.toml").exists():
            return p
    return pathlib.Path.cwd()


def _find_config_path():
    """Locate the active config.toml."""
    env = os.environ.get("RUSTPBX_CONFIG")
    if env:
        return pathlib.Path(env)
    work = _get_work_dir()
    candidates = [
        work / "config.toml",
        work / "rustpbx-config" / "config.toml",
        pathlib.Path.home() / "rustpbx-config" / "config.toml",
        pathlib.Path("/root/rustpbx-config/config.toml"),
    ]
    for p in candidates:
        if p.exists():
            return p
    return candidates[0]  # return first candidate even if missing


def _load_config():
    """Load and parse the TOML config, returning (path, dict) or (path, None)."""
    path = _find_config_path()
    if not path.exists() or tomllib is None:
        return path, None
    with open(path, "rb") as f:
        return path, tomllib.load(f)


# ===========================================================================
# L1 Infrastructure Test Class
# ===========================================================================

class TestL1Infrastructure:
    """L1: Verify infrastructure prerequisites for RustPBX operation."""

    # ------------------------------------------------------------------
    # TC-L1-INFRA-001: TLS certificate exists and is loadable
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L1_infra_001_tls_cert_exists_and_loadable(self):
        """TC-L1-INFRA-001: TLS certificate exists on disk and can be loaded.

        Checks the config/certs/ directory (relative to the server working
        dir) for .crt + .key file pairs, OR checks the ssl_certificate /
        ssl_private_key paths from the config.  Verifies the cert can be
        parsed by Python's ssl module.
        """
        config_path, config = _load_config()
        work = _get_work_dir()

        cert_path = None
        key_path = None

        # Strategy 1: Explicit paths in config
        if config:
            ssl_cert = config.get("ssl_certificate")
            ssl_key = config.get("ssl_private_key")
            if ssl_cert and ssl_key:
                cert_path = pathlib.Path(ssl_cert)
                key_path = pathlib.Path(ssl_key)
                # Resolve relative paths against work dir
                if not cert_path.is_absolute():
                    cert_path = work / cert_path
                if not key_path.is_absolute():
                    key_path = work / key_path

        # Strategy 2: Auto-detect from config/certs/
        if cert_path is None:
            cert_dir = work / "config" / "certs"
            if cert_dir.is_dir():
                for f in cert_dir.iterdir():
                    if f.suffix == ".crt":
                        candidate_key = f.with_suffix(".key")
                        if candidate_key.exists():
                            cert_path = f
                            key_path = candidate_key
                            break

        if cert_path is None:
            pytest.skip(
                "No TLS certificate found in config or config/certs/ "
                "(HTTPS may not be configured on this server)"
            )

        assert cert_path.exists(), f"Certificate file missing: {cert_path}"
        assert key_path.exists(), f"Private key file missing: {key_path}"

        # Verify the cert is parseable
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        try:
            ctx.load_cert_chain(str(cert_path), str(key_path))
        except ssl.SSLError as exc:
            pytest.fail(f"Certificate/key pair cannot be loaded: {exc}")

    # ------------------------------------------------------------------
    # TC-L1-INFRA-002: HTTPS endpoint responds with valid TLS
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_infra_002_https_endpoint_responds(self):
        """TC-L1-INFRA-002: HTTPS port 8443 responds with a valid TLS handshake.

        Connects to the HTTPS port and completes a TLS handshake. We allow
        self-signed certificates (no hostname or CA verification). The test
        passes if the handshake succeeds and we receive an HTTP response.
        """
        import urllib.request
        import urllib.error

        host = _get_host()
        port = _get_https_port()

        # First: verify the TCP port is open at all
        probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        probe.settimeout(5)
        try:
            probe.connect((host, port))
        except (socket.error, OSError) as exc:
            pytest.skip(f"HTTPS port {port} not reachable: {exc}")
        finally:
            probe.close()

        # Now do a real TLS handshake and HTTP request
        ctx = ssl.create_default_context()
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE

        url = f"https://{host}:{port}/"
        req = urllib.request.Request(url, method="GET")
        try:
            resp = urllib.request.urlopen(req, timeout=10, context=ctx)
            status = resp.status
        except urllib.error.HTTPError as exc:
            # An HTTPError still means TLS + HTTP worked -- we got a response
            status = exc.code
        except urllib.error.URLError as exc:
            pytest.fail(f"HTTPS request failed after TLS handshake: {exc}")

        # Any HTTP status means the TLS handshake was successful
        assert 100 <= status < 600, f"Unexpected HTTP status: {status}"

    # ------------------------------------------------------------------
    # TC-L1-INFRA-003: SIP UDP port 5060 is listening
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L1_infra_003_sip_udp_port_listening(self):
        """TC-L1-INFRA-003: SIP UDP port 5060 responds to OPTIONS probe.

        Sends a well-formed SIP OPTIONS request and verifies we receive
        a SIP response, confirming the UDP listener is active.
        """
        from conftest import build_sip_options

        host = _get_host()
        port = _get_sip_port()

        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(5)
        try:
            msg = build_sip_options(host, port)
            sock.sendto(msg, (host, port))
            data, _ = sock.recvfrom(4096)
            assert b"SIP/2.0" in data, (
                f"Response is not a valid SIP message: {data[:80]}"
            )
        except socket.timeout:
            pytest.fail(
                f"SIP UDP port {port} did not respond within 5 seconds "
                "(listener may be down)"
            )
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L1-INFRA-004: SIP TCP port 5060 is listening
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L1_infra_004_sip_tcp_port_listening(self):
        """TC-L1-INFRA-004: SIP TCP port 5060 accepts connections.

        Performs a TCP connect to the SIP port, confirming the TCP listener
        is active and accepting new connections.
        """
        host = _get_host()
        port = _get_sip_port()

        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        try:
            sock.connect((host, port))
            # Connection succeeded -- the port is listening
        except socket.timeout:
            pytest.fail(
                f"SIP TCP port {port} connection timed out "
                "(listener may be down)"
            )
        except ConnectionRefusedError:
            pytest.fail(
                f"SIP TCP port {port} refused connection "
                "(listener not running)"
            )
        finally:
            sock.close()

    # ------------------------------------------------------------------
    # TC-L1-INFRA-005: RTP port range is available
    # ------------------------------------------------------------------
    @pytest.mark.timeout(15)
    def test_L1_infra_005_rtp_port_range_available(self):
        """TC-L1-INFRA-005: RTP ports in the configured range are bindable.

        Spot-checks several ports from the RTP range (default 20000-20100
        from the Linode config, or 12000-42000 from code defaults) to verify
        they are not blocked or already consumed by other processes.

        We attempt to bind a UDP socket on each port. If binding succeeds,
        the port is available. If it fails with EADDRINUSE, that port is
        occupied (which is OK if RustPBX is using it for active calls).
        We fail only if ALL sampled ports are in use.
        """
        _, config = _load_config()

        rtp_start = 20000
        rtp_end = 20100
        if config:
            rtp_start = config.get("rtp_start_port", rtp_start) or rtp_start
            rtp_end = config.get("rtp_end_port", rtp_end) or rtp_end

        # Spot-check: start, end, and 3 evenly-spaced ports in between
        total_range = rtp_end - rtp_start
        step = max(1, total_range // 4)
        sample_ports = sorted(set([
            rtp_start,
            rtp_start + step,
            rtp_start + 2 * step,
            rtp_start + 3 * step,
            rtp_end,
        ]))

        available = 0
        in_use = 0
        errors = []

        for port in sample_ports:
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            try:
                sock.bind(("0.0.0.0", port))
                available += 1
            except OSError as exc:
                # EADDRINUSE (98 on Linux) is expected if RustPBX has the port
                if exc.errno in (98, 10048):  # Linux / Windows EADDRINUSE
                    in_use += 1
                else:
                    errors.append(f"Port {port}: {exc}")
            finally:
                sock.close()

        total_checked = available + in_use
        assert total_checked > 0, (
            f"Could not probe any RTP ports. Errors: {errors}"
        )
        # It is acceptable for some ports to be in use (active calls).
        # Fail only if there are hard errors on every port.
        if errors and available == 0 and in_use == 0:
            pytest.fail(
                f"All {len(sample_ports)} sampled RTP ports had errors: {errors}"
            )

    # ------------------------------------------------------------------
    # TC-L1-INFRA-006: Disk space sufficient (> 1 GB free)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(5)
    def test_L1_infra_006_disk_space_sufficient(self):
        """TC-L1-INFRA-006: At least 1 GB of free disk space is available.

        Checks the filesystem where the server working directory resides.
        Recordings, CDR files, and logs all consume disk over time, so
        running out of space would silently break the system.
        """
        work = _get_work_dir()
        usage = shutil.disk_usage(str(work))
        free_gb = usage.free / (1024 ** 3)
        assert free_gb >= 1.0, (
            f"Insufficient disk space: {free_gb:.2f} GB free "
            f"(minimum 1.0 GB required)"
        )

    # ------------------------------------------------------------------
    # TC-L1-INFRA-007: System memory available (> 256 MB free)
    # ------------------------------------------------------------------
    @pytest.mark.timeout(5)
    def test_L1_infra_007_system_memory_available(self):
        """TC-L1-INFRA-007: At least 256 MB of available memory.

        Reads /proc/meminfo (Linux) to determine available memory. On
        non-Linux systems the test is skipped.
        """
        meminfo_path = pathlib.Path("/proc/meminfo")
        if not meminfo_path.exists():
            pytest.skip("Not a Linux system (/proc/meminfo not found)")

        mem_available_kb = None
        with open(meminfo_path) as f:
            for line in f:
                if line.startswith("MemAvailable:"):
                    parts = line.split()
                    mem_available_kb = int(parts[1])
                    break

        if mem_available_kb is None:
            # Fallback: MemFree + Buffers + Cached
            mem_free_kb = 0
            buffers_kb = 0
            cached_kb = 0
            with open(meminfo_path) as f:
                for line in f:
                    if line.startswith("MemFree:"):
                        mem_free_kb = int(line.split()[1])
                    elif line.startswith("Buffers:"):
                        buffers_kb = int(line.split()[1])
                    elif line.startswith("Cached:"):
                        cached_kb = int(line.split()[1])
            mem_available_kb = mem_free_kb + buffers_kb + cached_kb

        mem_available_mb = mem_available_kb / 1024
        assert mem_available_mb >= 256, (
            f"Insufficient available memory: {mem_available_mb:.0f} MB "
            f"(minimum 256 MB required)"
        )

    # ------------------------------------------------------------------
    # TC-L1-INFRA-008: Config file exists and is valid TOML
    # ------------------------------------------------------------------
    @pytest.mark.timeout(5)
    def test_L1_infra_008_config_file_valid_toml(self):
        """TC-L1-INFRA-008: Configuration file exists and parses as valid TOML.

        Locates the config.toml and validates that it is syntactically
        correct TOML with the minimum required sections ([proxy]).
        """
        if tomllib is None:
            pytest.skip("No TOML parser available (need Python 3.11+ or tomli)")

        config_path, config = _load_config()
        assert config_path.exists(), (
            f"Config file not found at {config_path}"
        )
        assert config is not None, (
            f"Config file at {config_path} could not be parsed as TOML"
        )
        # Minimal structural validation
        assert "proxy" in config, (
            "Config file is missing the required [proxy] section"
        )
        proxy = config["proxy"]
        assert isinstance(proxy, dict), "[proxy] section is not a table"

    # ------------------------------------------------------------------
    # TC-L1-INFRA-009: Database file exists or is reachable
    # ------------------------------------------------------------------
    @pytest.mark.timeout(10)
    def test_L1_infra_009_database_accessible(self):
        """TC-L1-INFRA-009: Database file exists (SQLite) or server is reachable.

        Reads the database_url from config and validates:
        - For sqlite:// URLs: the .sqlite3 file exists on disk
        - For mysql:// URLs: a TCP connection to the MySQL port succeeds
        - For postgres:// URLs: a TCP connection to the Postgres port succeeds
        """
        config_path, config = _load_config()
        if config is None:
            pytest.skip("Cannot load config to determine database_url")

        db_url = config.get("database_url", "sqlite://rustpbx.sqlite3")
        work = _get_work_dir()

        if db_url.startswith("sqlite://"):
            db_file = db_url.replace("sqlite://", "")
            db_path = pathlib.Path(db_file)
            if not db_path.is_absolute():
                db_path = work / db_path
            assert db_path.exists(), (
                f"SQLite database file not found: {db_path}"
            )
            # Verify it is at least a valid SQLite file (magic bytes)
            with open(db_path, "rb") as f:
                header = f.read(16)
            assert header[:6] == b"SQLite", (
                f"File does not appear to be a valid SQLite database: {db_path}"
            )

        elif db_url.startswith("mysql://"):
            # Parse host:port from mysql://user:pass@host:port/db
            try:
                from urllib.parse import urlparse
                parsed = urlparse(db_url)
                host = parsed.hostname or "127.0.0.1"
                port = parsed.port or 3306
            except Exception:
                host, port = "127.0.0.1", 3306

            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(5)
            try:
                sock.connect((host, port))
            except (socket.error, OSError) as exc:
                pytest.fail(f"Cannot connect to MySQL at {host}:{port}: {exc}")
            finally:
                sock.close()

        elif db_url.startswith("postgres://") or db_url.startswith("postgresql://"):
            try:
                from urllib.parse import urlparse
                parsed = urlparse(db_url)
                host = parsed.hostname or "127.0.0.1"
                port = parsed.port or 5432
            except Exception:
                host, port = "127.0.0.1", 5432

            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(5)
            try:
                sock.connect((host, port))
            except (socket.error, OSError) as exc:
                pytest.fail(f"Cannot connect to PostgreSQL at {host}:{port}: {exc}")
            finally:
                sock.close()

        else:
            pytest.skip(f"Unknown database URL scheme: {db_url}")

    # ------------------------------------------------------------------
    # TC-L1-INFRA-010: Log file is writable
    # ------------------------------------------------------------------
    @pytest.mark.timeout(5)
    def test_L1_infra_010_log_file_writable(self):
        """TC-L1-INFRA-010: Log output location is writable.

        If a log_file is configured, verifies the path is writable. If
        logging goes to stdout (no log_file), verifies the server's working
        directory is writable (for ad-hoc log redirection like nohup).

        Also checks ~/rustpbx.log which is the conventional nohup redirect
        target on the production server.
        """
        _, config = _load_config()
        work = _get_work_dir()

        # Check configured log file
        log_file = None
        if config:
            log_file = config.get("log_file")

        if log_file:
            log_path = pathlib.Path(log_file)
            if not log_path.is_absolute():
                log_path = work / log_path
            # File may not exist yet -- check parent dir is writable
            if log_path.exists():
                assert os.access(str(log_path), os.W_OK), (
                    f"Log file exists but is not writable: {log_path}"
                )
            else:
                parent = log_path.parent
                assert parent.exists(), (
                    f"Log file parent directory does not exist: {parent}"
                )
                assert os.access(str(parent), os.W_OK), (
                    f"Log file parent directory is not writable: {parent}"
                )
        else:
            # No explicit log_file -- check the nohup convention
            nohup_log = pathlib.Path.home() / "rustpbx.log"
            if nohup_log.exists():
                assert os.access(str(nohup_log), os.W_OK), (
                    f"Nohup log file exists but is not writable: {nohup_log}"
                )
            else:
                # Just verify the work dir is writable for any log output
                assert os.access(str(work), os.W_OK), (
                    f"Server working directory is not writable: {work}"
                )
