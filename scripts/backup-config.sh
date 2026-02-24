#!/usr/bin/env bash
# backup-config.sh — Configuration backup with version control for RustPBX
#
# Copies configuration files to a versioned backup directory, optionally
# tracking changes in a git repository and creating tarballs.
#
# Usage:
#   ./scripts/backup-config.sh [OPTIONS]
#
# Options:
#   --git-init        Initialize a git repo in the backup directory
#   --commit          Create a timestamped git commit with current config state
#   --tarball         Create a compressed tarball of current configuration
#   --push            Push to the configured git remote after committing
#   --backup-dir DIR  Override backup directory (default: ~/backups/rustpbx/config-repo)
#   --tarball-dir DIR Override tarball output directory (default: ~/backups/rustpbx/config-tarballs)
#   --remote URL      Set the git remote URL (used with --git-init or --push)
#   --log FILE        Log output to file (in addition to stdout)
#   --quiet           Suppress non-error output
#   --help            Show this help message
#
# Environment variables (override defaults):
#   RUSTPBX_DIR          RustPBX working directory (default: ~/rustpbx)
#   RUSTPBX_CONFIG       Path to config.toml (default: ~/rustpbx-config/config.toml)
#   CONFIG_BACKUP_DIR    Versioned backup directory (same as --backup-dir)
#   CONFIG_TARBALL_DIR   Tarball output directory (same as --tarball-dir)
#   CONFIG_GIT_REMOTE    Git remote URL (same as --remote)
#
# Examples:
#   # First time: initialize git repo and commit config
#   ./scripts/backup-config.sh --git-init --commit
#
#   # Routine backup: commit current state
#   ./scripts/backup-config.sh --commit
#
#   # Create a tarball snapshot for offsite transfer
#   ./scripts/backup-config.sh --tarball
#
#   # Full backup: commit + tarball + push to remote
#   ./scripts/backup-config.sh --commit --tarball --push --remote git@backup:rustpbx-config.git
#
# Cron example:
#   # Daily config snapshot at 1 AM
#   0 1 * * * /home/user/rustpbx/scripts/backup-config.sh --commit --tarball --log /var/log/rustpbx-config-backup.log 2>&1

set -euo pipefail

# ── Configuration Defaults ──────────────────────────────────────────────────

RUSTPBX_DIR="${RUSTPBX_DIR:-$HOME/rustpbx}"
RUSTPBX_CONFIG="${RUSTPBX_CONFIG:-$HOME/rustpbx-config/config.toml}"
CONFIG_BACKUP_DIR="${CONFIG_BACKUP_DIR:-$HOME/backups/rustpbx/config-repo}"
CONFIG_TARBALL_DIR="${CONFIG_TARBALL_DIR:-$HOME/backups/rustpbx/config-tarballs}"
CONFIG_GIT_REMOTE="${CONFIG_GIT_REMOTE:-}"

# ── Internal State ──────────────────────────────────────────────────────────

DO_GIT_INIT=0
DO_COMMIT=0
DO_TARBALL=0
DO_PUSH=0
LOG_FILE=""
QUIET=0

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

    if [ -n "$LOG_FILE" ]; then
        echo "[$prefix] $msg" >> "$LOG_FILE"
    fi

    if [ "$QUIET" -eq 0 ]; then
        echo -e "${color}[$prefix]${NC} $msg"
    fi
}

log()  { _log_raw "$CYAN"   "cfg-backup" "$@"; }
warn() { _log_raw "$YELLOW" "cfg-backup" "$@"; }
err()  { _log_raw "$RED"    "cfg-backup" "$@" >&2; }
ok()   { _log_raw "$GREEN"  "cfg-backup" "$@"; }

die() {
    err "$@"
    exit 1
}

# ── Config File Discovery ──────────────────────────────────────────────────

# Resolve the generated_dir / config root from config.toml (where routes/
# trunks/ etc. live). Falls back to $RUSTPBX_DIR/config.
resolve_config_root() {
    local path=""

    if [ -f "$RUSTPBX_CONFIG" ]; then
        path=$(grep -E '^\s*generated_dir\s*=' "$RUSTPBX_CONFIG" 2>/dev/null \
            | head -1 \
            | sed 's/.*=\s*//' \
            | sed 's/^"//' \
            | sed 's/"$//' \
            | tr -d '[:space:]')
    fi

    if [ -z "$path" ]; then
        path="./config"
    fi

    if [[ "$path" == ./* ]]; then
        path="$RUSTPBX_DIR/${path#./}"
    elif [[ "$path" != /* ]]; then
        path="$RUSTPBX_DIR/$path"
    fi

    echo "$path"
}

# Copy all relevant config files into the backup directory, preserving
# relative structure. Returns the number of files copied.
copy_config_files() {
    local dest="$1"
    local count=0

    # 1. Main config.toml
    if [ -f "$RUSTPBX_CONFIG" ]; then
        mkdir -p "$dest"
        cp -p "$RUSTPBX_CONFIG" "$dest/config.toml"
        log "Copied: config.toml"
        count=$((count + 1))
    else
        warn "Main config file not found: $RUSTPBX_CONFIG"
    fi

    # 2. Config root directory (routes/, trunks/, acl/, certs/, etc.)
    local config_root
    config_root=$(resolve_config_root)

    if [ -d "$config_root" ]; then
        # Copy route files
        if [ -d "$config_root/routes" ]; then
            mkdir -p "$dest/routes"
            local route_files
            route_files=$(find "$config_root/routes" -maxdepth 1 -name '*.toml' -type f 2>/dev/null)
            if [ -n "$route_files" ]; then
                for f in $route_files; do
                    cp -p "$f" "$dest/routes/"
                    log "Copied: routes/$(basename "$f")"
                    count=$((count + 1))
                done
            fi
        fi

        # Copy trunk files
        if [ -d "$config_root/trunks" ]; then
            mkdir -p "$dest/trunks"
            local trunk_files
            trunk_files=$(find "$config_root/trunks" -maxdepth 1 -name '*.toml' -type f 2>/dev/null)
            if [ -n "$trunk_files" ]; then
                for f in $trunk_files; do
                    cp -p "$f" "$dest/trunks/"
                    log "Copied: trunks/$(basename "$f")"
                    count=$((count + 1))
                done
            fi
        fi

        # Copy ACL files
        if [ -d "$config_root/acl" ]; then
            mkdir -p "$dest/acl"
            local acl_files
            acl_files=$(find "$config_root/acl" -maxdepth 1 -type f ! -name '.placeholder' 2>/dev/null)
            if [ -n "$acl_files" ]; then
                for f in $acl_files; do
                    cp -p "$f" "$dest/acl/"
                    log "Copied: acl/$(basename "$f")"
                    count=$((count + 1))
                done
            fi
        fi

        # Copy TLS certificates (public certs and keys — warn about private keys)
        if [ -d "$config_root/certs" ]; then
            mkdir -p "$dest/certs"
            local cert_files
            cert_files=$(find "$config_root/certs" -maxdepth 1 -type f 2>/dev/null)
            if [ -n "$cert_files" ]; then
                for f in $cert_files; do
                    local basename_f
                    basename_f=$(basename "$f")
                    # Warn about private key files but still back them up
                    if [[ "$basename_f" == *.key ]] || [[ "$basename_f" == *private* ]]; then
                        warn "Backing up private key file: certs/$basename_f — ensure backup is secured!"
                    fi
                    cp -p "$f" "$dest/certs/"
                    log "Copied: certs/$basename_f"
                    count=$((count + 1))
                done
            fi
        fi

        # Copy queue configuration
        if [ -d "$config_root/queue" ]; then
            mkdir -p "$dest/queue"
            local queue_files
            queue_files=$(find "$config_root/queue" -maxdepth 1 -type f ! -name '.placeholder' 2>/dev/null)
            if [ -n "$queue_files" ]; then
                for f in $queue_files; do
                    cp -p "$f" "$dest/queue/"
                    log "Copied: queue/$(basename "$f")"
                    count=$((count + 1))
                done
            fi
        fi

        # Copy sound files manifest (not the actual WAV files — just list them)
        if [ -d "$config_root/sounds" ]; then
            mkdir -p "$dest/sounds"
            find "$config_root/sounds" -maxdepth 1 -type f -name '*.wav' 2>/dev/null \
                | while read -r f; do basename "$f"; done \
                | sort > "$dest/sounds/manifest.txt"
            log "Created: sounds/manifest.txt"
            count=$((count + 1))
        fi
    else
        warn "Config root directory not found: $config_root"
    fi

    # 3. If config.toml lives in a separate config directory (e.g., ~/rustpbx-config/),
    #    also copy any route/trunk files from there.
    local config_parent
    config_parent=$(dirname "$RUSTPBX_CONFIG")
    if [ -d "$config_parent" ] && [ "$config_parent" != "$config_root" ]; then
        if [ -d "$config_parent/routes" ]; then
            mkdir -p "$dest/routes"
            for f in "$config_parent"/routes/*.toml; do
                [ -f "$f" ] || continue
                local bname
                bname=$(basename "$f")
                if [ ! -f "$dest/routes/$bname" ]; then
                    cp -p "$f" "$dest/routes/"
                    log "Copied (external): routes/$bname"
                    count=$((count + 1))
                fi
            done
        fi
        if [ -d "$config_parent/trunks" ]; then
            mkdir -p "$dest/trunks"
            for f in "$config_parent"/trunks/*.toml; do
                [ -f "$f" ] || continue
                local bname
                bname=$(basename "$f")
                if [ ! -f "$dest/trunks/$bname" ]; then
                    cp -p "$f" "$dest/trunks/"
                    log "Copied (external): trunks/$bname"
                    count=$((count + 1))
                fi
            done
        fi
    fi

    echo "$count"
}

# ── Git Operations ──────────────────────────────────────────────────────────

# Initialize a git repository in the backup directory.
git_init() {
    log "Initializing git repository in $CONFIG_BACKUP_DIR"

    mkdir -p "$CONFIG_BACKUP_DIR"

    if [ -d "$CONFIG_BACKUP_DIR/.git" ]; then
        warn "Git repository already exists in $CONFIG_BACKUP_DIR"
    else
        git -C "$CONFIG_BACKUP_DIR" init
        ok "Git repository initialized"
    fi

    # Create a .gitignore to exclude noisy/temporary files
    if [ ! -f "$CONFIG_BACKUP_DIR/.gitignore" ]; then
        cat > "$CONFIG_BACKUP_DIR/.gitignore" <<'GITIGNORE'
# Temporary files
*.tmp
*.swp
*~

# OS files
.DS_Store
Thumbs.db

# Do not track large media files
sounds/*.wav
GITIGNORE
        log "Created .gitignore"
    fi

    # Set up the remote if provided
    if [ -n "$CONFIG_GIT_REMOTE" ]; then
        if git -C "$CONFIG_BACKUP_DIR" remote get-url origin &>/dev/null; then
            git -C "$CONFIG_BACKUP_DIR" remote set-url origin "$CONFIG_GIT_REMOTE"
            log "Updated git remote 'origin' to $CONFIG_GIT_REMOTE"
        else
            git -C "$CONFIG_BACKUP_DIR" remote add origin "$CONFIG_GIT_REMOTE"
            log "Added git remote 'origin': $CONFIG_GIT_REMOTE"
        fi
    fi

    ok "Git init completed"
}

# Commit current config state with a timestamped message.
git_commit() {
    if [ ! -d "$CONFIG_BACKUP_DIR/.git" ]; then
        die "No git repository in $CONFIG_BACKUP_DIR. Run with --git-init first."
    fi

    log "Copying config files to backup directory..."

    local file_count
    file_count=$(copy_config_files "$CONFIG_BACKUP_DIR")

    if [ "$file_count" -eq 0 ]; then
        warn "No configuration files found to back up"
        return 0
    fi

    log "Copied $file_count configuration file(s)"

    # Stage all changes
    git -C "$CONFIG_BACKUP_DIR" add -A

    # Check if there are any changes to commit
    if git -C "$CONFIG_BACKUP_DIR" diff --cached --quiet 2>/dev/null; then
        log "No configuration changes detected since last commit"
        return 0
    fi

    # Build commit message
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S %Z')
    local hostname
    hostname=$(hostname 2>/dev/null || echo "unknown")

    local changed_files
    changed_files=$(git -C "$CONFIG_BACKUP_DIR" diff --cached --name-only 2>/dev/null | head -20)

    local commit_msg
    commit_msg="Config backup: $timestamp

Host: $hostname
Files changed:
$changed_files"

    git -C "$CONFIG_BACKUP_DIR" commit -m "$commit_msg"

    local commit_hash
    commit_hash=$(git -C "$CONFIG_BACKUP_DIR" rev-parse --short HEAD)
    ok "Committed config snapshot: $commit_hash"

    # Show a brief summary
    local total_commits
    total_commits=$(git -C "$CONFIG_BACKUP_DIR" rev-list --count HEAD 2>/dev/null || echo "?")
    log "Total config snapshots: $total_commits"
}

# Push to the configured remote.
git_push() {
    if [ ! -d "$CONFIG_BACKUP_DIR/.git" ]; then
        die "No git repository in $CONFIG_BACKUP_DIR. Run with --git-init first."
    fi

    # Check if a remote is configured
    if ! git -C "$CONFIG_BACKUP_DIR" remote get-url origin &>/dev/null; then
        if [ -n "$CONFIG_GIT_REMOTE" ]; then
            git -C "$CONFIG_BACKUP_DIR" remote add origin "$CONFIG_GIT_REMOTE"
            log "Added git remote 'origin': $CONFIG_GIT_REMOTE"
        else
            die "No git remote configured. Use --remote URL to set one."
        fi
    fi

    log "Pushing to remote..."

    # Determine the current branch name
    local branch
    branch=$(git -C "$CONFIG_BACKUP_DIR" branch --show-current 2>/dev/null || echo "main")
    if [ -z "$branch" ]; then
        branch="main"
    fi

    if git -C "$CONFIG_BACKUP_DIR" push -u origin "$branch"; then
        ok "Pushed to remote successfully"
    else
        err "Push to remote failed"
        return 1
    fi
}

# ── Tarball Creation ────────────────────────────────────────────────────────

create_tarball() {
    log "Creating configuration tarball..."

    mkdir -p "$CONFIG_TARBALL_DIR"

    # First, copy config files to a temporary staging directory
    local staging_dir
    staging_dir=$(mktemp -d "${TMPDIR:-/tmp}/rustpbx-config-XXXXXX")

    # Ensure cleanup on exit
    trap 'rm -rf "$staging_dir"' EXIT

    local file_count
    file_count=$(copy_config_files "$staging_dir/rustpbx-config")

    if [ "$file_count" -eq 0 ]; then
        warn "No configuration files found to archive"
        rm -rf "$staging_dir"
        trap - EXIT
        return 0
    fi

    local timestamp
    timestamp=$(date '+%Y%m%d-%H%M%S')
    local tarball="$CONFIG_TARBALL_DIR/rustpbx-config-${timestamp}.tar.gz"

    tar czf "$tarball" -C "$staging_dir" "rustpbx-config"

    # Clean up staging
    rm -rf "$staging_dir"
    trap - EXIT

    local size
    size=$(stat -c%s "$tarball" 2>/dev/null || stat -f%z "$tarball" 2>/dev/null || echo "unknown")
    ok "Tarball created: $tarball ($size bytes)"

    # Rotate old tarballs — keep the most recent 30
    local tarball_count
    tarball_count=$(find "$CONFIG_TARBALL_DIR" -maxdepth 1 -name 'rustpbx-config-*.tar.gz' -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$tarball_count" -gt 30 ]; then
        local to_remove=$((tarball_count - 30))
        log "Rotating tarballs: $tarball_count total, removing $to_remove oldest"
        find "$CONFIG_TARBALL_DIR" -maxdepth 1 -name 'rustpbx-config-*.tar.gz' -type f \
            | sort \
            | head -n "$to_remove" \
            | while read -r old; do
                log "Removing old tarball: $(basename "$old")"
                rm -f "$old"
            done
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
            --git-init)
                DO_GIT_INIT=1
                shift
                ;;
            --commit)
                DO_COMMIT=1
                shift
                ;;
            --tarball)
                DO_TARBALL=1
                shift
                ;;
            --push)
                DO_PUSH=1
                shift
                ;;
            --backup-dir)
                [ $# -ge 2 ] || die "--backup-dir requires a DIR argument"
                CONFIG_BACKUP_DIR="$2"
                shift 2
                ;;
            --tarball-dir)
                [ $# -ge 2 ] || die "--tarball-dir requires a DIR argument"
                CONFIG_TARBALL_DIR="$2"
                shift 2
                ;;
            --remote)
                [ $# -ge 2 ] || die "--remote requires a URL argument"
                CONFIG_GIT_REMOTE="$2"
                shift 2
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
    # At least one action must be requested
    if [ "$DO_GIT_INIT" -eq 0 ] && [ "$DO_COMMIT" -eq 0 ] && [ "$DO_TARBALL" -eq 0 ]; then
        die "No action specified. Use --git-init, --commit, and/or --tarball. See --help."
    fi

    # Push requires commit
    if [ "$DO_PUSH" -eq 1 ] && [ "$DO_COMMIT" -eq 0 ] && [ "$DO_GIT_INIT" -eq 0 ]; then
        warn "--push specified without --commit; will attempt to push existing commits"
    fi

    # Git operations require git
    if [ "$DO_GIT_INIT" -eq 1 ] || [ "$DO_COMMIT" -eq 1 ] || [ "$DO_PUSH" -eq 1 ]; then
        if ! command -v git &>/dev/null; then
            die "git is not installed. Please install git and try again."
        fi
    fi

    # Tarball requires tar
    if [ "$DO_TARBALL" -eq 1 ]; then
        if ! command -v tar &>/dev/null; then
            die "tar is not installed. Please install tar and try again."
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

    log "=== RustPBX Configuration Backup ==="
    log "Config source: $RUSTPBX_CONFIG"
    log "Backup directory: $CONFIG_BACKUP_DIR"

    local failed=0

    # Step 1: Git init (if requested)
    if [ "$DO_GIT_INIT" -eq 1 ]; then
        git_init || { err "Git init failed"; failed=1; }
    fi

    # Step 2: Copy + commit (if requested)
    if [ "$DO_COMMIT" -eq 1 ]; then
        git_commit || { err "Git commit failed"; failed=1; }
    fi

    # Step 3: Push (if requested)
    if [ "$DO_PUSH" -eq 1 ]; then
        git_push || { err "Git push failed"; failed=1; }
    fi

    # Step 4: Create tarball (if requested)
    if [ "$DO_TARBALL" -eq 1 ]; then
        create_tarball || { err "Tarball creation failed"; failed=1; }
    fi

    # Summary
    if [ "$failed" -eq 0 ]; then
        ok "=== Configuration backup completed successfully ==="
    else
        err "=== Configuration backup completed with errors ==="
        exit 1
    fi
}

main "$@"
