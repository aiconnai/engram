#!/usr/bin/env bash
# docs/harness/bin/quarterly-audit.sh
#
# Evidence-only cleanup and drift audit for Engram.
# This script never deletes, archives, rewrites, or gates code.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT" || exit 2

TIMESTAMP="$(date -u +%Y-%m-%dT%H%M%SZ)"
REPORT_DIR="docs/harness/audits"
REPORT="$REPORT_DIR/${TIMESTAMP}-quarterly-audit.md"
LAST_FILE="docs/harness/.quarterly-audit-last"

mkdir -p "$REPORT_DIR"

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "FAIL: required command not found: $1" >&2
    exit 127
  fi
}

append_cmd() {
  local title="$1"
  local cmd="$2"
  local output
  local status

  {
    echo
    echo "### $title"
    echo
    echo '```bash'
    echo "$cmd"
    echo '```'
    echo
    echo '```text'
  } >> "$REPORT"

  output="$(bash -o pipefail -c "$cmd" 2>&1)"
  status=$?
  if [ -n "$output" ]; then
    printf '%s\n' "$output" >> "$REPORT"
  else
    echo "(no output)" >> "$REPORT"
  fi
  {
    echo "exit_status=$status"
    echo '```'
  } >> "$REPORT"
}

append_decision_table() {
  local title="$1"
  {
    echo
    echo "### $title"
    echo
    echo "| Item | Evidence | Decision | Owner | Follow-up |"
    echo "|---|---|---|---|---|"
    echo "|  |  | Keep / Archive / Delete |  |  |"
  } >> "$REPORT"
}

need git
need rg

cat > "$REPORT" <<EOF
# Quarterly Harness Audit

Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Repo: \`engram\`
Mode: evidence-only

This report gathers evidence for human cleanup and drift review. It does not declare pass/fail and does not delete, archive, or rewrite anything.

## How To Use

1. Review each evidence section.
2. Fill decision tables with \`Keep\`, \`Archive\`, or \`Delete\`.
3. Convert accepted cleanup into focused tasks or issues.
4. Keep exceptions documented in \`docs/harness/WHAT_WE_DONT_DO.md\`, \`docs/harness/INVARIANTS.md\`, or an ADR.
EOF

append_cmd "Current branch and commit" "git branch --show-current && git log -1 --oneline"
append_cmd "Working tree status" "git status --short"
append_cmd "Harness policy references" "rg -n 'WHAT_WE_DONT_DO|CODE_REVIEW_POLICY|review-gate|doctor.sh|sensors.sh|baseline.sh|quarterly-audit' docs/harness README.md AGENTS.md Claude.md 2>/dev/null | head -160"
append_cmd "Schema and migration references" "rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160"
append_cmd "MCP reference count and manual count risks" "rg -n 'MCP_TOOLS|[0-9]+\\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160"
append_cmd "Temporary, legacy, and cleanup markers" "rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180"
append_cmd "Optional dependencies and feature gates" "rg -n -i 'optional = true|features =|default-features|\\[features\\]' Cargo.toml sdks docs 2>/dev/null | head -180"
append_cmd "Harness generated artifacts volume" "find docs/harness/reviews docs/harness/progress docs/harness/audits -maxdepth 1 -type f 2>/dev/null | sort | wc -l | tr -d ' '"

append_decision_table "Harness Policy Decisions"
append_decision_table "MCP And Docs Drift Decisions"
append_decision_table "Storage And Migration Decisions"
append_decision_table "Cleanup Follow-ups"

{
  echo
  echo "## Human Review Notes"
  echo
  echo "- Decisions:"
  echo "- Follow-up issues:"
  echo "- Exceptions approved:"
  echo "- Next audit date:"
} >> "$REPORT"

printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$REPORT" > "$LAST_FILE"

echo "Quarterly audit evidence written to $REPORT"
echo "Last-audit pointer updated at $LAST_FILE"
