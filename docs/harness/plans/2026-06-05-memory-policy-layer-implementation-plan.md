# Memory Policy Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Titans/dreaming-inspired memory policy layer that scores, reinforces, decays, promotes, and explains explicit Engram memories without moving canonical truth out of SQLite, vectors, FTS, graph edges, or provenance records.

**Architecture:** Keep `memories` and existing derived indexes canonical. Add a deterministic `intelligence::memory_policy` module plus a durable `memory_policy` record keyed by `memory_id`; retrieval-time adaptation reranks already retrieved candidates and writes only bounded, auditable reinforcement/decay events. Reuse existing salience, lifecycle, feedback, retention, and enrichment-event surfaces instead of creating a parallel opaque memory system.

**Synthesis v1 decision:** Any background synthesis/dreaming pipeline must emit reviewable candidates only. It must not automatically create, update, delete, expire, or promote canonical memories. This plan may add policy primitives that a later synthesis pipeline can use, but Phase 1 does not grant automatic write authority to synthesized facts.

**Tech Stack:** Rust, SQLite/WAL via `rusqlite`, existing Engram storage query modules, MCP tool handlers, existing hybrid search/reranker stack, generated MCP docs.

---

## Scope Boundary

This plan is product work, not harness work. It must be executed as a separate feature task because it touches storage schema, intelligence modules, hooks, search ranking, MCP contracts, docs, and tests.

Do not implement learned ranking, online model updates, or latent memory persistence in Phase 1. Phase 1 is deterministic, inspectable, and auditable.

Do not implement automatic synthesized memory writes in Phase 1. A future synthesis pipeline may propose candidates, but v1 candidates must remain review-only until a human or explicit agent action accepts them through existing canonical memory APIs.

Do not silently delete stale or contradictory memories. Phase 1 demotes, archives, or explains; deletion remains explicit and provenance-backed.

Do not couple policy state to embeddings. Policy state must survive embedding rebuilds and must be removable with the canonical memory row.

Do not make `policy_rerank` the default in Phase 1. Retrieval behavior must remain backward-compatible unless the caller opts in.

## Dreaming-Lite Product Boundary

The ChatGPT memory/dreaming model is useful for Engram as an evaluation and product framing signal, not as an implementation blueprint. Engram's stricter contract is source-backed, reviewable, local-first memory: the system can synthesize candidates, but canonical truth remains explicit and inspectable.

Phase 1 of this plan implements the policy substrate only:

- Score explicit memories for salience, retention, and retrieval priority.
- Explain why a memory was promoted, demoted, reinforced, or treated as risky.
- Preserve all policy changes as SQLite rows plus append-only enrichment/audit events.
- Keep retrieval-time policy opt-in through `policy_rerank`.
- Leave background synthesis as a later layer that consumes policy/explanation primitives.

The later synthesis pipeline should follow this v1 behavior:

- Emit reviewable candidates only.
- Attach provenance to every candidate, including source memory IDs, source documents, timestamps, and triggering event.
- Classify each candidate as `new_fact`, `preference`, `project_state`, `stale_fact`, `contradiction`, or `summary`.
- Suggest one action: `create`, `update`, `supersede`, `expire`, `promote`, `demote`, or `ignore`.
- Include a policy explanation and confidence score, but require explicit approval before canonical mutation.
- Record review decisions so rejected or corrected candidates become feedback for later synthesis.

The core product question for v1 is:

```text
Can a new agent start a session and get current, source-backed, non-stale project context without rereading the whole repo?
```

Phase 1 supports that question by making memory ranking, confidence, conflict, decay, and explanation explicit. It does not yet answer the full synthesis workflow.

## Evaluation Criteria

Evaluate this work with the same three objectives that motivated the dreaming discussion, adapted to Engram:

1. **Carry forward useful context:** new agents can recover active project state, durable decisions, and relevant prior work from explicit memories without broad repo rereads.
2. **Follow preferences and constraints:** retrieval and explanation surface repo/process constraints, user preferences, active harness rules, and safety boundaries when relevant.
3. **Stay current over time:** old trip/project/session facts decay, conflict, or require review rather than continuing to dominate retrieval silently.

Minimum acceptance evidence for Phase 1:

- Default search ranking is unchanged without `policy_rerank`.
- `policy_rerank=true` can improve ordering using policy scores while preserving source-backed result content.
- `memory_explain` can explain surprising high-priority or low-priority memories using stable reason codes.
- `memory_reconcile_conflict` demotes or marks conflict risk without deleting canonical facts.
- All policy writes are auditable through `enrichment_events`.

## Existing Surfaces To Reuse

Engram already has useful building blocks:

- `src/intelligence/salience.rs` computes recency, frequency, importance, feedback, lifecycle suggestions, and salience history.
- `src/mcp/handlers/quality.rs` exposes `salience_*`, `quality_*`, conflict, duplicate, and source trust tools.
- `src/storage/feedback.rs` processes `memory_feedback`-style utility signals into utility tracking.
- `src/storage/enrichment_events.rs` is an append-only audit log suitable for policy score/reinforcement/decay events.
- `src/mcp/handlers/lifecycle.rs` and `src/storage/queries/retention.rs` already handle lifecycle state and retention policy.
- `src/mcp/handlers/search.rs` already centralizes MCP search behavior after `hybrid_search`.
- `src/hooks/` already has lifecycle hook plumbing, with `post_tool_use.rs` containing an inert auto-memory placeholder.

## File Structure

Create these files:

- `src/intelligence/memory_policy/mod.rs`: public policy types, engine facade, and re-exports.
- `src/intelligence/memory_policy/features.rs`: deterministic feature extraction from a `Memory`, current query/session context, metadata, feedback, and contradiction counts.
- `src/intelligence/memory_policy/scoring.rs`: heuristic v1 scoring for `salience_score`, `retention_score`, and `retrieval_priority`.
- `src/intelligence/memory_policy/explain.rs`: stable explanation strings and machine-readable reason codes.
- `src/intelligence/memory_policy/events.rs`: structured policy event input types for hooks, retrieval feedback, explicit remember, contradiction, promotion, and decay.
- `src/storage/queries/memory_policy.rs`: SQLite CRUD for `memory_policy`, reinforcement, contradiction, decay, and append-only audit emission.
- `src/mcp/handlers/memory_policy.rs`: MCP handlers for policy tools.

Modify these files:

- `src/intelligence/mod.rs`: export the new memory policy module.
- `src/storage/migrations.rs`: bump `SCHEMA_VERSION` from `40` to `41` and add `migrate_v41`.
- `src/storage/queries/mod.rs`: export `memory_policy` query functions.
- `src/storage/queries/core.rs`: initialize or refresh policy state when memories are created, promoted, expired, or lifecycle-mutated.
- `src/mcp/handlers/mod.rs`: route new policy tool names.
- `src/mcp/tools/memory.rs`: add public MCP schemas for policy tools.
- `src/mcp/tools/registry.rs`: update generated registry if this repository still checks it in.
- `src/mcp/handlers/search.rs`: optionally apply policy reranking after hybrid search and before response assembly.
- `src/mcp/tools/search.rs`: add `policy_rerank` and `policy_explain` request parameters to `memory_search`.
- `src/hooks/post_tool_use.rs`: convert successful memory/search/user-action hook metadata into policy events.
- `src/hooks/session_end.rs`: emit a policy event summary for session-end payloads without writing hidden facts.
- `tests/mcp_protocol_tests.rs`: cover policy tools via `tools/list` and `tools/call`.
- `src/storage/queries/tests.rs`: cover migration version and policy persistence.
- `docs/MCP_TOOLS.md`: regenerate after tool changes.
- `docs/AI_GUIDE.md`: document policy-layer behavior and auditability.
- `docs/USING_ENGRAM_IN_A_REPO.md`: explain how agents should use policy tools.
- `README.md`: add a concise product note under memory/retrieval features.
- `docs/harness/canvas/YYYY-MM-DD-memory-policy-layer.md`: required Review Canvas before post-review because this touches storage, MCP, hooks, intelligence, and search.
- `docs/harness/progress.md`: record execution summary after implementation because this is domain behavior.

## Data Model

Add one durable policy record per live memory:

```sql
CREATE TABLE IF NOT EXISTS memory_policy (
    memory_id INTEGER PRIMARY KEY,
    salience_score REAL NOT NULL DEFAULT 0.5 CHECK (salience_score >= 0.0 AND salience_score <= 1.0),
    retention_score REAL NOT NULL DEFAULT 0.5 CHECK (retention_score >= 0.0 AND retention_score <= 1.0),
    retrieval_priority REAL NOT NULL DEFAULT 0.5 CHECK (retrieval_priority >= 0.0 AND retrieval_priority <= 1.0),
    last_reinforced_at TEXT,
    reinforcement_count INTEGER NOT NULL DEFAULT 0 CHECK (reinforcement_count >= 0),
    contradiction_count INTEGER NOT NULL DEFAULT 0 CHECK (contradiction_count >= 0),
    policy_version TEXT NOT NULL DEFAULT 'heuristic-v1',
    policy_reason TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_memory_policy_retrieval_priority
    ON memory_policy(retrieval_priority DESC);

CREATE INDEX IF NOT EXISTS idx_memory_policy_retention_score
    ON memory_policy(retention_score ASC);
```

No synthesized candidate table is part of Phase 1. If a later phase adds reviewable synthesis candidates, it should use a separate table keyed independently from `memories` so proposed facts cannot be confused with accepted canonical memories. Candidate acceptance must call the existing canonical memory APIs and then create or update `memory_policy` for the accepted memory.

Use `enrichment_events` for the append-only audit trail:

```json
{
  "event_type": "memory_policy_score",
  "triggered_by": "memory_score",
  "params": {
    "policy_version": "heuristic-v1",
    "dry_run": false
  },
  "outcome": {
    "memory_id": 42,
    "salience_score": 0.78,
    "retention_score": 0.91,
    "retrieval_priority": 0.82,
    "policy_reason": "novelty:high access:reinforced contradiction:none"
  }
}
```

## Scoring Contract

Phase 1 policy version is `heuristic-v1`.

Feature ranges are normalized to `0.0..=1.0`.

`salience_score`:

```text
0.30 * novelty
+ 0.20 * recency
+ 0.20 * explicit_importance
+ 0.15 * source_confidence
+ 0.15 * utility_signal
- 0.20 * contradiction_risk
```

`retention_score`:

```text
0.25 * salience_score
+ 0.25 * reinforcement_strength
+ 0.20 * durability_signal
+ 0.15 * source_confidence
+ 0.15 * graph_centrality_proxy
- 0.25 * contradiction_risk
- 0.15 * age_decay
```

`retrieval_priority`:

```text
0.45 * hybrid_search_score
+ 0.25 * salience_score
+ 0.20 * session_relevance
+ 0.10 * retention_score
- 0.20 * contradiction_risk
```

Clamp all outputs to `0.0..=1.0`. Persist final scores, policy version, and explanation reason for writes. For retrieval-only reranking, do not mutate canonical facts; only update reinforcement if the caller explicitly records feedback or uses a mutating policy tool.

## MCP Contract

Add these tools as Phase 1 public surface:

- `memory_score`: calculate and optionally persist policy scores for one memory.
- `memory_promote`: promote a memory by policy, with optional canonical tier promotion only when explicitly requested.
- `memory_decay`: run policy decay for a workspace or one memory; default must be dry-run.
- `memory_explain`: explain the current policy record, score components, and audit trail.
- `memory_reconcile_conflict`: record contradiction/conflict handling and update policy confidence without deleting facts.

Do not add `memory_synthesis_run`, `memory_summary_get`, or candidate approval tools in Phase 1 unless this plan is explicitly split into a second feature. Those tools belong to the reviewable-candidate synthesis layer, not the policy substrate.

Keep existing tools:

- `salience_get` remains the lower-level salience view.
- `salience_decay_run` remains lifecycle decay.
- `memory_promote_to_permanent` remains the explicit tier mutation.
- `memory_explain_utility` remains utility feedback explanation.

## Task 0: Review Canvas And Execution Boundary

**Files:**

- Create: `docs/harness/canvas/YYYY-MM-DD-memory-policy-layer.md`
- Modify after implementation: `docs/harness/progress.md`

- [ ] **Step 1: Create the Review Canvas before code changes**

Use this content:

```markdown
# Review Canvas: memory-policy-layer

## Intent

Add an auditable memory policy layer that computes salience, retention, retrieval priority, promotion, decay, and conflict explanations over explicit Engram memories.

This implementation intentionally does not auto-write synthesized memories. Synthesis v1 is reviewable candidates only; this policy layer provides scoring and explanations that future candidates can reuse.

## Approaches Considered

| Approach | Outcome | Reason |
|---|---|---|
| Store learned state in model weights | Rejected | Breaks deletion, review, provenance, and reproducibility. |
| Auto-write synthesized memories | Rejected for v1 | Highest risk of stale or wrong memory pollution; reviewable candidates must come first. |
| Auto-write high-confidence synthesized memories | Deferred | Better UX later, but requires calibrated confidence, conflict handling, and review history. |
| Extend existing salience only | Rejected | Salience lacks retention and retrieval-priority policy records. |
| Add separate policy table keyed by memory ID | Accepted | Keeps canonical facts explicit and makes policy debuggable. |

## Hot Path Complexity

Search adds one optional rerank pass over returned candidates. Phase 1 must not add a full-table scan to default retrieval.

## Edge Cases

| Case | Expected Behavior |
|---|---|
| Memory has no policy row | Compute transient default and optionally upsert when mutating. |
| Memory is archived | Keep it excluded unless caller opts into archived results. |
| Memory has contradictions | Demote confidence and retrieval priority, do not delete. |
| Existing DB at schema 40 | Migration creates policy table and backfills lazily. |
| Synthesizer proposes stale project state | Candidate remains review-only; policy can explain freshness risk but cannot mutate canonical memory automatically. |

## Breakage Risk

| Surface | Risk | Mitigation |
|---|---|---|
| SQLite schema | High | Bump `SCHEMA_VERSION`, migration tests, schema version tests. |
| MCP tools | High | Protocol tests, generated reference update. |
| Search ranking | Medium | Add explicit `policy_rerank` parameter first; avoid default behavior flip. |
| Hooks | Medium | Best-effort policy events only; never abort user flow. |
| Future synthesis | High | Keep candidates separate from canonical memories and require explicit approval in v1. |
```

- [ ] **Step 2: Record the implementation as product work**

Add a short progress entry only after implementation is complete:

```markdown
## YYYY-MM-DD — Memory policy layer Phase 1

- Added deterministic `heuristic-v1` memory policy scoring with durable policy records.
- Added MCP tools for score, promote, decay, explain, and conflict reconciliation.
- Integrated optional retrieval-time policy reranking without storing truth in latent state.
- Preserved SQLite/FTS/vector/graph/provenance as canonical state.
```

## Task 1: Schema And Storage Queries

**Files:**

- Modify: `src/storage/migrations.rs`
- Create: `src/storage/queries/memory_policy.rs`
- Modify: `src/storage/queries/mod.rs`
- Modify: `src/storage/queries/tests.rs`

- [ ] **Step 1: Write storage tests first**

Add tests covering:

```rust
#[test]
fn memory_policy_record_round_trips_and_clamps_scores() {
    let storage = crate::storage::Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let memory = crate::storage::queries::create_memory(
                conn,
                &crate::types::CreateMemoryInput {
                    content: "Policy test memory".to_string(),
                    importance: Some(0.6),
                    ..Default::default()
                },
            )?;

            let record = crate::storage::queries::memory_policy::upsert_policy_record(
                conn,
                crate::storage::queries::memory_policy::PolicyRecordInput {
                    memory_id: memory.id,
                    salience_score: 1.5,
                    retention_score: -1.0,
                    retrieval_priority: 0.75,
                    policy_version: "heuristic-v1".to_string(),
                    policy_reason: "test clamp".to_string(),
                },
            )?;

            assert_eq!(record.memory_id, memory.id);
            assert_eq!(record.salience_score, 1.0);
            assert_eq!(record.retention_score, 0.0);
            assert_eq!(record.retrieval_priority, 0.75);
            assert_eq!(record.policy_version, "heuristic-v1");
            Ok(())
        })
        .expect("policy round trip");
}

#[test]
fn memory_policy_reinforcement_updates_count_and_timestamp() {
    let storage = crate::storage::Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let memory = crate::storage::queries::create_memory(
                conn,
                &crate::types::CreateMemoryInput {
                    content: "Reinforced policy memory".to_string(),
                    ..Default::default()
                },
            )?;

            crate::storage::queries::memory_policy::record_reinforcement(
                conn,
                memory.id,
                0.2,
                "test",
            )?;

            let record = crate::storage::queries::memory_policy::get_policy_record(conn, memory.id)?
                .expect("policy row");
            assert_eq!(record.reinforcement_count, 1);
            assert!(record.last_reinforced_at.is_some());
            assert!(record.retention_score > 0.5);
            Ok(())
        })
        .expect("reinforcement");
}
```

- [ ] **Step 2: Add migration v41**

Change:

```rust
pub const SCHEMA_VERSION: i32 = 41;
```

Add to `run_migrations`:

```rust
if current_version < 41 {
    migrate_v41(conn)?;
}
```

Add `migrate_v41` using the SQL from the Data Model section and finish with:

```sql
INSERT INTO schema_version (version) VALUES (41);
```

- [ ] **Step 3: Implement storage query types**

Create `src/storage/queries/memory_policy.rs` with these public shapes:

```rust
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
use crate::types::MemoryId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRecord {
    pub memory_id: MemoryId,
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub last_reinforced_at: Option<String>,
    pub reinforcement_count: i64,
    pub contradiction_count: i64,
    pub policy_version: String,
    pub policy_reason: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PolicyRecordInput {
    pub memory_id: MemoryId,
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub policy_version: String,
    pub policy_reason: String,
}

fn clamp_score(score: f32) -> f32 {
    score.clamp(0.0, 1.0)
}
```

Implement these functions:

```rust
pub fn get_policy_record(conn: &Connection, memory_id: MemoryId) -> Result<Option<PolicyRecord>>;

pub fn upsert_policy_record(
    conn: &Connection,
    input: PolicyRecordInput,
) -> Result<PolicyRecord>;

pub fn record_reinforcement(
    conn: &Connection,
    memory_id: MemoryId,
    boost: f32,
    triggered_by: &str,
) -> Result<PolicyRecord>;

pub fn record_contradiction(
    conn: &Connection,
    memory_id: MemoryId,
    triggered_by: &str,
    reason: &str,
) -> Result<PolicyRecord>;

pub fn emit_policy_event(
    conn: &Connection,
    triggered_by: &str,
    record: &PolicyRecord,
    dry_run: bool,
) {
    let operation_id = uuid::Uuid::new_v4().to_string();
    emit_best_effort(
        conn,
        &EnrichmentEvent {
            operation_id: &operation_id,
            event_type: "memory_policy_score",
            memory_id: Some(record.memory_id),
            version_id: None,
            triggered_by,
            agent_id: None,
            workspace: None,
            params: serde_json::json!({"policy_version": record.policy_version, "dry_run": dry_run}),
            outcome: serde_json::json!(record),
            status: "completed",
            dry_run,
        },
    );
}
```

- [ ] **Step 4: Export the storage module**

Add to `src/storage/queries/mod.rs`:

```rust
pub mod memory_policy;
pub use memory_policy::*;
```

- [ ] **Step 5: Run focused storage tests**

Run:

```bash
cargo test memory_policy -- --nocapture
```

Expected: policy storage tests pass.

## Task 2: Deterministic Policy Engine

**Files:**

- Create: `src/intelligence/memory_policy/mod.rs`
- Create: `src/intelligence/memory_policy/features.rs`
- Create: `src/intelligence/memory_policy/scoring.rs`
- Create: `src/intelligence/memory_policy/explain.rs`
- Create: `src/intelligence/memory_policy/events.rs`
- Modify: `src/intelligence/mod.rs`

- [ ] **Step 1: Write engine unit tests**

Create tests that assert:

```rust
#[test]
fn novelty_and_explicit_remember_raise_salience() {
    let features = PolicyFeatures {
        novelty: 0.9,
        recency: 1.0,
        explicit_importance: 0.8,
        source_confidence: 0.9,
        utility_signal: 0.5,
        contradiction_risk: 0.0,
        reinforcement_strength: 0.0,
        durability_signal: 0.7,
        graph_centrality_proxy: 0.2,
        age_decay: 0.0,
        session_relevance: 0.5,
        hybrid_search_score: 0.0,
    };
    let score = score_policy(&features);
    assert!(score.salience_score > 0.7);
    assert!(score.retention_score > 0.45);
}

#[test]
fn contradictions_reduce_all_policy_scores() {
    let mut features = PolicyFeatures::neutral();
    features.contradiction_risk = 1.0;
    let score = score_policy(&features);
    assert!(score.salience_score < 0.5);
    assert!(score.retention_score < 0.5);
}
```

- [ ] **Step 2: Define policy types**

In `mod.rs`:

```rust
pub mod events;
pub mod explain;
pub mod features;
pub mod scoring;

pub use events::{PolicyEvent, PolicyEventKind};
pub use explain::{explain_policy_score, PolicyExplanation};
pub use features::{extract_features, PolicyFeatureInput, PolicyFeatures};
pub use scoring::{score_policy, PolicyScore, POLICY_VERSION};
```

- [ ] **Step 3: Implement feature extraction**

`PolicyFeatureInput` should include:

```rust
pub struct PolicyFeatureInput<'a> {
    pub memory: &'a crate::types::Memory,
    pub existing_policy: Option<&'a crate::storage::queries::memory_policy::PolicyRecord>,
    pub event: Option<&'a crate::intelligence::memory_policy::PolicyEvent>,
    pub hybrid_search_score: Option<f32>,
    pub session_relevance: Option<f32>,
}
```

Implement `extract_features` so explicit `remember_this`, `user_correction`, `resolved_decision`, and high utility events raise novelty/salience. `contradiction` events and existing `contradiction_count` raise contradiction risk.

- [ ] **Step 4: Implement scoring exactly from the Scoring Contract**

`PolicyScore`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyScore {
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub policy_version: String,
    pub policy_reason: String,
}
```

Use `POLICY_VERSION: &str = "heuristic-v1"`.

- [ ] **Step 5: Export from intelligence**

Add to `src/intelligence/mod.rs`:

```rust
pub mod memory_policy;
pub use memory_policy::{
    explain_policy_score, extract_features, score_policy, PolicyEvent, PolicyEventKind,
    PolicyExplanation, PolicyFeatureInput, PolicyFeatures, PolicyScore,
};
```

- [ ] **Step 6: Run focused intelligence tests**

Run:

```bash
cargo test memory_policy --lib -- --nocapture
```

Expected: policy engine tests pass.

## Task 3: Policy Initialization And Reinforcement Integration

**Files:**

- Modify: `src/storage/queries/core.rs`
- Modify: `src/hooks/post_tool_use.rs`
- Modify: `src/hooks/session_end.rs`
- Add tests inside modified modules.

- [ ] **Step 1: Initialize policy record on `create_memory`**

After `let id = conn.last_insert_rowid();` and before returning `get_memory_internal`, load the new memory, extract features, score it, and upsert policy. Keep this inside the same transaction and do not call external services.

Expected behavior:

```rust
let memory = get_memory_internal(conn, id, false)?;
let features = crate::intelligence::memory_policy::extract_features(
    crate::intelligence::memory_policy::PolicyFeatureInput {
        memory: &memory,
        existing_policy: None,
        event: None,
        hybrid_search_score: None,
        session_relevance: None,
    },
);
let score = crate::intelligence::memory_policy::score_policy(&features);
let policy = crate::storage::queries::memory_policy::upsert_policy_record(
    conn,
    crate::storage::queries::memory_policy::PolicyRecordInput {
        memory_id: memory.id,
        salience_score: score.salience_score,
        retention_score: score.retention_score,
        retrieval_priority: score.retrieval_priority,
        policy_version: score.policy_version,
        policy_reason: score.policy_reason,
    },
)?;
crate::storage::queries::memory_policy::emit_policy_event(conn, "create_memory", &policy, false);
Ok(memory)
```

- [ ] **Step 2: Reinforce policy on promotion**

In `promote_to_permanent`, after canonical tier promotion succeeds, call `record_reinforcement(conn, id, 0.25, "memory_promote_to_permanent")`. This makes existing promotion visible to the policy layer without changing the semantics of `memory_promote_to_permanent`.

- [ ] **Step 3: Convert `PostToolUseHandler` to best-effort policy events**

Add optional storage to `PostToolUseHandler` like `SessionEndHandler` already does. Successful `memory_search`, `memory_get`, `memory_expand`, and explicit policy tools should reinforce returned memory IDs when IDs are present in hook metadata. This handler must never abort the hook on storage errors.

Do not infer new facts from arbitrary tool outputs in this hook. It may emit policy/reinforcement events for existing memory IDs only. Any future synthesized fact from tool output must go through the reviewable-candidate pipeline described in the Dreaming-Lite Product Boundary.

- [ ] **Step 4: Add hook tests**

Add tests that seed a memory, fire `PostToolUse` metadata with a returned memory ID, and assert `reinforcement_count` increments. Add a second test for malformed metadata that asserts `HookResult::Continue`.

- [ ] **Step 5: Run focused integration tests**

Run:

```bash
cargo test post_tool_use --lib -- --nocapture
cargo test promote_to_permanent --lib -- --nocapture
```

Expected: hook and promotion tests pass.

## Task 4: Retrieval-Time Policy Reranking

**Files:**

- Modify: `src/mcp/handlers/search.rs`
- Modify: `src/mcp/tools/search.rs`
- Add unit tests or protocol tests covering response shape.

- [ ] **Step 1: Add opt-in search parameters**

Add these to `memory_search` schema:

```json
"policy_rerank": {
  "type": "boolean",
  "default": false,
  "description": "Apply memory policy retrieval_priority as an opt-in rerank layer after hybrid search."
},
"policy_explain": {
  "type": "boolean",
  "default": false,
  "description": "Include policy score and reason for each reranked result when policy_rerank is true."
}
```

- [ ] **Step 2: Add a pure rerank helper**

Place in `src/intelligence/memory_policy/scoring.rs`:

```rust
pub fn blend_retrieval_priority(hybrid_score: f32, policy_priority: f32) -> f32 {
    (0.85 * hybrid_score + 0.15 * policy_priority).clamp(0.0, 1.0)
}
```

- [ ] **Step 3: Apply reranking in `memory_search`**

After `hybrid_search` returns and before the existing `Reranker`, if `policy_rerank` is true:

```rust
let policy_rerank = params
    .get("policy_rerank")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
let policy_explain = params
    .get("policy_explain")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
```

For each candidate, read `memory_policy` by `memory_id`. If absent, compute transient `heuristic-v1` score without persisting. Sort by blended score. Include `policy` object only when `policy_explain` is true.

- [ ] **Step 4: Keep default ranking unchanged**

Add a test that calls `memory_search` without `policy_rerank` and asserts the response shape remains compatible with existing tests.

- [ ] **Step 5: Run focused search tests**

Run:

```bash
cargo test memory_search --test mcp_protocol_tests -- --nocapture
```

Expected: existing search protocol tests pass plus policy-rerank shape test.

## Task 5: MCP Policy Tools

**Files:**

- Create: `src/mcp/handlers/memory_policy.rs`
- Modify: `src/mcp/handlers/mod.rs`
- Modify: `src/mcp/tools/memory.rs`
- Modify: `tests/mcp_protocol_tests.rs`

- [ ] **Step 1: Add MCP protocol tests first**

Cover these calls:

```json
{"name": "memory_score", "arguments": {"id": 1, "persist": true}}
{"name": "memory_promote", "arguments": {"id": 1, "canonical_tier": false}}
{"name": "memory_decay", "arguments": {"workspace": "default", "dry_run": true}}
{"name": "memory_explain", "arguments": {"id": 1}}
{"name": "memory_reconcile_conflict", "arguments": {"id": 1, "reason": "superseded by newer user correction"}}
```

Assert each returns JSON without an `"error"` key for seeded valid IDs.

- [ ] **Step 2: Implement `memory_score`**

Handler behavior:

```rust
pub fn memory_score(ctx: &HandlerContext, params: Value) -> Value {
    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };
    let persist = params.get("persist").and_then(|v| v.as_bool()).unwrap_or(false);

    ctx.storage
        .with_transaction(|conn| {
            let memory = crate::storage::queries::get_memory(conn, id)?;
            let existing = crate::storage::queries::memory_policy::get_policy_record(conn, id)?;
            let features = crate::intelligence::memory_policy::extract_features(
                crate::intelligence::memory_policy::PolicyFeatureInput {
                    memory: &memory,
                    existing_policy: existing.as_ref(),
                    event: None,
                    hybrid_search_score: None,
                    session_relevance: None,
                },
            );
            let score = crate::intelligence::memory_policy::score_policy(&features);
            if persist {
                let record = crate::storage::queries::memory_policy::upsert_policy_record(
                    conn,
                    crate::storage::queries::memory_policy::PolicyRecordInput {
                        memory_id: id,
                        salience_score: score.salience_score,
                        retention_score: score.retention_score,
                        retrieval_priority: score.retrieval_priority,
                        policy_version: score.policy_version.clone(),
                        policy_reason: score.policy_reason.clone(),
                    },
                )?;
                crate::storage::queries::memory_policy::emit_policy_event(conn, "memory_score", &record, false);
                Ok(json!({"memory_id": id, "persisted": true, "policy": record}))
            } else {
                Ok(json!({"memory_id": id, "persisted": false, "policy": score}))
            }
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
```

- [ ] **Step 3: Implement `memory_promote`**

Default behavior reinforces policy only. If `canonical_tier: true`, call existing `promote_to_permanent`. Return both `policy` and `canonical_memory` when canonical tier promotion was requested.

- [ ] **Step 4: Implement `memory_decay`**

Default `dry_run` to true. In dry-run mode compute candidate policy changes without mutation. In apply mode update `memory_policy` scores and lifecycle state only through existing lifecycle/salience semantics.

- [ ] **Step 5: Implement `memory_explain`**

Return:

```json
{
  "memory_id": 1,
  "policy": {},
  "components": {},
  "reason": "novelty:medium reinforcement:2 contradictions:0",
  "audit": {
    "latest_policy_event_count": 3
  }
}
```

- [ ] **Step 6: Implement `memory_reconcile_conflict`**

Require `id` and `reason`. Increment contradiction count and emit `memory_policy_conflict` event. Do not delete or mutate content.

- [ ] **Step 7: Wire handler dispatch**

Add module:

```rust
pub mod memory_policy;
```

Add dispatch arms:

```rust
"memory_score" => memory_policy::memory_score(ctx, params),
"memory_promote" => memory_policy::memory_promote(ctx, params),
"memory_decay" => memory_policy::memory_decay(ctx, params),
"memory_explain" => memory_policy::memory_explain(ctx, params),
"memory_reconcile_conflict" => memory_policy::memory_reconcile_conflict(ctx, params),
```

- [ ] **Step 8: Add MCP tool definitions**

Add `ToolDef` entries in `src/mcp/tools/memory.rs`. Mark `memory_score` as mutating only when `persist=true` is semantically possible; because MCP annotations are static, use mutating for `memory_score`. Mark `memory_explain` read-only. Mark `memory_decay` mutating because apply mode exists.

- [ ] **Step 9: Run MCP protocol tests**

Run:

```bash
cargo test memory_policy --test mcp_protocol_tests -- --nocapture
```

Expected: new policy tools are listed and callable.

## Task 6: Documentation And Generated Reference

**Files:**

- Modify: `docs/MCP_TOOLS.md`
- Modify: `docs/AI_GUIDE.md`
- Modify: `docs/USING_ENGRAM_IN_A_REPO.md`
- Modify: `README.md`

- [ ] **Step 1: Regenerate MCP tools reference**

Run:

```bash
./scripts/generate-mcp-reference.sh
```

Expected: `docs/MCP_TOOLS.md` includes `memory_score`, `memory_promote`, `memory_decay`, `memory_explain`, and `memory_reconcile_conflict`.

- [ ] **Step 2: Document the product boundary**

Add this statement to `docs/AI_GUIDE.md`:

```markdown
Engram's memory policy layer ranks and manages explicit memories; it does not store canonical truth in model weights or hidden session state. SQLite rows, FTS, embeddings, graph edges, and provenance/audit records remain the inspectable source of truth.
```

- [ ] **Step 3: Document agent usage**

Add guidance to `docs/USING_ENGRAM_IN_A_REPO.md`:

```markdown
Use `memory_score` before promoting uncertain context, `memory_explain` before trusting a surprising result, and `memory_reconcile_conflict` when a newer correction contradicts older memory. Use `memory_promote_to_permanent` only when you intend to change canonical retention tier.
```

- [ ] **Step 4: Add README feature summary**

Add:

```markdown
- Memory policy layer: deterministic scoring for salience, retention, retrieval priority, reinforcement, decay, and conflict demotion over explicit memories.
```

## Task 7: Full Validation And Review Gate

**Files:**

- Modify after validation: `docs/harness/progress.md`

- [ ] **Step 1: Run formatting**

Run:

```bash
cargo fmt --all -- --check
```

Expected: PASS.

- [ ] **Step 2: Run lint**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS.

- [ ] **Step 3: Run focused Rust tests**

Run:

```bash
cargo test memory_policy -- --nocapture
cargo test salience --lib -- --nocapture
cargo test memory_search --test mcp_protocol_tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Run MCP reference check**

Run:

```bash
./scripts/generate-mcp-reference.sh --check
```

Expected: PASS.

- [ ] **Step 5: Run harness doctor**

Run:

```bash
bash docs/harness/bin/doctor.sh
```

Expected: PASS.

- [ ] **Step 6: Run full local CI gate**

Run:

```bash
make ci
```

Expected: PASS.

- [ ] **Step 7: Run post review gate**

Run:

```bash
bash docs/harness/bin/review-gate.sh post memory-policy-layer
```

Expected: a reviewer artifact with `REVIEW_VERDICT: PASS`.

## Rollout Sequence

Phase 1 ships only deterministic policy:

- durable policy records
- policy explanation
- explicit promotion/decay/conflict tools
- optional retrieval reranking via `policy_rerank`
- hook-based reinforcement as best-effort, auditable metadata
- no automatic synthesized memory writes
- no candidate table yet

Phase 2 can add learned reranking:

- train outside canonical storage
- keep model output as policy scores and explanations
- keep updates bounded and logged
- retain deterministic fallback

Phase 3 can add reviewable synthesis candidates:

- generate candidates from sessions, hooks, reviews, transcripts, and documents
- classify candidates as `new_fact`, `preference`, `project_state`, `stale_fact`, `contradiction`, or `summary`
- require explicit approval before writing canonical memories
- record accept/reject/edit decisions as feedback
- expose a source-backed memory summary for session start and harness continuity

Phase 4 can add test-time adaptation:

- session-local ranking state only
- resettable per session or project
- never canonical fact mutation without provenance
- full audit emission when a score is persisted

## Self-Review

Spec coverage:

- Event capture maps to `events.rs`, hooks, and `enrichment_events`.
- Memory policy layer maps to `memory_policy` engine and `memory_policy` table.
- Canonical store remains unchanged except for policy metadata keyed to `memory_id`.
- Retrieval-time adaptation is opt-in through `policy_rerank`.
- Promotion and decay use explicit MCP tools and existing lifecycle semantics.
- New MCP tools match the requested names.

No hidden state:

- No latent model memory is introduced.
- No deletion is automatic.
- No embedding-coupled policy state is introduced.
- No synthesized candidate is automatically promoted to canonical memory in v1.

Primary residual risk:

- Existing `salience_*`, `memory_explain_utility`, and new `memory_*` policy tools overlap conceptually. Documentation must make the difference explicit: salience is a component, utility is feedback history, policy is the durable decision layer.
- Future synthesis can become unsafe if it bypasses review. Keep the v1 rule explicit: synthesis emits candidates only; canonical writes require approval.
