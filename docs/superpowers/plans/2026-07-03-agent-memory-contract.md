# Agent Memory Contract Implementation Plan

> **For agentic workers:** execute task-by-task. This plan is the canonical C1 plan; `.omo` drafts are research notes only.

**Goal:** Establish a safe, MCP-visible agent memory contract before adding new automation. The contract makes recall/writeback rules explicit, reuses existing Engram primitives, and prevents generated memories from becoming trusted instructions by default.

**Baseline:** C0 is landed on `main` as `74c7404 refactor(mcp): consolidate tool registry definitions`; PR #108 lifecycle predicate unification is merged as `e156810 feat(lifecycle): unify decay lifecycle predicate (#108)`.

**Architecture:** Add a small read-only MCP contract surface first. Later slices reuse `dream_candidates` for pending agent writebacks, opt-in recall traces per workspace, and a daily handoff recipe. No schema migration in the first slice; `SCHEMA_VERSION` remains 44.

**Tech Stack:** Rust MCP handlers, canonical MCP tool registry, protocol tests, generated `docs/MCP_TOOLS.md`.

---

## Must NOT Have

- Do not copy OB1 schemas or introduce a new writeback table in C1.0.
- Do not add a schema migration in C1.0; `SCHEMA_VERSION` stays 44.
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
