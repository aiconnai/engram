# Review Canvas — Markdown portability core

## Intent and alternatives

Approved design: one deep module for Markdown export/import and thin CLI/MCP
adapters. Compared with only removing HandlerContext (retains MCP ownership), or
introducing storage/filesystem traits (no second production implementation), move
workflow ownership and use real SQLite/tempfiles through the typed seam.

## Hot-path complexity

Preserve existing snapshot query, full-batch render and directory-relative IO.
Import keeps bounded file collection, parse-before-transaction, plan-before-write
and one immediate transaction when confirmed. No extra network/provider calls.

## Edge cases

- Foreign and missing IDs remain indistinguishable; check on mutation connection.
- Errors after commit retain unknown outcome, never claim applied=0.
- Empty export preserves MCP error versus public Rust success-zero behavior.
- Symlink/hardlink/ACL/active SQLite protections remain in moved implementation.

## Breakage risk

| Area | Risk | Validation |
|---|---|---|
| Permission env/required mode | Unintended widening | Existing permission/safety suites + typed seam tests |
| Typed result translation | JSON/report drift | Existing adapter and vault tests + golden comparison |
| Filesystem moves | Visibility/FFI behavior drift | Existing Unix safety tests and G1 regressions |
| Imports/reexports | Source compatibility | Public imports unchanged; CI feature build |
| Commit error | False success or rollback claim | Existing post-commit unknown-outcome regression |

No schema, defaults, MCP catalog, SDK contract or gate-policy changes intended.
