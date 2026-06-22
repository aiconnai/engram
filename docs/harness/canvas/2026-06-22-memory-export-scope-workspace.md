# Review Canvas: memory-export-scope-workspace

Date: 2026-06-22
Owner: Codex
Scope: Recover the useful `memory_export` workspace/scope fix from an old
stash while leaving stale aggregate stash content behind.

## Trigger

| Trigger | Evidence |
|---|---|
| MCP tool behavior change | `memory_export` now honors `workspace` and rejects unsupported embedding export requests explicitly. |
| Storage serialization contract | Exported memories now carry `scope_type` and `scope_id`; import restores them. |
| Stash recovery | The source was an old aggregate stash with unrelated stale changes, so only the narrow behavior fix is carried forward. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Apply the whole stash | Rejected | The stash mixes stale generated artifacts, old storage-query split work, and unrelated docs/config. |
| Drop the stash entirely | Rejected | `memory_export` still ignored documented params on current `main`. |
| Recover only export/import behavior | Accepted | Smallest useful change with focused storage tests and regenerated MCP reference. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `memory_export` with no workspace | Same query shape, explicit `ListOptions` construction | Scope fields add two serialized fields per memory | Existing all-memory export behavior preserved. |
| `memory_export` with workspace | Narrows list query by normalized workspace | Smaller output | Invalid workspace returns structured handler error through existing path. |
| `memory_import` | Same per-memory loop | No persistent schema change | Scope is parsed from export payload before `create_memory`. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Uppercase workspace input should match normalized stored workspace | Unit test exports with `Some("TEAM")` and returns only `team`. |
| Non-global exported scope should survive import | Unit test imports a `session` scoped memory and checks DB scope columns. |
| Scoped payload missing `scope_id` should not silently become global | Unit test expects failed import with `missing scope_id`. |
| Duplicate import with `skip_duplicates=true` should count as skipped | Unit test imports the same payload twice and checks `skipped=1`. |

## Breakage Risk

| Risk | Impact | Mitigation | Rollback | Verification |
|---|---|---|---|---|
| Existing export consumers ignore new fields | Low | Fields are additive and serde defaults keep old import payloads valid | Revert export field additions | Focused storage tests. |
| `include_embeddings=true` callers now receive explicit error | Low | The schema now marks it reserved/unsupported instead of promising behavior | Revert handler/schema change | MCP reference regenerated and checked. |
| Scope parsing rejects malformed export payloads | Medium | Malformed scoped payloads fail visibly instead of importing under wrong scope | Revert scope validation block | Missing `scope_id` regression test. |

## Decision

Proceed.

Reason: The current MCP schema advertised options that were ignored or
misleading, and export/import lost scope information. The recovered change is
small, testable, and separates the real fix from stale aggregate stash content.
