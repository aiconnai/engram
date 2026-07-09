use std::collections::HashMap;
use std::time::Instant;

use super::core::{TursoBackend, MEMORY_COLUMNS};
use crate::error::{EngramError, Result};
use crate::storage::backend::{
    validate_savepoint_name, BatchCreateResult, BatchDeleteResult, CloudSyncBackend,
    DerivedIndexHealth, DerivedIndexStatus, HealthStatus, StorageBackend, StorageStats, SyncDelta,
    SyncResult, SyncState, TransactionalBackend,
};
use crate::storage::queries::compute_dedup_hash;
use crate::types::{
    normalize_workspace, CreateMemoryInput, CrossReference, EdgeType, LifecycleState, ListOptions,
    MatchInfo, Memory, MemoryId, MemoryTier, RelationSource, SearchOptions, SearchResult,
    SearchStrategy, SortField, SortOrder, UpdateMemoryInput,
};
use chrono::{DateTime, Utc};

impl StorageBackend for TursoBackend {
    fn create_memory(&self, input: CreateMemoryInput) -> Result<Memory> {
        // Use tokio runtime to run async code in sync context
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
            let conn = self.conn.write().await;

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
                    LifecycleState::Active.to_string(),
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
            let mut memories = self
                .query_memories(&sql, vec![libsql::Value::Integer(id)])
                .await?;

            memories
                .pop()
                .ok_or_else(|| EngramError::NotFound(id))
        })
        })
    }

    fn create_memories_batch(&self, inputs: Vec<CreateMemoryInput>) -> Result<BatchCreateResult> {
        let start = Instant::now();
        let mut created = Vec::new();
        let mut failed = Vec::new();

        for (idx, input) in inputs.into_iter().enumerate() {
            match self.create_memory(input) {
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

    fn get_memory(&self, id: MemoryId) -> Result<Option<Memory>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let sql = format!(
                    "SELECT {} FROM memories WHERE id = ? AND valid_to IS NULL",
                    MEMORY_COLUMNS
                );
                let memories = self
                    .query_memories(&sql, vec![libsql::Value::Integer(id)])
                    .await?;

                Ok(memories.into_iter().next())
            })
        })
    }

    fn delete_memories_batch(&self, ids: Vec<MemoryId>) -> Result<BatchDeleteResult> {
        let mut deleted_count = 0;
        let mut not_found = Vec::new();
        let mut failed = Vec::new();

        for id in ids {
            match self.delete_memory(id) {
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

    fn update_memory(&self, id: MemoryId, input: UpdateMemoryInput) -> Result<Memory> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
            let conn = self.conn.write().await;
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
            let mut memories = self
                .query_memories(&sql, vec![libsql::Value::Integer(id)])
                .await?;
            memories.pop().ok_or_else(|| EngramError::NotFound(id))
        })
        })
    }

    fn delete_memory(&self, id: MemoryId) -> Result<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
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

    fn list_memories(&self, options: ListOptions) -> Result<Vec<Memory>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let mut sql = format!(
                    "SELECT {} FROM memories WHERE valid_to IS NULL",
                    MEMORY_COLUMNS
                );
                let mut params: Vec<libsql::Value> = Vec::new();

                if let Some(ref workspace) = options.workspace {
                    sql.push_str(" AND workspace = ?");
                    params.push(libsql::Value::Text(workspace.clone()));
                } else if let Some(ref workspaces) = options.workspaces {
                    if !workspaces.is_empty() {
                        let placeholders = vec!["?"; workspaces.len()].join(", ");
                        sql.push_str(&format!(" AND workspace IN ({})", placeholders));
                        for workspace in workspaces {
                            params.push(libsql::Value::Text(workspace.clone()));
                        }
                    }
                }

                if let Some(ref scope) = options.scope {
                    sql.push_str(" AND scope_type = ?");
                    params.push(libsql::Value::Text(scope.scope_type().to_string()));
                    if let Some(scope_id) = scope.scope_id() {
                        sql.push_str(" AND scope_id = ?");
                        params.push(libsql::Value::Text(scope_id.to_string()));
                    } else {
                        sql.push_str(" AND scope_id IS NULL");
                    }
                }

                if let Some(ref memory_type) = options.memory_type {
                    sql.push_str(" AND memory_type = ?");
                    params.push(libsql::Value::Text(memory_type.as_str().to_string()));
                }

                if let Some(ref tier) = options.tier {
                    sql.push_str(" AND tier = ?");
                    params.push(libsql::Value::Text(tier.as_str().to_string()));
                }

                if let Some(ref tags) = options.tags {
                    if !tags.is_empty() {
                        let placeholders = vec!["?"; tags.len()].join(", ");
                        sql.push_str(&format!(
                            " AND id IN (
                            SELECT mt.memory_id
                            FROM memory_tags mt
                            JOIN tags t ON t.id = mt.tag_id
                            WHERE t.name IN ({})
                            GROUP BY mt.memory_id
                            HAVING COUNT(DISTINCT t.name) = ?
                        )",
                            placeholders
                        ));
                        for tag in tags {
                            params.push(libsql::Value::Text(tag.clone()));
                        }
                        params.push(libsql::Value::Integer(tags.len() as i64));
                    }
                }

                if !options.include_archived {
                    sql.push_str(" AND (lifecycle_state IS NULL OR lifecycle_state != 'archived')");
                }

                let sort_field = options.sort_by.unwrap_or(SortField::CreatedAt);
                let sort_order = options.sort_order.unwrap_or(SortOrder::Desc);
                let sort_column = match sort_field {
                    SortField::CreatedAt => "created_at",
                    SortField::UpdatedAt => "updated_at",
                    SortField::LastAccessedAt => "last_accessed_at",
                    SortField::Importance => "importance",
                    SortField::AccessCount => "access_count",
                };
                let sort_dir = match sort_order {
                    SortOrder::Asc => "ASC",
                    SortOrder::Desc => "DESC",
                };
                sql.push_str(&format!(" ORDER BY {} {}", sort_column, sort_dir));

                if let Some(limit) = options.limit {
                    sql.push_str(&format!(" LIMIT {}", limit));
                }

                if let Some(offset) = options.offset {
                    sql.push_str(&format!(" OFFSET {}", offset));
                }

                self.query_memories(&sql, params).await
            })
        })
    }

    fn count_memories(&self, options: ListOptions) -> Result<i64> {
        let mut options = options;
        options.limit = None;
        options.offset = None;
        let memories = self.list_memories(options)?;
        Ok(memories.len() as i64)
    }

    fn search_memories(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                // Simple LIKE-based search (full hybrid search would need vector support)
                let mut sql = format!(
                    "SELECT {} FROM memories WHERE valid_to IS NULL AND content LIKE ?",
                    MEMORY_COLUMNS
                );
                let mut params = vec![libsql::Value::Text(format!("%{}%", query))];

                if !options.include_archived {
                    sql.push_str(" AND (lifecycle_state IS NULL OR lifecycle_state != 'archived')");
                }

                if let Some(ref workspace) = options.workspace {
                    sql.push_str(" AND workspace = ?");
                    params.push(libsql::Value::Text(workspace.clone()));
                } else if let Some(ref workspaces) = options.workspaces {
                    if !workspaces.is_empty() {
                        let placeholders = vec!["?"; workspaces.len()].join(", ");
                        sql.push_str(&format!(" AND workspace IN ({})", placeholders));
                        for workspace in workspaces {
                            params.push(libsql::Value::Text(workspace.clone()));
                        }
                    }
                }

                if let Some(ref scope) = options.scope {
                    sql.push_str(" AND scope_type = ?");
                    params.push(libsql::Value::Text(scope.scope_type().to_string()));
                    if let Some(scope_id) = scope.scope_id() {
                        sql.push_str(" AND scope_id = ?");
                        params.push(libsql::Value::Text(scope_id.to_string()));
                    } else {
                        sql.push_str(" AND scope_id IS NULL");
                    }
                }

                if let Some(ref memory_type) = options.memory_type {
                    sql.push_str(" AND memory_type = ?");
                    params.push(libsql::Value::Text(memory_type.as_str().to_string()));
                } else if !options.include_transcripts {
                    sql.push_str(" AND memory_type != 'transcript_chunk'");
                }

                if let Some(ref tier) = options.tier {
                    sql.push_str(" AND tier = ?");
                    params.push(libsql::Value::Text(tier.as_str().to_string()));
                }

                sql.push_str(" ORDER BY importance DESC");
                if let Some(limit) = options.limit {
                    sql.push_str(&format!(" LIMIT {}", limit));
                } else {
                    sql.push_str(" LIMIT 20");
                }

                let memories = self.query_memories(&sql, params).await?;

                Ok(memories
                    .into_iter()
                    .map(|memory| SearchResult {
                        memory,
                        score: 1.0,
                        match_info: MatchInfo {
                            strategy: SearchStrategy::KeywordOnly,
                            matched_terms: vec![query.to_string()],
                            highlights: Vec::new(),
                            semantic_score: None,
                            keyword_score: Some(1.0),
                        },
                    })
                    .collect())
            })
        })
    }

    fn create_crossref(
        &self,
        from_id: MemoryId,
        to_id: MemoryId,
        edge_type: EdgeType,
        score: f32,
    ) -> Result<CrossReference> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
            let conn = self.conn.write().await;
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

    fn get_crossrefs(&self, memory_id: MemoryId) -> Result<Vec<CrossReference>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.read().await;
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
                    let edge_type_str: String =
                        row.get(2).unwrap_or_else(|_| "related_to".to_string());
                    let source_str: String = row.get(6).unwrap_or_else(|_| "auto".to_string());
                    let created_at_str: String =
                        row.get(8).unwrap_or_else(|_| Utc::now().to_rfc3339());
                    let valid_from_str: String =
                        row.get(9).unwrap_or_else(|_| Utc::now().to_rfc3339());
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

    fn delete_crossref(&self, from_id: MemoryId, to_id: MemoryId) -> Result<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
            let conn = self.conn.write().await;
            let now = chrono::Utc::now().to_rfc3339();

            conn.execute(
                "UPDATE crossrefs SET valid_to = ? WHERE from_id = ? AND to_id = ? AND valid_to IS NULL",
                libsql::params![now, from_id, to_id],
            ).await.map_err(|e| EngramError::Storage(e.to_string()))?;

            Ok(())
        })
        })
    }

    fn list_tags(&self) -> Result<Vec<(String, i64)>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.read().await;
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

    fn get_memories_by_tag(&self, tag: &str, limit: Option<usize>) -> Result<Vec<Memory>> {
        self.list_memories(ListOptions {
            tags: Some(vec![tag.to_string()]),
            limit: limit.map(|l| l as i64),
            ..Default::default()
        })
    }

    fn list_workspaces(&self) -> Result<Vec<(String, i64)>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.read().await;
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

    fn get_workspace_stats(&self, workspace: &str) -> Result<HashMap<String, i64>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
            let conn = self.conn.read().await;

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

    fn move_to_workspace(&self, ids: Vec<MemoryId>, workspace: &str) -> Result<usize> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
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

    fn get_stats(&self) -> Result<StorageStats> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.read().await;

                let memory_count: i64 = conn
                    .query("SELECT COUNT(*) FROM memories WHERE valid_to IS NULL", ())
                    .await
                    .ok()
                    .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                    .map(|r| r.get(0).unwrap_or(0))
                    .unwrap_or(0);

                let crossref_count: i64 = conn
                    .query("SELECT COUNT(*) FROM crossrefs WHERE valid_to IS NULL", ())
                    .await
                    .ok()
                    .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                    .map(|r| r.get(0).unwrap_or(0))
                    .unwrap_or(0);

                let tag_count: i64 = conn
                    .query("SELECT COUNT(DISTINCT tag_id) FROM memory_tags", ())
                    .await
                    .ok()
                    .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                    .map(|r| r.get(0).unwrap_or(0))
                    .unwrap_or(0);

                let schema_version: i32 = conn
                    .query("SELECT COALESCE(MAX(version), 0) FROM schema_version", ())
                    .await
                    .ok()
                    .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                    .map(|r| r.get(0).unwrap_or(0))
                    .unwrap_or(0);

                Ok(StorageStats {
                    total_memories: memory_count,
                    total_tags: tag_count,
                    total_crossrefs: crossref_count,
                    total_versions: 0,
                    total_identities: 0,
                    total_entities: 0,
                    db_size_bytes: 0,
                    memories_with_embeddings: 0,
                    memories_pending_embedding: 0,
                    last_sync: None,
                    sync_pending: false,
                    storage_mode: "turso".to_string(),
                    schema_version,
                    workspaces: HashMap::new(),
                    type_counts: HashMap::new(),
                    tier_counts: HashMap::new(),
                })
            })
        })
    }

    fn health_check(&self) -> Result<HealthStatus> {
        let start = Instant::now();

        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        let result = tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.read().await;
                conn.query("SELECT 1", ()).await
            })
        });

        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(_) => Ok(HealthStatus {
                healthy: true,
                latency_ms,
                error: None,
                details: HashMap::from([
                    ("backend".to_string(), "turso".to_string()),
                    ("url".to_string(), self.config.url.clone()),
                ]),
                derived_indexes: self
                    .get_stats()
                    .ok()
                    .map(|stats| {
                        vec![DerivedIndexHealth::external(
                            "memories",
                            DerivedIndexStatus::Healthy,
                            stats.total_memories,
                            stats.total_memories,
                            HashMap::from([("index".to_string(), "memories".to_string())]),
                        )]
                    })
                    .unwrap_or_else(|| {
                        vec![DerivedIndexHealth::external(
                            "memories",
                            DerivedIndexStatus::Unavailable,
                            0,
                            0,
                            HashMap::from([
                                ("index".to_string(), "memories".to_string()),
                                (
                                    "error".to_string(),
                                    "failed to read index stats".to_string(),
                                ),
                            ]),
                        )]
                    }),
            }),
            Err(e) => Ok(HealthStatus {
                healthy: false,
                latency_ms,
                error: Some(e.to_string()),
                details: HashMap::from([
                    ("backend".to_string(), "turso".to_string()),
                    ("url".to_string(), self.config.url.clone()),
                ]),
                derived_indexes: vec![DerivedIndexHealth::external(
                    "memories",
                    DerivedIndexStatus::Unavailable,
                    0,
                    0,
                    HashMap::from([
                        ("index".to_string(), "memories".to_string()),
                        ("error".to_string(), e.to_string()),
                    ]),
                )],
            }),
        }
    }

    fn optimize(&self) -> Result<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute("VACUUM", ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }

    fn backend_name(&self) -> &'static str {
        "turso"
    }

    fn schema_version(&self) -> Result<i32> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.read().await;
                let version: i32 = conn
                    .query("SELECT COALESCE(MAX(version), 0) FROM schema_version", ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?
                    .next()
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.get(0).unwrap_or(0))
                    .unwrap_or(0);
                Ok(version)
            })
        })
    }
}

impl TransactionalBackend for TursoBackend {
    fn with_transaction<F, T>(&self, _f: F) -> Result<T>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<T>,
    {
        Err(EngramError::Storage(
            "TursoBackend::with_transaction is unsupported: callers should use real \
             Turso/libSQL transaction APIs until a transaction-scoped StorageBackend exists"
                .to_string(),
        ))
    }

    fn savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute(&format!("SAVEPOINT {}", name), ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }

    fn release_savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute(&format!("RELEASE SAVEPOINT {}", name), ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }

    fn rollback_to_savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }
}

impl CloudSyncBackend for TursoBackend {
    fn push(&self) -> Result<SyncResult> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;
        tokio::task::block_in_place(|| rt.block_on(self.sync()))
    }

    fn pull(&self) -> Result<SyncResult> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;
        tokio::task::block_in_place(|| rt.block_on(self.sync()))
    }

    fn sync_delta(&self, _since_version: u64) -> Result<SyncDelta> {
        Err(EngramError::Sync(
            "Turso backend does not expose CloudSyncBackend::sync_delta; embedded replica \
             synchronization is handled by sync()"
                .to_string(),
        ))
    }

    fn sync_state(&self) -> Result<SyncState> {
        Err(EngramError::Sync(
            "Turso backend does not expose CloudSyncBackend::sync_state; embedded replica \
             synchronization is handled by sync()"
                .to_string(),
        ))
    }

    fn force_sync(&self) -> Result<SyncResult> {
        self.push()
    }
}

// Turso tests moved to tests/turso_backend_tests.rs (integration test)
// to avoid libsql/rusqlite SQLite initialization conflict under --all-features.
// Run with: cargo test --test turso_backend_tests --features turso
