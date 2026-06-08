# Dream Snapshot Review Pipeline Implementation Plan

**Goal:** Productize Engram's existing dream/consolidation primitives into a
non-destructive, reviewable memory synthesis pipeline for agentic engineering
work.

**Architecture:** Keep canonical memory state in existing `memories`, FTS,
vectors, graph, context events, enrichment events, and memory policy records.
Dream jobs read those sources and write separate review candidates. Candidates
can be inspected, accepted, edited, rejected, archived, or applied through
explicit MCP actions. No dream job may silently create, update, delete, expire,
or promote canonical memories.

**External inspiration:** Claude Dreams provides the job shape: asynchronous
lifecycle, separate output, inspectable progress, cancellation, archival, and
partial output preserved after failure. OpenAI's memory dreaming article
provides the quality target: carry forward useful context, follow durable
preferences/constraints, and stay current over time.

**Existing Engram substrate to reuse:**

- `src/dream/mod.rs`: current dream-phase scheduler and consolidation runner.
- `dream_runs` / `dream_locks`: current run history and advisory locking.
- `memory_policy`: deterministic salience, retention, retrieval priority, and
  policy explanations.
- `enrichment_events`: append-only audit trail for automated enrichment.
- `context_events`, `context_artifacts`, `context_summaries`, and
  `context_build_bundle`: operational context and artifact provenance.
- Temporal graph and contradiction tools for freshness/conflict detection.
- Harness docs and `context_record`/`harness_record` for dogfooding.

---

## Scope Boundary

This is product work, not harness-bootstrap work. It touches storage, MCP
surface, intelligence, dream phase, context bundling, docs, and tests. Execute it
as separate feature issues with Review Canvas and post-review evidence.

In scope:

- RFC/contract for reviewable dream snapshots.
- Durable dream job and candidate model.
- Candidate generation from memories, context events, harness events, temporal
  signals, and memory policy.
- Read/review/apply MCP tools.
- Freshness and contradiction candidate logic.
- Provenance through enrichment events and source links.
- Evaluation fixtures for continuity, preferences/constraints, freshness, and
  provenance correctness.

Out of scope for v1:

- Automatic accepted-memory writes without explicit confirmation.
- Learned ranking or model-weight memory.
- External LLM dependency as a default path.
- UI beyond MCP/CLI-accessible review surfaces.
- Raw transcript/log retention by default.
- Replacing Git, Huly, CI, repository docs, AGENTS.md, or harness docs.

---

## Product Contract

Dream snapshots are derived memory proposals, not canonical facts.

Every candidate must include:

- candidate id and job id;
- workspace and optional task/session/repo scope;
- candidate kind:
  `summary`, `preference`, `constraint`, `project_state`, `stale_fact`,
  `contradiction`, `merge`, `promotion`, `decay`, `temporal_update`;
- proposed action:
  `create`, `update`, `merge`, `supersede`, `expire`, `promote`, `demote`,
  `ignore`;
- content preview and optional proposed full content;
- confidence score and stable reason codes;
- policy explanation from `memory_policy` when source memories exist;
- source memory ids, context event ids, artifact ids, document paths, commits,
  issue ids, and timestamps where available;
- freshness state: `current`, `stale`, `future_due`, `expired`, `conflicted`,
  or `unknown`;
- sensitivity/retention labels;
- review state: `pending`, `accepted`, `edited`, `rejected`, `applied`,
  `archived`.

Candidate application must:

- require explicit `confirm=true`;
- call existing canonical memory APIs where possible;
- emit `enrichment_events` for candidate creation, review decision, and apply;
- retain rejected/edited decisions as feedback for later dream jobs;
- never hard-delete source memories.

---

## Proposed Data Model

Use a new schema version after a dedicated RFC/Review Canvas. Exact SQL belongs
to the implementation issue, but the v1 shape should be:

```sql
dream_jobs (
    id TEXT PRIMARY KEY,
    workspace TEXT NOT NULL,
    status TEXT NOT NULL,
    instructions TEXT,
    model_profile TEXT NOT NULL DEFAULT 'deterministic-local-v1',
    input_summary_json TEXT NOT NULL DEFAULT '{}',
    output_summary_json TEXT NOT NULL DEFAULT '{}',
    error_json TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    finished_at TEXT,
    archived_at TEXT
);

dream_candidates (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES dream_jobs(id) ON DELETE CASCADE,
    workspace TEXT NOT NULL,
    kind TEXT NOT NULL,
    proposed_action TEXT NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'pending',
    confidence REAL NOT NULL,
    freshness_state TEXT NOT NULL DEFAULT 'unknown',
    content_preview TEXT NOT NULL,
    proposed_content TEXT,
    reason_codes TEXT NOT NULL DEFAULT '[]',
    policy_explanation_json TEXT NOT NULL DEFAULT '{}',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    reviewed_at TEXT,
    applied_at TEXT
);

dream_candidate_sources (
    candidate_id TEXT NOT NULL REFERENCES dream_candidates(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    source_ref TEXT,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (candidate_id, source_type, source_id)
);
```

Do not reuse `memories` for unaccepted candidates. A proposed fact must be
visibly distinct from an accepted memory.

---

## MCP Surface

Add tools incrementally:

- `dream_create`: create a reviewable dream job. Default is local deterministic
  synthesis. Inputs: workspace, source selectors, instructions, dry-run limits.
- `dream_get`: retrieve job status, summary, errors, and candidate counts.
- `dream_list`: list non-archived jobs by workspace/status.
- `dream_cancel`: cancel pending/running jobs idempotently.
- `dream_archive`: archive terminal jobs.
- `dream_candidates_list`: list candidates with filters by kind/action/state.
- `dream_candidate_get`: inspect one candidate with full provenance.
- `dream_candidate_review`: accept/edit/reject/archive a candidate.
- `dream_candidate_apply`: apply an accepted candidate with `confirm=true`.
- `dream_eval_run`: run fixed local eval fixtures and report metrics.

Deprecate nothing in v1. Existing `dream_run_now` can remain as a legacy/manual
trigger while new tools are added behind `dream-phase`.

---

## Implementation Phases

### Phase 0: RFC And Review Canvas

Files:

- Create `docs/rfcs/0007-dream-snapshot-review-pipeline.md`.
- Create `docs/harness/canvas/YYYY-MM-DD-dream-snapshot-review-pipeline.md`.
- Update this plan only if the RFC materially changes scope.

Before creating the RFC, confirm numbering:

```bash
rg --files docs/rfcs | sort
test ! -e docs/rfcs/0007-dream-snapshot-review-pipeline.md
```

As of this plan, `0007` is free. This repo already has a duplicated `0003`
number, so the RFC lane must run the preflight instead of assuming the next
number.

Acceptance:

- RFC defines lifecycle, candidate contract, provenance, safety defaults, and
  non-destructive guarantee.
- Review Canvas covers storage, MCP, hooks/intelligence, stale facts,
  cross-SDK impact, and migration risk.
- No code or schema change yet.

### Phase 1: Storage And Job Lifecycle

Files likely touched:

- `src/storage/migrations.rs`
- `src/storage/queries/dream_jobs.rs`
- `src/storage/queries/mod.rs`
- `src/dream/mod.rs`
- `docs/SCHEMA.md`

Acceptance:

- `dream_jobs`, `dream_candidates`, and `dream_candidate_sources` migrate cleanly.
- CRUD/list helpers validate state transitions and timestamps.
- Cancel/archive semantics are idempotent where appropriate.
- Existing `dream_runs` remains readable and is not silently reinterpreted.
- Migration tests cover table existence, indexes, state checks, and schema
  version drift.

### Phase 2: Deterministic Candidate Generator

Files likely touched:

- `src/dream/mod.rs`
- `src/intelligence/auto_consolidate.rs`
- `src/intelligence/memory_policy/*`
- `src/storage/operational_context.rs`
- new `src/dream/candidates.rs`

Acceptance:

- Generates candidates without mutating canonical memory.
- Covers duplicate/merge, stale fact, contradiction, promotion/decay, and
  summary candidates.
- Uses `memory_policy` explanations where available.
- Attaches source memory/context/artifact references.
- Emits candidate-created `enrichment_events`.

### Phase 3: MCP Review Tools

Files likely touched:

- `src/mcp/handlers/dream.rs`
- `src/mcp/handlers/mod.rs`
- `src/mcp/tools/registry.rs`
- `src/mcp/tools/mod.rs`
- `tests/mcp_protocol_tests.rs`
- `docs/MCP_TOOLS.md`

Acceptance:

- New dream tools are listed and callable through MCP.
- Read-only annotations are correct for list/get/eval.
- Mutating annotations are correct for create/review/apply/archive/cancel.
- `dream_candidate_apply` requires `confirm=true`.
- Protocol tests cover happy path, invalid state transitions, and no silent
  canonical mutation before apply.

### Phase 4: Freshness Engine

Files likely touched:

- `src/dream/candidates.rs`
- `src/graph/temporal.rs`
- `src/storage/temporal.rs`
- `src/mcp/handlers/temporal.rs`
- `tests/dream_integration.rs`

Acceptance:

- Time-sensitive facts can become `future_due`, `expired`, or `stale`.
- Contradictory temporal claims produce review candidates instead of silent
  overwrites.
- Freshness logic uses RFC3339 UTC and never panics on malformed metadata.
- Tests include "planned next Friday" and "completed after date passes" cases.

### Phase 5: Harness Dogfooding And Context Bundles

Files likely touched:

- `src/mcp/handlers/harness.rs`
- `src/mcp/handlers/context.rs`
- `src/storage/operational_context.rs`
- `docs/harness/README.md`
- `docs/USING_ENGRAM_IN_A_REPO.md`

Acceptance:

- Dream candidates can be generated from harness/context events.
- `context_build_bundle` can include accepted dream snapshot summaries with
  provenance and staleness markers.
- Harness review/sensors/doctor outcomes are summarized without storing raw logs
  by default.
- A fresh agent can ask for current project context and see source-backed,
  freshness-aware summaries.

### Phase 6: Eval Suite And Docs

Files likely touched:

- `tests/dream_integration.rs`
- new `tests/dream_eval_tests.rs`
- `docs/AI_GUIDE.md`
- `docs/USING_ENGRAM_IN_A_REPO.md`
- `README.md`
- `docs/MCP_TOOLS.md`

Acceptance:

- Eval cases cover:
  - carry forward useful context;
  - follow preferences and constraints;
  - stay current over time;
  - provenance correctness;
  - reject unsafe raw-log capture.
- Docs explain that dream output is candidate memory until reviewed/applied.
- `make ci`, MCP reference generation, and `doctor.sh` pass before closing the
  final implementation issue.

---

## Huly Issue Breakdown

Created in Huly project `ENGRA` on 2026-06-07:

1. `ENGRA-94` — `Dream Snapshot Pipeline: RFC and review candidate contract`
2. `ENGRA-95` — `Dream Snapshot Pipeline: storage schema and lifecycle`
3. `ENGRA-96` — `Dream Snapshot Pipeline: deterministic candidate generator`
4. `ENGRA-97` — `Dream Snapshot Pipeline: MCP review tools`
5. `ENGRA-98` — `Dream Snapshot Pipeline: freshness engine`
6. `ENGRA-99` — `Dream Snapshot Pipeline: harness dogfooding and context bundles`
7. `ENGRA-100` — `Dream Snapshot Pipeline: eval suite and docs`

Each issue should reference this plan:

```text
docs/harness/plans/2026-06-07-dream-snapshot-review-pipeline.md
```

---

## Verification Plan

For plan-only work:

```bash
bash docs/harness/bin/doctor.sh
git diff --check docs/harness/plans/2026-06-07-dream-snapshot-review-pipeline.md
```

For implementation issues:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test dream --all-features -- --nocapture
cargo test dream_candidate --all-features -- --nocapture
cargo test memory_search --test mcp_protocol_tests -- --nocapture
./scripts/generate-mcp-reference.sh --check
bash docs/harness/bin/doctor.sh
make ci
```

Run `review-gate.sh post <task-id>` for storage, MCP, dream, or harness-facing
changes. A Review Canvas is required for all phases that modify schema, MCP
surface, dream/intelligence code, or harness-facing context behavior.
