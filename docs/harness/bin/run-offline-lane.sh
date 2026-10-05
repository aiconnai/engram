#!/usr/bin/env bash
# docs/harness/bin/run-offline-lane.sh
#
# Mandatory OFFLINE harness lane (task H2). Runs, fail-closed, the deterministic regression
# suites that guard the harness itself:
#
#   validator_unit   python3 -m unittest ... test_validate_evidence.py
#   validator_self   validate-evidence.py --self-test
#   fixtures         test-fixtures.sh
#   live_state       test-check-live-state.sh
#   review_gate      test-review-gate.sh
#   lane_contract    python3 -m unittest ... test_offline_lane.py  (this runner's own fail-closed contract)
#   sandbox_unit     python3 -m unittest ... test_sandbox_adapter.py (H3 sandbox adapter, OFFLINE: fake docker CLI,
#                    no Docker needed; the real-Docker smoke is run-sandbox-smoke.sh and is NOT part of this lane)
#   context_budget   python3 -m unittest ... test_context_budget.py (H6: live summary / bootstrap budgets, byte-exact
#                    history retention, link + anchor checker, mandatory read order, measurement tool)
#   runner_unit      python3 -m unittest -v test_runner.py test_scope.py test_evidence_integrity.py (H4 trusted
#                    runner, scope checker, evidence recorder/verifier; OFFLINE: stub docker CLI, no Docker
#                    needed; the real-Docker smoke is run-runner-smoke.sh and is NOT part of this lane)
#   merge_gate       python3 -m unittest ... test_merge_gate.py (H5 read-only merge-policy
#                    evaluator over a real stub-docker H4 run, receipt producer/consumers and the
#                    agent-evidence.yml read-only contract; OFFLINE)
#   retention        python3 -m unittest ... test_retention.py (O2 retention manifest, backup/restore and the
#                    disposable-clone recovery proof; hermetic synthetic repos, no network)
#   standing_checks  python3 -m unittest ... test_standing_checks.py (O4 read-only standing checks with ownership:
#                    goals validation, per-goal lock, timeout/failure/missing artifact never pass, alert-only
#                    reports, read-only workflow contract; OFFLINE: stub docker CLI, no Docker needed)
#
# It is wired into sensors.sh (quick and full), scripts/ci.sh and the required
# "Test (ubuntu-latest)" job of .github/workflows/ci.yml; doctor.sh fails if that wiring
# disappears. Fixtures and manual runs are not a substitute for this lane.
#
# Fail-closed rules (a component is NOT passing unless every one holds):
#   * exit code 0;
#   * its own summary line was found and reports a count greater than zero and no failures;
#   * the count is at or above the tripwire floor below (a silently deleted test fails the lane;
#     raising a floor is normal, lowering one needs review);
#   * nothing was skipped or "not run" (review_gate may only tolerate HARNESS_LANE_MAX_NOT_RUN
#     platform-capability gaps, default 0).
# jsonschema must be importable: the present half of the "jsonschema present == absent" claim
# is exercised for real, never skipped.
#
# Output: one "==> [offline-lane]" header per component, then a summary and a final line
#   OFFLINE_LANE: PASS components=12 checks=<n>  (exit 0)
#   OFFLINE_LANE: FAIL ...                         (exit 1)
# Exit 2 is reserved for usage errors.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
if [ -z "$REPO_ROOT" ]; then
  echo "OFFLINE_LANE: FAIL reason=repo-root-unresolved" >&2
  exit 1
fi
cd "$REPO_ROOT" || exit 1

if [ "$#" -gt 0 ]; then
  echo "Usage: bash docs/harness/bin/run-offline-lane.sh   (no arguments)" >&2
  exit 2
fi

MAX_NOT_RUN="${HARNESS_LANE_MAX_NOT_RUN:-0}"
case "$MAX_NOT_RUN" in
  '' | *[!0-9]*) echo "OFFLINE_LANE: FAIL reason=invalid-HARNESS_LANE_MAX_NOT_RUN" >&2; exit 1 ;;
esac

# Tripwire floors (current counts: unittest 76, self-test 18, fixtures 39, live-state 40,
# review-gate 39 tests, lane contract 19, sandbox adapter 54, context budget 49, runner 75,
# merge gate 47, retention 21, standing checks 52; every floor is the exact count so a silently
# shadowed or dropped test trips the lane; docs/harness/tests/test_offline_lane.py mirrors them).
FLOOR_VALIDATOR_UNIT=76
FLOOR_VALIDATOR_SELF=18
FLOOR_FIXTURES=39
FLOOR_LIVE_STATE=40
FLOOR_REVIEW_GATE=39
FLOOR_LANE_CONTRACT=19
FLOOR_SANDBOX_UNIT=54
FLOOR_CONTEXT_BUDGET=49
FLOOR_RUNNER_UNIT=75
FLOOR_MERGE_GATE=47
FLOOR_RETENTION=21
FLOOR_STANDING_CHECKS=52

LANE_TMP="$(mktemp -d)"
trap 'rm -rf "$LANE_TMP"' EXIT

FAILED_COMPONENTS=()
SUMMARY_ROWS=()
TOTAL_CHECKS=0
COMPONENTS_RUN=0

fail_lane() {
  echo "OFFLINE_LANE: FAIL reason=$1" >&2
  exit 1
}

command -v python3 >/dev/null 2>&1 || fail_lane "python3-missing"
command -v git >/dev/null 2>&1 || fail_lane "git-missing"
if ! python3 -c 'import jsonschema' >/dev/null 2>&1; then
  echo "jsonschema is required so the 'present' mode is really exercised (pip install jsonschema, or apt install python3-jsonschema)." >&2
  fail_lane "jsonschema-missing"
fi

# --- parsers: print the check count on success; return non-zero with a reason on stderr ----------

parse_validator_unit() {
  local log="$1" n
  n="$(sed -n 's/^Ran \([0-9][0-9]*\) tests\{0,1\} in .*/\1/p' "$log" | tail -1)"
  [ -n "$n" ] && [ "$n" -gt 0 ] || { echo "no 'Ran N tests' line with N>0" >&2; return 1; }
  [ "$(grep -c '^OK$' "$log")" -eq 1 ] || { echo "unittest did not end with a plain OK (skips, expected failures or failures are not allowed)" >&2; return 1; }
  printf '%s' "$n"
}

parse_validator_self() {
  local log="$1" line passed failed
  line="$(grep '^SELF_TEST_RESULT: ' "$log" | tail -1)"
  passed="$(printf '%s' "$line" | sed -n 's/.*passed=\([0-9][0-9]*\).*/\1/p')"
  failed="$(printf '%s' "$line" | sed -n 's/.*failed=\([0-9][0-9]*\).*/\1/p')"
  [ -n "$passed" ] && [ "$passed" -gt 0 ] && [ "$failed" = "0" ] || { echo "self-test summary missing, empty or failing: '$line'" >&2; return 1; }
  printf '%s' "$passed"
}

parse_fixtures() {
  local log="$1" passed failed
  passed="$(sed -n 's/^Passed: \([0-9][0-9]*\)$/\1/p' "$log" | tail -1)"
  failed="$(sed -n 's/^Failed: \([0-9][0-9]*\)$/\1/p' "$log" | tail -1)"
  [ -n "$passed" ] && [ "$passed" -gt 0 ] && [ "$failed" = "0" ] || { echo "fixtures summary missing, empty or failing (passed='$passed' failed='$failed')" >&2; return 1; }
  grep -q '^ALL TESTS PASSED$' "$log" || { echo "missing 'ALL TESTS PASSED'" >&2; return 1; }
  printf '%s' "$passed"
}

parse_live_state() {
  local log="$1" n
  n="$(sed -n 's/^PASS check-live-state regression suite (assertions: \([0-9][0-9]*\))$/\1/p' "$log" | tail -1)"
  [ -n "$n" ] && [ "$n" -gt 0 ] || { echo "no PASS summary with assertions>0" >&2; return 1; }
  printf '%s' "$n"
}

parse_review_gate() {
  local log="$1" line tests passed failed not_run
  line="$(grep '^Tests: ' "$log" | tail -1)"
  tests="$(printf '%s' "$line" | sed -n 's/^Tests: \([0-9][0-9]*\) .*/\1/p')"
  passed="$(printf '%s' "$line" | sed -n 's/.*Assertions passed: \([0-9][0-9]*\).*/\1/p')"
  failed="$(printf '%s' "$line" | sed -n 's/.*Assertions failed: \([0-9][0-9]*\).*/\1/p')"
  not_run="$(printf '%s' "$line" | sed -n 's/.*Not run: \([0-9][0-9]*\).*/\1/p')"
  [ -n "$tests" ] && [ "$tests" -gt 0 ] && [ -n "$passed" ] && [ "$passed" -gt 0 ] && [ "$failed" = "0" ] || {
    echo "review-gate summary missing, empty or failing: '$line'" >&2
    return 1
  }
  [ -n "$not_run" ] && [ "$not_run" -le "$MAX_NOT_RUN" ] || {
    echo "review-gate reports $not_run NOT RUN check(s); the lane tolerates at most $MAX_NOT_RUN (not run is never pass)" >&2
    return 1
  }
  printf '%s' "$tests"
}

# --- runner ------------------------------------------------------------------------------------

run_component() {
  local name="$1" floor="$2" parser="$3"
  shift 3
  local log="$LANE_TMP/$name.log" rc=0 count="" reason=""
  COMPONENTS_RUN=$((COMPONENTS_RUN + 1))
  echo "==> [offline-lane] $name"
  "$@" >"$log" 2>&1 || rc=$?
  if [ "$rc" -ne 0 ]; then
    reason="exit-$rc"
    echo "FAIL: $name exited $rc" >&2
    tail -n 60 "$log" >&2
  elif ! count="$("$parser" "$log" 2>"$LANE_TMP/$name.reason")"; then
    reason="$(tr '\n' ' ' <"$LANE_TMP/$name.reason")"
    echo "FAIL: $name: $reason" >&2
    tail -n 30 "$log" >&2
  elif [ "$count" -lt "$floor" ]; then
    reason="count-$count-below-floor-$floor"
    echo "FAIL: $name reported $count checks, below the tripwire floor $floor (tests deleted or not discovered?)" >&2
  fi
  if [ -n "$reason" ]; then
    FAILED_COMPONENTS+=("$name")
    SUMMARY_ROWS+=("  FAIL  $name  ($reason)")
    return 0
  fi
  TOTAL_CHECKS=$((TOTAL_CHECKS + count))
  SUMMARY_ROWS+=("  PASS  $name  checks=$count (floor $floor)")
}

run_component validator_unit "$FLOOR_VALIDATOR_UNIT" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py' -v
run_component validator_self "$FLOOR_VALIDATOR_SELF" parse_validator_self \
  python3 docs/harness/bin/validate-evidence.py --self-test
run_component fixtures "$FLOOR_FIXTURES" parse_fixtures \
  bash docs/harness/bin/test-fixtures.sh
run_component live_state "$FLOOR_LIVE_STATE" parse_live_state \
  bash docs/harness/bin/test-check-live-state.sh
run_component review_gate "$FLOOR_REVIEW_GATE" parse_review_gate \
  bash docs/harness/bin/test-review-gate.sh
run_component lane_contract "$FLOOR_LANE_CONTRACT" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_offline_lane.py' -v
run_component sandbox_unit "$FLOOR_SANDBOX_UNIT" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py' -v
run_component context_budget "$FLOOR_CONTEXT_BUDGET" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_context_budget.py' -v
run_component runner_unit "$FLOOR_RUNNER_UNIT" parse_validator_unit \
  python3 -m unittest -v docs/harness/tests/test_runner.py docs/harness/tests/test_scope.py docs/harness/tests/test_evidence_integrity.py
run_component merge_gate "$FLOOR_MERGE_GATE" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_merge_gate.py' -v
run_component retention "$FLOOR_RETENTION" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_retention.py' -v
run_component standing_checks "$FLOOR_STANDING_CHECKS" parse_validator_unit \
  python3 -m unittest discover -s docs/harness/tests -p 'test_standing_checks.py' -v

echo
echo "=== offline lane summary ==="
printf '%s\n' "${SUMMARY_ROWS[@]}"

if [ "${#FAILED_COMPONENTS[@]}" -gt 0 ]; then
  echo "OFFLINE_LANE: FAIL components_failed=${#FAILED_COMPONENTS[*]} names=$(IFS=,; echo "${FAILED_COMPONENTS[*]}")" >&2
  exit 1
fi
if [ "$COMPONENTS_RUN" -ne 12 ] || [ "$TOTAL_CHECKS" -le 0 ]; then
  fail_lane "unexpected-component-count-$COMPONENTS_RUN-checks-$TOTAL_CHECKS"
fi
echo "OFFLINE_LANE: PASS components=$COMPONENTS_RUN checks=$TOTAL_CHECKS"
