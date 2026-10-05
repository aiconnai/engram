#!/usr/bin/env bash
# docs/harness/bin/run-runner-smoke.sh
#
# Real-Docker end-to-end smoke of the trusted runner with the FAKE writer (task H4). It is deliberately NOT
# part of run-offline-lane.sh / sensors.sh / the required CI job: those must not depend on Docker.
# The offline runner/scope/evidence tests are the runner_unit lane component instead.
#
# Final line (exactly one of):
#   RUNNER_SMOKE: PASS tests=<n>                  exit 0
#   RUNNER_SMOKE: FAIL reason=...                 exit 1
#   RUNNER_SMOKE: UNAVAILABLE reason=...          exit 3   (docker/daemon/pinned image missing;
#                                                            NOT a pass, never converted to one)
# There is no host fallback: when the sandbox cannot be provided nothing is executed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
cd "$REPO_ROOT" || { echo "RUNNER_SMOKE: FAIL reason=repo-root-unresolved" >&2; exit 1; }

if [ "$#" -gt 0 ]; then
  echo "Usage: bash docs/harness/bin/run-runner-smoke.sh   (no arguments)" >&2
  exit 2
fi

FLOOR_TESTS=4
command -v python3 >/dev/null 2>&1 || { echo "RUNNER_SMOKE: FAIL reason=python3-missing" >&2; exit 1; }

PREFLIGHT="$(python3 docs/harness/tests/test_runner_smoke.py --preflight 2>&1)"
PREFLIGHT_RC=$?
if [ "$PREFLIGHT_RC" -eq 3 ]; then
  echo "$PREFLIGHT"
  echo "RUNNER_SMOKE: UNAVAILABLE reason=${PREFLIGHT#*reason=}"
  exit 3
elif [ "$PREFLIGHT_RC" -ne 0 ]; then
  echo "$PREFLIGHT" >&2
  echo "RUNNER_SMOKE: FAIL reason=preflight-error" >&2
  exit 1
fi

LOG="$(mktemp)"
trap 'rm -f "$LOG"' EXIT
python3 -m unittest discover -s docs/harness/tests -p 'test_runner_smoke.py' -v >"$LOG" 2>&1
RC=$?
N="$(sed -n 's/^Ran \([0-9][0-9]*\) tests\{0,1\} in .*/\1/p' "$LOG" | tail -1)"
if [ "$RC" -ne 0 ]; then
  tail -n 60 "$LOG" >&2
  echo "RUNNER_SMOKE: FAIL reason=exit-$RC" >&2
  exit 1
fi
if [ -z "$N" ] || [ "$N" -lt "$FLOOR_TESTS" ] || [ "$(grep -c '^OK$' "$LOG")" -ne 1 ]; then
  tail -n 30 "$LOG" >&2
  echo "RUNNER_SMOKE: FAIL reason=tests-${N:-none}-below-floor-$FLOOR_TESTS-or-skipped" >&2
  exit 1
fi
echo "RUNNER_SMOKE: PASS tests=$N"
