//! CRUD method bodies for `impl StorageBackend for TursoBackend`.
//!
//! Split out of `impls.rs` (ENG storage split) to keep files under the
//! repository's line-count limit. These are `pub(super)` free functions
//! called directly from the trait method stubs in `impls.rs`; behavior is
//! unchanged from the original single-file implementation.

use std::time::Instant;

use super::core::{TursoBackend, MEMORY_COLUMNS};
use crate::error::{EngramError, Result};
use crate::storage::backend::{BatchCreateResult, BatchDeleteResult, StorageBackend};
use crate::storage::queries::compute_dedup_hash;
use crate::types::{
    normalize_workspace, CreateMemoryInput, Memory, MemoryId, MemoryTier, UpdateMemoryInput,
};
use chrono::Utc;

pub(super) fn create_memory(backend: &TursoBackend, input: CreateMemoryInput) -> Result<Memory> {
    // Use tokio runtime to run async code in sync context
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
        let conn = backend.conn.write().await;

        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let importance = input.importance.unwrap_or(0.5);

        let workspace = normalize_workspace(input.workspace.as_deref().unwrap_or("default"))
            .map_err(|e| EngramError::InvalidInput(e.to_string()))?;

        let metadata_json = serde_json::to_string(&input.metadata)?;
        let scope_type = input.scope.scope_type();
        let scope_id = input.scope.scope_id().map(|s| s.to_string());
        let tier = input.tier;

        let expires_at = match tier {
            MemoryTier::Permanent => {
                if input.ttl_seconds.is_some() && input.ttl_seconds != Some(0) {
                    return Err(EngramError::InvalidInput(
                        "Permanent tier memories cannot have a TTL. Use Daily tier for expiring memories.".to_string(),
                    ));
                }
                None
            }
            MemoryTier::Daily => {
                let ttl = input.ttl_seconds.filter(|&t| t > 0).unwrap_or(86400);
                Some((now + chrono::Duration::seconds(ttl)).to_rfc3339())
            }
        };

        let content_hash = compute_dedup_hash(&input.content);
        let event_time = input.event_time.map(|dt| dt.to_rfc3339());

        conn.execute(
            "INSERT INTO memories (
                content, memory_type, importance, metadata, created_at, updated_at, valid_from,
                scope_type, scope_id, workspace, tier, expires_at, content_hash,
                event_time, event_duration_seconds, trigger_pattern, summary_of_id, lifecycle_state
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                input.content.clone(),
                input.memory_type.as_str(),
                importance as f64,
                metadata_json,
                now_str.clone(),
                now_str.clone(),
                now_str,
                scope_type,
                scope_id,
                workspace,
                tier.as_str(),
                expires_at,
                content_hash,
                event_time,
                input.event_duration_seconds,
                input.trigger_pattern.clone(),
                input.summary_of_id,
                crate::types::LifecycleState::Active.to_string(),
            ],
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        let id = conn.last_insert_rowid();

        // Insert tags
        for tag in &input.tags {
            // Ensure tag exists
            conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?)",
                libsql::params![tag.clone()],
            ).await.ok();

            // Link tag to memory
            conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag_id)
                 SELECT ?, id FROM tags WHERE name = ?",
                libsql::params![id, tag.clone()],
            ).await.ok();
        }

        drop(conn);

        let sql = format!(
            "SELECT {} FROM memories WHERE id = ? AND valid_to IS NULL",
            MEMORY_COLUMNS
        );
        let mut memories = backend
            .query_memories(&sql, vec![libsql::Value::Integer(id)])
            .await?;

        memories
            .pop()
            .ok_or_else(|| EngramError::NotFound(id))
    })
    })
}

pub(super) fn create_memories_batch(
    backend: &TursoBackend,
    inputs: Vec<CreateMemoryInput>,
) -> Result<BatchCreateResult> {
    let start = Instant::now();
    let mut created = Vec::new();
    let mut failed = Vec::new();

    for (idx, input) in inputs.into_iter().enumerate() {
        match backend.create_memory(input) {
            Ok(memory) => created.push(memory),
            Err(e) => failed.push((idx, e.to_string())),
        }
    }

    Ok(BatchCreateResult {
        created,
        failed,
        elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

pub(super) fn get_memory(backend: &TursoBackend, id: MemoryId) -> Result<Option<Memory>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let sql = format!(
                "SELECT {} FROM memories WHERE id = ? AND valid_to IS NULL",
                MEMORY_COLUMNS
            );
            let memories = backend
                .query_memories(&sql, vec![libsql::Value::Integer(id)])
                .await?;

            Ok(memories.into_iter().next())
        })
    })
}

pub(super) fn delete_memories_batch(
    backend: &TursoBackend,
    ids: Vec<MemoryId>,
) -> Result<BatchDeleteResult> {
    let mut deleted_count = 0;
    let mut not_found = Vec::new();
    let mut failed = Vec::new();

    for id in ids {
        match backend.delete_memory(id) {
            Ok(()) => deleted_count += 1,
            Err(EngramError::NotFound(_)) => not_found.push(id),
            Err(e) => failed.push((id, e.to_string())),
        }
    }

    Ok(BatchDeleteResult {
        deleted_count,
        not_found,
        failed,
    })
}

pub(super) fn update_memory(
    backend: &TursoBackend,
    id: MemoryId,
    input: UpdateMemoryInput,
) -> Result<Memory> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
        let conn = backend.conn.write().await;
        let now = Utc::now().to_rfc3339();

        let mut updates = vec!["updated_at = ?".to_string()];
        let mut params: Vec<libsql::Value> = vec![libsql::Value::Text(now)];

        if let Some(ref content) = input.content {
            updates.push("content = ?".to_string());
            params.push(libsql::Value::Text(content.clone()));
            let new_hash = compute_dedup_hash(content);
            updates.push("content_hash = ?".to_string());
            params.push(libsql::Value::Text(new_hash));
        }

        if let Some(ref memory_type) = input.memory_type {
            updates.push("memory_type = ?".to_string());
            params.push(libsql::Value::Text(memory_type.as_str().to_string()));
        }

        if let Some(importance) = input.importance {
            updates.push("importance = ?".to_string());
            params.push(libsql::Value::Real(importance as f64));
        }

        if let Some(ref metadata) = input.metadata {
            let metadata_json =
                serde_json::to_string(metadata).map_err(EngramError::Serialization)?;
            updates.push("metadata = ?".to_string());
            params.push(libsql::Value::Text(metadata_json));
        }

        if let Some(ref scope) = input.scope {
            updates.push("scope_type = ?".to_string());
            params.push(libsql::Value::Text(scope.scope_type().to_string()));
            updates.push("scope_id = ?".to_string());
            match scope.scope_id() {
                Some(id) => params.push(libsql::Value::Text(id.to_string())),
                None => params.push(libsql::Value::Null),
            }
        }

        if let Some(event_time) = &input.event_time {
            updates.push("event_time = ?".to_string());
            match event_time {
                Some(dt) => params.push(libsql::Value::Text(dt.to_rfc3339())),
                None => params.push(libsql::Value::Null),
            }
        }

        if let Some(trigger_pattern) = &input.trigger_pattern {
            updates.push("trigger_pattern = ?".to_string());
            match trigger_pattern {
                Some(value) => params.push(libsql::Value::Text(value.clone())),
                None => params.push(libsql::Value::Null),
            }
        }

        if let Some(ttl) = input.ttl_seconds {
            let mut rows = conn
                .query(
                    "SELECT tier FROM memories WHERE id = ? AND valid_to IS NULL",
                    libsql::params![id],
                )
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            let tier_row = rows
                .next()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            let tier_str: String = match tier_row {
                Some(row) => row.get(0).unwrap_or_else(|_| "permanent".to_string()),
                None => return Err(EngramError::NotFound(id)),
            };

            let tier = tier_str.parse().unwrap_or(MemoryTier::Permanent);

            if ttl <= 0 {
                if tier == MemoryTier::Daily {
                    return Err(EngramError::InvalidInput(
                        "Cannot remove expiration from a Daily tier memory. Use promote_to_permanent first.".to_string(),
                    ));
                }
                updates.push("expires_at = NULL".to_string());
            } else {
                if tier == MemoryTier::Permanent {
                    return Err(EngramError::InvalidInput(
                        "Cannot set expiration on a Permanent tier memory. Permanent memories cannot expire.".to_string(),
                    ));
                }
                let expires_at = (Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339();
                updates.push("expires_at = ?".to_string());
                params.push(libsql::Value::Text(expires_at));
            }
        }

        updates.push("version = version + 1".to_string());
        params.push(libsql::Value::Integer(id));

        let sql = format!(
            "UPDATE memories SET {} WHERE id = ? AND valid_to IS NULL",
            updates.join(", ")
        );

        conn.execute(&sql, params)
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?;

        if let Some(ref tags) = input.tags {
            conn.execute(
                "DELETE FROM memory_tags WHERE memory_id = ?",
                libsql::params![id],
            )
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?;

            for tag in tags {
                conn.execute(
                    "INSERT OR IGNORE INTO tags (name) VALUES (?)",
                    libsql::params![tag.clone()],
                )
                .await
                .ok();

                conn.execute(
                    "INSERT OR IGNORE INTO memory_tags (memory_id, tag_id)
                     SELECT ?, id FROM tags WHERE name = ?",
                    libsql::params![id, tag.clone()],
                )
                .await
                .ok();
            }
        }

        drop(conn);

        let sql = format!(
            "SELECT {} FROM memories WHERE id = ? AND valid_to IS NULL",
            MEMORY_COLUMNS
        );
        let mut memories = backend
            .query_memories(&sql, vec![libsql::Value::Integer(id)])
            .await?;
        memories.pop().ok_or_else(|| EngramError::NotFound(id))
    })
    })
}

pub(super) fn delete_memory(backend: &TursoBackend, id: MemoryId) -> Result<()> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.write().await;
            let now = chrono::Utc::now().to_rfc3339();

            // Soft delete by setting valid_to
            let affected = conn
                .execute(
                    "UPDATE memories SET valid_to = ? WHERE id = ? AND valid_to IS NULL",
                    libsql::params![now, id],
                )
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            if affected == 0 {
                return Err(EngramError::NotFound(id));
            }

            Ok(())
        })
    })
}
