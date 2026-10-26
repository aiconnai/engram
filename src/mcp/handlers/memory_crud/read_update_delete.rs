//! Read/update/delete + list tools.
use serde_json::{json, Value};

use super::super::HandlerContext;
use super::strip_private_content;
use crate::mcp::error::ToolError;
use crate::mcp::progress::ProgressReporterExt;
use crate::mcp::workspace_guard::ensure_memory_access;
use crate::realtime::RealtimeEvent;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
use crate::storage::queries::*;
use crate::types::*;

pub(crate) fn extract_id(params: &Value, name: &str) -> std::result::Result<i64, Value> {
    match params.get(name) {
        None | Some(Value::Null) => Err(ToolError::missing_argument(name).into_value()),
        Some(v) => match v.as_i64() {
            Some(id) if id > 0 => Ok(id),
            Some(_) => Err(
                ToolError::invalid_params(format!("{name} must be a positive integer"))
                    .into_value(),
            ),
            None => {
                Err(ToolError::invalid_params(format!("{name} must be an integer")).into_value())
            }
        },
    }
}

pub(crate) fn validate_non_negative_int(
    params: &Value,
    name: &str,
) -> std::result::Result<Option<i64>, Value> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => match v.as_i64() {
            Some(val) if val >= 0 => Ok(Some(val)),
            _ => Err(
                ToolError::invalid_params(format!("{name} must be a non-negative integer"))
                    .into_value(),
            ),
        },
    }
}

pub fn memory_get(ctx: &HandlerContext, params: Value) -> Value {
    let id = match extract_id(&params, "id") {
        Ok(id) => id,
        Err(err) => return err,
    };
    let do_strip = params
        .get("strip_private")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    ctx.storage
        .with_connection(|conn| {
            // Authorize the persisted row before access tracking/reinforcement.
            ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let mut memory = get_memory(conn, id)?;
            if let Ok(Some(new_stability)) =
                crate::intelligence::stability::record_reinforcement(conn, id, chrono::Utc::now())
            {
                memory.stability = new_stability;
            }
            if do_strip {
                memory.content = strip_private_content(&memory.content);
            }
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Variant of `memory_get` that always strips `<private>…</private>` sections.
///
/// Equivalent to calling `memory_get` with `strip_private: true`.
pub fn memory_get_public(ctx: &HandlerContext, params: Value) -> Value {
    let id = match extract_id(&params, "id") {
        Ok(id) => id,
        Err(err) => return err,
    };
    ctx.storage
        .with_connection(|conn| {
            ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let mut memory = get_memory(conn, id)?;
            if let Ok(Some(new_stability)) =
                crate::intelligence::stability::record_reinforcement(conn, id, chrono::Utc::now())
            {
                memory.stability = new_stability;
            }
            memory.content = strip_private_content(&memory.content);
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Drop deleted / content-changed memories from the in-memory vector index so
/// search never serves a stale or deleted vector. The embedding drain re-inserts
/// the fresh vector once a changed memory is re-embedded.
fn drop_from_vector_index(ctx: &HandlerContext, ids: &[i64]) {
    let mut index = ctx.hnsw_index.write();
    for id in ids {
        index.remove(id);
    }
}

pub fn memory_update(ctx: &HandlerContext, params: Value) -> Value {
    let id = match extract_id(&params, "id") {
        Ok(id) => id,
        Err(err) => return err,
    };
    let input: UpdateMemoryInput = match serde_json::from_value(params.clone()) {
        Ok(i) => i,
        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
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
        // Same transaction as the mutation: the row cannot change workspace
        // between authorization and update.
        ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
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
            if changes.iter().any(|c| c == "content") {
                drop_from_vector_index(ctx, &[memory.id]);
            }
            if let Some(ref manager) = ctx.realtime {
                manager.broadcast(RealtimeEvent::memory_updated(
                    memory.id,
                    changes,
                    memory.workspace.clone(),
                ));
            }
            json!(memory)
        }
        Err(e) => ToolError::from(e).into_value(),
    }
}

pub fn memory_delete(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::collect_supersedes_chain;

    let id = match extract_id(&params, "id") {
        Ok(id) => id,
        Err(err) => return err,
    };
    let cascade_chain = params
        .get("cascade_chain")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if cascade_chain {
        let result = ctx.storage.with_transaction(|conn| {
            let chain = collect_supersedes_chain(conn, id)?;
            ensure_chain_access(conn, ctx, id, &chain)?;
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
            Ok((chain, workspace))
        });

        match result {
            Ok((deleted_ids, workspace)) => {
                drop_from_vector_index(ctx, &deleted_ids);
                for &deleted_id in &deleted_ids {
                    ctx.search_cache.invalidate_for_memory(deleted_id);
                    if let (Some(manager), Some(workspace)) = (&ctx.realtime, workspace.as_deref())
                    {
                        manager.broadcast(RealtimeEvent::memory_deleted(deleted_id, workspace));
                    }
                }
                let count = deleted_ids.len();
                json!({"deleted_ids": deleted_ids, "count": count})
            }
            Err(e) => ToolError::from(e).into_value(),
        }
    } else {
        let result = ctx.storage.with_transaction(|conn| {
            ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
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
            Ok((id, workspace))
        });

        match result {
            Ok((deleted_id, workspace)) => {
                drop_from_vector_index(ctx, &[deleted_id]);
                ctx.search_cache.invalidate_for_memory(deleted_id);
                if let (Some(manager), Some(workspace)) = (&ctx.realtime, workspace.as_deref()) {
                    manager.broadcast(RealtimeEvent::memory_deleted(deleted_id, workspace));
                }
                json!({"deleted": deleted_id})
            }
            Err(e) => ToolError::from(e).into_value(),
        }
    }
}

pub fn memory_list(ctx: &HandlerContext, params: Value) -> Value {
    if let Err(err) = validate_non_negative_int(&params, "limit") {
        return err;
    }
    if let Err(err) = validate_non_negative_int(&params, "offset") {
        return err;
    }
    let options: ListOptions = match serde_json::from_value(params) {
        Ok(opts) => opts,
        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
    };
    ctx.storage
        .with_connection(|conn| {
            let memories = list_memories(conn, &options)?;
            Ok(json!(memories))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_delete_batch(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::{
        collect_supersedes_chain, get_memory, BatchDeleteResult, BatchError,
    };

    let ids_val = match params.get("ids") {
        None | Some(Value::Null) => return ToolError::missing_argument("ids").into_value(),
        Some(v) => v,
    };
    let arr = match ids_val.as_array() {
        Some(arr) => arr,
        None => return ToolError::invalid_params("ids must be an array of integers").into_value(),
    };
    let mut ids: Vec<i64> = Vec::with_capacity(arr.len());
    for item in arr {
        match item.as_i64() {
            Some(id) if id > 0 => ids.push(id),
            _ => {
                return ToolError::invalid_params("ids array must contain positive integers")
                    .into_value()
            }
        }
    }

    if ids.is_empty() {
        return ToolError::invalid_params("No valid IDs provided").into_value();
    }

    let cascade_chain = params
        .get("cascade_chain")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let initial_count = ids.len() as u64;
    ctx.reporter().step(
        0,
        initial_count,
        format!("Starting batch deletion of {initial_count} memories"),
    );

    let result = ctx.storage.with_transaction(|conn| {
        let target_ids: Vec<i64> = if cascade_chain {
            let mut expanded: Vec<i64> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for (i, &id) in ids.iter().enumerate() {
                ctx.reporter().step(
                    (i + 1) as u64,
                    initial_count,
                    format!(
                        "Resolving supersedes chain for memory {id} ({}/{})",
                        i + 1,
                        initial_count
                    ),
                );
                let chain = collect_supersedes_chain(conn, id)?;
                ensure_chain_access(conn, ctx, id, &chain)?;
                for chain_id in chain {
                    if seen.insert(chain_id) {
                        expanded.push(chain_id);
                    }
                }
            }
            expanded
        } else {
            for &id in &ids {
                ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            }
            ids.clone()
        };

        // Capture workspaces before deletion
        let workspaces: Vec<(i64, Option<String>)> = target_ids
            .iter()
            .map(|&id| (id, get_memory(conn, id).ok().map(|m| m.workspace)))
            .collect();

        let total_work = target_ids.len() as u64;
        let mut deleted = Vec::new();
        let mut failed = Vec::new();

        for (index, &id) in target_ids.iter().enumerate() {
            let step_num = (index + 1) as u64;
            match delete_memory(conn, id) {
                Ok(()) => {
                    ctx.reporter().step(
                        step_num,
                        total_work,
                        format!("Deleted memory {id} ({step_num}/{total_work})"),
                    );
                    deleted.push(id);
                }
                Err(e) => {
                    ctx.reporter().step(
                        step_num,
                        total_work,
                        format!("Failed deleting memory {id} ({step_num}/{total_work}): {e}"),
                    );
                    failed.push(BatchError {
                        index,
                        id: Some(id),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok((
            BatchDeleteResult {
                total_deleted: deleted.len(),
                total_failed: failed.len(),
                deleted,
                failed,
            },
            workspaces,
            total_work,
        ))
    });

    match result {
        Ok((res, workspaces, total_work)) => {
            ctx.reporter().complete(
                total_work,
                format!("Successfully deleted {} memories", res.total_deleted),
            );

            drop_from_vector_index(ctx, &res.deleted);
            // Invalidate search cache and broadcast realtime events for deleted items
            for (id, workspace) in workspaces {
                if res.deleted.contains(&id) {
                    ctx.search_cache.invalidate_for_memory(id);
                    if let (Some(manager), Some(ws)) = (&ctx.realtime, workspace.as_deref()) {
                        manager.broadcast(RealtimeEvent::memory_deleted(id, ws));
                    }
                }
            }

            json!(res)
        }
        Err(e) => ToolError::from(e).into_value(),
    }
}

/// Authorize every member of a supersedes chain inside the deleting
/// transaction. A foreign member is reported as the requested root not being
/// found, so the error never names a row the caller did not supply.
fn ensure_chain_access(
    conn: &rusqlite::Connection,
    ctx: &HandlerContext,
    root_id: i64,
    chain: &[i64],
) -> crate::error::Result<()> {
    for &member in chain {
        if ensure_memory_access(conn, ctx.principal.as_ref(), member).is_err() {
            return Err(crate::error::EngramError::NotFound(root_id));
        }
    }
    Ok(())
}
