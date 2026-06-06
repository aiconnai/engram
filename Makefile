# Lightweight developer targets for Engram
#
# Primary target for contributors:
#   make ci
#
# This runs exactly the same gates that are required to be green
# on every pull request (see .github/workflows/ci.yml and scripts/ci.sh).
#
# Expensive / slow checks (macOS matrix, full property/golden tests,
# coverage, benchmarks, cargo-deny, security audit) live in scheduled
# or manually triggered CI only.
ifeq ($(strip $(CI_FEATURES)),)
CI_FEATURES := $(strip $(shell sed -n 's/^CI_FEATURES=//p' scripts/ci-features.env))
endif
ifeq ($(strip $(CI_REQUIRED_FEATURES)),)
CI_REQUIRED_FEATURES := $(strip $(shell sed -n 's/^CI_REQUIRED_FEATURES=//p' scripts/ci-required-features.env))
endif

.PHONY: ci
ci:
	@./scripts/ci.sh

.PHONY: fmt
fmt:
	cargo fmt --all -- --check

.PHONY: clippy
clippy:
	cargo clippy --all-targets --no-default-features --features $(CI_REQUIRED_FEATURES) -- -D warnings

.PHONY: test
test:
	CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features $(CI_REQUIRED_FEATURES) --lib --tests -- --test-threads=1
	CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features $(CI_REQUIRED_FEATURES) --bin engram-server
	CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features $(CI_REQUIRED_FEATURES) --bin engram-watcher

.PHONY: full-feature-check
full-feature-check:
	cargo clippy --all-targets --all-features -- -D warnings
	CARGO_BUILD_JOBS=1 cargo test --profile ci --all-features --lib --tests -- --test-threads=1

.PHONY: backend-smoke
backend-smoke:
	CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features local-embeddings --lib embedding::onnx
	CARGO_BUILD_JOBS=1 cargo test --profile ci --no-default-features --features openai,neural-rerank --lib search::neural_rerank

.PHONY: docs
docs:
	./scripts/generate-mcp-reference.sh --check
	RUSTDOCFLAGS="-D warnings" cargo doc --no-default-features --features $(CI_REQUIRED_FEATURES) --no-deps --document-private-items
