#[cfg(feature = "dream-phase")]
#[test]
fn test_dream_phase_integration() {
    use engram::dream::{run_once_all, DreamConfig};
    use engram::storage::Storage;
    use rusqlite::params;

    let storage = Storage::open_in_memory().unwrap();

    // Insert memories in the past
    storage
        .with_connection(|conn| {
            for i in 0..3 {
                conn.execute(
                    "INSERT INTO memories (content, workspace, created_at, updated_at)
                 VALUES (?, 'default', datetime('now', '-2 days'), datetime('now', '-2 days'))",
                    params![format!(
                        "Rust programming language is safe and fast. Session {}.",
                        i
                    )],
                )?;
            }
            Ok(())
        })
        .unwrap();

    let mut test_config = DreamConfig::default();
    test_config.consolidation.min_age_hours = 24.0;

    // 2. Run Dream Phase
    let report = run_once_all(&storage, &test_config);

    // 3. Verify report
    assert_eq!(report.workspaces.len(), 1);
    assert_eq!(report.workspaces[0].groups_found, 1);
    assert_eq!(report.errors.len(), 0);

    // 4. Verify history table (Slice 5)
    storage
        .with_connection(|conn| {
            let count: i64 =
                conn.query_row("SELECT COUNT(*) FROM dream_runs", [], |row| row.get(0))?;
            assert_eq!(count, 1);
            Ok(())
        })
        .unwrap();

    // 5. Verify digest memory created (Slice 3)
    storage
        .with_connection(|conn| {
            let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE memory_type = 'summary' AND workspace = 'default'",
            [],
            |row| row.get(0)
        )?;
            assert!(count >= 1);
            Ok(())
        })
        .unwrap();

    // 6. Verify advisory lock (Slice 4)
    // Try to acquire lock with different owner - should fail
    let mut other_config = test_config.clone();
    other_config.owner_id = "other_owner".to_string();

    // Manually acquire lock
    storage
        .with_transaction(|conn| {
            engram::storage::queries::acquire_dream_lock(
                conn,
                "dream_phase_all",
                "manual_owner",
                3600,
            )
        })
        .unwrap();

    let report2 = run_once_all(&storage, &other_config);
    assert_eq!(report2.workspaces.len(), 0);
    assert_eq!(report2.errors.len(), 1);
    assert!(report2.errors[0].contains("lock"));
}

#[cfg(feature = "dream-phase")]
#[test]
fn test_mcp_tool_dream_run_now() {
    use engram::embedding::EmbeddingCache;
    use engram::mcp::handlers::{dispatch, HandlerContext};
    use engram::search::{FuzzyEngine, SearchConfig, SearchResultCache};
    use engram::storage::Storage;
    use parking_lot::Mutex;
    use serde_json::json;
    use std::sync::Arc;

    let storage = Storage::open_in_memory().unwrap();

    // Setup HandlerContext
    let ctx = HandlerContext {
        storage: storage.clone(),
        embedder: engram::embedding::create_embedder(&Default::default()).unwrap(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(Default::default())),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
    };

    let result = dispatch(&ctx, "dream_run_now", json!({}));

    assert_eq!(result.get("status").unwrap(), "success");
    assert!(result.get("report").is_some());
}

#[cfg(feature = "dream-phase")]
#[test]
fn test_mcp_dream_candidate_review_and_apply() {
    use engram::embedding::EmbeddingCache;
    use engram::mcp::handlers::{dispatch, HandlerContext};
    use engram::search::{FuzzyEngine, SearchConfig, SearchResultCache};
    use engram::storage::Storage;
    use parking_lot::Mutex;
    use rusqlite::params;
    use serde_json::json;
    use std::sync::Arc;

    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            for content in [
                "Release checklist requires local CI before merge.",
                "Huly owns issue metadata for implementation planning.",
            ] {
                conn.execute(
                    "INSERT INTO memories (content, workspace, importance, created_at, updated_at)
                     VALUES (?1, 'default', 0.8, datetime('now'), datetime('now'))",
                    params![content],
                )?;
            }
            Ok(())
        })
        .unwrap();

    let ctx = HandlerContext {
        storage: storage.clone(),
        embedder: engram::embedding::create_embedder(&Default::default()).unwrap(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(Default::default())),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
    };

    let create = dispatch(
        &ctx,
        "dream_create",
        json!({
            "job_id": "mcp-dream-job",
            "workspace": "default",
            "run": true,
            "max_candidates": 1
        }),
    );
    assert_eq!(create.get("status").unwrap(), "success");
    let candidate_id = create["report"]["candidate_ids"][0]
        .as_str()
        .expect("candidate id")
        .to_string();

    let listed = dispatch(
        &ctx,
        "dream_candidates_list",
        json!({"job_id": "mcp-dream-job", "review_state": "pending"}),
    );
    assert_eq!(listed["count"], 1);

    let reviewed = dispatch(
        &ctx,
        "dream_candidate_review",
        json!({"id": candidate_id, "review_state": "accepted"}),
    );
    assert_eq!(reviewed.get("status").unwrap(), "success");

    let dry_run = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": reviewed["candidate"]["id"], "dry_run": true}),
    );
    assert_eq!(dry_run.get("status").unwrap(), "dry_run");

    let before_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    let applied = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": reviewed["candidate"]["id"], "confirm": true}),
    );
    assert_eq!(applied.get("status").unwrap(), "completed");
    let after_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(after_count, before_count + 1);

    let applied_again = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": reviewed["candidate"]["id"], "confirm": true}),
    );
    assert_eq!(applied_again.get("status").unwrap(), "already_applied");
}
