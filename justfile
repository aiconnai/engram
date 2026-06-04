set shell := ["bash", "-cu"]

ci_features := `sed -n 's/^CI_FEATURES=//p' scripts/ci-features.env`

alias c := ci

ci:
    @./scripts/ci.sh

pre-commit: fmt clippy

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    # Mirrors the core of the required ubuntu test job (lib + integration loop)
    CARGO_BUILD_JOBS=1 cargo test --features "${CI_FEATURES:-{{ci_features}}}" --lib -- --test-threads=1
    for test_file in tests/*.rs; do
        test_name="$(basename "$test_file" .rs)"
        CARGO_BUILD_JOBS=1 cargo test --features "${CI_FEATURES:-{{ci_features}}}" --test "$test_name" -- --test-threads=1
    done
    CARGO_BUILD_JOBS=1 cargo test --features local-embeddings --lib embedding::onnx
    CARGO_BUILD_JOBS=1 cargo test --features neural-rerank --lib search::neural_rerank
    CARGO_BUILD_JOBS=1 cargo test --bin engram-server
    CARGO_BUILD_JOBS=1 cargo test --features watcher --bin engram-watcher

docs:
    ./scripts/generate-mcp-reference.sh --check
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items
