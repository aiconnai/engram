#!/usr/bin/env bash
# docs/harness/bin/bootstrap.sh
#
# Contract (Engram):
#   - Exits 0 even if files are missing (degrades gracefully)
#   - Prints <= 55 lines to stdout (slightly higher than reference due to Rust/MCP specifics)
#   - Completes in < 800 ms
#   - Output contains: "engram harness state", "Branch:", "Active sprint", "Read next"
#   - No side effects (read-only)
#   - Security contract remains static/read-only first; see docs/harness/security/anthropic-reference-harness.md
#   - Safe for any agent CLI: Claude Code, Claude Code Sonnet reviewer, Codex, Cursor, Aider, etc.
#
# Called manually at session start per AGENTS.md / Claude.md.
# Later can be wired to session_start hook or MCP tool.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd 2>/dev/null || echo .)"
cd "$REPO_ROOT" 2>/dev/null || true

BIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd 2>/dev/null || echo docs/harness/bin)"
if [ -f "$BIN_DIR/lib.sh" ]; then
  # shellcheck source=docs/harness/bin/lib.sh
  source "$BIN_DIR/lib.sh"
else
  field_value() {
    printf ''
  }
fi

echo "=== engram harness state ==="

BRANCH="$(git branch --show-current 2>/dev/null || echo unknown)"
if git status --porcelain 2>/dev/null | grep -q .; then
  DIRTY_STATE="dirty"
else
  DIRTY_STATE="clean"
fi
task_id_from_value() {
  local value="$1"
  value="${value%% — *}"
  value="${value%% - *}"
  value="${value%% *}"
  printf '%s' "$value"
}

review_for_task() {
  local task_id="$1"
  local review=""

  if [ -n "$task_id" ]; then
    review="$(find docs/harness/reviews -type f -name "*${task_id}*post.md" ! -name '*.raw' 2>/dev/null | sort | tail -1 || true)"
    if [ -z "$review" ]; then
      review="$(find docs/harness/reviews -type f -name "*${task_id}*.md" ! -name '*.raw' 2>/dev/null | sort | tail -1 || true)"
    fi
  fi

  printf '%s' "$review"
}

ACTIVE_TASK=""

echo "Branch: ${BRANCH} (${DIRTY_STATE})"
echo "Last commit: $(git log -1 --oneline 2>/dev/null || echo none)"
echo

echo "--- Active work ---"
if [ -f docs/harness/progress.md ]; then
  ACTIVE_TASK="$(field_value docs/harness/progress.md "Active task")"
  echo "Sprint: $(field_value docs/harness/progress.md "Active sprint")"
  echo "Task: $ACTIVE_TASK"
  echo "Active plan: $(field_value docs/harness/progress.md "Active plan")"
else
  echo "(progress.md missing — run doctor.sh)"
fi
echo

echo "--- Last review gate ---"
ACTIVE_TASK_ID="$(task_id_from_value "$ACTIVE_TASK")"
LATEST_REVIEW="$(review_for_task "$ACTIVE_TASK_ID")"
if [ -n "${LATEST_REVIEW:-}" ]; then
  echo "File: $LATEST_REVIEW"
  VERDICT_LINE="$(grep -iE '^(PASS|FAIL)([[:space:]:.,;-]|$)' "$LATEST_REVIEW" 2>/dev/null | tail -1 || true)"
  if [ -n "$VERDICT_LINE" ]; then
    echo "Verdict: $VERDICT_LINE"
  else
    echo "Verdict: (no parseable PASS/FAIL line)"
  fi
elif [ -n "$ACTIVE_TASK_ID" ]; then
  echo "(no review artifact for active task: $ACTIVE_TASK_ID)"
else
  echo "(no reviews yet — review-gate.sh not run)"
fi
echo

echo "--- Sensors (last run) ---"
if [ -f docs/harness/.sensors-last ]; then
  cat docs/harness/.sensors-last
else
  echo "(never run — sensors.sh not executed yet)"
fi
echo

echo "--- Engram specifics ---"
if command -v cargo >/dev/null 2>&1; then
  echo "Cargo: $(cargo --version 2>/dev/null | head -1 || echo present)"
else
  echo "Cargo: not on PATH"
fi

if [ -x "$(command -v just 2>/dev/null)" ]; then
  echo "just ci available: yes (preferred local gate)"
elif [ -f justfile ] && command -v make >/dev/null 2>&1; then
  echo "just missing; fallback: make ci"
elif [ -f justfile ]; then
  echo "justfile present, but 'just' command unavailable"
else
  echo "justfile: missing (run with caution)"
fi

# Quick MCP surface signal (non-authoritative): derive from the source registry,
# not generated docs, so the bootstrap does not count markdown headings.
if [ -f src/mcp/tools/registry.rs ]; then
  if command -v python3 >/dev/null 2>&1; then
    if MCP_COUNTS="$(python3 -c '
from pathlib import Path
import re

catalog_dir = Path("src/mcp/tools/catalog")
if catalog_dir.exists():
    registry = "\n".join(p.read_text() for p in catalog_dir.glob("*.rs"))
else:
    registry = Path("src/mcp/tools/registry.rs").read_text()
tools_mod = Path("src/mcp/tools/mod.rs").read_text()
all_names = re.findall(r"\bname:\s*\"([^\"]+)\"", registry)
feature_gated = set()
match_body = re.search(
    r"fn tool_feature_available\(name: &str\) -> bool \{.*?match name \{(.*?)\n\s*_\s*=>\s*true,",
    tools_mod,
    re.S,
)
if match_body:
    for arm in re.finditer(r"((?:\s*\"[^\"]+\"\s*\|?)+)\s*=>\s*cfg!\(", match_body.group(1)):
        feature_gated.update(re.findall(r"\"([^\"]+)\"", arm.group(1)))
active_count = len([name for name in all_names if name not in feature_gated])
print(f"{active_count} active / {len(all_names)} total")
' 2>/dev/null)"; then
      echo "MCP tools (source): ${MCP_COUNTS}"
    else
      MCP_TOTAL="$(grep -h -c 'ToolDef {' src/mcp/tools/catalog/*.rs 2>/dev/null || grep -c 'ToolDef {' src/mcp/tools/registry.rs 2>/dev/null || echo '?')"
      echo "MCP tools (source total): ${MCP_TOTAL}"
    fi
  else
    MCP_TOTAL="$(grep -h -c 'ToolDef {' src/mcp/tools/catalog/*.rs 2>/dev/null || grep -c 'ToolDef {' src/mcp/tools/registry.rs 2>/dev/null || echo '?')"
    echo "MCP tools (source total): ${MCP_TOTAL}"
  fi
elif [ -f docs/MCP_TOOLS.md ]; then
  MCP_COUNT="$(grep -c '^### `' docs/MCP_TOOLS.md 2>/dev/null || echo '?')"
  echo "MCP tools (docs fallback): ${MCP_COUNT}"
fi
echo

echo "--- Read next (in order) ---"
echo "  docs/harness/SPEC.md                 (active sprint scope)"
echo "  docs/harness/INVARIANTS.md           (hard process rules; canonical)"
echo "  docs/harness/WHAT_WE_DONT_DO.md      (negative scope; no hidden expansion)"
echo "  docs/harness/GATES.md                (sensors + review criteria + fake-success)"
echo "  docs/harness/CODE_REVIEW_POLICY.md   (local policy for external reviewer)"
echo "  docs/harness/security/anthropic-reference-harness.md (security boundary)"
echo "  docs/harness/README.md               (workflow + harness loop)"
echo "  docs/harness/progress.md             (live state)"
echo "  AGENTS.md + Claude.md                (onboarding)"
echo "  INVARIANTS.md (root)                 (data invariants)"
echo "  STANDARDS.md + ERRORS_AND_LESSONS.md (governance)"
echo
echo "Then run: bash docs/harness/bin/doctor.sh"
