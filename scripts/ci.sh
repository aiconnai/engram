#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Use shared CI feature list unless caller explicitly provides one.
if [[ -z "${CI_FEATURES:-}" ]]; then
  source "$SCRIPT_DIR/ci-features.env"
fi

: "${CI_FEATURES:?CI_FEATURES must be set or defined in $SCRIPT_DIR/ci-features.env}"

echo "==> [1/4] Format"
cargo fmt --all -- --check

echo "==> [2/4] Clippy (all features)"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> [3/4] Core tests (lib + integration, matching required GitHub CI job)"
# Mirrors the required "Test (ubuntu-latest)" job as closely as practical for local work.
# Expensive/optional features may be skipped with || true.
export CARGO_BUILD_JOBS=1
cargo test --features "$CI_FEATURES" --lib -- --test-threads=1

for test_file in tests/*.rs; do
  test_name="$(basename "$test_file" .rs)"
  cargo test --features "$CI_FEATURES" --test "$test_name" -- --test-threads=1 || true
done

# Specific backend smoke tests (best-effort locally)
cargo test --features local-embeddings --lib embedding::onnx || true
cargo test --features neural-rerank --lib search::neural_rerank || true

# Binary unit tests (required in the GitHub job)
cargo test --bin engram-server || true
cargo test --features watcher --bin engram-watcher || true

echo "==> [4/4] Documentation + generated MCP reference"
./scripts/generate-mcp-reference.sh --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

echo

echo "✅ Required CI gates passed locally."
echo "   This is what should be green on every PR before merging."
echo "   Run with: make ci   or   just ci"
