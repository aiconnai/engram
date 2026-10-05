#!/usr/bin/env bash
# docs/harness/bin/doctor.sh
#
# Fast read-only consistency check for the engram harness layout and wiring.
# Exits 0 when consistent, 1 on validation failures, 2 on usage/env errors.
#
# Validates:
# - Required files and executability
# - Cross-references between README, bootstrap, review-gate, GATES, CODE_REVIEW_POLICY
# - Security contract anchors and scan/triage tuning files
# - Drift between SPEC.md and progress.md (sprint/task/plan)
# - Active plan file exists
# - Latest review for active task has parseable PASS/FAIL and explicit REVIEW_VERDICT marker (if present)
# - .sensors-last format (if present)
# - bootstrap.sh output size (<= 50 lines) and exit code
# - Live summary budget (progress.md <= 150 lines) and the churn-free structural live-state check (H6)
# - Exclusion records (if sensors-last indicates pass_with_exclusion)

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: bash docs/harness/bin/doctor.sh [--json]

Options:
  --json      Emit one machine-readable JSON object to stdout.
  -h, --help  Show this help.
EOF
}

JSON_MODE=0
SHOW_HELP=0
ARG_ERROR=""
for arg in "$@"; do
  case "$arg" in
    --json)
      JSON_MODE=1
      ;;
    -h|--help)
      SHOW_HELP=1
      ;;
    *)
      ARG_ERROR="unknown argument: $arg"
      ;;
  esac
done

if [ "$SHOW_HELP" -eq 1 ]; then
  usage
  exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: cannot resolve repo root from script location" >&2
  exit 2
fi

cd "$REPO_ROOT"

FAILURES=()
WARNINGS=()
CHECKS=()

add_check() {
  local id="$1"
  local status="$2"
  local message="$3"
  local path="${4:-}"
  CHECKS+=("${id}"$'\037'"${status}"$'\037'"${message}"$'\037'"${path}")
}

fail() {
  local message="$1"
  local id="${2:-}"
  local path="${3:-}"
  FAILURES+=("$message")
  if [ -n "$id" ]; then
    add_check "$id" "fail" "$message" "$path"
  fi
}

warn() {
  local message="$1"
  local id="${2:-}"
  local path="${3:-}"
  WARNINGS+=("$message")
  if [ -n "$id" ]; then
    add_check "$id" "warn" "$message" "$path"
  fi
}

join_records() {
  local separator="$1"
  shift || true
  local first=1
  local item
  for item in "$@"; do
    if [ "$first" -eq 0 ]; then
      printf '%s' "$separator"
    fi
    printf '%s' "$item"
    first=0
  done
}

emit_json() {
  local exit_code="$1"
  local status="$2"
  local summary="$3"
  local timestamp
  local rs=$'\036'
  local us=$'\037'
  local warnings_joined=""
  local failures_joined=""
  local checks_joined=""

  if ! command -v python3 >/dev/null 2>&1; then
    echo "ERROR: python3 is required for doctor.sh --json" >&2
    exit 2
  fi

  timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  if [ "${#WARNINGS[@]}" -gt 0 ]; then
    warnings_joined="$(join_records "$rs" "${WARNINGS[@]}")"
  fi
  if [ "${#FAILURES[@]}" -gt 0 ]; then
    failures_joined="$(join_records "$rs" "${FAILURES[@]}")"
  fi
  if [ "${#CHECKS[@]}" -gt 0 ]; then
    checks_joined="$(join_records "$rs" "${CHECKS[@]}")"
  fi

  DOCTOR_RS="$rs" \
  DOCTOR_US="$us" \
  DOCTOR_SCHEMA_VERSION="harness-json-v1" \
  DOCTOR_TOOL="doctor" \
  DOCTOR_MODE="json" \
  DOCTOR_STATUS="$status" \
  DOCTOR_EXIT_CODE="$exit_code" \
  DOCTOR_REPO_ROOT="$REPO_ROOT" \
  DOCTOR_TIMESTAMP="$timestamp" \
  DOCTOR_ACTIVE_PLAN="${ACTIVE_PLAN:-}" \
  DOCTOR_ACTIVE_TASK="${ACTIVE_TASK:-}" \
  DOCTOR_SUMMARY="$summary" \
  DOCTOR_WARNINGS="$warnings_joined" \
  DOCTOR_FAILURES="$failures_joined" \
  DOCTOR_CHECKS="$checks_joined" \
  python3 -c 'import json
import os

rs = os.environ["DOCTOR_RS"]
us = os.environ["DOCTOR_US"]


def split_records(name):
    value = os.environ.get(name, "")
    return [] if value == "" else value.split(rs)


def checks():
    output = []
    for record in split_records("DOCTOR_CHECKS"):
        parts = record.split(us)
        while len(parts) < 4:
            parts.append("")
        item = {
            "id": parts[0],
            "status": parts[1],
            "message": parts[2],
        }
        if parts[3]:
            item["path"] = parts[3]
        output.append(item)
    return output


payload = {
    "schema_version": os.environ["DOCTOR_SCHEMA_VERSION"],
    "tool": os.environ["DOCTOR_TOOL"],
    "mode": os.environ["DOCTOR_MODE"],
    "status": os.environ["DOCTOR_STATUS"],
    "exit_code": int(os.environ["DOCTOR_EXIT_CODE"]),
    "repo_root": os.environ["DOCTOR_REPO_ROOT"],
    "timestamp": os.environ["DOCTOR_TIMESTAMP"],
    "active_plan": os.environ.get("DOCTOR_ACTIVE_PLAN", ""),
    "active_task": os.environ.get("DOCTOR_ACTIVE_TASK", ""),
    "summary": os.environ["DOCTOR_SUMMARY"],
    "warnings": split_records("DOCTOR_WARNINGS"),
    "failures": split_records("DOCTOR_FAILURES"),
    "checks": checks(),
    "artifacts": [],
}

print(json.dumps(payload, ensure_ascii=False, separators=(",", ":")))'
}

if [ -n "$ARG_ERROR" ]; then
  if [ "$JSON_MODE" -eq 1 ]; then
    fail "$ARG_ERROR" "usage_error:argument" ""
    emit_json 2 "usage_error" "$ARG_ERROR"
  else
    echo "ERROR: $ARG_ERROR" >&2
    usage >&2
  fi
  exit 2
fi

require_file() {
  local path="$1"
  if [ -f "$path" ]; then
    add_check "required_file:$path" "pass" "required file exists" "$path"
  else
    fail "missing file: $path" "required_file:$path" "$path"
  fi
}

require_exec() {
  local path="$1"
  if [ -x "$path" ]; then
    add_check "required_exec:$path" "pass" "required script is executable" "$path"
  else
    fail "not executable: $path" "required_exec:$path" "$path"
  fi
}

require_bash_syntax() {
  local path="$1"
  if bash -n "$path" >/dev/null 2>&1; then
    add_check "bash_syntax:$path" "pass" "bash syntax is valid" "$path"
  else
    fail "bash syntax invalid: $path" "bash_syntax:$path" "$path"
  fi
}

check_shellcheck() {
  local path="$1"
  if ! command -v shellcheck >/dev/null 2>&1; then
    add_check "shellcheck:$path" "skipped" "shellcheck is not installed" "$path"
    return 0
  fi
  if shellcheck -x "$path" >/dev/null 2>&1; then
    add_check "shellcheck:$path" "pass" "shellcheck passed" "$path"
  else
    warn "shellcheck failed: $path" "shellcheck:$path" "$path"
  fi
}

require_dir() {
  local path="$1"
  if [ -d "$path" ]; then
    add_check "required_dir:$path" "pass" "required directory exists" "$path"
  else
    fail "missing directory: $path" "required_dir:$path" "$path"
  fi
}

require_grep() {
  local path="$1"
  local pattern="$2"
  local label="$3"
  if [ ! -f "$path" ] || ! grep -qE "$pattern" "$path"; then
    fail "missing reference in $path: $label" "cross_reference:$path:$label" "$path"
  else
    add_check "cross_reference:$path:$label" "pass" "required reference exists: $label" "$path"
  fi
}

# Core required structure
require_file docs/harness/SPEC.md
require_file docs/harness/INVARIANTS.md
require_file docs/harness/WHAT_WE_DONT_DO.md
require_file docs/harness/GATES.md
require_file docs/harness/CODE_REVIEW_POLICY.md
require_file docs/harness/JSON_OUTPUTS.md
require_file docs/harness/SKILLS.md
require_file docs/harness/README.md
require_file docs/harness/progress.md
require_file docs/harness/security/anthropic-reference-harness.md
require_file .claude/scan-extras.txt
require_file .claude/fp-rules.txt
require_file docs/harness/bin/bootstrap.sh
require_file docs/harness/bin/doctor.sh
require_file docs/harness/bin/baseline.sh
require_file docs/harness/bin/quarterly-audit.sh
require_file docs/harness/bin/check-pr-title.sh
require_file docs/harness/bin/pr-title-policy.sh
require_file docs/harness/bin/harness-stats.sh
require_file docs/harness/bin/harness-decision-log.sh
require_file docs/harness/bin/harness-risk-register.sh
require_file docs/harness/risk-register.yaml
require_file docs/harness/decisions/harness-decision-log.yaml
require_dir docs/harness/progress
require_dir docs/harness/reviews
require_dir docs/harness/known-issues
require_dir docs/harness/security
require_dir docs/harness/canvas
require_dir docs/harness/audits
require_file docs/harness/canvas/README.md
require_file docs/harness/canvas/TEMPLATE.md

# Scripts that must be executable (review-gate and check-commit-msg are optional in early v0 but preferred)
require_exec docs/harness/bin/bootstrap.sh
require_exec docs/harness/bin/doctor.sh
require_exec docs/harness/bin/baseline.sh
require_exec docs/harness/bin/quarterly-audit.sh
require_exec docs/harness/bin/check-pr-title.sh
require_exec docs/harness/bin/pr-title-policy.sh
require_exec docs/harness/bin/harness-stats.sh
require_exec docs/harness/bin/harness-decision-log.sh
require_exec docs/harness/bin/harness-risk-register.sh
require_exec docs/harness/bin/run-offline-lane.sh

# If the advanced scripts exist, they should be executable
SCRIPT_PATHS=(
  docs/harness/bin/bootstrap.sh
  docs/harness/bin/doctor.sh
  docs/harness/bin/baseline.sh
  docs/harness/bin/quarterly-audit.sh
  docs/harness/bin/check-pr-title.sh
  docs/harness/bin/pr-title-policy.sh
  docs/harness/bin/harness-stats.sh
  docs/harness/bin/harness-decision-log.sh
  docs/harness/bin/harness-risk-register.sh
  docs/harness/bin/run-offline-lane.sh
)
OPTIONAL_SCRIPT_PATHS=(
  docs/harness/bin/sensors.sh
  docs/harness/bin/review-gate.sh
  docs/harness/bin/check-commit-msg.sh
  docs/harness/bin/vc-gate.sh
  docs/harness/bin/lib.sh
)
for script_path in "${OPTIONAL_SCRIPT_PATHS[@]}"; do
  if [ -f "$script_path" ]; then
    require_exec "$script_path"
    SCRIPT_PATHS+=("$script_path")
  fi
done
for script_path in "${SCRIPT_PATHS[@]}"; do
  require_bash_syntax "$script_path"
  check_shellcheck "$script_path"
done

if bash docs/harness/bin/check-pr-title.sh --title "align lifecycle hook contracts" >/dev/null 2>&1; then
  add_check "pr_title_guard:allow_plain" "pass" "plain PR title is accepted" "docs/harness/bin/check-pr-title.sh"
else
  fail "check-pr-title.sh rejects a plain PR title" "pr_title_guard:allow_plain" "docs/harness/bin/check-pr-title.sh"
fi

if bash docs/harness/bin/pr-title-policy.sh --title "align lifecycle hook contracts" >/dev/null 2>&1; then
  add_check "pr_title_policy:allow_plain" "pass" "plain PR title is accepted by canonical policy" "docs/harness/bin/pr-title-policy.sh"
else
  fail "pr-title-policy.sh rejects a plain PR title" "pr_title_policy:allow_plain" "docs/harness/bin/pr-title-policy.sh"
fi

PR_TITLE_POLICY_STATUS=0
bash docs/harness/bin/pr-title-policy.sh --title "[codex] align lifecycle hook contracts" >/dev/null 2>&1 || PR_TITLE_POLICY_STATUS=$?
if [ "$PR_TITLE_POLICY_STATUS" -eq 4 ]; then
  add_check "pr_title_policy:block_codex_marker_exit4" "pass" "canonical policy rejects [codex] with exit 4" "docs/harness/bin/pr-title-policy.sh"
else
  fail "pr-title-policy.sh must reject [codex] PR marker with exit 4 (got ${PR_TITLE_POLICY_STATUS})" "pr_title_policy:block_codex_marker_exit4" "docs/harness/bin/pr-title-policy.sh"
fi

PR_TITLE_POLICY_SPACED_STATUS=0
bash docs/harness/bin/pr-title-policy.sh --title "[ CoDeX ] align lifecycle hook contracts" >/dev/null 2>&1 || PR_TITLE_POLICY_SPACED_STATUS=$?
if [ "$PR_TITLE_POLICY_SPACED_STATUS" -eq 4 ]; then
  add_check "pr_title_policy:block_spaced_codex_marker_exit4" "pass" "canonical policy rejects spaced/mixed-case [codex] with exit 4" "docs/harness/bin/pr-title-policy.sh"
else
  fail "pr-title-policy.sh must reject spaced/mixed-case [codex] PR marker with exit 4 (got ${PR_TITLE_POLICY_SPACED_STATUS})" "pr_title_policy:block_spaced_codex_marker_exit4" "docs/harness/bin/pr-title-policy.sh"
fi

if bash docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts" >/dev/null 2>&1; then
  fail "check-pr-title.sh allows forbidden [codex] PR marker" "pr_title_guard:block_codex_marker" "docs/harness/bin/check-pr-title.sh"
else
  add_check "pr_title_guard:block_codex_marker" "pass" "forbidden [codex] PR marker is blocked" "docs/harness/bin/check-pr-title.sh"
fi

CHECK_PR_TITLE_STATUS=0
bash docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts" >/dev/null 2>&1 || CHECK_PR_TITLE_STATUS=$?
if [ "$CHECK_PR_TITLE_STATUS" -eq 4 ]; then
  add_check "pr_title_guard:block_codex_marker_exit4" "pass" "compat wrapper shares canonical exit 4" "docs/harness/bin/check-pr-title.sh"
else
  fail "check-pr-title.sh must share canonical [codex] exit 4 (got ${CHECK_PR_TITLE_STATUS})" "pr_title_guard:block_codex_marker_exit4" "docs/harness/bin/check-pr-title.sh"
fi

if bash docs/harness/bin/check-pr-title.sh --pr --help >/dev/null 2>&1; then
  fail "check-pr-title.sh allows option-like PR identifiers" "pr_title_guard:block_option_like_pr" "docs/harness/bin/check-pr-title.sh"
else
  add_check "pr_title_guard:block_option_like_pr" "pass" "option-like PR identifiers are blocked before gh invocation" "docs/harness/bin/check-pr-title.sh"
fi

if bash docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts" --help >/dev/null 2>&1; then
  fail "check-pr-title.sh allows trailing help to bypass title validation" "pr_title_guard:block_trailing_help_title" "docs/harness/bin/check-pr-title.sh"
else
  add_check "pr_title_guard:block_trailing_help_title" "pass" "trailing help cannot bypass title validation" "docs/harness/bin/check-pr-title.sh"
fi

if bash docs/harness/bin/check-pr-title.sh --pr 91 --help >/dev/null 2>&1; then
  fail "check-pr-title.sh allows trailing help to bypass PR validation" "pr_title_guard:block_trailing_help_pr" "docs/harness/bin/check-pr-title.sh"
else
  add_check "pr_title_guard:block_trailing_help_pr" "pass" "trailing help cannot bypass PR validation" "docs/harness/bin/check-pr-title.sh"
fi

if bash docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts" --title "align lifecycle hook contracts" >/dev/null 2>&1; then
  fail "check-pr-title.sh allows duplicate title arguments to bypass validation" "pr_title_guard:block_duplicate_title" "docs/harness/bin/check-pr-title.sh"
else
  add_check "pr_title_guard:block_duplicate_title" "pass" "duplicate title arguments cannot bypass validation" "docs/harness/bin/check-pr-title.sh"
fi

# Cross-references (bootstrap + README point at the policy and doctor)
require_grep docs/harness/bin/bootstrap.sh 'CODE_REVIEW_POLICY\.md' 'read-next includes the local review policy'
require_grep docs/harness/bin/bootstrap.sh 'WHAT_WE_DONT_DO\.md' 'read-next includes negative-scope policy'
require_grep docs/harness/bin/bootstrap.sh 'anthropic-reference-harness\.md' 'read-next includes security boundary'
require_grep docs/harness/README.md 'WHAT_WE_DONT_DO\.md' 'workflow mentions the negative-scope policy'
require_grep docs/harness/README.md 'CODE_REVIEW_POLICY\.md' 'structure table or workflow mentions the policy file'
require_grep docs/harness/README.md 'anthropic-reference-harness\.md' 'workflow mentions the security boundary'
require_grep docs/harness/README.md '\.claude/scan-extras\.txt' 'workflow mentions scan tuning'
require_grep docs/harness/README.md '\.claude/fp-rules\.txt' 'workflow mentions false-positive tuning'
require_grep docs/harness/README.md 'doctor\.sh' 'workflow mentions the doctor check'
require_grep docs/harness/README.md 'JSON_OUTPUTS\.md' 'workflow mentions the JSON output contract'
require_grep docs/harness/README.md 'known-issues/' 'structure table or workflow mentions known issues'
require_grep docs/harness/README.md 'baseline\.sh' 'workflow mentions baseline snapshots'
require_grep docs/harness/README.md 'quarterly-audit\.sh' 'workflow mentions evidence-only audits'
require_grep docs/harness/README.md 'Sensor modes' 'workflow lists optional sensor modes'
require_grep docs/harness/README.md 'check-pr-title\.sh' 'workflow mentions PR title validation'
require_grep docs/harness/README.md 'pr-title-policy\.sh' 'workflow mentions canonical PR title policy'
require_grep docs/harness/GATES.md 'WHAT_WE_DONT_DO\.md' 'gates reference negative-scope policy'
require_grep docs/harness/GATES.md 'anthropic-reference-harness\.md' 'gates reference security boundary'
require_grep docs/harness/GATES.md '\.claude/scan-extras\.txt' 'gates reference scan tuning'
require_grep docs/harness/GATES.md '\.claude/fp-rules\.txt' 'gates reference false-positive tuning'
require_grep docs/harness/GATES.md 'Review Canvas' 'gates define review canvas requirement'
require_grep docs/harness/GATES.md 'baseline\.sh' 'gates document baseline snapshots'
require_grep docs/harness/GATES.md 'quarterly-audit\.sh' 'gates document evidence-only audit'
require_grep docs/harness/GATES.md 'optional lanes do not replace the full gate' 'gates preserve full sensor gate'
require_grep docs/harness/GATES.md 'docs/harness/bin' 'gates protect harness script changes'
require_grep docs/harness/GATES.md 'JSON_OUTPUTS\.md' 'gates reference JSON output contract'
require_grep docs/harness/GATES.md 'check-pr-title\.sh' 'gates document PR title validation'
require_grep docs/harness/GATES.md 'pr-title-policy\.sh' 'gates document canonical PR title policy'
require_grep docs/harness/GATES.md 'Exclus' 'documented exclusion policy exists'
require_grep docs/harness/GATES.md 'known-issue' 'exclusion policy points at known-issue docs'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'WHAT_WE_DONT_DO\.md' 'review policy enforces negative-scope policy'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'anthropic-reference-harness\.md' 'review policy enforces security boundary'
require_grep docs/harness/CODE_REVIEW_POLICY.md '\.claude/scan-extras\.txt' 'review policy references scan tuning'
require_grep docs/harness/CODE_REVIEW_POLICY.md '\.claude/fp-rules\.txt' 'review policy references false-positive tuning'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'Review Canvas' 'review policy checks complex-change canvas evidence'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'Harness script changes' 'review policy checks harness scripts directly'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'Finding Format|Finding format|severidade' 'review policy defines finding format'
require_grep docs/harness/CODE_REVIEW_POLICY.md 'PASS <resumo|Harness Output Contract' 'review policy defines output contract'
require_grep docs/harness/JSON_OUTPUTS.md 'doctor\.sh --json' 'JSON contract documents doctor JSON mode'
require_grep docs/harness/JSON_OUTPUTS.md 'harness-json-v1' 'JSON contract defines schema version'
require_grep docs/harness/JSON_OUTPUTS.md 'usage_error' 'JSON contract defines usage error status'
require_grep docs/harness/bin/review-gate.sh 'WHAT_WE_DONT_DO\.md' 'review-gate prompt includes negative-scope policy'
require_grep docs/harness/bin/review-gate.sh 'anthropic-reference-harness\.md' 'review-gate prompt includes security boundary'
require_grep docs/harness/bin/review-gate.sh '\.claude/scan-extras\.txt' 'review-gate prompt includes scan tuning'
require_grep docs/harness/bin/review-gate.sh '\.claude/fp-rules\.txt' 'review-gate prompt includes false-positive tuning'
require_grep docs/harness/bin/review-gate.sh 'Review Canvas' 'review-gate prompt includes review canvas checks'
require_grep docs/harness/bin/review-gate.sh 'docs/harness/bin' 'review-gate protects harness script changes'
require_grep docs/harness/bin/sensors.sh 'quick' 'sensors supports quick mode'
require_grep docs/harness/bin/sensors.sh 'full' 'sensors supports full mode'
require_grep docs/harness/bin/sensors.sh 'docs' 'sensors supports docs mode'
require_grep docs/harness/bin/sensors.sh 'mcp' 'sensors supports mcp mode'
require_grep docs/harness/bin/sensors.sh 'baseline' 'sensors supports baseline mode'
require_grep docs/harness/bin/sensors.sh 'status' 'sensors supports status mode'
require_grep docs/harness/bin/sensors.sh '\-\-json' 'sensors supports JSON status output'
require_grep docs/harness/bin/sensors.sh 'pr-title-policy\.sh' 'sensors runs canonical PR title policy'
require_grep docs/harness/bin/sensors.sh 'anthropic-reference-harness\.md' 'sensors summary includes security boundary'
require_grep docs/harness/bin/sensors.sh '\.claude/scan-extras\.txt' 'sensors summary includes scan tuning'
require_grep docs/harness/bin/sensors.sh '\.claude/fp-rules\.txt' 'sensors summary includes false-positive tuning'
require_grep docs/harness/INVARIANTS.md 'Static/read-only first' 'invariants declare static/read-only default'
require_grep docs/harness/INVARIANTS.md 'PR title.*\[codex\]' 'invariants forbid codex PR title marker'
require_grep docs/harness/INVARIANTS.md 'ADR.*sandbox|sandbox.*ADR' 'invariants require ADR and sandbox for autonomous execution'
require_grep docs/harness/INVARIANTS.md '\.claude/scan-extras\.txt' 'invariants point tuning outside core policy'
require_grep docs/harness/INVARIANTS.md '\.claude/fp-rules\.txt' 'invariants point false-positive tuning outside core policy'
require_grep docs/harness/security/anthropic-reference-harness.md 'ENGRAM-HARNESS-SECURITY-CONTRACT-v1' 'security contract version anchor'
require_grep docs/harness/security/anthropic-reference-harness.md 'DEFAULT_MODE=static_read_only' 'security contract default mode anchor'
require_grep docs/harness/security/anthropic-reference-harness.md 'AUTONOMOUS_EXECUTION_REQUIRES_ADR=true' 'security contract ADR anchor'
require_grep docs/harness/security/anthropic-reference-harness.md 'NO_CREDENTIAL_MOUNTS=true' 'security contract credential anchor'
require_grep docs/harness/security/anthropic-reference-harness.md 'TUNING_FILES=\.claude/scan-extras\.txt,\.claude/fp-rules\.txt' 'security contract tuning anchor'
require_grep .claude/scan-extras.txt 'scan-extras' 'scan tuning file identifies itself'
require_grep .claude/fp-rules.txt 'fp-rules' 'false-positive tuning file identifies itself'

# Offline mandatory lane (H2). The lane, its suites and its wiring into sensors.sh (quick AND full),
# scripts/ci.sh and the required "Test (ubuntu-latest)" CI job must all stay in place; removing any
# of them is a doctor failure. Fixtures or manual runs are not a substitute for the lane.
require_file docs/harness/tests/test_validate_evidence.py
require_file docs/harness/tests/test_offline_lane.py
# H6 short, resumable context: live summary + byte-exact history, budget doc, link checker, measure tool.
require_file docs/harness/context-budget.md
require_file docs/harness/progress-history.md
require_file docs/harness/tests/test_context_budget.py
require_file docs/harness/bin/doc_links.py
require_exec docs/harness/bin/check-doc-links.py
require_exec docs/harness/bin/measure-context.py
# H3 sandbox adapter, registry and fake-writer fixtures. The unit tests run in the offline lane; the
# real-Docker smoke (run-sandbox-smoke.sh) is deliberately NOT wired into the lane or CI.
require_file docs/harness/checks/registry.json
require_file docs/harness/tests/fake_writer.py
require_file docs/harness/tests/sandbox_test_support.py
require_file docs/harness/tests/test_sandbox_adapter.py
require_file docs/harness/tests/test_sandbox_smoke.py
require_exec docs/harness/bin/sandbox-adapter.py
require_file docs/harness/bin/sandbox_registry.py
require_exec docs/harness/bin/run-sandbox-smoke.sh
# H4 trusted runner (fake writer only), scope checker and external evidence. Offline tests run in the lane;
# the real-Docker smoke (run-runner-smoke.sh) is deliberately NOT wired into the lane or CI.
require_exec docs/harness/bin/run-task.py
require_exec docs/harness/bin/check-scope.py
require_exec docs/harness/bin/record-evidence.py
require_file docs/harness/bin/harness_git.py
require_exec docs/harness/bin/run-runner-smoke.sh
require_file docs/harness/tests/runner_test_support.py
require_file docs/harness/tests/test_runner.py
require_file docs/harness/tests/test_scope.py
require_file docs/harness/tests/test_evidence_integrity.py
require_file docs/harness/tests/test_runner_smoke.py
require_file docs/harness/bin/validate-evidence.py
require_file docs/harness/bin/test-fixtures.sh
require_file docs/harness/bin/test-check-live-state.sh
require_file docs/harness/bin/test-review-gate.sh
require_grep docs/harness/bin/run-offline-lane.sh "unittest discover -s docs/harness/tests -p 'test_validate_evidence\.py'" 'offline lane runs the validator unit tests'
require_grep docs/harness/bin/run-offline-lane.sh 'validate-evidence\.py --self-test' 'offline lane runs the validator self-test'
require_grep docs/harness/bin/run-offline-lane.sh 'bash docs/harness/bin/test-fixtures\.sh' 'offline lane runs the fixtures suite'
require_grep docs/harness/bin/run-offline-lane.sh 'bash docs/harness/bin/test-check-live-state\.sh' 'offline lane runs the live-state suite'
require_grep docs/harness/bin/run-offline-lane.sh 'bash docs/harness/bin/test-review-gate\.sh' 'offline lane runs the review-gate suite'
require_grep docs/harness/bin/run-offline-lane.sh "unittest discover -s docs/harness/tests -p 'test_offline_lane\.py'" 'offline lane runs its own fail-closed contract tests'
require_grep docs/harness/bin/run-offline-lane.sh "unittest discover -s docs/harness/tests -p 'test_sandbox_adapter\.py'" 'offline lane runs the sandbox adapter unit tests (H3, no Docker needed)'
require_grep docs/harness/bin/run-offline-lane.sh "unittest discover -s docs/harness/tests -p 'test_context_budget\.py'" 'offline lane runs the context budget and routing tests (H6)'
require_grep docs/harness/bin/run-offline-lane.sh 'unittest -v docs/harness/tests/test_runner\.py docs/harness/tests/test_scope\.py docs/harness/tests/test_evidence_integrity\.py' 'offline lane runs the runner, scope and evidence tests (H4, no Docker needed)'

# H5 read-only merge-policy evaluator: its tests are a mandatory lane component, and the
# workflow that runs it in CI stays read-only (contents: read only, no pull_request_target, no
# secrets, evaluator from the base checkout).
require_exec docs/harness/bin/merge-gate.py
require_file docs/harness/tests/test_merge_gate.py
require_file docs/harness/bin/merge_gate_ci.py
require_file docs/harness/tests/merge_gate_test_support.py
require_grep docs/harness/bin/run-offline-lane.sh \
  "unittest discover -s docs/harness/tests -p 'test_merge_gate\.py'" \
  'offline lane runs the merge-gate evaluator tests (H5)'

# O2 retention manifest/backup/restore tool: its hermetic tests are a mandatory lane component.
require_exec docs/harness/bin/retention-manifest.py
require_file docs/harness/tests/test_retention.py
require_file docs/harness/retention/review-raw.manifest.json
require_file docs/OPERATIONS_GIT_RETENTION.md
require_grep docs/harness/bin/run-offline-lane.sh "unittest discover -s docs/harness/tests -p 'test_retention\.py'" 'offline lane runs the retention tests (O2)'

# O4 read-only standing checks with ownership (alert-only): runner, goals registry + schema, runbook and their hermetic
# tests are a mandatory lane component; the scheduled workflow stays read-only and alert-only.
require_exec docs/harness/bin/run-standing-checks.py
require_file docs/harness/tests/test_standing_checks.py
require_file docs/harness/goals/registry.json
require_file docs/harness/goals/README.md
require_file docs/harness/schemas/goal-v1.schema.json
require_grep docs/harness/bin/run-offline-lane.sh "unittest discover -s docs/harness/tests -p 'test_standing_checks\.py'" 'offline lane runs the standing-checks tests (O4)'
STANDING_WF=.github/workflows/standing-checks.yml
STANDING_CODE="$(grep -v '^[[:space:]]*#' "$STANDING_WF" 2>/dev/null || true)"
if [ ! -f "$STANDING_WF" ]; then
  fail "standing-checks workflow is missing" "standing_checks_workflow:read_only" "$STANDING_WF"
elif grep -Eq 'pull_request|secrets\.|write|git (commit|push)|gh (pr|issue)' <<<"$STANDING_CODE" \
  || [ "$(grep -A2 '^permissions:$' <<<"$STANDING_CODE" | sed -n 2,3p)" != "  contents: read" ] \
  || ! grep -Eq 'run-standing-checks\.py run --mode scheduled' <<<"$STANDING_CODE"; then
  fail "standing-checks workflow is not read-only/alert-only (pull_request trigger, secrets, write permission, commit/push/gh or runner missing)" "standing_checks_workflow:read_only" "$STANDING_WF"
else
  add_check "standing_checks_workflow:read_only" "pass" "standing-checks workflow is read-only, alert-only and runs the standing-checks runner" "$STANDING_WF"
fi
AGENT_EVIDENCE_WF=.github/workflows/agent-evidence.yml
AE_ID=merge_gate_workflow
AGENT_EVIDENCE_CODE="$(grep -v '^[[:space:]]*#' "$AGENT_EVIDENCE_WF" 2>/dev/null || true)"
AE_PERMS="$(grep -A2 '^permissions:$' <<<"$AGENT_EVIDENCE_CODE" | sed -n 2,3p)"
AE_EVAL='^[[:space:]]+python3 trusted/docs/harness/bin/merge-gate\.py'
if [ ! -f "$AGENT_EVIDENCE_WF" ]; then
  fail "agent-evidence workflow is missing" "$AE_ID:read_only" "$AGENT_EVIDENCE_WF"
elif grep -Eq 'pull_request_target|secrets\.|write' <<<"$AGENT_EVIDENCE_CODE" \
  || [ "$AE_PERMS" != "  contents: read" ] \
  || ! grep -Eq "$AE_EVAL" <<<"$AGENT_EVIDENCE_CODE"; then
  AE_MSG="agent-evidence workflow is not read-only (pull_request_target, secrets, write"
  AE_MSG="$AE_MSG permission, extra permissions or evaluator outside trusted/)"
  fail "$AE_MSG" "$AE_ID:read_only" "$AGENT_EVIDENCE_WF"
else
  add_check "$AE_ID:read_only" "pass" \
    "agent-evidence workflow is read-only and runs the base-revision evaluator" "$AGENT_EVIDENCE_WF"
fi
# Injection guard: a ${{ }} expression inside a run: script (inline or block) would splice
# PR-controlled text into shell; expressions must reach scripts through env:.
if [ -f "$AGENT_EVIDENCE_WF" ] && ! awk '
  { match($0, /^ */); w = RLENGTH; s = substr($0, w + 1)
    if (inrun && s != "" && w > ind) { if (index($0, "${{")) bad = 1; next }
    inrun = 0; k = s; sub(/^- /, "", k)
    if (k ~ /^run:/) { inrun = 1; ind = w; if (index(k, "${{")) bad = 1 } }
  END { exit bad ? 1 : 0 }' <<<"$AGENT_EVIDENCE_CODE"; then
  fail "agent-evidence workflow has a \${{ }} expression inside a run: script (use env:)" \
    "$AE_ID:no_inline_expressions" "$AGENT_EVIDENCE_WF"
else
  add_check "$AE_ID:no_inline_expressions" "pass" \
    "agent-evidence run: scripts take expressions only through env" "$AGENT_EVIDENCE_WF"
fi

# Active (non-comment) call sites only: a mention in a comment must not satisfy the check.
lane_wiring_check() {
  local id="$1" path="$2" text="$3" pattern="$4" label="$5"
  if [ -f "$path" ] && grep -Eq "^[^#]*${pattern}" <<<"$text"; then
    add_check "offline_lane_wiring:${id}" "pass" "offline lane is wired: ${label}" "$path"
  else
    fail "offline lane wiring missing: ${label} ($path)" "offline_lane_wiring:${id}" "$path"
  fi
}
SENSORS_QUICK_BLOCK="$(awk '/^  quick\)$/{f=1;next} /^  docs\)$/{f=0} f' docs/harness/bin/sensors.sh 2>/dev/null || true)"
SENSORS_FULL_BLOCK="$(awk '/^resolve_ci_required_features$/{f=1} f' docs/harness/bin/sensors.sh 2>/dev/null || true)"
SENSORS_HELPER_BLOCK="$(awk '/^run_offline_lane\(\) \{$/{f=1} f{print} /^\}$/{if(f) exit}' docs/harness/bin/sensors.sh 2>/dev/null || true)"
CI_TEST_JOB_BLOCK="$(awk '/^  test:$/{f=1;next} f && /^  [A-Za-z0-9_-]+:$/{f=0} f' .github/workflows/ci.yml 2>/dev/null || true)"
CI_LANE_STEP_BLOCK="$(awk '/^      - name: Offline harness lane \(/{f=1;print;next} f && /^      - /{f=0} f' .github/workflows/ci.yml 2>/dev/null || true)"
lane_wiring_check "sensors_helper" docs/harness/bin/sensors.sh "$SENSORS_HELPER_BLOCK" 'bash docs/harness/bin/run-offline-lane\.sh' 'sensors.sh run_offline_lane helper runs the runner'
lane_wiring_check "sensors_quick" docs/harness/bin/sensors.sh "$SENSORS_QUICK_BLOCK" '\brun_offline_lane\b' 'sensors.sh quick mode'
lane_wiring_check "sensors_full" docs/harness/bin/sensors.sh "$SENSORS_FULL_BLOCK" 'bash docs/harness/bin/run-offline-lane\.sh' 'sensors.sh full mode'
lane_wiring_check "ci_script" scripts/ci.sh "$(cat scripts/ci.sh 2>/dev/null || true)" 'bash .*docs/harness/bin/run-offline-lane\.sh' 'scripts/ci.sh'
lane_wiring_check "ci_job_name" .github/workflows/ci.yml "$CI_TEST_JOB_BLOCK" 'name: Test \(ubuntu-latest\)' 'ci.yml test job keeps the required name "Test (ubuntu-latest)"'
lane_wiring_check "ci_step" .github/workflows/ci.yml "$CI_LANE_STEP_BLOCK" 'run: bash docs/harness/bin/run-offline-lane\.sh' 'ci.yml required Test job step'
if grep -Eq '^[[:space:]]*(continue-on-error|if):' <<<"$CI_LANE_STEP_BLOCK" \
  || grep -Eq '^    (continue-on-error|if):' <<<"$CI_TEST_JOB_BLOCK"; then
  fail "offline lane CI step or the Test job is conditional/continue-on-error; the lane must be unconditional" "offline_lane_wiring:ci_unconditional" ".github/workflows/ci.yml"
else
  add_check "offline_lane_wiring:ci_unconditional" "pass" "offline lane CI step is unconditional" ".github/workflows/ci.yml"
fi

# Repository skills inventory and frontmatter validation
require_grep docs/harness/README.md 'SKILLS\.md' 'README mentions SKILLS.md'
# shellcheck disable=SC2016  # literal backticks are part of the grep pattern
require_grep docs/harness/SKILLS.md '`loop-engineering`' 'SKILLS documents loop-engineering'
# shellcheck disable=SC2088  # literal tilde is part of the grep pattern
require_grep docs/harness/SKILLS.md '~/.codex/skills' 'SKILLS documents personal skill location'

UNTRACKED_SKILLS="$(git ls-files --others --exclude-standard -- 'skills/*/SKILL.md' 2>/dev/null || true)"
if [ -n "$UNTRACKED_SKILLS" ]; then
  fail "untracked repo-local skills found: $UNTRACKED_SKILLS" "skills:untracked" ""
else
  add_check "skills:untracked" "pass" "repo-local skills are tracked or ignored" "skills"
fi

# Extract the YAML frontmatter block (between the leading `---` and the next `---`).
# Fail-closed: returns nothing unless the file starts with a `---` fence AND a
# matching closing `---` fence is observed. A malformed/unterminated block yields
# no output, so downstream field checks fail.
skill_frontmatter() {
  local file="$1"
  awk '
    NR == 1 && $0 !~ /^---[[:space:]]*$/ { exit }
    NR == 1 { next }
    /^---[[:space:]]*$/ { closed = 1; exit }
    { buf = buf $0 "\n" }
    END { if (closed) printf "%s", buf }
  ' "$file" 2>/dev/null || true
}

# Validate a `key:` line exists inside the frontmatter block, optionally matching a value regex.
require_frontmatter_field() {
  local file="$1"
  local key="$2"
  local value_re="$3"
  local label="$4"
  local block
  block="$(skill_frontmatter "$file")"
  if [ -z "$block" ]; then
    fail "skill has no YAML frontmatter block: $file ($label)" "skill_frontmatter:$file:$key" "$file"
    return
  fi
  if [ "$key" = "name" ]; then
    local actual
    actual="$(printf '%s\n' "$block" | awk -F':[[:space:]]*' '$1 == "name" { print $2; exit }')"
    if [ "$actual" = "$value_re" ]; then
      add_check "skill_frontmatter:$file:$key" "pass" "frontmatter field present: $label" "$file"
    else
      fail "skill frontmatter missing/invalid $key: $file ($label)" "skill_frontmatter:$file:$key" "$file"
    fi
  elif printf '%s\n' "$block" | grep -qE "^${key}:[[:space:]]*${value_re}[[:space:]]*$"; then
    add_check "skill_frontmatter:$file:$key" "pass" "frontmatter field present: $label" "$file"
  else
    fail "skill frontmatter missing/invalid $key: $file ($label)" "skill_frontmatter:$file:$key" "$file"
  fi
}

# Parse the canonical set of skill names from the SKILLS.md "Current Skills" table only.
# The "Available for Follow-Up" table lists skills that are documented but NOT yet ported,
# so it must not count as inventory membership.
current_skills_set() {
  awk '
    /^## Current Skills/ { in_section = 1; next }
    /^## / { in_section = 0 }
    in_section && /^\| `[^`]+` \|/ {
      line = $0
      sub(/^\| `/, "", line)
      sub(/`.*$/, "", line)
      print line
    }
  ' docs/harness/SKILLS.md 2>/dev/null | sort -u
}

CURRENT_SKILLS="$(current_skills_set)"

skill_in_current() {
  local name="$1"
  printf '%s\n' "$CURRENT_SKILLS" | grep -qxF "$name"
}

# disk -> inventory: every on-disk skill must have valid frontmatter and be in the Current Skills table.
while IFS= read -r skill_file; do
  [ -n "$skill_file" ] || continue
  skill_dir="$(basename "$(dirname "$skill_file")")"
  require_frontmatter_field "$skill_file" "name" "$skill_dir" "skill name matches dir: $skill_dir"
  require_frontmatter_field "$skill_file" "description" ".+" "skill has description: $skill_dir"
  if skill_in_current "$skill_dir"; then
    add_check "skill_inventoried:$skill_dir" "pass" "skill is in SKILLS.md Current Skills table: $skill_dir" "$skill_file"
  else
    fail "on-disk skill not in SKILLS.md Current Skills table: $skill_dir (follow-up-only or unlisted skills must be promoted before landing)" "skill_inventoried:$skill_dir" "$skill_file"
  fi
done < <(find skills -mindepth 2 -maxdepth 2 -name SKILL.md | sort)

# inventory -> disk: every skill named in the Current Skills table must exist on disk.
while IFS= read -r inv_skill; do
  [ -n "$inv_skill" ] || continue
  if [ -f "skills/$inv_skill/SKILL.md" ]; then
    add_check "skill_inventory_exists:$inv_skill" "pass" "inventoried skill exists on disk: $inv_skill" "skills/$inv_skill/SKILL.md"
  else
    fail "inventoried skill missing on disk: skills/$inv_skill/SKILL.md" "skill_inventory_exists:$inv_skill" "skills/$inv_skill/SKILL.md"
  fi
done < <(printf '%s\n' "$CURRENT_SKILLS")

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

require_field_value() {
  local value="$1"
  local file="$2"
  local field="$3"
  local id="$4"
  if [ -n "$value" ]; then
    add_check "$id" "pass" "$field field is present" "$file"
  else
    fail "$file missing $field field" "$id" "$file"
  fi
}

check_equal_field() {
  local left="$1"
  local right="$2"
  local label="$3"
  local id="$4"
  if [ -n "$left" ] && [ -n "$right" ]; then
    if [ "$left" = "$right" ]; then
      add_check "$id" "pass" "$label matches between SPEC.md and progress.md" "docs/harness/progress.md"
    else
      fail "$label drift: SPEC='$left' progress='$right'" "$id" "docs/harness/progress.md"
    fi
  fi
}

# Drift checks between SPEC and progress
if [ -f docs/harness/SPEC.md ] && [ -f docs/harness/progress.md ]; then
  SPEC_SPRINT="$(field_value docs/harness/SPEC.md "Active sprint")"
  PROGRESS_SPRINT="$(field_value docs/harness/progress.md "Active sprint")"
  SPEC_TASK="$(field_value docs/harness/SPEC.md "Active task")"
  PROGRESS_TASK="$(field_value docs/harness/progress.md "Active task")"
  SPEC_PLAN="$(field_value docs/harness/SPEC.md "Active plan")"
  PROGRESS_PLAN="$(field_value docs/harness/progress.md "Active plan")"

  require_field_value "$SPEC_SPRINT" docs/harness/SPEC.md "Active sprint" "active_plan:spec_active_sprint"
  require_field_value "$PROGRESS_SPRINT" docs/harness/progress.md "Active sprint" "active_plan:progress_active_sprint"
  require_field_value "$SPEC_TASK" docs/harness/SPEC.md "Active task" "active_plan:spec_active_task"
  require_field_value "$PROGRESS_TASK" docs/harness/progress.md "Active task" "active_plan:progress_active_task"
  require_field_value "$SPEC_PLAN" docs/harness/SPEC.md "Active plan" "active_plan:spec_active_plan"
  require_field_value "$PROGRESS_PLAN" docs/harness/progress.md "Active plan" "active_plan:progress_active_plan"

  check_equal_field "$SPEC_SPRINT" "$PROGRESS_SPRINT" "Active sprint" "active_plan:drift_sprint"
  check_equal_field "$SPEC_TASK" "$PROGRESS_TASK" "Active task" "active_plan:drift_task"
  check_equal_field "$SPEC_PLAN" "$PROGRESS_PLAN" "Active plan" "active_plan:drift_plan"
fi

# Active plan file must exist
ACTIVE_PLAN="$(field_value docs/harness/progress.md "Active plan")"
if [ -z "$ACTIVE_PLAN" ]; then
  ACTIVE_PLAN="$(field_value docs/harness/SPEC.md "Active plan")"
fi
if [ -n "$ACTIVE_PLAN" ]; then
  ACTIVE_LOG="$(normalize_harness_path "$ACTIVE_PLAN")"
  if [ ! -f "$ACTIVE_LOG" ]; then
    fail "active plan log missing: $ACTIVE_LOG" "active_plan:file" "$ACTIVE_LOG"
  else
    add_check "active_plan:file" "pass" "active plan log exists" "$ACTIVE_LOG"
  fi
else
  add_check "active_plan:file" "skipped" "no active plan field was available" ""
fi

# Latest review for active task should have a verdict marker if it exists.
ACTIVE_TASK="$(field_value docs/harness/progress.md "Active task")"
ACTIVE_TASK_ID="$(task_id_from_value "$ACTIVE_TASK")"
ACTIVE_REVIEW="$(review_for_task "$ACTIVE_TASK_ID")"
if [ -n "$ACTIVE_REVIEW" ]; then
  if grep -qE '^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL)[[:space:]].+$' "$ACTIVE_REVIEW"; then
    add_check "review_verdict:$ACTIVE_TASK_ID" "pass" "active task review has REVIEW_VERDICT marker" "$ACTIVE_REVIEW"
  elif grep -qE '^(PASS|FAIL)([[:space:]:.,;-]|$)' "$ACTIVE_REVIEW"; then
    warn "active task review artifact is missing REVIEW_VERDICT marker: $ACTIVE_REVIEW (expected from review-gate post hard gate)" "review_verdict:$ACTIVE_TASK_ID" "$ACTIVE_REVIEW"
  else
    fail "active task review artifact has no PASS/FAIL verdict: $ACTIVE_REVIEW" "review_verdict:$ACTIVE_TASK_ID" "$ACTIVE_REVIEW"
  fi
elif [ -n "$ACTIVE_TASK_ID" ]; then
  warn "no review artifact found for active task: $ACTIVE_TASK_ID (expected after first post-gate)" "review_verdict:$ACTIVE_TASK_ID" "docs/harness/reviews"
else
  add_check "review_verdict:active_task" "skipped" "no active task field was available" ""
fi

# .sensors-last format (when present)
if [ -f docs/harness/.sensors-last ]; then
  if ! grep -qE '^status=(pass|pass_with_exclusion|fail)$' docs/harness/.sensors-last; then
    fail ".sensors-last is not parseable (expected status=...)" "sensors_last:format" "docs/harness/.sensors-last"
  else
    add_check "sensors_last:format" "pass" ".sensors-last has parseable status" "docs/harness/.sensors-last"
  fi
else
  warn ".sensors-last missing (run sensors.sh at least once)" "sensors_last:format" "docs/harness/.sensors-last"
fi

if [ -f docs/harness/.sensors-log ]; then
  if command -v python3 >/dev/null 2>&1; then
    if SENSORS_LOG_ERROR="$(python3 -c 'import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
required = {
    "schema_version",
    "timestamp",
    "tool",
    "mode",
    "status",
    "duration_sec",
    "ci_status",
    "doctor_status",
    "ci_command",
    "artifacts",
}
allowed_status = {"pass", "pass_with_exclusion", "fail"}
lines = [line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
if not lines:
    raise SystemExit("empty JSONL log")
for index, line in enumerate(lines, start=1):
    try:
        item = json.loads(line)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"line {index}: invalid JSON: {exc}") from exc
    missing = sorted(required - item.keys())
    if missing:
        missing_keys = ", ".join(missing)
        raise SystemExit(f"line {index}: missing keys: {missing_keys}")
    schema_version = item["schema_version"]
    tool = item["tool"]
    status = item["status"]
    if schema_version != "sensors-log-v1":
        raise SystemExit(f"line {index}: unexpected schema_version={schema_version!r}")
    if tool != "sensors":
        raise SystemExit(f"line {index}: unexpected tool={tool!r}")
    if status not in allowed_status:
        raise SystemExit(f"line {index}: unexpected status={status!r}")
    if not isinstance(item["duration_sec"], int) or item["duration_sec"] < 0:
        raise SystemExit(f"line {index}: duration_sec must be a non-negative integer")
    if not isinstance(item["artifacts"], list):
        raise SystemExit(f"line {index}: artifacts must be an array")' docs/harness/.sensors-log 2>&1)"; then
      add_check "sensors_log:format" "pass" ".sensors-log has parseable sensors-log-v1 JSONL" "docs/harness/.sensors-log"
    else
      fail ".sensors-log is not parseable: $SENSORS_LOG_ERROR" "sensors_log:format" "docs/harness/.sensors-log"
    fi
  else
    warn "python3 unavailable; cannot validate .sensors-log JSONL" "sensors_log:format" "docs/harness/.sensors-log"
  fi
else
  warn ".sensors-log missing (run sensors.sh at least once for historical measurements)" "sensors_log:format" "docs/harness/.sensors-log"
fi

# Live summary budget (H6): progress.md stays a short, resumable summary; history lives in progress-history.md.
PROGRESS_LINES="$(wc -l <docs/harness/progress.md 2>/dev/null | tr -d ' ' || echo 999)"
if [ "${PROGRESS_LINES:-999}" -gt 150 ]; then
  fail "progress.md has ${PROGRESS_LINES} lines (live summary budget <= 150; move history to progress-history.md, see context-budget.md)" "live_summary:budget" "docs/harness/progress.md"
else
  add_check "live_summary:budget" "pass" "progress.md live summary is within 150 lines (${PROGRESS_LINES})" "docs/harness/progress.md"
fi

# Churn-free live-state enforcement on the real progress.md (H6): required fields, active plan, authoritative
# review, reconciliation rows and a well-formed Last commit id. Ancestry is verified when possible; a shallow clone or
# a SHA-rewriting merge (squash/rebase) is a WARN here, never a failure. The strict check (HEAD/parent + sensors
# timestamp, `check-live-state.sh --progress docs/harness/progress.md`) stays the task-closing check.
if [ -f docs/harness/bin/check-live-state.sh ] && [ -f docs/harness/progress.md ]; then
  if LIVE_STATE_OUTPUT="$(bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md --structural 2>&1)"; then
    LIVE_STATE_ANCESTRY="$(printf '%s\n' "$LIVE_STATE_OUTPUT" | sed -n 's/^ancestor_check=//p' | head -1)"
    case "$LIVE_STATE_ANCESTRY" in
      ancestor)
        add_check "live_state:structural" "pass" "progress.md live state is structurally consistent with the repository" "docs/harness/progress.md"
        ;;
      skipped-shallow)
        warn "ancestry not verified (shallow clone): progress.md Last commit could not be checked against HEAD" "live_state:structural" "docs/harness/progress.md"
        ;;
      unreachable)
        warn "progress.md Last commit is not reachable from HEAD (squash/rebase merges rewrite SHAs, or history diverged); refresh Last commit" "live_state:structural" "docs/harness/progress.md"
        ;;
      *)
        fail "progress.md live state structural check gave no usable ancestry verdict (ancestor_check='${LIVE_STATE_ANCESTRY}')" "live_state:structural" "docs/harness/progress.md"
        ;;
    esac
  else
    fail "progress.md live state is structurally inconsistent: $(printf '%s' "$LIVE_STATE_OUTPUT" | grep -v '^head=\|^mode=\|^ancestor_check=\|^worktree_status=\|^remediation:' | tr '\n' ' ')" "live_state:structural" "docs/harness/progress.md"
  fi
fi

# Bootstrap contract: runs and produces limited output
if BOOTSTRAP_OUTPUT="$(bash docs/harness/bin/bootstrap.sh 2>/dev/null)"; then
  add_check "bootstrap_contract:exec" "pass" "bootstrap.sh executed cleanly" "docs/harness/bin/bootstrap.sh"
  BOOTSTRAP_LINES="$(printf '%s\n' "$BOOTSTRAP_OUTPUT" | wc -l | tr -d ' ')"
else
  fail "bootstrap.sh failed to execute cleanly" "bootstrap_contract:exec" "docs/harness/bin/bootstrap.sh"
  BOOTSTRAP_LINES=999
fi
if [ "$BOOTSTRAP_LINES" -gt 50 ]; then
  fail "bootstrap output too long: ${BOOTSTRAP_LINES} lines (contract <= 50, docs/harness/context-budget.md)" "bootstrap_contract:output_size" "docs/harness/bin/bootstrap.sh"
else
  add_check "bootstrap_contract:output_size" "pass" "bootstrap output size is within contract" "docs/harness/bin/bootstrap.sh"
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
    fail "sensors context indicates pass_with_exclusion but no known_issue path was provided" "exclusion_record:known_issue" ""
  elif ! grep -qF "$KNOWN_ISSUE" docs/harness/progress.md; then
    fail ".sensors-last / context indicates exclusion not mentioned in progress.md: $KNOWN_ISSUE" "exclusion_record:known_issue" "docs/harness/progress.md"
  else
    add_check "exclusion_record:known_issue" "pass" "known issue exclusion is recorded in progress.md" "docs/harness/progress.md"
  fi
else
  add_check "exclusion_record:known_issue" "skipped" "no pass_with_exclusion context present" ""
fi

# Summary
if [ "${#FAILURES[@]}" -gt 0 ]; then
  if [ "$JSON_MODE" -eq 1 ]; then
    emit_json 1 "fail" "harness doctor found ${#FAILURES[@]} issue(s)"
    exit 1
  fi
  echo "FAIL harness doctor found ${#FAILURES[@]} issue(s)"
  for item in "${FAILURES[@]}"; do
    echo "- $item"
  done
  exit 1
fi

if [ "$JSON_MODE" -eq 1 ]; then
  if [ "${#WARNINGS[@]}" -gt 0 ]; then
    emit_json 0 "warn" "harness doctor passed with ${#WARNINGS[@]} warning(s)"
  else
    emit_json 0 "pass" "harness doctor passed"
  fi
  exit 0
fi

echo "OK harness doctor"
if [ "${#WARNINGS[@]}" -gt 0 ]; then
  for item in "${WARNINGS[@]}"; do
    echo "WARN: $item"
  done
fi

echo "Checked: required docs + executables, negative-scope/canvas/baseline/audit wiring, cross-references to policy/doctor, SPEC<->progress drift, active plan existence, review verdict presence, .sensors-last format, bootstrap contract, and exclusion records."
