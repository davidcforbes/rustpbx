#!/usr/bin/env bash
# upstream-sync.sh — Evaluate and merge upstream changes from restsend/rustpbx
#
# Usage:
#   ./scripts/upstream-sync.sh check     # See what's new upstream (no changes)
#   ./scripts/upstream-sync.sh diff      # Show full diff of upstream changes
#   ./scripts/upstream-sync.sh merge     # Merge upstream into local main
#   ./scripts/upstream-sync.sh log       # Show upstream commit log since last sync

set -euo pipefail

ACTION="${1:-check}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[upstream]${NC} $*"; }
warn() { echo -e "${YELLOW}[upstream]${NC} $*"; }

# Ensure we have the upstream remote
if ! git remote get-url upstream &>/dev/null; then
    log "Adding upstream remote..."
    git remote add upstream https://github.com/restsend/rustpbx.git
fi

log "Fetching upstream..."
git fetch upstream 2>/dev/null

# Find the merge base (where our main and upstream diverged)
LOCAL_HEAD=$(git rev-parse main)
UPSTREAM_HEAD=$(git rev-parse upstream/main)
MERGE_BASE=$(git merge-base main upstream/main)

if [ "$LOCAL_HEAD" = "$UPSTREAM_HEAD" ]; then
    echo -e "${GREEN}[upstream] Already up to date with upstream.${NC}"
    exit 0
fi

BEHIND=$(git rev-list --count main..upstream/main)
AHEAD=$(git rev-list --count upstream/main..main)

log "Status: ${BEHIND} commits behind upstream, ${AHEAD} commits ahead (local changes)"

case "$ACTION" in
    check)
        echo ""
        log "=== Upstream commits not yet merged ==="
        git log --oneline main..upstream/main | head -30
        echo ""
        if [ "$BEHIND" -gt 0 ]; then
            warn "Run './scripts/upstream-sync.sh diff' to see changes"
            warn "Run './scripts/upstream-sync.sh merge' to merge them"
        fi
        ;;

    log)
        echo ""
        log "=== Full upstream commit log since last sync ==="
        git log --stat main..upstream/main
        ;;

    diff)
        echo ""
        log "=== Diff: upstream changes not yet in our main ==="
        git diff main...upstream/main --stat
        echo ""
        echo "---"
        log "Showing file-level diff (press q to quit)..."
        git diff main...upstream/main
        ;;

    files)
        echo ""
        log "=== Files changed upstream since last sync ==="
        git diff main...upstream/main --name-status
        ;;

    merge)
        CURRENT_BRANCH=$(git branch --show-current)
        if [ "$CURRENT_BRANCH" != "main" ]; then
            warn "Switching to main branch first..."
            git checkout main
        fi

        log "Creating backup tag before merge..."
        git tag -f "pre-upstream-sync-$(date +%Y%m%d)" main

        log "Merging upstream/main into main..."
        if git merge upstream/main --no-edit; then
            echo -e "${GREEN}[upstream] Merge successful!${NC}"
            log "New commits merged: $BEHIND"
            log ""
            log "Review the merge, then push with: git push origin main"
        else
            echo -e "${RED}[upstream] Merge has conflicts!${NC}"
            echo ""
            log "Resolve conflicts, then:"
            log "  git add <resolved files>"
            log "  git commit"
            log "  git push origin main"
            echo ""
            log "Or abort with: git merge --abort"
        fi
        ;;

    *)
        echo "Usage: $0 {check|log|diff|files|merge}"
        echo ""
        echo "  check  — Quick summary of upstream changes (default)"
        echo "  log    — Full commit log of upstream changes"
        echo "  diff   — Show actual code diff"
        echo "  files  — List changed files only"
        echo "  merge  — Merge upstream into local main"
        exit 1
        ;;
esac
