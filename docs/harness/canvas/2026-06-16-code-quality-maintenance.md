# Review Canvas: code-quality-maintenance

Date: 2026-06-16
Owner: Codex
Scope: Fix reproduced SDK test failures and remove verified unused dependency/target surface from the cleanup report.

## Trigger

| Trigger | Evidence |
|---|---|
| More than 200 changed lines | TypeScript SDK tests were rewritten around the current public object-style API. |
| SDK public contract | Python `filter_` keyword, TypeScript option fields, and JSON-RPC request IDs are externally visible. |
| Dependency manifest cleanup | Root `Cargo.toml`, `Cargo.lock`, and `engram-wasm/Cargo.toml` changed. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Apply every finding in the attached report | Rejected | Medium-confidence dead modules touch public exports and need a separate compatibility review. |
| Fix only tests without SDK changes | Rejected | Python `close()` and `filter_` were real client bugs; TypeScript lacked MCP-supported filter/media URL options. |
| Remove all cargo-machete findings | Rejected | `prost` is required by generated gRPC code behind the `grpc` feature, so it is explicitly ignored instead. |
| Focus on reproduced failures and verified dependency cleanup | Accepted | Keeps the change bounded while making the report's high-confidence failures observable green. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Python `_mcp_call` | Constant-time closed-client guard | None | Avoids using a closed client and keeps normal request path unchanged. |
| TypeScript `mcpCall` | One integer increment per request | One number field per client | Enables distinct JSON-RPC request IDs. |
| Cargo dependency graph | Build resolution shrinks | Lockfile shrinks | Removed unused direct dependencies and dev-dependencies. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Calling `close()` twice or after `_client = None` | Python SDK test suite covers both cases. |
| Using advanced filters in SDK calls | Python and TypeScript tests assert outgoing MCP `filter` payloads. |
| gRPC feature still needs generated `prost` types | `cargo check -p engram-core --features grpc --all-targets`. |
| WASM crate still builds after removing unused wasm test dependency | Native and `wasm32-unknown-unknown` `cargo check` for `engram-wasm`. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Python callers using `filter=` keyword | Keyword call breaks | Project policy already documents `filter_`; tests lock `filter_` mapping. | `uv run --with pytest-asyncio pytest`. |
| TypeScript options drift from README | SDK docs become misleading | README option lists updated with added fields. | `npm run type-check` and `npm test`. |
| Removing dummy binary surprises users | `cargo run --bin engram-core` no longer prints `Hello, world!` | Actual entrypoints remain `engram-server`, `engram-cli`, `engram-bench`, watcher, and agent. | `cargo check --all-targets`. |
| Removing unused deps hides feature dependency | gRPC build could fail if `prost` removed | Retained `prost` and added cargo-machete ignore. | `cargo machete` and `cargo check --features grpc`. |

## Decision

Proceed:

The change is bounded to reproduced SDK failures, manifest hygiene with live evidence, and the redundant dummy binary target. Medium-confidence module deletions and helper refactors stay out of scope.
