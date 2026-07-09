//! Fact ingest tools (single + batch).
use serde_json::{json, Value};

use super::super::HandlerContext;
use crate::realtime::RealtimeEvent;
use crate::types::*;

/// Append-only fact ingest — always inserts a NEW memory with `memory_type = "fact"`.
///
/// Lighter than `memory_create`: no dedup, no upsert, minimal overhead.
pub fn memory_ingest_fact(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_memory;
    use std::collections::HashMap;

    let fact = match params.get("fact").and_then(|v| v.as_str()) {
        Some(f) => f.to_string(),
        None => return json!({"error": "fact is required"}),
    };

    let source = params
        .get("source")
        .and_then(|v| v.as_str())
        .map(String::from);
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let workspace = Some(
        params
            .get("workspace")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string(),
    );
    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let importance = params
        .get("importance")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .or(Some(0.8));

    let scope = match params
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("global")
    {
        "global" => MemoryScope::Global,
        other => return json!({"error": format!("unsupported scope '{}'; use 'global'", other)}),
    };

    let mut metadata: HashMap<String, Value> = HashMap::new();
    if let Some(ref src) = source {
        metadata.insert("source".to_string(), json!(src));
    }
    if let Some(ref sid) = session_id {
        metadata.insert("session_id".to_string(), json!(sid));
    }

    let ws_str = workspace.clone();
    let input = CreateMemoryInput {
        content: fact,
        memory_type: MemoryType::Fact,
        tags,
        metadata,
        importance,
        scope,
        workspace,
        tier: MemoryTier::Permanent,
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: DedupMode::Allow,
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    // Run the DB insert inside a transaction; fuzzy index update happens after commit.
    let result = ctx
        .storage
        .with_transaction(|conn| create_memory(conn, &input));

    match result {
        Ok(memory) => {
            ctx.search_cache.invalidate_for_workspace(ws_str.as_deref());
            {
                let mut fuzzy = ctx.fuzzy_engine.lock();
                fuzzy.add_to_vocabulary(&memory.content);
            }
            if let Some(ref manager) = ctx.realtime {
                manager.broadcast(RealtimeEvent::memory_created(
                    memory.id,
                    memory.content.clone(),
                ));
            }
            json!({"id": memory.id, "created": true})
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

/// Batch variant of `memory_ingest_fact` — inserts all facts atomically in a single transaction.
///
/// If any item is invalid (missing `fact`) or any insert fails, the entire batch is rolled back.
pub fn memory_ingest_fact_batch(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::create_memory;
    use std::collections::HashMap;
    use std::collections::HashSet;

    let facts = match params.get("facts").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return json!({"error": "facts array is required"}),
    };

    if facts.is_empty() {
        return json!({"error": "facts must have at least 1 item"});
    }

    let default_workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let default_scope = match params
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("global")
    {
        "global" => MemoryScope::Global,
        other => return json!({"error": format!("unsupported scope '{}'; use 'global'", other)}),
    };

    // Validate all items up-front before touching the DB.
    let mut inputs: Vec<(CreateMemoryInput, String)> = Vec::with_capacity(facts.len());
    for (idx, item) in facts.iter().enumerate() {
        let fact = match item.get("fact").and_then(|v| v.as_str()) {
            Some(f) => f.to_string(),
            None => {
                return json!({"error": format!(
                    "item at index {} is missing required field 'fact'",
                    idx
                )})
            }
        };

        let source = item
            .get("source")
            .and_then(|v| v.as_str())
            .map(String::from);
        let session_id = item
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let workspace = item
            .get("workspace")
            .and_then(|v| v.as_str())
            .unwrap_or(&default_workspace)
            .to_string();
        let tags: Vec<String> = item
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let importance = item
            .get("importance")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .or(Some(0.8));

        let mut metadata: HashMap<String, Value> = HashMap::new();
        if let Some(ref src) = source {
            metadata.insert("source".to_string(), json!(src));
        }
        if let Some(ref sid) = session_id {
            metadata.insert("session_id".to_string(), json!(sid));
        }

        let input = CreateMemoryInput {
            content: fact,
            memory_type: MemoryType::Fact,
            tags,
            metadata,
            importance,
            scope: default_scope.clone(),
            workspace: Some(workspace.clone()),
            tier: MemoryTier::Permanent,
            defer_embedding: false,
            ttl_seconds: None,
            dedup_mode: DedupMode::Allow,
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        };
        inputs.push((input, workspace));
    }

    // All inputs are valid — run a single atomic transaction.
    let result = ctx.storage.with_transaction(|conn| {
        let mut created = Vec::with_capacity(inputs.len());
        for (input, _ws) in &inputs {
            let memory = create_memory(conn, input)?;
            created.push(memory);
        }
        Ok(created)
    });

    match result {
        Ok(memories) => {
            // Invalidate cache for every distinct workspace that was written.
            let workspaces: HashSet<&str> = inputs.iter().map(|(_input, ws)| ws.as_str()).collect();
            for ws in &workspaces {
                ctx.search_cache.invalidate_for_workspace(Some(ws));
            }

            {
                let mut fuzzy = ctx.fuzzy_engine.lock();
                for memory in &memories {
                    fuzzy.add_to_vocabulary(&memory.content);
                }
            }
            for memory in &memories {
                if let Some(ref manager) = ctx.realtime {
                    manager.broadcast(RealtimeEvent::memory_created(
                        memory.id,
                        memory.content.clone(),
                    ));
                }
            }
            let ids: Vec<i64> = memories.iter().map(|m| m.id).collect();
            json!({"count": memories.len(), "ids": ids})
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}
