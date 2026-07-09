//! SQLite implementation of the StorageBackend trait (ENG-15)
//!
//! This module provides a SQLite-based storage backend that implements
//! the `StorageBackend` trait, allowing the existing SQLite storage
//! to be used through the abstracted interface.

use std::collections::HashMap;
use std::time::Instant;

use crate::error::{EngramError, Result};
use crate::types::{
    CreateCrossRefInput, CreateMemoryInput, CrossReference, EdgeType, ListOptions, Memory,
    MemoryId, SearchOptions, SearchResult, StorageConfig, UpdateMemoryInput, WorkspaceStats,
};

use super::backend::{
    validate_savepoint_name, BatchCreateResult, BatchDeleteResult, CloudSyncBackend, HealthStatus,
    StorageBackend, StorageStats, SyncDelta, SyncResult, SyncState, TransactionalBackend,
};
use super::connection::Storage;
use super::queries::{
    self, delete_memory_batch, get_related, get_sync_delta, get_sync_version, list_tags,
};
use crate::search::{hybrid_search, SearchConfig};

mod health;
pub use health::health_check_storage;

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
    fn with_transaction<F, T>(&self, _f: F) -> Result<T>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<T>,
    {
        Err(EngramError::Storage(
            "SqliteBackend::with_transaction is unsupported: callers should use real \
             SQLite transaction APIs until a transaction-scoped StorageBackend exists"
                .to_string(),
        ))
    }

    fn savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        self.storage.with_connection(|conn| {
            conn.execute(&format!("SAVEPOINT {}", name), [])?;
            Ok(())
        })
    }

    fn release_savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        self.storage.with_connection(|conn| {
            conn.execute(&format!("RELEASE SAVEPOINT {}", name), [])?;
            Ok(())
        })
    }

    fn rollback_to_savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        self.storage.with_connection(|conn| {
            conn.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), [])?;
            Ok(())
        })
    }
}

impl CloudSyncBackend for SqliteBackend {
    fn push(&self) -> Result<SyncResult> {
        Err(EngramError::Sync(
            "SQLite backend does not implement CloudSyncBackend::push; use the sync module"
                .to_string(),
        ))
    }

    fn pull(&self) -> Result<SyncResult> {
        Err(EngramError::Sync(
            "SQLite backend does not implement CloudSyncBackend::pull; use the sync module"
                .to_string(),
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EngramError;
    use crate::types::{MemoryScope, MemoryTier, MemoryType};

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

    #[test]
    fn test_with_transaction_reports_unsupported_wrapper_when_called() {
        let backend = SqliteBackend::in_memory().unwrap();

        let result = backend.with_transaction(|_| Ok(()));

        assert!(
            matches!(result, Err(EngramError::Storage(message)) if message.contains("transaction-scoped StorageBackend"))
        );
    }

    #[test]
    fn test_savepoint_rejects_invalid_names_before_sql() {
        let backend = SqliteBackend::in_memory().unwrap();

        for name in ["", "1bad", "bad-name", "bad name", "bad;DROP"] {
            let savepoint = backend.savepoint(name);
            let release = backend.release_savepoint(name);
            let rollback = backend.rollback_to_savepoint(name);

            assert!(
                matches!(savepoint, Err(EngramError::InvalidInput(message)) if message.contains("savepoint name"))
            );
            assert!(
                matches!(release, Err(EngramError::InvalidInput(message)) if message.contains("savepoint name"))
            );
            assert!(
                matches!(rollback, Err(EngramError::InvalidInput(message)) if message.contains("savepoint name"))
            );
        }
    }

    #[test]
    fn test_valid_savepoint_rollback_restores_temp_table_state() {
        let backend = SqliteBackend::in_memory().unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute("CREATE TEMP TABLE savepoint_probe (value TEXT)", [])?;
                Ok(())
            })
            .unwrap();

        backend.savepoint("sp_valid_1").unwrap();
        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO savepoint_probe (value) VALUES ('rolled-back')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        backend.rollback_to_savepoint("sp_valid_1").unwrap();
        backend.release_savepoint("sp_valid_1").unwrap();

        let count = backend
            .storage()
            .with_connection(|conn| {
                let count = conn.query_row("SELECT COUNT(*) FROM savepoint_probe", [], |row| {
                    row.get::<_, i64>(0)
                })?;
                Ok(count)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_push_and_pull_report_unsupported_sync() {
        let backend = SqliteBackend::in_memory().unwrap();

        let push = backend.push();
        let pull = backend.pull();

        assert!(
            matches!(push, Err(EngramError::Sync(message)) if message.contains("SQLite backend"))
        );
        assert!(
            matches!(pull, Err(EngramError::Sync(message)) if message.contains("SQLite backend"))
        );
    }
}
