# Review Canvas: mcp-misc-split

Date: 2026-07-08
Owner: Ronaldo + Claude
Scope: Split `src/mcp/handlers/misc.rs` into tool-family submodules per ADR-CLEANUP-20260708-2 row 1, with no behavior changes.

## Trigger

| Trigger | Evidence |
|---|---|
| Oversized production file | `misc.rs` exceeded the production-file budget flagged by cleanup inspection (#130 / ADR-CLEANUP-20260708-2). |
| Mechanical mass move | Handlers relocated across new submodules (`tags`, `import_export`, `maintenance`, `images`, `auto_tag`, `langfuse_sync`, `meilisearch`, `discovery`); mistakes are easy to miss in review. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Split by tool family into `misc/` submodules, re-export via `mod.rs` | Accepted | Matches ADR row 1; keeps call sites and dispatch table unchanged. |
| Rewrite dispatch to route per-module directly | Rejected | Behavior change out of scope; increases review risk. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Tool dispatch | None | None | Pure code move; same function signatures and registration. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Handler dropped or duplicated during move | Diff of `pub fn` inventory old vs new (verified identical, 132 handlers). |
| Feature-gated code (`meilisearch`, `langfuse`) misplaced | `cargo check` with `--features langfuse,meilisearch` plus default build. |
| Reindex thread panics on runtime creation | Replaced `expect` with logged error in `meilisearch.rs` spawn (review finding). |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Semantic drift in mechanical move | High | No logic edits; byte-identical moves except lint fixes | `cargo test --locked` full suite green; clippy `--all-targets` clean |
| Panicking thread on reindex | Medium | Builder-based runtime creation with error log | Post-gate review finding fixed; clippy/tests re-run |

## Decision

Proceed.

Reason: Mechanical split verified by handler-inventory diff, full test suite, clippy, and independent post-gate review (REVIEW_VERDICT: PASS after fixes).
