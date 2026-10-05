#!/usr/bin/env bash
#
# Validate that local CI wrappers and workflow stay aligned on shared CI settings.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKFLOW_FILE="$SCRIPT_DIR/../.github/workflows/ci.yml"
CI_FEATURES_FILE="$SCRIPT_DIR/ci-features.env"

if [[ ! -f "$CI_FEATURES_FILE" ]]; then
  echo "error: missing $CI_FEATURES_FILE"
  exit 1
fi

source "$CI_FEATURES_FILE"

if [[ -z "${CI_FEATURES:-}" ]]; then
  echo "error: CI_FEATURES in $CI_FEATURES_FILE is empty"
  exit 1
fi

printf 'Using CI_FEATURES: %s\n' "$CI_FEATURES"

status=0

check_file() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  if ! grep -qF -- "$pattern" "$file"; then
    echo "error: $message"
    status=1
  fi
}

check_file_regex() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  if ! grep -qE -- "$pattern" "$file"; then
    echo "error: $message"
    status=1
  fi
}

# Required PR lane: scripts/ci.sh and the required GitHub job share one feature list.
check_file_regex "$SCRIPT_DIR/ci.sh" 'source .*/ci-required-features\.env' "scripts/ci.sh is not loading ci-required-features.env"
check_file "$WORKFLOW_FILE" "source scripts/ci-required-features.env" "GitHub workflow is not loading ci-required-features.env"
check_file "$WORKFLOW_FILE" '--features "$CI_REQUIRED_FEATURES"' "GitHub workflow missing expected --features \"\$CI_REQUIRED_FEATURES\" usage"
check_file "Makefile" "CI_REQUIRED_FEATURES :=" "Makefile does not read CI_REQUIRED_FEATURES from ci-required-features.env"
check_file "justfile" "ci_required_features :=" "justfile does not read CI_REQUIRED_FEATURES from ci-required-features.env"
# `make ci` / `just ci` must delegate to scripts/ci.sh (single source of local gates).
check_file "Makefile" "@./scripts/ci.sh" "Makefile ci target does not delegate to scripts/ci.sh"
check_file "justfile" "@./scripts/ci.sh" "justfile ci target does not delegate to scripts/ci.sh"

# Optional full-feature lane (scheduled/extended CI) still shares ci-features.env.
check_file "$WORKFLOW_FILE" "source scripts/ci-features.env" "GitHub workflow is not loading ci-features.env"
check_file "$WORKFLOW_FILE" '--features "$CI_FEATURES"' "GitHub workflow missing expected --features \"\$CI_FEATURES\" usage"

# Historical baseline integrity lane: the full argv of the workflow and scripts/ci.sh is
# validated by executing the contract tests (argv extraction + fake-python3 recording),
# not by grepping for strings. Only the non-recursive, git-independent classes run here.
if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 is required for the historical baseline integrity contract"
  status=1
elif ! python3 "$SCRIPT_DIR/test_check_quality_ci_contract.py" \
  WorkflowArgvContract LocalLaneContract LaneAgreement DocsNaming GateWiring; then
  echo "error: historical baseline integrity contract (--criterion argv, local/CI agreement) failed"
  status=1
fi

if grep -qE '^  CI_FEATURES:' "$WORKFLOW_FILE"; then
  echo "error: workflow still hard-codes CI_FEATURES in top-level env; remove duplication"
  status=1
fi

if [[ $status -ne 0 ]]; then
  exit 1
fi

echo "✅ CI parity checks passed."
