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
    cargo test --profile ci --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --lib --tests
    cargo test --profile ci --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --bin engram-server
    cargo test --profile ci --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --bin engram-watcher

full-feature-check:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --profile ci --all-features --lib --tests

backend-smoke:
    cargo test --profile ci --no-default-features --features local-embeddings --lib embedding::onnx
    cargo test --profile ci --no-default-features --features openai,neural-rerank --lib search::neural_rerank

docs:
    ./scripts/generate-mcp-reference.sh --check
    RUSTDOCFLAGS="-D warnings" cargo doc --no-default-features --features "${CI_REQUIRED_FEATURES:-{{ci_required_features}}}" --no-deps --document-private-items

version-check:
    python3 scripts/bump-version.py --check
    python3 scripts/check-release-channels.py --matrix docs/releases/channel-matrix.toml

version-refresh-matrix:
    python3 scripts/bump-version.py --refresh-matrix

loop-security:
    @bash scripts/run-agentshield-loop.sh

