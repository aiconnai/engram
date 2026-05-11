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
    };

    let result = dispatch(&ctx, "dream_run_now", json!({}));

    assert_eq!(result.get("status").unwrap(), "success");
    assert!(result.get("report").is_some());
}
