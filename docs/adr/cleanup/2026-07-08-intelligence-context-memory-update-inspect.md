---
adr: ADR-CLEANUP-20260708-1
track: oversized files follow-up inspect
service: engram intelligence module
status: Proposed
---

# Intelligence oversized-file inspect: `context_quality.rs` and `memory_update.rs`

## Scope

This is the follow-up Inspect pass requested by the oversized-files cleanup ADR
for the two `src/intelligence/*` files that had a high function-count / line-count
signature of accumulated scope:

- `src/intelligence/context_quality.rs` — 1,216 lines
- `src/intelligence/memory_update.rs` — 1,219 lines

No Rust files were edited for this assessment. The commands used for evidence were
structural only: line counts, top-level item grep, caller grep, focused tests, and
manual reads of the two files.

Baseline focused tests before this report:

- `cargo test intelligence::context_quality --locked` — 3 passed
- `cargo test intelligence::memory_update --locked` — 11 passed

## Findings

### 1. `memory_update.rs` is the cleaner first split candidate

Structural evidence:

| Range | Lines | Responsibility |
|---|---:|---|
| 41–197 | 157 | Public types, `CREATE_UPDATE_LOG_TABLE`, enum parsing |
| 198–344 | 147 | `update_log` persistence: `create_update_log`, `list_update_logs`, `map_log_row` |
| 345–602 | 258 | Detection engine: `UpdateDetector`, constants, four classifier functions |
| 603–683 | 81 | Mutation path: `apply_update` |
| 684–813 | 130 | Internal helpers: workspace fetch, keyword/entity/year/hash/tag helpers |
| 814–1219 | 406 | Unit tests and test-only SQLite fixture helpers |

The seams are already marked by section headers and are cohesive:

- Log persistence does not call the detector.
- The detector calls only read-side helpers and classifier helpers.
- `apply_update` is the only mutation path for existing memory content/tags/type.
- Tests are a large, independent tail block with their own fixture helpers.

Public fan-out is modest. External callers found in `src/` are primarily:

- `src/intelligence/mod.rs` re-exporting `apply_update`, `create_update_log`,
  `list_update_logs`, `UpdateDetector`, and public types.
- `src/mcp/handlers/evolution.rs` directly using `crate::intelligence::memory_update::UpdateDetector`.

That means a split can preserve both surfaces if `memory_update/mod.rs` re-exports
the same public items and keeps `pub mod memory_update;` unchanged from
`src/intelligence/mod.rs`.

Recommended split axis:

```text
src/intelligence/memory_update.rs ->
src/intelligence/memory_update/
  mod.rs        # facade; public re-exports; shared imports if needed
  types.rs      # ConflictType, UpdateAction, UpdateCandidate, UpdateResult, UpdateLogEntry
  log.rs        # CREATE_UPDATE_LOG_TABLE, create_update_log, list_update_logs, map_log_row
  detector.rs   # UpdateDetector, MIN_CONFIDENCE/MAX_RECENT_MEMORIES, classifiers
  apply.rs      # apply_update and mutation-local helpers such as add_tag_to_json/sha256_hex
  helpers.rs    # fetch_workspace_memories, extract_keywords, keyword_overlap, shared_entity_count, contains_old_year
  tests.rs      # existing tests moved out of production module body
```

Implementation note: `ConflictType` in this module intentionally collides by name
with `context_quality::ConflictType`; `src/intelligence/mod.rs` currently exports it
as `UpdateConflictType`. A split must not rename the type or change either direct
path (`crate::intelligence::memory_update::ConflictType`) or the aliased re-export.

Verdict: high-confidence, low-risk split candidate. Do this before
`context_quality.rs`.

### 2. `context_quality.rs` is a real split candidate, but has wider public fan-out

Structural evidence:

| Range | Lines | Responsibility |
|---|---:|---|
| 28–295 | 268 | Public quality/conflict/duplicate/source-trust types and config |
| 296–489 | 194 | Text and embedding duplicate detection |
| 490–510 | 21 | `cosine_similarity` helper |
| 511–760 | 250 | Conflict detection, conflict persistence, conflict resolution |
| 761–989 | 229 | Quality scoring and scoring helpers |
| 990–1111 | 122 | Workspace quality report |
| 1112–1177 | 66 | Source trust get/update |
| 1178–1216 | 39 | Tests |

The seams are real, but the blast radius is higher than `memory_update.rs` because
many public functions are re-exported through `src/intelligence/mod.rs` and called
from MCP handlers:

- `src/mcp/handlers/quality.rs` calls quality score/report/duplicates/conflicts/source trust functions.
- `src/intelligence/auto_consolidate.rs` calls `find_near_duplicates` and `detect_conflicts`.
- `src/intelligence/mod.rs` re-exports the public types and functions.

Recommended split axis:

```text
src/intelligence/context_quality.rs ->
src/intelligence/context_quality/
  mod.rs          # facade; public re-exports; shared imports if needed
  types.rs        # ConflictType, ConflictSeverity, ResolutionType, ValidationStatus,
                  # MemoryConflict, DuplicateCandidate, EnhancedQualityScore,
                  # QualitySuggestion, SourceTrustScore, QualityReport, QualityIssue,
                  # ContextQualityConfig
  duplicates.rs   # calculate_text_similarity, find_near_duplicates,
                  # get_pending_duplicates, find_semantic_duplicates, cosine_similarity
  conflicts.rs    # detect_conflicts, create_conflict, get_unresolved_conflicts, resolve_conflict
  scoring.rs      # calculate_quality_score and scoring/suggestion helpers
  report.rs       # generate_quality_report
  source_trust.rs # get_source_trust, update_source_trust
  tests.rs        # existing tests
```

Implementation note: `calculate_quality_score` currently uses a private
`get_source_trust_for_memory` helper that reads `memory.metadata["origin"]`, while
`source_trust.rs` would expose the public get/update API. Keep that helper local to
`scoring.rs`; do not force a public API coupling between scoring and source-trust
management just to make the split look symmetrical.

Verdict: valid split candidate, medium-low risk, but should come after
`memory_update.rs` because it has more public exports and MCP handler call-sites.

## Ranked recommendations

| Rank | Confidence | Risk | Target | Proposed action |
|---:|---|---|---|---|
| 1 | high | low | `memory_update.rs` | Split by existing section headers into `memory_update/{types,log,detector,helpers,apply,tests}.rs`; keep `mod.rs` re-exports exactly equivalent. |
| 2 | high | low-medium | `context_quality.rs` | Split by quality subdomain into `context_quality/{types,duplicates,conflicts,scoring,report,source_trust,tests}.rs`; keep `src/intelligence/mod.rs` exports unchanged. |

## Proposed implementation order

1. Dedicated PR: `refactor(intelligence): split memory update module`.
2. Dedicated PR after #1 merges: `refactor(intelligence): split context quality module`.

Do not batch the two files. Both are pure move-and-re-export refactors, but batching
would make review harder and obscure any accidental public API drift.

## Verification gate for each implementation PR

Minimum local verification:

```bash
cargo test intelligence::memory_update --locked
cargo test intelligence::context_quality --locked
cargo test mcp_protocol_tests --locked
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
/opt/homebrew/bin/python3.12 scripts/generate_mcp_reference.py --check
bash docs/harness/bin/doctor.sh
git diff --check
```

For the final implementation PR before merge, also run the repo CI equivalent:

```bash
PATH="/opt/homebrew/bin:$PATH" make ci
```

## Non-goals

- No behavior changes to update detection, quality scoring, conflict resolution, or source trust.
- No schema/migration edits.
- No MCP registry/docs changes unless generated-reference check exposes accidental drift.
- No renaming of `ConflictType` in either module.
- No attempt to redesign scoring/detection heuristics in the same PR as a split.

## Rollback

Each split should be a single mechanical commit. If any import or visibility issue
escapes review, `git revert` of that one commit restores the flat module without
schema or data migration consequences.
