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

SUPPRESS_ABORT_MESSAGE=0

cleanup_tmp() {
  if [ -n "${CI_OUTPUT:-}" ] && [ -f "$CI_OUTPUT" ]; then
    rm -f "$CI_OUTPUT"
  fi
}

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

Default/full: clean run of `just ci` (or `make ci` fallback) + harness doctor.
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

emit_status_json() {
  if ! command -v python3 >/dev/null 2>&1; then
    echo "ERROR: python3 is required for sensors.sh status --json" >&2
    exit 2
  fi

  python3 - "$REPO_ROOT" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

repo_root = Path(sys.argv[1])
snapshot_path = repo_root / "docs/harness/.sensors-last"
repo_snapshot_path = "docs/harness/.sensors-last"

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
    "last_mode": last_mode,
    "last_timestamp": last_timestamp,
    "ci_status": ci_status,
    "doctor_status": doctor_status,
    "known_issue": fields.get("excluded_known_issue", ""),
    "excluded_sensor": fields.get("excluded_sensor", ""),
    "exclusion_reason": fields.get("excluded_reason", "")
}

print(json.dumps(payload, ensure_ascii=False, separators=(",", ":")))
sys.exit(exit_code)
PY
}

write_sensors_last() {
  local status="$1"
  local ci_status="$2"
  local doctor_status="$3"
  local mode="$4"
  local ci_label="$5"

  {
    echo "status=$status"
    echo "ci_status=$ci_status"
    echo "doctor_status=$doctor_status"
    echo "mode=$mode"
    echo "timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    [ -n "${EXCLUSION_NOTE:-}" ] && echo "$EXCLUSION_NOTE"
    [ -n "$EXCLUDE_SENSOR" ] && echo "excluded_sensor=$EXCLUDE_SENSOR"
    [ -n "$KNOWN_ISSUE" ] && echo "excluded_known_issue=$KNOWN_ISSUE"
    [ -n "$EXCLUSION_REASON" ] && echo "excluded_reason=$EXCLUSION_REASON"
    echo "ci=$ci_label + harness doctor"
  } > docs/harness/.sensors-last
}

run_step() {
  local label="$1"
  shift
  echo "==> [harness] $label"
  "$@"
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
    if run_step "baseline snapshot" bash docs/harness/bin/baseline.sh && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "baseline"
      echo "PASS (baseline lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "baseline"
    echo "FAIL"
    exit 1
    ;;
  quick)
    if run_step "fmt" cargo fmt --all -- --check && run_step "cargo check" cargo check && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "cargo fmt + cargo check"
      echo "PASS (quick lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "cargo fmt + cargo check"
    echo "FAIL"
    exit 1
    ;;
  docs)
    if run_step "MCP reference check" ./scripts/generate-mcp-reference.sh --check && run_step "rustdoc" env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "mcp reference + rustdoc"
      echo "PASS (docs lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "mcp reference + rustdoc"
    echo "FAIL"
    exit 1
    ;;
  mcp)
    if run_step "MCP reference check" ./scripts/generate-mcp-reference.sh --check && run_step "MCP protocol tests" cargo test --test mcp_protocol_tests && run_step "harness doctor" bash docs/harness/bin/doctor.sh; then
      write_sensors_last "pass" "pass" "pass" "$MODE" "mcp reference + protocol tests"
      echo "PASS (mcp lane green)"
      exit 0
    fi
    write_sensors_last "fail" "fail" "fail" "$MODE" "mcp reference + protocol tests"
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
write_sensors_last "$STATUS" "$CI_STATUS" "$DOCTOR_STATUS" "$MODE" "${CI_COMMAND[*]-missing}"

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
