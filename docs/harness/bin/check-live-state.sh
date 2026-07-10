#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: bash docs/harness/bin/check-live-state.sh --progress PROGRESS_PATH
EOF
}

PROGRESS_PATH=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --progress)
      if [ "$#" -lt 2 ] || [ -z "${2:-}" ]; then
        echo "ERROR --progress requires PROGRESS_PATH" >&2
        usage >&2
        exit 2
      fi
      PROGRESS_PATH="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$PROGRESS_PATH" ]; then
  echo "ERROR --progress is required" >&2
  usage >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

if [ ! -f "$PROGRESS_PATH" ]; then
  echo "FAIL live state check failed"
  echo "missing progress file: $PROGRESS_PATH"
  echo "remediation: pass an existing docs/harness/progress.md-compatible file with --progress"
  exit 1
fi

source docs/harness/bin/lib.sh

FAILURES=()

add_failure() {
  FAILURES+=("$1")
}

require_progress_text() {
  local needle="$1"
  local label="$2"
  if ! grep -Fq "$needle" "$PROGRESS_PATH"; then
    add_failure "missing progress reconciliation: $label"
  fi
}

require_file_text() {
  local path="$1"
  local needle="$2"
  local label="$3"
  if ! grep -Fq "$needle" "$path"; then
    add_failure "workflow drift: $label not found in $path"
  fi
}

read_sensor_field() {
  local key="$1"
  awk -F= -v key="$key" '$1 == key { print $2; exit }' docs/harness/.sensors-last 2>/dev/null || true
}

HEAD_FULL="$(git rev-parse HEAD)"
HEAD_SHORT="$(git rev-parse --short HEAD)"
PARENT_FULL="$(git rev-parse HEAD^ 2>/dev/null || true)"
PARENT_SHORT="$(git rev-parse --short HEAD^ 2>/dev/null || true)"
if git status --porcelain | grep -q .; then
  WORKTREE_STATUS="dirty"
else
  WORKTREE_STATUS="clean"
fi

LAST_COMMIT="$(field_value "$PROGRESS_PATH" "Last commit")"
LAST_REVIEW="$(field_value "$PROGRESS_PATH" "Last review")"
LAST_SENSORS="$(field_value "$PROGRESS_PATH" "Last sensors")"
LAST_CHECK="$(field_value "$PROGRESS_PATH" "Last live-state check")"
ACTIVE_PLAN="$(field_value "$PROGRESS_PATH" "Active plan")"

for required_field in "Last commit" "Last review" "Last sensors" "Last live-state check" "Active plan"; do
  case "$required_field" in
    "Last commit") required_value="$LAST_COMMIT" ;;
    "Last review") required_value="$LAST_REVIEW" ;;
    "Last sensors") required_value="$LAST_SENSORS" ;;
    "Last live-state check") required_value="$LAST_CHECK" ;;
    "Active plan") required_value="$ACTIVE_PLAN" ;;
  esac
  if [ -z "$required_value" ]; then
    add_failure "missing required field: $required_field"
  fi
done

if [ -n "$LAST_COMMIT" ] \
  && [ "$LAST_COMMIT" != "$HEAD_FULL" ] \
  && [ "$LAST_COMMIT" != "$HEAD_SHORT" ] \
  && [ "$LAST_COMMIT" != "$PARENT_FULL" ] \
  && [ "$LAST_COMMIT" != "$PARENT_SHORT" ]; then
  add_failure "stale Last commit: found $LAST_COMMIT, expected $HEAD_SHORT or first parent $PARENT_SHORT"
fi

if [ -n "$ACTIVE_PLAN" ] && [ ! -f "$ACTIVE_PLAN" ]; then
  add_failure "active plan does not exist: $ACTIVE_PLAN"
fi

if [ -n "$LAST_REVIEW" ]; then
  REVIEW_PATH="$(printf '%s\n' "$LAST_REVIEW" | grep -Eo 'docs/harness/reviews/[^` ]+\.md' | head -1 || true)"
  if [ -z "$REVIEW_PATH" ]; then
    add_failure "Last review does not name a docs/harness/reviews/*.md artifact"
  elif [ ! -f "$REVIEW_PATH" ]; then
    add_failure "Last review artifact does not exist: $REVIEW_PATH"
  else
    if ! grep -Eq '^REVIEW_VERDICT:[[:space:]]*PASS[[:space:]]' "$REVIEW_PATH"; then
      add_failure "Last review artifact lacks REVIEW_VERDICT: PASS: $REVIEW_PATH"
    fi
    if ! grep -Fq "engram-10-of-10-live-state" <<<"$REVIEW_PATH"; then
      add_failure "Last review artifact is stale for this task: $REVIEW_PATH"
    fi
  fi
fi

if [ ! -f docs/harness/.sensors-last ]; then
  add_failure "missing docs/harness/.sensors-last"
else
  SENSOR_STATUS="$(read_sensor_field status)"
  SENSOR_MODE="$(read_sensor_field mode)"
  SENSOR_TIMESTAMP="$(read_sensor_field timestamp)"

  if [ -z "$SENSOR_STATUS" ] || [ -z "$SENSOR_MODE" ] || [ -z "$SENSOR_TIMESTAMP" ]; then
    add_failure ".sensors-last missing status, mode, or timestamp"
  fi
  if [ -n "$LAST_SENSORS" ] && [ -n "$SENSOR_STATUS" ] && ! grep -Fq "status=$SENSOR_STATUS" <<<"$LAST_SENSORS"; then
    add_failure "Last sensors does not record status=$SENSOR_STATUS"
  fi
  if [ -n "$LAST_SENSORS" ] && [ -n "$SENSOR_MODE" ] && ! grep -Fq "$SENSOR_MODE" <<<"$LAST_SENSORS"; then
    add_failure "Last sensors does not record mode=$SENSOR_MODE"
  fi
  if [ -n "$LAST_SENSORS" ] && [ -n "$SENSOR_TIMESTAMP" ] && ! grep -Fq "$SENSOR_TIMESTAMP" <<<"$LAST_SENSORS"; then
    add_failure "Last sensors does not record timestamp $SENSOR_TIMESTAMP"
  fi
fi

if [ -n "$LAST_CHECK" ] && ! grep -Fq "check-live-state.sh --progress docs/harness/progress.md" <<<"$LAST_CHECK"; then
  add_failure "Last live-state check does not name the checker CLI invocation"
fi

require_file_text .github/workflows/ci.yml "name: Format" "required Format check"
require_file_text .github/workflows/ci.yml "name: Clippy" "required Clippy check"
require_file_text .github/workflows/ci.yml "name: Test (ubuntu-latest)" "required Test (ubuntu-latest) check"
require_file_text .github/workflows/ci.yml "name: Documentation" "required Documentation check"
require_file_text .github/workflows/harness-contract.yml "name: Harness Contract" "required Harness Contract check"
require_file_text .github/workflows/harness-contract.yml "name: Harness Doctor Advisory" "advisory Harness Doctor check"
require_file_text .github/workflows/ci.yml "name: Security Audit" "advisory Security Audit check"
require_file_text .github/workflows/ci.yml "name: Cargo Deny" "advisory Cargo Deny check"

require_progress_text "| \`Format\` | branch-required | \`.github/workflows/ci.yml\` |" "Format branch-required row"
require_progress_text "| \`Clippy\` | branch-required | \`.github/workflows/ci.yml\` |" "Clippy branch-required row"
require_progress_text "| \`Test (ubuntu-latest)\` | branch-required | \`.github/workflows/ci.yml\` |" "Test branch-required row"
require_progress_text "| \`Documentation\` | branch-required | \`.github/workflows/ci.yml\` |" "Documentation branch-required row"
require_progress_text "| \`Security Audit\` | branch-required | \`.github/workflows/ci.yml\` |" "Security Audit branch-required row"
require_progress_text "| \`Cargo Deny\` | branch-required | \`.github/workflows/ci.yml\` |" "Cargo Deny branch-required row"
require_progress_text "| \`Harness Contract\` | not in \`required_status_checks.contexts\` | \`.github/workflows/harness-contract.yml\` |" "Harness Contract non-inferred row"
require_progress_text "| \`Harness Doctor Advisory\` | advisory workflow job | \`.github/workflows/harness-contract.yml\` |" "Harness Doctor advisory row"

echo "head=$HEAD_SHORT"
echo "worktree_status=$WORKTREE_STATUS"

if [ "${#FAILURES[@]}" -gt 0 ]; then
  echo "FAIL live state check failed"
  for failure in "${FAILURES[@]}"; do
    echo "$failure"
  done
  echo "remediation: update Last commit in $PROGRESS_PATH to $HEAD_SHORT after running rtk git rev-parse HEAD"
  echo "remediation: update Last sensors in $PROGRESS_PATH from docs/harness/.sensors-last after running rtk bash docs/harness/bin/sensors.sh quick or full"
  echo "remediation: restore the progress live-state field table and required/advisory workflow reconciliation rows"
  exit 1
fi

echo "PASS live state matches current repository facts"
