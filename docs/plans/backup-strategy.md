# RustPBX Backup Strategy and Disaster Recovery

## 1. System Inventory

RustPBX persists state across several distinct subsystems. Each has different
storage characteristics, growth rates, and criticality levels.

### 1.1 SQLite / MySQL / PostgreSQL Database

| Item | Details |
|------|---------|
| Default URL | `sqlite://rustpbx.sqlite3` (relative to working dir) |
| Alternatives | PostgreSQL (`postgres://...`), MySQL (`mysql://...`) |
| Config key | `database_url` in config.toml |
| Contents | Users, extensions, departments, SIP trunks, routing rules, call records (CDR), presence, frequency limits, queue state, transcript metadata |
| Growth rate | Moderate -- grows with call volume (one CDR row per call) |
| Criticality | **Critical** -- loss means loss of all provisioned config and call history |

The database schema is managed by SeaORM migrations defined in
`src/models/migration.rs`. Tables include: `user`, `department`, `extension`,
`extension_department`, `sip_trunk`, `presence`, `routing`, `queue`,
`call_record`, and `frequency_limit`. Several index migrations optimize CDR
queries. Transcript text is stored inline in the `call_record` table via the
`transcript_text` column.

### 1.2 Recording Files (WAV)

| Item | Details |
|------|---------|
| Default path | `./config/recorders/` |
| Config key | `[recording].path` or falls back to `default_config_recorder_path()` |
| Format | WAV (mu-law or A-law encoded, stereo, 8kHz) |
| Naming | `{call_id}.wav` |
| Growth rate | **High** -- ~480 KB/min per call at 8kHz stereo mu-law |
| Criticality | Important for compliance; not required for system operation |

### 1.3 Transcript Sidecar Files

| Item | Details |
|------|---------|
| Location | Next to recording file, e.g. `{call_id}.transcript.json` |
| Contents | Structured JSON with segments, timestamps, channel labels, analysis |
| Growth rate | Small per file (~2-20 KB each) |
| Criticality | Moderate -- can be regenerated from recordings if transcription tooling is available |

### 1.4 Call Detail Records (CDR JSON)

| Item | Details |
|------|---------|
| Default path | `./config/cdr/` |
| Config key | `[callrecord].root` |
| Structure | `{root}/{YYYYMMDD}/{call_id}.json` |
| Alternatives | S3 upload, HTTP webhook |
| Growth rate | Low per file (~1-5 KB each) |
| Criticality | Important -- primary audit trail alongside database CDR |

### 1.5 SIP Flow Traces

| Item | Details |
|------|---------|
| Default path | `./config/sipflow/` |
| Config key | `[sipflow].root` |
| Structure | Daily or hourly subdirectories |
| Growth rate | Moderate -- depends on call volume and flush settings |
| Criticality | Low -- diagnostic data, useful for troubleshooting |

### 1.6 Configuration Files

| Item | Details |
|------|---------|
| Main config | `config.toml` (path passed via `--conf`) |
| Route files | `config/routes/*.toml` (loaded via `routes_files` glob) |
| Trunk files | `config/trunks/*.toml` (loaded via `trunks_files` glob) |
| ACL files | `config/acl/` |
| Queue config | `config/queue/` |
| Sound files | `config/sounds/*.wav` |
| Growth rate | Rarely changes |
| Criticality | **Critical** -- required to start the system |

### 1.7 TLS Certificates

| Item | Details |
|------|---------|
| Config keys | `ssl_certificate`, `ssl_private_key`, `https_addr` |
| ACME addon | Can auto-provision via Let's Encrypt (addon-acme feature) |
| Typical path | `config/certs/` |
| Criticality | **Critical** for HTTPS/TLS -- without them, WebRTC and secure admin console fail |

### 1.8 Voicemail Files (Future)

Not yet implemented. When added, voicemail will likely follow the same pattern
as recordings (WAV files in a configurable directory). The backup strategy
should accommodate this when the feature lands.

---

## 2. RTO / RPO Targets

Recovery Time Objective (RTO) = maximum acceptable downtime.
Recovery Point Objective (RPO) = maximum acceptable data loss window.

| Component | RPO | RTO | Rationale |
|-----------|-----|-----|-----------|
| **Database** | 1 hour | 15 minutes | Contains all provisioning and CDR. Hourly snapshots limit data loss. SQLite restore is fast (single file copy). |
| **Configuration files** | 0 (real-time) | 5 minutes | Version-controlled in Git. Any committed state can be restored instantly. |
| **TLS certificates** | 0 (real-time) | 5 minutes | Stored in Git or backed up with config. ACME addon can re-provision if needed. |
| **Recordings** | 24 hours | 4 hours | Large files; daily sync to backup is acceptable. Not required for call routing. |
| **Transcripts** | 24 hours | 4 hours | Sidecar to recordings; same schedule. Can be regenerated. |
| **CDR JSON files** | 24 hours | 4 hours | Supplementary to database CDR. Daily sync sufficient. |
| **SIP flow traces** | 7 days | 24 hours | Diagnostic only. Weekly backup or best-effort. |
| **Voicemail (future)** | 24 hours | 4 hours | Same tier as recordings. |

---

## 3. Backup Strategies

### 3.1 Database Backup

**SQLite** (default and most common deployment):

```bash
# Online backup using SQLite .backup command (safe during writes)
sqlite3 /path/to/rustpbx.sqlite3 ".backup '/path/to/backup/rustpbx-$(date +%Y%m%d-%H%M%S).sqlite3'"
```

The SQLite `.backup` API creates a consistent snapshot even while the database
is being written to. This is the recommended approach over raw file copies.

**PostgreSQL**:

```bash
pg_dump -Fc -f /path/to/backup/rustpbx-$(date +%Y%m%d-%H%M%S).pgdump "$DATABASE_URL"
```

**MySQL**:

```bash
mysqldump --single-transaction -u user -p dbname > /path/to/backup/rustpbx-$(date +%Y%m%d-%H%M%S).sql
```

**Schedule**: Hourly via cron, with the rotation policy described in Section 4.

### 3.2 Recording and Transcript Backup

Use `rsync` to an offsite backup server or `aws s3 sync` to object storage:

```bash
# rsync to backup server
rsync -az --delete ./config/recorders/ backup-host:/backups/rustpbx/recorders/

# Or S3 sync
aws s3 sync ./config/recorders/ s3://my-bucket/rustpbx/recorders/ --storage-class STANDARD_IA
```

RustPBX also natively supports S3 upload for call records via the
`[callrecord]` config section with `type = "s3"`. If configured, CDR JSON and
optionally media files are uploaded automatically, reducing the need for
separate backup of those files.

**Schedule**: Daily via cron (aligns with 24-hour RPO).

### 3.3 Configuration Backup

Configuration files should be managed in a Git repository (as this project
already is). The backup strategy is simply:

1. All config files (`config.toml`, `config/routes/*.toml`, `config/trunks/*.toml`)
   are committed to Git.
2. Sensitive values (trunk passwords, database credentials) should use
   environment variables or a secrets manager rather than being committed
   in plaintext.
3. Push to a remote Git repository (GitHub, GitLab, etc.) for offsite backup.

**RPO**: Effectively zero for any committed state.

### 3.4 TLS Certificate Backup

- If using ACME/Let's Encrypt: certificates can be re-provisioned automatically.
  Back up the account key if re-provisioning is undesirable.
- If using manually provisioned certificates: store them in the Git repo
  (encrypted with `git-crypt` or similar) or in a secrets manager.
- Back up `config/certs/` alongside configuration files.

### 3.5 Full System Backup

For comprehensive disaster recovery, periodic full-system snapshots complement
component-level backups:

- **VM/cloud snapshots**: Use provider-specific snapshot tools (Linode snapshots,
  AWS EBS snapshots, LVM snapshots).
- **Schedule**: Weekly or before major changes.
- **Retention**: Keep at least 2 recent snapshots.

---

## 4. Implementation Plan

### 4.1 Database Backup Script with Rotation

The `scripts/backup.sh` script implements database and configuration backup
with the following rotation policy:

| Tier | Retention | Schedule |
|------|-----------|----------|
| Hourly | Last 24 hourly backups | Every hour |
| Daily | Last 7 daily backups | Once per day (midnight) |
| Weekly | Last 4 weekly backups | Sunday midnight |
| Monthly | Last 12 monthly backups | 1st of month |

Hourly backups are promoted to daily/weekly/monthly tiers by creating hard
links, so no extra disk space is used for identical files.

### 4.2 Recording Sync Script

A companion section in `scripts/backup.sh` handles recording sync:

```bash
# Example cron entry for daily recording sync
0 2 * * * /path/to/rustpbx/scripts/backup.sh sync-recordings
```

### 4.3 Monitoring and Alerting

The backup script:

- Writes a status file (`backup-status.json`) after each run with timestamp,
  success/failure, and backup sizes.
- Exits with non-zero status on failure, compatible with cron email alerts.
- Logs to syslog or a dedicated backup log file.

Recommended monitoring integration:

```bash
# Cron with email on failure
MAILTO=admin@example.com
0 * * * * /path/to/rustpbx/scripts/backup.sh backup-db 2>&1 | logger -t rustpbx-backup
0 2 * * * /path/to/rustpbx/scripts/backup.sh sync-recordings 2>&1 | logger -t rustpbx-backup
```

### 4.4 Restore Procedures

See Section 5 (Disaster Recovery Runbook) for step-by-step restore procedures.

---

## 5. Disaster Recovery Runbook

### 5.1 Full System Restore (from scratch)

Prerequisites: A fresh Linux server with the same architecture.

```
Step 1: Install prerequisites
    - Install Rust toolchain (or use pre-built binary)
    - Install SQLite3 CLI tools
    - Install rsync, curl, jq

Step 2: Restore configuration
    git clone <your-config-repo> ~/rustpbx-config

Step 3: Restore the binary
    Option A: Build from source
        git clone https://github.com/restsend/rustpbx.git ~/rustpbx
        cd ~/rustpbx && cargo build --release

    Option B: Copy pre-built binary
        scp backup-host:/backups/rustpbx/binary/rustpbx ~/rustpbx/target/release/rustpbx
        chmod +x ~/rustpbx/target/release/rustpbx

Step 4: Restore the database
    cp /backups/rustpbx/db/latest/rustpbx.sqlite3 ~/rustpbx/rustpbx.sqlite3

Step 5: Restore recordings (if needed)
    rsync -az backup-host:/backups/rustpbx/recorders/ ~/rustpbx/config/recorders/

Step 6: Restore TLS certificates
    cp /backups/rustpbx/certs/* ~/rustpbx/config/certs/

Step 7: Configure firewall
    ufw allow 5060/udp    # SIP
    ufw allow 5060/tcp    # SIP over TCP
    ufw allow 8080/tcp    # HTTP admin
    ufw allow 8443/tcp    # HTTPS admin
    ufw allow 20000:20100/udp  # RTP media

Step 8: Start RustPBX
    cd ~/rustpbx
    nohup ./target/release/rustpbx --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

Step 9: Verify
    - Check logs: tail -f ~/rustpbx.log
    - Test SIP registration: register a phone
    - Test call routing: make a test call
    - Check admin console: https://<server-ip>:8443/console
```

### 5.2 Partial Failure Recovery

#### Database corruption or loss

```
1. Stop RustPBX
    kill $(pgrep rustpbx)

2. Identify the most recent good backup
    ls -lt /backups/rustpbx/db/hourly/

3. Restore the database
    cp /backups/rustpbx/db/hourly/rustpbx-YYYYMMDD-HHMMSS.sqlite3 \
       ~/rustpbx/rustpbx.sqlite3

4. Start RustPBX
    cd ~/rustpbx && nohup ./target/release/rustpbx \
        --conf ~/rustpbx-config/config.toml > ~/rustpbx.log 2>&1 &

5. Verify call records and extensions are intact in the admin console

Note: Calls that occurred between the backup and the failure are lost.
CDR JSON files in ./config/cdr/ may contain records not yet in the
restored database -- these can be manually reviewed if needed.
```

#### Configuration file corruption

```
1. Restore from Git
    cd ~/rustpbx-config
    git checkout -- config.toml
    git checkout -- routes/

2. Restart RustPBX (it supports config reload)
    # Or send a reload signal if supported
    kill -HUP $(pgrep rustpbx)
```

#### Recording directory loss

```
1. Restore from backup
    rsync -az backup-host:/backups/rustpbx/recorders/ ~/rustpbx/config/recorders/

2. No restart needed -- recordings are read on demand by the web console

3. Transcript sidecars will also be restored if they were backed up
   If not, re-run transcription for specific calls from the admin console
```

#### TLS certificate expiry or loss

```
1. If using ACME addon:
    - Restart RustPBX; the ACME addon will re-provision certificates
    - Or trigger re-provisioning from the admin console

2. If using manual certificates:
    - Restore from backup: cp /backups/certs/* ~/rustpbx/config/certs/
    - Restart RustPBX
```

### 5.3 Recovery Testing Schedule

| Test | Frequency | Procedure |
|------|-----------|-----------|
| Database restore to staging | Monthly | Restore latest backup to a test instance, verify data integrity |
| Full system rebuild | Quarterly | Follow Section 5.1 on a fresh VM, verify all functionality |
| Configuration rollback | After each config change | Verify Git revert restores previous working state |
| Recording restore | Quarterly | Restore a sample of recordings, verify playback |
| Backup script health check | Weekly | Verify `backup-status.json` shows recent successful run |

### 5.4 Emergency Contact and Escalation

Document the following for your deployment:

- **Primary on-call**: (name, phone, email)
- **Backup server access**: (host, credentials location)
- **Git repository**: (URL, deploy keys location)
- **Cloud provider console**: (URL, account)
- **SIP trunk provider**: (Telnyx dashboard, support number)

---

## 6. Backup Storage Sizing Estimates

Assumptions: 100 calls/day average, 3-minute average call duration.

| Component | Per-call size | Daily growth | Monthly growth | Annual growth |
|-----------|--------------|--------------|----------------|---------------|
| Database CDR row | ~2 KB | ~200 KB | ~6 MB | ~73 MB |
| CDR JSON file | ~3 KB | ~300 KB | ~9 MB | ~110 MB |
| Recording (WAV) | ~1.4 MB | ~140 MB | ~4.2 GB | ~51 GB |
| Transcript JSON | ~10 KB | ~1 MB | ~30 MB | ~365 MB |
| SIP flow trace | ~5 KB | ~500 KB | ~15 MB | ~183 MB |
| **Total** | | **~142 MB** | **~4.3 GB** | **~52 GB** |

Plan backup storage accordingly. Recordings dominate storage; consider:

- Compressing older recordings (gzip reduces WAV by ~50-60%)
- Using RustPBX's built-in archive feature (`[archive]` config section)
- Tiered storage (move recordings older than 90 days to cold storage)
- Adjusting recording retention policies based on compliance requirements
