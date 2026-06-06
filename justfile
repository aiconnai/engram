set shell := ["bash", "-cu"]

ci_features := `sed -n 's/^CI_FEATURES=//p' scripts/ci-features.env`
ci_required_features := `sed -n 's/^CI_REQUIRED_FEATURES=//p' scripts/ci-required-features.env`

alias c := ci

ci:
    @./scripts/ci.sh

pre-commit: fmt clippy

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" -- -D warnings

test:
    # Mirrors the core of the required ubuntu test job (lib + integration loop)
    CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --lib --tests -- --test-threads=1
    CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --bin engram-server
    CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --bin engram-watcher

full-feature-check:
    cargo clippy --all-targets --all-features -- -D warnings
    CARGO_BUILD_JOBS=1 cargo test --profile ci --all-features --lib --tests -- --test-threads=1

backend-smoke:
    CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features local-embeddings --lib embedding::onnx
    CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features openai,neural-rerank --lib search::neural_rerank

docs:
    ./scripts/generate-mcp-reference.sh --check
    RUSTDOCFLAGS="-D warnings" cargo doc --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --no-deps --document-private-items
