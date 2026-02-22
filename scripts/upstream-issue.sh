#!/usr/bin/env bash
# upstream-issue.sh — File a bug report on the upstream restsend/rustpbx repo
#
# Usage:
#   ./scripts/upstream-issue.sh                          # Interactive
#   ./scripts/upstream-issue.sh --title "Bug title" \
#       --body-file docs/github-issue-ack-routing.md     # From file
#   ./scripts/upstream-issue.sh --from-commit abc123     # From a fix commit
#
# Requires: gh CLI authenticated with GitHub

set -euo pipefail

UPSTREAM_REPO="restsend/rustpbx"

CYAN='\033[0;36m'
NC='\033[0m'
log() { echo -e "${CYAN}[issue]${NC} $*"; }

# Check gh auth
if ! gh auth status &>/dev/null 2>&1; then
    echo "Error: gh CLI not authenticated. Run: gh auth login"
    exit 1
fi

# Parse arguments
TITLE=""
BODY=""
BODY_FILE=""
FROM_COMMIT=""
LABELS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --title) TITLE="$2"; shift 2 ;;
        --body) BODY="$2"; shift 2 ;;
        --body-file) BODY_FILE="$2"; shift 2 ;;
        --from-commit) FROM_COMMIT="$2"; shift 2 ;;
        --label) LABELS="$LABELS --label $2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

# If --from-commit, generate issue body from the commit
if [ -n "$FROM_COMMIT" ]; then
    COMMIT_MSG=$(git log --format="%s" -1 "$FROM_COMMIT")
    COMMIT_BODY=$(git log --format="%b" -1 "$FROM_COMMIT")
    COMMIT_DIFF=$(git show --stat "$FROM_COMMIT")
    COMMIT_PATCH=$(git format-patch -1 "$FROM_COMMIT" --stdout)

    if [ -z "$TITLE" ]; then
        TITLE="Bug: $COMMIT_MSG"
    fi

    BODY="## Bug Report (from local fix)

**Commit:** \`$FROM_COMMIT\`
**Message:** $COMMIT_MSG

$COMMIT_BODY

## Files Changed

\`\`\`
$COMMIT_DIFF
\`\`\`

## Suggested Patch

<details>
<summary>Click to expand patch</summary>

\`\`\`diff
$COMMIT_PATCH
\`\`\`

</details>

## Environment

- RustPBX version: $(grep '^version' Cargo.toml 2>/dev/null | head -1 | cut -d'"' -f2 || echo 'unknown')
- Reported from: davidcforbes/rustpbx (private fork)
"
fi

# If --body-file, read the file
if [ -n "$BODY_FILE" ]; then
    if [ ! -f "$BODY_FILE" ]; then
        echo "Error: File not found: $BODY_FILE"
        exit 1
    fi
    BODY=$(cat "$BODY_FILE")
    # Try to extract title from the file (first # heading)
    if [ -z "$TITLE" ]; then
        TITLE=$(grep "^#\s" "$BODY_FILE" | head -1 | sed 's/^#\s*//')
    fi
fi

# Interactive mode if no title
if [ -z "$TITLE" ]; then
    echo ""
    log "=== File upstream issue on $UPSTREAM_REPO ==="
    echo ""
    echo "Available issue templates in docs/:"
    ls docs/github-issue-*.md 2>/dev/null | while read f; do
        echo "  $f"
    done
    echo ""
    echo "Tip: Use --body-file to submit from a template, or --from-commit to submit a fix"
    echo ""
    echo "Usage:"
    echo "  $0 --title 'Bug title' --body 'Description'"
    echo "  $0 --body-file docs/github-issue-ack-routing.md"
    echo "  $0 --from-commit abc1234"
    exit 0
fi

# Submit the issue
log "Filing issue on $UPSTREAM_REPO..."
log "Title: $TITLE"

gh issue create \
    --repo "$UPSTREAM_REPO" \
    --title "$TITLE" \
    --body "$BODY" \
    $LABELS

log "Issue filed successfully!"
