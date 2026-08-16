#!/usr/bin/env bash
# docs/harness/bin/sensors.sh
#
# Deterministic local gate for engram harness.
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
# shellcheck source=docs/harness/bin/lib.sh
source "$BIN_DIR/lib.sh"

SUPPRESS_ABORT_MESSAGE=0

# shellcheck disable=SC2329  # invoked indirectly via on_exit/trap
cleanup_tmp() {
  if [ -n "${CI_OUTPUT:-}" ] && [ -f "$CI_OUTPUT" ]; then
    rm -f "$CI_OUTPUT"
  fi
}

# shellcheck disable=SC2329  # invoked via trap EXIT below
on_exit() {
  local rc=$?
  cleanup_tmp
  if [ "$rc" -ne 0 ] && [ "${SUPPRESS_ABORT_MESSAGE:-0}" -ne 1 ]; then
    echo "FAIL: sensors.sh aborted" >&2
  fi
}

trap on_exit EXIT

usage() {
  cat >&2 <<'USAGE'
Usage:
  docs/harness/bin/sensors.sh [full]
  docs/harness/bin/sensors.sh quick
  docs/harness/bin/sensors.sh docs
  docs/harness/bin/sensors.sh mcp
  docs/harness/bin/sensors.sh baseline
  docs/harness/bin/sensors.sh status --json
  docs/harness/bin/sensors.sh [--exclude-sensor <name> --known-issue docs/harness/known-issues/YYYY-MM-DD-slug.md --reason "short reason"]

Default/full: clean granular run of fmt -> clippy -> test (lib/integration) -> wasm -> doc -> MCP reference
check, then PR title policy and harness doctor.
Optional lanes are developer aids and do not replace the full gate.
Status JSON is a read-only snapshot of docs/harness/.sensors-last and does not run the gate.
Exclusion mode is reserved for documented external-dependency outages (ex.: API embedding, watcher GUI, socket/grpc transport)
and must be pre-registered in progress.md + known-issue file.
USAGE
}

MODE="full"
JSON_MODE=0
EXCLUDE_SENSOR=""
KNOWN_ISSUE=""
EXCLUSION_REASON=""
CI_OUTPUT=""
SENSORS_LAST_PATH="docs/harness/.sensors-last"
SENSORS_LOG_PATH="docs/harness/.sensors-log"
SENSORS_LOG_MAX_BYTES="${SENSORS_LOG_MAX_BYTES:-1048576}"
SENSORS_LOG_ROTATIONS="${SENSORS_LOG_ROTATIONS:-5}"
SENSORS_STARTED_AT="$SECONDS"
CI_STEP_NAMES=()
CI_STEP_STATUSES=()
CI_STEP_ORDER=()
CI_REQUIRED_FEATURES_VALUE=""
CI_FEATURES_SOURCE="default"

set_ci_step_status() {
  local step="$1"
  local status="$2"
  local idx=""
  idx=0
  while [ "$idx" -lt "${#CI_STEP_NAMES[@]}" ]; do
    if [ "${CI_STEP_NAMES[$idx]}" = "$step" ]; then
      CI_STEP_STATUSES[idx]="$status"
      return 0
    fi
    idx=$((idx + 1))
  done
  CI_STEP_NAMES+=("$step")
  CI_STEP_STATUSES+=("$status")
}

get_ci_step_status() {
  local step="$1"
  local idx=""
  idx=0
  while [ "$idx" -lt "${#CI_STEP_NAMES[@]}" ]; do
    if [ "${CI_STEP_NAMES[$idx]}" = "$step" ]; then
      printf '%s' "${CI_STEP_STATUSES[$idx]}"
      return 0
    fi
    idx=$((idx + 1))
  done
  printf '%s' "not_run"
}

resolve_ci_required_features() {
  if [ -n "${CI_REQUIRED_FEATURES:-}" ]; then
    CI_REQUIRED_FEATURES_VALUE="$CI_REQUIRED_FEATURES"
    CI_FEATURES_SOURCE="env"
    return 0
  fi
  if [ -f scripts/ci-required-features.env ]; then
    # shellcheck source=scripts/ci-required-features.env
    source scripts/ci-required-features.env
    if [ -n "${CI_REQUIRED_FEATURES:-}" ]; then
      CI_REQUIRED_FEATURES_VALUE="$CI_REQUIRED_FEATURES"
      CI_FEATURES_SOURCE="scripts/ci-required-features.env"
      return 0
    fi
  fi
  CI_REQUIRED_FEATURES_VALUE=""
  CI_FEATURES_SOURCE="fallback-empty"
}

ci_step_status_json() {
  local step
  local status
  local output="{"

  for step in ${CI_STEP_ORDER[@]+"${CI_STEP_ORDER[@]}"}; do
    status="$(get_ci_step_status "$step")"
    output="${output}\"${step}\":\"$(json_escape "$status")\","
  done
  if [ "$output" = "{" ]; then
    echo "{}"
    return
  fi
  echo "${output%,}}"
}

emit_status_json() {
  if ! command -v python3 >/dev/null 2>&1; then
    echo "ERROR: python3 is required for sensors.sh status --json" >&2
    exit 2
  fi

  python3 -c 'import json
import sys
from datetime import datetime, timezone
from pathlib import Path

repo_root = Path(sys.argv[1])
snapshot_path = repo_root / "docs/harness/.sensors-last"
repo_snapshot_path = "docs/harness/.sensors-last"
log_path = repo_root / "docs/harness/.sensors-log"
repo_log_path = "docs/harness/.sensors-log"

fields = {}
warnings = []
failures = []
checks = []
artifacts = []


def add_check(check_id, status, message, path=None):
    item = {"id": check_id, "status": status, "message": message}
    if path is not None:
        item["path"] = path
    checks.append(item)


if snapshot_path.exists():
    artifacts.append({
        "path": repo_snapshot_path,
        "kind": "sensors_last",
        "format": "key_value"
    })
    for raw_line in snapshot_path.read_text(encoding="utf-8").splitlines():
        if "=" not in raw_line:
            continue
        key, value = raw_line.split("=", 1)
        fields[key] = value
    add_check("sensors_last:file", "pass", ".sensors-last exists", repo_snapshot_path)
else:
    warnings.append(".sensors-last missing; run sensors.sh at least once")
    add_check("sensors_last:file", "warn", ".sensors-last missing", repo_snapshot_path)

if log_path.exists():
    artifacts.append({
        "path": repo_log_path,
        "kind": "sensors_log",
        "format": "jsonl"
    })
    add_check("sensors_log:file", "pass", ".sensors-log exists", repo_log_path)

last_status = fields.get("status", "")
ci_status = fields.get("ci_status", "")
doctor_status = fields.get("doctor_status", "")
last_mode = fields.get("mode", "")
last_timestamp = fields.get("timestamp", "")

valid_statuses = {"pass", "pass_with_exclusion", "fail"}
if snapshot_path.exists():
    if last_status not in valid_statuses:
        failures.append(".sensors-last has invalid status")
        add_check("sensors_last:status", "fail", ".sensors-last status is invalid", repo_snapshot_path)
    else:
        add_check("sensors_last:status", "pass", ".sensors-last status is parseable", repo_snapshot_path)

    for key, value in {
        "ci_status": ci_status,
        "doctor_status": doctor_status,
        "mode": last_mode,
        "timestamp": last_timestamp
    }.items():
        if value == "":
            warnings.append(f".sensors-last missing {key}")
            add_check(f"sensors_last:{key}", "warn", f".sensors-last missing {key}", repo_snapshot_path)
        else:
            add_check(f"sensors_last:{key}", "pass", f".sensors-last {key} is present", repo_snapshot_path)

if failures:
    common_status = "fail"
    exit_code = 1
elif last_status == "fail" or ci_status == "fail" or doctor_status == "fail":
    common_status = "fail"
    exit_code = 1
elif last_status == "pass_with_exclusion" or ci_status == "pass_with_exclusion":
    common_status = "warn"
    exit_code = 0
elif warnings:
    common_status = "warn"
    exit_code = 0
else:
    common_status = "pass"
    exit_code = 0

summary = "sensors status snapshot"
if last_status:
    summary = f"sensors last status: {last_status}"
elif warnings:
    summary = "sensors status unavailable"

payload = {
    "schema_version": "harness-json-v1",
    "tool": "sensors",
    "mode": "status",
    "status": common_status,
    "exit_code": exit_code,
    "timestamp": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "summary": summary,
    "warnings": warnings,
    "failures": failures,
    "checks": checks,
    "artifacts": artifacts,
    "repo_root": str(repo_root),
    "log_path": repo_log_path if log_path.exists() else "",
    "last_mode": last_mode,
    "last_timestamp": last_timestamp,
    "ci_status": ci_status,
    "doctor_status": doctor_status,
    "known_issue": fields.get("excluded_known_issue", ""),
    "excluded_sensor": fields.get("excluded_sensor", ""),
    "exclusion_reason": fields.get("excluded_reason", "")
}

print(json.dumps(payload, ensure_ascii=False, separators=(",", ":")))
sys.exit(exit_code)' "$REPO_ROOT"
}

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  value="${value//$'\r'/\\r}"
  value="${value//$'\t'/\\t}"
  printf '%s' "$value"
}

sensors_duration_sec() {
  printf '%s' "$((SECONDS - SENSORS_STARTED_AT))"
}

rotate_sensors_log() {
  local max_bytes="$SENSORS_LOG_MAX_BYTES"
  local rotations="$SENSORS_LOG_ROTATIONS"
  local size=""
  local i=""

  case "$max_bytes" in
    ''|*[!0-9]*) max_bytes=1048576 ;;
  esac
  case "$rotations" in
    ''|*[!0-9]*) rotations=5 ;;
  esac
  if [ "$rotations" -lt 1 ]; then
    rotations=1
  fi
  if [ ! -f "$SENSORS_LOG_PATH" ]; then
    return 0
  fi

  size="$(wc -c < "$SENSORS_LOG_PATH" | tr -d ' ')"
  if [ "${size:-0}" -lt "$max_bytes" ]; then
    return 0
  fi

  i=$((rotations - 1))
  while [ "$i" -ge 1 ]; do
    if [ -f "${SENSORS_LOG_PATH}.${i}" ]; then
      mv "${SENSORS_LOG_PATH}.${i}" "${SENSORS_LOG_PATH}.$((i + 1))"
    fi
    i=$((i - 1))
  done
  mv "$SENSORS_LOG_PATH" "${SENSORS_LOG_PATH}.1"
  rm -f "${SENSORS_LOG_PATH}.$((rotations + 1))"
}

append_sensors_log() {
  local status="$1"
  local ci_status="$2"
  local doctor_status="$3"
  local mode="$4"
  local ci_label="$5"
  local timestamp="$6"
  local duration_sec="$7"
  local exclusion_json="null"

  rotate_sensors_log

  if [ -n "$EXCLUDE_SENSOR" ] || [ -n "$KNOWN_ISSUE" ] || [ -n "$EXCLUSION_REASON" ]; then
    exclusion_json="{\"sensor\":\"$(json_escape "$EXCLUDE_SENSOR")\",\"known_issue\":\"$(json_escape "$KNOWN_ISSUE")\",\"reason\":\"$(json_escape "$EXCLUSION_REASON")\"}"
  fi

  printf '{"schema_version":"sensors-log-v1","timestamp":"%s","tool":"sensors","mode":"%s","status":"%s","duration_sec":%s,"ci_status":"%s","doctor_status":"%s","ci_command":"%s","ci_steps":%s,"exclusion":%s,"artifacts":[{"path":"%s","kind":"sensors_last","format":"key_value"}]}\n' \
    "$(json_escape "$timestamp")" \
    "$(json_escape "$mode")" \
    "$(json_escape "$status")" \
    "$duration_sec" \
    "$(json_escape "$ci_status")" \
    "$(json_escape "$doctor_status")" \
    "$(json_escape "$ci_label")" \
    "$(ci_step_status_json)" \
    "$exclusion_json" \
    "$SENSORS_LAST_PATH" >> "$SENSORS_LOG_PATH"
}

write_sensors_last() {
  local status="$1"
  local ci_status="$2"
  local doctor_status="$3"
  local mode="$4"
  local ci_label="$5"
  local timestamp=""
  local duration_sec=""

  timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  duration_sec="$(sensors_duration_sec)"

  {
    echo "status=$status"
    echo "ci_status=$ci_status"
    echo "doctor_status=$doctor_status"
    echo "mode=$mode"
    echo "timestamp=$timestamp"
    echo "duration_sec=$duration_sec"
    [ -n "${EXCLUSION_NOTE:-}" ] && echo "$EXCLUSION_NOTE"
    [ -n "$EXCLUDE_SENSOR" ] && echo "excluded_sensor=$EXCLUDE_SENSOR"
    [ -n "$KNOWN_ISSUE" ] && echo "excluded_known_issue=$KNOWN_ISSUE"
    [ -n "$EXCLUSION_REASON" ] && echo "excluded_reason=$EXCLUSION_REASON"
    echo "ci=$ci_label + harness doctor"
  } > "$SENSORS_LAST_PATH"

  append_sensors_log "$status" "$ci_status" "$doctor_status" "$mode" "$ci_label + harness doctor" "$timestamp" "$duration_sec"
}

run_step() {
  local label="$1"
  shift
  echo "==> [harness] $label"
  "$@"
}

run_ci_step() {
  local label="$1"
  local step_key="$2"
  shift 2
  local rc=0
  local step_start_at="$SECONDS"

  CI_STEP_ORDER+=("$step_key")
  echo "==> [harness] $label"
  set_ci_step_status "$step_key" "pass"

  set +e
  "$@" >"$CI_OUTPUT" 2>&1
  rc=$?
  set -e

  if [ "$rc" -ne 0 ]; then
    set_ci_step_status "$step_key" "fail"
  fi

  if [ "$rc" -ne 0 ] && [ -n "$CI_OUTPUT" ]; then
    echo "FAIL: $label command failed after $((SECONDS - step_start_at))s"
    if [ -f "$CI_OUTPUT" ]; then
      cat "$CI_OUTPUT"
    fi
  fi

  return "$rc"
}

ci_feature_args() {
  if [ -z "$CI_REQUIRED_FEATURES_VALUE" ]; then
    echo ""
    return 0
  fi
  echo "--features ${CI_REQUIRED_FEATURES_VALUE}"
}

# shellcheck disable=SC2329  # invoked indirectly through run_ci_step
check_wasm_target_installed() {
  rustup target list --installed | grep -qx "wasm32-unknown-unknown"
}

run_expected_exit() {
  local label="$1"
  local expected="$2"
  shift 2
  local actual

  echo "==> [harness] $label"
  set +e
  "$@"
  actual=$?
  set -e

  if [ "$actual" -ne "$expected" ]; then
    echo "FAIL: $label exited $actual; expected $expected" >&2
    return 1
  fi
}

run_pr_title_policy() {
  run_step "PR title policy accepts clean title" \
    bash docs/harness/bin/pr-title-policy.sh --title "fix: clean title" || return 1
  run_expected_exit "PR title policy rejects [codex] title" 4 \
    bash docs/harness/bin/pr-title-policy.sh --title "[codex] fix: bad title" || return 1
  run_expected_exit "PR title policy rejects spaced mixed-case codex title" 4 \
    bash docs/harness/bin/pr-title-policy.sh --title "[ CoDeX ] fix: bad title" || return 1
}

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
    full|quick|docs|mcp|baseline)
      MODE="$1"
      shift
      ;;
    status)
      MODE="status"
      shift
      ;;
    --json)
      JSON_MODE=1
      shift
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

if [ "$JSON_MODE" -eq 1 ] && [ "$MODE" != "status" ]; then
  echo "ERROR: --json is currently supported only by 'sensors.sh status --json'" >&2
  exit 2
fi

if [ "$MODE" = "status" ]; then
  if [ "$JSON_MODE" -eq 1 ]; then
    SUPPRESS_ABORT_MESSAGE=1
    emit_status_json
    exit $?
  fi
  if [ -f docs/harness/.sensors-last ]; then
    cat docs/harness/.sensors-last
    exit 0
  fi
  echo "WARN: docs/harness/.sensors-last missing (run sensors.sh at least once)" >&2
  exit 1
fi

if [ "$MODE" != "full" ] && { [ -n "$EXCLUDE_SENSOR" ] || [ -n "$KNOWN_ISSUE" ] || [ -n "$EXCLUSION_REASON" ]; }; then
  echo "ERROR: documented exclusions are supported only by the full canonical gate" >&2
  exit 2
fi

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

echo "==> [harness] sensors.sh starting at $(date -u +%Y-%m-%dT%H:%M:%SZ) (mode=$MODE)"
echo "==> [harness] security contract: docs/harness/security/anthropic-reference-harness.md (DEFAULT_MODE=static_read_only)"
echo "==> [harness] tuning files: .claude/scan-extras.txt, .claude/fp-rules.txt"

CI_STATUS="pass"
DOCTOR_STATUS="pass"
EXCLUSION_NOTE=""

if [ -n "$EXCLUDE_SENSOR" ]; then
  echo "==> [harness] running with exclusion: $EXCLUDE_SENSOR (reason: $EXCLUSION_REASON)"
  EXCLUSION_NOTE="excluded=$EXCLUDE_SENSOR known_issue=$KNOWN_ISSUE reason=\"$EXCLUSION_REASON\""
fi

case "$MODE" in
  baseline)
    if run_step "baseline snapshot" bash docs/harness/bin/baseline.sh && run_pr_title_policy && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "baseline"
      echo "PASS (baseline lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "baseline"
    echo "FAIL"
    exit 1
    ;;
  quick)
    if run_step "fmt" cargo fmt --all -- --check && run_step "cargo check" cargo check && run_pr_title_policy && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "cargo fmt + cargo check + pr-title-policy"
      echo "PASS (quick lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "cargo fmt + cargo check + pr-title-policy"
    echo "FAIL"
    exit 1
    ;;
  docs)
    if run_step "MCP reference check" ./scripts/generate-mcp-reference.sh --check && run_step "rustdoc" env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items && run_pr_title_policy && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "mcp reference + rustdoc + pr-title-policy"
      echo "PASS (docs lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "mcp reference + rustdoc + pr-title-policy"
    echo "FAIL"
    exit 1
    ;;
  mcp)
    if run_step "MCP reference check" ./scripts/generate-mcp-reference.sh --check && run_step "MCP protocol tests" cargo test --test mcp_protocol_tests && run_pr_title_policy && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "mcp reference + protocol tests + pr-title-policy"
      echo "PASS (mcp lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "mcp reference + protocol tests + pr-title-policy"
    echo "FAIL"
    exit 1
    ;;
  full)
    ;;
  *)
    echo "ERROR: unknown sensor mode '$MODE'" >&2
    usage
    exit 2
    ;;
esac

resolve_ci_required_features

echo "==> [harness] CI features source: ${CI_FEATURES_SOURCE}; CI_REQUIRED_FEATURES=${CI_REQUIRED_FEATURES_VALUE:-<empty>}"

CI_REQUIRED_FEATURES_ENV=""
if [ -n "$CI_REQUIRED_FEATURES_VALUE" ]; then
  CI_REQUIRED_FEATURES_ENV="CI_REQUIRED_FEATURES=$CI_REQUIRED_FEATURES_VALUE"
fi

if [ -z "$CI_REQUIRED_FEATURES_VALUE" ]; then
  echo "WARN: CI_REQUIRED_FEATURES not available; using cargo defaults"
fi

echo "==> [harness] running granular CI steps"
CI_OUTPUT="$(mktemp)"
CI_STATUS="pass"
CI_COMMAND_LABEL="fmt + clippy + test_lib + test_integration + test_integration_watch + wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check"
CI_FEATURE_ARGS="$(ci_feature_args)"

# shellcheck disable=SC2086  # intentional word splitting: env pairs + feature args
run_ci_step "fmt" "fmt" cargo fmt --all -- --check && \
run_ci_step "clippy" "clippy" env $CI_REQUIRED_FEATURES_ENV cargo clippy --all-targets --no-default-features $CI_FEATURE_ARGS -- -D warnings || CI_STATUS="fail"

if [ "$CI_STATUS" = "pass" ]; then
  # shellcheck disable=SC2086  # intentional word splitting: env pairs + feature args
  run_ci_step "test_lib" "test_lib" \
    env $CI_REQUIRED_FEATURES_ENV \
    cargo test --profile ci --no-default-features $CI_FEATURE_ARGS --lib --tests || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  # shellcheck disable=SC2086  # intentional word splitting: env pairs + feature args
  run_ci_step "test_integration" "test_integration" \
    env $CI_REQUIRED_FEATURES_ENV \
    cargo test --profile ci --no-default-features $CI_FEATURE_ARGS --bin engram-server || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  # shellcheck disable=SC2086  # intentional word splitting: env pairs + feature args
  run_ci_step "test_integration_watch" "test_integration_watch" \
    env $CI_REQUIRED_FEATURES_ENV \
    cargo test --profile ci --no-default-features $CI_FEATURE_ARGS --bin engram-watcher || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  run_ci_step "wasm_target" "wasm_target" check_wasm_target_installed || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  run_ci_step "wasm_all_targets" "wasm_all_targets" cargo check -p engram-wasm --all-targets || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  run_ci_step "wasm_wasm_target" "wasm_wasm_target" cargo check -p engram-wasm --target wasm32-unknown-unknown || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  # shellcheck disable=SC2086  # intentional word splitting: env pairs + feature args
  run_ci_step "doc" "doc" \
    env RUSTDOCFLAGS="-D warnings" $CI_REQUIRED_FEATURES_ENV \
    cargo doc --no-default-features $CI_FEATURE_ARGS --no-deps --document-private-items || CI_STATUS="fail"
fi

if [ "$CI_STATUS" = "pass" ]; then
  run_ci_step "ref_check" "ref_check" ./scripts/generate-mcp-reference.sh --check || CI_STATUS="fail"
fi

if [ "$CI_STATUS" != "pass" ]; then
  if [ -n "$EXCLUDE_SENSOR" ] && is_expected_excluded_failure "$EXCLUDE_SENSOR"; then
    CI_STATUS="pass_with_exclusion"
    echo "    mapped failure to pass_with_exclusion via documented exclusion"
  else
    CI_STATUS="fail"
  fi
fi

rm -f "$CI_OUTPUT"

if ! run_pr_title_policy; then
  CI_STATUS="fail"
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
write_sensors_last "$STATUS" "$CI_STATUS" "$DOCTOR_STATUS" "$MODE" "$CI_COMMAND_LABEL + pr-title-policy"

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
