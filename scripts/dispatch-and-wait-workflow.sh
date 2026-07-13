#!/usr/bin/env bash
set -euo pipefail

die() { echo "dispatch-and-wait-workflow: $*" >&2; exit 1; }

assert_sha() {
  local expected=$1 actual=$2
  [[ "$expected" =~ ^[0-9a-f]{40}$ ]] || die "expected SHA must be a full lowercase SHA"
  [[ "$actual" == "$expected" ]] || die "workflow head SHA mismatch"
}

self_test() {
  assert_sha "$(printf 'a%.0s' {1..40})" "$(printf 'a%.0s' {1..40})"
  echo "dispatch-and-wait-workflow self-test: PASS"
}

self_test_sha_mismatch() {
  if (assert_sha "$(printf 'a%.0s' {1..40})" "$(printf 'b%.0s' {1..40})") >/dev/null 2>&1; then
    die "SHA mismatch self-test accepted a stale run"
  fi
  echo "dispatch-and-wait-workflow SHA mismatch self-test: PASS"
}

workflow=''
ref=''
mode=dispatch
timeout=1800
fields=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workflow) workflow=${2:-}; mode=dispatch; shift 2 ;;
    --existing-workflow) workflow=${2:-}; mode=existing; shift 2 ;;
    --ref) ref=${2:-}; shift 2 ;;
    --field) fields+=("${2:-}"); shift 2 ;;
    --timeout) timeout=${2:-}; shift 2 ;;
    --self-test) self_test; exit 0 ;;
    --self-test-sha-mismatch) self_test_sha_mismatch; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ -n "$workflow" && -n "$ref" ]] || die "--workflow/--existing-workflow and --ref are required"
[[ "$timeout" =~ ^[1-9][0-9]*$ ]] || die "timeout must be positive seconds"
command -v gh >/dev/null || die "gh is required"

expected_sha=
for field in "${fields[@]}"; do
  [[ "$field" == *=* ]] || die "--field must be key=value"
  if [[ "$field" == sha=* ]]; then expected_sha=${field#sha=}; fi
done
if [[ -z "$expected_sha" ]]; then
  expected_sha="$(git rev-parse "${ref}^{commit}")"
fi
assert_sha "$expected_sha" "$expected_sha"

before="$(mktemp)"
trap 'rm -f "$before"' EXIT
gh run list --workflow "$workflow" --limit 100 --json databaseId --jq '.[].databaseId' > "$before"
if [[ "$mode" == dispatch ]]; then
  command=(gh workflow run "$workflow" --ref "$ref")
  for field in "${fields[@]}"; do command+=(-f "$field"); done
  "${command[@]}" >&2
fi

deadline=$((SECONDS + timeout))
run_id=
while (( SECONDS < deadline )); do
  runs="$(gh run list --workflow "$workflow" --limit 100 --json databaseId,headSha,headBranch,event,status,conclusion)"
  run_id="$(RUNS="$runs" BEFORE="$before" EXPECTED_SHA="$expected_sha" MODE="$mode" python3 - <<'PY'
import json, os
before = set(open(os.environ["BEFORE"]).read().split())
for run in json.loads(os.environ["RUNS"]):
    if run["headSha"] != os.environ["EXPECTED_SHA"]:
        continue
    if os.environ["MODE"] == "dispatch" and str(run["databaseId"]) in before:
        continue
    print(run["databaseId"])
    break
PY
)"
  [[ -n "$run_id" ]] && break
  sleep 3
done
[[ -n "$run_id" ]] || die "timed out waiting for workflow run"

actual_sha="$(gh run view "$run_id" --json headSha --jq .headSha)"
assert_sha "$expected_sha" "$actual_sha"
gh run watch "$run_id" --exit-status >&2
conclusion="$(gh run view "$run_id" --json conclusion --jq .conclusion)"
[[ "$conclusion" == success ]] || die "workflow concluded: $conclusion"
printf '%s\n' "$run_id"
