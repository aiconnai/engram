//! Integration tests for Autonomous Sleep-Time Dream & Memory Consolidation Worker (RFC 0008).

#[cfg(feature = "dream-phase")]
mod tests {
    use parking_lot::Mutex;
    use std::sync::Arc;

    use engram::dream::{DreamPipeline, DreamPipelineConfig};
    use engram::embedding::{create_embedder, EmbeddingCache};
    use engram::mcp::handlers::dream::{dream_consolidation_status, dream_insights, dream_run_now};
    use engram::mcp::handlers::HandlerContext;
    use engram::search::{FuzzyEngine, SearchConfig, SearchResultCache};
    use engram::storage::queries::{create_memory, get_memory, list_memories};
    use engram::storage::Storage;
    use engram::types::{CreateMemoryInput, LifecycleState, ListOptions, MemoryType};
    use serde_json::json;

    fn setup_test_context() -> (Storage, HandlerContext) {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&Default::default()).expect("embedder");
        let ctx = HandlerContext {
            storage: storage.clone(),
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(Default::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
                engram::search::HnswConfig::new(
                    embedder.dimensions(),
                    engram::search::VectorMetric::Cosine,
                ),
            ))),
            progress_reporter: None,
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        };
        (storage, ctx)
    }

    fn insert_embedding(conn: &rusqlite::Connection, memory_id: i64, vector: &[f32]) {
        let bytes: Vec<u8> = vector.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT INTO embeddings (memory_id, embedding, model, dimensions) VALUES (?1, ?2, 'test-model', ?3)",
            rusqlite::params![memory_id, bytes, vector.len() as i64],
        )
        .unwrap();
    }

    #[test]
    fn test_procedural_distillation_from_episodic_memories() {
        let (storage, _ctx) = setup_test_context();
        let workspace = "test_procedural";

        // 1. Create episodic task session memories
        let input1 = CreateMemoryInput {
            content: "Session Task: Fixed flaky database connection. Lesson: Always configure busy_timeout to at least 30000ms to avoid locking conflicts.".to_string(),
            memory_type: MemoryType::Episodic,
            workspace: Some(workspace.to_string()),
            tags: vec!["session".to_string(), "database".to_string(), "error".to_string()],
            importance: Some(0.7),
            ..Default::default()
        };
        let input2 = CreateMemoryInput {
            content: "Investigation: Out of memory on large files. Fix: Use stream chunks and take(64MB) to bound decompression buffers.".to_string(),
            memory_type: MemoryType::Episodic,
            workspace: Some(workspace.to_string()),
            tags: vec!["task".to_string(), "memory-leak".to_string()],
            importance: Some(0.75),
            ..Default::default()
        };

        let m1 = storage
            .with_transaction(|conn| create_memory(conn, &input1))
            .unwrap();
        let m2 = storage
            .with_transaction(|conn| create_memory(conn, &input2))
            .unwrap();

        // 2. Run dream consolidation pass
        let config = DreamPipelineConfig {
            enable_procedural_distillation: true,
            enable_deduplication: false,
            enable_graph_optimization: false,
            enable_thematic_digest: true,
            dry_run: false,
            ..Default::default()
        };

        let result = DreamPipeline::run_workspace(&storage, workspace, &config).unwrap();
        assert_eq!(result.episodic_scanned, 2);
        assert_eq!(result.procedural_rules_extracted, 2);
        assert!(result.digest_memory_id.is_some());

        // 3. Verify that distilled procedural memories exist and link back
        let all_memories = storage
            .with_connection(|conn| {
                list_memories(
                    conn,
                    &ListOptions {
                        workspace: Some(workspace.to_string()),
                        include_archived: false,
                        ..Default::default()
                    },
                )
            })
            .unwrap();

        let procedural = all_memories
            .iter()
            .filter(|m| m.memory_type == MemoryType::Procedural)
            .collect::<Vec<_>>();

        assert_eq!(procedural.len(), 2);
        assert!(procedural
            .iter()
            .any(|p| p.summary_of_id == Some(m1.id) && p.content.contains("busy_timeout")));
        assert!(procedural
            .iter()
            .any(|p| p.summary_of_id == Some(m2.id) && p.content.contains("take(64MB)")));
    }

    #[test]
    fn test_semantic_deduplication_and_archival() {
        let (storage, _ctx) = setup_test_context();
        let workspace = "test_dedup";

        // 1. Insert two nearly identical memories with matching embeddings
        let input1 = CreateMemoryInput {
            content: "API endpoint /v1/search requires Bearer token authentication in Authorization header.".to_string(),
            workspace: Some(workspace.to_string()),
            importance: Some(0.9),
            ..Default::default()
        };
        let input2 = CreateMemoryInput {
            content: "API endpoint /v1/search needs Bearer token authentication in the Authorization header.".to_string(),
            workspace: Some(workspace.to_string()),
            importance: Some(0.6),
            ..Default::default()
        };

        let m1 = storage
            .with_transaction(|conn| create_memory(conn, &input1))
            .unwrap();
        let m2 = storage
            .with_transaction(|conn| create_memory(conn, &input2))
            .unwrap();

        // Set high-similarity embeddings (e.g. 0.98 cosine similarity)
        let emb1 = vec![1.0, 0.0, 0.0, 0.0];
        let emb2 = vec![0.98, 0.02, 0.0, 0.0];

        storage
            .with_transaction(|conn| {
                insert_embedding(conn, m1.id, &emb1);
                insert_embedding(conn, m2.id, &emb2);
                Ok(())
            })
            .unwrap();

        // 2. Run dream consolidation pass with semantic deduplication
        let config = DreamPipelineConfig {
            semantic_dedup_threshold: 0.90,
            enable_procedural_distillation: false,
            enable_deduplication: true,
            enable_graph_optimization: false,
            enable_thematic_digest: false,
            dry_run: false,
            ..Default::default()
        };

        let result = DreamPipeline::run_workspace(&storage, workspace, &config).unwrap();
        assert_eq!(result.duplicates_found, 1);
        assert_eq!(result.duplicates_archived, 1);

        // 3. Verify that m2 (lower importance) is archived and m1 remains active
        let loaded1 = storage
            .with_connection(|conn| get_memory(conn, m1.id))
            .unwrap();
        let loaded2 = storage
            .with_connection(|conn| get_memory(conn, m2.id))
            .unwrap();

        assert_eq!(loaded1.lifecycle_state, LifecycleState::Active);
        assert_eq!(loaded2.lifecycle_state, LifecycleState::Archived);
        assert_eq!(loaded2.summary_of_id, Some(m1.id));
    }

    #[test]
    fn test_thematic_insights_and_mcp_handlers() {
        let (storage, ctx) = setup_test_context();
        let workspace = "test_mcp_dream";

        let input = CreateMemoryInput {
            content: "Rule: Always enforce tenant isolation on multi-agent queries.".to_string(),
            memory_type: MemoryType::Procedural,
            workspace: Some(workspace.to_string()),
            tags: vec!["security".to_string(), "tenant".to_string()],
            importance: Some(0.8),
            ..Default::default()
        };
        storage
            .with_transaction(|conn| create_memory(conn, &input))
            .unwrap();

        // 1. Trigger dream_run_now via MCP handler
        let run_res = dream_run_now(
            &ctx,
            json!({
                "workspace": workspace,
                "dry_run": false
            }),
        );
        assert_eq!(run_res["status"], "success");

        // 2. Query dream_consolidation_status via MCP handler
        let status_res = dream_consolidation_status(
            &ctx,
            json!({
                "workspace": workspace
            }),
        );
        assert_eq!(status_res["status"], "success");
        assert!(
            status_res["metrics"]["distilled_procedural_rules"]
                .as_i64()
                .unwrap()
                >= 1
        );
        assert!(status_res["metrics"]["thematic_digests"].as_i64().unwrap() >= 1);

        // 3. Query dream_insights via MCP handler
        let insights_res = dream_insights(
            &ctx,
            json!({
                "workspace": workspace
            }),
        );
        assert_eq!(insights_res["status"], "success");
        assert!(!insights_res["insights"]["digests"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(!insights_res["insights"]["procedural_rules"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
