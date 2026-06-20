# Review Canvas: hooks-contracts

Date: 2026-06-20
Owner: Codex
Scope: Align lifecycle hook wiring and PostToolUse behavior with implemented contracts.

## Trigger

| Trigger | Evidence |
|---|---|
| Hooks changed | `src/bin/server.rs` wires `LifecycleHook::Stop`; `src/hooks/post_tool_use.rs` changes PostToolUse side effects. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Implement automatic memory creation from tool output | Rejected | Hidden memory writes would expand hook side effects without a reviewed contract. |
| Wire `StopHandler` and document PostToolUse as policy reinforcement only | Accepted | Matches existing implemented behavior and removes fake-success logging. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `tools/call` post-hook | Unchanged O(number of returned IDs) | Unchanged | PostToolUse still only scans metadata and optionally reinforces policy. |
| Stop lifecycle hook | Constant | None | Uses existing handler, which currently returns `Continue`. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Tool output includes content but no explicit memory ID | Unit test asserts PostToolUse does not create synthetic memories. |
| Old placeholder branch is reintroduced | Integration test scans the local hook source for the legacy fake-success markers. |
| Stop hook enabled in server wiring | Server hook wiring test triggers `LifecycleHook::Stop` and expects `Continue`. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Callers relied on `auto_memory` field | Compile-time failure exposes the mismatch; field only guarded a TODO branch. | Remove misleading field instead of preserving no-op behavior, and record the feature-gated public API cleanup in `CHANGELOG.md`. | `cargo test --features hooks post_tool_use`. |
| Stop hook wiring changes runtime path | Low; handler returns `Continue`. | Regression test covers manager dispatch. | `cargo test --features hooks test_hook_wiring`. |

## Decision

Proceed:

The change narrows hook behavior to the implemented contract and adds tests for the two previously misleading surfaces.
