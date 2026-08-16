#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CI_ALLOW_OPTIONAL_TEST_FAILURES="${CI_ALLOW_OPTIONAL_TEST_FAILURES:-0}"
CI_RUN_BACKEND_SMOKE="${CI_RUN_BACKEND_SMOKE:-0}"
CI_RUN_FULL_FEATURES="${CI_RUN_FULL_FEATURES:-0}"

run_optional() {
  if [[ "${CI_ALLOW_OPTIONAL_TEST_FAILURES}" == "1" ]]; then
    "$@" || true
  else
    "$@"
  fi
}

# Use the required PR feature list unless caller explicitly provides one.
if [[ -z "${CI_REQUIRED_FEATURES:-}" ]]; then
  source "$SCRIPT_DIR/ci-required-features.env"
fi

: "${CI_REQUIRED_FEATURES:?CI_REQUIRED_FEATURES must be set or defined in $SCRIPT_DIR/ci-required-features.env}"

echo "==> [1/5] Format"
cargo fmt --all -- --check

echo "==> [2/5] Clippy (required PR features)"
cargo clippy --all-targets --no-default-features --features "$CI_REQUIRED_FEATURES" -- -D warnings

echo "==> [3/5] Core tests (lib + integration, matching required GitHub CI job)"
# Mirrors the required "Test (ubuntu-latest)" job as closely as practical for local work.
cargo test --profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --lib --tests

if [[ "$CI_RUN_FULL_FEATURES" == "1" ]]; then
  echo "==> Optional full feature checks"
  run_optional cargo clippy --all-targets --all-features -- -D warnings
  run_optional cargo test --profile ci --all-features --lib --tests
fi

if [[ "$CI_RUN_BACKEND_SMOKE" == "1" ]]; then
  echo "==> Optional backend smoke tests"
  run_optional cargo test --profile ci --no-default-features --features local-embeddings --lib embedding::onnx
  run_optional cargo test --profile ci --no-default-features --features openai,neural-rerank --lib search::neural_rerank
fi

# Binary unit tests (required in the GitHub job)
cargo test --profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --bin engram-server
cargo test --profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --bin engram-watcher

echo "==> [4/5] WASM crate"
if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
  echo "wasm32-unknown-unknown target is required for CI parity." >&2
  echo "Install it with: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi
cargo check -p engram-wasm --all-targets
cargo check -p engram-wasm --target wasm32-unknown-unknown

echo "==> [5/5] Documentation + generated MCP reference"
./scripts/generate-mcp-reference.sh --check
python3 "$SCRIPT_DIR/test_check_pdf_worker_packaging.py"
python3 "$SCRIPT_DIR/check-pdf-worker-packaging.py"
RUSTDOCFLAGS="-D warnings" cargo doc --no-default-features --features "$CI_REQUIRED_FEATURES" --no-deps --document-private-items

echo

echo "✅ Required CI gates passed locally."
echo "   This is what should be green on every PR before merging."
echo "   Run with: make ci   or   just ci"

if [[ "$CI_ALLOW_OPTIONAL_TEST_FAILURES" == "1" ]]; then
  echo
  echo "ℹ Optional test failures were allowed during this run."
  echo "  To enforce strict behavior (required for PR parity), unset CI_ALLOW_OPTIONAL_TEST_FAILURES."
fi

if [[ "$CI_RUN_FULL_FEATURES" != "1" ]]; then
  echo
  echo "Optional full feature checks were skipped."
  echo "Run with CI_RUN_FULL_FEATURES=1 ./scripts/ci.sh to include them."
fi

if [[ "$CI_RUN_BACKEND_SMOKE" != "1" ]]; then
  echo
  echo "Optional backend smoke tests were skipped."
  echo "Run with CI_RUN_BACKEND_SMOKE=1 ./scripts/ci.sh to include them."
fi
