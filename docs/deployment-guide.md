# RustPBX Deployment and Operations Guide

This guide covers installation, configuration, deployment, and day-to-day operations for RustPBX in both development and production environments.

---

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation from Source](#installation-from-source)
3. [Installation from Docker](#installation-from-docker)
4. [Initial Configuration](#initial-configuration)
5. [TLS/HTTPS Setup](#tlshttps-setup)
6. [Database Setup](#database-setup)
7. [SIP Trunk Configuration](#sip-trunk-configuration)
8. [Extension and User Setup](#extension-and-user-setup)
9. [Route Configuration](#route-configuration)
10. [Recording Setup](#recording-setup)
11. [Transcription Setup](#transcription-setup)
12. [Monitoring and Health Checks](#monitoring-and-health-checks)
13. [Log Management](#log-management)
14. [Backup Procedures](#backup-procedures)
15. [Running as a Service](#running-as-a-service)
16. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Operating System

- **Linux** (recommended): Debian 12 (Bookworm), Ubuntu 22.04+, RHEL 9+, or any modern Linux distribution.
- **macOS**: Supported for development.
- **Windows**: Supported for development (WSL2 recommended for production-like testing).

### Hardware

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 1 core | 2+ cores |
| RAM | 512 MB | 2+ GB |
| Disk | 1 GB | 20+ GB (depends on recording volume) |

### Network Ports

The following ports must be accessible (open in firewall/security groups):

| Port | Protocol | Service | Required |
|------|----------|---------|----------|
| 5060 | UDP/TCP | SIP signaling | Yes |
| 5061 | TCP | SIP TLS (SIPS) | Optional |
| 8080 | TCP | HTTP API and console | Yes |
| 8443 | TCP | HTTPS API and console | Optional |
| 8089 | TCP | WebSocket (SIP/WebRTC) | Optional |
| 20000-20100 | UDP | RTP media (configurable range) | Yes |

**Important**: For production, use a focused RTP port range (e.g., 20000-20100) rather than the default large range (12000-42000). Open only the ports you actually need.

### Build Dependencies

**Linux (Debian/Ubuntu):**

```bash
apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libasound2-dev \
    libopus-dev \
    libssl-dev \
    git \
    curl
```

**Linux (RHEL/Fedora):**

```bash
dnf install -y \
    gcc gcc-c++ cmake \
    pkg-config \
    alsa-lib-devel \
    opus-devel \
    openssl-devel \
    git
```

**macOS:**

```bash
brew install cmake opus
```

### Runtime Dependencies

For the Docker image or binary deployment, only these runtime libraries are needed:

- `ca-certificates`
- `libopus0` (Debian) / `opus` (RHEL)
- `tzdata`

---

## Installation from Source

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Ensure Rust 1.75 or later:

```bash
rustup update stable
rustc --version
```

### 2. Clone the Repository

```bash
git clone https://github.com/restsend/rustpbx.git
cd rustpbx
```

### 3. Build

**Full build (all default features):**

```bash
cargo build --release
```

The binary is produced at `target/release/rustpbx`.

**Minimal build (reduced binary size):**

```bash
cargo build --release --no-default-features --features console
```

**Features available:**

| Feature | Default | Description |
|---------|---------|-------------|
| `opus` | Yes | Opus audio codec support. |
| `console` | Yes | Web management console. |
| `addon-acme` | Yes | Let's Encrypt certificate automation. |
| `addon-transcript` | Yes | Post-call transcription. |
| `addon-archive` | Yes | CDR archival. |
| `addon-wholesale` | No | Wholesale billing (commercial). |
| `addon-endpoint-manager` | No | Endpoint management (commercial). |
| `addon-enterprise-auth` | No | Enterprise authentication (commercial). |
| `commerce` | No | All commercial features combined. |

### 4. Verify the Build

```bash
./target/release/rustpbx --version
```

### 5. Validate Configuration

```bash
./target/release/rustpbx --conf config.toml check-config
```

This checks that all configured ports are available and the configuration syntax is valid without starting the server.

---

## Installation from Docker

### Pull the Image

**Community Edition:**

```bash
docker pull ghcr.io/restsend/rustpbx:latest
```

**Commerce Edition (includes wholesale features):**

```bash
docker pull docker.cnb.cool/miuda.ai/rustpbx:latest
```

### Prepare Directories

```bash
mkdir -p db config recorders
```

### Create Configuration

Create a `config.toml` file (see [Initial Configuration](#initial-configuration) below).

### Run the Container

```bash
docker run -d \
  --name rustpbx \
  --net host \
  -v $(pwd)/db:/app/db \
  -v $(pwd)/config.toml:/app/config.toml \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/recorders:/app/config/recorders \
  ghcr.io/restsend/rustpbx:latest \
  --conf /app/config.toml
```

**Notes on Docker networking:**
- `--net host` is strongly recommended for SIP/RTP. It avoids NAT issues between the container and host.
- If you cannot use host networking, you must map all SIP and RTP ports explicitly and set `external_ip` to the host's IP address.
- Set the timezone: add `-e TZ=America/New_York` to the docker run command.

### Create a Super Admin (Docker)

```bash
docker exec rustpbx /app/rustpbx \
  --conf /app/config.toml \
  --super-username admin \
  --super-password YourSecurePassword \
  --super-email admin@example.com
```

### Docker on Windows (Git Bash)

When running Docker commands from Git Bash on Windows, prefix with `MSYS_NO_PATHCONV=1` to prevent path mangling:

```bash
MSYS_NO_PATHCONV=1 docker run -d \
  --name rustpbx \
  --net host \
  -v $(pwd)/config.toml:/app/config.toml \
  ghcr.io/restsend/rustpbx:latest \
  --conf /app/config.toml
```

---

## Initial Configuration

### Minimal Configuration

Create a `config.toml` with the following minimal setup:

```toml
http_addr = "0.0.0.0:8080"
database_url = "sqlite://rustpbx.sqlite3"

[console]
base_path = "/console"
allow_registration = false

[proxy]
addr = "0.0.0.0"
udp_port = 5060
modules = ["auth", "registrar", "call"]

# In-memory test users
[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "password1001" },
    { username = "1002", password = "password1002" },
]
```

### Start the Server

```bash
./target/release/rustpbx --conf config.toml
```

### Create the First Admin

In a separate terminal:

```bash
./target/release/rustpbx --conf config.toml \
  --super-username admin \
  --super-password admin123 \
  --super-email admin@example.com
```

### Access the Console

Open `http://your-server:8080/console/` in a browser and log in with the super admin credentials.

### NAT Configuration

If your server is behind NAT (common for cloud deployments), you **must** set the `external_ip`:

```toml
# Your server's public IP address (no port)
external_ip = "203.0.113.10"

# Use a focused RTP port range and open it in your firewall
rtp_start_port = 20000
rtp_end_port = 20100
```

Also add your server's public IP to the realms so SIP authentication works:

```toml
[proxy]
realms = ["203.0.113.10"]
```

---

## TLS/HTTPS Setup

### Method 1: ACME (Let's Encrypt) via Built-in Addon

RustPBX includes a built-in ACME addon for automated Let's Encrypt certificates.

1. Ensure your server has a public DNS record pointing to it.
2. Ensure port 80 is accessible (for HTTP-01 challenge).
3. Navigate to the console: **SSL Certificates** sidebar item.
4. Enter your domain and request a certificate.

The ACME addon handles the challenge at `/.well-known/acme-challenge/` automatically.

### Method 2: Manual TLS Certificate

1. Obtain certificates (e.g., from Let's Encrypt using certbot, or self-signed):

```bash
# Let's Encrypt with certbot
certbot certonly --standalone -d pbx.example.com

# Or generate self-signed (for testing only)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=pbx.example.com"
```

2. Configure HTTPS in `config.toml`:

```toml
https_addr = "0.0.0.0:8443"
ssl_certificate = "/etc/letsencrypt/live/pbx.example.com/fullchain.pem"
ssl_private_key = "/etc/letsencrypt/live/pbx.example.com/privkey.pem"
```

3. For SIP TLS, also set certificates in the proxy section:

```toml
[proxy]
tls_port = 5061
ssl_certificate = "/etc/letsencrypt/live/pbx.example.com/fullchain.pem"
ssl_private_key = "/etc/letsencrypt/live/pbx.example.com/privkey.pem"
```

4. Enable secure cookies in the console:

```toml
[console]
secure_cookie = true
```

### Self-Signed Certificates for Development

For local WebRTC testing with Chrome, you can either:

- Install the self-signed CA cert in your system's trusted root store, or
- Launch Chrome with the flag: `--unsafely-treat-insecure-origin-as-secure="https://your-server:8443"`

---

## Database Setup

### SQLite (Default, Recommended for Single Server)

SQLite is the default and requires no additional setup:

```toml
database_url = "sqlite://rustpbx.sqlite3"
```

The database file is created automatically on first start. For production, use an absolute path:

```toml
database_url = "sqlite:///var/lib/rustpbx/rustpbx.sqlite3"
```

### MySQL

```toml
database_url = "mysql://user:password@localhost:3306/rustpbx"
```

Create the database first:

```sql
CREATE DATABASE rustpbx CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'rustpbx'@'localhost' IDENTIFIED BY 'your-password';
GRANT ALL PRIVILEGES ON rustpbx.* TO 'rustpbx'@'localhost';
FLUSH PRIVILEGES;
```

RustPBX handles schema migrations automatically on startup.

### PostgreSQL

```toml
database_url = "postgres://user:password@localhost:5432/rustpbx"
```

Create the database first:

```sql
CREATE DATABASE rustpbx;
CREATE USER rustpbx WITH PASSWORD 'your-password';
GRANT ALL PRIVILEGES ON DATABASE rustpbx TO rustpbx;
```

---

## SIP Trunk Configuration

### Telnyx Example

1. Create a SIP Connection in the Telnyx portal (credential authentication mode).
2. Note the username and password.
3. Configure the trunk:

```toml
[proxy.trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
transport = "udp"
username = "your-telnyx-sip-username"
password = "your-telnyx-sip-password"
direction = "bidirectional"
max_calls = 50
max_cps = 5
codec = ["PCMU", "PCMA"]
```

4. For inbound call delivery, enable trunk registration:

```toml
[proxy.trunks.telnyx]
# ... (above fields)
register = true
register_expires = 3600
```

### External Trunk File

For cleaner organization, define trunks in separate files:

```toml
# config/trunks/telnyx.toml
[trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
username = "your-username"
password = "your-password"
direction = "bidirectional"
```

And reference them from the main config:

```toml
[proxy]
trunks_files = ["config/trunks/*.toml"]
```

### Outbound Route for the Trunk

Create a route to send calls through the trunk:

```toml
# config/routes/telnyx.toml
[[routes]]
name = "Telnyx Outbound"
priority = 10
direction = "outbound"

[routes.match]
"to.user" = "^\\+?1[2-9][0-9]{9}$"

dest = "telnyx"

[routes.rewrite]
"from.user" = "+17072833106"
```

### Inbound Route from the Trunk

```toml
[[routes]]
name = "Telnyx Inbound"
priority = 20
direction = "inbound"
source_trunks = ["telnyx"]

[routes.match]
"to.user" = "^\\+?17072833106$"

[routes.rewrite]
"to.user" = "1001"
```

---

## Extension and User Setup

### Via Configuration File (Memory Backend)

For simple deployments, define users directly:

```toml
[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "secret1001", display_name = "Alice" },
    { username = "1002", password = "secret1002", display_name = "Bob" },
    { username = "1003", password = "secret1003", display_name = "Charlie" },
]
```

### Via Web Console (Extension Backend)

For dynamic management through the web console:

1. Enable the extension backend:

```toml
[[proxy.user_backends]]
type = "extension"
```

2. Log in to the console at `http://your-server:8080/console/`.
3. Navigate to **Extensions**.
4. Click **Add Extension** and fill in the details.

Extensions created through the console are stored in the database and persist across restarts.

### Combining Backends

You can chain multiple backends. They are queried in order:

```toml
# First check memory users (always available, fast)
[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "secret1001" },
]

# Then check database extensions (managed via console)
[[proxy.user_backends]]
type = "extension"
```

### Realm Configuration

The `realms` setting is critical for SIP authentication. Users must register with a realm that matches one of the configured values:

```toml
[proxy]
realms = ["10.0.0.55", "pbx.example.com"]
```

If no realms are configured, the proxy uses the request's host as the realm. This can cause authentication failures when the SIP client connects to an IP that does not match the user's realm.

---

## Route Configuration

### Managing Routes via Console

The web console provides a route editor (navigate to **Routes**) where you can create, edit, and reorder routes without editing files.

### Route Evaluation Order

1. Routes are sorted by `priority` (higher value = evaluated first).
2. Within the same priority, routes are evaluated in definition order.
3. The first matching route is used; subsequent routes are skipped.

### Common Routing Patterns

**Internal calls (extension to extension):**

```toml
[[proxy.routes]]
name = "Internal Calls"
priority = 100

[proxy.routes.match]
"to.user" = "^[0-9]{3,4}$"

# No dest = forward to registered user
```

**Outbound to PSTN via trunk:**

```toml
[[proxy.routes]]
name = "Outbound PSTN"
priority = 10
direction = "outbound"

[proxy.routes.match]
"to.user" = "^\\+?1[2-9][0-9]{9}$"

dest = "telnyx"
```

**Reject specific patterns:**

```toml
[[proxy.routes]]
name = "Block Premium"
priority = 50

[proxy.routes.match]
"to.user" = "^1900"

action = "reject"

[proxy.routes.reject]
code = 403
reason = "Premium numbers not allowed"
```

**Load-balanced outbound with multiple trunks:**

```toml
[[proxy.routes]]
name = "Load Balanced Outbound"
priority = 10

[proxy.routes.match]
"to.user" = "^\\+?1"

dest = ["trunk-a", "trunk-b", "trunk-c"]
select = "rr"                     # round-robin
```

---

## Recording Setup

### Enable Recording

```toml
[recording]
enabled = true
auto_start = true
path = "./config/recorders"
directions = ["inbound", "outbound", "internal"]
```

### Configure CDR Storage

```toml
[callrecord]
type = "local"
root = "./config/cdr"
```

### Per-Trunk Recording

You can override recording settings per trunk:

```toml
[proxy.trunks.telnyx.recording]
enabled = true
auto_start = true
directions = ["inbound", "outbound"]
```

### Ensure Directories Exist

```bash
mkdir -p config/recorders config/cdr
```

### Recording File Format

Recordings are stored as WAV files by default (8kHz, mu-law, stereo). The left channel contains the caller audio and the right channel contains the callee audio.

If the `opus` feature is compiled in, OGG/Opus format may also be available.

---

## Transcription Setup

### Prerequisites

You need an external transcription command-line tool. Options include:

- **groq-sensevoice-wrapper**: Wrapper around the Groq Whisper API.
- **sensevoice-cli**: Local SenseVoice ASR model.
- Any CLI tool that accepts a WAV file and outputs text.

### Configuration

```toml
[proxy]
addons = ["transcript"]

[proxy.transcript]
command = "groq-sensevoice-wrapper"
models_path = "/opt/models"           # For local model tools
timeout_secs = 120
```

### Enable in Console

1. Navigate to the console.
2. Open **Call Records**.
3. Click on a completed call.
4. The transcription appears automatically if the addon is configured and the call was recorded.

### Verify Transcription

After a recorded call completes, check the call detail view in the console. If transcription fails, check the server logs for error messages from the transcription command.

---

## Monitoring and Health Checks

### SIP Flow Monitoring

Enable SIP flow capture for detailed call tracing:

```toml
[sipflow]
type = "local"
root = "./config/sipflow"
subdirs = "daily"
```

SIP flow data is viewable in the console under **Call Records** (click a call to see the SIP message flow).

### Quality Monitoring

```toml
[quality]
enabled = true
watchdog_interval_secs = 2
loss_warning_pct = 1.0
loss_critical_pct = 5.0
jitter_warning_ms = 30.0
jitter_critical_ms = 50.0
```

Quality metrics (MOS score, packet loss, jitter) are tracked per call and viewable in call records.

### HTTP Health Check

The HTTP server responds on the configured `http_addr`. You can use `http_access_skip_paths` to suppress access logs for health check endpoints:

```toml
http_access_skip_paths = ["/health"]
```

A basic health check (from a load balancer or monitoring system):

```bash
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/console/
```

### Process Monitoring

Monitor the `rustpbx` process with standard tools:

```bash
# Check if running
pgrep -x rustpbx

# Check resource usage
ps aux | grep rustpbx

# Check open ports
ss -tlnp | grep rustpbx
ss -ulnp | grep rustpbx
```

### AMI Event Stream

For integration with monitoring systems that support the Asterisk Manager Interface:

```toml
[ami]
allows = ["127.0.0.1", "10.0.0.0/8"]
```

Connect via TCP to receive real-time events about registrations, calls, and channel states.

---

## Log Management

### Configuration

```toml
log_level = "info"            # debug, info, warn, error
log_file = "/var/log/rustpbx.log"
```

If `log_file` is not set, logs go to stderr (suitable for systemd journal or Docker logging).

### Log Levels

| Level | Use Case |
|-------|----------|
| `error` | Production (minimal output). |
| `warn` | Production (includes warnings). |
| `info` | Standard production logging. |
| `debug` | Development and troubleshooting. |
| `trace` | Extremely verbose, for deep debugging only. |

### Environment-Based Override

You can override the log level at runtime using the `RUST_LOG` environment variable:

```bash
RUST_LOG=rustpbx=debug,sqlx=info ./target/release/rustpbx --conf config.toml
```

### Log Rotation

When using a log file, set up external rotation with logrotate:

```
# /etc/logrotate.d/rustpbx
/var/log/rustpbx.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    copytruncate
}
```

The `copytruncate` directive avoids needing to send a signal to RustPBX, since the process keeps the file open.

---

## Backup Procedures

RustPBX includes a backup script at `scripts/backup.sh` that handles database, configuration, and recording backups.

### Setup

```bash
chmod +x scripts/backup.sh
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUSTPBX_DIR` | `~/rustpbx` | RustPBX working directory. |
| `RUSTPBX_CONFIG` | `~/rustpbx-config/config.toml` | Path to config file. |
| `BACKUP_DIR` | `~/backups/rustpbx` | Base backup directory. |
| `BACKUP_REMOTE` | _(none)_ | rsync target for offsite backup. |
| `BACKUP_S3_BUCKET` | _(none)_ | S3 bucket for recording sync. |
| `DATABASE_URL` | from config | Override database URL. |
| `SQLITE_CMD` | `sqlite3` | Path to sqlite3 binary. |

### Backup Commands

```bash
# Back up the database
./scripts/backup.sh backup-db

# Snapshot configuration files
./scripts/backup.sh backup-config

# Sync recordings to backup location
./scripts/backup.sh sync-recordings

# Run all backup tasks
./scripts/backup.sh backup-all

# Show backup status
./scripts/backup.sh status

# Restore database from a backup file
./scripts/backup.sh restore-db /path/to/backup.sqlite3
```

### Cron Schedule

Set up automated backups:

```bash
# Edit crontab
crontab -e
```

Add these entries:

```cron
# Hourly database backup
0 * * * * /home/user/rustpbx/scripts/backup.sh backup-db 2>&1 | logger -t rustpbx-backup

# Daily recording sync at 2 AM
0 2 * * * /home/user/rustpbx/scripts/backup.sh sync-recordings 2>&1 | logger -t rustpbx-backup

# Daily full backup at 3 AM
0 3 * * * /home/user/rustpbx/scripts/backup.sh backup-all 2>&1 | logger -t rustpbx-backup
```

### Retention Policy

The backup script maintains a tiered retention policy:

| Tier | Retention |
|------|-----------|
| Hourly | Last 24 backups |
| Daily | Last 7 backups |
| Weekly | Last 4 backups |
| Monthly | Last 12 backups |

Hourly backups are automatically promoted to daily (at midnight), weekly (on Sundays), and monthly (on the 1st).

### Database Restore

**Important**: Stop RustPBX before restoring:

```bash
# Stop the service
kill $(pgrep rustpbx)

# Restore
./scripts/backup.sh restore-db ~/backups/rustpbx/db/hourly/rustpbx-20260223-120000.sqlite3

# Restart the service
cd ~/rustpbx && ./target/release/rustpbx --conf ~/rustpbx-config/config.toml
```

For PostgreSQL, use `pg_restore` directly:

```bash
pg_restore -d postgres://user:pass@localhost/rustpbx backup.pgdump
```

---

## Running as a Service

### Systemd (Linux)

Create `/etc/systemd/system/rustpbx.service`:

```ini
[Unit]
Description=RustPBX SIP PBX Server
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=rustpbx
Group=rustpbx
WorkingDirectory=/opt/rustpbx
ExecStart=/opt/rustpbx/rustpbx --conf /etc/rustpbx/config.toml
Restart=always
RestartSec=5
LimitNOFILE=65535

# Environment
Environment=RUST_LOG=rustpbx=info,sqlx=warn

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/rustpbx /var/log/rustpbx /opt/rustpbx/config

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
# Create service user
useradd --system --no-create-home --shell /usr/sbin/nologin rustpbx

# Create directories
mkdir -p /opt/rustpbx /var/lib/rustpbx /var/log/rustpbx /etc/rustpbx
cp target/release/rustpbx /opt/rustpbx/
cp -r templates static config /opt/rustpbx/
cp config.toml /etc/rustpbx/
chown -R rustpbx:rustpbx /opt/rustpbx /var/lib/rustpbx /var/log/rustpbx

# Enable and start
systemctl daemon-reload
systemctl enable rustpbx
systemctl start rustpbx

# Check status
systemctl status rustpbx
journalctl -u rustpbx -f
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: "3.8"

services:
  rustpbx:
    image: ghcr.io/restsend/rustpbx:latest
    container_name: rustpbx
    network_mode: host
    restart: unless-stopped
    environment:
      - TZ=America/New_York
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./db:/app/db
      - ./config:/app/config
      - ./recorders:/app/config/recorders
    command: ["--conf", "/app/config.toml"]
```

Run:

```bash
docker compose up -d
docker compose logs -f
```

### Running in Background (Simple)

For quick setups without systemd:

```bash
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
```

---

## Troubleshooting

### SIP Registration Fails with 401 Unauthorized

**Cause**: Realm mismatch. The SIP client is connecting to an address not in the configured realms.

**Fix**: Add your server's IP (and/or domain) to the realms list:

```toml
[proxy]
realms = ["10.0.0.55", "your-public-ip", "pbx.example.com"]
```

### No Audio in Calls

**Cause 1**: Missing `external_ip` when behind NAT.

**Fix**: Set `external_ip` to your public IP address:

```toml
external_ip = "203.0.113.10"
```

**Cause 2**: RTP ports blocked by firewall.

**Fix**: Open the RTP port range in your firewall:

```bash
# UFW
ufw allow 20000:20100/udp

# iptables
iptables -A INPUT -p udp --dport 20000:20100 -j ACCEPT

# firewalld
firewall-cmd --permanent --add-port=20000-20100/udp
firewall-cmd --reload
```

### WebRTC Browser Phone Fails to Connect

**Cause 1**: HTTPS not enabled. WebRTC requires a secure context.

**Fix**: Enable HTTPS (see [TLS/HTTPS Setup](#tlshttps-setup)) or launch Chrome with:

```bash
chrome --unsafely-treat-insecure-origin-as-secure="http://your-server:8080"
```

**Cause 2**: Missing ICE server configuration.

**Fix**: Add STUN servers:

```toml
[[ice_servers]]
urls = ["stun:stun.l.google.com:19302"]
```

### Outbound Trunk Calls Fail

**Cause 1**: Incorrect trunk credentials.

**Fix**: Verify username and password with your SIP provider.

**Cause 2**: Firewall blocking outbound SIP.

**Fix**: Ensure outbound UDP port 5060 is allowed, and the trunk destination can be reached:

```bash
# Test connectivity
nc -zu sip.telnyx.com 5060
```

**Cause 3**: NAT issues causing ACK routing failures.

**Fix**: Ensure `nat_fix = true` (the default) is set in the proxy config.

### Console Pages Show Empty Data

**Cause**: Console reads from the database, not from config files. Extensions, trunks, and routes defined only in TOML files will not appear in the console lists.

**Fix**: Use the console UI to create resources, or use both config files and the extension backend together. Config file resources take effect for call routing but are not displayed in the console management pages.

### Port Already in Use

**Error**: `Address 0.0.0.0:5060 is unavailable (Address already in use)`

**Fix**: Check what process is using the port:

```bash
ss -tulnp | grep 5060
# or
lsof -i :5060
```

Stop the conflicting process or change the port in your configuration.

### Database Migration Errors

RustPBX runs migrations automatically. If they fail:

1. Check the logs for specific error messages.
2. Ensure the database URL is correct and the database server is reachable.
3. For SQLite, ensure the directory is writable.
4. Try backing up and deleting the database file to start fresh (SQLite only).

### Recordings Not Being Created

1. Verify recording is enabled:

```toml
[recording]
enabled = true
auto_start = true
```

2. Ensure the recording directory exists and is writable:

```bash
mkdir -p config/recorders
```

3. Check that the `directions` list includes the call direction you want to record.

### Configuration Reload

RustPBX supports hot-reload for trunks, routes, queues, and ACLs through the console UI. For other configuration changes, a restart is required.

The server also supports automatic retry on startup failures (up to 10 retries with 5-second intervals) and graceful restart via the reload mechanism in the console.

### SIGTERM and Graceful Shutdown

RustPBX handles `SIGTERM` and `Ctrl+C` for graceful shutdown. Active calls are allowed to complete their current transactions before the server exits.

```bash
# Graceful stop
kill -TERM $(pgrep rustpbx)

# Check if stopped
pgrep rustpbx
```
