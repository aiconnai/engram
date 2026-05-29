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
  if ! rg -q --fixed-strings -- "$pattern" "$file"; then
    echo "error: $message"
    status=1
  fi
}

check_file_regex() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  if ! rg -q --pcre2 "$pattern" "$file"; then
    echo "error: $message"
    status=1
  fi
}

check_file_regex "$SCRIPT_DIR/ci.sh" 'source .*/ci-features\.env' "scripts/ci.sh is not loading ci-features.env"
check_file "$WORKFLOW_FILE" "source scripts/ci-features.env" "GitHub workflow is not loading ci-features.env"
check_file "Makefile" "CI_FEATURES :=" "Makefile does not read CI_FEATURES from ci-features.env"
check_file "justfile" "ci_features :=" "justfile does not read CI_FEATURES from ci-features.env"
check_file "$WORKFLOW_FILE" '--features "$CI_FEATURES"' "GitHub workflow missing expected --features \"$CI_FEATURES\" usage"

if rg -q '^  CI_FEATURES:' "$WORKFLOW_FILE"; then
  echo "error: workflow still hard-codes CI_FEATURES in top-level env; remove duplication"
  status=1
fi

if [[ $status -ne 0 ]]; then
  exit 1
fi

echo "✅ CI parity checks passed."
