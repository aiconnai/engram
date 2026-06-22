# Review Canvas: ENGRA-150 query-layer lifecycle updates

Date: 2026-06-22
Owner: Codex
Scope: Route Dream and lifecycle handler memory lifecycle transitions through storage query-layer update bookkeeping.

## Trigger

| Trigger | Evidence |
|---|---|
| MCP handler mutation change | `src/mcp/handlers/dream.rs` and `src/mcp/handlers/lifecycle.rs` no longer issue raw `UPDATE memories` lifecycle writes. |
| Storage mutation invariant | Lifecycle transitions now create memory version rows and memory update events like other canonical updates. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Add `lifecycle_state` to public `UpdateMemoryInput` | Rejected | It would expand the public Rust/MCP update input shape for an internal cleanup. |
| Add dedicated query-layer lifecycle transition helper | Accepted | Keeps handler behavior scoped while centralizing version/event/sync bookkeeping. |
| Keep handler SQL and only add tests | Rejected | Would leave the reviewed raw mutation path in place. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `lifecycle_run` transition loop | One extra read/version/event write per transitioned memory | One memory version row and one memory event per transition | Matches update-layer bookkeeping expectations; lifecycle dry-run remains unchanged. |
| `dream_candidate_apply` expire path | One extra read/version/event write per expired target | One memory version row and one memory event per target | Only runs after explicit candidate acceptance and `confirm=true`. |
| `memory_set_lifecycle` | One extra read/version/event write | One memory version row and one memory event | Response shape for missing IDs is preserved. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Expired-but-current rows should keep old lifecycle setter behavior | Helper reads `valid_to IS NULL` rows without adding an `expires_at` filter. |
| Missing or soft-deleted memory in `memory_set_lifecycle` | Preserve the existing `{"error":"Memory not found"}` response. |
| Dream expire target became inactive before apply | Existing feature-gated dream regression remains green. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Extra version/event rows change counts in downstream tests | Medium | Added focused assertions where lifecycle transitions now intentionally use update bookkeeping. | `cargo test -p engram-core --lib lifecycle_tests --locked` |
| Public MCP schema drift | Low | Did not add a new `memory_update` field or change tool schemas. | No MCP reference regeneration required. |
| Handler-local raw update remains | Medium | Grep targeted handlers for `UPDATE memories`. | `grep "UPDATE memories" src/mcp/handlers/dream.rs src/mcp/handlers/lifecycle.rs` returns no matches. |

## Decision

Proceed.

Reason: The change removes the reviewed handler-local lifecycle writes while preserving the existing handler contracts and adding update-layer bookkeeping evidence.
