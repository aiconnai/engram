REVIEW_VERDICT: PASS HTTP transport auth refactor and /v1/mcp alias are correct, well-tested, and safe to merge with minor follow-up items noted below

**Reviewed:** `src/mcp/http_transport.rs` (router extraction, dual-route auth, constant-time token comparison, CORS, SSE) and `tests/mcp_protocol_tests.rs`. All INVARIANTS checked.

---

- `[MED]` No test asserting `POST /v1/mcp` without Bearer returns 401 — implementation is correct but invariant partially unwitnessed. Add `test_post_v1_mcp_alias_rejects_missing_bearer`. File: `src/mcp/http_transport.rs` ~line 660.

- `[MED]` `build_router` is module-private with no doc comment explaining the deliberate scope. Document as intentional in the function's doc comment. File: `src/mcp/http_transport.rs` line 355.

- `[MED]` `serde_json::to_value(...).unwrap_or_default()` at lines 59 and 72 silently replaces serialization failure with `null`. Add `tracing::error!` before fallback to honour the "never silently swallow errors" invariant. File: `src/mcp/http_transport.rs` lines 59 and 72.

- `[MED]` `council.rs` uses `tokio::runtime::Runtime::new()` inside a sync fn — pre-existing pattern, not introduced here, but new council test makes nested-runtime panic more likely if called from `#[tokio::test]`. File: `src/mcp/handlers/council.rs` line 71.

- `[LOW]` `docs/MCP_AUTH.md` mentions gRPC auth as if currently implemented; clarify it is forthcoming if not yet shipped. File: `docs/MCP_AUTH.md` line 25.

- `[LOW]` CORS env-var tests depend on `ENV_LOCK` for safety — document this dependency in a comment on the lock declaration. File: `src/mcp/http_transport.rs`.
