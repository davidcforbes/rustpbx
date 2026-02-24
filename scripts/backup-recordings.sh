#!/usr/bin/env bash
# backup-recordings.sh — Incremental recording sync for RustPBX
#
# Syncs recording files, transcript sidecars, CDR files, and SIP flow traces
# to a local or remote backup destination using rsync.
#
# Usage:
#   ./scripts/backup-recordings.sh [OPTIONS]
#
# Options:
#   --dest PATH       Backup destination (local path or rsync remote)
#   --dry-run         Show what would be transferred without doing it
#   --bwlimit KB      Bandwidth limit in KB/s for rsync
#   --delete          Delete files at destination that no longer exist at source
#   --no-cdr          Skip CDR file sync
#   --no-sipflow      Skip SIP flow trace sync
#   --no-transcripts  Skip transcript sidecar sync
#   --log FILE        Log output to file (in addition to stdout)
#   --quiet           Suppress non-error output (still logs to file if --log)
#   --help            Show this help message
#
# Environment variables (override defaults):
#   RUSTPBX_DIR       RustPBX working directory (default: ~/rustpbx)
#   RUSTPBX_CONFIG    Path to config.toml (default: ~/rustpbx-config/config.toml)
#   RECORDING_DIR     Recording source directory (overrides config lookup)
#   CDR_DIR           CDR source directory (overrides default)
#   SIPFLOW_DIR       SIP flow trace directory (overrides default)
#   BACKUP_DEST       Backup destination (same as --dest)
#   BACKUP_BWLIMIT    Bandwidth limit in KB/s (same as --bwlimit)
#
# Examples:
#   # Sync to a local backup directory
#   ./scripts/backup-recordings.sh --dest /mnt/backup/rustpbx/recordings
#
#   # Sync to a remote server via rsync/SSH
#   ./scripts/backup-recordings.sh --dest user@backup-host:/data/rustpbx/
#
#   # Preview what would be synced (dry run) with bandwidth limit
#   ./scripts/backup-recordings.sh --dest /backup --dry-run --bwlimit 5000
#
# Cron example:
#   # Every 6 hours, sync recordings with 10MB/s bandwidth limit
#   0 */6 * * * /home/user/rustpbx/scripts/backup-recordings.sh --dest /backup/rustpbx --bwlimit 10000 --log /var/log/rustpbx-backup.log 2>&1

set -euo pipefail

# ── Configuration Defaults ──────────────────────────────────────────────────

RUSTPBX_DIR="${RUSTPBX_DIR:-$HOME/rustpbx}"
RUSTPBX_CONFIG="${RUSTPBX_CONFIG:-$HOME/rustpbx-config/config.toml}"
RECORDING_DIR="${RECORDING_DIR:-}"
CDR_DIR="${CDR_DIR:-}"
SIPFLOW_DIR="${SIPFLOW_DIR:-}"
BACKUP_DEST="${BACKUP_DEST:-}"
BACKUP_BWLIMIT="${BACKUP_BWLIMIT:-}"

# ── Internal State ──────────────────────────────────────────────────────────

DRY_RUN=0
DELETE_FLAG=0
SYNC_CDR=1
SYNC_SIPFLOW=1
SYNC_TRANSCRIPTS=1
LOG_FILE=""
QUIET=0
EXIT_CODE=0

# ── Colors ──────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── Logging ─────────────────────────────────────────────────────────────────

_log_raw() {
    local color="$1"
    local prefix="$2"
    shift 2
    local msg
    msg="$(date '+%Y-%m-%d %H:%M:%S') $*"

    # Write to log file (without color codes)
    if [ -n "$LOG_FILE" ]; then
        echo "[$prefix] $msg" >> "$LOG_FILE"
    fi

    # Write to stdout (with color) unless quiet
    if [ "$QUIET" -eq 0 ]; then
        echo -e "${color}[$prefix]${NC} $msg"
    fi
}

log()  { _log_raw "$CYAN"   "rec-backup" "$@"; }
warn() { _log_raw "$YELLOW" "rec-backup" "$@"; }
err()  { _log_raw "$RED"    "rec-backup" "$@" >&2; }
ok()   { _log_raw "$GREEN"  "rec-backup" "$@"; }

die() {
    err "$@"
    exit 1
}

# ── Config Parsing ──────────────────────────────────────────────────────────

# Resolve the recording directory from config.toml or use the default.
resolve_recording_dir() {
    if [ -n "$RECORDING_DIR" ]; then
        echo "$RECORDING_DIR"
        return
    fi

    local path=""

    if [ -f "$RUSTPBX_CONFIG" ]; then
        path=$(grep -A5 '^\[recording\]' "$RUSTPBX_CONFIG" 2>/dev/null \
            | grep -E '^\s*path\s*=' \
            | head -1 \
            | sed 's/.*=\s*//' \
            | sed 's/^"//' \
            | sed 's/"$//' \
            | tr -d '[:space:]')
    fi

    if [ -z "$path" ]; then
        path="./config/recorders"
    fi

    # Resolve relative paths against RUSTPBX_DIR
    if [[ "$path" == ./* ]]; then
        path="$RUSTPBX_DIR/${path#./}"
    elif [[ "$path" != /* ]]; then
        path="$RUSTPBX_DIR/$path"
    fi

    echo "$path"
}

# Resolve the CDR directory from config.toml or use the default.
resolve_cdr_dir() {
    if [ -n "$CDR_DIR" ]; then
        echo "$CDR_DIR"
        return
    fi

    local path=""

    if [ -f "$RUSTPBX_CONFIG" ]; then
        path=$(grep -A5 '^\[callrecord\]' "$RUSTPBX_CONFIG" 2>/dev/null \
            | grep -E '^\s*root\s*=' \
            | head -1 \
            | sed 's/.*=\s*//' \
            | sed 's/^"//' \
            | sed 's/"$//' \
            | tr -d '[:space:]')
    fi

    if [ -z "$path" ]; then
        path="./config/cdr"
    fi

    if [[ "$path" == ./* ]]; then
        path="$RUSTPBX_DIR/${path#./}"
    elif [[ "$path" != /* ]]; then
        path="$RUSTPBX_DIR/$path"
    fi

    echo "$path"
}

# Resolve the SIP flow directory from config.toml or use the default.
resolve_sipflow_dir() {
    if [ -n "$SIPFLOW_DIR" ]; then
        echo "$SIPFLOW_DIR"
        return
    fi

    local path=""

    if [ -f "$RUSTPBX_CONFIG" ]; then
        path=$(grep -A5 '^\[sipflow\]' "$RUSTPBX_CONFIG" 2>/dev/null \
            | grep -E '^\s*root\s*=' \
            | head -1 \
            | sed 's/.*=\s*//' \
            | sed 's/^"//' \
            | sed 's/"$//' \
            | tr -d '[:space:]')
    fi

    if [ -z "$path" ]; then
        path="./config/sipflow"
    fi

    if [[ "$path" == ./* ]]; then
        path="$RUSTPBX_DIR/${path#./}"
    elif [[ "$path" != /* ]]; then
        path="$RUSTPBX_DIR/$path"
    fi

    echo "$path"
}

# ── Sync Helper ─────────────────────────────────────────────────────────────

# Run rsync with the common options for a single source directory.
#   $1 — source directory (must end with /)
#   $2 — destination subdirectory name (appended to BACKUP_DEST)
#   $3 — human-readable label for log messages
sync_directory() {
    local src="$1"
    local dest_subdir="$2"
    local label="$3"

    if [ ! -d "$src" ]; then
        warn "$label directory not found: $src (skipping)"
        return 0
    fi

    # Ensure source path ends with / for rsync semantics (sync contents)
    [[ "$src" != */ ]] && src="$src/"

    # Build the destination path.
    # If BACKUP_DEST already ends with /, don't double up.
    local dest="${BACKUP_DEST%/}/$dest_subdir/"

    # Count files at source
    local file_count
    file_count=$(find "$src" -type f 2>/dev/null | wc -l | tr -d ' ')
    log "$label: $file_count files in $src"

    # Build rsync arguments
    local rsync_args=("--archive" "--human-readable" "--itemize-changes")

    if [ "$DELETE_FLAG" -eq 1 ]; then
        rsync_args+=("--delete")
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        rsync_args+=("--dry-run")
        log "$label: DRY RUN — no files will be transferred"
    fi

    if [ -n "$BACKUP_BWLIMIT" ]; then
        rsync_args+=("--bwlimit=$BACKUP_BWLIMIT")
    fi

    rsync_args+=("$src" "$dest")

    log "$label: syncing to $dest"

    # For local destinations, ensure the parent directory exists
    if [[ "$dest" != *:* ]]; then
        mkdir -p "$dest"
    fi

    if rsync "${rsync_args[@]}"; then
        ok "$label: sync completed"
    else
        local rc=$?
        err "$label: rsync failed with exit code $rc"
        EXIT_CODE=1
    fi
}

# ── Argument Parsing ────────────────────────────────────────────────────────

show_help() {
    sed -n '2,/^$/{ s/^# \?//; p; }' "$0"
    exit 0
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --dest)
                [ $# -ge 2 ] || die "--dest requires a PATH argument"
                BACKUP_DEST="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=1
                shift
                ;;
            --bwlimit)
                [ $# -ge 2 ] || die "--bwlimit requires a KB argument"
                BACKUP_BWLIMIT="$2"
                shift 2
                ;;
            --delete)
                DELETE_FLAG=1
                shift
                ;;
            --no-cdr)
                SYNC_CDR=0
                shift
                ;;
            --no-sipflow)
                SYNC_SIPFLOW=0
                shift
                ;;
            --no-transcripts)
                SYNC_TRANSCRIPTS=0
                shift
                ;;
            --log)
                [ $# -ge 2 ] || die "--log requires a FILE argument"
                LOG_FILE="$2"
                shift 2
                ;;
            --quiet|-q)
                QUIET=1
                shift
                ;;
            --help|-h)
                show_help
                ;;
            *)
                die "Unknown option: $1 (use --help for usage)"
                ;;
        esac
    done
}

# ── Validation ──────────────────────────────────────────────────────────────

validate() {
    # Destination is required
    if [ -z "$BACKUP_DEST" ]; then
        die "No backup destination specified. Use --dest PATH or set BACKUP_DEST."
    fi

    # rsync must be available
    if ! command -v rsync &>/dev/null; then
        die "rsync is not installed. Please install rsync and try again."
    fi

    # Bandwidth limit must be a number if set
    if [ -n "$BACKUP_BWLIMIT" ]; then
        if ! [[ "$BACKUP_BWLIMIT" =~ ^[0-9]+$ ]]; then
            die "--bwlimit must be a positive integer (KB/s), got: $BACKUP_BWLIMIT"
        fi
    fi

    # Initialize log file if specified
    if [ -n "$LOG_FILE" ]; then
        local log_dir
        log_dir=$(dirname "$LOG_FILE")
        if [ ! -d "$log_dir" ]; then
            mkdir -p "$log_dir" || die "Cannot create log directory: $log_dir"
        fi
        touch "$LOG_FILE" 2>/dev/null || die "Cannot write to log file: $LOG_FILE"
    fi
}

# ── Main ────────────────────────────────────────────────────────────────────

main() {
    parse_args "$@"
    validate

    log "=== RustPBX Recording Backup ==="
    log "Destination: $BACKUP_DEST"
    [ "$DRY_RUN" -eq 1 ] && log "Mode: DRY RUN"
    [ -n "$BACKUP_BWLIMIT" ] && log "Bandwidth limit: ${BACKUP_BWLIMIT} KB/s"
    [ "$DELETE_FLAG" -eq 1 ] && log "Delete mode: enabled (files removed from source will be removed from backup)"

    # 1. Sync recordings (always — this is the primary purpose)
    local rec_dir
    rec_dir=$(resolve_recording_dir)
    sync_directory "$rec_dir" "recorders" "Recordings"

    # 2. Sync transcript sidecars (.transcript.json files live alongside recordings)
    #    They are already included in the recordings sync above since rsync --archive
    #    copies everything. However, if transcripts are stored separately, we handle
    #    that here by also scanning for transcript files in the recording directory.
    if [ "$SYNC_TRANSCRIPTS" -eq 1 ]; then
        # Transcripts are typically sidecar files alongside .wav files,
        # so they are already captured by the recordings sync.
        # Log the count for visibility.
        if [ -d "$rec_dir" ]; then
            local transcript_count
            transcript_count=$(find "$rec_dir" -type f -name '*.transcript.json' 2>/dev/null | wc -l | tr -d ' ')
            if [ "$transcript_count" -gt 0 ]; then
                log "Transcripts: $transcript_count sidecar files included in recordings sync"
            fi
        fi
    fi

    # 3. Sync CDR files
    if [ "$SYNC_CDR" -eq 1 ]; then
        local cdr_dir
        cdr_dir=$(resolve_cdr_dir)
        sync_directory "$cdr_dir" "cdr" "CDR"
    fi

    # 4. Sync SIP flow traces
    if [ "$SYNC_SIPFLOW" -eq 1 ]; then
        local sipflow_dir
        sipflow_dir=$(resolve_sipflow_dir)
        sync_directory "$sipflow_dir" "sipflow" "SIP Flow"
    fi

    # Summary
    if [ "$EXIT_CODE" -eq 0 ]; then
        ok "=== All sync tasks completed successfully ==="
    else
        err "=== Some sync tasks failed (see errors above) ==="
    fi

    exit "$EXIT_CODE"
}

main "$@"
