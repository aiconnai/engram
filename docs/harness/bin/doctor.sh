#!/usr/bin/env bash
# docs/harness/bin/doctor.sh
#
# Fast read-only consistency check for the engram harness layout and wiring.
# Exits 0 when consistent, 1 on validation failures, 2 on usage/env errors.
#
# Validates:
# - Required files and executability
# - Cross-references between README, bootstrap, review-gate, GATES, CODE_REVIEW_POLICY
# - Drift between SPEC.md and progress.md (sprint/task/plan)
# - Active plan file exists
# - Latest review for active task has parseable PASS/FAIL and explicit REVIEW_VERDICT marker (if present)
# - .sensors-last format (if present)
# - bootstrap.sh output size and exit code
# - Exclusion records (if sensors-last indicates pass_with_exclusion)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: cannot resolve repo root from script location" >&2
  exit 2
fi

cd "$REPO_ROOT"

FAILURES=()
WARNINGS=()

fail() { FAILURES+=("$1"); }
warn() { WARNINGS+=("$1"); }

require_file() {
  local path="$1"
  [ -f "$path" ] || fail "missing file: $path"
}

require_exec() {
  local path="$1"
  [ -x "$path" ] || fail "not executable: $path"
}

require_dir() {
  local path="$1"
  [ -d "$path" ] || fail "missing directory: $path"
}

require_grep() {
  local path="$1"
  local pattern="$2"
  local label="$3"
  if [ ! -f "$path" ] || ! grep -qE "$pattern" "$path"; then
    fail "missing reference in $path: $label"
  fi
}

# Core required structure
require_file docs/harness/SPEC.md
require_file docs/harness/INVARIANTS.md
require_file docs/harness/WHAT_WE_DONT_DO.md
require_file docs/harness/GATES.md
require_file docs/harness/CODE_REVIEW_POLICY.md
require_file docs/harness/README.md
require_file docs/harness/progress.md
require_file docs/harness/bin/bootstrap.sh
require_file docs/harness/bin/doctor.sh
require_file docs/harness/bin/baseline.sh
require_file docs/harness/bin/quarterly-audit.sh
require_dir docs/harness/progress
require_dir docs/harness/reviews
require_dir docs/harness/known-issues
require_dir docs/harness/canvas
require_dir docs/harness/audits
require_file docs/harness/canvas/README.md
require_file docs/harness/canvas/TEMPLATE.md

# Scripts that must be executable (review-gate and check-commit-msg are optional in early v0 but preferred)
require_exec docs/harness/bin/bootstrap.sh
require_exec docs/harness/bin/doctor.sh
require_exec docs/harness/bin/baseline.sh
require_exec docs/harness/bin/quarterly-audit.sh

# If the advanced scripts exist, they should be executable
[ -f docs/harness/bin/sensors.sh ] && require_exec docs/harness/bin/sensors.sh || true
[ -f docs/harness/bin/review-gate.sh ] && require_exec docs/harness/bin/review-gate.sh || true
[ -f docs/harness/bin/check-commit-msg.sh ] && require_exec docs/harness/bin/check-commit-msg.sh || true

# Cross-references (bootstrap + README point at the policy and doctor)
require_grep docs/harness/bin/bootstrap.sh 'CODE_REVIEW_POLICY\.md' 'read-next includes the local review policy'
require_grep docs/harness/bin/bootstrap.sh 'WHAT_WE_DONT_DO\.md' 'read-next includes negative-scope policy'
require_grep docs/harness/README.md 'WHAT_WE_DONT_DO\.md' 'workflow mentions the negative-scope policy'
require_grep docs/harness/README.md 'CODE_REVIEW_POLICY\.md' 'structure table or workflow mentions the policy file'
require_grep docs/harness/README.md 'doctor\.sh' 'workflow mentions the doctor check'
require_grep docs/harness/README.md 'known-issues/' 'structure table or workflow mentions known issues'
require_grep docs/harness/README.md 'baseline\.sh' 'workflow mentions baseline snapshots'
require_grep docs/harness/README.md 'quarterly-audit\.sh' 'workflow mentions evidence-only audits'
require_grep docs/harness/README.md 'Sensor modes' 'workflow lists optional sensor modes'
require_grep docs/harness/GATES.md 'WHAT_WE_DONT_DO\.md' 'gates reference negative-scope policy'
require_grep docs/harness/GATES.md 'Review Canvas' 'gates define review canvas requirement'
require_grep docs/harness/GATES.md 'baseline\.sh' 'gates document baseline snapshots'
require_grep docs/harness/GATES.md 'quarterly-audit\.sh' 'gates document evidence-only audit'
require_grep docs/harness/GATES.md 'optional lanes do not replace the full gate' 'gates preserve full sensor gate'
require_grep docs/harness/GATES.md 'docs/harness/bin' 'gates protect harness script changes'
require_grep docs/harness/GATES.md 'Exclus' 'documented exclusion policy exists'
require_grep docs/harness/GATES.md 'known-issue' 'exclusion policy points at known-issue docs'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'WHAT_WE_DONT_DO\.md' 'review policy enforces negative-scope policy'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'Review Canvas' 'review policy checks complex-change canvas evidence'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'Harness script changes' 'review policy checks harness scripts directly'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'Finding Format|Finding format|severidade' 'review policy defines finding format'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'PASS <resumo|Harness Output Contract' 'review policy defines output contract'
require_grep docs/harness/bin/review-gate.sh 'WHAT_WE_DONT_DO\.md' 'review-gate prompt includes negative-scope policy'
require_grep docs/harness/bin/review-gate.sh 'Review Canvas' 'review-gate prompt includes review canvas checks'
require_grep docs/harness/bin/review-gate.sh 'docs/harness/bin' 'review-gate protects harness script changes'
require_grep docs/harness/bin/sensors.sh 'quick' 'sensors supports quick mode'
require_grep docs/harness/bin/sensors.sh 'full' 'sensors supports full mode'
require_grep docs/harness/bin/sensors.sh 'docs' 'sensors supports docs mode'
require_grep docs/harness/bin/sensors.sh 'mcp' 'sensors supports mcp mode'
require_grep docs/harness/bin/sensors.sh 'baseline' 'sensors supports baseline mode'

field_value() {
  local file="$1"
  local key="$2"
  awk -F'|' -v key="$key" '
    $2 ~ "^[[:space:]]*" key "[[:space:]]*$" {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", $3)
      gsub(/^`|`$/, "", $3)
      print $3
      exit
    }
  ' "$file" 2>/dev/null || true
}

field_from_file() {
  local file="$1"
  local key="$2"
  sed -n "s/^${key}=//p" "$file" 2>/dev/null | head -n 1 || true
}

normalize_harness_path() {
  local path="$1"
  case "$path" in
    docs/harness/*) printf '%s' "$path" ;;
    ./progress/*) printf 'docs/harness/%s' "${path#./}" ;;
    progress/*) printf 'docs/harness/%s' "$path" ;;
    *) printf '%s' "$path" ;;
  esac
}

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

# Drift checks between SPEC and progress
if [ -f docs/harness/SPEC.md ] && [ -f docs/harness/progress.md ]; then
  SPEC_SPRINT="$(field_value docs/harness/SPEC.md "Active sprint")"
  PROGRESS_SPRINT="$(field_value docs/harness/progress.md "Active sprint")"
  SPEC_TASK="$(field_value docs/harness/SPEC.md "Active task")"
  PROGRESS_TASK="$(field_value docs/harness/progress.md "Active task")"
  SPEC_PLAN="$(field_value docs/harness/SPEC.md "Active plan")"
  PROGRESS_PLAN="$(field_value docs/harness/progress.md "Active plan")"

  [ -n "$SPEC_SPRINT" ] || fail "SPEC.md missing Active sprint field"
  [ -n "$PROGRESS_SPRINT" ] || fail "progress.md missing Active sprint field"
  [ -n "$SPEC_TASK" ] || fail "SPEC.md missing Active task field"
  [ -n "$PROGRESS_TASK" ] || fail "progress.md missing Active task field"
  [ -n "$SPEC_PLAN" ] || fail "SPEC.md missing Active plan field"
  [ -n "$PROGRESS_PLAN" ] || fail "progress.md missing Active plan field"

  if [ -n "$SPEC_SPRINT" ] && [ -n "$PROGRESS_SPRINT" ] && [ "$SPEC_SPRINT" != "$PROGRESS_SPRINT" ]; then
    fail "Active sprint drift: SPEC='$SPEC_SPRINT' progress='$PROGRESS_SPRINT'"
  fi
  if [ -n "$SPEC_TASK" ] && [ -n "$PROGRESS_TASK" ] && [ "$SPEC_TASK" != "$PROGRESS_TASK" ]; then
    fail "Active task drift: SPEC='$SPEC_TASK' progress='$PROGRESS_TASK'"
  fi
  if [ -n "$SPEC_PLAN" ] && [ -n "$PROGRESS_PLAN" ] && [ "$SPEC_PLAN" != "$PROGRESS_PLAN" ]; then
    fail "Active plan drift: SPEC='$SPEC_PLAN' progress='$PROGRESS_PLAN'"
  fi
fi

# Active plan file must exist
ACTIVE_PLAN="$(field_value docs/harness/progress.md "Active plan")"
if [ -z "$ACTIVE_PLAN" ]; then
  ACTIVE_PLAN="$(field_value docs/harness/SPEC.md "Active plan")"
fi
if [ -n "$ACTIVE_PLAN" ]; then
  ACTIVE_LOG="$(normalize_harness_path "$ACTIVE_PLAN")"
  if [ ! -f "$ACTIVE_LOG" ]; then
    fail "active plan log missing: $ACTIVE_LOG"
  fi
fi

# Latest review for active task should have a verdict marker if it exists.
ACTIVE_TASK="$(field_value docs/harness/progress.md "Active task")"
ACTIVE_TASK_ID="$(task_id_from_value "$ACTIVE_TASK")"
ACTIVE_REVIEW="$(review_for_task "$ACTIVE_TASK_ID")"
if [ -n "$ACTIVE_REVIEW" ]; then
  if grep -qE '^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL)[[:space:]].+$' "$ACTIVE_REVIEW"; then
    :
  elif grep -qE '^(PASS|FAIL)([[:space:]:.,;-]|$)' "$ACTIVE_REVIEW"; then
    warn "active task review artifact is missing REVIEW_VERDICT marker: $ACTIVE_REVIEW (expected from review-gate post hard gate)"
  else
    fail "active task review artifact has no PASS/FAIL verdict: $ACTIVE_REVIEW"
  fi
elif [ -n "$ACTIVE_TASK_ID" ]; then
  warn "no review artifact found for active task: $ACTIVE_TASK_ID (expected after first post-gate)"
fi

# .sensors-last format (when present)
if [ -f docs/harness/.sensors-last ]; then
  if ! grep -qE '^status=(pass|pass_with_exclusion|fail)$' docs/harness/.sensors-last; then
    fail ".sensors-last is not parseable (expected status=...)"
  fi
else
  warn ".sensors-last missing (run sensors.sh at least once)"
fi

# Bootstrap contract: runs and produces limited output
if BOOTSTRAP_OUTPUT="$(bash docs/harness/bin/bootstrap.sh 2>/dev/null)"; then
  BOOTSTRAP_LINES="$(printf '%s\n' "$BOOTSTRAP_OUTPUT" | wc -l | tr -d ' ')"
else
  fail "bootstrap.sh failed to execute cleanly"
  BOOTSTRAP_LINES=999
fi
if [ "$BOOTSTRAP_LINES" -gt 60 ]; then
  fail "bootstrap output too long: ${BOOTSTRAP_LINES} lines (contract <= ~55)"
fi

# If sensors run reports an exclusion in CI status, confirm the known-issue is registered.
SENSORS_CONTEXT_SENSOR="${SENSORS_CONTEXT_SENSOR:-}"
SENSORS_CONTEXT_KNOWN_ISSUE="${SENSORS_CONTEXT_KNOWN_ISSUE:-}"
SENSORS_CONTEXT_CI_STATUS="${SENSORS_CONTEXT_CI_STATUS:-}"

CI_EXCLUSION_STATUS="pass"
KNOWN_ISSUE=""
if [ -n "$SENSORS_CONTEXT_CI_STATUS" ]; then
  CI_EXCLUSION_STATUS="$SENSORS_CONTEXT_CI_STATUS"
elif [ -f docs/harness/.sensors-last ]; then
  CI_EXCLUSION_STATUS="$(field_from_file docs/harness/.sensors-last ci_status)"
  [ -n "$CI_EXCLUSION_STATUS" ] || CI_EXCLUSION_STATUS="$(field_from_file docs/harness/.sensors-last status)"
fi

if [ "$CI_EXCLUSION_STATUS" = "pass_with_exclusion" ]; then
  if [ -n "$SENSORS_CONTEXT_KNOWN_ISSUE" ]; then
    KNOWN_ISSUE="$SENSORS_CONTEXT_KNOWN_ISSUE"
  elif [ -f docs/harness/.sensors-last ]; then
    KNOWN_ISSUE="$(field_from_file docs/harness/.sensors-last excluded_known_issue || true)"
    if [ -z "$KNOWN_ISSUE" ]; then
      KNOWN_ISSUE="$(sed -n 's/.*known_issue=\(.*\) reason=.*/\1/p' docs/harness/.sensors-last | head -n 1 || true)"
    fi
    if [ -z "$KNOWN_ISSUE" ]; then
      KNOWN_ISSUE="$(sed -n 's/.*known_issue=\([^ ]*\).*/\1/p' docs/harness/.sensors-last | head -n 1 || true)"
    fi
  fi

  if [ -z "$KNOWN_ISSUE" ]; then
    fail "sensors context indicates pass_with_exclusion but no known_issue path was provided"
  elif ! grep -qF "$KNOWN_ISSUE" docs/harness/progress.md; then
    fail ".sensors-last / context indicates exclusion not mentioned in progress.md: $KNOWN_ISSUE"
  fi
fi

# Summary
if [ "${#FAILURES[@]}" -gt 0 ]; then
  echo "FAIL harness doctor found ${#FAILURES[@]} issue(s)"
  for item in "${FAILURES[@]}"; do
    echo "- $item"
  done
  exit 1
fi

echo "OK harness doctor"
if [ "${#WARNINGS[@]}" -gt 0 ]; then
  for item in "${WARNINGS[@]}"; do
    echo "WARN: $item"
  done
fi

echo "Checked: required docs + executables, negative-scope/canvas/baseline/audit wiring, cross-references to policy/doctor, SPEC<->progress drift, active plan existence, review verdict presence, .sensors-last format, bootstrap contract, and exclusion records."
