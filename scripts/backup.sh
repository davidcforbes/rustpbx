#!/usr/bin/env bash
# backup.sh — RustPBX database, configuration, and recording backup
#
# Usage:
#   ./scripts/backup.sh backup-db           # Back up the database (hourly cron)
#   ./scripts/backup.sh backup-config       # Back up config files (snapshot)
#   ./scripts/backup.sh sync-recordings     # Sync recordings to backup location
#   ./scripts/backup.sh backup-all          # Run all backup tasks
#   ./scripts/backup.sh rotate              # Run retention rotation only
#   ./scripts/backup.sh status              # Show backup status
#   ./scripts/backup.sh restore-db <file>   # Restore database from a backup file
#
# Environment variables (override defaults):
#   RUSTPBX_DIR          — RustPBX working directory (default: ~/rustpbx)
#   RUSTPBX_CONFIG       — Path to config.toml (default: ~/rustpbx-config/config.toml)
#   BACKUP_DIR           — Base backup directory (default: ~/backups/rustpbx)
#   BACKUP_REMOTE        — rsync target for offsite recordings (optional)
#   BACKUP_S3_BUCKET     — S3 bucket for recording sync (optional)
#   DATABASE_URL         — Override database URL from config
#   SQLITE_CMD           — Path to sqlite3 binary (default: sqlite3)
#
# Cron examples:
#   # Hourly database backup
#   0 * * * * /home/user/rustpbx/scripts/backup.sh backup-db 2>&1 | logger -t rustpbx-backup
#
#   # Daily recording sync at 2 AM
#   0 2 * * * /home/user/rustpbx/scripts/backup.sh sync-recordings 2>&1 | logger -t rustpbx-backup
#
#   # Daily full backup at 3 AM
#   0 3 * * * /home/user/rustpbx/scripts/backup.sh backup-all 2>&1 | logger -t rustpbx-backup

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────

RUSTPBX_DIR="${RUSTPBX_DIR:-$HOME/rustpbx}"
RUSTPBX_CONFIG="${RUSTPBX_CONFIG:-$HOME/rustpbx-config/config.toml}"
BACKUP_DIR="${BACKUP_DIR:-$HOME/backups/rustpbx}"
BACKUP_REMOTE="${BACKUP_REMOTE:-}"
BACKUP_S3_BUCKET="${BACKUP_S3_BUCKET:-}"
SQLITE_CMD="${SQLITE_CMD:-sqlite3}"

# Retention policy
HOURLY_KEEP=24
DAILY_KEEP=7
WEEKLY_KEEP=4
MONTHLY_KEEP=12

# Subdirectories
DB_BACKUP_DIR="$BACKUP_DIR/db"
CONFIG_BACKUP_DIR="$BACKUP_DIR/config"
STATUS_FILE="$BACKUP_DIR/backup-status.json"
LOG_FILE="$BACKUP_DIR/backup.log"

# ── Colors ───────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── Utility Functions ────────────────────────────────────────────────────────

log()  { echo -e "${CYAN}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"; }
warn() { echo -e "${YELLOW}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"; }
err()  { echo -e "${RED}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*" >&2; }
ok()   { echo -e "${GREEN}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"; }

die() {
    err "$@"
    write_status "failed" "$*"
    exit 1
}

ensure_dirs() {
    mkdir -p "$DB_BACKUP_DIR"/{hourly,daily,weekly,monthly}
    mkdir -p "$CONFIG_BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"
}

write_status() {
    local status="$1"
    local message="${2:-}"
    local timestamp
    timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

    cat > "$STATUS_FILE" <<EOF
{
  "timestamp": "$timestamp",
  "status": "$status",
  "message": "$message",
  "backup_dir": "$BACKUP_DIR",
  "db_backups": $(count_backups "$DB_BACKUP_DIR"),
  "config_backups": $(count_backups "$CONFIG_BACKUP_DIR")
}
EOF
}

count_backups() {
    local dir="$1"
    if [ -d "$dir" ]; then
        find "$dir" -type f 2>/dev/null | wc -l | tr -d ' '
    else
        echo "0"
    fi
}

# Parse database_url from config.toml (handles both quoted and unquoted values)
get_database_url() {
    if [ -n "${DATABASE_URL:-}" ]; then
        echo "$DATABASE_URL"
        return
    fi

    if [ ! -f "$RUSTPBX_CONFIG" ]; then
        die "Config file not found: $RUSTPBX_CONFIG"
    fi

    local url
    url=$(grep -E '^\s*database_url\s*=' "$RUSTPBX_CONFIG" \
        | head -1 \
        | sed 's/.*=\s*//' \
        | sed 's/^"//' \
        | sed 's/"$//' \
        | sed "s/^'//" \
        | sed "s/'$//" \
        | tr -d '[:space:]')

    if [ -z "$url" ]; then
        # Fall back to default
        echo "sqlite://rustpbx.sqlite3"
    else
        echo "$url"
    fi
}

# Extract the SQLite file path from a sqlite:// URL
sqlite_path_from_url() {
    local url="$1"
    local path="${url#sqlite://}"

    # If relative path, resolve relative to RUSTPBX_DIR
    if [[ "$path" != /* ]]; then
        path="$RUSTPBX_DIR/$path"
    fi

    echo "$path"
}

# Get the recorder path from config.toml
get_recorder_path() {
    local path
    path=$(grep -A5 '^\[recording\]' "$RUSTPBX_CONFIG" 2>/dev/null \
        | grep -E '^\s*path\s*=' \
        | head -1 \
        | sed 's/.*=\s*//' \
        | sed 's/^"//' \
        | sed 's/"$//' \
        | tr -d '[:space:]')

    if [ -z "$path" ]; then
        path="./config/recorders"
    fi

    # Resolve relative paths
    if [[ "$path" == ./* ]]; then
        path="$RUSTPBX_DIR/${path#./}"
    elif [[ "$path" != /* ]]; then
        path="$RUSTPBX_DIR/$path"
    fi

    echo "$path"
}

# ── Database Backup ──────────────────────────────────────────────────────────

backup_db() {
    ensure_dirs
    log "Starting database backup..."

    local db_url
    db_url=$(get_database_url)

    local timestamp
    timestamp=$(date '+%Y%m%d-%H%M%S')
    local day_of_week
    day_of_week=$(date '+%u')  # 1=Monday, 7=Sunday
    local day_of_month
    day_of_month=$(date '+%d')
    local hour
    hour=$(date '+%H')

    if [[ "$db_url" == sqlite://* ]]; then
        backup_sqlite "$db_url" "$timestamp" "$day_of_week" "$day_of_month" "$hour"
    elif [[ "$db_url" == postgres://* ]]; then
        backup_postgres "$db_url" "$timestamp" "$day_of_week" "$day_of_month" "$hour"
    elif [[ "$db_url" == mysql://* ]]; then
        backup_mysql "$db_url" "$timestamp" "$day_of_week" "$day_of_month" "$hour"
    else
        die "Unsupported database URL scheme: $db_url"
    fi

    rotate_db_backups
    ok "Database backup completed successfully"
}

backup_sqlite() {
    local db_url="$1"
    local timestamp="$2"
    local day_of_week="$3"
    local day_of_month="$4"
    local hour="$5"

    local db_path
    db_path=$(sqlite_path_from_url "$db_url")

    if [ ! -f "$db_path" ]; then
        die "SQLite database not found: $db_path"
    fi

    if ! command -v "$SQLITE_CMD" &>/dev/null; then
        die "sqlite3 command not found. Install sqlite3 or set SQLITE_CMD."
    fi

    local backup_file="$DB_BACKUP_DIR/hourly/rustpbx-${timestamp}.sqlite3"

    log "Backing up SQLite database: $db_path"
    log "Destination: $backup_file"

    # Use SQLite's online backup API (safe during concurrent writes)
    "$SQLITE_CMD" "$db_path" ".backup '$backup_file'"

    local size
    size=$(stat -c%s "$backup_file" 2>/dev/null || stat -f%z "$backup_file" 2>/dev/null || echo "unknown")
    log "Backup size: $size bytes"

    # Promote to daily/weekly/monthly via hard links
    promote_backup "$backup_file" "$timestamp" "$day_of_week" "$day_of_month" "$hour" "sqlite3"
}

backup_postgres() {
    local db_url="$1"
    local timestamp="$2"
    local day_of_week="$3"
    local day_of_month="$4"
    local hour="$5"

    if ! command -v pg_dump &>/dev/null; then
        die "pg_dump command not found. Install PostgreSQL client tools."
    fi

    local backup_file="$DB_BACKUP_DIR/hourly/rustpbx-${timestamp}.pgdump"

    log "Backing up PostgreSQL database"
    log "Destination: $backup_file"

    pg_dump -Fc -f "$backup_file" "$db_url"

    local size
    size=$(stat -c%s "$backup_file" 2>/dev/null || stat -f%z "$backup_file" 2>/dev/null || echo "unknown")
    log "Backup size: $size bytes"

    promote_backup "$backup_file" "$timestamp" "$day_of_week" "$day_of_month" "$hour" "pgdump"
}

backup_mysql() {
    local db_url="$1"
    local timestamp="$2"
    local day_of_week="$3"
    local day_of_month="$4"
    local hour="$5"

    if ! command -v mysqldump &>/dev/null; then
        die "mysqldump command not found. Install MySQL client tools."
    fi

    # Parse mysql://user:pass@host:port/dbname
    local url_no_scheme="${db_url#mysql://}"
    local userpass="${url_no_scheme%%@*}"
    local hostdb="${url_no_scheme#*@}"
    local user="${userpass%%:*}"
    local pass="${userpass#*:}"
    local host="${hostdb%%/*}"
    local dbname="${hostdb#*/}"

    local backup_file="$DB_BACKUP_DIR/hourly/rustpbx-${timestamp}.sql.gz"

    log "Backing up MySQL database: $dbname on $host"
    log "Destination: $backup_file"

    mysqldump --single-transaction -h "$host" -u "$user" -p"$pass" "$dbname" \
        | gzip > "$backup_file"

    local size
    size=$(stat -c%s "$backup_file" 2>/dev/null || stat -f%z "$backup_file" 2>/dev/null || echo "unknown")
    log "Backup size: $size bytes"

    promote_backup "$backup_file" "$timestamp" "$day_of_week" "$day_of_month" "$hour" "sql.gz"
}

# Create hard links for daily/weekly/monthly tiers
promote_backup() {
    local backup_file="$1"
    local timestamp="$2"
    local day_of_week="$3"
    local day_of_month="$4"
    local hour="$5"
    local ext="$6"

    # Promote to daily at midnight (hour 00)
    if [ "$hour" = "00" ]; then
        local daily_file="$DB_BACKUP_DIR/daily/rustpbx-${timestamp}.$ext"
        ln -f "$backup_file" "$daily_file" 2>/dev/null || cp "$backup_file" "$daily_file"
        log "Promoted to daily: $daily_file"

        # Promote to weekly on Sunday (day 7)
        if [ "$day_of_week" = "7" ]; then
            local weekly_file="$DB_BACKUP_DIR/weekly/rustpbx-${timestamp}.$ext"
            ln -f "$backup_file" "$weekly_file" 2>/dev/null || cp "$backup_file" "$weekly_file"
            log "Promoted to weekly: $weekly_file"
        fi

        # Promote to monthly on the 1st
        if [ "$day_of_month" = "01" ]; then
            local monthly_file="$DB_BACKUP_DIR/monthly/rustpbx-${timestamp}.$ext"
            ln -f "$backup_file" "$monthly_file" 2>/dev/null || cp "$backup_file" "$monthly_file"
            log "Promoted to monthly: $monthly_file"
        fi
    fi
}

# ── Retention Rotation ───────────────────────────────────────────────────────

rotate_db_backups() {
    log "Rotating database backups..."

    rotate_tier "$DB_BACKUP_DIR/hourly" "$HOURLY_KEEP"
    rotate_tier "$DB_BACKUP_DIR/daily" "$DAILY_KEEP"
    rotate_tier "$DB_BACKUP_DIR/weekly" "$WEEKLY_KEEP"
    rotate_tier "$DB_BACKUP_DIR/monthly" "$MONTHLY_KEEP"
}

rotate_tier() {
    local dir="$1"
    local keep="$2"

    if [ ! -d "$dir" ]; then
        return
    fi

    local count
    count=$(find "$dir" -maxdepth 1 -type f | wc -l | tr -d ' ')

    if [ "$count" -le "$keep" ]; then
        return
    fi

    local to_remove=$((count - keep))
    log "Rotating $dir: $count files, keeping $keep, removing $to_remove"

    # Remove oldest files first (sorted by name, which includes timestamp)
    find "$dir" -maxdepth 1 -type f | sort | head -n "$to_remove" | while read -r file; do
        log "Removing old backup: $(basename "$file")"
        rm -f "$file"
    done
}

# ── Configuration Backup ────────────────────────────────────────────────────

backup_config() {
    ensure_dirs
    log "Starting configuration backup..."

    local timestamp
    timestamp=$(date '+%Y%m%d-%H%M%S')
    local archive="$CONFIG_BACKUP_DIR/config-${timestamp}.tar.gz"

    # Collect config files that exist
    local files_to_backup=()

    if [ -f "$RUSTPBX_CONFIG" ]; then
        files_to_backup+=("$RUSTPBX_CONFIG")
    fi

    # Also grab all config directory contents relative to RUSTPBX_DIR
    local config_dir="$RUSTPBX_DIR/config"
    if [ -d "$config_dir" ]; then
        files_to_backup+=("$config_dir")
    fi

    # Also grab rustpbx-config directory if it's a separate location
    local config_parent
    config_parent=$(dirname "$RUSTPBX_CONFIG")
    if [ -d "$config_parent" ] && [ "$config_parent" != "$config_dir" ]; then
        files_to_backup+=("$config_parent")
    fi

    if [ ${#files_to_backup[@]} -eq 0 ]; then
        warn "No configuration files found to back up"
        return
    fi

    log "Creating config archive: $archive"
    tar czf "$archive" "${files_to_backup[@]}" 2>/dev/null || {
        # If tar fails (e.g., permission issues), try without problematic files
        warn "Some files could not be archived, retrying with available files..."
        tar czf "$archive" --ignore-failed-read "${files_to_backup[@]}" 2>/dev/null || true
    }

    local size
    size=$(stat -c%s "$archive" 2>/dev/null || stat -f%z "$archive" 2>/dev/null || echo "unknown")
    log "Config archive size: $size bytes"

    # Keep only the last 30 config snapshots
    rotate_tier "$CONFIG_BACKUP_DIR" 30

    ok "Configuration backup completed"
}

# ── Recording Sync ───────────────────────────────────────────────────────────

sync_recordings() {
    log "Starting recording sync..."

    local recorder_path
    recorder_path=$(get_recorder_path)

    if [ ! -d "$recorder_path" ]; then
        warn "Recordings directory not found: $recorder_path"
        warn "Nothing to sync."
        return
    fi

    local file_count
    file_count=$(find "$recorder_path" -type f -name '*.wav' 2>/dev/null | wc -l | tr -d ' ')
    log "Found $file_count recording files in $recorder_path"

    # Sync to remote backup server via rsync
    if [ -n "$BACKUP_REMOTE" ]; then
        log "Syncing recordings to remote: $BACKUP_REMOTE"
        rsync -az --progress "$recorder_path/" "$BACKUP_REMOTE/recorders/" || {
            err "rsync to $BACKUP_REMOTE failed"
            return 1
        }
        ok "Recordings synced to remote: $BACKUP_REMOTE"
    fi

    # Sync to S3
    if [ -n "$BACKUP_S3_BUCKET" ]; then
        if ! command -v aws &>/dev/null; then
            err "AWS CLI not found. Install it to use S3 sync."
            return 1
        fi

        log "Syncing recordings to S3: $BACKUP_S3_BUCKET"
        aws s3 sync "$recorder_path/" "s3://$BACKUP_S3_BUCKET/recorders/" \
            --storage-class STANDARD_IA || {
            err "S3 sync to $BACKUP_S3_BUCKET failed"
            return 1
        }
        ok "Recordings synced to S3: $BACKUP_S3_BUCKET"
    fi

    # Also sync CDR JSON files and transcript sidecars
    local cdr_path="$RUSTPBX_DIR/config/cdr"
    if [ -d "$cdr_path" ]; then
        if [ -n "$BACKUP_REMOTE" ]; then
            log "Syncing CDR files to remote..."
            rsync -az "$cdr_path/" "$BACKUP_REMOTE/cdr/" || warn "CDR rsync failed"
        fi
        if [ -n "$BACKUP_S3_BUCKET" ]; then
            log "Syncing CDR files to S3..."
            aws s3 sync "$cdr_path/" "s3://$BACKUP_S3_BUCKET/cdr/" || warn "CDR S3 sync failed"
        fi
    fi

    # Sync SIP flow traces
    local sipflow_path="$RUSTPBX_DIR/config/sipflow"
    if [ -d "$sipflow_path" ]; then
        if [ -n "$BACKUP_REMOTE" ]; then
            log "Syncing SIP flow traces to remote..."
            rsync -az "$sipflow_path/" "$BACKUP_REMOTE/sipflow/" || warn "SIP flow rsync failed"
        fi
    fi

    # If neither remote target is configured, sync to local backup dir
    if [ -z "$BACKUP_REMOTE" ] && [ -z "$BACKUP_S3_BUCKET" ]; then
        local local_rec_backup="$BACKUP_DIR/recorders"
        mkdir -p "$local_rec_backup"
        log "No remote target configured. Syncing to local backup: $local_rec_backup"
        rsync -a "$recorder_path/" "$local_rec_backup/" || {
            err "Local rsync failed"
            return 1
        }

        if [ -d "$cdr_path" ]; then
            mkdir -p "$BACKUP_DIR/cdr"
            rsync -a "$cdr_path/" "$BACKUP_DIR/cdr/" || warn "Local CDR sync failed"
        fi

        ok "Recordings synced to local backup directory"
    fi

    ok "Recording sync completed"
}

# ── Restore ──────────────────────────────────────────────────────────────────

restore_db() {
    local backup_file="${1:-}"

    if [ -z "$backup_file" ]; then
        err "Usage: $0 restore-db <backup-file>"
        err ""
        err "Available backups:"
        find "$DB_BACKUP_DIR" -type f | sort -r | head -20
        exit 1
    fi

    if [ ! -f "$backup_file" ]; then
        die "Backup file not found: $backup_file"
    fi

    local db_url
    db_url=$(get_database_url)

    if [[ "$db_url" != sqlite://* ]]; then
        die "Restore is currently only automated for SQLite. For PostgreSQL, use: pg_restore -d \$DATABASE_URL $backup_file"
    fi

    local db_path
    db_path=$(sqlite_path_from_url "$db_url")

    # Check if RustPBX is running
    if pgrep -x rustpbx &>/dev/null; then
        warn "RustPBX appears to be running. Stop it before restoring:"
        warn "  kill \$(pgrep rustpbx)"
        die "Cannot restore while RustPBX is running"
    fi

    # Create a safety backup of current database
    if [ -f "$db_path" ]; then
        local safety_backup="${db_path}.pre-restore.$(date '+%Y%m%d-%H%M%S')"
        log "Creating safety backup of current database: $safety_backup"
        cp "$db_path" "$safety_backup"
    fi

    log "Restoring database from: $backup_file"
    log "Destination: $db_path"
    cp "$backup_file" "$db_path"

    # Verify the restored database
    if "$SQLITE_CMD" "$db_path" "SELECT count(*) FROM sqlite_master;" &>/dev/null; then
        ok "Database restored and verified successfully"
        ok "Start RustPBX to use the restored database"
    else
        err "Restored database appears corrupt!"
        if [ -f "${db_path}.pre-restore."* ]; then
            warn "Your previous database was saved as: ${db_path}.pre-restore.*"
        fi
        exit 1
    fi
}

# ── All Backups ──────────────────────────────────────────────────────────────

backup_all() {
    log "=== Starting full backup ==="

    local failed=0

    backup_db || { err "Database backup failed"; failed=1; }
    backup_config || { err "Config backup failed"; failed=1; }
    sync_recordings || { err "Recording sync failed"; failed=1; }

    if [ "$failed" -eq 0 ]; then
        write_status "success" "Full backup completed"
        ok "=== Full backup completed successfully ==="
    else
        write_status "partial" "Some backup tasks failed"
        warn "=== Full backup completed with errors ==="
        exit 1
    fi
}

# ── Status ───────────────────────────────────────────────────────────────────

show_status() {
    echo ""
    log "=== RustPBX Backup Status ==="
    echo ""

    if [ -f "$STATUS_FILE" ]; then
        log "Last backup status:"
        cat "$STATUS_FILE"
        echo ""
    else
        warn "No backup status file found. Backups may not have been run."
        echo ""
    fi

    log "Backup directory: $BACKUP_DIR"
    echo ""

    if [ -d "$DB_BACKUP_DIR" ]; then
        log "Database backups:"
        echo "  Hourly:  $(find "$DB_BACKUP_DIR/hourly" -type f 2>/dev/null | wc -l | tr -d ' ') files"
        echo "  Daily:   $(find "$DB_BACKUP_DIR/daily" -type f 2>/dev/null | wc -l | tr -d ' ') files"
        echo "  Weekly:  $(find "$DB_BACKUP_DIR/weekly" -type f 2>/dev/null | wc -l | tr -d ' ') files"
        echo "  Monthly: $(find "$DB_BACKUP_DIR/monthly" -type f 2>/dev/null | wc -l | tr -d ' ') files"
        echo ""

        local latest
        latest=$(find "$DB_BACKUP_DIR" -type f | sort -r | head -1)
        if [ -n "$latest" ]; then
            log "Most recent backup: $latest"
            local size
            size=$(stat -c%s "$latest" 2>/dev/null || stat -f%z "$latest" 2>/dev/null || echo "unknown")
            echo "  Size: $size bytes"
            echo ""
        fi
    fi

    if [ -d "$CONFIG_BACKUP_DIR" ]; then
        log "Config backups: $(find "$CONFIG_BACKUP_DIR" -type f | wc -l | tr -d ' ') snapshots"
    fi

    echo ""

    # Show database info
    local db_url
    db_url=$(get_database_url 2>/dev/null || echo "unknown")
    log "Database URL: $db_url"

    if [[ "$db_url" == sqlite://* ]]; then
        local db_path
        db_path=$(sqlite_path_from_url "$db_url")
        if [ -f "$db_path" ]; then
            local db_size
            db_size=$(stat -c%s "$db_path" 2>/dev/null || stat -f%z "$db_path" 2>/dev/null || echo "unknown")
            log "Database file: $db_path ($db_size bytes)"
        else
            warn "Database file not found: $db_path"
        fi
    fi

    local recorder_path
    recorder_path=$(get_recorder_path 2>/dev/null || echo "unknown")
    if [ -d "$recorder_path" ]; then
        local rec_count
        rec_count=$(find "$recorder_path" -type f -name '*.wav' 2>/dev/null | wc -l | tr -d ' ')
        local rec_size
        rec_size=$(du -sh "$recorder_path" 2>/dev/null | cut -f1 || echo "unknown")
        log "Recordings: $rec_count files ($rec_size) in $recorder_path"
    fi

    echo ""
}

# ── Main ─────────────────────────────────────────────────────────────────────

ACTION="${1:-}"

case "$ACTION" in
    backup-db)
        backup_db
        write_status "success" "Database backup completed"
        ;;
    backup-config)
        backup_config
        write_status "success" "Config backup completed"
        ;;
    sync-recordings)
        sync_recordings
        write_status "success" "Recording sync completed"
        ;;
    backup-all)
        backup_all
        ;;
    rotate)
        rotate_db_backups
        ok "Rotation completed"
        ;;
    status)
        show_status
        ;;
    restore-db)
        restore_db "${2:-}"
        ;;
    *)
        echo "RustPBX Backup Script"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  backup-db         Back up the database (hourly cron)"
        echo "  backup-config     Snapshot configuration files"
        echo "  sync-recordings   Sync recordings to backup target"
        echo "  backup-all        Run all backup tasks"
        echo "  rotate            Run retention rotation only"
        echo "  status            Show backup status and statistics"
        echo "  restore-db FILE   Restore database from a backup file"
        echo ""
        echo "Environment variables:"
        echo "  RUSTPBX_DIR       RustPBX working directory (default: ~/rustpbx)"
        echo "  RUSTPBX_CONFIG    Path to config.toml"
        echo "  BACKUP_DIR        Base backup directory (default: ~/backups/rustpbx)"
        echo "  BACKUP_REMOTE     rsync target for offsite recordings"
        echo "  BACKUP_S3_BUCKET  S3 bucket for recording sync"
        echo ""
        echo "Rotation policy:"
        echo "  Hourly:  keep $HOURLY_KEEP"
        echo "  Daily:   keep $DAILY_KEEP"
        echo "  Weekly:  keep $WEEKLY_KEEP"
        echo "  Monthly: keep $MONTHLY_KEEP"
        exit 1
        ;;
esac
