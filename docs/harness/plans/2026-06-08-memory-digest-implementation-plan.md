# Implementation Plan: `memory_digest`

Date: 2026-06-08
Tracker: Huly `ENGRA-103`
RFC: `docs/rfcs/0008-memory-digest.md`

## Goal

Add a read-only MCP tool, `memory_digest(topic)`, that returns one actionable
topic package for agents: concise digest, source memory IDs, related edges,
Operational Context sections, staleness/provenance signals, and suggested next
actions.

## Ground Rules

- Reuse existing retrieval/context primitives.
- Do not add storage schema in v1.
- Do not call an LLM in v1.
- Do not store, mutate, expire, supersede, consolidate, or auto-save memories.
- Do not retrieve raw artifact content.
- Preserve source IDs for every durable claim.

## Existing Surfaces To Reuse

| Surface | Current role | Digest use |
|---|---|---|
| `src/mcp/handlers/smart_retrieve.rs` | Query classification and merged retrieval | Initial source memory selection and strategy audit |
| `src/mcp/handlers/context.rs::memory_build_context` | Budgeted memory context, timeframe/type filters, graph option | Context block and selected memory IDs |
| `src/mcp/handlers/context.rs::context_build_bundle` | Operational Context resume bundle | Decisions, blockers, verification, staleness warnings |
| `src/mcp/handlers/graph.rs::memory_related` / SQL crossrefs | Memory relationships | `relationships` section |
| `src/mcp/tools/registry.rs` | Tool schema source | Register `memory_digest` |
| `tests/mcp_protocol_tests.rs` | MCP request/response coverage | Tools/list and tools/call regression |

## Files To Change

Implementation PR:

- `src/mcp/handlers/digest.rs` - new request parsing, orchestration, response
  shaping, unit tests if useful.
- `src/mcp/handlers/mod.rs` - module declaration and dispatch arm.
- `src/mcp/tools/registry.rs` - tool definition and JSON schema.
- `src/mcp/tools/mod.rs` - read-only annotation expectations if the test list is
  extended; no feature gate expected.
- `tests/mcp_protocol_tests.rs` or `tests/memory_digest_tests.rs` - behavioral
  MCP tests.
- `docs/MCP_TOOLS.md` - regenerated through
  `./scripts/generate-mcp-reference.sh`.
- `docs/harness/progress.md` and active progress log - implementation evidence.

Planning artifacts already created:

- `docs/rfcs/0008-memory-digest.md`
- `docs/harness/plans/2026-06-08-memory-digest-implementation-plan.md`
- `docs/harness/canvas/2026-06-08-memory-digest.md`

## Work Breakdown

1. **Preflight**
   - Run `bash docs/harness/bin/bootstrap.sh`.
   - Run `bash docs/harness/bin/doctor.sh`.
   - Run `bash docs/harness/bin/vc-gate.sh start ENGRA-103`.
   - Expected: clean worktree and doctor PASS.

2. **Request/Response Types**
   - Add `src/mcp/handlers/digest.rs`.
   - Define local request parsing helpers for `topic`, `workspace`, `mode`,
     `limit`, `related_depth`, `total_budget`, `include_types`, `timeframe`,
     graph/context booleans, branch, and commit.
   - Clamp numeric inputs to RFC bounds and return structured errors for invalid
     enums or empty topic.
   - Verification: `cargo test digest --lib`.

3. **Initial Retrieval**
   - Invoke `smart_retrieve::memory_smart_retrieve` with the topic, workspace,
     and limit.
   - Extract memory IDs, previews, scores when present, and strategy audit.
   - Deduplicate by memory ID while preserving order.
   - Verification: unit test seeded memories appear once in `top_memories`.

4. **Context And Graph**
   - Invoke or reuse `memory_build_context` semantics for budget, timeframe,
     type filters, related depth, and graph inclusion.
   - Fetch crossref edges for selected memory IDs when `include_graph=true`.
   - Verification: test linked memories produce relationship rows and
     `include_graph=false` suppresses them.

5. **Operational Context**
   - When `include_operational_context=true`, call `context_build_bundle` with
     compatible scope fields.
   - Include decisions, open items, recent verification, and staleness warnings
     if available; otherwise return empty arrays.
   - Verification: no Operational Context data still yields a valid empty
     section.

6. **Digest Shaping**
   - Build deterministic extractive `digest.summary`, `key_points`, and
     `open_questions` from selected previews and operational sections.
   - Build `next_actions` from high-ranked source memories and open/stale
     signals.
   - Add `provenance.tools_or_strategies`, `source_memory_ids`,
     `source_context_event_ids`, and `omitted`.
   - Verification: every `key_points[]` and `next_actions[]` entry has a source
     ID or explicit warning.

7. **MCP Registration**
   - Add the tool to `src/mcp/tools/registry.rs` with
     `ToolAnnotations::read_only()` and `ToolTier::Essential` or `Standard`.
   - Add dispatch in `src/mcp/handlers/mod.rs`.
   - Run `./scripts/generate-mcp-reference.sh`.
   - Verification:
     `./scripts/generate-mcp-reference.sh --check`.

8. **Protocol Tests**
   - Add tools/list assertion for `memory_digest` and read-only annotation.
   - Add tools/call tests for valid topic, empty topic, and no-result topic.
   - Add graph relationship regression.
   - Verification:
     `cargo test --test mcp_protocol_tests memory_digest`.

9. **Final Gates**
   - Run `cargo fmt --all -- --check`.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`.
   - Run `bash docs/harness/bin/sensors.sh`.
   - Run `bash docs/harness/bin/doctor.sh`.
   - Run `bash docs/harness/bin/review-gate.sh post ENGRA-103`.

## Parallelization

This is small enough for one implementation lane, but it can split safely:

- Lane A: handler request/response and deterministic digest shaping.
- Lane B: MCP registry/docs/protocol tests.
- Lane C: graph and Operational Context fixtures.

Merge order: A -> C -> B. `src/mcp/handlers/mod.rs` and
`src/mcp/tools/registry.rs` should have a single owner in the final integration
lane to avoid trivial conflicts.

## Acceptance Criteria

- `memory_digest` appears in `tools/list`.
- `memory_digest` is read-only and available without feature flags.
- `tools/call` returns the RFC top-level fields.
- Seeded source memories are visible by ID in `top_memories` and `provenance`.
- Relationship edges are present only when requested.
- No schema migration is introduced.
- MCP reference generation check passes.
- Progress and review artifacts record the implementation evidence.
