//! Enrichment audit tool handlers (ENG-1240).
//!
//! Provides MCP tools:
//! - `memory_enrichment_timeline` – per-memory enrichment history
//! - `memory_enrichment_audit`    – global enrichment event query with filters
//! - `memory_replay_at_time`      – point-in-time memory state + temporal graph edges

use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

use crate::graph::temporal::edges_for_memory_at;

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
            // pos starts at 1 for the mandatory memory_id condition; each
            // optional filter advances it so the limit placeholder is always
            // correct regardless of which filters are active.
            let mut pos: usize = 1;
            let mut conditions: Vec<String> = {
                let c = format!("e.memory_id = ?{pos}");
                pos += 1;
                vec![c]
            };

            if event_type.is_some() {
                conditions.push(format!("e.event_type = ?{pos}"));
                pos += 1;
            }
            if !include_dry_runs {
                conditions.push("e.dry_run = 0".to_string());
            }
            // NOTE: include_snapshots=false suppresses version_snapshot data in
            // the row output, but does NOT filter rows by event_type.

            let limit_pos = pos;
            let where_clause = conditions.join(" AND ");
            let sql = format!(
                "SELECT e.id, e.operation_id, e.event_type, e.memory_id, e.version_id,
                        e.triggered_by, e.agent_id, e.workspace, e.params, e.outcome,
                        e.status, e.dry_run, e.created_at,
                        mv.content, mv.version
                 FROM enrichment_events e
                 LEFT JOIN memory_versions mv ON mv.id = e.version_id
                 WHERE {where_clause}
                 ORDER BY e.created_at DESC, e.id DESC
                 LIMIT ?{limit_pos}"
            );

            let mut stmt = conn.prepare(&sql)?;

            let rows: Vec<Value> = if let Some(ref et) = event_type {
                stmt.query_map(params![memory_id, et.as_str(), limit], |row| {
                    row_to_json_with_snapshot(row, include_snapshots)
                })?
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

    // Validate RFC3339 timestamps
    if let Some(ref s) = since {
        if chrono::DateTime::parse_from_rfc3339(s).is_err() {
            return json!({"error": format!("'since' must be a valid RFC3339 timestamp, got: {s}")});
        }
    }
    if let Some(ref u) = until {
        if chrono::DateTime::parse_from_rfc3339(u).is_err() {
            return json!({"error": format!("'until' must be a valid RFC3339 timestamp, got: {u}")});
        }
    }

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

            if event_type.is_some() {
                cond!("event_type");
            }
            if triggered_by.is_some() {
                cond!("triggered_by");
            }
            if agent_id.is_some() {
                cond!("agent_id");
            }
            if status.is_some() {
                cond!("status");
            }
            if workspace.is_some() {
                cond!("workspace");
            }
            if operation_id.is_some() {
                cond!("operation_id");
            }
            if memory_id.is_some() {
                cond!("memory_id");
            }
            if version_id.is_some() {
                cond!("version_id");
            }
            if dry_run.is_some() {
                cond!("dry_run");
            }
            if since.is_some() {
                cond!("created_at", ">=");
            }
            if until.is_some() {
                cond!("created_at", "<=");
            }

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
                 ORDER BY created_at {order_dir}, id {order_dir}
                 LIMIT ?{limit_pos}"
            );

            let mut stmt = conn.prepare(&sql)?;

            // Bind values in the same order as conditions.
            use rusqlite::types::ToSql;
            let mut bind_vals: Vec<Box<dyn ToSql>> = Vec::new();
            if let Some(ref v) = event_type {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(ref v) = triggered_by {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(ref v) = agent_id {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(ref v) = status {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(ref v) = workspace {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(ref v) = operation_id {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(v) = memory_id {
                bind_vals.push(Box::new(v));
            }
            if let Some(v) = version_id {
                bind_vals.push(Box::new(v));
            }
            if let Some(v) = dry_run {
                bind_vals.push(Box::new(v as i32));
            }
            if let Some(ref v) = since {
                bind_vals.push(Box::new(v.clone()));
            }
            if let Some(ref v) = until {
                bind_vals.push(Box::new(v.clone()));
            }
            bind_vals.push(Box::new(limit));

            let refs: Vec<&dyn ToSql> = bind_vals.iter().map(|b| b.as_ref()).collect();

            let rows: Vec<Value> = stmt
                .query_map(refs.as_slice(), row_to_json)?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            let count = rows.len();

            // Build filters_applied map from all active filters.
            let mut applied = serde_json::Map::new();
            if let Some(ref v) = event_type {
                applied.insert("event_type".into(), json!(v));
            }
            if let Some(ref v) = triggered_by {
                applied.insert("triggered_by".into(), json!(v));
            }
            if let Some(ref v) = agent_id {
                applied.insert("agent_id".into(), json!(v));
            }
            if let Some(ref v) = status {
                applied.insert("status".into(), json!(v));
            }
            if let Some(ref v) = workspace {
                applied.insert("workspace".into(), json!(v));
            }
            if let Some(ref v) = operation_id {
                applied.insert("operation_id".into(), json!(v));
            }
            if let Some(v) = memory_id {
                applied.insert("memory_id".into(), json!(v));
            }
            if let Some(v) = version_id {
                applied.insert("version_id".into(), json!(v));
            }
            if let Some(v) = dry_run {
                applied.insert("dry_run".into(), json!(v));
            }
            if let Some(ref v) = since {
                applied.insert("since".into(), json!(v));
            }
            if let Some(ref v) = until {
                applied.insert("until".into(), json!(v));
            }
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

/// Replay a memory's content as it existed at `timestamp` with optional event trail.
///
/// - `memory_id` (required): memory identifier
/// - `timestamp` (required): RFC3339 point-in-time
/// - `event_limit` (optional): max number of enrichment events to include (default 50, max 200)
/// - `include_failed` (optional): include failed events (default false)
/// - `event_type` (optional): restrict replay log to a specific event type
/// - `include_dry_runs` (optional): include dry-run events (default false)
/// - `include_events` (optional): include replay event list (default true)
pub fn memory_replay_at_time(ctx: &HandlerContext, params: Value) -> Value {
    let memory_id = match params.get("memory_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "memory_id is required"}),
    };

    let timestamp = match params.get("timestamp").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return json!({"error": "timestamp is required"}),
    };

    let as_of = match chrono::DateTime::parse_from_rfc3339(&timestamp) {
        Ok(ts) => ts.with_timezone(&chrono::Utc),
        Err(_) => {
            return json!({"error": format!("timestamp must be RFC3339, got: {timestamp}")});
        }
    };

    let event_type = params
        .get("event_type")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let include_dry_runs = params
        .get("include_dry_runs")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let include_events = params
        .get("include_events")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let include_failed = params
        .get("include_failed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let event_limit = params
        .get("event_limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50)
        .min(200) as i64;

    ctx.storage
        .with_connection(|conn| {
            let mut response = serde_json::Map::new();

            response.insert("memory_id".into(), json!(memory_id));
            response.insert("timestamp".into(), json!(timestamp));

            // Pick the latest memory version valid at the requested point in time.
            {
                let mut stmt = conn.prepare_cached(
                    r#"
                    SELECT version, content, tags, metadata, created_at, created_by, change_summary
                    FROM memory_versions
                    WHERE memory_id = ?1 AND julianday(created_at) <= julianday(?2)
                    ORDER BY version DESC
                    LIMIT 1
                    "#,
                )?;

                let state: Option<Value> = stmt
                    .query_row(params![memory_id, as_of.to_rfc3339()], |row| {
                        let version: i64 = row.get(0)?;
                        let content: String = row.get(1)?;
                        let tags_str: String = row.get(2)?;
                        let metadata_str: String = row.get(3)?;
                        let created_at: String = row.get(4)?;
                        let created_by: Option<String> = row.get(5)?;
                        let change_summary: Option<String> = row.get(6)?;

                        let tags = serde_json::from_str(&tags_str).unwrap_or(json!([]));
                        let metadata = serde_json::from_str(&metadata_str).unwrap_or(json!({}));

                        Ok(json!({
                            "version": version,
                            "content": content,
                            "tags": tags,
                            "metadata": metadata,
                            "created_at": created_at,
                            "created_by": created_by,
                            "change_summary": change_summary,
                        }))
                    })
                    .optional()?;

                match state {
                    Some(state) => {
                        response.insert("found".into(), json!(true));
                        response.insert("state".into(), state);
                    }
                    None => {
                        response.insert("found".into(), json!(false));
                        response.insert("state".into(), Value::Null);
                    }
                }
            }

            let mut event_rows = Vec::new();
            if include_events {
                let mut conditions: Vec<String> = vec!["e.memory_id = ?1".to_string()];
                let mut bind_vals: Vec<rusqlite::types::Value> = vec![
                    rusqlite::types::Value::Integer(memory_id),
                    rusqlite::types::Value::Text(as_of.to_rfc3339()),
                ];

                // Include successful events by default; optional failure events can be requested.
                if !include_failed {
                    conditions.push("e.status IN (?3, ?4)".to_string());
                    bind_vals.push(rusqlite::types::Value::Text("completed".to_string()));
                    bind_vals.push(rusqlite::types::Value::Text("skipped".to_string()));
                }
                if !include_dry_runs {
                    let p = bind_vals.len() + 1;
                    conditions.push(format!("e.dry_run = ?{p}"));
                    bind_vals.push(rusqlite::types::Value::Integer(0));
                }
                if let Some(ref et) = event_type {
                    let p = bind_vals.len() + 1;
                    conditions.push(format!("e.event_type = ?{p}"));
                    bind_vals.push(rusqlite::types::Value::Text(et.clone()));
                }

                let limit_pos = bind_vals.len() + 1;
                let where_clause = format!("WHERE {}", conditions.join(" AND "));
                let sql = format!(
                    "SELECT id, operation_id, event_type, memory_id, version_id, \
                        triggered_by, agent_id, workspace, params, outcome, \
                        status, dry_run, created_at \
                     FROM enrichment_events e \
                     {where_clause} \
                     AND julianday(e.created_at) <= julianday(?2) \
                     ORDER BY julianday(e.created_at) DESC, e.id DESC \
                     LIMIT ?{limit_pos}"
                );

                bind_vals.push(rusqlite::types::Value::Integer(event_limit));

                let mut bound_params: Vec<&dyn rusqlite::types::ToSql> =
                    Vec::with_capacity(bind_vals.len());
                for v in &bind_vals {
                    bound_params.push(v);
                }

                let mut stmt = conn.prepare_cached(&sql)?;
                let rows = stmt
                    .query_map(bound_params.as_slice(), row_to_json)?
                    .collect::<rusqlite::Result<Vec<_>>>()?;

                event_rows = rows;
            }

            response.insert("events".into(), json!(event_rows));
            response.insert("events_count".into(), json!(event_rows.len()));
            response.insert("requested_timestamp".into(), json!(as_of.to_rfc3339()));

            // Temporal graph edges where this memory is an endpoint, active at timestamp.
            // Uses edges_for_memory_at (SQL-filtered) instead of snapshot_at to avoid
            // loading the entire temporal graph when only one memory's edges are needed.
            let temporal_edges = edges_for_memory_at(conn, memory_id, &as_of.to_rfc3339())
                .inspect_err(|e| {
                    tracing::warn!(
                        memory_id,
                        error = %e,
                        "temporal_edges query failed in memory_replay_at_time"
                    );
                })
                .unwrap_or_default();
            response.insert("temporal_edges".into(), json!(temporal_edges));
            response.insert("temporal_edges_count".into(), json!(temporal_edges.len()));

            Ok(json!(response))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── shared row mappers ────────────────────────────────────────────────────────

/// Row mapper for queries that SELECT 13 base columns from `enrichment_events`.
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
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(
                    embedder.dimensions(),
                    crate::search::VectorMetric::Cosine,
                ),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
        };
        (ctx, memory_id)
    }

    #[test]
    fn test_memory_enrichment_timeline_returns_events() {
        let (ctx, memory_id) = ctx_with_event();
        let result = memory_enrichment_timeline(&ctx, serde_json::json!({"memory_id": memory_id}));
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
        let result = memory_enrichment_audit(&ctx, serde_json::json!({"status": "completed"}));
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
        let result = memory_enrichment_audit(&ctx, serde_json::json!({"status": "invalid"}));
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

    #[test]
    fn test_memory_replay_at_time_returns_latest_state_before_timestamp() {
        let (ctx, memory_id) = ctx_with_event();

        ctx.storage
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
                     VALUES (?1, 1, 'v1', '[]', '{}', '2026-01-01T00:00:00Z')",
                    params![memory_id],
                )?;
                conn.execute(
                    "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
                     VALUES (?1, 2, 'v2', '[\"a\"]', '{\"k\":1}', '2026-01-02T00:00:00Z')",
                    params![memory_id],
                )?;
                conn.execute(
                    "INSERT INTO enrichment_events
                         (operation_id, event_type, memory_id, triggered_by, params, outcome, status, dry_run, created_at)
                     VALUES ('op-1', 'consolidation', ?1, 'memory_consolidate', '{}', '{\"ok\":true}', 'completed', 0, '2026-01-02T00:00:00Z')",
                    params![memory_id],
                )?;
                Ok(())
            })
            .expect("seed replay test data");

        let result = memory_replay_at_time(
            &ctx,
            serde_json::json!({
                "memory_id": memory_id,
                "timestamp": "2026-01-02T00:00:00Z",
                "include_events": true,
            }),
        );
        assert_eq!(result["memory_id"], memory_id);
        assert_eq!(result["found"], true);
        assert_eq!(result["state"]["version"], 2);
        assert_eq!(result["state"]["content"], "v2");
        let events = result["events"].as_array().expect("events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event_type"], "consolidation");
        assert_eq!(result["events_count"], 1);
    }

    #[test]
    fn test_memory_replay_at_time_preserves_subsecond_boundary() {
        let (ctx, memory_id) = ctx_with_event();

        ctx.storage
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
                     VALUES (?1, 1, 'early', '[]', '{}', '2026-01-02T00:00:00.050Z')",
                    params![memory_id],
                )?;
                conn.execute(
                    "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
                     VALUES (?1, 2, 'future', '[]', '{}', '2026-01-02T00:00:00.900Z')",
                    params![memory_id],
                )?;
                conn.execute(
                    "INSERT INTO enrichment_events
                         (operation_id, event_type, memory_id, triggered_by, params, outcome, status, dry_run, created_at)
                     VALUES ('op-early', 'consolidation', ?1, 'memory_consolidate', '{}', '{\"ok\":true}', 'completed', 0, '2026-01-02T00:00:00.050Z')",
                    params![memory_id],
                )?;
                conn.execute(
                    "INSERT INTO enrichment_events
                         (operation_id, event_type, memory_id, triggered_by, params, outcome, status, dry_run, created_at)
                     VALUES ('op-future', 'consolidation', ?1, 'memory_consolidate', '{}', '{\"ok\":true}', 'completed', 0, '2026-01-02T00:00:00.900Z')",
                    params![memory_id],
                )?;
                Ok(())
            })
            .expect("seed replay subsecond boundary data");

        let result = memory_replay_at_time(
            &ctx,
            serde_json::json!({
                "memory_id": memory_id,
                "timestamp": "2026-01-02T00:00:00.100Z",
                "include_events": true,
            }),
        );

        assert_eq!(result["found"], true);
        assert_eq!(result["state"]["version"], 1);
        assert_eq!(result["state"]["content"], "early");

        let events = result["events"].as_array().expect("events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["operation_id"], "op-early");
        assert_eq!(result["events_count"], 1);
    }
}
