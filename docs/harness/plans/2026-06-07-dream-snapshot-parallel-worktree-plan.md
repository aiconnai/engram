# Dream Snapshot Parallel Worktree Execution Plan

**Goal:** Code `ENGRA-94` through `ENGRA-100` with maximum safe parallelism
using subagents and Git worktrees, while preserving Engram's harness gates,
schema invariants, MCP contract discipline, and review requirements.

**Primary product plan:** `docs/harness/plans/2026-06-07-dream-snapshot-review-pipeline.md`

**Tracker issues:**

- `ENGRA-94` — RFC and review candidate contract
- `ENGRA-95` — storage schema and lifecycle
- `ENGRA-96` — deterministic candidate generator
- `ENGRA-97` — MCP review tools
- `ENGRA-98` — freshness engine
- `ENGRA-99` — harness dogfooding and context bundles
- `ENGRA-100` — eval suite and docs

---

## Non-Negotiable Constraints

Parallelism is useful only if it does not create hidden contract drift.

- Every worktree starts with `bash docs/harness/bin/bootstrap.sh`.
- Every subagent reads the same source plan and its issue-specific prompt.
- No agent edits files outside its ownership list without coordination.
- Storage migration work is serialized through `ENGRA-95`.
- MCP tool registry and `docs/MCP_TOOLS.md` are serialized through `ENGRA-97`
  and regenerated again in final integration.
- Canonical progress updates are coordinated by the orchestrator to avoid
  conflicting edits to `docs/harness/progress.md`.
- Each coding lane leaves a lane handoff artifact with commands run, files
  changed, tests run, tests skipped, risks, and next merge dependency.
- Final integration runs full gates before any completion claim.

Current repo note: the main worktree is dirty. Before creating any coding
worktree, the orchestrator must make the base explicit:

- Commit the two plan artifacts as a planning commit, or intentionally park them
  outside the feature branches.
- Commit, stash, or otherwise park unrelated dirty product work before running
  `vc-gate.sh start` in any lane.
- Record the exact clean base SHA used for worktree creation.

Do not start implementation lanes from an ambiguous dirty `main`.

---

## Parallelization Model

Use one orchestrator plus seven implementation subagents. Two additional
reviewer/verifier subagents should be independent from the implementers.

### Orchestrator

Owns:

- worktree creation and base branch selection;
- Huly status/dependency updates;
- canonical `docs/harness/progress.md` updates;
- integration branch;
- final `docs/MCP_TOOLS.md` regeneration;
- final `make ci`, `doctor.sh`, and review-gate.

Does not implement feature logic except trivial merge conflict resolution.

### Subagent Lanes

| Lane | Issue | Worktree | Branch | Parallel group | Primary ownership |
|---|---|---|---|---|---|
| A | `ENGRA-94` | `../engram-engra-94` | `feature/engra-94-dream-rfc` | serial first | RFC, Review Canvas, contract |
| B | `ENGRA-95` | `../engram-engra-95` | `feature/engra-95-dream-storage` | wave 1 | migrations, storage queries, schema docs |
| C | `ENGRA-96` | `../engram-engra-96` | `feature/engra-96-dream-generator` | wave 1/2 | candidate generator, dream modules |
| D | `ENGRA-97` | `../engram-engra-97` | `feature/engra-97-dream-mcp` | wave 2 | MCP handlers/tools/tests |
| E | `ENGRA-98` | `../engram-engra-98` | `feature/engra-98-dream-freshness` | wave 2 | freshness, temporal candidate logic |
| F | `ENGRA-99` | `../engram-engra-99` | `feature/engra-99-dream-harness-context` | wave 3 | context bundle and harness dogfooding |
| G | `ENGRA-100` | `../engram-engra-100` | `feature/engra-100-dream-evals-docs` | wave 1/3 | eval fixtures, user docs, final docs |

### Reviewer Lanes

| Lane | Role | Timing |
|---|---|---|
| R1 | Contract reviewer | After `ENGRA-94`; verifies RFC/canvas against product and harness constraints |
| R2 | Code reviewer | After each coding lane reaches green focused checks |
| R3 | Integration verifier | After integration branch merges all lanes; runs end-to-end checks |

---

## Worktree Setup

Do this from the parent directory or from the current repo root.

0. Clean or park the current dirty state:

```bash
git status --short
git add docs/harness/plans/2026-06-07-dream-snapshot-review-pipeline.md \
        docs/harness/plans/2026-06-07-dream-snapshot-parallel-worktree-plan.md
git commit -m "docs(harness): plan dream snapshot implementation"
```

If there are unrelated dirty files, do not include them in this planning commit.
Commit them separately, stash them, or move the dream worktrees from a clean
known base SHA. The execution run starts only after the chosen base is clean.

1. Establish the base:

```bash
git rev-parse --short HEAD
git status --short
```

Expected: `git status --short` is empty. If it is not empty, stop.

2. Confirm RFC numbering:

```bash
rg --files docs/rfcs | sort
test ! -e docs/rfcs/0007-dream-snapshot-review-pipeline.md
```

As of this plan, `0007` is free. The repo already has a duplicated `0003`
number (`0003-search-index-v2-benchmark.md` and `0003-search-index-v2.md`), so
Lane A must run this preflight before creating the RFC.

3. Create the RFC lane first:

```bash
git worktree add ../engram-engra-94 -b feature/engra-94-dream-rfc HEAD
```

4. After `ENGRA-94` is reviewed and merged to the integration base, create a
shared base branch:

```bash
git branch feature/dream-snapshot-base feature/engra-94-dream-rfc
```

5. Create parallel coding worktrees from `feature/dream-snapshot-base`:

```bash
git worktree add ../engram-engra-95  -b feature/engra-95-dream-storage         feature/dream-snapshot-base
git worktree add ../engram-engra-96  -b feature/engra-96-dream-generator       feature/dream-snapshot-base
git worktree add ../engram-engra-97  -b feature/engra-97-dream-mcp             feature/dream-snapshot-base
git worktree add ../engram-engra-98  -b feature/engra-98-dream-freshness       feature/dream-snapshot-base
git worktree add ../engram-engra-99  -b feature/engra-99-dream-harness-context feature/dream-snapshot-base
git worktree add ../engram-engra-100 -b feature/engra-100-dream-evals-docs     feature/dream-snapshot-base
```

6. In every worktree:

```bash
bash docs/harness/bin/bootstrap.sh
bash docs/harness/bin/doctor.sh
bash docs/harness/bin/vc-gate.sh start ENGRA-XX
```

If `vc-gate.sh start` fails because the worktree is dirty, stop and resolve the
dirty state before coding.

---

## Wave Plan

### Wave 0: Contract First

Only `ENGRA-94` codes in this wave.

Subagent A owns:

- `docs/rfcs/0007-dream-snapshot-review-pipeline.md`
- `docs/harness/canvas/YYYY-MM-DD-dream-snapshot-review-pipeline.md`
- optional edits to the product plan if the RFC changes scope

Before creating the RFC, Lane A must run:

```bash
rg --files docs/rfcs | sort
test ! -e docs/rfcs/0007-dream-snapshot-review-pipeline.md
```

Do not touch Rust code in Wave 0.

Verification:

```bash
bash docs/harness/bin/doctor.sh
git diff --check
```

Exit condition:

- RFC accepted by human or reviewer.
- Review Canvas exists before schema/MCP work begins.
- Integration base branch is created from the reviewed RFC branch.

### Wave 1: Storage, Generator Skeleton, Eval Skeleton

Run B, C, and G in parallel after Wave 0.

Subagent B, `ENGRA-95`, owns storage:

- `src/storage/migrations.rs`
- `src/storage/queries/dream_jobs.rs`
- `src/storage/queries/mod.rs`
- storage-focused tests
- `docs/SCHEMA.md`

Subagent C, `ENGRA-96`, starts generator work but must avoid hard dependency on
unmerged storage by using pure structs and in-memory fixtures first:

- `src/dream/candidates.rs`
- `src/dream/mod.rs` module wiring and thin orchestration
- candidate generator unit tests

Subagent G, `ENGRA-100`, starts eval fixtures and docs skeleton:

- `tests/dream_eval_tests.rs` with ignored or fixture-only tests until storage
  lands
- `docs/AI_GUIDE.md` draft sections
- `docs/USING_ENGRAM_IN_A_REPO.md` draft sections

Rules:

- C must not edit `src/storage/migrations.rs`.
- B and E must not edit `src/dream/mod.rs`; they request wiring through C's
  lane handoff.
- G must not regenerate `docs/MCP_TOOLS.md`.
- B must not add public MCP tools.

Wave 1 focused verification:

```bash
cargo fmt --all -- --check
cargo test dream --all-features -- --nocapture
bash docs/harness/bin/doctor.sh
```

Storage lane additionally runs:

```bash
cargo test migrations --lib -- --nocapture
cargo test dream_jobs --lib -- --nocapture
```

### Wave 2: MCP And Freshness

Start after storage helpers from `ENGRA-95` are merged into the integration
base. Rebase C, D, E, and G onto that integration base.

Subagent D, `ENGRA-97`, owns MCP:

- `src/mcp/handlers/dream.rs`
- `src/mcp/handlers/mod.rs`
- `src/mcp/tools/mod.rs`
- `src/mcp/tools/registry.rs`
- `tests/mcp_protocol_tests.rs`
- `docs/MCP_TOOLS.md`

Subagent E, `ENGRA-98`, owns freshness:

- `src/dream/freshness.rs`
- `src/dream/candidates.rs` freshness extension points
- focused temporal/freshness tests

Subagent C continues generator integration against real storage:

- candidate persistence calls
- enrichment event emission
- no canonical memory mutation tests

Rules:

- D is the only lane allowed to edit MCP registry files during this wave.
- E may add pure helper functions in temporal modules, but must not change MCP
  temporal tool contracts unless coordinated with D.
- E must not edit `src/dream/mod.rs`; any module wiring request goes to C or
  the orchestrator during integration.
- C and E coordinate through small structs/functions in `src/dream/candidates.rs`
  to avoid overlapping large rewrites.

Wave 2 focused verification:

```bash
cargo fmt --all -- --check
cargo test dream_candidate --all-features -- --nocapture
cargo test freshness --all-features -- --nocapture
cargo test memory_search --test mcp_protocol_tests -- --nocapture
./scripts/generate-mcp-reference.sh --check
bash docs/harness/bin/doctor.sh
```

### Wave 3: Harness Dogfooding And Final Evals

Start after generator and MCP review tools are usable.

Subagent F, `ENGRA-99`, owns harness/context integration:

- `src/mcp/handlers/harness.rs`
- `src/mcp/handlers/context.rs`
- `src/storage/operational_context.rs`
- context bundle tests
- harness/repo usage docs

Subagent G, `ENGRA-100`, finishes evals and docs:

- non-ignored eval tests where practical
- README/AI guide/repo usage guide
- final eval runbook

Rules:

- F must not alter storage schema unless escalated to orchestrator.
- G must not modify behavior to make evals pass; failing evals must drive fixes
  back to the owning lane.
- Final `docs/MCP_TOOLS.md` regeneration belongs to orchestrator after D/F/G
  merge.

Wave 3 focused verification:

```bash
cargo fmt --all -- --check
cargo test dream --all-features -- --nocapture
cargo test context_build_bundle --test mcp_protocol_tests -- --nocapture
cargo test dream_eval --all-features -- --nocapture
bash docs/harness/bin/doctor.sh
```

---

## File Ownership Matrix

| Path | Owner | Notes |
|---|---|---|
| `docs/rfcs/0007-*` | A | Contract only |
| `docs/harness/canvas/*dream-snapshot*` | A | Required before code |
| `src/storage/migrations.rs` | B | Serialized; no other lane edits |
| `src/storage/queries/dream_jobs.rs` | B | New module |
| `docs/SCHEMA.md` | B | Schema docs only |
| `src/dream/candidates.rs` | C | Coordinate with E for freshness hooks |
| `src/dream/freshness.rs` | E | Freshness-specific |
| `src/dream/mod.rs` | C | Sole owner for module wiring/thin orchestration; B/E request via handoff |
| `src/mcp/handlers/dream.rs` | D | MCP only |
| `src/mcp/tools/registry.rs` | D | Serialized |
| `tests/mcp_protocol_tests.rs` | D/F/G | D owns dream tool tests; F owns bundle tests |
| `docs/MCP_TOOLS.md` | D then orchestrator | Regenerate again in integration |
| `src/mcp/handlers/context.rs` | F | Harness/context integration |
| `src/mcp/handlers/harness.rs` | F | Harness dogfooding |
| `tests/dream_eval_tests.rs` | G | Eval suite |
| `README.md`, `docs/AI_GUIDE.md`, `docs/USING_ENGRAM_IN_A_REPO.md` | G | User docs |
| `docs/harness/progress.md` | Orchestrator | Canonical summary, avoid lane conflicts |

Any agent needing to cross ownership must stop and ask the orchestrator to
update the ownership matrix.

---

## Subagent Prompt Template

Give each implementation subagent this base prompt plus issue-specific details:

```text
You are working in worktree: <absolute worktree path>
Issue: ENGRA-XX — <title>
Branch: <branch>

Before planning or editing:
1. Run `bash docs/harness/bin/bootstrap.sh`.
2. Read `docs/harness/plans/2026-06-07-dream-snapshot-review-pipeline.md`.
3. Read `docs/harness/plans/2026-06-07-dream-snapshot-parallel-worktree-plan.md`.
4. Read the RFC/canvas once ENGRA-94 exists.

You own only these paths:
<path list>

You must not edit:
<forbidden paths>

Deliver:
1. Minimal implementation for ENGRA-XX.
2. Focused tests listed in the plan.
3. A lane handoff note with files changed, commands run, tests skipped, risks,
   and merge dependencies.
4. No broad refactors.

Stop conditions:
- Need to edit another lane's owned file.
- Storage schema conflict.
- MCP registry conflict.
- Two failed attempts at the same fix.
- Any uncertainty about canonical memory mutation safety.
```

Suggested lane handoff file:

```text
docs/harness/progress/dream-snapshot/ENGRA-XX-handoff.md
```

The orchestrator consolidates those handoffs into canonical progress docs during
integration.

---

## Integration Strategy

Create one integration branch after `ENGRA-94`:

```bash
git worktree add ../engram-dream-integration -b feature/dream-snapshot-integration feature/dream-snapshot-base
```

Merge order:

1. `ENGRA-95` storage schema and lifecycle
2. `ENGRA-96` generator
3. `ENGRA-98` freshness
4. `ENGRA-97` MCP review tools
5. `ENGRA-99` harness/context dogfooding
6. `ENGRA-100` evals and docs

Reasoning:

- Storage must land before durable generator/MCP behavior.
- Generator and freshness should settle before MCP exposes stable response
  shapes.
- Harness dogfooding depends on MCP and generator behavior.
- Final eval/docs should reflect the integrated behavior, not each lane's draft.

After each merge:

```bash
cargo fmt --all -- --check
bash docs/harness/bin/doctor.sh
```

After MCP lane merge:

```bash
./scripts/generate-mcp-reference.sh --check
cargo test dream --all-features -- --nocapture
cargo test memory_search --test mcp_protocol_tests -- --nocapture
```

Final integration gate:

```bash
cargo fmt --all -- --check
make full-feature-check
cargo test dream --all-features -- --nocapture
cargo test dream_eval --all-features -- --nocapture
cargo test memory_search --test mcp_protocol_tests -- --nocapture
./scripts/generate-mcp-reference.sh --check
bash docs/harness/bin/doctor.sh
make ci
bash docs/harness/bin/review-gate.sh post dream-snapshot-review-pipeline
```

---

## Merge Conflict Hotspots

Expected hotspots:

- `src/storage/migrations.rs`
- `src/dream/mod.rs`
- `src/mcp/handlers/mod.rs`
- `src/mcp/tools/registry.rs`
- `tests/mcp_protocol_tests.rs`
- `docs/MCP_TOOLS.md`
- `docs/harness/progress.md`

Mitigations:

- Add new modules instead of large edits to existing files.
- Keep `src/dream/mod.rs` to module declarations and thin orchestration.
- Regenerate `docs/MCP_TOOLS.md` only in D and final integration.
- Keep progress updates in lane handoff files until orchestration merge.
- Rebase each lane after `ENGRA-95` lands.

---

## Definition Of Done

Per lane:

- Worktree starts from declared base.
- Bootstrap and doctor run.
- Focused tests pass.
- Lane handoff file exists.
- No forbidden-path edits.
- Review-gate prompt generated or reviewer artifact recorded for non-trivial
  code.

Whole project:

- All seven issues merged into integration branch.
- `docs/MCP_TOOLS.md` regenerated and checked.
- Full `make ci` passes.
- `doctor.sh` passes.
- Post-review gate returns `REVIEW_VERDICT: PASS`.
- Huly issues updated with final evidence.
- Main worktree remains clean except intentional release artifacts.

---

## Recommended Maximum Parallelism

Do not run all seven as unconstrained writers. The safe maximum is:

- Wave 0: 1 implementation agent + 1 reviewer.
- Wave 1: 3 implementation agents + 1 roaming reviewer.
- Wave 2: 3 implementation agents + 1 reviewer.
- Wave 3: 2 implementation agents + 1 integration verifier.

That gives high throughput while respecting the two serial contracts that matter:
schema first, MCP surface second.
