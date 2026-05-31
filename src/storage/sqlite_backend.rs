//! SQLite implementation of the StorageBackend trait (ENG-15)
//!
//! This module provides a SQLite-based storage backend that implements
//! the `StorageBackend` trait, allowing the existing SQLite storage
//! to be used through the abstracted interface.

use std::collections::HashMap;
use std::time::Instant;

use crate::embedding::{
    get_embedding_queue_health, DEFAULT_MAX_EMBEDDING_RETRIES, DEFAULT_STALE_PROCESSING_AFTER,
};
use crate::error::Result;
use crate::types::{
    CreateCrossRefInput, CreateMemoryInput, CrossReference, EdgeType, ListOptions, Memory,
    MemoryId, SearchOptions, SearchResult, StorageConfig, UpdateMemoryInput, WorkspaceStats,
};

use super::backend::{
    BatchCreateResult, BatchDeleteResult, CloudSyncBackend, DerivedIndexHealth, DerivedIndexKind,
    DerivedIndexStatus, HealthStatus, StorageBackend, StorageStats, SyncDelta, SyncResult,
    SyncState, TransactionalBackend,
};
use super::connection::Storage;
use super::queries::{
    self, delete_memory_batch, get_related, get_sync_delta, get_sync_version, list_tags,
};
use crate::search::{hybrid_search, SearchConfig};

/// SQLite-based storage backend
///
/// This implements the `StorageBackend` trait using SQLite as the
/// underlying database. It wraps the existing `Storage` struct and
/// delegates to the functions in `queries.rs`.
pub struct SqliteBackend {
    storage: Storage,
}

impl SqliteBackend {
    /// Create a new SQLite backend with the given configuration
    pub fn new(config: StorageConfig) -> Result<Self> {
        let storage = Storage::open(config)?;
        Ok(Self { storage })
    }

    /// Create an in-memory SQLite backend (useful for testing)
    pub fn in_memory() -> Result<Self> {
        let storage = Storage::open_in_memory()?;
        Ok(Self { storage })
    }

    /// Get a reference to the underlying Storage
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Get a mutable reference to the underlying Storage
    pub fn storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }
}

/// Check SQLite storage health using an already-open storage handle.
///
/// This is intentionally separate from `SqliteBackend::new(...).health_check()`
/// so read-only callers, such as CLI status reporting, do not reopen the
/// database and trigger migrations or connection pragmas.
pub fn health_check_storage(storage: &Storage) -> Result<HealthStatus> {
    let start = Instant::now();

    let storage_mode_warning = storage.storage_mode_warning();
    let db_path = storage.db_path().to_string();

    let result = storage.with_connection(|conn| {
        conn.query_row("SELECT 1", [], |_| Ok(()))?;

        let quick_check: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
        let quick_check_ok = quick_check == "ok";
        let quick_check_status = if quick_check_ok {
            "ok".to_string()
        } else {
            quick_check
        };

        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;
        let reclaimable_bytes = page_size * freelist_count;
        let db_size_bytes = page_size * page_count;

        let derived_indexes = sqlite_derived_index_health(conn)?;

        Ok((
            derived_indexes,
            quick_check_status,
            quick_check_ok,
            page_size,
            page_count,
            db_size_bytes,
            freelist_count,
            reclaimable_bytes,
        ))
    });

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok((
            derived_indexes,
            quick_check,
            quick_check_ok,
            page_size,
            page_count,
            db_size_bytes,
            freelist_count,
            reclaimable_bytes,
        )) => {
            let mut details = HashMap::from([
                ("db_path".to_string(), db_path),
                (
                    "storage_mode".to_string(),
                    format!("{:?}", storage.storage_mode()),
                ),
                ("quick_check".to_string(), quick_check.clone()),
                ("page_size".to_string(), page_size.to_string()),
                ("page_count".to_string(), page_count.to_string()),
                ("db_size_bytes".to_string(), db_size_bytes.to_string()),
                ("freelist_count".to_string(), freelist_count.to_string()),
                (
                    "reclaimable_bytes".to_string(),
                    reclaimable_bytes.to_string(),
                ),
            ]);
            if let Some(warning) = storage_mode_warning {
                details.insert("warning".to_string(), warning);
            }

            let healthy = quick_check_ok;
            Ok(HealthStatus {
                healthy,
                latency_ms,
                error: if healthy {
                    None
                } else {
                    Some(format!("quick_check failed: {quick_check}"))
                },
                details,
                derived_indexes,
            })
        }
        Err(e) => Ok(HealthStatus {
            healthy: false,
            latency_ms,
            error: Some(e.to_string()),
            details: HashMap::from([("db_path".to_string(), db_path)]),
            derived_indexes: Vec::new(),
        }),
    }
}

impl StorageBackend for SqliteBackend {
    fn create_memory(&self, input: CreateMemoryInput) -> Result<Memory> {
        self.storage
            .with_transaction(|conn| queries::create_memory(conn, &input))
    }

    fn get_memory(&self, id: MemoryId) -> Result<Option<Memory>> {
        self.storage
            .with_connection(|conn| match queries::get_memory(conn, id) {
                Ok(memory) => Ok(Some(memory)),
                Err(crate::error::EngramError::NotFound(_)) => Ok(None),
                Err(e) => Err(e),
            })
    }

    fn update_memory(&self, id: MemoryId, input: UpdateMemoryInput) -> Result<Memory> {
        self.storage
            .with_transaction(|conn| queries::update_memory(conn, id, &input))
    }

    fn delete_memory(&self, id: MemoryId) -> Result<()> {
        self.storage
            .with_transaction(|conn| queries::delete_memory(conn, id))
    }

    fn create_memories_batch(&self, inputs: Vec<CreateMemoryInput>) -> Result<BatchCreateResult> {
        let start = Instant::now();
        let mut created = Vec::new();
        let mut failed = Vec::new();

        self.storage.with_transaction(|conn| {
            for (idx, input) in inputs.into_iter().enumerate() {
                match queries::create_memory(conn, &input) {
                    Ok(memory) => created.push(memory),
                    Err(e) => failed.push((idx, e.to_string())),
                }
            }
            Ok(())
        })?;

        Ok(BatchCreateResult {
            created,
            failed,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    fn delete_memories_batch(&self, ids: Vec<MemoryId>) -> Result<BatchDeleteResult> {
        self.storage.with_transaction(|conn| {
            let result = delete_memory_batch(conn, &ids)?;
            let mut not_found = Vec::new();
            let mut failed = Vec::new();

            for err in &result.failed {
                if let Some(id) = err.id {
                    let msg = err.error.clone();
                    // Heuristic to detect not found errors from bulk operation
                    if msg.to_lowercase().contains("notfound")
                        || msg.to_lowercase().contains("not found")
                    {
                        not_found.push(id);
                    } else {
                        failed.push((id, msg));
                    }
                }
            }

            Ok(BatchDeleteResult {
                deleted_count: result.total_deleted,
                not_found,
                failed,
            })
        })
    }

    fn list_memories(&self, options: ListOptions) -> Result<Vec<Memory>> {
        self.storage
            .with_connection(|conn| queries::list_memories(conn, &options))
    }

    fn count_memories(&self, options: ListOptions) -> Result<i64> {
        self.storage.with_connection(|conn| {
            let now = chrono::Utc::now().to_rfc3339();

            let mut sql = String::from("SELECT COUNT(DISTINCT m.id) FROM memories m");
            let mut conditions = vec!["m.valid_to IS NULL".to_string()];
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            // Exclude expired memories
            conditions.push("(m.expires_at IS NULL OR m.expires_at > ?)".to_string());
            params.push(Box::new(now));

            // Tag filter (requires join)
            if let Some(ref tags) = options.tags {
                if !tags.is_empty() {
                    sql.push_str(
                        " JOIN memory_tags mt ON m.id = mt.memory_id
                          JOIN tags t ON mt.tag_id = t.id",
                    );
                    let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
                    conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
                    for tag in tags {
                        params.push(Box::new(tag.clone()));
                    }
                }
            }

            // Type filter
            if let Some(ref memory_type) = options.memory_type {
                conditions.push("m.memory_type = ?".to_string());
                params.push(Box::new(memory_type.as_str().to_string()));
            }

            // Metadata filter (JSON)
            if let Some(ref metadata_filter) = options.metadata_filter {
                for (key, value) in metadata_filter {
                    queries::metadata_value_to_param(key, value, &mut conditions, &mut params)?;
                }
            }

            // Scope filter
            if let Some(ref scope) = options.scope {
                conditions.push("m.scope_type = ?".to_string());
                params.push(Box::new(scope.scope_type().to_string()));
                if let Some(scope_id) = scope.scope_id() {
                    conditions.push("m.scope_id = ?".to_string());
                    params.push(Box::new(scope_id.to_string()));
                } else {
                    conditions.push("m.scope_id IS NULL".to_string());
                }
            }

            // Workspace filter
            if let Some(ref workspace) = options.workspace {
                conditions.push("m.workspace = ?".to_string());
                params.push(Box::new(workspace.clone()));
            }

            // Tier filter
            if let Some(ref tier) = options.tier {
                conditions.push("m.tier = ?".to_string());
                params.push(Box::new(tier.as_str().to_string()));
            }

            // Archived filter
            if !options.include_archived {
                conditions.push(
                    "(m.lifecycle_state IS NULL OR m.lifecycle_state != 'archived')".to_string(),
                );
            }

            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
            let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;

            Ok(count)
        })
    }

    fn search_memories(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        self.storage.with_connection(|conn| {
            let config = SearchConfig::default();
            // Note: hybrid_search expects embedding if vector search is desired.
            // Here we only perform lexical/fuzzy search unless embedding is handled higher up.
            // The trait signature doesn't take embedding, implying embedding generation happens
            // outside or inside if we had the embedder.
            // Since SqliteBackend doesn't have the Embedder, we pass None.
            hybrid_search(conn, query, None, &options, &config)
        })
    }

    fn create_crossref(
        &self,
        from_id: MemoryId,
        to_id: MemoryId,
        edge_type: EdgeType,
        score: f32,
    ) -> Result<CrossReference> {
        self.storage.with_transaction(|conn| {
            let input = CreateCrossRefInput {
                from_id,
                to_id,
                edge_type,
                strength: Some(score),
                source_context: None,
                pinned: false,
            };
            queries::create_crossref(conn, &input)
        })
    }

    fn get_crossrefs(&self, memory_id: MemoryId) -> Result<Vec<CrossReference>> {
        self.storage
            .with_connection(|conn| get_related(conn, memory_id))
    }

    fn delete_crossref(&self, from_id: MemoryId, to_id: MemoryId) -> Result<()> {
        self.storage.with_transaction(|conn| {
            // Try to delete both directions if bidirectional?
            // The trait implies directed deletion.
            // We use queries::delete_crossref which takes an edge type.
            // We'll delete all edge types for this pair.
            for edge_type in EdgeType::all() {
                // Ignore result (might not exist for all types)
                let _ = queries::delete_crossref(conn, from_id, to_id, *edge_type);
            }
            Ok(())
        })
    }

    fn list_tags(&self) -> Result<Vec<(String, i64)>> {
        self.storage.with_connection(|conn| {
            let tags = list_tags(conn)?;
            Ok(tags.into_iter().map(|t| (t.name, t.count)).collect())
        })
    }

    fn get_memories_by_tag(&self, tag: &str, limit: Option<usize>) -> Result<Vec<Memory>> {
        self.storage.with_connection(|conn| {
            let options = ListOptions {
                tags: Some(vec![tag.to_string()]),
                limit: limit.map(|v| v as i64),
                ..Default::default()
            };
            queries::list_memories(conn, &options)
        })
    }

    fn list_workspaces(&self) -> Result<Vec<(String, i64)>> {
        self.storage.with_connection(|conn| {
            let workspaces = queries::list_workspaces(conn)?;
            Ok(workspaces
                .into_iter()
                .map(|w| (w.workspace, w.memory_count))
                .collect())
        })
    }

    fn get_workspace_stats(&self, workspace: &str) -> Result<HashMap<String, i64>> {
        self.storage.with_connection(|conn| {
            let stats: WorkspaceStats = queries::get_workspace_stats(conn, workspace)?;
            let mut map = HashMap::new();
            map.insert("memory_count".to_string(), stats.memory_count);
            map.insert("permanent_count".to_string(), stats.permanent_count);
            map.insert("daily_count".to_string(), stats.daily_count);
            Ok(map)
        })
    }

    fn move_to_workspace(&self, ids: Vec<MemoryId>, workspace: &str) -> Result<usize> {
        self.storage.with_transaction(|conn| {
            let mut moved = 0usize;
            for id in ids {
                if queries::move_to_workspace(conn, id, workspace).is_ok() {
                    moved += 1;
                }
            }
            Ok(moved)
        })
    }

    fn get_stats(&self) -> Result<StorageStats> {
        self.storage.with_connection(queries::get_stats)
    }

    fn health_check(&self) -> Result<HealthStatus> {
        health_check_storage(&self.storage)
    }

    fn optimize(&self) -> Result<()> {
        self.storage.vacuum()?;
        self.storage.checkpoint()?;
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "sqlite"
    }

    fn schema_version(&self) -> Result<i32> {
        self.storage.with_connection(|conn| {
            let version: i32 = conn
                .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            Ok(version)
        })
    }
}

impl TransactionalBackend for SqliteBackend {
    fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<T>,
    {
        // Note: This is where we would ideally pass a transaction-aware
        // backend wrapper. For now, since SQLite doesn't support nested
        // transactions easily without savepoints (which we are adding),
        // we just execute the closure.
        // The closure expects &dyn StorageBackend, so we pass self.
        f(self)
    }

    fn savepoint(&self, name: &str) -> Result<()> {
        self.storage.with_connection(|conn| {
            conn.execute(&format!("SAVEPOINT {}", name), [])?;
            Ok(())
        })
    }

    fn release_savepoint(&self, name: &str) -> Result<()> {
        self.storage.with_connection(|conn| {
            conn.execute(&format!("RELEASE SAVEPOINT {}", name), [])?;
            Ok(())
        })
    }

    fn rollback_to_savepoint(&self, name: &str) -> Result<()> {
        self.storage.with_connection(|conn| {
            conn.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), [])?;
            Ok(())
        })
    }
}

impl CloudSyncBackend for SqliteBackend {
    fn push(&self) -> Result<SyncResult> {
        // Placeholder - actual cloud sync is handled by the sync module
        Ok(SyncResult {
            success: true,
            pushed_count: 0,
            pulled_count: 0,
            conflicts_resolved: 0,
            error: None,
            new_version: 0,
        })
    }

    fn pull(&self) -> Result<SyncResult> {
        // Placeholder - actual cloud sync is handled by the sync module
        Ok(SyncResult {
            success: true,
            pushed_count: 0,
            pulled_count: 0,
            conflicts_resolved: 0,
            error: None,
            new_version: 0,
        })
    }

    fn sync_delta(&self, since_version: u64) -> Result<SyncDelta> {
        self.storage.with_connection(|conn| {
            let delta = get_sync_delta(conn, since_version as i64)?;
            Ok(SyncDelta {
                created: delta.created,
                updated: delta.updated,
                deleted: delta.deleted,
                version: delta.to_version as u64,
            })
        })
    }

    fn sync_state(&self) -> Result<SyncState> {
        self.storage.with_connection(|conn| {
            let version = get_sync_version(conn)?;
            let (last_sync, pending_changes): (Option<String>, i64) = conn
                .query_row(
                    "SELECT last_sync, pending_changes FROM sync_state WHERE id = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap_or((None, 0));

            let last_sync = last_sync.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            });

            Ok(SyncState {
                local_version: version.version as u64,
                remote_version: None,
                last_sync,
                has_pending_changes: pending_changes > 0,
                pending_count: pending_changes as usize,
            })
        })
    }

    fn force_sync(&self) -> Result<SyncResult> {
        // Push then pull
        self.push()?;
        self.pull()
    }
}

fn sqlite_derived_index_health(conn: &rusqlite::Connection) -> Result<Vec<DerivedIndexHealth>> {
    Ok(vec![
        sqlite_embedding_health(conn)?,
        sqlite_fts_health(conn)?,
        sqlite_graph_health(conn)?,
    ])
}

fn sqlite_embedding_health(conn: &rusqlite::Connection) -> Result<DerivedIndexHealth> {
    let queue = get_embedding_queue_health(
        conn,
        DEFAULT_STALE_PROCESSING_AFTER,
        DEFAULT_MAX_EMBEDDING_RETRIES,
    )?;

    let live_memories = count_i64(conn, "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL")?;
    let indexed = count_i64(
        conn,
        "SELECT COUNT(*) FROM embeddings e
         JOIN memories m ON m.id = e.memory_id
         WHERE m.valid_to IS NULL",
    )?;
    let flagged_without_row = count_i64(
        conn,
        "SELECT COUNT(*) FROM memories m
         LEFT JOIN embeddings e ON e.memory_id = m.id
         WHERE m.valid_to IS NULL AND m.has_embedding = 1 AND e.memory_id IS NULL",
    )?;
    let row_without_flag = count_i64(
        conn,
        "SELECT COUNT(*) FROM embeddings e
         JOIN memories m ON m.id = e.memory_id
         WHERE m.valid_to IS NULL AND m.has_embedding = 0",
    )?;
    let orphaned = count_i64(
        conn,
        "SELECT COUNT(*) FROM embeddings e
         LEFT JOIN memories m ON m.id = e.memory_id
         WHERE m.id IS NULL OR m.valid_to IS NOT NULL",
    )?;
    let (embedding_profile_rows, embedding_profile_bytes_total, embedding_profile_bytes_avg) = conn
        .query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(LENGTH(embedding)), 0),
                COALESCE(CAST(AVG(LENGTH(embedding)) AS INTEGER), 0)
             FROM embeddings",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )?;
    let (embedding_profile_bytes_min, embedding_profile_bytes_max) = conn.query_row(
        "SELECT
            COALESCE(MIN(LENGTH(embedding)), 0),
            COALESCE(MAX(LENGTH(embedding)), 0)
         FROM embeddings",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )?;

    let status = if queue.stale_processing > 0
        || queue.failed > 0
        || flagged_without_row > 0
        || row_without_flag > 0
        || orphaned > 0
    {
        DerivedIndexStatus::Degraded
    } else if queue.pending > 0 || queue.processing > 0 {
        DerivedIndexStatus::Backlogged
    } else {
        DerivedIndexStatus::Healthy
    };

    let oldest_pending_age = match queue.oldest_pending_seconds {
        Some(age) => age.to_string(),
        None => "none".to_string(),
    };

    Ok(DerivedIndexHealth {
        name: "embeddings".to_string(),
        kind: DerivedIndexKind::Embedding,
        status,
        source_count: live_memories,
        indexed_count: indexed,
        pending_count: queue.pending + queue.processing,
        stale_count: queue.stale_processing,
        failed_count: queue.failed,
        orphaned_count: orphaned,
        details: HashMap::from([
            ("pending".to_string(), queue.pending.to_string()),
            ("processing".to_string(), queue.processing.to_string()),
            (
                "stale_processing".to_string(),
                queue.stale_processing.to_string(),
            ),
            ("failed".to_string(), queue.failed.to_string()),
            (
                "zero_retry_failed".to_string(),
                queue.zero_retry_failed.to_string(),
            ),
            (
                "retryable_failed".to_string(),
                queue.retryable_failed.to_string(),
            ),
            (
                "exhausted_failed".to_string(),
                queue.exhausted_failed.to_string(),
            ),
            (
                "max_retry_count".to_string(),
                queue.max_retry_count.to_string(),
            ),
            ("oldest_pending_age".to_string(), oldest_pending_age.clone()),
            ("oldest_pending_age_seconds".to_string(), oldest_pending_age),
            (
                "oldest_processing_age".to_string(),
                queue
                    .oldest_processing_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "oldest_processing_age_seconds".to_string(),
                queue
                    .oldest_processing_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "oldest_failed_age".to_string(),
                queue
                    .oldest_failed_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "oldest_failed_age_seconds".to_string(),
                queue
                    .oldest_failed_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            ("retry_count_0".to_string(), queue.retry_count_0.to_string()),
            ("retry_count_1".to_string(), queue.retry_count_1.to_string()),
            ("retry_count_2".to_string(), queue.retry_count_2.to_string()),
            (
                "retry_count_3_plus".to_string(),
                queue.retry_count_3_plus.to_string(),
            ),
            (
                "embedding_profile_rows".to_string(),
                embedding_profile_rows.to_string(),
            ),
            (
                "embedding_profile_bytes_total".to_string(),
                embedding_profile_bytes_total.to_string(),
            ),
            (
                "embedding_profile_bytes_avg".to_string(),
                embedding_profile_bytes_avg.to_string(),
            ),
            (
                "embedding_profile_bytes_min".to_string(),
                embedding_profile_bytes_min.to_string(),
            ),
            (
                "embedding_profile_bytes_max".to_string(),
                embedding_profile_bytes_max.to_string(),
            ),
            (
                "flagged_without_embedding_row".to_string(),
                flagged_without_row.to_string(),
            ),
            (
                "embedding_row_without_flag".to_string(),
                row_without_flag.to_string(),
            ),
        ]),
    })
}

fn sqlite_fts_health(conn: &rusqlite::Connection) -> Result<DerivedIndexHealth> {
    let source_count = count_i64(conn, "SELECT COUNT(*) FROM memories")?;
    let (rowid_source, rowid_column) = if sqlite_table_exists(conn, "memories_fts_docsize")? {
        ("memories_fts_docsize", "id")
    } else {
        ("memories_fts", "rowid")
    };
    let indexed_count = count_i64(conn, &format!("SELECT COUNT(*) FROM {rowid_source}"))?;
    let missing = count_i64(
        conn,
        &format!(
            "SELECT COUNT(*) FROM memories m
             WHERE m.id NOT IN (SELECT {rowid_column} FROM {rowid_source})"
        ),
    )?;
    let orphaned = count_i64(
        conn,
        &format!(
            "SELECT COUNT(*) FROM {rowid_source}
             WHERE {rowid_column} NOT IN (SELECT id FROM memories)"
        ),
    )?;
    let status = if missing > 0 || orphaned > 0 {
        DerivedIndexStatus::Degraded
    } else {
        DerivedIndexStatus::Healthy
    };

    Ok(DerivedIndexHealth {
        name: "memories_fts".to_string(),
        kind: DerivedIndexKind::FullText,
        status,
        source_count,
        indexed_count,
        pending_count: 0,
        stale_count: missing,
        failed_count: 0,
        orphaned_count: orphaned,
        details: HashMap::from([
            ("missing_rows".to_string(), missing.to_string()),
            ("drift_rows".to_string(), missing.to_string()),
        ]),
    })
}

fn sqlite_graph_health(conn: &rusqlite::Connection) -> Result<DerivedIndexHealth> {
    let source_count = count_i64(conn, "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL")?;
    let indexed_count = count_i64(
        conn,
        "SELECT COUNT(*) FROM crossrefs WHERE valid_to IS NULL",
    )?;
    let orphaned = count_i64(
        conn,
        "SELECT COUNT(*) FROM crossrefs c
         LEFT JOIN memories mf ON mf.id = c.from_id
         LEFT JOIN memories mt ON mt.id = c.to_id
         WHERE c.valid_to IS NULL
           AND (mf.id IS NULL OR mt.id IS NULL OR mf.valid_to IS NOT NULL OR mt.valid_to IS NOT NULL)",
    )?;
    let status = if orphaned > 0 {
        DerivedIndexStatus::Degraded
    } else {
        DerivedIndexStatus::Healthy
    };

    Ok(DerivedIndexHealth {
        name: "crossrefs".to_string(),
        kind: DerivedIndexKind::Graph,
        status,
        source_count,
        indexed_count,
        pending_count: 0,
        stale_count: 0,
        failed_count: 0,
        orphaned_count: orphaned,
        details: HashMap::new(),
    })
}

fn count_i64(conn: &rusqlite::Connection, sql: &str) -> Result<i64> {
    Ok(conn.query_row(sql, [], |row| row.get(0))?)
}

fn sqlite_table_exists(conn: &rusqlite::Connection, table_name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        [table_name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MemoryScope, MemoryTier, MemoryType};
    use rusqlite::params;

    fn test_memory_input(content: &str) -> CreateMemoryInput {
        CreateMemoryInput {
            content: content.to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["health".to_string()],
            metadata: HashMap::new(),
            importance: Some(0.5),
            scope: MemoryScope::Global,
            workspace: Some("default".to_string()),
            tier: MemoryTier::Permanent,
            defer_embedding: true,
            ttl_seconds: None,
            dedup_mode: crate::types::DedupMode::Allow,
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        }
    }

    #[test]
    fn test_create_in_memory() {
        let backend = SqliteBackend::in_memory().unwrap();
        assert_eq!(backend.backend_name(), "sqlite");
    }

    #[test]
    fn test_health_check() {
        let backend = SqliteBackend::in_memory().unwrap();
        let health = backend.health_check().unwrap();
        assert!(health.healthy, "health check failed: {:?}", health.error);
        assert!(health.latency_ms >= 0.0);
    }

    #[test]
    fn test_health_check_reports_derived_index_contract() {
        let backend = SqliteBackend::in_memory().unwrap();
        backend
            .create_memory(CreateMemoryInput {
                content: "contract health memory".to_string(),
                memory_type: MemoryType::Note,
                tags: vec!["health".to_string()],
                metadata: HashMap::new(),
                importance: Some(0.5),
                scope: MemoryScope::Global,
                workspace: Some("default".to_string()),
                tier: MemoryTier::Permanent,
                defer_embedding: false,
                ttl_seconds: None,
                dedup_mode: crate::types::DedupMode::Allow,
                dedup_threshold: None,
                event_time: None,
                event_duration_seconds: None,
                trigger_pattern: None,
                summary_of_id: None,
                media_url: None,
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        assert!(health.healthy, "health check failed: {:?}", health.error);

        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embeddings health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Backlogged);
        assert_eq!(embeddings.pending_count, 1);

        let fts = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "memories_fts")
            .expect("fts health");
        assert_eq!(fts.kind, DerivedIndexKind::FullText);
        assert_eq!(fts.status, DerivedIndexStatus::Healthy);

        let graph = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "crossrefs")
            .expect("graph health");
        assert_eq!(graph.kind, DerivedIndexKind::Graph);
        assert_eq!(graph.status, DerivedIndexStatus::Healthy);
    }

    #[test]
    fn test_health_check_reports_fts_degraded_when_rows_missing() {
        let backend = SqliteBackend::in_memory().unwrap();
        backend
            .create_memory(test_memory_input("fts-1 missing row"))
            .unwrap();
        backend
            .create_memory(test_memory_input("fts-2 missing row"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                // Remove all indexed rows to make FTS source-index drift visible.
                conn.execute(
                    "INSERT INTO memories_fts(memories_fts) VALUES('delete-all')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let fts = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "memories_fts")
            .expect("fts health");
        assert_eq!(fts.kind, DerivedIndexKind::FullText);
        assert_eq!(fts.status, DerivedIndexStatus::Degraded);
        assert_eq!(fts.stale_count, 2);
    }

    #[test]
    fn test_health_check_reports_graph_degraded_for_orphaned_crossrefs() {
        let backend = SqliteBackend::in_memory().unwrap();
        let source = backend
            .create_memory(test_memory_input("crossref source"))
            .unwrap();
        let target = backend
            .create_memory(test_memory_input("crossref target"))
            .unwrap();

        backend
            .create_crossref(source.id, target.id, EdgeType::RelatedTo, 0.8)
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "UPDATE memories SET valid_to = ? WHERE id = ?",
                    params![chrono::Utc::now().to_rfc3339(), source.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let graph = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "crossrefs")
            .expect("graph health");
        assert_eq!(graph.kind, DerivedIndexKind::Graph);
        assert_eq!(graph.status, DerivedIndexStatus::Degraded);
        assert_eq!(graph.orphaned_count, 1);
    }

    #[test]
    fn test_health_check_reports_embedding_degraded_for_failed_queue_rows() {
        let backend = SqliteBackend::in_memory().unwrap();
        let memory = backend
            .create_memory(test_memory_input("failed queue row"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO embedding_queue (memory_id, status, queued_at, retry_count)
                     VALUES (?, 'failed', datetime('now'), 0)",
                    params![memory.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.failed_count, 1);
    }

    #[test]
    fn test_health_check_reports_embedding_degraded_for_stale_queue_rows() {
        let backend = SqliteBackend::in_memory().unwrap();
        let memory = backend
            .create_memory(test_memory_input("stale queue row"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO embedding_queue (memory_id, status, queued_at, started_at, retry_count)
                     VALUES (?, 'processing', datetime('now'), datetime('now','-1 hour'), 0)",
                    params![memory.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.stale_count, 1);
    }

    #[test]
    fn test_health_check_embedding_details_include_queue_state_counters() {
        let backend = SqliteBackend::in_memory().unwrap();
        let pending = backend
            .create_memory(test_memory_input("state counter pending"))
            .unwrap();
        let processing = backend
            .create_memory(test_memory_input("state counter processing"))
            .unwrap();
        let retryable_failed = backend
            .create_memory(test_memory_input("state counter retryable failed"))
            .unwrap();
        let exhausted_failed = backend
            .create_memory(test_memory_input("state counter exhausted failed"))
            .unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        backend
            .storage()
            .with_connection(|conn| {
                let stale_started = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
                let old_pending = (chrono::Utc::now() - chrono::Duration::minutes(15)).to_rfc3339();

                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at)
                     VALUES (?, 'pending', ?)",
                    params![pending.id, old_pending],
                )?;
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at, started_at, retry_count)
                     VALUES (?, 'processing', ?, ?, 0)",
                    params![processing.id, now, stale_started],
                )?;
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at, retry_count)
                     VALUES (?, 'failed', ?, 1)",
                    params![retryable_failed.id, now],
                )?;
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at, retry_count)
                     VALUES (?, 'failed', ?, 4)",
                    params![exhausted_failed.id, now],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");

        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.details["pending"], "1");
        assert_eq!(embeddings.details["processing"], "1");
        assert_eq!(embeddings.details["stale_processing"], "1");
        assert_eq!(embeddings.details["failed"], "2");
        assert_eq!(embeddings.details["retryable_failed"], "1");
        assert_eq!(embeddings.details["exhausted_failed"], "1");
        assert_eq!(embeddings.details["max_retry_count"], "4");
        assert_ne!(embeddings.details["oldest_pending_age"], "none");
        assert_ne!(embeddings.details["oldest_pending_age_seconds"], "none");
    }

    #[test]
    fn test_health_check_reports_embedding_degraded_for_flag_mismatch() {
        let backend = SqliteBackend::in_memory().unwrap();
        let memory = backend
            .create_memory(test_memory_input("flag mismatch"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                // Mark as embedded without an embeddings row.
                conn.execute(
                    "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                    params![memory.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.pending_count, 0);
        assert_eq!(embeddings.indexed_count, 0);
        assert_eq!(embeddings.stale_count, 0);
        assert_eq!(embeddings.orphaned_count, 0);
    }

    #[test]
    fn test_get_stats() {
        let backend = SqliteBackend::in_memory().unwrap();
        let stats = backend.get_stats().unwrap();
        assert_eq!(stats.total_memories, 0);
        assert!(stats.storage_mode.starts_with("sqlite"));
    }

    #[test]
    fn test_crud_operations() {
        let backend = SqliteBackend::in_memory().unwrap();

        // Create
        let input = CreateMemoryInput {
            content: "Test memory".to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
            importance: Some(0.5),
            scope: MemoryScope::Global,
            workspace: Some("default".to_string()),
            tier: MemoryTier::Permanent,
            defer_embedding: true,
            ttl_seconds: None,
            dedup_mode: crate::types::DedupMode::Allow,
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        };

        let memory = backend.create_memory(input).unwrap();
        assert_eq!(memory.content, "Test memory");
        assert_eq!(memory.memory_type, MemoryType::Note);

        // Read
        let retrieved = backend.get_memory(memory.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, memory.id);

        // Update
        let update_input = UpdateMemoryInput {
            content: Some("Updated memory".to_string()),
            memory_type: None,
            tags: None,
            metadata: None,
            importance: None,
            scope: None,
            ttl_seconds: None,
            event_time: None,
            trigger_pattern: None,
            media_url: None,
        };
        let updated = backend.update_memory(memory.id, update_input).unwrap();
        assert_eq!(updated.content, "Updated memory");

        // Delete
        backend.delete_memory(memory.id).unwrap();
        let deleted = backend.get_memory(memory.id).unwrap();
        assert!(deleted.is_none());
    }
}
