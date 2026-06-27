REVIEW_VERDICT: FAIL writer enumeration still misses an automatic compression scheduler path

- [BLOCKER] The spec still under-enumerates lifecycle writers. It claims exactly four decay-policy/compression engines and says they are all manual MCP tools with no scheduler (`docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md:34`, `:43`), but `src/bin/server.rs:122` defines an optional compression scheduler and `src/bin/server.rs:726` starts it when `ENGRAM_COMPRESSION_INTERVAL` / `--compression-interval-seconds` is enabled. That scheduler calls `compress_old_memories` (`src/bin/server.rs:746`), which applies the same age/importance/access-count predicate (`src/storage/queries/retention.rs:237`) and writes `lifecycle_state='archived'` (`src/storage/queries/retention.rs:312`). The spec classifies this site only as `retention_policy_apply` compression/domain (`docs/...design.md:55`), but the server scheduler is not that explicit retention policy path. This would leave a fifth automatic compression/lifecycle writer outside `decide_lifecycle_state`.

  Compact inventory:
  - `src/mcp/handlers/lifecycle.rs:178` → `lifecycle_run` decay engine → agree.
  - `src/mcp/handlers/lifecycle.rs:184` → `lifecycle_run` decay engine → agree.
  - `src/intelligence/salience.rs:450` → `run_salience_decay` decay engine → agree.
  - `src/mcp/handlers/memory_policy.rs:146` + `src/mcp/handlers/memory_policy.rs:352` → `memory_decay` decay engine → agree.
  - `src/mcp/handlers/summarize.rs:329` → `memory_archive_old` decay/compression engine → agree.
  - `src/storage/queries/retention.rs:182` → retention max-count domain writer → agree.
  - `src/storage/queries/retention.rs:312` → dual-use: retention compression domain writer AND server compression scheduler writer → disagree/incomplete.
  - `src/storage/queries/retention.rs:202` → retention auto-delete `valid_to` write by `created_at` → agree.
  - `src/intelligence/consolidation_offline.rs:568` → consolidation domain writer → agree.
  - `src/intelligence/context_quality.rs:730`, `:737` → conflict-resolution domain writer → agree.
  - `src/mcp/handlers/dream.rs:377` → approved dream expire action → agree.
  - `src/mcp/handlers/lifecycle.rs:239` → manual `memory_set_lifecycle` via helper → agree.
  - `src/storage/queries/lifecycle.rs:28`, `:39` → query-layer helper → agree.
  - `src/storage/migrations.rs:926`, `src/storage/turso_backend.rs:651` → initializer/default active state → agree as non-engine.
  - `src/storage/queries/tests.rs:1543`, `benches/search.rs:328` → test/benchmark fixtures → irrelevant non-engines.

- [HIGH] Public MCP contract cleanup is under-specified for the behavior changes. The spec mentions compatibility translation only for `stale_days`/`archive_days` (`docs/...design.md:246`) and says salience docs/tests must change (`docs/...design.md:251`), but current public metadata still advertises old behavior: `memory_decay` says it updates active lifecycle transitions (`docs/MCP_TOOLS.md:823`, `src/mcp/tools/registry.rs:641`), `memory_archive_old` says it moves originals to archived state (`docs/MCP_TOOLS.md:1282`, `src/mcp/tools/registry.rs:1185`), `lifecycle_run` still exposes `min_importance` (`docs/MCP_TOOLS.md:1335`, `src/mcp/tools/registry.rs:1235`), and `salience_decay_run` says it updates lifecycle states (`docs/MCP_TOOLS.md:1490`, `src/mcp/tools/registry.rs:1403`). The implementation plan needs explicit registry/schema/reference updates for all changed tool surfaces, not only the Rust predicate.

- [MED] Other reviewed decisions look sound once the scheduler gap is fixed: `memory_archive_old` should be disarmed rather than kept as a second canonical-predicate caller because otherwise it remains a parallel lifecycle writer; `memory_decay` can defer retention-score input as long as score updates remain and lifecycle transitions stop; retention auto-delete is correctly documented as creation-age based (`src/storage/queries/retention.rs:198`); `SalienceScore.suggested_state` currently uses the legacy predicate (`src/intelligence/salience.rs:211`, `:254`) and the spec’s delegation to the canonical predicate is necessary; storage create/update paths store raw `f32` importance without clamping (`src/storage/queries/core.rs:653`, `:901`), so predicate-local normalization is required.
