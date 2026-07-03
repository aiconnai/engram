# Agent Memory Contract Implementation Plan

> **For agentic workers:** execute task-by-task. This plan is the canonical C1 plan; `.omo` drafts are research notes only.

**Goal:** Establish a safe, MCP-visible agent memory contract before adding new automation. The contract makes recall/writeback rules explicit, reuses existing Engram primitives, and prevents generated memories from becoming trusted instructions by default.

**Baseline:** C0 is landed on `main` as `74c7404 refactor(mcp): consolidate tool registry definitions`; PR #108 lifecycle predicate unification is merged as `e156810 feat(lifecycle): unify decay lifecycle predicate (#108)`. C1.0 landed on `main` as PR #114 (`e62c8a4 feat(mcp): expose agent memory contract`).

**Architecture:** Add a small read-only MCP contract surface first. Later slices reuse `dream_candidates` for pending agent writebacks, opt-in recall traces per workspace, and a daily handoff recipe. C1.0 had no schema migration; C1.1 adds migration v45 only to extend the existing `dream_candidates.kind` CHECK with `agent_writeback`.

**Tech Stack:** Rust MCP handlers, canonical MCP tool registry, protocol tests, generated `docs/MCP_TOOLS.md`.

---

## Must NOT Have

- Do not copy OB1 schemas or introduce a new writeback table in C1.0.
- Do not add a schema migration in C1.0; `SCHEMA_VERSION` stays 44.
- Do not add a new writeback table in C1.1; migration v45 only expands the
  existing `dream_candidates.kind` CHECK for `agent_writeback`.
- Do not let AI-generated memory become a trusted instruction by default.
- Do not add a second enrichment/write path around `memory_create`, `context_seed`, or existing enrichment events.
- Do not apply agent writebacks without review and explicit confirm/dry-run semantics.
- Do not make recall tracing always-on; it remains opt-in until storage growth is measured.
- Do not hand-edit `docs/MCP_TOOLS.md`; regenerate it from the registry.
- Do not build dashboard/catalog polish before governance and doctor surfaces exist.

---

## File Map

| Action | Path | Responsibility |
|---|---|---|
| Create | `src/mcp/handlers/agent_memory_contract.rs` | Static, read-only Agent Memory Contract response. |
| Modify | `src/mcp/handlers/mod.rs` | Dispatch `memory_agent_contract`. |
| Modify | `src/mcp/tools/registry.rs` | Register `memory_agent_contract` as a standard read-only tool. |
| Modify | `tests/mcp_protocol_tests.rs` | Protocol tests for listing, annotations, and contract payload. |
| Regenerate | `docs/MCP_TOOLS.md` | Generated MCP reference. |
| Modify | `docs/AI_GUIDE.md` | Human-facing usage guidance for the contract. |
| Modify | `src/storage/migrations.rs` | C1.1 migration v45 for the existing `dream_candidates` candidate kind CHECK. |
| Modify | `src/storage/dream_snapshots.rs` | Allow `agent_writeback` through storage-level candidate validation. |
| Create | `src/mcp/handlers/agent_writeback.rs` | Advanced MCP handler that creates pending writeback candidates with dry-run/confirm/evidence gates. |

---

## Task 1: Add the C1.0 read-only contract tool

- [ ] Add `memory_agent_contract` as an MCP-only, read-only, standard-tier tool.
- [ ] Return a stable JSON contract with:
  - `contract_version: "agent-memory-contract-v0"`;
  - recall surfaces: `memory_smart_retrieve`, `memory_digest`, `memory_get_public`;
  - write surfaces: `memory_create`, `memory_create_batch`, `context_seed`;
  - pending writeback plan: `dream_candidates` with kind `agent_writeback`;
  - pending-review visibility: `dream_candidate_*` review/apply tools require the Advanced tier and `dream-phase`;
  - enforcement rules: evidence-only/generated memory defaults, no trusted instruction by default, review/apply confirm or dry-run;
  - provenance surfaces: enrichment events and operational context artifacts;
  - rollout state: no schema migration, recall traces opt-in planned.
- [ ] Keep handler behavior deterministic and independent of storage state.

**QA:**

```bash
rtk cargo test --test mcp_protocol_tests memory_agent_contract
```

---

## Task 2: Register and document the MCP contract

- [ ] Add the registry definition with `readOnlyHint: true` and no destructive/open-world input schema.
- [ ] Regenerate `docs/MCP_TOOLS.md`.
- [ ] Add `docs/AI_GUIDE.md` guidance that agents should read `memory_agent_contract` before writeback automation.

**QA:**

```bash
rtk ./scripts/generate-mcp-reference.sh
rtk ./scripts/generate-mcp-reference.sh --check
```

---

## Task 3: Preserve future slice boundaries

Future work must remain separate from C1.0:

1. `agent_writeback` candidate kind on existing `dream_candidates` review/apply flow.
   The review sequence must list `dream_candidates_list`,
   `dream_candidate_get`, `dream_candidate_review`, and
   `dream_candidate_apply`, and must state that these tools require Advanced
   tool exposure (`ENGRAM_TOOL_TIER=advanced` or `all`) plus `dream-phase`.
2. Workspace opt-in recall trace setting and storage budget checks.
3. Daily agent handoff recipe using contract + doctor + provenance.
4. Thin CLI wrapper only after the MCP handler is stable.

## Task 4: C1.1 pending agent writeback candidates

- [x] Add `agent_writeback` to the existing `dream_candidates` kind validation
  and schema CHECK via migration v45.
- [x] Add Advanced-tier `memory_agent_writeback` behind the `dream-phase`
  feature gate.
- [x] Default `memory_agent_writeback` to `dry_run=true`.
- [x] Require `confirm=true` when `dry_run=false`.
- [x] Require at least one source (`source_memory_ids` or structured
  `evidence`) before creating a pending candidate.
- [x] Confirmed calls create only pending `dream_candidates` records and
  `dream_candidate_sources`; they do not mutate canonical memory.
- [x] Reuse `dream_candidate_get`, `dream_candidate_review`, and
  `dream_candidate_apply` for inspection, review, dry-run apply, and final
  canonical mutation.
- [x] Post-review hardening: dry-run/live response shapes are isomorphic,
  `agent_writeback` applies as `learning`, duplicate candidate ids return clean
  conflicts, synthetic writeback jobs complete after candidate creation,
  caller-provided jobs require writeback provenance plus pending status, and
  reserved governance metadata keys are rejected case-insensitively.
- [x] Contract payload now documents concrete validation rules and replaces the
  ambiguous forever-true migration flag with a structured v45 migration object.

**QA:**

```bash
rtk cargo test --lib storage::migrations::tests::test_dream_candidates_allow_agent_writeback_kind
rtk cargo test --features dream-phase --test mcp_protocol_tests memory_agent_writeback_tool_is_advanced_dry_run_mutating_surface
rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_requires_review_before_canonical_apply
rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_rejects_reuse_and_spoofing
```

Targeted QA status on 2026-07-03: all three commands above passed after the
handler split, plus `rtk cargo test --test mcp_protocol_tests
memory_agent_contract_dispatches_governance_contract`, `rtk cargo fmt --all
-- --check`, `rtk git diff --check`, and `rtk
./scripts/generate-mcp-reference.sh --check`.

Full local gate status on 2026-07-03: `rtk cargo check --workspace
--all-targets --locked`, `rtk cargo clippy --workspace --all-targets
--all-features --locked -- -D warnings`, `rtk cargo test --workspace
--all-targets --locked`, and `rtk bash docs/harness/bin/sensors.sh` passed.
Manual MCP stdio smoke with `--features dream-phase` verified both default
dry-run and confirmed pending-candidate creation plus `dream_candidate_get`.

---

## Verification Gate

Before PR handoff, run:

```bash
rtk cargo fmt --check
rtk cargo test --test mcp_protocol_tests memory_agent_contract
rtk ./scripts/generate-mcp-reference.sh --check
rtk bash docs/harness/bin/doctor.sh
```

If the slice modifies more than metadata and protocol tests, also run:

```bash
rtk cargo test
rtk bash docs/harness/bin/sensors.sh
```

---

## Acceptance Criteria

- `memory_agent_contract` appears in `tools/list` as read-only standard-tier MCP tool.
- Calling `memory_agent_contract` returns the governed recall/writeback/provenance rules listed above.
- Public docs and generated MCP reference include the tool.
- No schema migration and no new writeback table are introduced.
- Future writeback implementation is constrained to dream candidates with review/apply semantics.
