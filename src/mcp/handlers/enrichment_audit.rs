//! Enrichment audit tool handlers (ENG-1240).
//!
//! Provides two MCP tools:
//! - `memory_enrichment_timeline` – per-memory enrichment history
//! - `memory_enrichment_audit`    – global enrichment event query with filters

use rusqlite::params;
use serde_json::{json, Value};

use super::HandlerContext;

// ── memory_enrichment_timeline ────────────────────────────────────────────────

/// Return all enrichment events for a specific memory, ordered by creation
/// time descending.
pub fn memory_enrichment_timeline(ctx: &HandlerContext, params: Value) -> Value {
    let memory_id = match params.get("memory_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "memory_id is required"}),
    };
    let event_type = params
        .get("event_type")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let include_dry_runs = params
        .get("include_dry_runs")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let include_snapshots = params
        .get("include_snapshots")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20)
        .min(100) as i64;

    ctx.storage
        .with_connection(|conn| {
            // Build a parameterised query to avoid string injection.
            // rusqlite does not support dynamic IN-clauses, so we build the
            // WHERE clause manually and bind all values positionally.
            let mut conditions: Vec<String> = vec!["e.memory_id = ?1".to_string()];
            let mut bind_idx: usize = 2;

            if let Some(ref et) = event_type {
                conditions.push(format!("e.event_type = ?{bind_idx}"));
                bind_idx += 1;
                let _ = et; // used below via bind_values
            }
            if !include_dry_runs {
                conditions.push("e.dry_run = 0".to_string());
            }
            // NOTE: include_snapshots=false suppresses version_snapshot data in
            // the row output, but does NOT filter rows by event_type.

            let _ = bind_idx; // silence warning
            let where_clause = conditions.join(" AND ");
            let sql = format!(
                "SELECT e.id, e.operation_id, e.event_type, e.memory_id, e.version_id,
                        e.triggered_by, e.agent_id, e.workspace, e.params, e.outcome,
                        e.status, e.dry_run, e.created_at,
                        mv.content, mv.version
                 FROM enrichment_events e
                 LEFT JOIN memory_versions mv ON mv.id = e.version_id
                 WHERE {where_clause}
                 ORDER BY e.created_at DESC
                 LIMIT ?{next}",
                next = if event_type.is_some() { 3 } else { 2 }
            );

            let mut stmt = conn.prepare(&sql)?;

            let rows: Vec<Value> = if let Some(ref et) = event_type {
                stmt.query_map(
                    params![memory_id, et.as_str(), limit],
                    |row| row_to_json_with_snapshot(row, include_snapshots),
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?
            } else {
                stmt.query_map(params![memory_id, limit], |row| {
                    row_to_json_with_snapshot(row, include_snapshots)
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
            };

            Ok(json!({
                "memory_id": memory_id,
                "count": rows.len(),
                "events": rows,
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── memory_enrichment_audit ───────────────────────────────────────────────────

/// Query enrichment events globally with optional filters.
pub fn memory_enrichment_audit(ctx: &HandlerContext, params: Value) -> Value {
    let event_type = params
        .get("event_type")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let triggered_by = params
        .get("triggered_by")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let agent_id = params
        .get("agent_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let status = params
        .get("status")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let operation_id = params
        .get("operation_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let memory_id = params.get("memory_id").and_then(|v| v.as_i64());
    let version_id = params.get("version_id").and_then(|v| v.as_i64());
    let dry_run = params.get("dry_run").and_then(|v| v.as_bool());
    let since = params
        .get("since")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let until = params
        .get("until")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let order = params
        .get("order")
        .and_then(|v| v.as_str())
        .unwrap_or("desc");
    let order_dir = if order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50)
        .min(200) as i64;

    // Validate status enum
    if let Some(ref s) = status {
        if !matches!(s.as_str(), "completed" | "failed" | "skipped") {
            return json!({"error": "status must be one of: completed, failed, skipped"});
        }
    }

    ctx.storage
        .with_connection(|conn| {
            // We collect conditions and bound values as serde_json Values so
            // we can pass them to rusqlite as a slice of dyn ToSql.
            let mut conditions: Vec<String> = Vec::new();
            // Positional bind index starts at 1; limit is always last.
            let mut pos: usize = 1;

            // Helper macro to push a condition and advance pos.
            macro_rules! cond {
                ($col:expr) => {{
                    conditions.push(format!("{} = ?{pos}", $col));
                    pos += 1;
                }};
                ($col:expr, $op:expr) => {{
                    conditions.push(format!("{} {} ?{pos}", $col, $op));
                    pos += 1;
                }};
            }

            if event_type.is_some()   { cond!("event_type"); }
            if triggered_by.is_some() { cond!("triggered_by"); }
            if agent_id.is_some()     { cond!("agent_id"); }
            if status.is_some()       { cond!("status"); }
            if workspace.is_some()    { cond!("workspace"); }
            if operation_id.is_some() { cond!("operation_id"); }
            if memory_id.is_some()    { cond!("memory_id"); }
            if version_id.is_some()   { cond!("version_id"); }
            if dry_run.is_some()      { cond!("dry_run"); }
            if since.is_some()        { cond!("created_at", ">="); }
            if until.is_some()        { cond!("created_at", "<="); }

            let limit_pos = pos;
            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            let sql = format!(
                "SELECT id, operation_id, event_type, memory_id, version_id,
                        triggered_by, agent_id, workspace, params, outcome,
                        status, dry_run, created_at
                 FROM enrichment_events
                 {where_clause}
                 ORDER BY created_at {order_dir}
                 LIMIT ?{limit_pos}"
            );

            let mut stmt = conn.prepare(&sql)?;

            // Bind values in the same order as conditions.
            use rusqlite::types::ToSql;
            let mut bind_vals: Vec<Box<dyn ToSql>> = Vec::new();
            if let Some(ref v) = event_type   { bind_vals.push(Box::new(v.clone())); }
            if let Some(ref v) = triggered_by { bind_vals.push(Box::new(v.clone())); }
            if let Some(ref v) = agent_id     { bind_vals.push(Box::new(v.clone())); }
            if let Some(ref v) = status       { bind_vals.push(Box::new(v.clone())); }
            if let Some(ref v) = workspace    { bind_vals.push(Box::new(v.clone())); }
            if let Some(ref v) = operation_id { bind_vals.push(Box::new(v.clone())); }
            if let Some(v)     = memory_id    { bind_vals.push(Box::new(v)); }
            if let Some(v)     = version_id   { bind_vals.push(Box::new(v)); }
            if let Some(v)     = dry_run      { bind_vals.push(Box::new(v as i32)); }
            if let Some(ref v) = since        { bind_vals.push(Box::new(v.clone())); }
            if let Some(ref v) = until        { bind_vals.push(Box::new(v.clone())); }
            bind_vals.push(Box::new(limit));

            let refs: Vec<&dyn ToSql> = bind_vals.iter().map(|b| b.as_ref()).collect();

            let rows: Vec<Value> = stmt
                .query_map(refs.as_slice(), row_to_json)?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            let count = rows.len();

            // Build filters_applied map from all active filters.
            let mut applied = serde_json::Map::new();
            if let Some(ref v) = event_type     { applied.insert("event_type".into(),    json!(v)); }
            if let Some(ref v) = triggered_by   { applied.insert("triggered_by".into(),  json!(v)); }
            if let Some(ref v) = agent_id       { applied.insert("agent_id".into(),      json!(v)); }
            if let Some(ref v) = status         { applied.insert("status".into(),        json!(v)); }
            if let Some(ref v) = workspace      { applied.insert("workspace".into(),     json!(v)); }
            if let Some(ref v) = operation_id   { applied.insert("operation_id".into(),  json!(v)); }
            if let Some(v)     = memory_id      { applied.insert("memory_id".into(),     json!(v)); }
            if let Some(v)     = version_id     { applied.insert("version_id".into(),    json!(v)); }
            if let Some(v)     = dry_run        { applied.insert("dry_run".into(),       json!(v)); }
            if let Some(ref v) = since          { applied.insert("since".into(),         json!(v)); }
            if let Some(ref v) = until          { applied.insert("until".into(),         json!(v)); }
            applied.insert("limit".into(), json!(limit));
            applied.insert("order".into(), json!(order));

            Ok(json!({
                "events": rows,
                "count": count,
                "filters_applied": applied,
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── shared row mappers ────────────────────────────────────────────────────────

/// Row mapper for queries that SELECT 13 base columns from enrichment_events
/// (no JOIN). Used by `memory_enrichment_audit`.
fn row_to_json(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let params_str: String = row.get(8)?;
    let outcome_str: String = row.get(9)?;
    let params_val: Value = serde_json::from_str(&params_str).unwrap_or(Value::Null);
    let outcome_val: Value = serde_json::from_str(&outcome_str).unwrap_or(Value::Null);
    let dry_run_int: i32 = row.get(11)?;
    Ok(json!({
        "id":           row.get::<_, i64>(0)?,
        "operation_id": row.get::<_, String>(1)?,
        "event_type":   row.get::<_, String>(2)?,
        "memory_id":    row.get::<_, Option<i64>>(3)?,
        "version_id":   row.get::<_, Option<i64>>(4)?,
        "triggered_by": row.get::<_, String>(5)?,
        "agent_id":     row.get::<_, Option<String>>(6)?,
        "workspace":    row.get::<_, Option<String>>(7)?,
        "params":       params_val,
        "outcome":      outcome_val,
        "status":       row.get::<_, String>(10)?,
        "dry_run":      dry_run_int != 0,
        "created_at":   row.get::<_, String>(12)?,
    }))
}

/// Row mapper for queries that SELECT 13 base columns + mv.content (13) +
/// mv.version (14) via LEFT JOIN memory_versions.  Used by
/// `memory_enrichment_timeline`.
fn row_to_json_with_snapshot(
    row: &rusqlite::Row<'_>,
    include_snapshots: bool,
) -> rusqlite::Result<Value> {
    let params_str: String = row.get(8)?;
    let outcome_str: String = row.get(9)?;
    let params_val: Value = serde_json::from_str(&params_str).unwrap_or(Value::Null);
    let outcome_val: Value = serde_json::from_str(&outcome_str).unwrap_or(Value::Null);
    let dry_run_int: i32 = row.get(11)?;

    let version_id: Option<i64> = row.get(4)?;
    let content: Option<String> = row.get(13)?;
    let version_num: Option<i64> = row.get(14)?;

    let version_snapshot = match (version_id, content, version_num) {
        (Some(_), Some(c), Some(v)) if include_snapshots => json!({
            "content_preview": c.chars().take(200).collect::<String>(),
            "version": v,
        }),
        _ => Value::Null,
    };

    Ok(json!({
        "id":               row.get::<_, i64>(0)?,
        "operation_id":     row.get::<_, String>(1)?,
        "event_type":       row.get::<_, String>(2)?,
        "memory_id":        row.get::<_, Option<i64>>(3)?,
        "version_id":       version_id,
        "triggered_by":     row.get::<_, String>(5)?,
        "agent_id":         row.get::<_, Option<String>>(6)?,
        "workspace":        row.get::<_, Option<String>>(7)?,
        "params":           params_val,
        "outcome":          outcome_val,
        "status":           row.get::<_, String>(10)?,
        "dry_run":          dry_run_int != 0,
        "created_at":       row.get::<_, String>(12)?,
        "version_snapshot": version_snapshot,
    }))
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::handlers::HandlerContext;
    use crate::storage::enrichment_events::{emit, EnrichmentEvent};

    fn ctx_with_event() -> (HandlerContext, i64) {
        use crate::embedding::{create_embedder, EmbeddingCache};
        use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
        use crate::storage::Storage;
        use crate::types::EmbeddingConfig;
        use parking_lot::Mutex;
        use std::sync::Arc;

        let storage = Storage::open_in_memory().expect("in-memory storage");
        let memory_id: i64 = storage
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO memories (content, memory_type, importance, visibility,
                                          metadata, valid_from)
                     VALUES ('test', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .expect("insert memory");

        storage
            .with_connection(|conn| {
                emit(
                    conn,
                    &EnrichmentEvent {
                        operation_id: "op-test-1",
                        event_type: "consolidation",
                        memory_id: Some(memory_id),
                        version_id: None,
                        triggered_by: "memory_consolidate",
                        agent_id: Some("agent-a"),
                        workspace: Some("default"),
                        params: serde_json::json!({}),
                        outcome: serde_json::json!({"ok": true}),
                        status: "completed",
                        dry_run: false,
                    },
                )?;
                Ok(())
            })
            .expect("emit event");

        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        let ctx = HandlerContext {
            storage,
            embedder,
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("langfuse runtime"),
            ),
        };
        (ctx, memory_id)
    }

    #[test]
    fn test_memory_enrichment_timeline_returns_events() {
        let (ctx, memory_id) = ctx_with_event();
        let result = memory_enrichment_timeline(
            &ctx,
            serde_json::json!({"memory_id": memory_id}),
        );
        assert_eq!(result["memory_id"], memory_id);
        assert_eq!(result["count"], 1);
        let events = result["events"].as_array().expect("events array");
        assert_eq!(events[0]["event_type"], "consolidation");
        assert_eq!(events[0]["status"], "completed");
        assert_eq!(events[0]["dry_run"], false);
    }

    #[test]
    fn test_memory_enrichment_timeline_requires_memory_id() {
        let (ctx, _) = ctx_with_event();
        let result = memory_enrichment_timeline(&ctx, serde_json::json!({}));
        assert!(result.get("error").is_some(), "should return error");
    }

    #[test]
    fn test_memory_enrichment_timeline_event_type_filter() {
        let (ctx, memory_id) = ctx_with_event();
        let result = memory_enrichment_timeline(
            &ctx,
            serde_json::json!({"memory_id": memory_id, "event_type": "no_such_type"}),
        );
        assert_eq!(result["count"], 0);
    }

    #[test]
    fn test_memory_enrichment_audit_no_filters() {
        let (ctx, _) = ctx_with_event();
        let result = memory_enrichment_audit(&ctx, serde_json::json!({}));
        let count = result["count"].as_u64().expect("count");
        assert!(count >= 1, "should return at least one event");
    }

    #[test]
    fn test_memory_enrichment_audit_status_filter() {
        let (ctx, _) = ctx_with_event();
        let result =
            memory_enrichment_audit(&ctx, serde_json::json!({"status": "completed"}));
        let count = result["count"].as_u64().expect("count");
        assert!(count >= 1);
        let events = result["events"].as_array().expect("events");
        for ev in events {
            assert_eq!(ev["status"], "completed");
        }
    }

    #[test]
    fn test_memory_enrichment_audit_invalid_status() {
        let (ctx, _) = ctx_with_event();
        let result =
            memory_enrichment_audit(&ctx, serde_json::json!({"status": "invalid"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_memory_enrichment_audit_event_type_filter() {
        let (ctx, _) = ctx_with_event();
        let result =
            memory_enrichment_audit(&ctx, serde_json::json!({"event_type": "consolidation"}));
        let events = result["events"].as_array().expect("events");
        assert!(!events.is_empty());
        for ev in events {
            assert_eq!(ev["event_type"], "consolidation");
        }
    }

    #[test]
    fn test_memory_enrichment_audit_limit_respected() {
        let (ctx, _) = ctx_with_event();
        let result = memory_enrichment_audit(&ctx, serde_json::json!({"limit": 1}));
        let count = result["count"].as_u64().expect("count");
        assert!(count <= 1);
    }

    #[test]
    fn test_memory_enrichment_audit_filters_applied_present() {
        let (ctx, _) = ctx_with_event();
        let result = memory_enrichment_audit(
            &ctx,
            serde_json::json!({"status": "completed", "limit": 10, "order": "asc"}),
        );
        let fa = result.get("filters_applied").expect("filters_applied key");
        assert_eq!(fa["status"], "completed");
        assert_eq!(fa["limit"], 10);
        assert_eq!(fa["order"], "asc");
    }

    #[test]
    fn test_memory_enrichment_audit_filters_applied_no_filters() {
        let (ctx, _) = ctx_with_event();
        let result = memory_enrichment_audit(&ctx, serde_json::json!({}));
        let fa = result.get("filters_applied").expect("filters_applied key");
        // limit and order are always present
        assert!(fa.get("limit").is_some());
        assert!(fa.get("order").is_some());
        // no optional filters set
        assert!(fa.get("status").is_none());
        assert!(fa.get("event_type").is_none());
    }

    #[test]
    fn test_memory_enrichment_timeline_version_snapshot_null_when_no_version() {
        let (ctx, memory_id) = ctx_with_event();
        // The test event has version_id = None, so version_snapshot must be null
        // regardless of include_snapshots value.
        let result = memory_enrichment_timeline(
            &ctx,
            serde_json::json!({"memory_id": memory_id, "include_snapshots": true}),
        );
        let events = result["events"].as_array().expect("events array");
        assert!(!events.is_empty());
        assert!(events[0]["version_snapshot"].is_null());
    }

    #[test]
    fn test_memory_enrichment_timeline_include_snapshots_false_forces_null() {
        let (ctx, memory_id) = ctx_with_event();
        // include_snapshots=false must produce null version_snapshot (no WHERE filter).
        let result = memory_enrichment_timeline(
            &ctx,
            serde_json::json!({"memory_id": memory_id, "include_snapshots": false}),
        );
        let events = result["events"].as_array().expect("events array");
        // row should still be returned (include_snapshots does not filter rows)
        assert_eq!(result["count"], 1);
        assert!(events[0]["version_snapshot"].is_null());
    }
}
