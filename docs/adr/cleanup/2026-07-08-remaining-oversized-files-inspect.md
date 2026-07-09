---
adr: ADR-CLEANUP-20260708-2
track: oversized files follow-up inspect
service: engram (single Rust crate)
author: Codex
status: Proposed
---

# Remaining oversized Rust files inspect

## Scope note

This is the follow-up Inspect pass after the first oversized-file cleanup wave:

- `markdown_export.rs` split landed in PR #119.
- `http_transport.rs` split landed in PR #121.
- `cli.rs` split landed in PR #124.
- `storage/queries/core.rs` split landed in PR #125.
- `storage/queries/tests.rs` split landed in PR #126.
- `context_quality.rs` / `memory_update.rs` were inspected in PR #127 and
  split in PRs #128 and #129.

This document does **not** implement another refactor. It refreshes the
oversized-file inventory and ranks the next candidates using static structure:
line counts, `#[cfg(test)]` split point, top-level `pub fn` / `fn` / `impl`
clusters, existing section comments, feature gates, and public re-export risk.

No production code was edited for this assessment.

## Fresh inventory

After the merged splits above, `src/` still has **42** Rust files above the
800-line cap. Total lines and the line where `#[cfg(test)]` starts matter: some
files are only oversized because they keep a large local test module next to a
small production module.

| File | Total lines | Approx. pre-test lines | Test lines | Initial read |
|---|---:|---:|---:|---|
| `src/mcp/tools/registry.rs` | 4762 | 4762 | 0 | generated-ish data table; keep excluded |
| `src/storage/migrations.rs` | 3104 | 2519 | 585 | append-only upgrade history; keep excluded |
| `src/storage/turso_backend.rs` | 1716 | 1716 | 0 | real production monolith, but backend/sync/schema risk |
| `src/mcp/handlers/context.rs` | 1652 | 1364 | 288 | real handler-family monolith with visible sections |
| `src/mcp/handlers/memory_crud.rs` | 1641 | 1391 | 250 | real handler-family monolith with many public tools |
| `src/embedding/queue.rs` | 1412 | 902 | 510 | production queue + health/hygiene + large tests |
| `src/mcp/handlers/harness.rs` | 1405 | 837 | 568 | four handler concerns + large tests |
| `src/storage/meilisearch_backend.rs` | 1339 | 990 | 349 | feature-gated backend with filter/doc/backend seams |
| `src/storage/sqlite_backend.rs` | 1304 | 829 | 475 | backend facade plus derived-index health checks/tests |
| `src/graph/conflicts.rs` | 1220 | 886 | 334 | conflict types/detection/resolution/persistence |
| `src/intelligence/consolidation_offline.rs` | 1158 | 734 | 424 | production under cap; test-heavy |
| `src/graph/temporal.rs` | 1128 | 491 | 637 | production under cap; test-heavy |
| `src/mcp/handlers/misc.rs` | 1115 | 1115 | 0 | real production handler-family monolith |
| `src/attestation/chain.rs` | 1110 | 531 | 579 | security-sensitive; oversized due tests |
| `src/graph/mod.rs` | 1101 | 919 | 182 | graph type/render/stats/filter/export/community mix |
| `src/intelligence/emotional.rs` | 1082 | 804 | 278 | just over cap; few large cohesive impls |
| `src/storage/auto_linker.rs` | 1079 | 499 | 580 | production under cap; test-heavy |
| `src/types.rs` | 1073 | 1073 | 0 | global DTO/type surface; public API design risk |
| `src/intelligence/salience.rs` | 1024 | 794 | 230 | production under cap by this heuristic |
| `src/intelligence/gardening.rs` | 1014 | 613 | 401 | production under cap; test-heavy |
| `src/mcp/handlers/lifecycle.rs` | 1013 | 582 | 431 | production under cap; test-heavy |
| `src/graph/duckdb_graph.rs` | 1012 | 523 | 489 | production under cap; feature-gated/test-heavy |
| `src/bin/server.rs` | 999 | 883 | 116 | binary entrypoint; split possible but lower blast-radius payoff |
| `src/context/record.rs` | 949 | 949 | 0 | operational-context record/artifact/redaction helpers |
| `src/intelligence/session_context.rs` | 946 | 780 | 166 | production under cap |
| `src/intelligence/proactive.rs` | 931 | 514 | 417 | production under cap; test-heavy |
| `src/storage/dream_snapshots.rs` | 923 | 750 | 173 | production under cap |
| `src/intelligence/project_context.rs` | 921 | 726 | 195 | production under cap |
| `src/search/feedback.rs` | 913 | 474 | 439 | production under cap; test-heavy |
| `src/storage/image_storage.rs` | 911 | 776 | 135 | production under cap |
| `src/storage/operational_context.rs` | 897 | 806 | 91 | just over cap; event/summary/artifact seam |
| `src/storage/graph_queries.rs` | 893 | 638 | 255 | production under cap |
| `src/mcp/handlers/enrichment_audit.rs` | 892 | 597 | 295 | production under cap |
| `src/storage/clustering.rs` | 880 | 547 | 333 | production under cap |
| `src/storage/filter.rs` | 867 | 449 | 418 | production under cap; test-heavy |
| `src/search/utility.rs` | 850 | 549 | 301 | production under cap |
| `src/mcp/handlers/multimodal.rs` | 842 | 607 | 235 | production under cap |
| `src/intelligence/context_compression.rs` | 828 | 555 | 273 | production under cap |
| `src/intelligence/compression_semantic.rs` | 825 | 518 | 307 | production under cap |
| `src/dream/candidates.rs` | 824 | 642 | 182 | production under cap |
| `src/mcp/handlers/search.rs` | 822 | 822 | 0 | barely oversized; search/feedback/compact mix |
| `src/intelligence/session_indexing.rs` | 807 | 695 | 112 | production under cap |

The useful signal is therefore not “42 files need splitting”. A large subset is
only oversized because tests are embedded in the same file. That is legitimate
cleanup, but lower priority than production files whose pre-test body is still
well over 800 lines.

## Findings

### A — still excluded from cleanup-pass splitting

- **`src/mcp/tools/registry.rs` (4762 lines).** Same verdict as the original
  oversized-file ADR: this is the MCP `ToolDef` registry data table, not a
  logic module. Splitting it without changing the registry storage format only
  spreads one review surface across fragments. Treat any future change as an
  RFC on registry storage/codegen, not as a line-count cleanup.

- **`src/storage/migrations.rs` (3104 lines).** Same verdict as before: it is
  append-only upgrade history. Even if version-range splitting is possible, the
  benefit is cosmetic and the blast radius is deployed-user upgrade paths. Do
  not include it in a cleanup pass.

### B — strongest new split candidates

- **`src/mcp/handlers/misc.rs` (1115 lines, no test module).** Despite the
  filename, this file already contains explicit section boundaries: tag
  utilities, import/export, maintenance, image handling, auto-tagging,
  Langfuse integration, Meilisearch tools, and tool discovery. The feature-gated
  Langfuse and Meilisearch sections are especially clean seams: they do not need
  to live beside tag validation or image upload.

  **Proposed split axis:** by tool family:
  `misc/{mod.rs,tags.rs,import_export.rs,maintenance.rs,images.rs,auto_tag.rs,langfuse.rs,meilisearch.rs,discovery.rs}`.
  Keep the public dispatch surface unchanged by re-exporting the same handler
  names from `misc/mod.rs`.

- **`src/embedding/queue.rs` (1412 lines; ~902 production + 510 tests).** The
  file has five separable concerns: queue/request types, `EmbeddingWorker`,
  durable status reads (`get_embedding_status`, `get_embedding`), retry/health
  accounting, explicit hygiene/drain operations, and tests. The seams are
  stronger than the line count suggests because public exports in
  `src/embedding/mod.rs` already enumerate the queue API and can remain stable.

  **Proposed split axis:**
  `queue/{mod.rs,worker.rs,status.rs,health.rs,hygiene.rs,tests.rs}`. Preserve
  `src/embedding/mod.rs` re-exports exactly. Because this touches async worker
  behavior and durable SQL queue states, verify with focused embedding-queue
  tests before any broad CI.

- **`src/mcp/handlers/context.rs` (1652 lines; ~1364 production).** The file
  mixes at least seven handler groups: Operational Context record/search/bundle,
  artifact retrieval, fact extraction/list/graph, prompt-context assembly,
  self-editing memory blocks, injection prompt/tool-output archive, and working
  memory/prepare-context helpers. Section comments already name several seams.

  **Proposed split axis:** by handler family:
  `context/{mod.rs,operational.rs,artifacts.rs,facts.rs,builder.rs,blocks.rs,injection.rs,tool_outputs.rs,tests.rs}`.
  This has higher blast radius than `misc.rs` because many tools route through
  it, but it is still a mechanical move-and-re-export refactor if no logic is
  changed.

- **`src/mcp/handlers/harness.rs` (1405 lines; ~837 production + 568 tests).**
  The public surface is four handlers — `handle_harness_record`,
  `handle_harness_status`, `handle_harness_handoff`, and
  `handle_harness_verify` — followed by helper/test code. This is a clean split
  by command responsibility.

  **Proposed split axis:**
  `harness/{mod.rs,record.rs,status.rs,handoff.rs,verify.rs,shared.rs,tests.rs}`.
  Keep the four handler names re-exported from `mod.rs`.

- **`src/mcp/handlers/memory_crud.rs` (1641 lines; ~1391 production).** This
  is no longer just CRUD. It includes base create/get/update/delete/list,
  `context_seed`, daily/permanent/checkpoint/boost behavior, episodic/procedural
  creation, procedure outcomes, expiration cleanup, batch create/delete, section
  creation, todo/issue creation, and fact ingest/batch ingest. The split is
  valuable, but the public function count is high and `handlers::dispatch` calls
  many of these names directly, so re-export accuracy is the main risk.

  **Proposed split axis:** by handler family, not by CRUD verb:
  `memory_crud/{mod.rs,create.rs,read_update_delete.rs,seed.rs,lifecycle.rs,procedural.rs,batch.rs,sections.rs,tasks.rs,facts.rs,shared.rs,tests.rs}`.

### C — good candidates, but not first

- **`src/storage/sqlite_backend.rs` (1304 lines; ~829 production).** The first
  half is a thin `StorageBackend`/`TransactionalBackend`/`CloudSyncBackend`
  wrapper; the oversized part is mostly derived-index health checks
  (`sqlite_embedding_health`, `sqlite_fts_health`, `sqlite_graph_health`) and
  their tests.

  **Proposed split axis:** extract health checks only:
  `sqlite_backend/{mod.rs,health.rs,tests.rs}`. This avoids pretending the
  backend trait impl itself has a natural CRUD-domain split.

- **`src/storage/meilisearch_backend.rs` (1339 lines; ~990 production).** Clear
  groups: document mapping/conversion, filter building, backend lifecycle/index
  calls, `StorageBackend` impl, tests. It is feature-gated and external-service
  adjacent, so run after simpler local-only splits.

  **Proposed split axis:**
  `meilisearch_backend/{mod.rs,document.rs,filters.rs,backend.rs,storage_impl.rs,tests.rs}`.

- **`src/graph/conflicts.rs` (1220 lines; ~886 production).** Clear conceptual
  split: conflict/resolution types, detection passes, resolver strategies,
  persistence/listing, row mappers, tests. This is a real candidate, but it is
  graph-correctness logic rather than a pure handler facade, so keep it behind
  the MCP handler splits in priority.

  **Proposed split axis:**
  `conflicts/{mod.rs,types.rs,detect.rs,resolve.rs,persistence.rs,tests.rs}`.

- **`src/graph/mod.rs` (1101 lines; ~919 production).** This module mixes graph
  core types, vis.js/HTML rendering, stats/centrality, filtering/neighborhood,
  DOT/GEXF export, and community detection. It has a natural split, but because
  it is the root of the `graph` module, it should be done carefully with exact
  public re-exports.

  **Proposed split axis:**
  `graph/{types.rs,render.rs,stats.rs,filter.rs,export.rs,communities.rs}` with
  `graph/mod.rs` retaining submodule declarations plus the existing public names.

- **`src/context/record.rs` (949 lines, no tests in file).** Only two public
  functions (`record_context` and `record_context_artifact`) account for most of
  the file, plus many redaction/metadata/date helpers. It has a plausible
  record-vs-artifact split, but it sits on the security boundary for retained
  Operational Context artifacts, so it should not be bundled with low-risk
  line-count cleanup.

  **Proposed split axis:**
  `record/{mod.rs,events.rs,artifacts.rs,redaction.rs,metadata.rs,types.rs}`
  after a dedicated read of artifact retention/redaction invariants.

### D — mostly test-heavy; split tests opportunistically

These files exceed 800 total lines but have production bodies under or close to
the cap once the local test module is separated: `graph/temporal.rs`,
`attestation/chain.rs`, `storage/auto_linker.rs`, `intelligence/gardening.rs`,
`mcp/handlers/lifecycle.rs`, `graph/duckdb_graph.rs`, `search/feedback.rs`,
`storage/filter.rs`, `search/utility.rs`, `mcp/handlers/multimodal.rs`,
`intelligence/context_compression.rs`, `intelligence/compression_semantic.rs`,
and similar entries in the inventory.

The right cleanup for these is usually `tests.rs` extraction only, preferably
when touching the module for related work. `attestation/chain.rs` deserves extra
care: the production logic is security-sensitive and only ~531 lines before
tests, so do not refactor the chain implementation merely to satisfy a total
line count.

### E — defer pending design

- **`src/storage/turso_backend.rs` (1716 production lines).** This is a real
  production monolith, but it combines feature-gated async backend setup, local
  schema initialization, migration DDL, row mapping, `StorageBackend`,
  `TransactionalBackend`, and cloud-sync stubs. A split is possible, but the
  right axis is backend architecture, not a quick file-size pass. Inspect Turso
  usage, feature coverage, and parity with `SqliteBackend` before moving code.

- **`src/types.rs` (1073 production lines).** This is an all-public DTO/type
  surface: `Memory`, workspace/lifecycle errors, memory/tier/scope enums,
  graph edge types, storage/search config, and input/options structs. Splitting
  it can preserve source compatibility with re-exports, but it changes the
  module organization clients import from. Treat as a public API design task,
  not as a cleanup-pass split.

- **Remaining intelligence files.** `emotional.rs` is just over the cap before
  tests and has a few large cohesive impls; `salience.rs`, `gardening.rs`, and
  `consolidation_offline.rs` are under the production cap by the pre-test
  heuristic. After the completed `memory_update`/`context_quality` splits, the
  intelligence cluster no longer looks like the highest-value next cleanup lane.

## Ranked proposals

| # | Confidence | Risk | Finding | Proposed change | Verification focus |
|---:|---|---|---|---|---|
| 1 | high | low | `mcp/handlers/misc.rs` has explicit section seams and no tests | Split by tool family under `misc/` and re-export same handler names | `cargo test --test mcp_protocol_tests --locked`; MCP reference check |
| 2 | high | low-medium | `embedding/queue.rs` mixes queue, worker, SQL status, health, hygiene, tests | Split under `embedding/queue/` by queue concern | `cargo test embedding::queue --locked`; queue hygiene/status tests |
| 3 | high | low-medium | `mcp/handlers/context.rs` mixes Operational Context, artifacts, facts, blocks, injection/output archive | Split by handler family under `context/` | focused context handler tests + MCP protocol tests |
| 4 | medium-high | low | `mcp/handlers/harness.rs` has four public handler concerns and large tests | Split into record/status/handoff/verify/shared/tests | `cargo test mcp::handlers::harness --locked`; doctor/review-gate if harness semantics touched |
| 5 | medium | medium | `mcp/handlers/memory_crud.rs` is a broad memory tool family with many dispatch callsites | Split by handler family under `memory_crud/` | MCP protocol tests; exact public handler name inventory |
| 6 | medium | low-medium | `storage/sqlite_backend.rs` mostly exceeds cap due derived-index health and tests | Extract `sqlite_backend/health.rs` plus tests | sqlite backend health tests |
| 7 | medium | medium | `storage/meilisearch_backend.rs` has document/filter/backend/storage-impl seams | Split feature-gated backend by concern | `cargo check --features meilisearch --locked`; meili unit tests |
| 8 | medium | medium | `graph/conflicts.rs` has type/detect/resolve/persistence seams | Split under `graph/conflicts/` | conflict detector/resolver tests |
| 9 | medium | medium | `graph/mod.rs` is root graph API plus rendering/export/community logic | Split internals while preserving root public API | graph tests + public API inventory |
| 10 | low-medium | medium-high | `context/record.rs` has record/artifact/redaction seams but touches artifact security boundary | Dedicated security-aware inspect before split | context artifact/redaction tests |
| 11 | low | high | `storage/turso_backend.rs` is large but architecture/feature parity should drive the split | Design read before implementation | feature-gated turso build/tests |
| 12 | low | public-API | `types.rs` is a global DTO surface | RFC/API design before split | public API inventory and downstream import review |

## Proposed for the next implementation pass

The best next implementation PR is **row 1: split `src/mcp/handlers/misc.rs` by
tool family**. It is the highest-confidence remaining candidate because:

1. it has no embedded test module masking the line count;
2. the file already has section comments that define the split;
3. it is a handler facade, so public behavior can be preserved by re-exporting
   the same function names;
4. feature-gated Langfuse/Meilisearch code can be isolated rather than
   interleaved with core tag/export/image utilities.

Run row 2 (`embedding/queue.rs`) after row 1 if the team wants to keep reducing
production oversized files. Rows 3–5 are also valid, but they touch broader MCP
tool families and should be one PR each.

## Deferred

- Do not re-open `registry.rs` or `migrations.rs` as cleanup-pass targets.
- Do not split test-heavy files just to reduce total line count unless the module
  is already being touched.
- Do not split `types.rs` without an API compatibility plan.
- Do not split `turso_backend.rs` before a feature/parity design read.

## Blast radius if row 1 is implemented

- Files touched: `src/mcp/handlers/misc.rs` moved to
  `src/mcp/handlers/misc/{mod.rs,tags.rs,import_export.rs,maintenance.rs,images.rs,auto_tag.rs,langfuse.rs,meilisearch.rs,discovery.rs}`.
- Public MCP dispatch paths stay unchanged because `handlers::dispatch` keeps
  calling `misc::memory_tags`, `misc::langfuse_sync`, `misc::discover_tools`,
  etc.
- Feature gates must stay on the same Langfuse and Meilisearch functions after
  the move.
- No schema, migrations, storage semantics, or MCP tool schemas should change.

## Rollback

A row-1 implementation should be a pure move-and-re-export refactor in a single
commit. `git revert` is sufficient. No data migration or deployment
coordination is required if the MCP handler names and generated reference output
remain unchanged.
