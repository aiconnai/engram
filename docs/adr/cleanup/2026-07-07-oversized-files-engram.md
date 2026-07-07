---
adr: ADR-CLEANUP-20260707-1
track: oversized files (adapted single-track pass; not one of the seven mbras-cleanup-pass tracks — Rust monolith, not the Go/TS/Elixir monorepo the skill targets)
service: engram (single crate, not a multi-service boundary)
author: Ronaldo (via Claude assessment)
status: Proposed
---

## Scope note

`mbras-cleanup-pass` is written for the MBRAS/IBVI Go+TS+Elixir monorepo and
its seven tracks (dedup, type consolidation, dead code, circular deps, type
strengthening, error handling, deprecated/AI-slop). None of those seven map
directly onto "file exceeds the 800-line cap in `coding-style.md`" for a
single Rust crate. This ADR borrows the skill's *discipline* — assessment
before action, evidence per finding, ranked confidence, explicit blast
radius, human gate before implementation — and applies it to the one
question asked: which of the 49 oversized files in `engram/src` are real
split candidates, and in what order.

**No files were edited to produce this assessment.**

## Findings

49 of 265 `.rs` files in `src/` exceed 800 lines (coding-style.md hard cap).
Inspected the 16 largest plus the `intelligence/*` cluster by internal
structure (`grep` for `^pub fn`, `^impl`, module boundaries, generation
mechanism) rather than by line count alone.

### Category A — not lists of logic; splitting is a trap

- **`src/mcp/tools/registry.rs` (4762 lines).** Not a Rust module. It is
  `include!("registry.rs")`'d directly into `src/mcp/tools/mod.rs:30` as a
  literal `&[ToolDef]` array expression — confirmed by
  `grep -rn "include!" src/mcp/tools/*.rs`. Contains 280 `ToolDef` struct
  literals (tool name, description, JSON schema string, tier, annotations),
  one per MCP tool. It is also the generation source for
  `scripts/generate_mcp_reference.py` (`DEFAULT_SOURCE = ROOT /
  "src/mcp/tools/registry.rs"`), which produces `docs/MCP_TOOLS.md`, and for
  the freshly-landed `catalog.rs` (PR #116) which classifies tools by group.
  Splitting this file means either (a) breaking the `include!` mechanism —
  a build-system change, not a cleanup — or (b) splitting into multiple
  `include!`'d fragments concatenated back into one array, which reduces
  file line-count but does not reduce the actual data volume or review
  surface; it only spreads one flat list across N files with no natural
  seam (tool count per domain is uneven and grows independently). The
  800-line rule was written for logic files; a flat data table entered once
  per tool is the wrong target for it.
  **Verdict: not a split candidate under this track. If addressed at all,
  it should be an RFC on the registry's storage format (e.g. moving schemas
  to sibling JSON/TOML files with a build script), not a cleanup-pass file
  split.**

- **`src/storage/migrations.rs` (3104 lines).** `run_migrations()` is a
  single function with a strictly ordered `if current_version < N { ... }`
  ladder from version 1 upward — confirmed by grep showing sequential
  `current_version < 1` through `< 15`+ guards. This is an append-only
  migration history: each block is a historical fact about what schema
  version N added, executed in order, never re-entered. Splitting by
  version range (e.g. `migrations/v1_10.rs`, `migrations/v11_20.rs`) is
  mechanically possible and lower-risk than it looks, since each block is
  self-contained SQL/DDL with no cross-block state — but the benefit is
  purely line-count optics; nobody edits migration 3 when adding migration
  40, so the file's size does not slow the common case (appending one new
  block at the bottom). The risk is non-zero: any refactor of a
  migrations file is exactly the kind of change `mbras-cleanup-pass`
  governance rule 4 would forbid outright in the MBRAS monorepo ("MUST NOT
  modify core.* schemas in a cleanup pass"). Engram is a single crate so
  that rule doesn't literally apply, but the spirit does — this file
  encodes irreversible upgrade history for every deployed database.
  **Verdict: technically splittable by version range, but benefit is
  cosmetic and risk (breaking a real user's upgrade path) is
  disproportionate. Defer indefinitely; do not include in this pass.**

- **`src/storage/queries/tests.rs` (2412 lines).** Test file, not
  production code. The 800-line guidance in `coding-style.md` is aimed at
  reviewability of production logic; a large test file split into many
  smaller ones has real value (parallel test authorship, faster incremental
  compiles) but is lower priority than production code and carries near-zero
  correctness risk either way.
  **Verdict: legitimate but low-priority split candidate — see Rank table.**

### Category B — real split candidates, with a structural seam already visible

- **`src/mcp/handlers/markdown_export.rs` (2021 lines).** Internal structure
  (via `grep -n "^pub fn \|^fn \|^impl "`) shows two clearly separable
  concerns living in one file: an **import** path (`parse_frontmatter`,
  `extract_body`, `classify_import_status`, `import_payload`,
  `memory_import_markdown`) and an **export** path
  (`memory_export_markdown`, `query_workspace_memories`,
  `build_related_map`, `format_memory_markdown`, `build_index_markdown`,
  `collect_md_files*`). These two directions share almost no helper
  functions (`frontmatter_tags`, `parse_tags`, `sanitize_filename` are the
  only shared surface). This is the cleanest seam found in the whole
  sweep.
  **Proposed split axis: by direction** —
  `handlers/markdown_export/{import.rs, export.rs, shared.rs}` as a
  submodule, re-exporting `memory_export_markdown` /
  `memory_import_markdown` from `mod.rs` so the public MCP handler surface
  is unchanged.

- **`src/mcp/http_transport.rs` (1923 lines).** Structure shows four
  distinct concerns: **rate limiting** (`is_rate_limit_allowed`,
  `apply_rate_limit`, `rate_limit_key`, `rate_limited_response`,
  `HttpTransportMetrics`), **the `/mcp` RPC handler** (`handle_mcp`),
  **SSE events** (`EventsQuery`, `parse_event_type`, `event_type_to_str`,
  `realtime_event_to_sse`, `handle_events`), and **router/server
  bootstrap** (`check_bearer`, `build_cors_layer`, `build_router`,
  `serve_http`, `handle_health`). Each concern already reads as a cohesive
  unit with its own naming prefix.
  **Proposed split axis: by concern** —
  `http_transport/{rate_limit.rs, mcp_handler.rs, events.rs, router.rs}`,
  keeping `serve_http` as the single public entry point re-exported from
  `mod.rs`.

- **`src/storage/queries/core.rs` (2634 lines).** 41 top-level `pub fn`s.
  Name-prefix clustering (`grep` + strip trailing suffix) shows
  `get_*` (4), `delete_*` (3), `list_*` (2+1 `list_memories`), `create_*`
  (2), plus one-off domain verbs (`search`, `search_by_*`,
  `release_dream_*`, `record_procedure_*`, `promote_to_*`, `move_to_*`,
  `memory_from_*`). This is the file CLAUDE.md's own routing rule ("never
  raw SQL in handlers; route writes through the query layer") makes the
  single mandatory funnel for all storage writes — high call-site fan-in,
  which raises the risk of a split relative to markdown_export or
  http_transport (any wrong re-export breaks compilation across every
  handler). The verb clustering is real but weaker than the other two
  Category B candidates — there is no single dominant axis (domain? verb?
  entity type?) as clean as import/export or rate-limit/handler/events.
  **Proposed split axis: by entity domain** (memory / session / identity /
  dream — matching the domains already used by MCP tool groups in the new
  `catalog.rs`), not by CRUD verb — grouping `get_memory`/`create_memory`
  together reads better than grouping all `get_*` together. This needs a
  short read of which entity each of the 41 functions actually touches
  before committing to the split, which is more design work than the other
  two candidates.

- **`src/bin/cli.rs` (1774 lines).** `clap` `Commands` enum with multiple
  `#[derive(Subcommand)]` blocks already visible in the grep output (lines
  48, 153, 205, 238, 270) — the file already separates cleanly by
  subcommand group at the type level; it just never got split into files to
  match. Lower priority only because it's a binary entry point with a
  single caller (the `cli` executable), so blast radius of getting the
  split wrong is contained to one artifact and caught immediately by
  `cargo build --bin engram-cli`.
  **Proposed split axis: by `Commands` variant group**, mirroring the
  existing four `Subcommand` enums.

### Category C — `src/intelligence/*` cluster (salience, gardening,
emotional, consolidation_offline, context_quality, memory_update; 1000–1220
lines each)

Function-count evidence:

| File | Lines | Top-level fns | `impl` blocks |
|---|---|---|---|
| `context_quality.rs` | 1216 | 19 | 7 |
| `memory_update.rs` | 1219 | 15 | 6 |
| `salience.rs` | 1024 | 10 | 7 |
| `gardening.rs` | 1014 | 8 | 3 |
| `consolidation_offline.rs` | 1158 | 8 | 3 |
| `emotional.rs` | 1082 | 4 | 6 |

This is **not** one uniform pattern, so it should not get one uniform
verdict:

- `context_quality.rs` and `memory_update.rs` have high function counts
  (19, 15) relative to their line counts (~65–80 lines/fn average) — this
  reads as **multiple genuinely-different responsibilities accumulated in
  one file over time** (module-discipline symptom), the kind of thing that
  happens when "add the new quality check here, it's already the quality
  file" wins over creating a new module five times in a row.
- `emotional.rs` has only 4 top-level fns but 6 `impl` blocks in 1082
  lines — average function/impl body length is much larger, suggesting a
  **smaller number of genuinely complex operations** rather than
  accumulated unrelated helpers. Splitting this one on function-count logic
  would be wrong; it needs a read of what the 6 impls actually do before
  proposing an axis.
- `gardening.rs` and `consolidation_offline.rs` sit in between (8 fns, ~130
  lines/fn) — plausibly a handful of substantial, cohesive operations, not
  obviously a discipline problem either way.

**Verdict: the cluster is not evidence of a single root cause.** Two files
(`context_quality.rs`, `memory_update.rs`) show the fn-count/line-count
signature of accumulated scope and are worth a follow-up Inspect pass to
find their internal seams (the same `grep -n "^pub fn\|^impl"` technique
used above for Category B, applied per-file, was not done here — this ADR
flags them as *candidates for the next Inspect pass*, not as pre-verified
splits, because I have not read their internal structure closely enough to
propose an axis without guessing). The other four are lower-confidence
symptoms and should not be batched with the first two under one assumption.

## Ranked proposals

| # | Confidence | Risk | Finding | Proposed change | Evidence |
|---|------------|------|---------|------------------|----------|
| 1 | high | low | `markdown_export.rs` mixes import/export with clean seam | Split into `markdown_export/{import,export,shared}.rs` submodule, same public re-exports | Grep of `^pub fn`/`^fn` shows near-zero shared helpers between the two directions |
| 2 | high | low-medium | `http_transport.rs` mixes rate-limit/RPC/SSE/router concerns with clean naming-prefix seams | Split into `http_transport/{rate_limit,mcp_handler,events,router}.rs`, keep `serve_http` as sole public entry | Grep shows four self-contained concern groups by function name prefix |
| 3 | medium | medium | `queries/core.rs` is the mandatory write-funnel (CLAUDE.md routing rule) with 41 fns, weak verb-only clustering | Needs a design read (which entity each fn touches) before proposing entity-domain split | Verb-prefix grep is suggestive but not conclusive; high fan-in raises re-export risk |
| 4 | medium | low | `cli.rs` already has 4 `Subcommand` enums that don't map to files | Split by existing `Commands` variant groups | Grep shows 4 pre-existing `#[derive(Subcommand)]` blocks as natural seam |
| 5 | medium | very low | `queries/tests.rs` (2412 lines) is a test file | Split by entity/domain to match whatever axis core.rs ends up using | Line count only; not yet inspected for internal structure |
| 6 | low | n/a (needs Inspect first) | `context_quality.rs`, `memory_update.rs` show accumulated-scope fn-count signature | Run per-file `^pub fn`/`^impl` Inspect before proposing any split axis | fn-count/line-count ratio only; internal structure not read |
| 7 | not proposed | n/a | `registry.rs` — flat `include!`'d data table, not logic | No file split; if pursued, RFC on storage format, not cleanup pass | `include!` mechanism confirmed at `mod.rs:30`; is generation source for `generate_mcp_reference.py` and `catalog.rs` |
| 8 | not proposed | n/a | `migrations.rs` — append-only, strictly ordered upgrade history | Defer indefinitely; cosmetic benefit, disproportionate risk to real upgrade paths | Sequential `if current_version < N` ladder confirmed by grep |

## Proposed for this pass

Nothing is proposed for *implementation* in this pass. Per the borrowed
discipline, only high-confidence + low-risk rows would qualify for
same-pass implementation, and even row 1 (the highest-confidence finding)
should get its own dedicated branch and PR rather than being bundled with
this assessment, consistent with "one track at a time, on a dedicated
branch."

If Ronaldo wants to proceed to implementation, the recommended order is
**1 → 2 → 4 → 3 → 5**, with 6 requiring a follow-up Inspect pass first and
7/8 excluded entirely.

## Deferred

- Row 3 (`queries/core.rs`) until an entity-domain read is done — proposing
  a split axis from verb-prefix grep alone risks the "plausible but wrong"
  failure mode this discipline exists to catch.
- Row 5 (`queries/tests.rs`) until row 3's axis is settled, so tests split
  the same way as the code they cover.
- Row 6 (`intelligence/*` cluster) pending a dedicated Inspect pass on
  `context_quality.rs` and `memory_update.rs` specifically — do not extend
  any conclusion to `gardening.rs`, `consolidation_offline.rs`, or
  `emotional.rs`, which show different signatures and were not
  demonstrated to share the same root cause.
- The remaining ~41 of the 49 oversized files not named above were not
  individually inspected in this pass; the 800-line list in the prior
  conversation turn is the full candidate pool but only the 16 largest plus
  the intelligence cluster were assessed here.

## Blast radius (if rows 1–2 are implemented)

- Files touched: `src/mcp/handlers/markdown_export.rs` →
  `src/mcp/handlers/markdown_export/{mod.rs,import.rs,export.rs,shared.rs}`;
  `src/mcp/http_transport.rs` →
  `src/mcp/http_transport/{mod.rs,rate_limit.rs,mcp_handler.rs,events.rs,router.rs}`.
- No public API change if `mod.rs` in each submodule re-exports the same
  function names currently exported from the flat file — verify with
  `cargo public-api` or a diff of `nm`/doc output before merging, per
  rust-agentic-engineering's "no public API changes without approval" rule.
- Test surface: existing tests for these handlers should be unaffected if
  imports resolve through the same module path; `cargo test` is the
  correct verification gate, not a new test category.
- No schema, security (HMAC/CPF/tenant), or migration surface touched.

## Rollback

Each proposed split is a pure move-and-re-export refactor with no logic
change — `git revert` of the single commit per row is sufficient. No
data migration, no backward-incompatible change, no deployment coordination
needed.
