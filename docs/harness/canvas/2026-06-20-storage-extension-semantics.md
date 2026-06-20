# Review Canvas: storage-extension-semantics

Date: 2026-06-20
Owner: Codex
Scope: Make storage extension trait placeholder behavior explicit and validate savepoint SQL identifiers.

## Trigger

| Trigger | Evidence |
|---|---|
| Storage behavior changed | `TransactionalBackend`, `CloudSyncBackend`, SQLite savepoints, and Turso sync extension semantics changed. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Keep placeholder success values | Rejected | Fake success can mislead callers into treating unsupported sync and transaction wrappers as real. |
| Add a new public unsupported-operation error variant | Rejected | Existing `Storage`, `Sync`, and `InvalidInput` variants express the failure without widening the public enum. |
| Return explicit errors and validate savepoint names | Accepted | Smallest behaviorally honest change with focused regression coverage. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Savepoint helpers | O(name length) | None | Validation runs before interpolating SQL. |
| Unsupported extension calls | Constant | None | Methods now return explicit errors instead of invoking fake work. |
| SQLite `sync_delta` / `sync_state` | Unchanged | Unchanged | Query-backed read-only sync state remains supported. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Savepoint names with spaces, punctuation, SQL separators, or digit starts | SQLite and Turso tests assert `InvalidInput` before SQL execution. |
| A caller assumes `with_transaction` invokes its closure atomically | SQLite and Turso tests assert the extension wrapper returns `Storage` error and does not call the closure. |
| Placeholder sync methods look successful | SQLite push/pull and Turso sync_delta/sync_state tests assert explicit `Sync` errors. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| External callers depended on fake success placeholders | Behavior changes from success-shaped no-op to explicit error. | Error message points to supported sync paths. | Focused backend tests. |
| Savepoint name validation rejects unusual names | Safer SQL boundary; callers must use simple identifiers. | Validator contract is documented in code and tests. | Invalid-name and valid-name tests. |
| Transaction wrapper callers expected closure execution | Exposes unsupported semantics rather than silently running outside a transaction. | Existing production code mostly uses `Storage::with_transaction`; extension trait now fails loudly. | `cargo test sqlite_backend`, Turso integration tests. |

## Verification

| Command | Result |
|---|---|
| `rtk cargo test sqlite_backend` | PASS, 15 passed. |
| `rtk cargo test --test turso_backend_tests --features turso` | PASS, 6 passed. |
| `rtk cargo clippy --workspace --all-targets --all-features -- -D warnings` | FAIL on unrelated `engram-wasm` warnings outside this task scope. |
| `rtk cargo clippy -p engram-core --all-targets --features turso -- -D warnings` | PASS. |
| `rtk git diff --check` | PASS. |

## Decision

Proceed:

The change removes fake-success storage APIs without changing schema or core storage CRUD paths.
