#!/usr/bin/env bash
# docs/harness/bin/baseline.sh
#
# Lightweight static repository snapshot for harness drift review.
# This is evidence only. It is not a substitute for sensors.sh or CI.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT" || exit 2

OUT="docs/harness/.baseline-last"
TMP="$(mktemp "${TMPDIR:-/tmp}/engram-baseline.XXXXXX")" || exit 3

if git status --porcelain >/tmp/engram-baseline-status.$$ 2>/dev/null && [ -s /tmp/engram-baseline-status.$$ ]; then
  DIRTY="yes"
else
  DIRTY="no"
fi
rm -f /tmp/engram-baseline-status.$$

if command -v just >/dev/null 2>&1; then
  CI_RUNNER="just"
else
  CI_RUNNER="make"
fi

{
  echo "timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "branch=$(git branch --show-current 2>/dev/null || true)"
  echo "commit=$(git log -1 --format=%H 2>/dev/null || true)"
  echo "dirty=$DIRTY"
  echo "cargo=$(cargo --version 2>/dev/null || echo missing)"
  echo "rustc=$(rustc --version 2>/dev/null || echo missing)"
  echo "ci_runner=$CI_RUNNER"
  echo "schema_version=$(rg -n 'SCHEMA_VERSION' src/storage 2>/dev/null | head -1 | sed 's/[[:space:]]\+/ /g')"
  echo "mcp_reference_sections=$(grep -c '^### ' docs/MCP_TOOLS.md 2>/dev/null || echo 0)"
  echo "harness_scripts=$(find docs/harness/bin -maxdepth 1 -type f | wc -l | tr -d ' ')"
  echo "review_artifacts=$(find docs/harness/reviews -maxdepth 1 -type f -name '*.md' 2>/dev/null | wc -l | tr -d ' ')"
} > "$TMP"

mv "$TMP" "$OUT"
echo "Baseline written to $OUT"
cat "$OUT"
