#!/usr/bin/env bash
# docs/harness/bin/sensors.sh
#
# Deterministic local gate for engram harness.
# Primary delegation: `just ci` (preferred) ou `make ci` (fallback), ou scripts/ci.sh.
# Também executa harness doctor.
#
# Suporte a exclusão documentada para falhas externas temporárias.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: cannot resolve repo root from script location" >&2
  exit 2
fi
cd "$REPO_ROOT"

BIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$BIN_DIR/lib.sh"

cleanup_tmp() {
  if [ -n "${CI_OUTPUT:-}" ] && [ -f "$CI_OUTPUT" ]; then
    rm -f "$CI_OUTPUT"
  fi
}

on_exit() {
  local rc=$?
  cleanup_tmp
  if [ "$rc" -ne 0 ]; then
    echo "FAIL: sensors.sh aborted" >&2
  fi
}

trap on_exit EXIT

usage() {
  cat >&2 <<'USAGE'
Usage: docs/harness/bin/sensors.sh [--exclude-sensor <name> --known-issue docs/harness/known-issues/YYYY-MM-DD-slug.md --reason "short reason"]

Default: clean run of `just ci` (or `make ci` fallback) + harness doctor.
Exclusion mode is reserved for documented external-dependency outages (ex.: API embedding, watcher GUI, socket/grpc transport)
and must be pre-registered in progress.md + known-issue file.
USAGE
}

EXCLUDE_SENSOR=""
KNOWN_ISSUE=""
EXCLUSION_REASON=""
CI_OUTPUT=""

normalize_plan_path() {
  local path="$1"
  case "$path" in
    docs/harness/*) printf '%s' "$path" ;;
    ./progress/*) printf 'docs/harness/%s' "${path#./}" ;;
    progress/*) printf 'docs/harness/%s' "$path" ;;
    *) printf '%s' "$path" ;;
  esac
}

is_expected_excluded_failure() {
  local sensor="$1"
  local test_pattern=""
  local error_pattern=""

  case "$sensor" in
    embedding-api-smoke)
      test_pattern='embedding|ext.*embedding|cohere|voyage|openai|ollama|tests/embedding|embedding_test'
      error_pattern='connection refused|connection reset|timeout|5[0-9]{2}|service unavailable|operation timed out|network|temporary failure'
      ;;
    watcher-gui)
      test_pattern='watcher|app_focus|file_watcher|fs_watcher|watcher_integration'
      error_pattern='gui|display|headless|wayland|x11|os error|permission denied|not supported'
      ;;
    external-embedding)
      test_pattern='embedding|external|provider|ext.*embedding|tests/external'
      error_pattern='provider|api|timeout|connection|5[0-9]{2}|unavailable|transport|rate limit|temporary failure'
      ;;
    grpc-transport)
      test_pattern='tests/grpc_transport\.rs|grpc_transport|scenario_[a-z_]+'
      error_pattern='operation not permitted|permission denied|bind|socket|endpoint'
      ;;
    *)
      return 1
      ;;
  esac

  if ! grep -qiE "$test_pattern" "$CI_OUTPUT"; then
    return 1
  fi
  if ! grep -qiE "$error_pattern" "$CI_OUTPUT"; then
    return 1
  fi

  local failed_tests=""
  local line

  if grep -Eq '^[[:space:]]*test .* \.\.\. FAILED$' "$CI_OUTPUT" >/dev/null; then
    failed_tests="$(grep -E '^[[:space:]]*test .* \.\.\. FAILED$' "$CI_OUTPUT" | sed -E 's/^[[:space:]]*test //; s/[[:space:]]+\.\.\. FAILED$//')"
  fi
  if grep -Eq "^thread '.*' panicked at" "$CI_OUTPUT" >/dev/null; then
    failed_tests="${failed_tests}
$(grep -E "^thread '.*' panicked at" "$CI_OUTPUT" | sed -E "s/^thread '([^']+)'.*$/\\1/")"
  fi

  local failed_count=0
  if [ -n "$failed_tests" ]; then
    while IFS= read -r line; do
      [ -z "$line" ] && continue
      failed_count=$((failed_count + 1))
      if ! echo "$line" | grep -qiE "$test_pattern"; then
        return 1
      fi
    done <<< "$failed_tests"
  else
    # If no explicit per-test failure markers are found, reject as unknown/unrelated failure.
    return 1
  fi

  if [ "$failed_count" -eq 0 ]; then
    return 1
  fi

  return 0
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --exclude-sensor)
      EXCLUDE_SENSOR="${2:-}"
      shift 2
      ;;
    --known-issue)
      KNOWN_ISSUE="${2:-}"
      shift 2
      ;;
    --reason)
      EXCLUSION_REASON="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

# Validate exclusion contract (very restrictive for v0)
if [ -n "$EXCLUDE_SENSOR" ] || [ -n "$KNOWN_ISSUE" ] || [ -n "$EXCLUSION_REASON" ]; then
  if [ -z "$EXCLUDE_SENSOR" ]; then
    echo "ERROR: exclusion requires --exclude-sensor" >&2
    exit 2
  fi
  case "$EXCLUDE_SENSOR" in
    embedding-api-smoke|watcher-gui|external-embedding|grpc-transport)
      ;;
    *)
      echo "ERROR: only specific external sensors may be excluded in v0 (see GATES.md). Got: $EXCLUDE_SENSOR" >&2
      exit 2
      ;;
  esac
  if [ -z "$KNOWN_ISSUE" ] || [ ! -f "$KNOWN_ISSUE" ]; then
    echo "ERROR: exclusion requires an existing --known-issue file under docs/harness/known-issues/" >&2
    exit 2
  fi
  case "$KNOWN_ISSUE" in
    docs/harness/known-issues/[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]-*.md) ;;
    *)
      echo "ERROR: --known-issue must use docs/harness/known-issues/YYYY-MM-DD-<slug>.md format" >&2
      exit 2
      ;;
  esac
  if [ -z "$EXCLUSION_REASON" ]; then
    echo "ERROR: exclusion requires --reason" >&2
    exit 2
  fi

  if [ ! -f docs/harness/progress.md ]; then
    echo "ERROR: docs/harness/progress.md missing; pre-registration cannot be verified." >&2
    exit 1
  fi

  ACTIVE_PLAN="$(field_value docs/harness/progress.md "Active plan")"
  if [ -z "$ACTIVE_PLAN" ]; then
    echo "ERROR: Active plan missing in docs/harness/progress.md; exclusion needs active-plan registration." >&2
    exit 1
  fi
  PLAN_FILE="$(normalize_plan_path "$ACTIVE_PLAN")"
  if [ ! -f "$PLAN_FILE" ]; then
    echo "ERROR: active plan file not found: $PLAN_FILE" >&2
    exit 1
  fi
  if ! grep -qF "$KNOWN_ISSUE" "$PLAN_FILE" && ! grep -qF "$KNOWN_ISSUE" docs/harness/progress.md; then
    echo "ERROR: known-issue not mentioned in active plan or progress.md. Record it first." >&2
    exit 1
  fi
fi

echo "==> [harness] sensors.sh starting at $(date -u +%Y-%m-%dT%H:%M:%SZ)"

CI_STATUS="pass"
DOCTOR_STATUS="pass"
EXCLUSION_NOTE=""

if [ -n "$EXCLUDE_SENSOR" ]; then
  echo "==> [harness] running with exclusion: $EXCLUDE_SENSOR (reason: $EXCLUSION_REASON)"
  EXCLUSION_NOTE="excluded=$EXCLUDE_SENSOR known_issue=$KNOWN_ISSUE reason=\"$EXCLUSION_REASON\""
fi

# Core delegation: the existing just ci contract (fmt + clippy -D + test parity + docs + MCP ref)
CI_COMMAND=()
if command -v just >/dev/null 2>&1; then
  CI_COMMAND=(just ci)
elif command -v make >/dev/null 2>&1; then
  CI_COMMAND=(make ci)
elif [ -f scripts/ci.sh ] && [ -x scripts/ci.sh ]; then
  CI_COMMAND=(bash scripts/ci.sh)
fi

if [ "${#CI_COMMAND[@]}" -eq 0 ]; then
  echo "FAIL: no local CI command available (need just ci, make ci, or scripts/ci.sh)"
  CI_STATUS="fail"
else
  echo "==> [harness] running CI command: ${CI_COMMAND[*]}"
  CI_OUTPUT="$(mktemp)"
  if "${CI_COMMAND[@]}" >"$CI_OUTPUT" 2>&1; then
    CI_STATUS="pass"
  else
    CI_EXIT=$?
    echo "FAIL: ${CI_COMMAND[*]} failed (exit=$CI_EXIT)"
    cat "$CI_OUTPUT"
    if [ -n "$EXCLUDE_SENSOR" ] && is_expected_excluded_failure "$EXCLUDE_SENSOR"; then
      CI_STATUS="pass_with_exclusion"
      echo "    mapped failure to pass_with_exclusion via documented exclusion"
    else
      CI_STATUS="fail"
    fi
  fi
fi

# Harness doctor (self-consistency of the harness itself)
if ! SENSORS_CONTEXT_CI_STATUS="$CI_STATUS" \
  SENSORS_CONTEXT_SENSOR="$EXCLUDE_SENSOR" \
  SENSORS_CONTEXT_KNOWN_ISSUE="$KNOWN_ISSUE" \
  SENSORS_CONTEXT_EXCLUSION_REASON="$EXCLUSION_REASON" \
  bash docs/harness/bin/doctor.sh; then
  echo "FAIL: harness doctor failed"
  DOCTOR_STATUS="fail"
fi

STATUS="pass"
if [ "$CI_STATUS" = "pass_with_exclusion" ] && [ "$DOCTOR_STATUS" = "pass" ]; then
  STATUS="pass_with_exclusion"
elif [ "$CI_STATUS" != "pass" ] || [ "$DOCTOR_STATUS" != "pass" ]; then
  STATUS="fail"
fi

# Record result (machine parseable for bootstrap / doctor)
{
  echo "status=$STATUS"
  echo "ci_status=$CI_STATUS"
  echo "doctor_status=$DOCTOR_STATUS"
  echo "timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  [ -n "$EXCLUSION_NOTE" ] && echo "$EXCLUSION_NOTE"
  [ -n "$EXCLUDE_SENSOR" ] && echo "excluded_sensor=$EXCLUDE_SENSOR"
  [ -n "$KNOWN_ISSUE" ] && echo "excluded_known_issue=$KNOWN_ISSUE"
  [ -n "$EXCLUSION_REASON" ] && echo "excluded_reason=$EXCLUSION_REASON"
  echo "ci=${CI_COMMAND[*]-missing} + harness doctor"
} > docs/harness/.sensors-last

echo
if [ "$STATUS" = "pass" ]; then
  echo "PASS (all deterministic gates green)"
  exit 0
elif [ "$STATUS" = "pass_with_exclusion" ]; then
  echo "PASS_WITH_EXCLUSION ($EXCLUSION_NOTE)"
  echo "This is NOT sufficient evidence for production closure without a clean run."
  exit 0
else
  echo "FAIL"
  if [ "$CI_STATUS" = "pass_with_exclusion" ]; then
    echo "CI was mapped as pass_with_exclusion, but another gate failed."
    echo "ci_status=$CI_STATUS doctor_status=$DOCTOR_STATUS"
  else
    echo "ci_status=$CI_STATUS doctor_status=$DOCTOR_STATUS"
  fi
  exit 1
fi
