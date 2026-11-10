PASS The diff makes `memory_consolidate_batch` and `memory_embedding_migrate` report honestly. Tests, docs, SDK comments and the canvas are consistent with each other, and I found no blocking problem. I read the diff and grepped the worktree; I did not run any tests or sensors.

- [LOW] `ConsolidationPolicy.dry_run` is now accepted but ignored (`src/intelligence/auto_consolidate.rs:231,406`).
  - A grep shows no other branch on `policy.dry_run` in `run_consolidation`, so the claim that nothing is applied holds.
  - The scheduler-config `dry_run` property (`src/mcp/tools/catalog/admin.rs:1011`) and the `dry_run` parameter of the consolidate-batch tool (line 1003) still carry no "report-only" text in their own descriptions. The tool-level description does say it. Adding a short note to those two parameters would avoid a caller reading `dry_run=false` as a real switch.
- [LOW] The neighbouring tool at `src/mcp/tools/catalog/admin.rs:1062` still says "merging and archiving similar memories; use dry_run to preview without writing."
  - I did not check whether it shares the same fake-success path. If it does, it is the same finding class as B-1. Otherwise ignore this.
- [LOW] `docs/harness/progress.md` records "Last commit `4e8bb283`", but the head under review is `0358ba36`, the docs-only commit on top. That is harmless. The independent post-review and the owner's H1 receipt are explicitly still pending, so this review does not substitute for them.

Checked and fine:
- **Embedding-migrate error path:** the new write-path error returns `success:false`, `migrated:0` and an `error` string. A test confirms it leaves `embedding_model` untouched.
- **Dry-run storage errors:** a storage failure on the dry-run count now surfaces as an error instead of `0`.
- **Persist failure:** `persist_report` failures are logged through the redaction helper.
- **Generated docs:** `docs/MCP_TOOLS.md` matches the catalog changes.
- **SDK and changelog:** the Python and TypeScript comments and the CHANGELOG entry name the behaviour change.
- **Scope:** no harness-script, gate or security-boundary changes are bundled in.
- **Canvas:** it has alternatives, edge cases and a risk table.
- **Sensors:** the full run is logged as pass.

REVIEW_VERDICT: PASS Both fake-success paths are now report-only or error with no writes, tests and docs match the code, and no scope creep or gate weakening was found.
