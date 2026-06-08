# RFC 0008: `memory_digest` Actionable Retrieval Digest

## Status

Proposed

## Tracker

Huly: `ENGRA-103`

## Context

Engram already has strong retrieval and context-building primitives:

- `memory_smart_retrieve` classifies a query and routes to search, graph, and
  project-context retrieval;
- `memory_build_context` builds a token-budgeted prompt context from hybrid
  search, optional graph expansion, timeframe filters, and memory type filters;
- `context_build_bundle` assembles scoped Operational Context for agent resume
  workflows;
- graph tools expose related memories, paths, entities, and cross-reference
  structure.

The missing product layer is a single read-only entry point that returns an
agent-ready package for a topic: concise digest, source memory IDs, relationship
signals, operational context, staleness warnings, and suggested next actions.

Today an agent needs to know which retrieval tools to compose and how to merge
their outputs. `memory_digest` should encode that composition without creating a
parallel index, storing new facts, or hiding provenance.

## Decision

Engram will add a read-only MCP tool named `memory_digest`.

`memory_digest` is an orchestrator over existing retrieval and context surfaces.
It does not create, update, expire, supersede, consolidate, or archive canonical
memories. It returns derived, lossy, provenance-backed context that helps an
agent decide what to inspect or do next.

The v1 implementation should be deterministic and local by default. It may use
extractive summarization, score grouping, metadata, graph edges, and Operational
Context sections. It must not require an LLM call.

## Product Boundary

`memory_digest` should help agents answer:

- what matters most about a topic right now;
- which memory IDs support the digest;
- which memories are related or in tension;
- which operational decisions, blockers, verification runs, or handoffs are
  relevant;
- what next actions are reasonable, and which source IDs justify them;
- what context is stale, missing, low-confidence, or intentionally omitted.

`memory_digest` should not:

- replace `memory_search`, `memory_get`, `memory_build_context`, or
  `context_build_bundle` when raw detail is needed;
- store new memories, candidates, summaries, artifacts, or graph edges;
- retrieve raw artifact content;
- auto-save chat logs or tool outputs;
- apply Dream candidates, supersessions, consolidations, or issue status
  updates;
- present derived text as source-of-truth without source IDs.

## Request Contract

Initial request shape:

```json
{
  "topic": "rate limiting rollout",
  "workspace": "default",
  "mode": "standard",
  "limit": 12,
  "related_depth": 1,
  "total_budget": 4096,
  "include_types": ["decision", "note", "project_state"],
  "timeframe": "all",
  "include_graph": true,
  "include_operational_context": true,
  "include_next_actions": true,
  "current_git_branch": "main",
  "current_commit_hash": "abc123"
}
```

Required fields:

- `topic`: non-empty natural-language topic or question.

Optional fields:

- `workspace`: memory workspace; defaults to `default`.
- `mode`: `brief`, `standard`, or `deep`; defaults to `standard`.
- `limit`: maximum source memories to surface; default `12`, range `1..=50`.
- `related_depth`: graph expansion depth; default `1`, range `0..=2`.
- `total_budget`: approximate response budget; default `4096`, range
  `512..=12000`.
- `include_types`: optional memory type allowlist.
- `timeframe`: `1h`, `24h`, `7d`, `30d`, or `all`; defaults to `all`.
- `include_graph`: include relationship edges; defaults to `true`.
- `include_operational_context`: include `context_build_bundle` sections when
  available; defaults to `true`.
- `include_next_actions`: include derived next actions; defaults to `true`.
- `current_git_branch` / `current_commit_hash`: forwarded to Operational
  Context staleness checks when provided.

Invalid enum values, empty topics, and out-of-range numeric inputs return
structured errors. The handler must not panic on malformed JSON.

## Response Contract

Initial response shape:

```json
{
  "topic": "rate limiting rollout",
  "workspace": "default",
  "mode": "standard",
  "generated_at": "2026-06-08T12:00:00Z",
  "digest": {
    "summary": "Short derived summary with source IDs.",
    "key_points": [
      {
        "text": "Auth is checked before the MCP HTTP rate limiter.",
        "source_memory_ids": [42],
        "source_context_event_ids": []
      }
    ],
    "open_questions": [
      {
        "text": "Confirm whether production dashboards expect the metric shift.",
        "source_memory_ids": [44]
      }
    ]
  },
  "top_memories": [
    {
      "id": 42,
      "memory_type": "decision",
      "preview": "Auth-before-rate-limit contract...",
      "score": 0.91,
      "why": ["smart_retrieve", "policy_priority"],
      "created_at": "2026-06-08T10:00:00Z"
    }
  ],
  "relationships": [
    {
      "from_id": 42,
      "to_id": 43,
      "edge_type": "derived_from",
      "strength": 0.75
    }
  ],
  "operational_context": {
    "decisions": [],
    "open_items": [],
    "recent_verification": [],
    "staleness_warnings": []
  },
  "next_actions": [
    {
      "action": "Inspect memory 42 before editing MCP auth docs.",
      "reason": "Highest ranked decision for the topic.",
      "source_memory_ids": [42]
    }
  ],
  "provenance": {
    "tools_or_strategies": [
      "memory_smart_retrieve",
      "memory_build_context",
      "context_build_bundle"
    ],
    "source_memory_ids": [42, 43, 44],
    "source_context_event_ids": [],
    "omitted": []
  },
  "warnings": []
}
```

The exact text of derived summaries may evolve, but these top-level fields are
the v1 contract: `topic`, `workspace`, `mode`, `generated_at`, `digest`,
`top_memories`, `relationships`, `operational_context`, `next_actions`,
`provenance`, and `warnings`.

Every durable claim in `digest.key_points`, `digest.open_questions`, and
`next_actions` must carry at least one source ID or an explicit warning that the
claim is low-confidence derived text.

## Implementation Strategy

Use a thin handler module, likely `src/mcp/handlers/digest.rs`, with these
steps:

1. Parse and validate `MemoryDigestRequest`.
2. Call the same retrieval path as `memory_smart_retrieve` for initial source
   memories and strategy audit.
3. Build a compact context with the same semantics as `memory_build_context`,
   forwarding budget, timeframe, memory types, graph inclusion, and workspace.
4. Fetch relationship edges for selected source memory IDs directly or through
   existing graph helpers.
5. Optionally call `context_build_bundle` for operational context when
   repository/task/session scope inputs are present or when
   `include_operational_context=true`.
6. Generate deterministic extractive digest sections from selected previews,
   memory types, policy/scoring metadata, and operational sections.
7. Return source IDs, strategy names, omitted-item reasons, and warnings.

The first implementation may call existing handlers internally, as
`memory_smart_retrieve` already does, if that keeps the change small. A later
refactor may extract shared retrieval helpers only after duplication becomes
material.

## MCP Surface

Add one tool:

- `memory_digest`: read-only actionable topic digest.

Registration requirements:

- `src/mcp/tools/registry.rs` defines the schema and description.
- `src/mcp/tools/mod.rs` exposes it by default without a feature gate.
- `src/mcp/handlers/mod.rs` dispatches to the new handler.
- `docs/MCP_TOOLS.md` is regenerated from code.
- `tests/mcp_protocol_tests.rs` or focused integration tests cover listing and
  calling the tool through the MCP request/response path.

## Safety And Invariants

- Read-only by contract; must use `ToolAnnotations::read_only()`.
- No schema migration in v1.
- No network I/O or LLM call in v1.
- No raw artifact content in the response.
- No canonical mutation, Dream candidate application, or consolidation.
- Source memory IDs and context event IDs remain inspectable.
- Design rationale: Chen, Su, and Chiang's "The Self-Correction Illusion"
  (arXiv:2606.05976) reports that models correct the same erroneous claim more
  reliably when it is presented as an external role, including a
  `system <memory>` block, than when it appears as the model's own thought.
  `memory_digest` therefore keeps claims externally addressable through source
  IDs instead of turning them into untraceable agent prose.
- Empty result sets return a structured empty digest, not an error.
- Malformed request inputs return structured validation errors.

## Tests

Required v1 tests:

- tool registry includes `memory_digest` and marks it read-only;
- `tools/call` with a valid topic returns parseable JSON with the required
  top-level fields;
- seeded memories appear in `top_memories` and `provenance.source_memory_ids`;
- graph-linked seeded memories produce `relationships` when
  `include_graph=true`;
- empty topic returns a validation error;
- no-result topic returns an empty digest with warnings or omitted reasons;
- `./scripts/generate-mcp-reference.sh --check` passes after docs generation.

## Non-Goals

- Document fragment trees.
- Supersession dry-run/apply.
- `memory_absorb_*` wrappers.
- New storage schema.
- New embedding backend behavior.
- SDK convenience wrappers in the first implementation PR unless needed for
  compatibility.

## Open Questions

- Should `brief` mode suppress Operational Context by default, or include a
  smaller section?
- Should `topic` and `query` both be accepted as aliases, or should v1 keep one
  canonical field?
- Should `next_actions` be strictly extractive/action templates in v1, or allow
  heuristic actions such as "inspect memory N" and "run context_build_bundle"?
- Should Operational Context be included only when scope fields are present, or
  always attempted with workspace fallback?

## Rollout

Implement behind the normal MCP surface, not a feature flag. Because this is a
new read-only tool, it is additive and should not break existing clients. The PR
must still follow MCP surface-change gates: generated reference, protocol tests,
review canvas, progress updates, and post-review before merge.
