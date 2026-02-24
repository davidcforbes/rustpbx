# RustPBX Disaster Recovery Runbook

This document provides step-by-step procedures for recovering RustPBX from
various failure scenarios. It is designed to be used during an active incident
by an operator with shell access to a replacement server and the backup
infrastructure.

**Prerequisite reading**: `docs/plans/backup-strategy.md` for system inventory,
RTO/RPO targets, and backup storage sizing estimates.

**Backup tooling reference**:
- `src/backup/mod.rs` -- Built-in `BackupService` (SQLite backup, rotation, API trigger)
- `scripts/backup.sh` -- Shell script for DB, config, and recording backup
- `scripts/backup-recordings.sh` -- rsync-based incremental recording sync
- `scripts/backup-config.sh` -- Git-based config versioning with tarballs

---

## Table of Contents

1. [Scenario 1: Complete Server Loss](#scenario-1-complete-server-loss)
2. [Scenario 2: Database Corruption](#scenario-2-database-corruption)
3. [Scenario 3: Recording Storage Failure](#scenario-3-recording-storage-failure)
4. [Scenario 4: Certificate Expiry or Loss](#scenario-4-certificate-expiry-or-loss)
5. [Scenario 5: Trunk Credential Compromise](#scenario-5-trunk-credential-compromise)
6. [Scenario 6: Partial Recovery Procedures](#scenario-6-partial-recovery-procedures)
7. [Recovery Testing Schedule](#recovery-testing-schedule)
8. [Escalation Procedures](#escalation-procedures)

---

## Conventions Used in This Document

- `$BACKUP_HOST` -- The server or path where backups are stored (e.g., `backup-host:/backups/rustpbx` or a local mount).
- `$NEW_SERVER` -- IP address or hostname of the replacement server.
- `$RUSTPBX_DIR` -- RustPBX working directory, typically `~/rustpbx`.
- `$CONFIG_DIR` -- Configuration directory, typically `~/rustpbx-config`.
- `$BACKUP_DIR` -- Backup root, typically `~/backups/rustpbx`.
- Commands assume a Debian/Ubuntu-based Linux server. Adjust package manager commands for other distributions.
- All shell commands should be run as the RustPBX service user unless otherwise noted.

---

## Scenario 1: Complete Server Loss

**Trigger**: Hardware failure, cloud instance termination, catastrophic OS failure, or data center loss.

**RTO Target**: 30 minutes

**RPO**: Database -- up to 1 hour of data loss (hourly backups). Recordings -- up to 24 hours (daily sync). Configuration -- zero (Git-versioned).

### Step 1: Provision a New Server (5 minutes)

Provision a fresh Linux server with the same architecture (x86_64) and at least
the same resources as the original.

```bash
# Example: Linode CLI
linode-cli linodes create \
    --type g6-standard-2 \
    --region us-east \
    --image linode/ubuntu22.04 \
    --root_pass "$ROOT_PASSWORD" \
    --label rustpbx-recovery

# Example: AWS CLI
aws ec2 run-instances \
    --image-id ami-0abcdef1234567890 \
    --instance-type t3.medium \
    --key-name rustpbx-key \
    --security-group-ids sg-xxxxxxxxx
```

Record the new server IP address:

```bash
export NEW_SERVER="<new-ip-address>"
```

### Step 2: Install Dependencies (5 minutes)

SSH into the new server and install prerequisites:

```bash
ssh root@$NEW_SERVER

# Update package lists
apt-get update

# Install required packages
apt-get install -y \
    sqlite3 \
    rsync \
    curl \
    jq \
    git \
    ufw \
    build-essential \
    pkg-config \
    libssl-dev

# If building from source, install the Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Step 3: Restore Configuration Files (3 minutes)

Restore configuration from the Git-versioned backup:

```bash
# Option A: Clone from remote Git repository
git clone <your-config-repo-url> ~/rustpbx-config

# Option B: Restore from backup tarball
mkdir -p ~/rustpbx-config
scp $BACKUP_HOST:/backups/rustpbx/config-tarballs/rustpbx-config-latest.tar.gz /tmp/
tar xzf /tmp/rustpbx-config-latest.tar.gz -C ~/rustpbx-config --strip-components=1

# Option C: Restore from the git-versioned config-repo backup
scp -r $BACKUP_HOST:/backups/rustpbx/config-repo/ ~/rustpbx-config/
```

Verify critical config files are present:

```bash
# Must exist
test -f ~/rustpbx-config/config.toml && echo "OK: config.toml" || echo "MISSING: config.toml"

# Check for route and trunk files
ls ~/rustpbx-config/routes/*.toml 2>/dev/null && echo "OK: route files" || echo "WARN: no route files"
ls ~/rustpbx-config/trunks/*.toml 2>/dev/null && echo "OK: trunk files" || echo "WARN: no trunk files"
```

### Step 4: Restore or Build the Binary (5-10 minutes)

```bash
# Option A: Copy pre-built binary from backup (fastest)
mkdir -p ~/rustpbx/target/release
scp $BACKUP_HOST:/backups/rustpbx/binary/rustpbx ~/rustpbx/target/release/rustpbx
chmod +x ~/rustpbx/target/release/rustpbx

# Option B: Pull Docker image
docker pull ghcr.io/restsend/rustpbx:latest

# Option C: Build from source (10+ minutes)
git clone https://github.com/restsend/rustpbx.git ~/rustpbx
cd ~/rustpbx
cargo build --release
```

If using the source build, also set up the symlink for config:

```bash
ln -s ~/rustpbx-config ~/rustpbx/rustpbx-config
```

### Step 5: Restore the Database (3 minutes)

```bash
# List available database backups (most recent first)
ssh $BACKUP_HOST "ls -lt /backups/rustpbx/db/hourly/ | head -10"

# Copy the most recent backup
scp $BACKUP_HOST:/backups/rustpbx/db/hourly/rustpbx-YYYYMMDD-HHMMSS.sqlite3 \
    ~/rustpbx/rustpbx.sqlite3

# Verify database integrity
sqlite3 ~/rustpbx/rustpbx.sqlite3 "PRAGMA integrity_check;"
# Expected output: ok

# Verify key tables exist and have data
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM sqlite_master WHERE type='table';"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM user;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM extension;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM call_record;" 2>/dev/null || echo "No call_record table (ok if fresh install)"
```

### Step 6: Restore TLS Certificates (2 minutes)

```bash
mkdir -p ~/rustpbx/config/certs

# Option A: Restore from config backup
cp ~/rustpbx-config/certs/* ~/rustpbx/config/certs/ 2>/dev/null

# Option B: Restore from dedicated cert backup
scp $BACKUP_HOST:/backups/rustpbx/certs/* ~/rustpbx/config/certs/

# Option C: Generate new self-signed certificates
openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout ~/rustpbx/config/certs/rustpbx.key \
    -out ~/rustpbx/config/certs/rustpbx.crt \
    -days 365 \
    -subj "/CN=$NEW_SERVER"

# Verify certificate files exist and are readable
ls -la ~/rustpbx/config/certs/
openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -dates
```

### Step 7: Restore Recordings (background, optional)

Recordings are not required for service operation. Start the restore in the
background while completing remaining steps:

```bash
mkdir -p ~/rustpbx/config/recorders

# Start rsync in the background (can take hours for large recording sets)
nohup rsync -az --progress \
    $BACKUP_HOST:/backups/rustpbx/recorders/ \
    ~/rustpbx/config/recorders/ \
    > ~/recording-restore.log 2>&1 &

echo "Recording restore running in background (PID: $!)"
echo "Monitor with: tail -f ~/recording-restore.log"
```

### Step 8: Update Configuration for New Server (2 minutes)

Edit `config.toml` to reflect the new server's IP address:

```bash
# Update external_ip in config.toml
# Replace OLD_IP with the previous server's IP and NEW_IP with the new one
sed -i "s/external_ip = \"OLD_IP\"/external_ip = \"$NEW_SERVER\"/" ~/rustpbx-config/config.toml

# Update realms if configured
sed -i "s/OLD_IP/$NEW_SERVER/g" ~/rustpbx-config/config.toml

# If using HTTPS, update ssl_certificate and ssl_private_key paths if they changed
# Verify the config is valid TOML
python3 -c "import tomllib; tomllib.load(open('$HOME/rustpbx-config/config.toml', 'rb'))" 2>/dev/null \
    || echo "WARN: python3 tomllib not available, skip TOML validation"
```

### Step 9: Configure Firewall (1 minute)

```bash
ufw allow 5060/udp   comment "SIP UDP"
ufw allow 5060/tcp   comment "SIP TCP"
ufw allow 8080/tcp   comment "HTTP admin console"
ufw allow 8443/tcp   comment "HTTPS admin console"
ufw allow 20000:20100/udp comment "RTP media range"
ufw --force enable
ufw status
```

Adjust the RTP port range to match `rtp_start_port` and `rtp_end_port` from
your `config.toml`.

### Step 10: Update DNS or IP References (2 minutes)

```bash
# Update DNS A record to point to $NEW_SERVER
# (provider-specific -- examples below)

# Cloudflare
curl -X PUT "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$RECORD_ID" \
    -H "Authorization: Bearer $CF_TOKEN" \
    -H "Content-Type: application/json" \
    --data "{\"type\":\"A\",\"name\":\"pbx.example.com\",\"content\":\"$NEW_SERVER\",\"ttl\":120}"

# If using a SIP trunk provider with IP-based ACL, update the allowed IP:
# - Telnyx: Dashboard > SIP Connections > IP Authentication
# - Twilio: Dashboard > SIP Trunking > IP Access Control Lists
```

### Step 11: Start RustPBX (1 minute)

```bash
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

# Wait a few seconds for startup
sleep 3

# Verify the process is running
pgrep -a rustpbx
```

### Step 12: Verification Checklist

Run through each item. All must pass before declaring recovery complete.

```bash
# 1. Process is running
pgrep rustpbx > /dev/null && echo "PASS: Process running" || echo "FAIL: Process not running"

# 2. No fatal errors in startup log
grep -i "error\|fatal\|panic" ~/rustpbx.log | head -5
# Expected: no critical errors (some transient warnings are acceptable)

# 3. SIP port is listening
ss -ulnp | grep :5060 && echo "PASS: SIP UDP listening" || echo "FAIL: SIP UDP not listening"
ss -tlnp | grep :5060 && echo "PASS: SIP TCP listening" || echo "FAIL: SIP TCP not listening"

# 4. HTTP admin console is accessible
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/console/ | grep -q 200 \
    && echo "PASS: HTTP console" || echo "FAIL: HTTP console"

# 5. HTTPS console (if configured)
curl -sk -o /dev/null -w "%{http_code}" https://localhost:8443/console/ | grep -q 200 \
    && echo "PASS: HTTPS console" || echo "FAIL: HTTPS console"

# 6. Database connectivity -- check via AMI API
curl -s http://localhost:8080/api/v1/status | jq .
# Should return JSON with system status

# 7. SIP registration test (requires a SIP client or test script)
# python3 tests/sip_test_call.py --server $NEW_SERVER --user 1001 --pass test1001 --register-only

# 8. Make a test call between two registered endpoints
# Register two softphones and place a call between them

# 9. Verify trunk connectivity
# Place an outbound call through the Telnyx trunk to a test number
```

### Step 13: Restore Backup Cron Jobs

```bash
# Re-install the cron schedule for ongoing backups
crontab -e

# Add these lines:
# 0 * * * *  ~/rustpbx/scripts/backup.sh backup-db 2>&1 | logger -t rustpbx-backup
# 0 2 * * *  ~/rustpbx/scripts/backup.sh sync-recordings 2>&1 | logger -t rustpbx-backup
# 0 1 * * *  ~/rustpbx/scripts/backup-config.sh --commit --tarball 2>&1 | logger -t rustpbx-config-backup
```

---

## Scenario 2: Database Corruption

**Trigger**: SQLite file corruption (disk errors, unclean shutdown, concurrent write conflicts), application errors returning `database disk image is malformed`, or unexpected empty query results.

**RTO Target**: 15 minutes

**RPO**: Up to 1 hour of call records and provisioning changes.

### Detection

Database corruption may manifest as:

- Application log errors: `database disk image is malformed`
- API endpoints returning 500 errors
- Admin console showing empty extension/user lists
- Call routing failures with database-related errors
- `PRAGMA integrity_check` returning errors

Run a manual integrity check:

```bash
sqlite3 ~/rustpbx/rustpbx.sqlite3 "PRAGMA integrity_check;"
```

Expected output for a healthy database: `ok`

Any other output indicates corruption. Common error patterns:
- `*** in database main ***` -- page-level corruption
- `row X missing from index` -- index corruption (may be repairable)
- `database disk image is malformed` -- severe corruption

### Recovery Procedure

#### Step 1: Stop the RustPBX Service

```bash
# Graceful stop
kill $(pgrep rustpbx)

# Wait for process to exit (up to 10 seconds)
for i in $(seq 1 10); do
    pgrep rustpbx > /dev/null || break
    sleep 1
done

# Force kill if still running
pgrep rustpbx > /dev/null && kill -9 $(pgrep rustpbx)

# Verify it is stopped
pgrep rustpbx > /dev/null && echo "FAIL: still running" || echo "OK: stopped"
```

#### Step 2: Preserve the Corrupted Database

Always save the corrupted database for post-incident analysis:

```bash
TIMESTAMP=$(date '+%Y%m%d-%H%M%S')
cp ~/rustpbx/rustpbx.sqlite3 ~/rustpbx/rustpbx.sqlite3.corrupted-$TIMESTAMP
echo "Corrupted database saved as: rustpbx.sqlite3.corrupted-$TIMESTAMP"
```

#### Step 3: Attempt Repair (Optional, if corruption is minor)

For index-only corruption, SQLite can sometimes repair itself:

```bash
sqlite3 ~/rustpbx/rustpbx.sqlite3 "REINDEX;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "PRAGMA integrity_check;"
```

If `REINDEX` fixes the issue, you can skip the restore and proceed to Step 6.

For more severe corruption, try a dump-and-reload:

```bash
sqlite3 ~/rustpbx/rustpbx.sqlite3 ".dump" > /tmp/rustpbx-dump.sql
sqlite3 ~/rustpbx/rustpbx.sqlite3.recovered < /tmp/rustpbx-dump.sql
sqlite3 ~/rustpbx/rustpbx.sqlite3.recovered "PRAGMA integrity_check;"
```

If the recovered database passes integrity check, use it:

```bash
cp ~/rustpbx/rustpbx.sqlite3.recovered ~/rustpbx/rustpbx.sqlite3
```

Otherwise, proceed to restore from backup.

#### Step 4: Identify and Restore the Latest Good Backup

```bash
# List available hourly backups (most recent first)
ls -lt ~/backups/rustpbx/db/hourly/ | head -10

# Or if using the built-in BackupService
ls -lt ~/rustpbx/backups/ | head -10

# Select the most recent backup
BACKUP_FILE=$(ls -t ~/backups/rustpbx/db/hourly/*.sqlite3 | head -1)
echo "Restoring from: $BACKUP_FILE"

# Verify the backup itself is not corrupted
sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;"
# Must output: ok

# Restore
cp "$BACKUP_FILE" ~/rustpbx/rustpbx.sqlite3
```

Alternatively, use the `backup.sh` restore command:

```bash
~/rustpbx/scripts/backup.sh restore-db "$BACKUP_FILE"
```

This script will:
1. Check that RustPBX is stopped
2. Create a safety backup of the current (corrupted) database
3. Copy the backup file into place
4. Verify the restored database with `SELECT count(*) FROM sqlite_master`

#### Step 5: Verify the Restored Database

```bash
sqlite3 ~/rustpbx/rustpbx.sqlite3 "PRAGMA integrity_check;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM user;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM extension;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM sip_trunk;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT count(*) FROM routing;"
sqlite3 ~/rustpbx/rustpbx.sqlite3 "SELECT max(created_at) FROM call_record;" 2>/dev/null
echo "Note: calls between $(basename $BACKUP_FILE) and the corruption event are lost."
```

#### Step 6: Start RustPBX

```bash
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
sleep 3
pgrep rustpbx > /dev/null && echo "OK: RustPBX started" || echo "FAIL: RustPBX did not start"
```

#### Step 7: Verify Service

```bash
# Check for database errors in log
grep -i "database\|sqlite\|malformed" ~/rustpbx.log | tail -5

# Test API
curl -s http://localhost:8080/api/v1/status | jq .

# Test admin console
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/console/
```

### If No Backup Is Available: Rebuild from Config

If no database backup exists, RustPBX can recreate its database schema on
startup from scratch. You will need to re-provision users and extensions:

```bash
# Remove the corrupted database (schema will be recreated on startup)
rm ~/rustpbx/rustpbx.sqlite3

# Start RustPBX -- SeaORM migrations will create all tables
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
sleep 3

# Re-create the admin superuser
cd ~/rustpbx
./target/release/rustpbx \
    --conf ~/rustpbx-config/config.toml \
    --super-username admin \
    --super-password admin123 \
    --super-email admin@rustpbx.local

# Memory-backend users (defined in config.toml [[proxy.user_backends]])
# are available automatically. Database-managed extensions must be
# re-created via the admin console or API.

# Trunks defined in config.toml [proxy.trunks.*] sections are
# available automatically. Database-managed trunks must be re-created.

# Routes defined in external route files (config/routes/*.toml) are
# loaded automatically. Database-managed routes must be re-created.
```

**Data loss**: All call records (CDR), presence state, frequency limits, and
any database-managed extensions/trunks/routes are lost. CDR JSON files in
`./config/cdr/` may still contain per-call records that can be reviewed
manually.

---

## Scenario 3: Recording Storage Failure

**Trigger**: Disk full, I/O errors on the recording volume, filesystem corruption, or mount point failure.

**RTO Target**: 5 minutes (to restore call service; recording restore is background)

**Impact**: Active calls may fail to record. Call routing and SIP operations are
not affected unless the storage failure causes broader system instability.

### Detection

Signs of recording storage failure:

- Application log warnings: `Failed to create recording file`, `No space left on device`, `I/O error`
- New calls are not producing `.wav` files in the recording directory
- `df -h` shows the recording volume at 100% usage
- Filesystem read-only mount after I/O errors

```bash
# Check disk space
df -h $(dirname $(grep -A5 '^\[recording\]' ~/rustpbx-config/config.toml | grep path | sed 's/.*= *"//;s/"//' || echo "./config/recorders"))

# Check for I/O errors
dmesg | tail -20 | grep -i "error\|fault\|readonly"

# Check if filesystem is mounted read-only
mount | grep "$(df --output=source ~/rustpbx/config/recorders 2>/dev/null | tail -1)" | grep "ro,"
```

### Immediate Action: Disable Recording to Preserve Call Service

If recordings are causing call failures, disable them immediately:

```bash
# Option 1: Edit config and restart (preferred for persistent change)
# Set recording.enabled = false in config.toml
sed -i 's/^auto_start = true/auto_start = false/' ~/rustpbx-config/config.toml

# Restart RustPBX
kill $(pgrep rustpbx)
sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

# Option 2: Free disk space immediately (if disk full)
# Remove oldest recordings to free space
ls -t ~/rustpbx/config/recorders/*.wav | tail -100 | xargs rm -f
echo "Removed 100 oldest recordings to free space"
df -h ~/rustpbx/config/recorders/
```

### Recovery: Expand Storage or Mount New Volume

```bash
# Option A: Resize existing volume (cloud-provider specific)
# Linode: Dashboard > Volumes > Resize
# AWS: aws ec2 modify-volume --volume-id vol-xxx --size 100
# Then resize the filesystem:
resize2fs /dev/sdX

# Option B: Mount a new volume for recordings
mkdir -p /mnt/recordings
mount /dev/sdY /mnt/recordings
# Add to fstab for persistence
echo "/dev/sdY /mnt/recordings ext4 defaults 0 2" >> /etc/fstab
# Update config.toml to point to new location
# [recording]
# path = "/mnt/recordings"

# Option C: Clean up disk space
# Remove SIP flow traces (diagnostic, lowest priority)
find ~/rustpbx/config/sipflow/ -mtime +7 -delete
# Remove old CDR JSON files already in backup
find ~/rustpbx/config/cdr/ -mtime +30 -delete
# Compress old recordings
find ~/rustpbx/config/recorders/ -name '*.wav' -mtime +90 -exec gzip {} \;
```

### Restore Recordings from Backup

```bash
# Restore from rsync backup server
rsync -az --progress \
    $BACKUP_HOST:/backups/rustpbx/recorders/ \
    ~/rustpbx/config/recorders/

# Or from S3
aws s3 sync s3://$BACKUP_S3_BUCKET/recorders/ ~/rustpbx/config/recorders/
```

### Re-enable Recording

Once storage is healthy:

```bash
# Re-enable recording in config
sed -i 's/^auto_start = false/auto_start = true/' ~/rustpbx-config/config.toml

# Restart RustPBX
kill $(pgrep rustpbx) && sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

# Verify recordings are being created for new calls
sleep 10  # Wait for a test call
ls -lt ~/rustpbx/config/recorders/ | head -5
```

### Prevention

Add disk space monitoring to cron:

```bash
# Add to crontab: alert when recording volume exceeds 85%
*/15 * * * * df -h ~/rustpbx/config/recorders | awk 'NR==2{gsub(/%/,""); if($5>85) print "WARN: Recording disk at "$5"%"}' | logger -t rustpbx-disk
```

---

## Scenario 4: Certificate Expiry or Loss

**Trigger**: TLS certificate has expired, certificate or private key file is
missing/corrupted, or HTTPS/WebRTC connections are failing with TLS errors.

**RTO Target**: 10 minutes

**Impact**: HTTPS admin console inaccessible, WebRTC browser phones fail to
connect, SIP-over-TLS connections fail. SIP over UDP/TCP and the HTTP console
continue to function.

### Detection

Symptoms of certificate problems:

- Browser shows `ERR_CERT_DATE_INVALID` or `NET::ERR_CERT_AUTHORITY_INVALID`
- Application log errors: `TLS handshake failed`, `certificate has expired`
- WebRTC softphones fail to connect via WSS
- `curl -k` works but `curl` without `-k` fails

```bash
# Check certificate expiry date
openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -enddate
# Output example: notAfter=Mar 15 00:00:00 2026 GMT

# Check if certificate has expired
openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -checkend 0
# Exit code 0 = not expired, exit code 1 = expired

# Test the HTTPS endpoint directly
openssl s_client -connect localhost:8443 -brief < /dev/null 2>&1 | head -5

# Verify certificate and key match
CERT_MOD=$(openssl x509 -noout -modulus -in ~/rustpbx/config/certs/rustpbx.crt | md5sum)
KEY_MOD=$(openssl rsa -noout -modulus -in ~/rustpbx/config/certs/rustpbx.key | md5sum)
[ "$CERT_MOD" = "$KEY_MOD" ] && echo "PASS: cert/key match" || echo "FAIL: cert/key mismatch"
```

### Recovery Option A: Generate New Self-Signed Certificate

Fastest recovery for development and internal deployments:

```bash
# Generate a new self-signed certificate (valid for 1 year)
openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout ~/rustpbx/config/certs/rustpbx.key \
    -out ~/rustpbx/config/certs/rustpbx.crt \
    -days 365 \
    -subj "/CN=$(hostname -f)" \
    -addext "subjectAltName=IP:$(curl -s ifconfig.me),IP:127.0.0.1,DNS:$(hostname -f)"

# Set correct permissions
chmod 600 ~/rustpbx/config/certs/rustpbx.key
chmod 644 ~/rustpbx/config/certs/rustpbx.crt

# Restart RustPBX to pick up the new certificate
kill $(pgrep rustpbx) && sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

# Verify
sleep 3
curl -sk https://localhost:8443/console/ -o /dev/null -w "%{http_code}\n"
```

Note: Clients will need to accept the new self-signed certificate. For Chrome,
the browser may need to be relaunched or the old certificate removed from
trusted roots.

### Recovery Option B: Restore Certificate from Backup

```bash
# Restore from config backup
scp $BACKUP_HOST:/backups/rustpbx/config-repo/certs/rustpbx.crt ~/rustpbx/config/certs/
scp $BACKUP_HOST:/backups/rustpbx/config-repo/certs/rustpbx.key ~/rustpbx/config/certs/

# Or extract from a config tarball
tar xzf /path/to/rustpbx-config-YYYYMMDD.tar.gz -C /tmp rustpbx-config/certs/
cp /tmp/rustpbx-config/certs/* ~/rustpbx/config/certs/

# Set correct permissions
chmod 600 ~/rustpbx/config/certs/rustpbx.key
chmod 644 ~/rustpbx/config/certs/rustpbx.crt

# Restart RustPBX
kill $(pgrep rustpbx) && sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
```

### Recovery Option C: ACME / Let's Encrypt Re-provisioning

If the deployment uses the ACME addon for automatic certificate management:

```bash
# The ACME addon will automatically re-provision on startup
# Ensure the domain DNS points to this server
dig +short pbx.example.com
# Must return the current server IP

# Restart RustPBX -- the ACME addon will request a new certificate
kill $(pgrep rustpbx) && sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

# Monitor the log for ACME activity
grep -i "acme\|certificate\|letsencrypt" ~/rustpbx.log
```

### Prevention: Certificate Expiry Monitoring

Add a cron job to alert before certificate expiry:

```bash
# Alert 30 days before expiry (add to crontab)
0 9 * * * openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -checkend 2592000 || echo "ALERT: RustPBX TLS certificate expires within 30 days" | mail -s "Cert Expiry Warning" admin@example.com

# Alert 7 days before expiry (higher urgency)
0 9 * * * openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -checkend 604800 || echo "URGENT: RustPBX TLS certificate expires within 7 days" | mail -s "URGENT: Cert Expiry" admin@example.com
```

### Verification

```bash
# 1. HTTPS is responding
curl -sk https://localhost:8443/console/ -o /dev/null -w "%{http_code}\n"
# Expected: 200

# 2. Certificate is not expired
openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -checkend 0 \
    && echo "PASS: cert valid" || echo "FAIL: cert expired"

# 3. Certificate and key match
CERT_MOD=$(openssl x509 -noout -modulus -in ~/rustpbx/config/certs/rustpbx.crt | md5sum)
KEY_MOD=$(openssl rsa -noout -modulus -in ~/rustpbx/config/certs/rustpbx.key | md5sum)
[ "$CERT_MOD" = "$KEY_MOD" ] && echo "PASS: cert/key match" || echo "FAIL: mismatch"

# 4. WebRTC connectivity (register a browser phone and verify connection)
```

---

## Scenario 5: Trunk Credential Compromise

**Trigger**: SIP trunk credentials (username/password) have been leaked,
unauthorized calls are observed in CDR, or the trunk provider alerts you to
suspicious activity.

**RTO Target**: 5 minutes (to stop unauthorized usage)

**Impact**: Financial loss from unauthorized outbound calls (toll fraud),
potential regulatory exposure.

### Immediate Action: Disable the Trunk (1 minute)

Stop all traffic through the compromised trunk immediately:

```bash
# Option 1: Comment out the trunk in config and restart
# In config.toml, comment out or remove the [proxy.trunks.telnyx] section
sed -i 's/^\[proxy\.trunks\.telnyx\]/# DISABLED - credential compromise\n# [proxy.trunks.telnyx]/' ~/rustpbx-config/config.toml

# If trunk is defined in an external file
mv ~/rustpbx-config/trunks/telnyx.toml ~/rustpbx-config/trunks/telnyx.toml.disabled

# Restart to apply
kill $(pgrep rustpbx) && sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

# Verify the trunk is no longer active
curl -s http://localhost:8080/api/v1/trunks | jq .
```

### Step 2: Rotate Credentials at the Provider (2 minutes)

Log into the trunk provider's dashboard and change the credentials:

**Telnyx**:
1. Log into https://portal.telnyx.com
2. Navigate to SIP Connections
3. Select the compromised connection
4. Generate new credentials or update the password
5. Note the new username and password

**Generic SIP provider**:
1. Log into the provider's portal
2. Navigate to SIP authentication settings
3. Change the password (use a strong random password)
4. If IP-based ACL is available, restrict to your server's IP only

```bash
# Generate a strong random password
openssl rand -base64 24
```

### Step 3: Update Config with New Credentials (1 minute)

```bash
# Edit config.toml with the new credentials
# Update the username and password under [proxy.trunks.telnyx]
vi ~/rustpbx-config/config.toml

# Or if using an external trunk file
vi ~/rustpbx-config/trunks/telnyx.toml

# Re-enable the trunk section (uncomment)
# Restart RustPBX
kill $(pgrep rustpbx) && sleep 2
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
```

### Step 4: Verify Trunk Operation

```bash
# Check that the trunk is registered/active
grep -i "trunk\|register\|telnyx" ~/rustpbx.log | tail -10

# Place a test outbound call to verify the new credentials work
# python3 tests/sip_test_call.py --server localhost --user 1001 --pass test1001 --dest +1XXXXXXXXXX
```

### Step 5: Audit for Unauthorized Calls

Review call records for any unauthorized usage:

```bash
# Check CDR database for unusual call patterns
sqlite3 ~/rustpbx/rustpbx.sqlite3 <<'SQL'
-- Calls in the last 24 hours through the trunk
SELECT created_at, caller, callee, duration, hangup_cause
FROM call_record
WHERE created_at > datetime('now', '-1 day')
ORDER BY created_at DESC
LIMIT 50;
SQL

# Check CDR JSON files for the same period
find ~/rustpbx/config/cdr/ -name '*.json' -mtime -1 | while read f; do
    jq -r '[.created_at, .caller, .callee, .duration] | @tsv' "$f" 2>/dev/null
done | sort

# Look for calls to premium rate numbers, international destinations,
# or destinations you do not normally call
sqlite3 ~/rustpbx/rustpbx.sqlite3 <<'SQL'
SELECT callee, count(*) as count, sum(duration) as total_duration
FROM call_record
WHERE created_at > datetime('now', '-7 days')
GROUP BY callee
ORDER BY count DESC
LIMIT 20;
SQL
```

### Step 6: Harden Against Future Compromise

```bash
# 1. Add IP-based ACL at the trunk provider (restrict to your server IP only)
echo "Action: Add IP ACL at trunk provider for $(curl -s ifconfig.me)"

# 2. Enable frequency limiting in RustPBX config
# Add to config.toml:
# [proxy]
# max_cps = 5          # maximum calls per second
# max_concurrent = 10  # maximum concurrent calls

# 3. Set up CDR alerting for unusual call patterns
# (integrate with monitoring system)

# 4. Commit the new config to version control
cd ~/rustpbx-config && git add -A && git commit -m "Rotate trunk credentials after compromise"
```

---

## Scenario 6: Partial Recovery Procedures

These procedures restore individual components when the rest of the system
is intact. They are faster than full recovery because they skip unnecessary
steps.

### 6.1 Database-Only Restore

**When to use**: Database is corrupted or lost, but config files and recordings
are intact on the same server.

```bash
# 1. Stop RustPBX
kill $(pgrep rustpbx)
sleep 2

# 2. Backup the current (corrupted) database
TIMESTAMP=$(date '+%Y%m%d-%H%M%S')
[ -f ~/rustpbx/rustpbx.sqlite3 ] && \
    mv ~/rustpbx/rustpbx.sqlite3 ~/rustpbx/rustpbx.sqlite3.bad-$TIMESTAMP

# 3. Find the latest good backup
BACKUP_FILE=$(ls -t ~/backups/rustpbx/db/hourly/*.sqlite3 2>/dev/null | head -1)
if [ -z "$BACKUP_FILE" ]; then
    # Try the built-in backup directory
    BACKUP_FILE=$(ls -t ~/rustpbx/backups/*.sqlite3 2>/dev/null | head -1)
fi

if [ -z "$BACKUP_FILE" ]; then
    echo "ERROR: No backup found. See 'Rebuild from Config' in Scenario 2."
    exit 1
fi

echo "Restoring from: $BACKUP_FILE"

# 4. Verify and restore
sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;" | grep -q "ok" || {
    echo "ERROR: Backup file is also corrupted. Try an older backup."
    exit 1
}
cp "$BACKUP_FILE" ~/rustpbx/rustpbx.sqlite3

# 5. Start RustPBX
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
sleep 3

# 6. Verify
pgrep rustpbx > /dev/null && echo "OK" || echo "FAIL"
curl -s http://localhost:8080/api/v1/status | jq .
```

**Time estimate**: 5 minutes.

### 6.2 Configuration-Only Restore

**When to use**: Config files are corrupted, accidentally deleted, or need
rollback to a previous version. Database and recordings are intact.

```bash
# 1. Stop RustPBX
kill $(pgrep rustpbx)
sleep 2

# Option A: Restore from Git (fastest, preferred)
cd ~/rustpbx-config
git log --oneline -10           # Review recent commits
git checkout HEAD -- .           # Restore all files to latest commit
# Or rollback to a specific commit:
# git checkout <commit-hash> -- .

# Option B: Restore from config tarball
TARBALL=$(ls -t ~/backups/rustpbx/config-tarballs/rustpbx-config-*.tar.gz | head -1)
echo "Restoring from: $TARBALL"
tar xzf "$TARBALL" -C /tmp
cp /tmp/rustpbx-config/config.toml ~/rustpbx-config/config.toml
cp -r /tmp/rustpbx-config/routes/ ~/rustpbx-config/routes/ 2>/dev/null
cp -r /tmp/rustpbx-config/trunks/ ~/rustpbx-config/trunks/ 2>/dev/null
cp -r /tmp/rustpbx-config/certs/ ~/rustpbx/config/certs/ 2>/dev/null

# Option C: Restore from config-repo backup
scp -r $BACKUP_HOST:/backups/rustpbx/config-repo/ /tmp/config-restore/
cp /tmp/config-restore/config.toml ~/rustpbx-config/config.toml
cp -r /tmp/config-restore/routes/ ~/rustpbx-config/routes/ 2>/dev/null
cp -r /tmp/config-restore/trunks/ ~/rustpbx-config/trunks/ 2>/dev/null

# 2. Verify config is valid
python3 -c "import tomllib; tomllib.load(open('$HOME/rustpbx-config/config.toml', 'rb'))" 2>/dev/null \
    && echo "OK: config.toml is valid TOML" \
    || echo "WARN: could not validate TOML (python3 tomllib not available)"

# 3. Start RustPBX
cd ~/rustpbx
nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &
sleep 3

# 4. Verify
pgrep rustpbx > /dev/null && echo "OK" || echo "FAIL"
grep -i "error\|fatal" ~/rustpbx.log | head -5
curl -s http://localhost:8080/api/v1/status | jq .
```

**Time estimate**: 3 minutes.

### 6.3 Recordings-Only Restore

**When to use**: Recording files are lost or corrupted, but the database (which
references them) and config files are intact.

```bash
# No service restart needed -- recordings are served on demand.

# 1. Restore from rsync backup
rsync -az --progress \
    $BACKUP_HOST:/backups/rustpbx/recorders/ \
    ~/rustpbx/config/recorders/

# Or from S3
aws s3 sync s3://$BACKUP_S3_BUCKET/recorders/ ~/rustpbx/config/recorders/

# Or from local backup
rsync -a ~/backups/rustpbx/recorders/ ~/rustpbx/config/recorders/

# 2. Verify sample recordings are playable
SAMPLE=$(ls ~/rustpbx/config/recorders/*.wav 2>/dev/null | head -1)
if [ -n "$SAMPLE" ]; then
    file "$SAMPLE"
    # Expected: RIFF (little-endian) data, WAVE audio, mu-law, stereo 8000 Hz
    echo "OK: sample recording verified"
else
    echo "WARN: no recordings found after restore"
fi

# 3. Restore transcript sidecars (included with recordings if backed up together)
ls ~/rustpbx/config/recorders/*.transcript.json 2>/dev/null | wc -l
echo "transcript sidecar files restored"

# 4. If transcripts are missing, re-run transcription from the admin console
#    for specific calls as needed.
```

**Time estimate**: Varies with recording volume (minutes to hours).

---

## Recovery Testing Schedule

Regular testing ensures that backup and recovery procedures actually work when
needed. Untested backups are not backups.

### Quarterly DR Drill Checklist

Perform these tests every 3 months. Document results and any issues found.

#### Test 1: Database Restore Verification

| Step | Action | Success Criteria |
|------|--------|------------------|
| 1 | Identify the latest hourly backup | File exists and is less than 2 hours old |
| 2 | Copy backup to a temporary location | `cp` succeeds |
| 3 | Run `PRAGMA integrity_check` on the copy | Returns `ok` |
| 4 | Query key tables: `user`, `extension`, `sip_trunk`, `routing` | Row counts match expected values |
| 5 | Query `call_record` for the most recent entry | Timestamp is within expected range |
| 6 | (Optional) Start a test RustPBX instance against the backup | Service starts without errors |

```bash
# Automated test script
BACKUP=$(ls -t ~/backups/rustpbx/db/hourly/*.sqlite3 | head -1)
TEMP_DB="/tmp/dr-test-$(date +%s).sqlite3"
cp "$BACKUP" "$TEMP_DB"

echo "=== DR Test: Database Restore ==="
echo "Backup file: $BACKUP"
echo "Backup age: $(stat -c '%Y' "$BACKUP" | xargs -I{} bash -c 'echo $(( ($(date +%s) - {}) / 60 )) minutes')"

INTEGRITY=$(sqlite3 "$TEMP_DB" "PRAGMA integrity_check;")
echo "Integrity: $INTEGRITY"
[ "$INTEGRITY" = "ok" ] && echo "PASS" || echo "FAIL"

for table in user extension sip_trunk routing call_record; do
    COUNT=$(sqlite3 "$TEMP_DB" "SELECT count(*) FROM $table;" 2>/dev/null || echo "N/A")
    echo "Table '$table': $COUNT rows"
done

rm -f "$TEMP_DB"
```

#### Test 2: Configuration Restore Verification

| Step | Action | Success Criteria |
|------|--------|------------------|
| 1 | List Git commits in config backup repo | At least 1 commit exists within the last 7 days |
| 2 | Checkout the latest commit to a temp dir | All expected files are present |
| 3 | Verify `config.toml` is valid TOML | Parse succeeds |
| 4 | Verify route and trunk files exist | At least 1 route file, trunk config present |
| 5 | Verify TLS certificates are present and not expired | `openssl x509 -checkend 0` passes |

```bash
echo "=== DR Test: Config Restore ==="

# Check Git repo freshness
if [ -d ~/backups/rustpbx/config-repo/.git ]; then
    LAST_COMMIT=$(git -C ~/backups/rustpbx/config-repo log -1 --format='%ci')
    echo "Last config commit: $LAST_COMMIT"
else
    echo "WARN: No config git repo found"
fi

# Check tarball freshness
LATEST_TAR=$(ls -t ~/backups/rustpbx/config-tarballs/*.tar.gz 2>/dev/null | head -1)
if [ -n "$LATEST_TAR" ]; then
    echo "Latest tarball: $LATEST_TAR"
    echo "Tarball age: $(stat -c '%Y' "$LATEST_TAR" | xargs -I{} bash -c 'echo $(( ($(date +%s) - {}) / 3600 )) hours')"
else
    echo "WARN: No config tarballs found"
fi

# Check cert expiry
if [ -f ~/rustpbx/config/certs/rustpbx.crt ]; then
    openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -checkend 2592000 \
        && echo "PASS: cert valid for >30 days" \
        || echo "WARN: cert expires within 30 days"
fi
```

#### Test 3: Recording Restore Verification

| Step | Action | Success Criteria |
|------|--------|------------------|
| 1 | Check backup freshness | Most recent recording in backup is less than 24 hours old |
| 2 | Select 3 random recordings from backup | Files exist and are non-zero size |
| 3 | Verify WAV format | `file` reports valid WAVE audio |
| 4 | Verify transcript sidecars (if applicable) | Corresponding `.transcript.json` exists |

```bash
echo "=== DR Test: Recording Backup ==="

BACKUP_REC="${BACKUP_HOST:-~/backups/rustpbx}/recorders"
if [ -d "$BACKUP_REC" ]; then
    LATEST=$(ls -t "$BACKUP_REC"/*.wav 2>/dev/null | head -1)
    echo "Latest backed-up recording: $LATEST"
    COUNT=$(ls "$BACKUP_REC"/*.wav 2>/dev/null | wc -l)
    echo "Total recordings in backup: $COUNT"

    # Spot-check 3 random files
    ls "$BACKUP_REC"/*.wav 2>/dev/null | shuf | head -3 | while read f; do
        SIZE=$(stat -c%s "$f")
        FORMAT=$(file "$f" | grep -o 'WAVE audio.*')
        echo "  $(basename $f): $SIZE bytes, $FORMAT"
    done
else
    echo "WARN: No recording backup directory found at $BACKUP_REC"
fi
```

#### Test 4: Full System Recovery (Annual or Semi-Annual)

| Step | Action | Success Criteria |
|------|--------|------------------|
| 1 | Provision a fresh test server | Server is accessible via SSH |
| 2 | Follow Scenario 1 end-to-end | All 12 steps complete without errors |
| 3 | Register a SIP phone and make a test call | Call connects and audio is bidirectional |
| 4 | Verify admin console access | Console loads, shows expected extensions |
| 5 | Verify recording and playback | New recording is created, playable in console |
| 6 | Measure total recovery time | Must be under 30 minutes |
| 7 | Tear down test server | Server deprovisioned |

Record the actual time for each step. Compare against RTO targets. Update
this runbook if any steps take longer than expected or if procedures need
correction.

### Testing Schedule Summary

| Test | Frequency | Owner | Last Tested | Next Due |
|------|-----------|-------|-------------|----------|
| Database restore | Quarterly | _______________ | _______________ | _______________ |
| Config restore | Quarterly | _______________ | _______________ | _______________ |
| Recording restore | Quarterly | _______________ | _______________ | _______________ |
| Full system recovery | Semi-annually | _______________ | _______________ | _______________ |
| Backup script health | Weekly (automated) | Cron | _______________ | Ongoing |
| Certificate expiry | Daily (automated) | Cron | _______________ | Ongoing |

---

## Escalation Procedures

### Severity Levels

| Level | Name | Description | Response Time | Examples |
|-------|------|-------------|---------------|----------|
| **SEV-1** | Critical | Complete service outage. No calls can be placed or received. | Immediate (< 15 min) | Server loss, total DB corruption, all trunks down |
| **SEV-2** | Major | Partial service degradation. Some calls affected. | < 30 min | Single trunk down, recording failure, HTTPS only down |
| **SEV-3** | Minor | Non-critical feature affected. Core call service operational. | < 4 hours | Transcript failures, CDR export errors, SIP trace loss |
| **SEV-4** | Low | Cosmetic or informational. No user impact. | Next business day | Backup rotation warning, disk space approaching threshold |

### Escalation Matrix

Fill in the contact details for your deployment:

| Role | Name | Phone | Email | When to Contact |
|------|------|-------|-------|-----------------|
| Primary On-Call | _______________ | _______________ | _______________ | SEV-1, SEV-2: immediately |
| Secondary On-Call | _______________ | _______________ | _______________ | SEV-1: if primary unreachable after 15 min |
| System Administrator | _______________ | _______________ | _______________ | Server provisioning, network, firewall |
| SIP Trunk Provider | _______________ | _______________ | _______________ | Trunk outages, credential rotation |
| Management | _______________ | _______________ | _______________ | SEV-1: after 30 min. SEV-2: after 2 hours |

### Communication Plan

#### During an Incident

1. **Acknowledge**: Within 5 minutes of detection, send an initial notification
   to the on-call team with:
   - What is affected (calls, recordings, admin console, etc.)
   - Estimated severity level
   - Who is investigating

2. **Status Updates**: Every 15 minutes during SEV-1, every 30 minutes during
   SEV-2, provide an update:
   - Current status (investigating / identified / fixing / monitoring)
   - Root cause (if known)
   - Estimated time to resolution
   - Impact scope (number of users affected, calls dropped, etc.)

3. **Resolution**: When service is restored:
   - Confirm all verification checks pass
   - Send an all-clear notification
   - Note the total outage duration and data loss (if any)

4. **Post-Incident Review** (within 48 hours of SEV-1 or SEV-2):
   - Timeline of events
   - Root cause analysis
   - What worked well in the recovery
   - What could be improved
   - Action items to prevent recurrence

#### Notification Template

```
Subject: [RustPBX] SEV-{1|2|3} - {Brief description}

Status: {Investigating | Identified | Fixing | Monitoring | Resolved}
Start time: YYYY-MM-DD HH:MM UTC
Duration: {X minutes / ongoing}

Impact:
- {Description of user impact}

Current actions:
- {What is being done}

Next update: {time}

Contact: {on-call name, phone}
```

### Key Resources Quick Reference

Keep this section updated with deployment-specific details:

| Resource | Location / URL |
|----------|---------------|
| RustPBX binary | `~/rustpbx/target/release/rustpbx` |
| Main config file | `~/rustpbx-config/config.toml` |
| Database file | `~/rustpbx/rustpbx.sqlite3` |
| Recordings | `~/rustpbx/config/recorders/` |
| TLS certificates | `~/rustpbx/config/certs/` |
| Application log | `~/rustpbx.log` |
| Backup directory | `~/backups/rustpbx/` |
| Backup status | `~/backups/rustpbx/backup-status.json` |
| Config Git repo | `~/backups/rustpbx/config-repo/` |
| Backup script | `~/rustpbx/scripts/backup.sh` |
| Admin console | `https://<server-ip>:8443/console/` |
| AMI API | `http://<server-ip>:8080/api/v1/` |
| SIP trunk dashboard | (provider-specific URL) |
| Cloud provider console | (provider-specific URL) |
| This runbook | `docs/plans/disaster-recovery-runbook.md` |

---

## Appendix A: Quick Reference Command Card

Print this section and keep it accessible for rapid incident response.

```
=== STOP SERVICE ===
kill $(pgrep rustpbx)

=== START SERVICE ===
cd ~/rustpbx && nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

=== CHECK SERVICE ===
pgrep -a rustpbx
curl -s http://localhost:8080/api/v1/status | jq .
tail -20 ~/rustpbx.log

=== CHECK DATABASE ===
sqlite3 ~/rustpbx/rustpbx.sqlite3 "PRAGMA integrity_check;"

=== LIST BACKUPS ===
~/rustpbx/scripts/backup.sh status
ls -lt ~/backups/rustpbx/db/hourly/ | head -5

=== RESTORE DATABASE ===
~/rustpbx/scripts/backup.sh restore-db <backup-file>

=== TRIGGER MANUAL BACKUP ===
~/rustpbx/scripts/backup.sh backup-all

=== CHECK CERT EXPIRY ===
openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -enddate

=== CHECK DISK SPACE ===
df -h ~/rustpbx/config/recorders/ ~/rustpbx/

=== CHECK FIREWALL ===
ufw status

=== CHECK SIP PORTS ===
ss -ulnp | grep :5060
ss -tlnp | grep :5060
```

---

## Appendix B: Backup Verification One-Liner

Run this daily (or add to cron) to verify all backup components are healthy:

```bash
#!/usr/bin/env bash
# backup-health-check.sh -- Quick health check for all backup components
PASS=0; FAIL=0; WARN=0

check() {
    local label="$1"; local result="$2"
    if [ "$result" = "PASS" ]; then
        echo "  [PASS] $label"; PASS=$((PASS+1))
    elif [ "$result" = "WARN" ]; then
        echo "  [WARN] $label"; WARN=$((WARN+1))
    else
        echo "  [FAIL] $label"; FAIL=$((FAIL+1))
    fi
}

echo "=== RustPBX Backup Health Check ==="
echo "Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""

# Database backup freshness (must be < 2 hours old)
LATEST_DB=$(ls -t ~/backups/rustpbx/db/hourly/*.sqlite3 2>/dev/null | head -1)
if [ -n "$LATEST_DB" ]; then
    AGE=$(( ($(date +%s) - $(stat -c '%Y' "$LATEST_DB")) / 60 ))
    [ "$AGE" -lt 120 ] && check "DB backup age: ${AGE}m" "PASS" || check "DB backup age: ${AGE}m (>2h)" "FAIL"
else
    check "DB backup exists" "FAIL"
fi

# Database backup integrity
if [ -n "$LATEST_DB" ]; then
    INTEGRITY=$(sqlite3 "$LATEST_DB" "PRAGMA integrity_check;" 2>/dev/null)
    [ "$INTEGRITY" = "ok" ] && check "DB backup integrity" "PASS" || check "DB backup integrity: $INTEGRITY" "FAIL"
fi

# Config backup freshness (must be < 48 hours old)
LATEST_CFG=$(ls -t ~/backups/rustpbx/config-tarballs/*.tar.gz 2>/dev/null | head -1)
if [ -n "$LATEST_CFG" ]; then
    AGE=$(( ($(date +%s) - $(stat -c '%Y' "$LATEST_CFG")) / 3600 ))
    [ "$AGE" -lt 48 ] && check "Config backup age: ${AGE}h" "PASS" || check "Config backup age: ${AGE}h (>48h)" "WARN"
else
    check "Config backup exists" "WARN"
fi

# Certificate expiry (must be > 30 days)
if [ -f ~/rustpbx/config/certs/rustpbx.crt ]; then
    openssl x509 -in ~/rustpbx/config/certs/rustpbx.crt -noout -checkend 2592000 2>/dev/null \
        && check "TLS cert valid >30d" "PASS" || check "TLS cert expires within 30d" "WARN"
else
    check "TLS cert exists" "WARN"
fi

# Disk space (recording volume must be < 85%)
REC_USAGE=$(df ~/rustpbx/config/recorders 2>/dev/null | awk 'NR==2{gsub(/%/,""); print $5}')
if [ -n "$REC_USAGE" ]; then
    [ "$REC_USAGE" -lt 85 ] && check "Disk usage: ${REC_USAGE}%" "PASS" || check "Disk usage: ${REC_USAGE}% (>85%)" "WARN"
fi

# Service running
pgrep rustpbx > /dev/null && check "RustPBX process running" "PASS" || check "RustPBX process running" "FAIL"

echo ""
echo "Summary: $PASS passed, $WARN warnings, $FAIL failures"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
```

---

## Document History

| Date | Author | Change |
|------|--------|--------|
| 2026-02-24 | _______________ | Initial version |

**Review schedule**: This document should be reviewed and updated:
- After every DR drill
- After every actual incident
- When backup infrastructure changes
- At minimum, every 6 months
