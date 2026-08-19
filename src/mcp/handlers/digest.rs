//! Actionable retrieval digest built from existing read-only primitives.

use std::collections::HashSet;

use chrono::Utc;
use serde_json::{json, Value};

use super::{context, smart_retrieve, HandlerContext};

const DEFAULT_LIMIT: usize = 12;
const MAX_LIMIT: usize = 50;
const DEFAULT_TOTAL_BUDGET: u64 = 4096;
const MIN_TOTAL_BUDGET: u64 = 512;
const MAX_TOTAL_BUDGET: u64 = 12_000;

#[derive(Debug, Clone)]
struct DigestRequest {
    topic: String,
    workspace: Option<String>,
    mode: String,
    limit: usize,
    related_depth: usize,
    total_budget: u64,
    include_types: Option<Vec<String>>,
    timeframe: String,
    include_graph: bool,
    include_operational_context: bool,
    include_next_actions: bool,
    current_git_branch: Option<String>,
    current_commit_hash: Option<String>,
}

#[derive(Debug, Clone)]
struct TopMemory {
    id: i64,
    memory_type: String,
    preview: String,
    score: Option<f64>,
    created_at: Option<String>,
}

/// Build an extractive, source-linked digest for a topic.
///
/// This handler is intentionally read-only. It orchestrates existing retrieval
/// and context tools, then normalizes their output into a compact response.
pub fn memory_digest(ctx: &HandlerContext, params: Value) -> Value {
    let request = match parse_request(&params) {
        Ok(request) => request,
        Err(message) => return json!({"error": message}),
    };

    let mut warnings = Vec::new();
    let mut strategies = vec!["memory_smart_retrieve".to_string()];

    let mut retrieve_params = json!({
        "query": request.topic.clone(),
        "limit": request.limit,
    });
    insert_optional_string(
        &mut retrieve_params,
        "workspace",
        request.workspace.as_deref(),
    );

    let retrieve_response = smart_retrieve::memory_smart_retrieve(ctx, retrieve_params);
    if let Some(error) = retrieve_response.get("error").and_then(Value::as_str) {
        warnings.push(format!("memory_smart_retrieve failed: {error}"));
    }

    let top_memories = collect_top_memories(&retrieve_response, request.include_types.as_deref());
    let source_memory_ids: Vec<i64> = top_memories.iter().map(|memory| memory.id).collect();

    let context_summary = build_memory_context_summary(ctx, &request, &mut warnings);
    if !context_summary.is_null() {
        strategies.push("memory_build_context".to_string());
    }

    let relationships =
        if request.include_graph && request.related_depth > 0 && !source_memory_ids.is_empty() {
            collect_relationships(ctx, &source_memory_ids, &mut warnings)
        } else {
            Vec::new()
        };

    let operational_context = if request.include_operational_context {
        strategies.push("context_build_bundle".to_string());
        build_operational_context(ctx, &request, &mut warnings)
    } else {
        json!({
            "included": false,
            "reason": "include_operational_context=false"
        })
    };

    if top_memories.is_empty() {
        warnings.push("No source memories matched the requested topic.".to_string());
    }

    json!({
        "topic": request.topic.clone(),
        "workspace": request.workspace.as_deref().unwrap_or("default"),
        "mode": request.mode,
        "generated_at": Utc::now().to_rfc3339(),
        "digest": build_digest_section(&request, &top_memories),
        "top_memories": top_memories_json(&top_memories),
        "relationships": relationships,
        "operational_context": operational_context,
        "next_actions": build_next_actions(&request, &top_memories),
        "provenance": {
            "source_memory_ids": source_memory_ids,
            "source_context_event_ids": source_context_event_ids(&operational_context),
            "tools_or_strategies": strategies,
            "context_summary": context_summary,
            "policy": {
                "read_only": true,
                "llm_used": false,
                "raw_artifact_content_returned": false
            }
        },
        "warnings": warnings
    })
}

fn parse_request(params: &Value) -> Result<DigestRequest, String> {
    let topic = params
        .get("topic")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|topic| !topic.is_empty())
        .ok_or_else(|| "topic is required".to_string())?
        .to_string();

    let mode = params
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("standard")
        .trim()
        .to_string();
    if !matches!(mode.as_str(), "brief" | "standard" | "deep") {
        return Err("mode must be one of: brief, standard, deep".to_string());
    }

    let timeframe = params
        .get("timeframe")
        .and_then(Value::as_str)
        .unwrap_or("all")
        .trim()
        .to_string();
    if !matches!(timeframe.as_str(), "1h" | "24h" | "7d" | "30d" | "all") {
        return Err("timeframe must be one of: 1h, 24h, 7d, 30d, all".to_string());
    }

    let include_types = match params.get("include_types").and_then(Value::as_array) {
        Some(values) => {
            let types: Vec<String> = values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect();
            if types.is_empty() {
                None
            } else {
                Some(types)
            }
        }
        None => None,
    };

    let limit = if let Some(v) = params.get("limit") {
        let val = v
            .as_u64()
            .ok_or_else(|| "limit must be a positive integer".to_string())?
            as usize;
        if !(1..=MAX_LIMIT).contains(&val) {
            return Err(format!("limit must be between 1 and {MAX_LIMIT}"));
        }
        val
    } else {
        DEFAULT_LIMIT
    };

    let related_depth = if let Some(v) = params.get("related_depth") {
        let val = v
            .as_u64()
            .ok_or_else(|| "related_depth must be an integer".to_string())?
            as usize;
        if val > 2 {
            return Err("related_depth must be between 0 and 2".to_string());
        }
        val
    } else {
        1
    };

    let total_budget = if let Some(v) = params.get("total_budget") {
        let val = v
            .as_u64()
            .ok_or_else(|| "total_budget must be a positive integer".to_string())?;
        if !(MIN_TOTAL_BUDGET..=MAX_TOTAL_BUDGET).contains(&val) {
            return Err(format!(
                "total_budget must be between {MIN_TOTAL_BUDGET} and {MAX_TOTAL_BUDGET}"
            ));
        }
        val
    } else {
        DEFAULT_TOTAL_BUDGET
    };

    Ok(DigestRequest {
        topic,
        workspace: optional_string(params, "workspace"),
        mode,
        limit,
        related_depth,
        total_budget,
        include_types,
        timeframe,
        include_graph: optional_bool(params, "include_graph", true),
        include_operational_context: optional_bool(params, "include_operational_context", true),
        include_next_actions: optional_bool(params, "include_next_actions", true),
        current_git_branch: optional_string(params, "current_git_branch"),
        current_commit_hash: optional_string(params, "current_commit_hash"),
    })
}

fn collect_top_memories(response: &Value, include_types: Option<&[String]>) -> Vec<TopMemory> {
    let mut seen = HashSet::new();
    let mut top_memories = Vec::new();

    for item in response
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(memory) = normalize_memory(item) {
            if !include_type_matches(&memory, include_types) {
                continue;
            }
            if seen.insert(memory.id) {
                top_memories.push(memory);
            }
        }
    }

    top_memories
}

fn normalize_memory(item: &Value) -> Option<TopMemory> {
    let memory = item.get("memory").unwrap_or(item);
    let id = memory_id(memory)?;
    let memory_type = memory
        .get("memory_type")
        .or_else(|| memory.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("note")
        .to_string();
    let content = memory.get("content").and_then(Value::as_str).unwrap_or("");
    let created_at = memory
        .get("created_at")
        .and_then(Value::as_str)
        .map(str::to_string);
    let score = item
        .get("score")
        .or_else(|| memory.get("score"))
        .and_then(Value::as_f64);

    Some(TopMemory {
        id,
        memory_type,
        preview: compact_preview(content, 260),
        score,
        created_at,
    })
}

fn memory_id(value: &Value) -> Option<i64> {
    value
        .get("id")
        .and_then(Value::as_i64)
        .or_else(|| value.get("memory_id").and_then(Value::as_i64))
}

fn include_type_matches(memory: &TopMemory, include_types: Option<&[String]>) -> bool {
    include_types
        .map(|types| {
            types
                .iter()
                .any(|candidate| candidate == &memory.memory_type)
        })
        .unwrap_or(true)
}

fn build_memory_context_summary(
    ctx: &HandlerContext,
    request: &DigestRequest,
    warnings: &mut Vec<String>,
) -> Value {
    let mut params = json!({
        "query": request.topic.clone(),
        "limit": request.limit,
        "depth": request.related_depth.max(1),
        "timeframe": request.timeframe,
        "total_budget": request.total_budget,
        "include_graph": request.include_graph,
    });
    insert_optional_string(&mut params, "workspace", request.workspace.as_deref());
    if let Some(include_types) = &request.include_types {
        params["include_types"] = json!(include_types);
    }

    let response = context::memory_build_context(ctx, params);
    if let Some(error) = response.get("error").and_then(Value::as_str) {
        warnings.push(format!("memory_build_context failed: {error}"));
        return Value::Null;
    }

    json!({
        "memories_used": response.get("memories_used").cloned().unwrap_or(Value::Null),
        "token_estimate": response.get("token_estimate").cloned().unwrap_or(Value::Null),
        "total_budget": response.get("total_budget").cloned().unwrap_or(Value::Null),
        "depth": response.get("depth").cloned().unwrap_or(Value::Null),
        "timeframe": response.get("timeframe").cloned().unwrap_or(Value::Null),
        "graph_node_count": response.pointer("/graph/node_count").cloned().unwrap_or(Value::Null)
    })
}

fn collect_relationships(
    ctx: &HandlerContext,
    source_memory_ids: &[i64],
    warnings: &mut Vec<String>,
) -> Vec<Value> {
    let selected: HashSet<i64> = source_memory_ids.iter().copied().collect();
    let result = ctx.storage.with_connection(|conn| {
        let mut rows = Vec::new();
        for id in source_memory_ids {
            let mut stmt = conn.prepare(
                "SELECT from_id, to_id, edge_type, score, confidence, strength
                 FROM crossrefs
                 WHERE (from_id = ?1 OR to_id = ?1) AND valid_to IS NULL
                 ORDER BY score DESC
                 LIMIT 50",
            )?;
            let related = stmt.query_map(rusqlite::params![id], |row| {
                Ok(json!({
                    "from_id": row.get::<_, i64>(0)?,
                    "to_id": row.get::<_, i64>(1)?,
                    "edge_type": row.get::<_, String>(2)?,
                    "score": row.get::<_, f64>(3)?,
                    "confidence": row.get::<_, f64>(4)?,
                    "strength": row.get::<_, f64>(5)?,
                }))
            })?;
            for row in related {
                rows.push(row?);
            }
        }
        Ok::<_, crate::error::EngramError>(rows)
    });

    let rows = match result {
        Ok(rows) => rows,
        Err(error) => {
            warnings.push(format!("relationship lookup failed: {error}"));
            return Vec::new();
        }
    };

    let mut seen = HashSet::new();
    rows.into_iter()
        .filter(|row| {
            let from_id = row
                .get("from_id")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let to_id = row.get("to_id").and_then(Value::as_i64).unwrap_or_default();
            selected.contains(&from_id) || selected.contains(&to_id)
        })
        .filter(|row| {
            let from_id = row
                .get("from_id")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let to_id = row.get("to_id").and_then(Value::as_i64).unwrap_or_default();
            let edge_type = row
                .get("edge_type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            seen.insert((from_id, to_id, edge_type))
        })
        .collect()
}

fn build_operational_context(
    ctx: &HandlerContext,
    request: &DigestRequest,
    warnings: &mut Vec<String>,
) -> Value {
    let mut params = json!({
        "query": request.topic.clone(),
        "max_results": request.limit.max(20),
        "section_limit": match request.mode.as_str() {
            "brief" => 3,
            "deep" => 10,
            _ => 6,
        },
        "include_artifact_pointers": false,
    });
    insert_optional_string(&mut params, "workspace", request.workspace.as_deref());
    insert_optional_string(
        &mut params,
        "current_git_branch",
        request.current_git_branch.as_deref(),
    );
    insert_optional_string(
        &mut params,
        "current_commit_hash",
        request.current_commit_hash.as_deref(),
    );

    let response = context::context_build_bundle(ctx, params);
    if let Some(error) = response.get("error").and_then(Value::as_str) {
        warnings.push(format!("context_build_bundle failed: {error}"));
        return json!({
            "included": false,
            "error": error
        });
    }

    json!({
        "included": true,
        "recent_decisions": response.get("recent_decisions").cloned().unwrap_or_else(|| json!([])),
        "unresolved_blockers": response.get("unresolved_blockers").cloned().unwrap_or_else(|| json!([])),
        "failures": response.get("failures").cloned().unwrap_or_else(|| json!([])),
        "commands_already_run": response.get("commands_already_run").cloned().unwrap_or_else(|| json!([])),
        "files_inspected_or_touched": response.get("files_inspected_or_touched").cloned().unwrap_or_else(|| json!([])),
        "stale_warnings": response.get("stale_warnings").cloned().unwrap_or_else(|| json!([])),
        "metrics": response.get("metrics").cloned().unwrap_or(Value::Null),
        "artifact_policy": response.get("artifact_policy").cloned().unwrap_or(Value::Null),
    })
}

fn build_digest_section(request: &DigestRequest, top_memories: &[TopMemory]) -> Value {
    let key_point_limit = match request.mode.as_str() {
        "brief" => 3,
        "deep" => 8,
        _ => 5,
    };
    let ids: Vec<i64> = top_memories.iter().map(|memory| memory.id).collect();
    let summary = if ids.is_empty() {
        format!(
            "No source memories were found for '{}'. Treat this digest as empty.",
            request.topic
        )
    } else {
        format!(
            "Found {} source memories for '{}'. Top source IDs: {}.",
            ids.len(),
            request.topic,
            ids.iter()
                .map(i64::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    };

    let key_points: Vec<Value> = top_memories
        .iter()
        .take(key_point_limit)
        .map(|memory| {
            json!({
                "text": memory.preview,
                "source_memory_ids": [memory.id],
                "source_context_event_ids": []
            })
        })
        .collect();

    json!({
        "summary": summary,
        "key_points": key_points,
        "open_questions": []
    })
}

fn top_memories_json(top_memories: &[TopMemory]) -> Vec<Value> {
    top_memories
        .iter()
        .map(|memory| {
            json!({
                "id": memory.id,
                "memory_type": memory.memory_type,
                "preview": memory.preview,
                "score": memory.score,
                "created_at": memory.created_at,
                "why": ["memory_smart_retrieve"]
            })
        })
        .collect()
}

fn build_next_actions(request: &DigestRequest, top_memories: &[TopMemory]) -> Vec<Value> {
    if !request.include_next_actions {
        return Vec::new();
    }

    match top_memories.first() {
        Some(memory) => vec![json!({
            "text": format!("Inspect memory {} before making a durable decision about '{}'.", memory.id, request.topic),
            "source_memory_ids": [memory.id],
            "source_context_event_ids": []
        })],
        None => vec![json!({
            "text": format!("Create or ingest source memories before relying on a digest for '{}'.", request.topic),
            "source_memory_ids": [],
            "source_context_event_ids": []
        })],
    }
}

fn source_context_event_ids(operational_context: &Value) -> Vec<i64> {
    let mut ids = HashSet::new();
    for section in [
        "recent_decisions",
        "unresolved_blockers",
        "failures",
        "relevant_context",
    ] {
        for item in operational_context
            .get(section)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            collect_context_ids(item, &mut ids);
        }
    }
    let mut ids: Vec<i64> = ids.into_iter().collect();
    ids.sort_unstable();
    ids
}

fn collect_context_ids(value: &Value, ids: &mut HashSet<i64>) {
    if let Some(id) = value
        .pointer("/provenance/event_id")
        .and_then(Value::as_i64)
    {
        ids.insert(id);
    }
    if let Some(id) = value.get("event_id").and_then(Value::as_i64) {
        ids.insert(id);
    }
}

fn compact_preview(content: &str, max_bytes: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<&str>>().join(" ");
    if normalized.len() <= max_bytes {
        return normalized;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !normalized.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}...", &normalized[..boundary])
}

fn optional_string(params: &Value, name: &str) -> Option<String> {
    params
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn optional_bool(params: &Value, name: &str, default: bool) -> bool {
    params.get(name).and_then(Value::as_bool).unwrap_or(default)
}

fn insert_optional_string(target: &mut Value, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        target[key] = json!(value);
    }
}
