#!/usr/bin/env bash
# docs/harness/bin/review-gate.sh
#
# Cross-CLI / cross-model review gate for engram harness.
#
# Supports the user's current workflow (Claude Code + Zed Gemini CLI side-by-side).
#
# Usage:
#   review-gate.sh pre  <task-id>                    # advisory; generates prompt + writes pre artifact
#   review-gate.sh post <task-id>                    # hard gate; reviews uncommitted or HEAD
#   review-gate.sh post <task-id> --range=main..HEAD # explicit range
#   review-gate.sh post <task-id> --review-file reviews/xxx.md  # provide reviewer output directly
#
# Environment:
#   REVIEWER_CLI=claude|gemini|codex|ollama|manual (affects prompt tone; default "manual")
#   REVIEWER_TIMEOUT_SECS=...                        (future non-interactive exec)
#
# The script builds a rich prompt including SPEC, INVARIANTS, WHAT_WE_DONT_DO, GATES, CODE_REVIEW_POLICY,
# docs/harness/security/anthropic-reference-harness.md, .claude/scan-extras.txt,
# .claude/fp-rules.txt, fake-success patterns, and the relevant diff (with harness artifacts excluded).
# It writes artifacts to docs/harness/reviews/ with iteration versioning.
# Verdict is parsed from an explicit marker line:
#   REVIEW_VERDICT: PASS ...
# (or FAIL ...), not from any other PASS/FAIL text.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: cannot resolve repo root" >&2
  exit 2
fi
cd "$REPO_ROOT"

TASK_ID="${2:-}"
MODE="${1:-}"
RANGE=""
REVIEW_FILE=""
PREV_REVIEW=""

# Parse remaining args
shift 2 || true
while [ "$#" -gt 0 ]; do
  case "$1" in
    --range) RANGE="$2"; shift 2 ;;
    --review-file) REVIEW_FILE="$2"; shift 2 ;;
    --prev) PREV_REVIEW="$2"; shift 2 ;;
    *) echo "ERROR: unknown arg $1" >&2; exit 2 ;;
  esac
done

if [ -z "$TASK_ID" ]; then
  echo "Usage: review-gate.sh <pre|post> <task-id> [--range ...] [--review-file ...]" >&2
  exit 2
fi

if [ "$MODE" != "pre" ] && [ "$MODE" != "post" ]; then
  echo "ERROR: mode must be 'pre' or 'post' (got '$MODE')" >&2
  echo "Usage: review-gate.sh <pre|post> <task-id> [--range ...] [--review-file ...]" >&2
  exit 2
fi

mkdir -p docs/harness/reviews

detect_harness_script_changes() {
  if [ -n "$RANGE" ]; then
    git diff --name-only "$RANGE" -- docs/harness/bin 2>/dev/null || true
  elif git diff --quiet --exit-code; then
    git show --name-only --format='' HEAD -- docs/harness/bin 2>/dev/null || true
  else
    {
      git diff --name-only -- docs/harness/bin 2>/dev/null || true
      git diff --cached --name-only -- docs/harness/bin 2>/dev/null || true
    } | sort -u
  fi
}

HARNESS_SCRIPT_CHANGES="$(detect_harness_script_changes)"

# Timestamp + iteration handling for artifact naming
DATE="$(date -u +%Y-%m-%d)"
BASE_NAME="${DATE}-${TASK_ID}"

# Find next iteration number for this (date + task)
NEXT_ITER=1
for f in docs/harness/reviews/"${BASE_NAME}"-v*-{pre,post}.md docs/harness/reviews/"${BASE_NAME}"-{pre,post}.md; do
  if [ -f "$f" ]; then
    # crude but effective
    CANDIDATE=$(echo "$f" | sed -E 's/.*-v([0-9]+)-.*/\1/' | grep -E '^[0-9]+$' || echo 1)
    if [ "$CANDIDATE" -ge "$NEXT_ITER" ]; then
      NEXT_ITER=$((CANDIDATE + 1))
    fi
  fi
done

SUFFIX=""
if [ "$NEXT_ITER" -gt 1 ]; then
  SUFFIX="-v${NEXT_ITER}"
fi

ARTIFACT_TYPE="post"
if [ "$MODE" = "pre" ]; then
  ARTIFACT_TYPE="pre"
fi

ARTIFACT_PATH="docs/harness/reviews/${BASE_NAME}${SUFFIX}-${ARTIFACT_TYPE}.md"
RAW_PATH="${ARTIFACT_PATH}.raw"

# Build the prompt
PROMPT_FILE="/tmp/engram-review-${TASK_ID}-$$.md"

{
  echo "# Engram Harness — External Reviewer Prompt"
  echo
  echo "**Task**: $TASK_ID"
  echo "**Mode**: $MODE"
  echo "**Date (UTC)**: $DATE"
  echo
  echo "## Instructions for the Reviewer"
  echo
  echo "You are acting as an independent senior engineer reviewing a diff for the engram project."
  echo "You were NOT the implementer. Your job is to find real problems introduced by the change."
  echo
  echo "Read the following documents (they are the source of truth for this review):"
  echo
  echo "- docs/harness/SPEC.md"
  echo "- docs/harness/INVARIANTS.md (process invariants — canonical)"
  echo "- docs/harness/WHAT_WE_DONT_DO.md (negative scope — no hidden expansion)"
  echo "- docs/harness/GATES.md (especially the fake-success patterns section)"
  echo "- docs/harness/CODE_REVIEW_POLICY.md (this policy)"
  echo "- docs/harness/security/anthropic-reference-harness.md (security boundary)"
  echo "- .claude/scan-extras.txt and .claude/fp-rules.txt (org-specific scan/triage tuning)"
  echo "- docs/harness/README.md (workflow)"
  echo "- Root INVARIANTS.md (data layer invariants for the memory system)"
  echo
  echo "Then review the diff below."
  echo
  echo "Additional harness-specific requirements:"
  echo "- Compare scope against docs/harness/WHAT_WE_DONT_DO.md. Flag hidden scope creep, gate weakening, or product changes bundled into harness work."
  echo "- Security boundary: flag autonomous Engram execution, implied sandboxing, credential mounts, network/egress expansion, or C/C++/ASAN pipeline import unless an ADR and explicit target contract are present."
  echo "- Tuning files: ensure .claude/scan-extras.txt and .claude/fp-rules.txt augment scan/triage behavior without weakening core INVARIANTS/GATES/POLICY or adding blanket suppressions."
  echo "- Review Canvas: if the diff is complex, verify that a matching docs/harness/canvas/YYYY-MM-DD-<task-id>.md exists and includes approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table."
  echo "- Harness script changes under docs/harness/bin/* are process-critical. Inspect shell safety, path handling, parseability, read-only guarantees, and whether the script weakens any existing gate."
  if [ -n "$HARNESS_SCRIPT_CHANGES" ]; then
    echo
    echo "Harness script changes detected:"
    printf '%s\n' "$HARNESS_SCRIPT_CHANGES" | sed 's/^/- /'
  fi
  echo
  echo "## Key Fake-Success Patterns (hunt these actively)"
  echo
  echo "1. Tests green only because local-embeddings feature was used; CI Linux parity fails."
  echo "2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes."
  echo "3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated."
  echo "4. Clippy clean but unwrap/expect in hot MCP handler, storage, or hook paths."
  echo "5. Snapshot/attestation tests pass but Merkle or crypto behavior changed."
  echo "6. Hooks (session_end, post_tool_use, etc.) or intelligence modules changed without integration coverage."
  echo "7. Harness doctor or sensors would have caught this but were not run."
  echo "8. Progress docs (harness or active plan) not updated for a domain change."
  echo "9. Cross-SDK (python/typescript) contract drift not reflected."
  echo "10. Reviewer is being shown a self-referential or incomplete prompt (call it out)."
  echo "11. Security boundary drift: static/read-only default weakened, autonomous execution implied, missing ADR/sandbox/egress/target contract, credential mounts allowed, or Anthropic C/C++/ASAN pipeline imported as default."
  echo
  echo "## Diff Under Review"
  echo
  echo '```diff'
} > "$PROMPT_FILE"

# Compute diff (respecting excludes)
GIT_EXCLUDES=(
  ":(exclude)docs/harness/reviews/*"
  ":(exclude)docs/harness/progress/*"
  ":(exclude)target/*"
  ":(exclude)engram-wasm/target/*"
  ":(exclude)coverage/*"
  ":(exclude)node_modules/*"
  ":(exclude)sdks/python/__pycache__/*"
  ":(exclude)sdks/typescript/node_modules/*"
)

if [ -n "$RANGE" ]; then
  git diff --unified=0 "$RANGE" -- "${GIT_EXCLUDES[@]}" >> "$PROMPT_FILE" 2>/dev/null || echo "(diff for range $RANGE produced no output or error)" >> "$PROMPT_FILE"
else
  # Default: changes vs HEAD if clean, or staged+unstaged if dirty
  if git diff --quiet --exit-code; then
    # clean working tree → review last commit
    git show --unified=0 HEAD -- "${GIT_EXCLUDES[@]}" >> "$PROMPT_FILE" 2>/dev/null || echo "(no diff in last commit)" >> "$PROMPT_FILE"
  else
    git diff --unified=0 -- "${GIT_EXCLUDES[@]}" >> "$PROMPT_FILE" 2>/dev/null || echo "(no diff)" >> "$PROMPT_FILE"
  fi
fi

{
  echo '```'
  echo
  echo "## Previous Review Context (if any)"
  echo
  if [ -n "$PREV_REVIEW" ] && [ -f "$PREV_REVIEW" ]; then
    echo "Previous review file: $PREV_REVIEW"
    echo "(Only [BLOCKER] and [HIGH] findings from a prior FAIL are carried; PASS/LOW are not.)"
    echo '```'
    # Very lightweight carry — in a real implementation we would parse and filter
    grep -E '^\[BLOCKER\]|\[HIGH\]' "$PREV_REVIEW" | head -20 || echo "(no high-severity carried findings parsed)"
    echo '```'
  else
    echo "(no previous review supplied for continuity)"
  fi
  echo
  echo "## Output Contract (strict)"
  echo
  echo "Your entire response must start with exactly one of:"
  echo
  echo "PASS <one-line summary of what was reviewed and why it is safe>"
  echo
  echo "or"
  echo
  echo "FAIL <one-line summary of the most important problem(s)>"
  echo
  echo "Then a short bullet list using [BLOCKER], [HIGH], [MED], [LOW]."
  echo "At most 3 substantive findings. Evidence and location required for each."
  echo "If nothing substantive: exactly one bullet with [LOW] No issues found..."
  echo
  echo "Remember: you are the external reviewer. Be evidence-driven and skeptical."
  echo
  echo "Machine-parseable verdict (required):"
  echo "Add exactly one line, anywhere in the response, beginning with:"
  echo "REVIEW_VERDICT: PASS <one-line summary>"
  echo "or"
  echo "REVIEW_VERDICT: FAIL <one-line summary>"
  echo "This line is required for hard post-gate enforcement."
} >> "$PROMPT_FILE"

# Write the prompt as the .raw for the artifact (human + parser can see it)
cp "$PROMPT_FILE" "$RAW_PATH"

echo "Review prompt written to: $RAW_PATH"
echo "Artifact target: $ARTIFACT_PATH"
echo

if [ "$MODE" = "pre" ]; then
  echo "=== PRE-GATE (advisory) ==="
  echo "The prompt above is advisory input for the implementer."
  echo "Copy the content of $RAW_PATH into your other CLI (Zed agent picker: Gemini CLI / Claude) if doing cross-CLI review."
  echo "Save the reviewer's full response as $ARTIFACT_PATH"
  echo "Pre-gates never block (exit 0)."
  cp "$RAW_PATH" "$ARTIFACT_PATH" 2>/dev/null || true
  echo "Pre-gate artifact (prompt copy) saved to $ARTIFACT_PATH"
  exit 0
fi

# POST mode — hard gate
echo "=== POST-GATE (hard) ==="
echo "For the dual-CLI workflow:"
echo "  1. Open $RAW_PATH"
echo "  2. In Zed, select the Gemini CLI agent from the agent picker, then paste the full prompt"
echo "  3. Save the complete reviewer response to $ARTIFACT_PATH"
echo "  4. Re-run this script with --review-file $ARTIFACT_PATH (or just run it again after the file exists)"
echo "  Zed handoff: Agent picker -> Gemini CLI -> paste $RAW_PATH -> save response to $ARTIFACT_PATH"
echo

if [ -n "$REVIEW_FILE" ] && [ -f "$REVIEW_FILE" ]; then
  ARTIFACT_PATH="$REVIEW_FILE"
fi

if [ ! -f "$ARTIFACT_PATH" ]; then
  if [ -n "$HARNESS_SCRIPT_CHANGES" ]; then
    echo "Harness script changes were detected under docs/harness/bin/*:"
    printf '%s\n' "$HARNESS_SCRIPT_CHANGES" | sed 's/^/- /'
    echo
    echo "These changes require independent post-review evidence."
    echo "Generated prompt: $RAW_PATH"
    echo "Save the external/human reviewer response with REVIEW_VERDICT: PASS|FAIL and re-run with --review-file."
    exit 1
  fi
  echo "No review file present at $ARTIFACT_PATH (or --review-file)."
  echo "This is expected on first post-gate run in the dual-CLI flow."
  echo "After you obtain the review from the other CLI, re-invoke with --review-file pointing at it."
  echo "For now, exiting with code 0 (no verdict available to enforce)."
  exit 0
fi

# Parse verdict from the explicit marker.
VERDICT_LINE="$(grep -Ei '^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL)[[:space:]].+$' "$ARTIFACT_PATH" 2>/dev/null | tail -1 || true)"
VERDICT="$(printf '%s' "$VERDICT_LINE" | sed -E 's/^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL).*/\1/I' | tr '[:lower:]' '[:upper:]' || true)"

if [ -z "$VERDICT_LINE" ]; then
  echo "⚠️  POST-GATE: no explicit review marker found in $ARTIFACT_PATH"
  echo "Expected a line matching:"
  echo "REVIEW_VERDICT: PASS <one-line summary>"
  echo "or"
  echo "REVIEW_VERDICT: FAIL <one-line summary>"
  echo
  echo "Prompt-only artifacts and pre-gate files are not valid as post-gate inputs."
  echo "Run the reviewer and provide the final review response via --review-file."
  echo "If this is the first post-gate iteration, re-run without --review-file so a fresh prompt is generated."
  exit 1
fi

if [ "$VERDICT" != "PASS" ] && [ "$VERDICT" != "FAIL" ]; then
  echo "⚠️  POST-GATE: malformed review marker in $ARTIFACT_PATH"
  echo "Marker line: $VERDICT_LINE"
  exit 1
fi

if [ "$VERDICT" = "PASS" ]; then
  echo "✅ POST-GATE PASS (from $ARTIFACT_PATH)"
  echo "Verdict marker: $VERDICT_LINE"
  exit 0
elif [ "$VERDICT" = "FAIL" ]; then
  echo "❌ POST-GATE FAIL (from $ARTIFACT_PATH)"
  echo "Verdict marker: $VERDICT_LINE"
  echo
  echo "Findings (last 30 lines of artifact for context):"
  tail -30 "$ARTIFACT_PATH"
  exit 1
fi
