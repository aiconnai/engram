//! Cross-reference / tag / workspace method bodies for
//! `impl StorageBackend for TursoBackend`.
//!
//! Split out of `impls.rs` (ENG storage split) to keep files under the
//! repository's line-count limit. These are `pub(super)` free functions
//! called directly from the trait method stubs in `impls.rs`; behavior is
//! unchanged from the original single-file implementation.

use std::collections::HashMap;

use super::core::TursoBackend;
use crate::error::{EngramError, Result};
use crate::storage::backend::StorageBackend;
use crate::types::{CrossReference, EdgeType, ListOptions, Memory, MemoryId, RelationSource};
use chrono::{DateTime, Utc};

pub(super) fn create_crossref(
    backend: &TursoBackend,
    from_id: MemoryId,
    to_id: MemoryId,
    edge_type: EdgeType,
    score: f32,
) -> Result<CrossReference> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
        let conn = backend.conn.write().await;
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO crossrefs
             (from_id, to_id, edge_type, score, confidence, strength, source, source_context, created_at, valid_from, pinned, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                from_id,
                to_id,
                edge_type.as_str(),
                score as f64,
                1.0f64,
                score as f64,
                "auto",
                Option::<String>::None,
                now_str.clone(),
                now_str,
                0i64,
                "{}",
            ],
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        Ok(CrossReference {
            from_id,
            to_id,
            edge_type,
            score,
            confidence: 1.0,
            strength: score,
            source: RelationSource::Auto,
            source_context: None,
            created_at: now,
            valid_from: now,
            valid_to: None,
            pinned: false,
            metadata: HashMap::new(),
        })
    })
    })
}

pub(super) fn get_crossrefs(
    backend: &TursoBackend,
    memory_id: MemoryId,
) -> Result<Vec<CrossReference>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.read().await;
            let stmt = conn
                .prepare(
                    "SELECT from_id, to_id, edge_type, score, confidence, strength, source,
                    source_context, created_at, valid_from, valid_to, pinned, metadata
             FROM crossrefs WHERE (from_id = ? OR to_id = ?) AND valid_to IS NULL",
                )
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            let rows = stmt
                .query(libsql::params![memory_id, memory_id])
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            let mut crossrefs = Vec::new();
            let mut rows = rows;

            while let Some(row) = rows
                .next()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?
            {
                let edge_type_str: String = row.get(2).unwrap_or_else(|_| "related_to".to_string());
                let source_str: String = row.get(6).unwrap_or_else(|_| "auto".to_string());
                let created_at_str: String = row.get(8).unwrap_or_else(|_| Utc::now().to_rfc3339());
                let valid_from_str: String = row.get(9).unwrap_or_else(|_| Utc::now().to_rfc3339());
                let valid_to_str: Option<String> = row.get(10).unwrap_or(None);
                let metadata_str: String = row.get(12).unwrap_or_else(|_| "{}".to_string());
                crossrefs.push(CrossReference {
                    from_id: row.get(0).unwrap_or(0),
                    to_id: row.get(1).unwrap_or(0),
                    edge_type: edge_type_str.parse().unwrap_or(EdgeType::RelatedTo),
                    score: row.get::<f64>(3).unwrap_or(0.0) as f32,
                    confidence: row.get::<f64>(4).unwrap_or(1.0) as f32,
                    strength: row.get::<f64>(5).unwrap_or(1.0) as f32,
                    source: match source_str.as_str() {
                        "manual" => RelationSource::Manual,
                        "llm" => RelationSource::Llm,
                        _ => RelationSource::Auto,
                    },
                    source_context: row.get(7).ok(),
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    valid_from: DateTime::parse_from_rfc3339(&valid_from_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    valid_to: valid_to_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    pinned: row.get::<i64>(11).unwrap_or(0) != 0,
                    metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
                });
            }

            Ok(crossrefs)
        })
    })
}

pub(super) fn delete_crossref(
    backend: &TursoBackend,
    from_id: MemoryId,
    to_id: MemoryId,
) -> Result<()> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
        let conn = backend.conn.write().await;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE crossrefs SET valid_to = ? WHERE from_id = ? AND to_id = ? AND valid_to IS NULL",
            libsql::params![now, from_id, to_id],
        ).await.map_err(|e| EngramError::Storage(e.to_string()))?;

        Ok(())
    })
    })
}

pub(super) fn list_tags(backend: &TursoBackend) -> Result<Vec<(String, i64)>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.read().await;
            let stmt = conn
                .prepare(
                    "SELECT t.name, COUNT(mt.memory_id) as count
             FROM tags t
             LEFT JOIN memory_tags mt ON t.id = mt.tag_id
             LEFT JOIN memories m ON mt.memory_id = m.id AND m.valid_to IS NULL
             GROUP BY t.id, t.name
             ORDER BY count DESC",
                )
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            let rows = stmt
                .query(())
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            let mut tags = Vec::new();
            let mut rows = rows;

            while let Some(row) = rows
                .next()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?
            {
                let name: String = row.get(0).unwrap_or_default();
                let count: i64 = row.get(1).unwrap_or(0);
                tags.push((name, count));
            }

            Ok(tags)
        })
    })
}

pub(super) fn get_memories_by_tag(
    backend: &TursoBackend,
    tag: &str,
    limit: Option<usize>,
) -> Result<Vec<Memory>> {
    backend.list_memories(ListOptions {
        tags: Some(vec![tag.to_string()]),
        limit: limit.map(|l| l as i64),
        ..Default::default()
    })
}

pub(super) fn list_workspaces(backend: &TursoBackend) -> Result<Vec<(String, i64)>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.read().await;
            let stmt = conn.prepare(
            "SELECT workspace, COUNT(*) FROM memories WHERE valid_to IS NULL GROUP BY workspace"
        ).await.map_err(|e| EngramError::Storage(e.to_string()))?;

            let rows = stmt
                .query(())
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            let mut workspaces = Vec::new();
            let mut rows = rows;

            while let Some(row) = rows
                .next()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?
            {
                let name: String = row.get(0).unwrap_or_else(|_| "default".to_string());
                let count: i64 = row.get(1).unwrap_or(0);
                workspaces.push((name, count));
            }

            Ok(workspaces)
        })
    })
}

pub(super) fn get_workspace_stats(
    backend: &TursoBackend,
    workspace: &str,
) -> Result<HashMap<String, i64>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
        let conn = backend.conn.read().await;

        let total: i64 = conn.query(
            "SELECT COUNT(*) FROM memories WHERE workspace = ? AND valid_to IS NULL",
            libsql::params![workspace.to_string()],
        ).await.map_err(|e| EngramError::Storage(e.to_string()))?
            .next().await.ok().flatten()
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        let permanent: i64 = conn.query(
            "SELECT COUNT(*) FROM memories WHERE workspace = ? AND tier = 'permanent' AND valid_to IS NULL",
            libsql::params![workspace.to_string()],
        ).await.map_err(|e| EngramError::Storage(e.to_string()))?
            .next().await.ok().flatten()
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        let daily: i64 = conn.query(
            "SELECT COUNT(*) FROM memories WHERE workspace = ? AND tier = 'daily' AND valid_to IS NULL",
            libsql::params![workspace.to_string()],
        ).await.map_err(|e| EngramError::Storage(e.to_string()))?
            .next().await.ok().flatten()
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        let mut stats = HashMap::new();
        stats.insert("memory_count".to_string(), total);
        stats.insert("permanent_count".to_string(), permanent);
        stats.insert("daily_count".to_string(), daily);
        Ok(stats)
    })
    })
}

pub(super) fn move_to_workspace(
    backend: &TursoBackend,
    ids: Vec<MemoryId>,
    workspace: &str,
) -> Result<usize> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.write().await;
            let mut moved = 0usize;

            for id in ids {
                let result = conn
                    .execute(
                        "UPDATE memories SET workspace = ? WHERE id = ? AND valid_to IS NULL",
                        libsql::params![workspace.to_string(), id],
                    )
                    .await;

                if result.is_ok() {
                    moved += 1;
                }
            }

            Ok(moved)
        })
    })
}
