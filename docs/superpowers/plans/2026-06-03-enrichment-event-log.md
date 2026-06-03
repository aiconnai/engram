# Enrichment Event Log Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an append-only `enrichment_events` table and wire 11 enrichment handlers to emit structured events, then expose two read-only MCP tools (`memory_enrichment_timeline`, `memory_enrichment_audit`) for querying them.

**Architecture:** New storage module `src/storage/enrichment_events.rs` provides `emit` / `emit_best_effort` / `latest_version_id`. Success events are emitted atomically inside the handler's `with_transaction`; failure events are emitted in a separate transaction in the `Err` arm. Three handlers currently using `with_connection` (`lifecycle_run`, `memory_garden`, `memory_summarize`) are converted to `with_transaction` as a prerequisite for atomicity.

**Tech Stack:** Rust, rusqlite, serde_json, uuid (already in Cargo.toml), chrono (already used).

**Spec:** `docs/superpowers/specs/2026-06-03-enrichment-event-log-design.md`

---

## File Map

| Action | Path | Responsibility |
|---|---|---|
| Create | `src/storage/enrichment_events.rs` | `EnrichmentEvent` struct, `emit`, `emit_best_effort`, `latest_version_id` |
| Modify | `src/storage/mod.rs` | Add `pub mod enrichment_events;` |
| Modify | `src/storage/migrations.rs` | Migration v40, update `SCHEMA_VERSION` to 40, update 3 hardcoded test assertions |
| Modify | `docs/SCHEMA.md` | Document `enrichment_events` table |
| Modify | `src/mcp/handlers/lifecycle.rs` | Convert `lifecycle_run` + `memory_set_lifecycle` + `retention_policy_apply` to emit |
| Modify | `src/mcp/handlers/autonomous.rs` | Convert `memory_garden` to `with_transaction` + emit |
| Modify | `src/mcp/handlers/summarize.rs` | Convert `memory_summarize` + `memory_archive_old` to `with_transaction` + emit |
| Modify | `src/mcp/handlers/auto_consolidate.rs` | `memory_consolidate_batch` + `memory_consolidate` emit |
| Modify | `src/mcp/handlers/misc.rs` | `memory_auto_tag` with `apply=true` emit |
| Modify | `src/mcp/handlers/context.rs` | `memory_extract_facts` emit |
| Modify | `src/mcp/handlers/compression.rs` | `memory_consolidate` emit (lives here at line 131) |
| Modify | `src/mcp/handlers/evolution.rs` | `memory_reflect` with `persist=true` emit |
| Create | `src/mcp/handlers/enrichment_audit.rs` | `memory_enrichment_timeline`, `memory_enrichment_audit` handlers |
| Modify | `src/mcp/handlers/mod.rs` | Wire 2 new tools |
| Modify | `src/mcp/tools.rs` | Register 2 new tools |

---

## Task 1: Migration v40 + schema version bump

**Files:**
- Modify: `src/storage/migrations.rs`
- Modify: `docs/SCHEMA.md`

- [ ] **Step 1.1: Write failing test for enrichment_events table**

Add inside `#[cfg(test)]` at the bottom of `src/storage/migrations.rs`:

```rust
#[test]
fn test_enrichment_events_table_exists() {
    let conn = in_memory_conn();
    let exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='enrichment_events'",
            [],
            |row| row.get(0),
        )
        .expect("query sqlite_master");
    assert_eq!(exists, 1, "enrichment_events table should exist after migration");
}

#[test]
fn test_enrichment_events_operation_id_not_null() {
    let conn = in_memory_conn();
    // NULL operation_id must be rejected by NOT NULL constraint
    let result = conn.execute(
        "INSERT INTO enrichment_events (operation_id, event_type, triggered_by, created_at)
         VALUES (NULL, 'test', 'test', '2026-01-01T00:00:00Z')",
        [],
    );
    assert!(result.is_err(), "NULL operation_id should be rejected");
}
```

- [ ] **Step 1.2: Run tests to confirm they fail**

```bash
cargo test test_enrichment_events --lib 2>&1 | grep -E "FAILED|error"
```
Expected: both tests fail (table does not exist yet).

- [ ] **Step 1.3: Update `SCHEMA_VERSION` constant**

In `src/storage/migrations.rs` line 8, change:
```rust
pub const SCHEMA_VERSION: i32 = 39;
```
to:
```rust
pub const SCHEMA_VERSION: i32 = 40;
```

- [ ] **Step 1.4: Add migrate_v40 function**

Add before the `#[cfg(test)]` block in `src/storage/migrations.rs`:

```rust
fn migrate_v40(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v40: Creating enrichment_events table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS enrichment_events (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            operation_id TEXT NOT NULL,
            event_type   TEXT NOT NULL,
            memory_id    INTEGER,
            version_id   INTEGER REFERENCES memory_versions(id) ON DELETE SET NULL,
            triggered_by TEXT NOT NULL,
            agent_id     TEXT,
            workspace    TEXT,
            params       TEXT NOT NULL DEFAULT '{}',
            outcome      TEXT NOT NULL DEFAULT '{}',
            status       TEXT NOT NULL DEFAULT 'completed'
                             CHECK (status IN ('completed', 'failed', 'skipped')),
            dry_run      INTEGER NOT NULL DEFAULT 0
                             CHECK (dry_run IN (0, 1)),
            created_at   TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_enrichment_by_memory
            ON enrichment_events(memory_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_type
            ON enrichment_events(event_type, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_operation
            ON enrichment_events(operation_id);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_triggered_by
            ON enrichment_events(triggered_by, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_workspace
            ON enrichment_events(workspace, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_time
            ON enrichment_events(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_version
            ON enrichment_events(version_id)
            WHERE version_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_agent
            ON enrichment_events(agent_id, created_at DESC)
            WHERE agent_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_status
            ON enrichment_events(status, created_at DESC);

        INSERT INTO schema_version (version) VALUES (40);
        "#,
    )?;

    tracing::info!("Migration v40 complete: enrichment_events table created");
    Ok(())
}
```

- [ ] **Step 1.5: Wire migrate_v40 into run_migrations**

Find the dispatch block in `run_migrations` (around line 178). The current last entry is:
```rust
    if current_version < SCHEMA_VERSION {
        migrate_v39(conn)?;
    }
```
Replace with:
```rust
    if current_version < 39 {
        migrate_v39(conn)?;
    }

    if current_version < SCHEMA_VERSION {
        migrate_v40(conn)?;
    }
```

- [ ] **Step 1.6: Update three hardcoded version assertions**

Search and replace all occurrences of `assert_eq!(version, 39` → `assert_eq!(version, 40` and `assert_eq!(SCHEMA_VERSION, 39` → `assert_eq!(SCHEMA_VERSION, 40` in `src/storage/migrations.rs`. There are three: lines 1970, 1975, and 2130.

- [ ] **Step 1.7: Run tests**

```bash
cargo test test_enrichment_events test_schema_version test_upgrade_from_v17 --lib 2>&1 | grep -E "ok|FAILED"
```
Expected: all pass.

- [ ] **Step 1.8: Update docs/SCHEMA.md**

Find the section listing tables (search for `## Tables` or the last table entry). Add after the last table entry:

```markdown
### `enrichment_events`

Append-only audit trail for automated enrichment operations (lifecycle transitions, consolidation, compression, gardening, etc.).

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `operation_id` | TEXT NOT NULL | UUID grouping single or batch operations |
| `event_type` | TEXT NOT NULL | `lifecycle_transition`, `consolidation`, `compression`, `garden`, `auto_tag`, `fact_ingest`, `evolution` |
| `memory_id` | INTEGER | No FK — preserved after hard delete |
| `version_id` | INTEGER | FK → `memory_versions(id)` ON DELETE SET NULL |
| `triggered_by` | TEXT NOT NULL | Handler name that triggered the event |
| `agent_id` | TEXT | Agent that triggered, when applicable |
| `workspace` | TEXT | Denormalized — preserved after memory delete |
| `params` | TEXT | JSON operation parameters (no sensitive data) |
| `outcome` | TEXT | JSON result (tokens_saved, ratio, etc.) |
| `status` | TEXT | `completed` \| `failed` \| `skipped` |
| `dry_run` | INTEGER | 0 or 1 |
| `created_at` | TEXT | RFC3339 UTC, filled by application |
```

- [ ] **Step 1.9: Commit**

```bash
git add src/storage/migrations.rs docs/SCHEMA.md
git commit -m "feat(ENG-1240): migration v40 — add enrichment_events table"
```

---

## Task 2: Storage module — `src/storage/enrichment_events.rs`

**Files:**
- Create: `src/storage/enrichment_events.rs`
- Modify: `src/storage/mod.rs`

- [ ] **Step 2.1: Write failing unit tests**

Create `src/storage/enrichment_events.rs` with the test module first:

```rust
//! Append-only enrichment event log (ENG-1240).

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::Result;

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
    pub status:       &'a str,
    pub dry_run:      bool,
}

pub fn emit(_conn: &Connection, _event: &EnrichmentEvent<'_>) -> Result<i64> {
    unimplemented!()
}

pub fn emit_best_effort(conn: &Connection, event: &EnrichmentEvent<'_>) -> Option<i64> {
    emit(conn, event).map_err(|e| tracing::warn!("enrichment_events emit failed: {e}")).ok()
}

pub fn latest_version_id(conn: &Connection, memory_id: i64) -> Result<Option<i64>> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM memory_versions WHERE memory_id = ?1 ORDER BY version DESC LIMIT 1",
            params![memory_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::run_migrations;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_emit_persists_all_fields() {
        let conn = test_conn();
        let event = EnrichmentEvent {
            operation_id: "op-abc-123",
            event_type:   "consolidation",
            memory_id:    Some(42),
            version_id:   None,
            triggered_by: "memory_consolidate_batch",
            agent_id:     Some("agent-x"),
            workspace:    Some("default"),
            params:       serde_json::json!({"threshold": 0.8}),
            outcome:      serde_json::json!({"merged": 3}),
            status:       "completed",
            dry_run:      false,
        };
        let id = emit(&conn, &event).expect("emit should succeed");
        assert!(id > 0);

        let row: (String, String, Option<i64>, String, Option<String>, Option<String>,
                  String, String, String, i32) = conn
            .query_row(
                "SELECT operation_id, event_type, memory_id, triggered_by, agent_id,
                         workspace, params, outcome, status, dry_run
                 FROM enrichment_events WHERE id = ?1",
                params![id],
                |r| Ok((
                    r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
                    r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?,
                    r.get(8)?, r.get(9)?,
                )),
            )
            .unwrap();

        assert_eq!(row.0, "op-abc-123");
        assert_eq!(row.1, "consolidation");
        assert_eq!(row.2, Some(42));
        assert_eq!(row.3, "memory_consolidate_batch");
        assert_eq!(row.4.as_deref(), Some("agent-x"));
        assert_eq!(row.5.as_deref(), Some("default"));
        assert_eq!(row.8, "completed");
        assert_eq!(row.9, 0);
    }

    #[test]
    fn test_emit_rejects_empty_operation_id() {
        let conn = test_conn();
        let event = EnrichmentEvent {
            operation_id: "",
            event_type:   "test",
            memory_id:    None,
            version_id:   None,
            triggered_by: "test",
            agent_id:     None,
            workspace:    None,
            params:       serde_json::json!({}),
            outcome:      serde_json::json!({}),
            status:       "completed",
            dry_run:      false,
        };
        assert!(emit(&conn, &event).is_err(), "empty operation_id must be rejected");
    }

    #[test]
    fn test_emit_best_effort_returns_none_on_bad_conn() {
        // Use a connection without the table to simulate DB error
        let conn = Connection::open_in_memory().unwrap();
        let event = EnrichmentEvent {
            operation_id: "op-1",
            event_type:   "test",
            memory_id:    None,
            version_id:   None,
            triggered_by: "test",
            agent_id:     None,
            workspace:    None,
            params:       serde_json::json!({}),
            outcome:      serde_json::json!({}),
            status:       "completed",
            dry_run:      false,
        };
        let result = emit_best_effort(&conn, &event);
        assert!(result.is_none());
    }

    #[test]
    fn test_latest_version_id_returns_none_when_no_versions() {
        let conn = test_conn();
        let result = latest_version_id(&conn, 99999).unwrap();
        assert!(result.is_none());
    }
}
```

- [ ] **Step 2.2: Run tests to confirm they fail**

```bash
cargo test enrichment_events --lib 2>&1 | grep -E "FAILED|error|unimplemented"
```
Expected: `test_emit_persists_all_fields` and `test_emit_rejects_empty_operation_id` fail (unimplemented).

- [ ] **Step 2.3: Add `pub mod enrichment_events;` to storage/mod.rs**

In `src/storage/mod.rs`, add after the `mod audit;` line:

```rust
pub mod enrichment_events;
```

- [ ] **Step 2.4: Implement `emit`**

Replace the `unimplemented!()` body in `emit`:

```rust
pub fn emit(conn: &Connection, event: &EnrichmentEvent<'_>) -> Result<i64> {
    if event.operation_id.is_empty() {
        return Err(crate::error::EngramError::Internal(
            "enrichment_events: operation_id must not be empty".into(),
        ));
    }
    let params_str = serde_json::to_string(&event.params)
        .unwrap_or_else(|_| "{}".to_string());
    let outcome_str = serde_json::to_string(&event.outcome)
        .unwrap_or_else(|_| "{}".to_string());
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO enrichment_events
             (operation_id, event_type, memory_id, version_id, triggered_by,
              agent_id, workspace, params, outcome, status, dry_run, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            event.operation_id,
            event.event_type,
            event.memory_id,
            event.version_id,
            event.triggered_by,
            event.agent_id,
            event.workspace,
            params_str,
            outcome_str,
            event.status,
            event.dry_run as i32,
            created_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}
```

- [ ] **Step 2.5: Run tests**

```bash
cargo test enrichment_events --lib 2>&1 | grep -E "ok|FAILED"
```
Expected: all 4 tests pass.

- [ ] **Step 2.6: Commit**

```bash
git add src/storage/enrichment_events.rs src/storage/mod.rs
git commit -m "feat(ENG-1240): add enrichment_events storage module with emit/emit_best_effort/latest_version_id"
```

---

## Task 3: Convert `lifecycle_run` to `with_transaction` + emit

**Files:**
- Modify: `src/mcp/handlers/lifecycle.rs`

- [ ] **Step 3.1: Write failing integration test**

In `src/mcp/handlers/lifecycle.rs`, add to the existing `#[cfg(test)]` block (or create one):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::handlers::test_helpers::make_ctx;

    #[test]
    fn test_lifecycle_run_emits_enrichment_event() {
        let ctx = make_ctx();
        // Seed a memory old enough to transition
        ctx.storage.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO memories (content, memory_type, importance, visibility, metadata,
                          valid_from, created_at, lifecycle_state, workspace)
                 VALUES ('old memory', 'note', 0.3, 'private', '{}',
                         datetime('now','-100 days'), datetime('now','-100 days'),
                         'active', 'default')",
                [],
            )?;
            Ok(())
        }).unwrap();

        let params = serde_json::json!({"workspace": "default", "dry_run": false});
        lifecycle_run(&ctx, params);

        let count: i32 = ctx.storage.with_connection(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM enrichment_events WHERE event_type = 'lifecycle_transition'
                 AND triggered_by = 'lifecycle_run'",
                [],
                |r| r.get(0),
            )?)
        }).unwrap();

        assert!(count > 0, "lifecycle_run should emit at least one enrichment event");
    }
}
```

- [ ] **Step 3.2: Run test to confirm it fails**

```bash
cargo test test_lifecycle_run_emits_enrichment_event --lib 2>&1 | grep -E "FAILED|ok"
```
Expected: FAILED.

- [ ] **Step 3.3: Add imports to lifecycle.rs**

At the top of `src/mcp/handlers/lifecycle.rs`, add:

```rust
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
```

- [ ] **Step 3.4: Emit inside `lifecycle_run`**

Find the inner loop where `lifecycle_state` is updated (inside `with_connection`). Convert the outer `with_connection` to `with_transaction` and add emit per transitioned memory. Locate the block (around line 94) that does the transition updates and add at its end, before the `Ok(json!(...))`:

```rust
// Emit enrichment event for each transitioned memory
let operation_id = uuid::Uuid::new_v4().to_string();
for mem_id in &transitioned_ids {  // collect IDs during the loop above
    emit_best_effort(conn, &EnrichmentEvent {
        operation_id: &operation_id,
        event_type:   "lifecycle_transition",
        memory_id:    Some(*mem_id),
        version_id:   None,
        triggered_by: "lifecycle_run",
        agent_id:     None,
        workspace:    workspace.map(|w| w),
        params:       serde_json::json!({"dry_run": dry_run}),
        outcome:      serde_json::json!({"new_state": "stale"}),
        status:       "completed",
        dry_run:      dry_run,
    });
}
```

> **Note:** You will need to collect `transitioned_ids: Vec<i64>` during the existing update loop. Add `let mut transitioned_ids: Vec<i64> = Vec::new();` before the loop and `transitioned_ids.push(id);` inside. Also change `with_connection` to `with_transaction`.

- [ ] **Step 3.5: Run test**

```bash
cargo test test_lifecycle_run_emits_enrichment_event --lib 2>&1 | grep -E "FAILED|ok"
cargo test lifecycle --lib 2>&1 | grep -E "FAILED|ok"
```
Expected: all pass.

- [ ] **Step 3.6: Commit**

```bash
git add src/mcp/handlers/lifecycle.rs
git commit -m "feat(ENG-1240): lifecycle_run emits enrichment_event per transition"
```

---

## Task 4: Convert `memory_garden` to `with_transaction` + emit

**Files:**
- Modify: `src/mcp/handlers/autonomous.rs`

- [ ] **Step 4.1: Write failing test**

Add to `src/mcp/handlers/autonomous.rs` `#[cfg(test)]`:

```rust
#[test]
fn test_memory_garden_emits_enrichment_event() {
    let ctx = make_ctx();
    let params = serde_json::json!({"workspace": "default"});
    memory_garden(&ctx, params);

    let count: i32 = ctx.storage.with_connection(|conn| {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM enrichment_events WHERE event_type = 'garden'
             AND triggered_by = 'memory_garden'",
            [],
            |r| r.get(0),
        )?)
    }).unwrap();

    assert_eq!(count, 1, "memory_garden should emit exactly one garden event");
}
```

- [ ] **Step 4.2: Run test to confirm it fails**

```bash
cargo test test_memory_garden_emits_enrichment_event --lib 2>&1 | grep -E "FAILED|ok"
```

- [ ] **Step 4.3: Implement**

In `src/mcp/handlers/autonomous.rs`, add import:
```rust
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
```

Find `memory_garden` (around line 201). Change `with_connection` to `with_transaction`. After `let report = gardener.garden(conn, workspace)?;`, add:

```rust
let operation_id = uuid::Uuid::new_v4().to_string();
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &operation_id,
    event_type:   "garden",
    memory_id:    None,
    version_id:   None,
    triggered_by: "memory_garden",
    agent_id:     None,
    workspace:    Some(workspace),
    params:       serde_json::json!({"dry_run": false}),
    outcome:      serde_json::json!({
        "memories_pruned": report.memories_pruned,
        "memories_merged": report.memories_merged,
        "memories_archived": report.memories_archived,
        "memories_compressed": report.memories_compressed,
        "tokens_freed": report.tokens_freed,
    }),
    status:       "completed",
    dry_run:      false,
});
```

- [ ] **Step 4.4: Run tests**

```bash
cargo test test_memory_garden_emits_enrichment_event --lib 2>&1 | grep -E "FAILED|ok"
```

- [ ] **Step 4.5: Commit**

```bash
git add src/mcp/handlers/autonomous.rs
git commit -m "feat(ENG-1240): memory_garden emits garden enrichment event"
```

---

## Task 5: Convert `memory_summarize` to `with_transaction` + emit

**Files:**
- Modify: `src/mcp/handlers/summarize.rs`

- [ ] **Step 5.1: Write failing test**

Add to `src/mcp/handlers/summarize.rs`:

```rust
#[test]
fn test_memory_summarize_emits_enrichment_event() {
    let ctx = make_ctx();
    // Create two source memories
    let ids: Vec<i64> = ctx.storage.with_transaction(|conn| {
        let mut ids = Vec::new();
        for i in 0..2 {
            conn.execute(
                "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
                 VALUES (?1, 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
                rusqlite::params![format!("source memory {i}")],
            )?;
            ids.push(conn.last_insert_rowid());
        }
        Ok(ids)
    }).unwrap();

    let params = serde_json::json!({"memory_ids": ids, "workspace": "default"});
    memory_summarize(&ctx, params);

    let count: i32 = ctx.storage.with_connection(|conn| {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM enrichment_events WHERE event_type = 'consolidation'
             AND triggered_by = 'memory_summarize'",
            [],
            |r| r.get(0),
        )?)
    }).unwrap();

    assert_eq!(count, 1, "memory_summarize should emit one consolidation event");
}
```

- [ ] **Step 5.2: Run test to confirm it fails**

```bash
cargo test test_memory_summarize_emits_enrichment_event --lib 2>&1 | grep -E "FAILED|ok"
```

- [ ] **Step 5.3: Implement**

In `src/mcp/handlers/summarize.rs`, add import:
```rust
use crate::storage::enrichment_events::{emit_best_effort, latest_version_id, EnrichmentEvent};
```

Find `memory_summarize` (around line 33). Change `with_connection` to `with_transaction`. After `create_memory(conn, &summary_input)?`, add:

```rust
let operation_id = uuid::Uuid::new_v4().to_string();
let vid = latest_version_id(conn, summary_memory.id).unwrap_or(None);
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &operation_id,
    event_type:   "consolidation",
    memory_id:    Some(summary_memory.id),
    version_id:   vid,
    triggered_by: "memory_summarize",
    agent_id:     None,
    workspace:    summary_memory.workspace.as_deref().or(Some("default")),
    params:       serde_json::json!({"source_count": memory_ids.len()}),
    outcome:      serde_json::json!({"summary_id": summary_memory.id}),
    status:       "completed",
    dry_run:      false,
});
```

- [ ] **Step 5.4: Run tests**

```bash
cargo test test_memory_summarize_emits_enrichment_event summarize --lib 2>&1 | grep -E "FAILED|ok"
```

- [ ] **Step 5.5: Commit**

```bash
git add src/mcp/handlers/summarize.rs
git commit -m "feat(ENG-1240): memory_summarize emits consolidation enrichment event"
```

---

## Task 6: Wire remaining 8 handlers

**Files:**
- Modify: `src/mcp/handlers/lifecycle.rs` (2 handlers)
- Modify: `src/mcp/handlers/auto_consolidate.rs` (2 handlers)
- Modify: `src/mcp/handlers/summarize.rs` (1 handler)
- Modify: `src/mcp/handlers/misc.rs` (1 handler)
- Modify: `src/mcp/handlers/context.rs` (1 handler)
- Modify: `src/mcp/handlers/evolution.rs` (1 handler)

For each handler below, the pattern is: add import (if not already there), add `emit_best_effort` call inside the existing transaction on the success path, and on the failure path emit with `status: "failed"` in a separate `ctx.storage.with_connection`.

**Add import to each file that doesn't already have it:**
```rust
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
```

### `lifecycle::memory_set_lifecycle` → `event_type = "lifecycle_transition"`

- [ ] **Step 6.1:** Write test asserting `memory_set_lifecycle` emits when called:

```rust
#[test]
fn test_memory_set_lifecycle_emits_event() {
    let ctx = make_ctx();
    let mem_id: i64 = ctx.storage.with_transaction(|conn| {
        conn.execute(
            "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
             VALUES ('test', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
            [],
        )?;
        Ok(conn.last_insert_rowid())
    }).unwrap();

    memory_set_lifecycle(&ctx, serde_json::json!({"memory_id": mem_id, "lifecycle_state": "archived"}));

    let count: i32 = ctx.storage.with_connection(|conn| {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM enrichment_events WHERE event_type='lifecycle_transition'
             AND triggered_by='memory_set_lifecycle' AND memory_id=?1",
            rusqlite::params![mem_id],
            |r| r.get(0),
        )?)
    }).unwrap();
    assert_eq!(count, 1);
}
```

- [ ] **Step 6.2:** Run — expect FAILED. Add emit inside `memory_set_lifecycle` success path:

```rust
let operation_id = uuid::Uuid::new_v4().to_string();
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &operation_id,
    event_type:   "lifecycle_transition",
    memory_id:    Some(id),
    version_id:   None,
    triggered_by: "memory_set_lifecycle",
    agent_id:     None,
    workspace:    None,
    params:       serde_json::json!({"lifecycle_state": lifecycle_state}),
    outcome:      serde_json::json!({}),
    status:       "completed",
    dry_run:      false,
});
```

- [ ] **Step 6.3:** Run — expect pass.

### `lifecycle::retention_policy_apply` → `event_type = "lifecycle_transition"`

- [ ] **Step 6.4:** Write test asserting an event is emitted when policy archives memories. Add emit call inside the archival loop of `retention_policy_apply`:

```rust
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &operation_id,
    event_type:   "lifecycle_transition",
    memory_id:    Some(mem_id),
    version_id:   None,
    triggered_by: "retention_policy_apply",
    agent_id:     None,
    workspace:    policy.workspace.as_deref(),
    params:       serde_json::json!({"policy_id": policy_id}),
    outcome:      serde_json::json!({"action": "archived"}),
    status:       "completed",
    dry_run:      false,
});
```

### `auto_consolidate::memory_consolidate_batch` → `event_type = "consolidation"`

- [ ] **Step 6.5:** In `memory_consolidate_batch`, generate one `operation_id` before the batch loop. For each consolidated memory, call:

```rust
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &batch_op_id,
    event_type:   "consolidation",
    memory_id:    Some(target_id),
    version_id:   None,
    triggered_by: "memory_consolidate_batch",
    agent_id:     None,
    workspace:    workspace,
    params:       serde_json::json!({"threshold": threshold}),
    outcome:      serde_json::json!({"consolidated_count": count}),
    status:       "completed",
    dry_run:      false,
});
```

### `compression::memory_consolidate` (at compression.rs:131) → `event_type = "consolidation"`

- [ ] **Step 6.6:** Add emit call after the consolidation completes. Use `latest_version_id` to fill `version_id` when the operation creates a new version.

### `summarize::memory_archive_old` → `event_type = "compression"`

- [ ] **Step 6.7:** Inside the archival loop of `memory_archive_old`, emit per archived memory:

```rust
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &operation_id,
    event_type:   "compression",
    memory_id:    Some(mem_id),
    version_id:   None,
    triggered_by: "memory_archive_old",
    agent_id:     None,
    workspace:    workspace,
    params:       serde_json::json!({"archive_days": archive_days}),
    outcome:      serde_json::json!({"archived": true}),
    status:       "completed",
    dry_run:      false,
});
```

### `misc::memory_auto_tag` — only when `apply = true` → `event_type = "auto_tag"`

- [ ] **Step 6.8:** Find the `apply == true` branch in `memory_auto_tag`. Add:

```rust
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &uuid::Uuid::new_v4().to_string(),
    event_type:   "auto_tag",
    memory_id:    Some(memory_id),
    version_id:   None,
    triggered_by: "memory_auto_tag",
    agent_id:     None,
    workspace:    None,
    params:       serde_json::json!({"apply": true, "tags_added": new_tags.len()}),
    outcome:      serde_json::json!({"tags": new_tags}),
    status:       "completed",
    dry_run:      false,
});
```

### `context::memory_extract_facts` → `event_type = "fact_ingest"`

- [ ] **Step 6.9:** After facts are persisted, emit:

```rust
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &uuid::Uuid::new_v4().to_string(),
    event_type:   "fact_ingest",
    memory_id:    Some(source_id),
    version_id:   None,
    triggered_by: "memory_extract_facts",
    agent_id:     None,
    workspace:    workspace,
    params:       serde_json::json!({"source_id": source_id}),
    outcome:      serde_json::json!({"facts_created": fact_ids.len()}),
    status:       "completed",
    dry_run:      false,
});
```

### `evolution::memory_reflect` — only when `persist = true` → `event_type = "evolution"`

- [ ] **Step 6.10:** In the `persist == true` branch, emit:

```rust
emit_best_effort(conn, &EnrichmentEvent {
    operation_id: &uuid::Uuid::new_v4().to_string(),
    event_type:   "evolution",
    memory_id:    Some(memory_id),
    version_id:   latest_version_id(conn, memory_id).unwrap_or(None),
    triggered_by: "memory_reflect",
    agent_id:     None,
    workspace:    None,
    params:       serde_json::json!({"persist": true}),
    outcome:      serde_json::json!({"reflection_id": reflection_id}),
    status:       "completed",
    dry_run:      false,
});
```

### Failure path test (spec requirement: failed events survive rollback)

- [ ] **Step 6.10b: Write and pass failure path test**

Add to `src/mcp/handlers/auto_consolidate.rs` tests:

```rust
#[test]
fn test_consolidate_emits_failed_event_on_error() {
    let ctx = make_ctx();
    // Pass a non-existent memory_id to force a failure
    let params = serde_json::json!({"memory_id": 999999});
    memory_consolidate(&ctx, params);

    // The operation failed, but we still expect a 'failed' event
    let count: i32 = ctx.storage.with_connection(|conn| {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM enrichment_events WHERE status='failed'
             AND triggered_by='memory_consolidate'",
            [],
            |r| r.get(0),
        )?)
    }).unwrap();
    assert!(count > 0, "failed operations must emit a 'failed' enrichment event");
}
```

In `memory_consolidate`, add to the `Err(e)` arm (after the failed `with_transaction`):

```rust
Err(e) => {
    // Emit failure event in a separate connection (original transaction rolled back)
    let _ = ctx.storage.with_connection(|conn| {
        emit_best_effort(conn, &EnrichmentEvent {
            operation_id: &uuid::Uuid::new_v4().to_string(),
            event_type:   "consolidation",
            memory_id:    Some(memory_id),
            version_id:   None,
            triggered_by: "memory_consolidate",
            agent_id:     None,
            workspace:    None,
            params:       serde_json::json!({}),
            outcome:      serde_json::json!({"error": e.to_string()}),
            status:       "failed",
            dry_run:      false,
        });
        Ok::<_, crate::error::EngramError>(())
    });
    json!({"error": e.to_string()})
}
```

- [ ] **Step 6.11: Run full test suite**

```bash
cargo test --lib 2>&1 | tail -5
```
Expected: no regressions.

- [ ] **Step 6.12: Commit**

```bash
git add src/mcp/handlers/lifecycle.rs src/mcp/handlers/auto_consolidate.rs \
        src/mcp/handlers/summarize.rs src/mcp/handlers/misc.rs \
        src/mcp/handlers/context.rs src/mcp/handlers/evolution.rs \
        src/mcp/handlers/compression.rs
git commit -m "feat(ENG-1240): wire remaining 8 handlers to emit enrichment events"
```

---

## Task 7: `memory_enrichment_timeline` MCP tool

**Files:**
- Create: `src/mcp/handlers/enrichment_audit.rs`
- Modify: `src/mcp/handlers/mod.rs`
- Modify: `src/mcp/tools.rs`

- [ ] **Step 7.1: Write failing test**

Create `src/mcp/handlers/enrichment_audit.rs`:

```rust
//! MCP handlers for enrichment event audit queries (ENG-1240).

use serde_json::{json, Value};
use super::HandlerContext;

pub fn memory_enrichment_timeline(_ctx: &HandlerContext, _params: Value) -> Value {
    json!({"error": "not implemented"})
}

pub fn memory_enrichment_audit(_ctx: &HandlerContext, _params: Value) -> Value {
    json!({"error": "not implemented"})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::handlers::test_helpers::make_ctx;
    use crate::storage::enrichment_events::{emit, EnrichmentEvent};

    fn seed_event(ctx: &HandlerContext, memory_id: i64, op_id: &str, status: &str) {
        ctx.storage.with_connection(|conn| {
            emit(conn, &EnrichmentEvent {
                operation_id: op_id,
                event_type:   "consolidation",
                memory_id:    Some(memory_id),
                version_id:   None,
                triggered_by: "test",
                agent_id:     None,
                workspace:    Some("default"),
                params:       serde_json::json!({}),
                outcome:      serde_json::json!({}),
                status,
                dry_run:      false,
            })
        }).unwrap();
    }

    #[test]
    fn test_timeline_returns_events_for_memory() {
        let ctx = make_ctx();
        seed_event(&ctx, 42, "op-1", "completed");
        seed_event(&ctx, 42, "op-2", "failed");
        seed_event(&ctx, 99, "op-3", "completed");  // different memory

        let result = memory_enrichment_timeline(&ctx, json!({"memory_id": 42}));
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(result["memory_id"], 42);
    }

    #[test]
    fn test_timeline_version_snapshot_is_null_when_no_version() {
        let ctx = make_ctx();
        seed_event(&ctx, 5, "op-1", "completed");

        let result = memory_enrichment_timeline(&ctx, json!({"memory_id": 5, "include_snapshots": true}));
        let events = result["events"].as_array().unwrap();
        assert!(!events.is_empty());
        assert!(events[0]["version_snapshot"].is_null());
    }

    #[test]
    fn test_timeline_respects_limit() {
        let ctx = make_ctx();
        for i in 0..5 {
            seed_event(&ctx, 10, &format!("op-{i}"), "completed");
        }
        let result = memory_enrichment_timeline(&ctx, json!({"memory_id": 10, "limit": 2}));
        assert_eq!(result["events"].as_array().unwrap().len(), 2);
    }
}
```

- [ ] **Step 7.2: Run tests to confirm they fail**

```bash
cargo test enrichment_audit --lib 2>&1 | grep -E "FAILED|error"
```

- [ ] **Step 7.3: Wire in mod.rs**

In `src/mcp/handlers/mod.rs`, add `mod enrichment_audit;` alongside other module declarations, and in the `dispatch` match:

```rust
"memory_enrichment_timeline" => enrichment_audit::memory_enrichment_timeline(ctx, params),
"memory_enrichment_audit" => enrichment_audit::memory_enrichment_audit(ctx, params),
```

- [ ] **Step 7.4: Implement `memory_enrichment_timeline`**

Replace the stub body:

```rust
pub fn memory_enrichment_timeline(ctx: &HandlerContext, params: Value) -> Value {
    let memory_id = match params.get("memory_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "memory_id is required"}),
    };
    let event_type = params.get("event_type").and_then(|v| v.as_str());
    let include_dry_runs = params.get("include_dry_runs").and_then(|v| v.as_bool()).unwrap_or(true);
    let include_snapshots = params.get("include_snapshots").and_then(|v| v.as_bool()).unwrap_or(true);
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).min(100);

    ctx.storage.with_connection(|conn| {
        let mut sql = String::from(
            "SELECT e.id, e.operation_id, e.event_type, e.triggered_by, e.agent_id,
                    e.status, e.dry_run, e.params, e.outcome, e.created_at, e.version_id,
                    mv.content, mv.version
             FROM enrichment_events e
             LEFT JOIN memory_versions mv ON e.version_id = mv.id
             WHERE e.memory_id = ?1"
        );
        let mut bind_idx = 2usize;
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(memory_id)];

        if !include_dry_runs {
            sql.push_str(" AND e.dry_run = 0");
        }
        if let Some(et) = event_type {
            sql.push_str(&format!(" AND e.event_type = ?{bind_idx}"));
            bind_values.push(Box::new(et.to_string()));
            bind_idx += 1;
        }
        sql.push_str(&format!(" ORDER BY e.created_at DESC LIMIT ?{bind_idx}"));
        bind_values.push(Box::new(limit));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(bind_values.iter().map(|b| b.as_ref())),
            |row| {
                let params_str: String = row.get(7).unwrap_or_default();
                let outcome_str: String = row.get(8).unwrap_or_default();
                let version_id: Option<i64> = row.get(10)?;
                let content: Option<String> = row.get(11)?;
                let version: Option<i64> = row.get(12)?;

                let version_snapshot = if include_snapshots {
                    match (version_id, content, version) {
                        (Some(_), Some(c), Some(v)) => json!({
                            "content_preview": c.chars().take(200).collect::<String>(),
                            "version": v
                        }),
                        _ => Value::Null,
                    }
                } else {
                    Value::Null
                };

                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "operation_id": row.get::<_, String>(1)?,
                    "event_type": row.get::<_, String>(2)?,
                    "triggered_by": row.get::<_, String>(3)?,
                    "agent_id": row.get::<_, Option<String>>(4)?,
                    "status": row.get::<_, String>(5)?,
                    "dry_run": row.get::<_, i32>(6)? == 1,
                    "params": serde_json::from_str::<Value>(&params_str).unwrap_or(json!({})),
                    "outcome": serde_json::from_str::<Value>(&outcome_str).unwrap_or(json!({})),
                    "created_at": row.get::<_, String>(9)?,
                    "version_id": version_id,
                    "version_snapshot": version_snapshot,
                }))
            },
        )?;

        let events: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
        let count = events.len();
        Ok(json!({"memory_id": memory_id, "events": events, "count": count}))
    })
    .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
```

- [ ] **Step 7.5: Run tests**

```bash
cargo test enrichment_audit --lib 2>&1 | grep -E "ok|FAILED"
```
Expected: all 3 timeline tests pass.

- [ ] **Step 7.6: Register in tools.rs**

Find where tool definitions are added (search for any existing tool registration, e.g., near `memory_events_poll`). Add:

```rust
Tool {
    name: "memory_enrichment_timeline".to_string(),
    description: "List all enrichment events for a specific memory (lifecycle transitions, consolidation, compression, etc.). Shows what automated operations affected this memory and why.".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "memory_id": {"type": "integer", "description": "Memory ID to query"},
            "event_type": {"type": "string", "description": "Filter by event type"},
            "include_dry_runs": {"type": "boolean", "default": true},
            "include_snapshots": {"type": "boolean", "default": true, "description": "Include version_snapshot inline when available"},
            "limit": {"type": "integer", "default": 20, "maximum": 100}
        },
        "required": ["memory_id"]
    }),
    annotations: Some(ToolAnnotations {
        read_only_hint: Some(true),
        destructive_hint: Some(false),
        ..Default::default()
    }),
},
```

- [ ] **Step 7.7: Commit**

```bash
git add src/mcp/handlers/enrichment_audit.rs src/mcp/handlers/mod.rs src/mcp/tools.rs
git commit -m "feat(ENG-1240): add memory_enrichment_timeline MCP tool"
```

---

## Task 8: `memory_enrichment_audit` MCP tool

**Files:**
- Modify: `src/mcp/handlers/enrichment_audit.rs`
- Modify: `src/mcp/tools.rs`

- [ ] **Step 8.1: Write failing tests**

Add to the test block in `enrichment_audit.rs`:

```rust
#[test]
fn test_audit_filter_by_status() {
    let ctx = make_ctx();
    seed_event(&ctx, 1, "op-1", "completed");
    seed_event(&ctx, 2, "op-2", "failed");
    seed_event(&ctx, 3, "op-3", "failed");

    let result = memory_enrichment_audit(&ctx, json!({"status": "failed"}));
    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e["status"] == "failed"));
}

#[test]
fn test_audit_filter_by_operation_id() {
    let ctx = make_ctx();
    seed_event(&ctx, 1, "batch-xyz", "completed");
    seed_event(&ctx, 2, "batch-xyz", "completed");
    seed_event(&ctx, 3, "other-op", "completed");

    let result = memory_enrichment_audit(&ctx, json!({"operation_id": "batch-xyz"}));
    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_audit_filters_applied_in_response() {
    let ctx = make_ctx();
    let result = memory_enrichment_audit(&ctx, json!({"status": "failed", "limit": 10}));
    assert!(result["filters_applied"].is_object());
}
```

- [ ] **Step 8.2: Run to confirm fail**

```bash
cargo test test_audit_filter --lib 2>&1 | grep -E "FAILED|ok"
```

- [ ] **Step 8.3: Implement `memory_enrichment_audit`**

```rust
pub fn memory_enrichment_audit(ctx: &HandlerContext, params: Value) -> Value {
    let event_type   = params.get("event_type").and_then(|v| v.as_str()).map(String::from);
    let triggered_by = params.get("triggered_by").and_then(|v| v.as_str()).map(String::from);
    let agent_id     = params.get("agent_id").and_then(|v| v.as_str()).map(String::from);
    let status       = params.get("status").and_then(|v| v.as_str()).map(String::from);
    let workspace    = params.get("workspace").and_then(|v| v.as_str()).map(String::from);
    let operation_id = params.get("operation_id").and_then(|v| v.as_str()).map(String::from);
    let memory_id    = params.get("memory_id").and_then(|v| v.as_i64());
    let version_id   = params.get("version_id").and_then(|v| v.as_i64());
    let dry_run      = params.get("dry_run").and_then(|v| v.as_bool());
    let since        = params.get("since").and_then(|v| v.as_str()).map(String::from);
    let until        = params.get("until").and_then(|v| v.as_str()).map(String::from);
    let order        = params.get("order").and_then(|v| v.as_str()).unwrap_or("desc");
    let limit        = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50).min(200);

    let order_clause = if order == "asc" { "ASC" } else { "DESC" };

    ctx.storage.with_connection(|conn| {
        let mut sql = String::from(
            "SELECT id, operation_id, event_type, triggered_by, agent_id,
                    status, dry_run, params, outcome, created_at, version_id, workspace, memory_id
             FROM enrichment_events WHERE 1=1"
        );
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        macro_rules! filter_str {
            ($field:expr, $val:expr) => {
                if let Some(ref v) = $val {
                    binds.push(Box::new(v.clone()));
                    sql.push_str(&format!(" AND {} = ?{}", $field, binds.len()));
                }
            };
        }
        macro_rules! filter_i64 {
            ($field:expr, $val:expr) => {
                if let Some(v) = $val {
                    binds.push(Box::new(v));
                    sql.push_str(&format!(" AND {} = ?{}", $field, binds.len()));
                }
            };
        }

        filter_str!("event_type", event_type);
        filter_str!("triggered_by", triggered_by);
        filter_str!("agent_id", agent_id);
        filter_str!("status", status);
        filter_str!("workspace", workspace);
        filter_str!("operation_id", operation_id);
        filter_i64!("memory_id", memory_id);
        filter_i64!("version_id", version_id);
        if let Some(dr) = dry_run {
            binds.push(Box::new(dr as i32));
            sql.push_str(&format!(" AND dry_run = ?{}", binds.len()));
        }
        if let Some(ref s) = since {
            binds.push(Box::new(s.clone()));
            sql.push_str(&format!(" AND created_at >= ?{}", binds.len()));
        }
        if let Some(ref u) = until {
            binds.push(Box::new(u.clone()));
            sql.push_str(&format!(" AND created_at <= ?{}", binds.len()));
        }

        sql.push_str(&format!(" ORDER BY created_at {} LIMIT ?{}", order_clause, binds.len() + 1));
        binds.push(Box::new(limit));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(binds.iter().map(|b| b.as_ref())),
            |row| {
                let params_str: String = row.get(7).unwrap_or_default();
                let outcome_str: String = row.get(8).unwrap_or_default();
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "operation_id": row.get::<_, String>(1)?,
                    "event_type": row.get::<_, String>(2)?,
                    "triggered_by": row.get::<_, String>(3)?,
                    "agent_id": row.get::<_, Option<String>>(4)?,
                    "status": row.get::<_, String>(5)?,
                    "dry_run": row.get::<_, i32>(6)? == 1,
                    "params": serde_json::from_str::<Value>(&params_str).unwrap_or(json!({})),
                    "outcome": serde_json::from_str::<Value>(&outcome_str).unwrap_or(json!({})),
                    "created_at": row.get::<_, String>(9)?,
                    "version_id": row.get::<_, Option<i64>>(10)?,
                    "workspace": row.get::<_, Option<String>>(11)?,
                    "memory_id": row.get::<_, Option<i64>>(12)?,
                }))
            },
        )?;

        let events: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
        let count = events.len();
        Ok(json!({
            "events": events,
            "count": count,
            "filters_applied": {
                "event_type": params.get("event_type"),
                "status": params.get("status"),
                "workspace": params.get("workspace"),
                "order": order,
                "limit": limit,
            }
        }))
    })
    .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
```

- [ ] **Step 8.4: Run tests**

```bash
cargo test enrichment_audit --lib 2>&1 | grep -E "ok|FAILED"
```
Expected: all 6 tests pass.

- [ ] **Step 8.5: Register in tools.rs**

```rust
Tool {
    name: "memory_enrichment_audit".to_string(),
    description: "Query enrichment events globally with filters. Use for compliance audit ('what failed in the last 24h?'), agent activity ('what did agent X do?'), or batch tracing ('show all events in operation Y').".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "event_type":   {"type": "string"},
            "triggered_by": {"type": "string"},
            "agent_id":     {"type": "string"},
            "status":       {"type": "string", "enum": ["completed", "failed", "skipped"]},
            "workspace":    {"type": "string"},
            "operation_id": {"type": "string", "description": "Retrieve all rows of a batch"},
            "memory_id":    {"type": "integer"},
            "version_id":   {"type": "integer", "description": "Which enrichment produced this snapshot?"},
            "dry_run":      {"type": "boolean"},
            "since":        {"type": "string", "description": "ISO8601 timestamp"},
            "until":        {"type": "string", "description": "ISO8601 timestamp"},
            "order":        {"type": "string", "enum": ["desc", "asc"], "default": "desc"},
            "limit":        {"type": "integer", "default": 50, "maximum": 200}
        }
    }),
    annotations: Some(ToolAnnotations {
        read_only_hint: Some(true),
        destructive_hint: Some(false),
        ..Default::default()
    }),
},
```

- [ ] **Step 8.6: Commit**

```bash
git add src/mcp/handlers/enrichment_audit.rs src/mcp/tools.rs
git commit -m "feat(ENG-1240): add memory_enrichment_audit MCP tool"
```

---

## Task 9: Protocol tests + docs

**Files:**
- Modify: `tests/mcp_protocol_tests.rs`

- [ ] **Step 9.1: Write protocol tests**

Add to `tests/mcp_protocol_tests.rs`:

```rust
#[test]
fn test_memory_enrichment_timeline_in_tools_list() {
    let handler = make_handler();
    let request = make_tools_list_request();
    let response = handler.handle_request(request);
    let tools = response["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"memory_enrichment_timeline"),
        "memory_enrichment_timeline missing from tools/list");
    assert!(names.contains(&"memory_enrichment_audit"),
        "memory_enrichment_audit missing from tools/list");
}

#[test]
fn test_memory_enrichment_timeline_call_returns_events_array() {
    let handler = make_handler();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "memory_enrichment_timeline",
            "arguments": {"memory_id": 999}
        }
    });
    let response = handler.handle_request(serde_json::from_value(request).unwrap());
    // memory_id 999 doesn't exist but should return empty events, not error
    let content = &response["result"]["content"][0]["text"];
    let body: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(body["events"].is_array());
    assert_eq!(body["memory_id"], 999);
}

#[test]
fn test_memory_enrichment_audit_call_returns_events_array() {
    let handler = make_handler();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "memory_enrichment_audit",
            "arguments": {"status": "failed", "limit": 5}
        }
    });
    let response = handler.handle_request(serde_json::from_value(request).unwrap());
    let content = &response["result"]["content"][0]["text"];
    let body: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(body["events"].is_array());
    assert!(body["filters_applied"].is_object());
}
```

- [ ] **Step 9.2: Run protocol tests**

```bash
cargo test mcp_protocol_tests 2>&1 | grep -E "ok|FAILED"
```
Expected: all 3 new tests pass alongside existing protocol tests.

- [ ] **Step 9.3: Regenerate MCP docs**

```bash
bash ./scripts/generate-mcp-reference.sh
```
Expected: `wrote docs/MCP_TOOLS.md`

- [ ] **Step 9.4: Run full sensor suite**

```bash
bash docs/harness/bin/sensors.sh
```
Expected: all gates green.

- [ ] **Step 9.5: Final commit**

```bash
git add tests/mcp_protocol_tests.rs docs/MCP_TOOLS.md
git commit -m "feat(ENG-1240): protocol tests for enrichment_audit tools + regenerate MCP_TOOLS.md"
```
