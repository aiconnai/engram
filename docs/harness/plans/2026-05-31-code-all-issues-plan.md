# Implementation Plan: Code All Open Issues (Engram)

**Origin**: Plan mode session started after mandatory harness bootstrap + ordered reading + doctor (green).  
**Date**: 2026-05-31  
**Active Sprint Context**: Harness Engineering v0 — bootstrap & core gates (in progress, multiple review iterations with history of FAILs).  
**User Request**: "code all the issues" (the 13 open GitHub issues in aiconnai/engram).

**Important**: This document lives in the repo so it survives sessions. The ephemeral session plan path had path-encoding issues with the write tool.

---

## 1. Summary & Strong Recommendation

**Do not attempt a big-bang implementation of all 13 issues.**

The request is a **multi-week program of work** that overlaps with (and depends on) the active operational Harness v0 sprint. Several issues have partial realization on `main` from recently merged PRs (#38–#42). The core harness memory features (#34–#37) are the dogfooding layer defined by RFC 0001 (still "Proposed").

**Recommended path**: A strictly gated, phased, evidence-based program that:
- Completes the current v0 operational harness sprint to a clean PASS first.
- Audits every open issue against current main (many deltas are smaller than the GitHub text suggests).
- Executes in dependency order with the project's own harness discipline applied to the work itself.
- Uses git worktrees for safe parallelism on independent items.
- Produces verifiable artifacts at every gate.

This plan was created in read-only plan mode after deep exploration of the harness docs, RFC 0001, source extension points (existing handoff, hooks, intelligence/session_context, CLI maintenance, MCP handlers), and recent PR history.

---

## 2. Key Discoveries from Exploration (Plan Mode)

### Harness v0 State
- `bootstrap.sh`, `doctor.sh`, `sensors.sh`, `review-gate.sh`, `check-commit-msg.sh` all exist and are functional.
- `doctor.sh` passed cleanly in this session.
- Multiple pre/post review artifacts exist (v1–v7 range), showing active iteration.
- Progress log (`progress/2026-05-30-harness-bootstrap.md`) covers scaffolding phase; scripts implementation and full green loop are the remaining v0 work.
- Invariants are strict and self-referential (bootstrap + ordered reading before planning/editing is non-negotiable).
- **Known CI Limitation (grpc_transport port bind)**: `make ci` is currently failing here due to `Operation not permitted` on socket bind in `tests/grpc_transport.rs`. This requires a known-issue + exclusion trail before any `pass_with_exclusion` claim can count.

### Relationship to the 13 Issues
- Issues #34, #35, #36, #37 are the **product** Harness Memory layer (RFC 0001 event families: Agent Session, Work Item, CI/Verification, Documentation, GitHub).
- They are explicitly **out of scope for v0** per the harness SPEC.
- Existing strong foundations:
  - `src/mcp/handlers/handoff.rs` — `session_land` already queries todos/issues/decisions and builds bootstrap prompts ("land the plane").
  - `src/hooks/session_end.rs` + pending_injections — cross-session continuity plumbing.
  - `src/intelligence/{session_context,project_context,context_builder}.rs` — rich context machinery.
  - `src/bin/cli.rs` — mature `maintenance` subcommand (status, compact, rebuild) from recent PRs.
- RFC 0001 is the **binding constraint** for the harness memory cluster. It emphasizes references over copies, provenance, summaries, exclusion of secrets/raw logs, and curated (not exhaustive) capture.

### Issue Staleness / Partial Completion
Recent merged PRs (#38 hygiene + MCP generator, #39 public prep, #40 storage maintenance, #41 Air review prompt, #42 docs-only CI skip) have advanced:
- MCP reference generator + CI guard.
- Maintenance/status CLI.
- Handoff foundations.
- Docs/CI parity/hygiene.

**Phase 1 must audit every issue against main** before any new code is written.

### Other
- No "harness" product terminology yet in src/ (only in docs/harness/ and the open issues) — confirms the feature work is ahead.
- Strong culture of generated docs, CI parity (`just ci` / `make ci`), and explicit progress recording.

---

## 3. Phased Execution Plan (Recommended Sequence)

### Phase 0: Close Harness Engineering v0 (Hard Prerequisite)
**Duration estimate**: 1–3 focused sessions.

**Must happen before touching product features**:
- Run full `bash docs/harness/bin/sensors.sh` (with `just ci` or `make ci` parity) until green.
- Execute `review-gate.sh post harness-bootstrap` (using available CLIs) until a clean `REVIEW_VERDICT: PASS ...` artifact with no [BLOCKER] items.
- Update `progress/2026-05-30-harness-bootstrap.md` and `progress.md` with evidence (commands run, outputs, review links).
- Ensure AGENTS.md, Claude.md, and any root docs are consistent with current harness reality.
- Proper Conventional Commit(s) with `harness` scope + explicit progress file updates (per INVARIANTS).
- Optional: Minor polish on scripts or known-issues if doctor/sensors surface them.

**Exit gate**: 
- doctor green.
- sensors green with documented exclusions (pass_with_exclusion only for pre-registered known-issues).
- At least one post review with hard `REVIEW_VERDICT: PASS` in a versioned artifact.
- Known-issue file exists for grpc_transport and is linked in progress before claiming exclusion-based pass.
- Sprint status updated to complete.
- No drift between SPEC.md and reality.

**Why mandatory first?** Building the product harness memory features while the operational harness has open FAIL history or incomplete loops violates the invariants the new features are meant to enforce. It is also the highest-signal way to dogfood the philosophy.

### Phase 1: Audit, Baseline, and Dependency Mapping
**Goal**: Know exactly what is left vs. what the GitHub issues claim, in traceable slices.

**Sub-phased for Traceability**:

- **1.1 P0/P1 Issue Snapshot + evidence baseline (high level):**
  - Build a first-pass matrix: already partially done vs remaining vs blocked.
  - Create mini artifact: `docs/harness/decisions/phase1-1-issue-snapshot.md`.
  - Include scope, sources, and "already on main" evidence.

- **1.2 Unification of plan sources (mandatory):**
  - First step in Phase 1: compare prior plan artifacts and choose the canonical source.
  - Recommend consolidating to this file as source of truth and record pointer(s) in the superseded copy.
  - Capture the decision + hash + date in a mini artifact and in progress.

- **1.3 Full per-issue audit:**
  - For all 13 issues: deep comparison of acceptance criteria vs. current main (code, docs, tests, MCP surfaces, CLI).
  - Produce living artifact: `docs/harness/plans/open-issues-audit-2026-05-31.md` (or similar) with columns: Issue #, Title, Priority/Area, Apparent Status on Main, Remaining Delta, RFC 0001 Mapping, Risk/Dependencies, Suggested Owner/Wave.
  - Identify any new work discovered during audit.
  - Refresh the full dependency DAG (textual + mermaid).
  - Decide RFC 0001 fate (accept, minor revision, new ADR).
  - Clarify scope for "harness record" event shapes (constrained by RFC 0001).
- **1.4 Dependency map + WIP slice lock:**
  - Consolidar ordem de execução de todos os blocos do plano com dependências explícitas.
  - Definir um único bloco de WIP por vez até que decisões/artefatos de risco sejam fechados.
  - Registrar em `docs/harness/decisions/phase1-4-dependency-map-2026-05-31.md`.

**Phase 1 exit condition**:
- mini artifacts committed for 1.1, 1.2, 1.3, and 1.4.
- progress updated at each sub-phase end.

**Deliverables**:
- Audit document (committed).
- Updated this plan (or a Phase 1 completion note).
- Clean "ready for implementation" backlog slice.
- Explicit evidence block for #27 and #21 and any other partially-advanced issue before proceeding (commands + outputs + artifacts).

**Parallelism note**: This phase is mostly read-only + docs; low risk.

### Phase 2: Decision & Contract Items (Unblockers)
Focus on P0/P1 decisions that gate later implementation:
- #28 Decide local REST vs MCP-only.
- #29 Search Index v2 RFC.
- #26 Derived index health contract (post-Chroma/HNSW lessons).
- #31 Prompt compression benchmark + decision record.
- #32 Markdown/Obsidian portability design (front-loaded as design).
- Any RFC 0001 follow-ups.

**Rules**:
- Each decision produces a durable artifact (RFC, ADR in `docs/decisions/`, or updated issue with clear outcome).
- Fase 2.3 está concluída para #31:
  - [RFC 0002](../rfcs/0002-compression-benchmarks-for-context.md)
  - [Decisão](../decisions/phase2-3-compression-benchmark-2026-05-31.md)
- Goes through full harness gates (sensors + review).
- Updates affected issues and this plan.

### Phase 3: MCP Reference Generator as Source of Truth (#27 + hygiene)
- Ensure `./scripts/generate-mcp-reference.sh --check` is a hard failing step in CI, local `make ci`/`just ci`, and harness sensors.
- Remove any remaining handwritten tool counts or conflicting manual docs.
- Make drift impossible (generated file wins; PRs that touch MCP tools must update or the generator).
- This is high-leverage and relatively low-risk once the generator is solid.

### Phase 4: Core Harness Memory Product Features (#34–#37 + supporting)
This is the heart of the user's request — the RFC 0001 dogfooding layer.

**Guiding constraints** (non-negotiable):
- RFC 0001 product boundary (references > copies, provenance, summaries, no secrets/raw logs by default, curated events).
- Issue acceptance criteria (especially #36: no completion claim without verification evidence; explicit fields for goal/files/decisions/tests/risks/blockers/next-steps; GitHub/plan references).
- Build on/extend existing surfaces rather than duplicate (handoff handler, hooks + pending_injections, intelligence context builders, CLI maintenance patterns, memory types like 'decision'/'todo'/'issue').

**Suggested internal order** (with some overlap):
4.1 `harness_record` surface (#34) — event kinds (decision, handoff, verification_result, failed_attempt, risk, assumption, issue_update). MCP tool(s) + optional CLI + hook producers. Distinguishable metadata for search.
4.2 `harness_status` assembler (#35) — current objective, active issues, recent decisions, dirty files, last verification, blockers, suggested next action. Token-aware, graceful degradation when git/GitHub unavailable.
4.3 Richer `harness_handoff` (#36) — extend existing `session_land`. Concise continuation packet. Persistence option as harness record. Strong verification evidence requirement.
4.4 Verification evidence workflow (#37) — manifest convention, record command+status+summary+evidence path/hash+timestamp, linkage to status/handoff, support for negative evidence. Tests for round-trip.

**Cross-cutting**:
- Strong provenance on every event.
- Integration with existing session/project context and hooks.
- Full test matrix (unit + integration) + examples in docs.
- Dogfood the new surfaces on the implementation work (meta).
- Update generated MCP reference automatically.

**Exit for Phase 4**: Working MCP + CLI surfaces, passing gates, usable handoff/status artifacts with verification links, docs + examples, issues updated with evidence.

### Phase 5: Remaining Supporting Work
- #30 Unify token budget and token-aware chunking (area:context) — impacts intelligence + context prep paths.
- Any remaining gaps from #21 (CLI), #25 (queue hygiene), storage items after audit.
- Portability implementation follow-up from Phase 2 design (#32).

### Phase 6: Integration, Dogfooding, Closeout & Release
- Apply the new harness memory tools to the entire body of work (record decisions, handoffs between phases, verification evidence from sensors/review gates).
- Full end-to-end sensors + multi-reviewer gates on the cumulative changes.
- Close all 13 issues with clear links to PRs, review artifacts, and verification evidence (or explicit deferral).
- Update RFC 0001 status if needed.
- Optional: Demonstrate value by using the harness on a subsequent unrelated task.

---

## 5. Execution Principles (Enforced)

- **Harness on the harness work**: Every non-trivial change follows bootstrap → doctor → sensors → pre/post review gates with `REVIEW_VERDICT` marker.
- **Evidence before claims**: No "it works" without reproducible command + output captured.
- **Single-process judgment**: Implementer persona ≠ final reviewer (cross-CLI or cross-model where possible).
- **Explicit files in commits**: Never `git add .` for feature work.
- **Progress recording**: Domain changes update the active progress log + (once available) harness memory records.
- **Worktrees for parallelism**: Independent issues (e.g., a pure decision RFC vs. token unification) can proceed in separate worktrees without blocking main.
- **Verification sub-agents / check-work patterns** encouraged on larger waves.
- **No fake success**: Explicitly hunt the GATES.md fake-success patterns (local-embeddings vs CI parity, schema version drift, MCP reference staleness, unwrap in hot paths, etc.).

---

## 6. Risks & Mitigations

- **v0 not actually green** → Phase 0 is a hard gate; do not proceed to product features.
- **Issue staleness** → Mandatory Phase 1 audit before coding; close with evidence instead of re-doing landed work.
- **Scope creep** → Frozen backlog slice after Phase 1; new work goes into a follow-up wave or separate issues.
- **MCP or schema impact** → Early tool definition review + generated reference must stay green.
- **Context loss over long program** → Use the emerging `harness_status` + `harness_handoff` on the program itself.
- **Review gate friction** → Use "manual" reviewer mode + prompt files when non-interactive execution is immature; still require the `REVIEW_VERDICT` marker.

---

## 7. Success Criteria (Concrete & Verifiable)

- Phase 0: v0 sprint closed with explicit `REVIEW_VERDICT: PASS` artifact (versioned review file) + documented grpc exclusion trail (if used) + sprint artifacts + progress records updated.
- Phase 1: Published audit document with per-issue deltas vs main.
- Phase 2: All decision items have durable recorded outcomes that unblock dependents.
- Phase 3: Generator is the enforced source of truth; drift is impossible in CI.
- Phase 4: Four surfaces (`record`, `status`, `handoff`, verification workflow) exist, match RFC 0001 + issue ACs, pass full gates, produce usable artifacts with provenance and verification links.
- Cumulative: All 13 issues closed with evidence (or properly deferred). The work itself was recorded using the new harness memory features where applicable.
- No violations of harness invariants during the program.

---

## 8. Immediate Recommended Next Actions

1. Maintain this plan as canonical phase source and record each sub-phase decision:
   - `docs/harness/decisions/phase1-1-issue-snapshot-2026-05-31.md`
   - `docs/harness/decisions/phase1-2-plan-source-unification-2026-05-31.md`
   - `docs/harness/decisions/phase1-3-open-issues-audit-2026-05-31.md`
2. Keep phase evidence in:
   - `docs/harness/plans/open-issues-audit-2026-05-31.md`
   - `docs/harness/progress/2026-05-30-harness-bootstrap.md`
3. Execute Phase 2 decisions with explicit gates and artifacts once dependencies are finalized.
5. Exit plan mode with an approved, frozen wave 1 slice.
6. Begin execution with full harness process.

---

## 9. Open Questions for Clarification (High Signal)

- Primary success metric for the overall effort? (e.g., "future agents can resume work on engram with one `harness_status` call", "Engram becomes the reference Memory Manager for other projects' harnesses", "close the ENGRA-* Linear backlog", etc.)
- Hard requirement to finish v0 operational sprint to 100% PASS before any product feature code?
- When the Phase 1 audit shows that parts of #21, #27, #36 foundations, etc. are already on main, preference for (a) audit + close with evidence or (b) treat GitHub text as the spec and enhance further?
- Desired dogfooding level for Phase 4: Should the implementation of harness_record/status/handoff be used to record the decisions and handoffs of the program itself?
- Any issues that are absolute "do these first" or "do not touch these yet"?
- Comfort level with multi-worktree + subagent parallelism from the start vs. more serial waves?

---

## 10. Document Control

- This plan is the coordination artifact for the effort.
- It will be updated after Phase 0 and Phase 1, and at major wave boundaries.
- All execution happens under the rules in `docs/harness/{SPEC,INVARIANTS,GATES,CODE_REVIEW_POLICY}.md`.
- Existing similar artifact exists in the repo and is being unified here as canonical source under this plan (see Section 1.2); any duplicate should point to this file.

**Status**: Draft in plan mode. This plan (including all 4 conditional approval adjustments from user 2026-05-31) is ready for user review and approval via exit_plan_mode. No implementation changes made during planning. All mandatory harness onboarding steps were completed before this document was authored.

---

## 11. Conditional Approval Adjustments (User Feedback 2026-05-31) — All Incorporated

User-provided conditional approval required 4 explicit adjustments; all are now in plan:

- **[Alta] Close v0 with explicit PASS first (non-optional)**: Phase 0 and Next Actions enforce `review-gate.sh post harness-bootstrap` producing versioned `REVIEW_VERDICT: PASS ...` before any issue work.
- **[Alta] Formal grpc_transport exclusion trail**: Known CI limitation is captured in Current State; pass_with_exclusion must include registered known-issue + progress trail before Phase 0 is treated complete.
- **[Médio] Phase 1 sub-phasing**: Replaced monolithic audit with 1.1/1.2/1.3 mini artifacts and per-subphase progress checkpoints.
- **[Médio] Plan source unification**: Phase 1.2 explicitly mandates canonical plan-source selection and superseding copy pointer to eliminate split-brain.
- **[Baixo] Partial issue claims re-verified before use**: #27/#21 and similar partial-completion claims now require fresh evidence blocks before downstream planning assumptions.

---

*End of plan (v1).*
