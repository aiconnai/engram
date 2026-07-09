//! Read/update/delete + list tools.
use serde_json::{json, Value};

use super::super::HandlerContext;
use super::strip_private_content;
use crate::realtime::RealtimeEvent;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
use crate::storage::queries::*;
use crate::types::*;

pub fn memory_get(ctx: &HandlerContext, params: Value) -> Value {
    let id = params.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let do_strip = params
        .get("strip_private")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    ctx.storage
        .with_connection(|conn| {
            let mut memory = get_memory(conn, id)?;
            if do_strip {
                memory.content = strip_private_content(&memory.content);
            }
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Variant of `memory_get` that always strips `<private>…</private>` sections.
///
/// Equivalent to calling `memory_get` with `strip_private: true`.
pub fn memory_get_public(ctx: &HandlerContext, params: Value) -> Value {
    let id = params.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    ctx.storage
        .with_connection(|conn| {
            let mut memory = get_memory(conn, id)?;
            memory.content = strip_private_content(&memory.content);
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_update(ctx: &HandlerContext, params: Value) -> Value {
    let id = params.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let input: UpdateMemoryInput = match serde_json::from_value(params.clone()) {
        Ok(i) => i,
        Err(e) => return json!({"error": e.to_string()}),
    };

    let mut changes = Vec::new();
    if input.content.is_some() {
        changes.push("content".to_string());
    }
    if input.memory_type.is_some() {
        changes.push("memory_type".to_string());
    }
    if input.tags.is_some() {
        changes.push("tags".to_string());
    }
    if input.metadata.is_some() {
        changes.push("metadata".to_string());
    }
    if input.importance.is_some() {
        changes.push("importance".to_string());
    }

    let result = ctx.storage.with_transaction(|conn| {
        let memory = update_memory(conn, id, &input)?;
        let op_id = uuid::Uuid::new_v4().to_string();
        emit_best_effort(
            conn,
            &EnrichmentEvent {
                operation_id: &op_id,
                event_type: "memory_updated",
                memory_id: Some(memory.id),
                version_id: None,
                triggered_by: "memory_update",
                agent_id: None,
                workspace: Some(memory.workspace.as_str()),
                params: json!({"fields_changed": changes}),
                outcome: json!({"id": memory.id}),
                status: "completed",
                dry_run: false,
            },
        );
        Ok(memory)
    });

    match result {
        Ok(memory) => {
            ctx.search_cache.invalidate_for_memory(memory.id);
            if let Some(ref manager) = ctx.realtime {
                manager.broadcast(RealtimeEvent::memory_updated(memory.id, changes));
            }
            json!(memory)
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn memory_delete(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::collect_supersedes_chain;

    let id = params.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let cascade_chain = params
        .get("cascade_chain")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if cascade_chain {
        let result = ctx.storage.with_transaction(|conn| {
            let chain = collect_supersedes_chain(conn, id)?;
            // Capture workspace from the root before any deletion.
            let workspace = get_memory(conn, id).ok().map(|m| m.workspace);
            let op_id = uuid::Uuid::new_v4().to_string();
            for &mem_id in &chain {
                delete_memory(conn, mem_id)?;
                // One audit event per deleted member so memory_enrichment_timeline
                // returns results for every memory_id in the chain, not just the root.
                emit_best_effort(
                    conn,
                    &EnrichmentEvent {
                        operation_id: &op_id,
                        event_type: "memory_deleted",
                        memory_id: Some(mem_id),
                        version_id: None,
                        triggered_by: "memory_delete",
                        agent_id: None,
                        workspace: workspace.as_deref(),
                        params: json!({"cascade_chain": true, "root_id": id}),
                        outcome: json!({"id": mem_id}),
                        status: "completed",
                        dry_run: false,
                    },
                );
            }
            Ok(chain)
        });

        match result {
            Ok(deleted_ids) => {
                for &deleted_id in &deleted_ids {
                    ctx.search_cache.invalidate_for_memory(deleted_id);
                    if let Some(ref manager) = ctx.realtime {
                        manager.broadcast(RealtimeEvent::memory_deleted(deleted_id));
                    }
                }
                let count = deleted_ids.len();
                json!({"deleted_ids": deleted_ids, "count": count})
            }
            Err(e) => json!({"error": e.to_string()}),
        }
    } else {
        let result = ctx.storage.with_transaction(|conn| {
            // Capture workspace before deletion while the row still exists.
            let workspace = get_memory(conn, id).ok().map(|m| m.workspace);
            delete_memory(conn, id)?;
            let op_id = uuid::Uuid::new_v4().to_string();
            emit_best_effort(
                conn,
                &EnrichmentEvent {
                    operation_id: &op_id,
                    event_type: "memory_deleted",
                    memory_id: Some(id),
                    version_id: None,
                    triggered_by: "memory_delete",
                    agent_id: None,
                    workspace: workspace.as_deref(),
                    params: json!({"cascade_chain": false}),
                    outcome: json!({"id": id}),
                    status: "completed",
                    dry_run: false,
                },
            );
            Ok(id)
        });

        match result {
            Ok(deleted_id) => {
                ctx.search_cache.invalidate_for_memory(deleted_id);
                if let Some(ref manager) = ctx.realtime {
                    manager.broadcast(RealtimeEvent::memory_deleted(deleted_id));
                }
                json!({"deleted": deleted_id})
            }
            Err(e) => json!({"error": e.to_string()}),
        }
    }
}

pub fn memory_list(ctx: &HandlerContext, params: Value) -> Value {
    let options: ListOptions = serde_json::from_value(params).unwrap_or_default();
    ctx.storage
        .with_connection(|conn| {
            let memories = list_memories(conn, &options)?;
            Ok(json!(memories))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_delete_batch(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::delete_memory_batch;
    use crate::storage::queries::collect_supersedes_chain;

    let ids: Vec<i64> = match params.get("ids").and_then(|v| v.as_array()) {
        Some(arr) => arr.iter().filter_map(|v| v.as_i64()).collect(),
        None => return json!({"error": "ids array is required"}),
    };

    if ids.is_empty() {
        return json!({"error": "No valid IDs provided"});
    }

    let cascade_chain = params
        .get("cascade_chain")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if cascade_chain {
        ctx.storage
            .with_transaction(|conn| {
                let mut expanded: Vec<i64> = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for &id in &ids {
                    let chain = collect_supersedes_chain(conn, id)?;
                    for chain_id in chain {
                        if seen.insert(chain_id) {
                            expanded.push(chain_id);
                        }
                    }
                }
                let result = delete_memory_batch(conn, &expanded)?;
                Ok(json!(result))
            })
            .unwrap_or_else(|e| json!({"error": e.to_string()}))
    } else {
        ctx.storage
            .with_connection(|conn| {
                let result = delete_memory_batch(conn, &ids)?;
                Ok(json!(result))
            })
            .unwrap_or_else(|e| json!({"error": e.to_string()}))
    }
}
