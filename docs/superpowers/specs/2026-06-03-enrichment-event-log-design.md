# ENG-1240 — Enrichment Event Log Design

**Date:** 2026-06-03  
**Issue:** ENG-1240 — ROADMAP: event sourcing for enrichment audit trail  
**Status:** Approved for implementation

---

## Problem

The enrichment pipeline (lifecycle transitions, consolidation, compression, evolution, gardening, auto-tagging, fact ingestion) mutates memories directly without emitting any structured events. When a memory is unexpectedly compressed, consolidated, or archived, there is no way to answer:

- Why did this change happen?
- Which agent or scheduled job triggered it?
- What were the parameters used?
- Did it succeed or fail?

`audit_log` exists in the schema but has zero call sites — it is dead code. `memory_events` is a sync queue, not an immutable audit trail. `memory_versions` stores content snapshots but not *why* a version was created.

---

## Goals

1. Append-only event log for every enrichment operation, with enough context to answer provenance questions.
2. Temporal navigation: given `memory_id`, list all enrichment events; given `operation_id`, reconstruct a batch run.
3. Failure visibility: `status = 'failed'` events are persisted even when the operation transaction rolls back.
4. Two MCP tools for querying: `memory_enrichment_timeline` (per-memory) and `memory_enrichment_audit` (global).

## Non-goals

- Cryptographic non-repudiation (handled separately by `src/attestation/`).
- Full event sourcing replay from scratch (`memory_versions` already provides state history; we add the *why*).
- Replacing `memory_events` (sync queue) or `audit_log` (user-action log).

---

## Schema — Migration v40

```sql
CREATE TABLE IF NOT EXISTS enrichment_events (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id TEXT NOT NULL,    -- UUID; always set, even for single-memory operations
    event_type   TEXT NOT NULL,
    memory_id    INTEGER,          -- no FK; preserved even after memory hard-delete
    version_id   INTEGER REFERENCES memory_versions(id) ON DELETE SET NULL,
    triggered_by TEXT NOT NULL,
    agent_id     TEXT,
    workspace    TEXT,             -- denormalized; preserved after memory delete
    params       TEXT NOT NULL DEFAULT '{}',
    outcome      TEXT NOT NULL DEFAULT '{}',
    status       TEXT NOT NULL DEFAULT 'completed'
                     CHECK (status IN ('completed', 'failed', 'skipped')),
    dry_run      INTEGER NOT NULL DEFAULT 0
                     CHECK (dry_run IN (0, 1)),
    created_at   TEXT NOT NULL     -- app fills: Utc::now().to_rfc3339()
);

CREATE INDEX idx_enrichment_by_memory
    ON enrichment_events(memory_id, created_at DESC);
CREATE INDEX idx_enrichment_by_type
    ON enrichment_events(event_type, created_at DESC);
CREATE INDEX idx_enrichment_by_operation
    ON enrichment_events(operation_id);
CREATE INDEX idx_enrichment_by_triggered_by
    ON enrichment_events(triggered_by, created_at DESC);
CREATE INDEX idx_enrichment_by_workspace
    ON enrichment_events(workspace, created_at DESC);
CREATE INDEX idx_enrichment_by_time
    ON enrichment_events(created_at DESC);
CREATE INDEX idx_enrichment_by_version
    ON enrichment_events(version_id)
    WHERE version_id IS NOT NULL;
CREATE INDEX idx_enrichment_by_agent
    ON enrichment_events(agent_id, created_at DESC)
    WHERE agent_id IS NOT NULL;
CREATE INDEX idx_enrichment_by_status
    ON enrichment_events(status, created_at DESC);
```

**Key design decisions:**

- `memory_id` has no FK — ID is preserved for audit even after hard delete.
- `operation_id` is `NOT NULL` — set on every event. Single-operation events generate their own UUID via `Uuid::new_v4().to_string()`. `emit` rejects empty strings.
- `event_type` has no CHECK constraint — new types should not require a migration.
- `created_at` is filled by the application (`Utc::now().to_rfc3339()`), not `CURRENT_TIMESTAMP`, to match the repo's RFC3339 UTC invariant.
- `workspace` is denormalized for the same reason as `memory_id`: preserve context after delete.

---

## Storage Module — `src/storage/enrichment_events.rs`

```rust
pub struct EnrichmentEvent<'a> {
    pub operation_id: &'a str,
    pub event_type:   &'a str,
    pub memory_id:    Option<i64>,
    pub version_id:   Option<i64>,
    pub triggered_by: &'a str,
    pub agent_id:     Option<&'a str>,
    pub workspace:    Option<&'a str>,
    pub params:       serde_json::Value,
    pub outcome:      serde_json::Value,
    pub status:       &'a str,   // "completed" | "failed" | "skipped"
    pub dry_run:      bool,
}

/// Insert an enrichment event; returns the new row id.
pub fn emit(conn: &Connection, event: &EnrichmentEvent<'_>) -> Result<i64>

/// Wrapper: absorbs errors, logs warn!, never panics.
/// Use this in handler success paths inside with_transaction.
pub fn emit_best_effort(conn: &Connection, event: &EnrichmentEvent<'_>) -> Option<i64>

/// Return the id of the latest memory_versions row for a given memory.
/// Only call this when the operation is known to create a memory_versions row
/// (e.g. update_memory). Most lifecycle/garden handlers mutate via direct SQL
/// and do NOT create versions — for those, set version_id = None.
pub fn latest_version_id(conn: &Connection, memory_id: i64) -> Result<Option<i64>>
```

---

## Handlers — Emission Points

### Emission rules by path

| Path | Where to emit | Transaction |
|---|---|---|
| Operation succeeds | Inside `with_transaction`, before `Ok(...)` | Atomic with operation. **Note:** `lifecycle_run`, `memory_garden`, and `memory_summarize` currently use `with_connection` — converting these to `with_transaction` is a prerequisite for atomicity on their success paths. |
| Operation fails | In the `Err(e)` arm, **outside** the failed transaction | Separate transaction |
| `dry_run = true` | Only if caller passes `record_dry_run_events: true` in tool params | Separate (no memory mutation). Handlers that set `record_dry_run_events` must document it in their tool schema. `memory_garden_preview` must not write audit rows by default — doing so would invalidate its `read_only` MCP annotation. |

### Handler list

| Handler | `event_type` | Notes |
|---|---|---|
| `lifecycle::lifecycle_run` | `lifecycle_transition` | 1 row per transitioned memory; `operation_id` groups the run |
| `lifecycle::memory_set_lifecycle` | `lifecycle_transition` | Single-memory manual transition |
| `lifecycle::retention_policy_apply` | `lifecycle_transition` | When policy executes archival |
| `auto_consolidate::memory_consolidate_batch` | `consolidation` | Batch; `operation_id` groups all rows |
| `compression::memory_consolidate` | `consolidation` | Lives in `compression.rs:131`, not `auto_consolidate`. Excludes pure dry-run. |
| `summarize::memory_summarize` | `consolidation` | When producing summary from multiple sources |
| `summarize::memory_archive_old` | `compression` | When archiving with summary |
| `misc::memory_auto_tag` | `auto_tag` | Only when `apply = true` |
| `context::memory_extract_facts` | `fact_ingest` | When persisting extracted facts |
| `autonomous::memory_garden` | `garden` | `memory_garden` only, not `memory_garden_preview` |
| `evolution::memory_reflect` | `evolution` | Only when `persist = true` |

### Handlers explicitly excluded

- `compression::memory_compress` — currently read-only (returns computed compression, does not persist).
- `evolution::memory_reflect` with `persist = false` — no state change, no event.
- `memory_garden_preview` — dry-run by semantics; no event unless `record_dry_run_events: true`.

---

## MCP Tools — `src/mcp/handlers/enrichment_audit.rs`

### `memory_enrichment_timeline`

Tier: **Standard**. Annotations: `read_only`.

```
Parameters:
  memory_id          integer  required
  event_type         string   optional filter
  include_dry_runs   boolean  default true
  include_snapshots  boolean  default true   -- includes version_snapshot inline
  limit              integer  default 20, max 100

Response:
  {
    memory_id: 123,
    events: [
      {
        id, operation_id, event_type, triggered_by, agent_id,
        status, dry_run, params, outcome, created_at, version_id,
        version_snapshot: {      -- null if version_id is null or version was deleted
          content_preview,       -- first 200 chars
          version
        } | null
      }
    ],
    count: N
  }
```

Implementation: single `LEFT JOIN memory_versions ON enrichment_events.version_id = memory_versions.id` — no N+1.

### `memory_enrichment_audit`

Tier: **Advanced**. Annotations: `read_only`.

```
Parameters:
  event_type   string   optional
  triggered_by string   optional
  agent_id     string   optional
  status       string   optional: "completed"|"failed"|"skipped"
  workspace    string   optional
  operation_id string   optional  -- retrieves entire batch
  memory_id    integer  optional  -- audit global with memory filter
  version_id   integer  optional  -- "which enrichment produced this snapshot?"
  dry_run      boolean  optional
  since        string   ISO8601 optional
  until        string   ISO8601 optional
  order        string   "desc"|"asc" default "desc"
  limit        integer  default 50, max 200

Response:
  {
    events: [...],    -- same shape as timeline, no inline version_snapshot
    count: N,
    filters_applied: {...}
  }
```

---

## Error Handling

1. **`emit_best_effort` never aborts the operation** — absorbs errors, emits `tracing::warn!`, returns `None`.
2. **`status = 'failed'` events survive rollback** — emitted in the `Err` arm using a fresh connection outside the failed transaction.
3. **Defensive JSON parsing** — `params`/`outcome` fields: if stored value is invalid JSON, return `{}` and log internally. `version_snapshot: null` if version row is missing.

---

## Testing

### Unit (`src/storage/enrichment_events.rs`)

- `emit` persists all fields correctly, including `operation_id`
- `latest_version_id` returns correct id after an update
- `emit_best_effort` returns `None` without panicking on DB error

### Integration (handler-level)

- `lifecycle_run` emits one event per transitioned memory with correct `operation_id`
- `memory_consolidate_batch` emits one event per memory, all sharing `operation_id`
- Handler that fails emits `status = 'failed'` event despite the operation rolling back
- `memory_auto_tag` with `apply = false` emits no event; with `apply = true` emits `auto_tag`

### Protocol (MCP)

- `memory_enrichment_timeline` returns correct shape including `version_snapshot`
- `memory_enrichment_audit` with `status = 'failed'` returns only failures
- Both tools appear in `tools/list` with correct tier and `read_only` annotation
- `memory_enrichment_audit` with `operation_id` returns all rows of a batch

---

## Propagation Checklist

Every PR implementing this feature must touch:

- [ ] `src/storage/enrichment_events.rs` (new module)
- [ ] `src/storage/mod.rs` (re-export)
- [ ] `src/storage/migrations.rs` (migration v40)
- [ ] `src/mcp/handlers/enrichment_audit.rs` (new handler file)
- [ ] `src/mcp/handlers/mod.rs` (wire 2 new tools)
- [ ] `src/mcp/tools.rs` (register 2 new tools)
- [ ] 11 emitting handlers (handler list above), including converting `lifecycle_run`, `memory_garden`, `memory_summarize` from `with_connection` to `with_transaction`
- [ ] `docs/MCP_TOOLS.md` via `./scripts/generate-mcp-reference.sh` (canonical harness wrapper)
- [ ] Unit + integration + protocol tests
