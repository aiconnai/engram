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
fn test_handler_context(
    storage: engram::storage::Storage,
) -> engram::mcp::handlers::HandlerContext {
    use engram::embedding::EmbeddingCache;
    use engram::mcp::handlers::HandlerContext;
    use engram::search::{FuzzyEngine, SearchConfig, SearchResultCache};
    use parking_lot::Mutex;
    use std::sync::Arc;

    HandlerContext {
        storage,
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
    }
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

#[cfg(feature = "dream-phase")]
#[test]
fn test_mcp_apply_create_and_ignore_candidates_without_targets() {
    use engram::mcp::handlers::dispatch;
    use engram::storage::{
        create_dream_candidate, create_dream_job, NewDreamCandidate, NewDreamJob, Storage,
    };
    use serde_json::json;

    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            create_dream_job(
                conn,
                &NewDreamJob {
                    id: Some("manual-no-target-job"),
                    workspace: "default",
                    instructions: Some("manual candidate contract regression"),
                    model_profile: None,
                    input_summary: &json!({}),
                },
            )?;
            create_dream_candidate(
                conn,
                &NewDreamCandidate {
                    id: Some("manual-create-no-targets"),
                    job_id: "manual-no-target-job",
                    workspace: "default",
                    kind: "summary",
                    proposed_action: "create",
                    confidence: 0.8,
                    freshness_state: "current",
                    content_preview: "Manual summary candidate.",
                    proposed_content: Some("Manual summary candidate."),
                    reason_codes: &json!(["manual_regression"]),
                    policy_explanation: &json!({}),
                    metadata: &json!({}),
                },
            )?;
            create_dream_candidate(
                conn,
                &NewDreamCandidate {
                    id: Some("manual-ignore-no-targets"),
                    job_id: "manual-no-target-job",
                    workspace: "default",
                    kind: "stale_fact",
                    proposed_action: "ignore",
                    confidence: 0.5,
                    freshness_state: "unknown",
                    content_preview: "Reviewer intentionally closes this candidate.",
                    proposed_content: None,
                    reason_codes: &json!(["manual_regression"]),
                    policy_explanation: &json!({}),
                    metadata: &json!({}),
                },
            )?;
            Ok(())
        })
        .unwrap();

    let ctx = test_handler_context(storage.clone());
    for id in ["manual-create-no-targets", "manual-ignore-no-targets"] {
        let reviewed = dispatch(
            &ctx,
            "dream_candidate_review",
            json!({"id": id, "review_state": "accepted"}),
        );
        assert_eq!(reviewed.get("status").unwrap(), "success");
    }

    let create_dry_run = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "manual-create-no-targets", "dry_run": true}),
    );
    assert_eq!(create_dry_run.get("status").unwrap(), "dry_run");
    assert_eq!(
        create_dry_run["planned"]["target_memory_ids"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let before_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    let created = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "manual-create-no-targets", "confirm": true}),
    );
    assert_eq!(created.get("status").unwrap(), "completed");

    let after_create_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(after_create_count, before_count + 1);

    let ignore_dry_run = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "manual-ignore-no-targets", "dry_run": true}),
    );
    assert_eq!(ignore_dry_run.get("status").unwrap(), "dry_run");
    assert_eq!(
        ignore_dry_run["planned"]["target_memory_ids"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ignore_dry_run["planned"]["will_mutate_canonical_memory"],
        json!(false)
    );

    let ignored = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "manual-ignore-no-targets", "confirm": true}),
    );
    assert_eq!(ignored.get("status").unwrap(), "completed");
    let after_ignore_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(after_ignore_count, after_create_count);
}

#[cfg(feature = "dream-phase")]
#[test]
fn test_mcp_expire_candidate_does_not_apply_when_target_is_no_longer_active() {
    use engram::mcp::handlers::dispatch;
    use engram::storage::{
        create_dream_candidate, create_dream_job, get_dream_candidate, NewDreamCandidate,
        NewDreamJob, Storage,
    };
    use rusqlite::params;
    use serde_json::json;

    let storage = Storage::open_in_memory().unwrap();
    let target_id: i64 = storage
        .with_connection(|conn| {
            conn.execute(
                "INSERT INTO memories (content, workspace, importance, created_at, updated_at)
                 VALUES ('Temporary note to expire.', 'default', 0.4, datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .unwrap();

    storage
        .with_connection(|conn| {
            create_dream_job(
                conn,
                &NewDreamJob {
                    id: Some("expire-race-job"),
                    workspace: "default",
                    instructions: Some("expire race regression"),
                    model_profile: None,
                    input_summary: &json!({}),
                },
            )?;
            create_dream_candidate(
                conn,
                &NewDreamCandidate {
                    id: Some("expire-race-candidate"),
                    job_id: "expire-race-job",
                    workspace: "default",
                    kind: "stale_fact",
                    proposed_action: "expire",
                    confidence: 0.9,
                    freshness_state: "expired",
                    content_preview: "Temporary note to expire.",
                    proposed_content: None,
                    reason_codes: &json!(["expired_memory"]),
                    policy_explanation: &json!({}),
                    metadata: &json!({
                        "target_memory_ids": [target_id],
                        "expiration_reason": "regression_target"
                    }),
                },
            )?;
            Ok(())
        })
        .unwrap();

    let ctx = test_handler_context(storage.clone());
    let reviewed = dispatch(
        &ctx,
        "dream_candidate_review",
        json!({"id": "expire-race-candidate", "review_state": "accepted"}),
    );
    assert_eq!(reviewed.get("status").unwrap(), "success");

    storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET valid_to = datetime('now') WHERE id = ?1",
                params![target_id],
            )?;
            Ok(())
        })
        .unwrap();

    let applied = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "expire-race-candidate", "confirm": true}),
    );
    assert!(
        applied.get("error").is_some(),
        "stale expire target should return an error, got {applied:?}"
    );

    let candidate = storage
        .with_connection(|conn| get_dream_candidate(conn, "expire-race-candidate"))
        .unwrap()
        .expect("candidate exists");
    assert_eq!(candidate.review_state, "accepted");
    assert!(candidate.application_result.is_none());
    assert!(candidate.applied_at.is_none());
}
