use super::document::{
    build_memory_from_doc, build_memory_from_input, generate_memory_id, MeilisearchMemory,
};
use super::filters::{
    build_filter_from_list_options, build_filter_from_search_options, escape_filter_value,
    sort_to_meili,
};
use super::MEMORIES_INDEX;
use crate::error::EngramError;
use crate::storage::backend::{BatchCreateResult, BatchDeleteResult, HealthStatus, StorageBackend};
use crate::types::{
    CreateMemoryInput, CrossReference, EdgeType, ListOptions, MatchInfo, Memory, MemoryId,
    MemoryTier, SearchOptions, SearchResult, SearchStrategy, SortField, SortOrder, StorageStats,
    UpdateMemoryInput,
};

use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::SearchResults;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct MeilisearchBackend {
    pub(super) client: Client,
    pub(super) rt: Arc<Runtime>,
    url: String,
    api_key: Option<String>,
}

impl MeilisearchBackend {
    pub fn new(url: &str, api_key: Option<&str>) -> Result<Self, EngramError> {
        let client = Client::new(url, api_key)
            .map_err(|e| EngramError::Storage(format!("Failed to create client: {}", e)))?;

        let rt = Runtime::new().map_err(|e| EngramError::Storage(e.to_string()))?;

        let backend = Self {
            client,
            rt: Arc::new(rt),
            url: url.to_string(),
            api_key: api_key.map(|key| key.to_string()),
        };

        backend.init_schema()?;

        Ok(backend)
    }

    fn init_schema(&self) -> Result<(), EngramError> {
        self.rt.block_on(async {
            let index = self.client.index(MEMORIES_INDEX);
            // Ensure index exists
            let task = self.client.create_index(MEMORIES_INDEX, Some("id")).await;
            if let Ok(task) = task {
                let _ = self.client.wait_for_task(task, None, None).await;
            }

            // Configure filterable attributes
            let filterable_task = index
                .set_filterable_attributes(&[
                    "tags",
                    "memory_type",
                    "created_at",
                    "updated_at",
                    "importance",
                    "access_count",
                    "workspace",
                    "tier",
                    "scope",
                    "scope_id",
                    "visibility",
                    "lifecycle_state",
                ])
                .await;
            if let Ok(task) = filterable_task {
                let _ = index.wait_for_task(task, None, None).await;
            }

            // Configure sortable attributes
            let sortable_task = index
                .set_sortable_attributes(&[
                    "created_at",
                    "updated_at",
                    "importance",
                    "access_count",
                    "last_accessed_at",
                ])
                .await;
            if let Ok(task) = sortable_task {
                let _ = index.wait_for_task(task, None, None).await;
            }

            Ok(())
        })
    }

    pub fn index_memory(&self, memory: &Memory) -> Result<(), EngramError> {
        let doc = MeilisearchMemory::from(memory);
        self.rt.block_on(async {
            let task = self
                .client
                .index(MEMORIES_INDEX)
                .add_documents(&[doc], Some("id"))
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            self.client
                .index(MEMORIES_INDEX)
                .wait_for_task(task, None, None)
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn index_memories(&self, memories: &[Memory]) -> Result<(), EngramError> {
        if memories.is_empty() {
            return Ok(());
        }
        let docs: Vec<MeilisearchMemory> = memories.iter().map(MeilisearchMemory::from).collect();
        self.rt.block_on(async {
            let task = self
                .client
                .index(MEMORIES_INDEX)
                .add_documents(&docs, Some("id"))
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            self.client
                .index(MEMORIES_INDEX)
                .wait_for_task(task, None, None)
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    /// Get the configured Meilisearch URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Whether an API key is configured
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get index statistics from Meilisearch
    pub fn get_index_stats(&self) -> Result<serde_json::Value, EngramError> {
        self.rt.block_on(async {
            let stats = self
                .client
                .index(MEMORIES_INDEX)
                .get_stats()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            Ok(serde_json::json!({
                "number_of_documents": stats.number_of_documents,
                "is_indexing": stats.is_indexing,
            }))
        })
    }

    /// Get facet distribution for a field (used for tag/workspace listing)
    fn get_facet_distribution(
        &self,
        field: &str,
        filter: Option<&str>,
    ) -> Result<HashMap<String, usize>, EngramError> {
        self.rt.block_on(async {
            let index = self.client.index(MEMORIES_INDEX);
            let mut search = index.search();
            search.with_query("");
            search.with_limit(0);
            let facet_fields = [field];
            search.with_facets(meilisearch_sdk::search::Selectors::Some(&facet_fields));
            if let Some(f) = filter {
                search.with_filter(f);
            }

            let results: SearchResults<MeilisearchMemory> = search
                .execute()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            let distribution = results
                .facet_distribution
                .and_then(|fd| fd.get(field).cloned())
                .unwrap_or_default();

            Ok(distribution)
        })
    }
}

impl StorageBackend for MeilisearchBackend {
    fn create_memory(&self, input: CreateMemoryInput) -> Result<Memory, EngramError> {
        let id = generate_memory_id();
        let now = chrono::Utc::now();
        let memory = build_memory_from_input(id, input, now)?;

        self.index_memory(&memory)?;
        Ok(memory)
    }

    fn get_memory(&self, id: MemoryId) -> Result<Option<Memory>, EngramError> {
        self.rt.block_on(async {
            match self
                .client
                .index(MEMORIES_INDEX)
                .get_document::<MeilisearchMemory>(&id.to_string())
                .await
            {
                Ok(doc) => Ok(Some(build_memory_from_doc(doc))),
                Err(meilisearch_sdk::errors::Error::Meilisearch(e))
                    if e.error_code == meilisearch_sdk::errors::ErrorCode::DocumentNotFound =>
                {
                    Ok(None)
                }
                Err(e) => Err(EngramError::Storage(e.to_string())),
            }
        })
    }

    fn update_memory(&self, id: MemoryId, input: UpdateMemoryInput) -> Result<Memory, EngramError> {
        let mut memory = self.get_memory(id)?.ok_or(EngramError::NotFound(id))?;
        let mut changed = false;
        let now = chrono::Utc::now();

        if let Some(content) = input.content {
            memory.content = content;
            memory.content_hash =
                Some(crate::storage::queries::compute_dedup_hash(&memory.content));
            changed = true;
        }
        if let Some(memory_type) = input.memory_type {
            memory.memory_type = memory_type;
            changed = true;
        }
        if let Some(tags) = input.tags {
            memory.tags = tags;
            changed = true;
        }
        if let Some(metadata) = input.metadata {
            memory.metadata = metadata;
            changed = true;
        }
        if let Some(importance) = input.importance {
            memory.importance = importance;
            changed = true;
        }
        if let Some(scope) = input.scope {
            memory.scope = scope;
            changed = true;
        }
        if let Some(event_time) = input.event_time {
            memory.event_time = event_time;
            changed = true;
        }
        if let Some(trigger_pattern) = input.trigger_pattern {
            memory.trigger_pattern = trigger_pattern;
            changed = true;
        }
        if let Some(ttl) = input.ttl_seconds {
            if ttl <= 0 {
                if memory.tier == MemoryTier::Daily {
                    return Err(EngramError::InvalidInput(
                        "Cannot remove expiration from a Daily tier memory. Use promote_to_permanent first.".to_string(),
                    ));
                }
                memory.expires_at = None;
            } else {
                if memory.tier == MemoryTier::Permanent {
                    return Err(EngramError::InvalidInput(
                        "Cannot set expiration on a Permanent tier memory. Permanent memories cannot expire.".to_string(),
                    ));
                }
                memory.expires_at = Some(now + chrono::Duration::seconds(ttl));
            }
            changed = true;
        }

        if changed {
            memory.updated_at = now;
            memory.version += 1;
        }

        self.index_memory(&memory)?;
        Ok(memory)
    }

    fn delete_memory(&self, id: MemoryId) -> Result<(), EngramError> {
        self.rt.block_on(async {
            let task = self
                .client
                .index(MEMORIES_INDEX)
                .delete_document(&id.to_string())
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            self.client
                .index(MEMORIES_INDEX)
                .wait_for_task(task, None, None)
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    // --- Batch Operations ---

    fn create_memories_batch(
        &self,
        inputs: Vec<CreateMemoryInput>,
    ) -> Result<BatchCreateResult, EngramError> {
        let start = std::time::Instant::now();
        let mut created = Vec::new();
        let mut docs = Vec::new();
        let mut failed = Vec::new();
        let now = chrono::Utc::now();

        for (idx, input) in inputs.into_iter().enumerate() {
            let id = generate_memory_id();
            match build_memory_from_input(id, input, now) {
                Ok(memory) => {
                    created.push(memory.clone());
                    docs.push(MeilisearchMemory::from(&memory));
                }
                Err(e) => failed.push((idx, e.to_string())),
            }
        }

        if !docs.is_empty() {
            self.rt.block_on(async {
                let task = self
                    .client
                    .index(MEMORIES_INDEX)
                    .add_documents(&docs, Some("id"))
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                self.client
                    .index(MEMORIES_INDEX)
                    .wait_for_task(task, None, None)
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok::<(), EngramError>(())
            })?;
        }

        Ok(BatchCreateResult {
            created,
            failed,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    fn delete_memories_batch(&self, ids: Vec<MemoryId>) -> Result<BatchDeleteResult, EngramError> {
        self.rt.block_on(async {
            let task = self
                .client
                .index(MEMORIES_INDEX)
                .delete_documents(&ids)
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            self.client
                .index(MEMORIES_INDEX)
                .wait_for_task(task, None, None)
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))
        })?;

        Ok(BatchDeleteResult {
            deleted_count: ids.len(),
            not_found: vec![],
            failed: vec![],
        })
    }

    // --- Query Operations ---

    fn list_memories(&self, options: ListOptions) -> Result<Vec<Memory>, EngramError> {
        let filter = build_filter_from_list_options(&options)?;
        let sort = sort_to_meili(
            options.sort_by.unwrap_or(SortField::CreatedAt),
            options.sort_order.unwrap_or(SortOrder::Desc),
        );
        let sort_refs = vec![sort.as_str()];

        self.rt.block_on(async {
            let index = self.client.index(MEMORIES_INDEX);
            let mut search = index.search();
            search.with_query("");
            search.with_limit(options.limit.unwrap_or(50) as usize);
            search.with_sort(&sort_refs);
            if let Some(ref filter) = filter {
                search.with_filter(filter);
            }

            let results: SearchResults<MeilisearchMemory> = search
                .execute()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            Ok(results
                .hits
                .into_iter()
                .map(|hit| build_memory_from_doc(hit.result))
                .collect())
        })
    }

    fn count_memories(&self, _options: ListOptions) -> Result<i64, EngramError> {
        self.rt.block_on(async {
            let stats = self
                .client
                .index(MEMORIES_INDEX)
                .get_stats()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            Ok(stats.number_of_documents as i64)
        })
    }

    fn search_memories(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, EngramError> {
        self.rt.block_on(async {
            let index = self.client.index(MEMORIES_INDEX);
            let mut search = index.search();

            search.with_query(query);
            search.with_limit(options.limit.unwrap_or(50) as usize);

            let filter = build_filter_from_search_options(&options)?;
            if let Some(ref filter) = filter {
                search.with_filter(filter);
            }

            let results: SearchResults<MeilisearchMemory> = search
                .execute()
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;

            Ok(results
                .hits
                .into_iter()
                .map(|hit| {
                    let memory = build_memory_from_doc(hit.result);
                    let score = hit.ranking_score.unwrap_or(0.0) as f32;

                    SearchResult {
                        memory,
                        score,
                        match_info: MatchInfo {
                            strategy: SearchStrategy::KeywordOnly, // Meilisearch is primarily keyword/typo-tolerant
                            matched_terms: vec![], // Would need parsing of hit._formatted or similar
                            highlights: vec![],
                            semantic_score: None,
                            keyword_score: Some(score),
                        },
                    }
                })
                .collect())
        })
    }

    // --- Graph Operations (Not supported in plain Meilisearch) ---

    fn create_crossref(
        &self,
        _from_id: MemoryId,
        _to_id: MemoryId,
        _edge_type: EdgeType,
        _score: f32,
    ) -> Result<CrossReference, EngramError> {
        Err(EngramError::Storage(
            "Graph operations not supported in Meilisearch backend".to_string(),
        ))
    }

    fn get_crossrefs(&self, _memory_id: MemoryId) -> Result<Vec<CrossReference>, EngramError> {
        Ok(vec![])
    }

    fn delete_crossref(&self, _from_id: MemoryId, _to_id: MemoryId) -> Result<(), EngramError> {
        Ok(())
    }

    // --- Tag Operations ---

    fn list_tags(&self) -> Result<Vec<(String, i64)>, EngramError> {
        let distribution = self.get_facet_distribution("tags", None)?;
        let mut tags: Vec<(String, i64)> = distribution
            .into_iter()
            .map(|(tag, count)| (tag, count as i64))
            .collect();
        tags.sort_by_key(|b| std::cmp::Reverse(b.1));
        Ok(tags)
    }

    fn get_memories_by_tag(
        &self,
        tag: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>, EngramError> {
        let options = SearchOptions {
            limit: Some(limit.unwrap_or(50) as i64),
            ..Default::default()
        };
        self.search_memories(tag, options)
            .map(|results| results.into_iter().map(|r| r.memory).collect())
    }

    // --- Workspace Operations ---

    fn list_workspaces(&self) -> Result<Vec<(String, i64)>, EngramError> {
        let distribution = self.get_facet_distribution("workspace", None)?;
        let mut workspaces: Vec<(String, i64)> = distribution
            .into_iter()
            .map(|(ws, count)| (ws, count as i64))
            .collect();
        workspaces.sort_by_key(|b| std::cmp::Reverse(b.1));
        Ok(workspaces)
    }

    fn get_workspace_stats(&self, workspace: &str) -> Result<HashMap<String, i64>, EngramError> {
        let filter = format!("workspace = \"{}\"", escape_filter_value(workspace));
        let type_dist = self.get_facet_distribution("memory_type", Some(&filter))?;
        let mut stats: HashMap<String, i64> =
            type_dist.into_iter().map(|(k, v)| (k, v as i64)).collect();
        let total: i64 = stats.values().sum();
        stats.insert("total".to_string(), total);
        Ok(stats)
    }

    fn move_to_workspace(&self, ids: Vec<MemoryId>, workspace: &str) -> Result<usize, EngramError> {
        let mut moved = 0;
        for id in &ids {
            if let Some(mut memory) = self.get_memory(*id)? {
                memory.workspace = workspace.to_string();
                memory.updated_at = chrono::Utc::now();
                self.index_memory(&memory)?;
                moved += 1;
            }
        }
        Ok(moved)
    }

    // --- Maintenance & Metadata ---

    fn get_stats(&self) -> Result<StorageStats, EngramError> {
        let count = self.count_memories(ListOptions::default())?;
        Ok(StorageStats {
            total_memories: count,
            storage_mode: "meilisearch".to_string(),
            ..Default::default()
        })
    }

    fn health_check(&self) -> Result<HealthStatus, EngramError> {
        super::health::health_check(self)
    }

    fn optimize(&self) -> Result<(), EngramError> {
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "meilisearch"
    }

    fn schema_version(&self) -> Result<i32, EngramError> {
        Ok(1)
    }
}
