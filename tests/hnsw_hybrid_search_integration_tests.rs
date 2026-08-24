//! Integration tests for HNSW Vector Acceleration Tier 2 (Hybrid Search & Storage Integration).

use std::sync::Arc;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::search::{
    hybrid_search, hybrid_search_with_hnsw, warmup_hnsw_from_db, AdaptiveCacheConfig, FuzzyEngine,
    HnswConfig, HnswIndex, SearchConfig, SearchResultCache, VectorMetric,
};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{
    CreateMemoryInput, EmbeddingConfig, MemoryType, SearchOptions, SearchStrategy,
};
use parking_lot::{Mutex, RwLock};
use serde_json::json;

fn create_test_context() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory sqlite storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    let hnsw_index = Arc::new(RwLock::new(HnswIndex::new(HnswConfig::new(
        embedder.dimensions(),
        VectorMetric::Cosine,
    ))));

    HandlerContext {
        storage,
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index,
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        progress_reporter: None,
        principal: None,
    }
}

#[test]
fn test_hnsw_warmup_from_sqlite() {
    let ctx = create_test_context();

    // Create several memories in SQLite
    let mem1 = ctx
        .storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Rust concurrency patterns with Arc and RwLock".to_string(),
                    memory_type: MemoryType::Note,
                    workspace: Some("default".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create mem1");

    let mem2 = ctx
        .storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Distributed consensus with Raft algorithm".to_string(),
                    memory_type: MemoryType::Note,
                    workspace: Some("default".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create mem2");

    // Manually store embeddings in SQLite embeddings table
    let v1 = ctx.embedder.embed(&mem1.content).expect("embed 1");
    let v2 = ctx.embedder.embed(&mem2.content).expect("embed 2");

    ctx.storage.with_connection(|conn| {
        for (id, v) in [(mem1.id, &v1), (mem2.id, &v2)] {
            let mut bytes = Vec::with_capacity(v.len() * 4);
            for f in v {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            conn.execute(
                "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                 VALUES (?1, ?2, 'tfidf', ?3, datetime('now'))",
                rusqlite::params![id, bytes, v.len()],
            )?;
        }
        Ok(())
    }).expect("insert embeddings");

    // Create a fresh HNSW index and warm it up
    let mut fresh_hnsw = HnswIndex::new(HnswConfig::new(
        ctx.embedder.dimensions(),
        VectorMetric::Cosine,
    ));
    assert_eq!(fresh_hnsw.len(), 0);

    let count = ctx
        .storage
        .with_connection(|conn| warmup_hnsw_from_db(conn, &mut fresh_hnsw))
        .expect("warmup");

    assert_eq!(count, 2);
    assert_eq!(fresh_hnsw.len(), 2);

    // Search fresh HNSW
    let results = fresh_hnsw.search(&v1, 5, None);
    assert!(!results.is_empty());
    assert_eq!(results[0].id, mem1.id);
    assert!(results[0].similarity > 0.99);
}

#[test]
fn test_live_memory_create_indexes_into_hnsw() {
    let ctx = create_test_context();

    let res = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Real-time stream processing with Apache Flink and Kafka",
            "workspace": "streaming-prod",
            "memory_type": "decision"
        }),
    );

    let id = res.get("id").and_then(|v| v.as_i64()).expect("memory id");
    assert!(id > 0);

    // Verify HNSW index contains the memory ID
    let hnsw = ctx.hnsw_index.read();
    assert_eq!(hnsw.len(), 1);

    // Query for similar content
    let q_emb = ctx
        .embedder
        .embed("stream processing Flink")
        .expect("embed query");
    let candidates = hnsw.search(&q_emb, 5, None);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, id);
    assert!(candidates[0].similarity > 0.0);
}

#[test]
fn test_hnsw_semantic_search_retrieval() {
    let ctx = create_test_context();

    // Ingest 3 distinct memories
    let res1 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Postgres database connection pool tuning max_connections and shared_buffers",
            "workspace": "infra",
            "memory_type": "note"
        }),
    );
    let id1 = res1
        .get("id")
        .expect("valid id")
        .as_i64()
        .expect("valid i64");

    let res2 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Frontend UI component design with Tailwind CSS and React hooks",
            "workspace": "frontend",
            "memory_type": "note"
        }),
    );
    let _id2 = res2
        .get("id")
        .expect("valid id")
        .as_i64()
        .expect("valid i64");

    let res3 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Database performance optimization for Postgres SQL slow queries",
            "workspace": "infra",
            "memory_type": "learning"
        }),
    );
    let id3 = res3
        .get("id")
        .expect("valid id")
        .as_i64()
        .expect("valid i64");

    // Perform semantic search via MCP handler
    let search_res = dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "Postgres database connection tuning and performance",
            "strategy": "semantic"
        }),
    );

    let results = search_res.as_array().expect("results array");
    assert!(!results.is_empty());

    let top_ids: Vec<i64> = results
        .iter()
        .filter_map(|r| {
            r.get("memory")
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_i64())
                .or_else(|| r.get("id").and_then(|v| v.as_i64()))
        })
        .collect();

    // Check that Postgres related memories (id1, id3) are at the top
    assert!(top_ids.contains(&id1));
    assert!(top_ids.contains(&id3));
}

#[test]
fn test_hnsw_rrf_hybrid_search() {
    let ctx = create_test_context();

    // Create memories
    let _ = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Kubernetes pod autoscaling using HPA and custom Prometheus metrics",
            "workspace": "k8s-infra",
            "memory_type": "note"
        }),
    );

    let _ = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Observability dashboards with Grafana and Prometheus monitoring",
            "workspace": "k8s-infra",
            "memory_type": "note"
        }),
    );

    // Hybrid search
    let search_res = dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "Prometheus autoscaling metrics",
            "strategy": "hybrid"
        }),
    );

    let results = search_res.as_array().expect("hybrid search results");
    assert!(!results.is_empty());
    assert!(results[0].get("score").is_some());
}

#[test]
fn test_hnsw_workspace_and_tier_isolation() {
    let ctx = create_test_context();

    // Create memory in workspace A
    let res_a = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Security audit findings for tenant isolation policy",
            "workspace": "tenant-alpha",
            "tier": "permanent"
        }),
    );
    let id_a = res_a.get("id").unwrap().as_i64().unwrap();

    // Create memory in workspace B
    let res_b = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Security audit findings for tenant isolation policy",
            "workspace": "tenant-beta",
            "tier": "permanent"
        }),
    );
    let id_b = res_b.get("id").unwrap().as_i64().unwrap();

    // Search scoped to workspace A
    let search_a = dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "Security audit tenant isolation",
            "workspace": "tenant-alpha",
            "strategy": "semantic"
        }),
    );

    let results_a = search_a.as_array().unwrap();
    let retrieved_ids: Vec<i64> = results_a
        .iter()
        .filter_map(|r| {
            r.get("memory")
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_i64())
                .or_else(|| r.get("id").and_then(|v| v.as_i64()))
        })
        .collect();

    assert!(retrieved_ids.contains(&id_a));
    assert!(
        !retrieved_ids.contains(&id_b),
        "Tenant B memory must not leak into Tenant A search"
    );
}

#[test]
fn test_hnsw_graceful_fallback_when_disabled_or_empty() {
    let ctx = create_test_context();

    // Memory in SQLite
    let mem = ctx
        .storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Fallback testing content for empty index".to_string(),
                    workspace: Some("default".to_string()),
                    ..Default::default()
                },
            )
        })
        .unwrap();

    let emb = ctx.embedder.embed(&mem.content).unwrap();

    ctx.storage
        .with_connection(|conn| {
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in &emb {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            conn.execute(
            "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
             VALUES (?1, ?2, 'tfidf', ?3, datetime('now'))",
            rusqlite::params![mem.id, bytes, emb.len()],
        )?;
            conn.execute(
                "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                rusqlite::params![mem.id],
            )?;
            Ok(())
        })
        .unwrap();

    // Create config with HNSW disabled
    let config_disabled = SearchConfig {
        hnsw_enabled: false,
        ..Default::default()
    };

    let options = SearchOptions {
        strategy: Some(SearchStrategy::SemanticOnly),
        ..Default::default()
    };

    let results = ctx
        .storage
        .with_connection(|conn| {
            hybrid_search_with_hnsw(
                conn,
                "Fallback testing",
                Some(&emb),
                &options,
                &config_disabled,
                Some(&*ctx.hnsw_index.read()),
            )
        })
        .unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].memory.id, mem.id);

    // Also test legacy signature `hybrid_search`
    let legacy_results = ctx
        .storage
        .with_connection(|conn| {
            hybrid_search(
                conn,
                "Fallback testing",
                Some(&emb),
                &options,
                &SearchConfig::default(),
            )
        })
        .unwrap();

    assert!(!legacy_results.is_empty());
    assert_eq!(legacy_results[0].memory.id, mem.id);
}

#[test]
fn test_hnsw_checkpoint_sqlite_persistence_and_restore() {
    let ctx = create_test_context();

    // Create 3 memories via MCP
    let res1 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Zero-copy serialization with flatbuffers and bincode",
            "workspace": "arch"
        }),
    );
    let id1 = res1.get("id").unwrap().as_i64().unwrap();

    let res2 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Adjacency list graph representation in memory",
            "workspace": "arch"
        }),
    );
    let id2 = res2.get("id").unwrap().as_i64().unwrap();

    // Save checkpoint into SQLite
    let ckpt_id = ctx
        .storage
        .with_connection(|conn| {
            engram::search::checkpoint_hnsw_to_db(conn, &ctx.hnsw_index.read(), "default")
        })
        .expect("checkpoint hnsw");

    assert!(ckpt_id > 0);

    // Verify record in SQLite
    let record = ctx
        .storage
        .with_connection(|conn| {
            engram::storage::queries::get_latest_hnsw_checkpoint(
                conn,
                "default",
                ctx.embedder.dimensions(),
            )
        })
        .expect("query checkpoint")
        .expect("checkpoint exists");

    assert_eq!(record.id, ckpt_id);
    assert_eq!(record.vector_count, 2);
    assert_eq!(record.dimensions, ctx.embedder.dimensions());

    // Create a fresh empty HnswIndex and restore via warmup_hnsw_from_db
    let mut restored_hnsw = HnswIndex::new(HnswConfig::new(
        ctx.embedder.dimensions(),
        VectorMetric::Cosine,
    ));
    assert_eq!(restored_hnsw.len(), 0);

    let count = ctx
        .storage
        .with_connection(|conn| engram::search::warmup_hnsw_from_db(conn, &mut restored_hnsw))
        .expect("warmup");

    assert_eq!(count, 2);
    assert_eq!(restored_hnsw.len(), 2);
    assert!(restored_hnsw.contains(&id1));
    assert!(restored_hnsw.contains(&id2));
}

#[test]
fn test_hnsw_incremental_warmup_after_checkpoint() {
    let ctx = create_test_context();

    // 1. Ingest initial memories
    let res1 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Consensus protocols Paxos and Raft",
            "workspace": "dist"
        }),
    );
    let id1 = res1.get("id").unwrap().as_i64().unwrap();

    let res2 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Byzantine fault tolerance and state machine replication",
            "workspace": "dist"
        }),
    );
    let id2 = res2.get("id").unwrap().as_i64().unwrap();

    // 2. Save checkpoint at T1
    ctx.storage
        .with_connection(|conn| {
            engram::search::checkpoint_hnsw_to_db(conn, &ctx.hnsw_index.read(), "default")
        })
        .expect("checkpoint 1");

    // Sleep briefly to ensure distinct timestamps
    std::thread::sleep(std::time::Duration::from_millis(50));

    // 3. Ingest 3rd memory at T2 > T1
    let res3 = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Vector similarity search and indexing with HNSW graphs",
            "workspace": "dist"
        }),
    );
    let id3 = res3.get("id").unwrap().as_i64().unwrap();

    // 4. Create fresh index and warm up
    let mut incremental_hnsw = HnswIndex::new(HnswConfig::new(
        ctx.embedder.dimensions(),
        VectorMetric::Cosine,
    ));

    let count = ctx
        .storage
        .with_connection(|conn| engram::search::warmup_hnsw_from_db(conn, &mut incremental_hnsw))
        .expect("warmup incremental");

    assert_eq!(count, 3);
    assert_eq!(incremental_hnsw.len(), 3);
    assert!(incremental_hnsw.contains(&id1));
    assert!(incremental_hnsw.contains(&id2));
    assert!(incremental_hnsw.contains(&id3));

    // Search 3rd memory
    let q = ctx.embedder.embed("vector similarity search HNSW").unwrap();
    let hits = incremental_hnsw.search(&q, 2, None);
    assert!(!hits.is_empty());
    assert_eq!(hits[0].id, id3);
}

#[test]
fn test_hnsw_checkpoint_pruning() {
    let ctx = create_test_context();

    // Save 5 checkpoints
    for _ in 0..5 {
        ctx.storage
            .with_connection(|conn| {
                engram::search::checkpoint_hnsw_to_db(conn, &ctx.hnsw_index.read(), "default")
            })
            .expect("checkpoint");
    }

    // Verify only the 2 most recent checkpoints are kept
    let total_checkpoints: i64 = ctx
        .storage
        .with_connection(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM hnsw_checkpoints WHERE model = 'default'",
                [],
                |r| r.get(0),
            )?;
            Ok(count)
        })
        .unwrap();

    assert_eq!(total_checkpoints, 2);
}
