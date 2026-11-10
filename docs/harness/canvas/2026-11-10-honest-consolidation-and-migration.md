# Review canvas — honest-consolidation-and-migration-2026-11-10

Base: main `85502ad8`. Branch: `fix/honest-consolidation-and-migration`.
Scope: findings B-1 and B-2 of the 2026-11-10 read-only review. The owner chose
(chat, 2026-11-10) to downgrade both tools to honest reporting now and implement
the real operations after the release. Triggers: MCP surface (tool descriptions
and response fields), intelligence/consolidation, embeddings.

| Alternative | Decision |
|---|---|
| B-1: implement real merges (soft-delete the merged memory, move crossrefs, version, event) | Deferred by the owner: storage and sync semantics need their own design |
| B-1: reject `dry_run=false` with an error | Rejected: the proposals are still useful, and the scheduler would turn every tick into an error |
| B-1: keep proposals, force `dry_run: true` on every report, add `applied: false` and a `note`, stop emitting `completed` events | Selected: additive response fields, no tag or count renamed |
| B-2: store the recomputed vectors and rebuild HNSW | Deferred by the owner: needs vector storage, index rebuild and dimension checks |
| B-2: forward to `memory_rebuild_embeddings` | Rejected: that tool clears the queue and resets `has_embedding` for every memory, which is a different and larger operation than the caller asked for |
| B-2: error without `dry_run`, no writes, point to `memory_rebuild_embeddings` | Selected |

Hot path: none. Both tools are advanced, admin-tier maintenance calls. B-2's
write path drops from O(n) embeddings plus O(n) updates to O(1). B-1 removes the
event loop.

| Edge / breakage risk | Evidence / containment |
|---|---|
| Caller relies on `success: true` from `memory_embedding_migrate` | It was false success; CHANGELOG `Fixed` names the change and the TypeScript SDK behaviour |
| TypeScript `admin.embeddingMigrate()` sends no `dry_run` | Now receives the error; documented in the SDK doc comment and CHANGELOG. No SDK signature change |
| Consumers counting `duplicates_merged` from history | Counts keep their names; `dry_run: 1` in `consolidation_runs` now marks them as proposals |
| Older `consolidation_runs` rows with `dry_run = 0` | Left as they are; they describe runs that applied nothing either (history is not rewritten) |
| `persist_report` failure | Was `let _ =`; now logged with redaction and still does not fail the run |
| Dry-run count storage error | Was reported as `0`; now an error response |
| `effective_removed_memory_ids` on a report | Unused outside tests; unchanged |

Tests (written first; RED observed on all three, then GREEN):
`intelligence::auto_consolidate::tests::run_without_dry_run_is_report_only`,
`mcp::handlers::auto_consolidate::tests::consolidate_batch_without_dry_run_is_report_only`,
`mcp::handlers::retrieval::tests::embedding_migrate_without_dry_run_reports_unsupported_and_writes_nothing`,
plus `embedding_migrate_dry_run_counts_embedded_memories` (already passing).

Required before upstream: full `sensors.sh`, independent post-review, owner's
H1 receipt. Push and PR for this branch need their own owner authorization.
