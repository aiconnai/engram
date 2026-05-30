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

.PHONY: ci
ci:
	@./scripts/ci.sh

.PHONY: fmt
fmt:
	cargo fmt --all -- --check

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: test
test:
	CARGO_BUILD_JOBS=1 cargo test --features $(CI_FEATURES) --lib -- --test-threads=1
	@for test_file in tests/*.rs; do \
		test_name="$$(basename "$$test_file" .rs)" ; \
		CARGO_BUILD_JOBS=1 cargo test --features $(CI_FEATURES) --test "$$test_name" -- --test-threads=1; \
	done
	CARGO_BUILD_JOBS=1 cargo test --features local-embeddings --lib embedding::onnx
	CARGO_BUILD_JOBS=1 cargo test --features neural-rerank --lib search::neural_rerank
	CARGO_BUILD_JOBS=1 cargo test --bin engram-server
	CARGO_BUILD_JOBS=1 cargo test --features watcher --bin engram-watcher

.PHONY: docs
docs:
	./scripts/generate-mcp-reference.sh --check
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items
