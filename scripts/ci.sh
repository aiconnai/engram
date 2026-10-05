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

# Historical baseline integrity lane (retrieval floors + committed Criterion snapshot).
# Mirrors the "Historical baseline integrity" steps of the required GitHub job with the
# same argv. It does NOT measure candidate performance (Q7/Q1b). A missing python3,
# input file or git history is a failure, never a skip.
run_quality_budget_lane() {
  echo "==> Historical baseline integrity (committed floors and Criterion snapshot; not candidate performance)"
  if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required for the historical baseline integrity lane." >&2
    return 1
  fi
  (
    cd "$SCRIPT_DIR/.."
    python3 scripts/check-quality-budgets.py \
      --budgets docs/quality/budgets.json \
      --retrieval tests/fixtures/retrieval_quality/baseline.json \
      --criterion benches/results/benchmark_results.txt
    echo "==> Historical baseline integrity self-test (not candidate performance; degraded inputs must be rejected)"
    python3 scripts/check-quality-budgets.py \
      --budgets docs/quality/budgets.json \
      --retrieval tests/fixtures/retrieval_quality/baseline.json \
      --criterion benches/results/benchmark_results.txt \
      --self-test-degraded
  )
}

case "${1:-}" in
  "") ;;
  quality-budgets)
    run_quality_budget_lane
    exit $?
    ;;
  *)
    echo "usage: scripts/ci.sh [quality-budgets]" >&2
    exit 2
    ;;
esac

# Use the required PR feature list unless caller explicitly provides one.
if [[ -z "${CI_REQUIRED_FEATURES:-}" ]]; then
  source "$SCRIPT_DIR/ci-required-features.env"
fi

: "${CI_REQUIRED_FEATURES:?CI_REQUIRED_FEATURES must be set or defined in $SCRIPT_DIR/ci-required-features.env}"

echo "==> [1/7] Format"
cargo fmt --all -- --check

echo "==> [2/7] Clippy (required PR features)"
cargo clippy --all-targets --no-default-features --features "$CI_REQUIRED_FEATURES" -- -D warnings

echo "==> [3/7] Historical baseline integrity lane (mirrors required GitHub job step order)"
run_quality_budget_lane
# Contract suite for the lane (argv of workflow and this script, parity). It invokes
# `ci.sh quality-budgets` (lane only), so it must stay outside run_quality_budget_lane.
python3 "$SCRIPT_DIR/test_check_quality_ci_contract.py"
# Mandatory offline harness lane (H2): validator, fixtures, live-state and review-gate regressions.
# Same runner as the required GitHub job and sensors.sh; zero tests or skips fail it.
bash "$SCRIPT_DIR/../docs/harness/bin/run-offline-lane.sh"

echo "==> [4/7] Core tests (lib + integration, matching required GitHub CI job)"
# Mirrors the required "Test (ubuntu-latest)" job as closely as practical for local work.
if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --cargo-profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --lib --tests --bin engram-server --bin engram-watcher
else
  cargo test --profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --lib --tests
fi

if [[ "$CI_RUN_FULL_FEATURES" == "1" ]]; then
  echo "==> Optional full feature checks"
  run_optional cargo clippy --all-targets --all-features -- -D warnings
  if command -v cargo-nextest >/dev/null 2>&1; then
    run_optional cargo nextest run --cargo-profile ci --all-features --lib --tests
  else
    run_optional cargo test --profile ci --all-features --lib --tests
  fi
fi

if [[ "$CI_RUN_BACKEND_SMOKE" == "1" ]]; then
  echo "==> Optional backend smoke tests"
  run_optional cargo test --profile ci --no-default-features --features local-embeddings --lib embedding::onnx
  run_optional cargo test --profile ci --no-default-features --features openai,neural-rerank --lib search::neural_rerank
fi

if ! command -v cargo-nextest >/dev/null 2>&1; then
  # Binary unit tests (already covered in nextest invocation above when nextest is present)
  cargo test --profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --bin engram-server
  cargo test --profile ci --no-default-features --features "$CI_REQUIRED_FEATURES" --bin engram-watcher
fi

echo "==> [5/7] Fast offline Python contract tests (blocking; mirror the required Test job)"
# Q1b: the candidate quality runner and its consumption rule. Only these unit tests are
# blocking; the runner itself is report-only (.github/workflows/quality-candidate.yml).
# Q4: SDK contract drift ratchet. Its unit tests import the Python SDK (needs httpx); the
# TypeScript sweep is static. `--only python` runs in python-sdk-live.yml.
(
  cd "$SCRIPT_DIR/.."
  python3 -m unittest scripts/test_run_quality_candidate.py
  python3 -m unittest scripts/test_quality_candidate_consume.py
  python3 -m unittest scripts/test_check_sdk_contract_alignment.py
  python3 scripts/check-sdk-contract-alignment.py --only typescript
)

echo "==> [6/7] WASM crate"
if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
  echo "wasm32-unknown-unknown target is required for CI parity." >&2
  echo "Install it with: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi
cargo check -p engram-wasm --all-targets
cargo check -p engram-wasm --target wasm32-unknown-unknown

echo "==> [7/7] Documentation + generated MCP reference"
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
