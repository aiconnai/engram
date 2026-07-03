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
fn test_mcp_memory_agent_writeback_requires_review_before_canonical_apply() {
    use engram::mcp::handlers::dispatch;
    use engram::storage::Storage;
    use rusqlite::params;
    use serde_json::json;

    let storage = Storage::open_in_memory().unwrap();
    let source_id: i64 = storage
        .with_connection(|conn| {
            conn.execute(
                "INSERT INTO memories (content, workspace, importance, created_at, updated_at)
                 VALUES ('Source fact for agent writeback.', 'default', 0.8, datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .unwrap();
    let ctx = test_handler_context(storage.clone());

    let missing_evidence = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({"proposed_content": "Agent-generated fact without evidence."}),
    );
    assert!(
        missing_evidence.get("error").is_some(),
        "agent writeback should require evidence, got {missing_evidence:?}"
    );

    let dry_run = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "proposed_content": "Agent-generated summary pending review.",
            "source_memory_ids": [source_id]
        }),
    );
    assert_eq!(dry_run.get("status").unwrap(), "dry_run");
    assert_eq!(dry_run["canonical_memory_mutated"], json!(false));
    assert_eq!(
        dry_run["candidate"]["candidate"]["kind"], "agent_writeback",
        "dry-run and live responses should expose the same candidate wrapper shape"
    );
    assert_eq!(dry_run["candidate"]["candidate"]["review_state"], "pending");
    assert_eq!(
        dry_run["candidate"]["sources"]
            .as_array()
            .expect("dry-run candidate sources")
            .len(),
        1
    );

    let unconfirmed = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "proposed_content": "Agent-generated summary pending review.",
            "source_memory_ids": [source_id],
            "dry_run": false
        }),
    );
    assert!(
        unconfirmed.get("error").is_some(),
        "live agent writeback should require confirm=true, got {unconfirmed:?}"
    );

    let before_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    let created = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "candidate_id": "agent-writeback-candidate-1",
            "job_id": "agent-writeback-job-1",
            "workspace": "default",
            "proposed_content": "Agent-generated summary pending review.",
            "source_memory_ids": [source_id],
            "evidence": [{
                "source_type": "session",
                "source_id": "session-1",
                "source_ref": "session:1",
                "evidence": {"excerpt": "Agent observed the source fact."}
            }],
            "dry_run": false,
            "confirm": true
        }),
    );
    assert_eq!(created.get("status").unwrap(), "success");
    assert_eq!(created["canonical_memory_mutated"], json!(false));
    assert_eq!(created["candidate"]["candidate"]["kind"], "agent_writeback");
    assert_eq!(created["candidate"]["candidate"]["review_state"], "pending");
    assert_eq!(
        created["candidate"]["sources"]
            .as_array()
            .expect("candidate sources")
            .len(),
        2
    );
    let after_create_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(
        after_create_count, before_count,
        "pending writeback creation must not mutate canonical memory"
    );
    let job_status: String = storage
        .with_connection(|conn| {
            conn.query_row(
                "SELECT status FROM dream_jobs WHERE id = 'agent-writeback-job-1'",
                [],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(
        job_status, "completed",
        "synthetic agent writeback jobs should not remain permanently pending"
    );

    let fetched = dispatch(
        &ctx,
        "dream_candidate_get",
        json!({"id": "agent-writeback-candidate-1"}),
    );
    assert_eq!(
        fetched["candidate"]["sources"]
            .as_array()
            .expect("fetched candidate sources")
            .len(),
        2
    );

    let premature_apply = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "agent-writeback-candidate-1", "confirm": true}),
    );
    assert!(
        premature_apply.get("error").is_some(),
        "pending agent writeback must be reviewed before apply, got {premature_apply:?}"
    );

    let reviewed = dispatch(
        &ctx,
        "dream_candidate_review",
        json!({"id": "agent-writeback-candidate-1", "review_state": "accepted"}),
    );
    assert_eq!(reviewed.get("status").unwrap(), "success");

    let apply_dry_run = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "agent-writeback-candidate-1", "dry_run": true}),
    );
    assert_eq!(apply_dry_run.get("status").unwrap(), "dry_run");
    assert_eq!(
        apply_dry_run["planned"]["kind"].as_str(),
        Some("agent_writeback")
    );

    let applied = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "agent-writeback-candidate-1", "confirm": true}),
    );
    assert_eq!(applied.get("status").unwrap(), "completed");
    let applied_memory_id = applied["application"]["canonical_memory_ids"][0]
        .as_i64()
        .expect("applied canonical memory id");
    let applied_memory_type: String = storage
        .with_connection(|conn| {
            conn.query_row(
                "SELECT memory_type FROM memories WHERE id = ?1",
                params![applied_memory_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(
        applied_memory_type, "learning",
        "agent_writeback candidates should map to a deliberate memory type"
    );
    let after_apply_count: i64 = storage
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(after_apply_count, before_count + 1);
}

#[cfg(feature = "dream-phase")]
#[test]
fn test_mcp_memory_agent_writeback_rejects_reuse_and_spoofing() {
    use engram::mcp::handlers::dispatch;
    use engram::storage::{create_dream_job, NewDreamJob, Storage};
    use serde_json::json;

    let storage = Storage::open_in_memory().unwrap();
    let source_id: i64 = storage
        .with_connection(|conn| {
            conn.execute(
                "INSERT INTO memories (content, workspace, importance, created_at, updated_at)
                 VALUES ('Source fact for reuse guard.', 'default', 0.8, datetime('now'), datetime('now'))",
                [],
            )?;
            let source_id = conn.last_insert_rowid();
            create_dream_job(
                conn,
                &NewDreamJob {
                    id: Some("normal-dream-job"),
                    workspace: "default",
                    instructions: Some("ordinary dream job"),
                    model_profile: None,
                    input_summary: &json!({"created_by": "dream_create"}),
                },
            )?;
            Ok(source_id)
        })
        .unwrap();
    let ctx = test_handler_context(storage.clone());

    let reused_normal_job = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "candidate_id": "reuse-normal-job-candidate",
            "job_id": "normal-dream-job",
            "proposed_content": "Agent writeback must not attach to ordinary dream jobs.",
            "source_memory_ids": [source_id],
            "dry_run": false,
            "confirm": true
        }),
    );
    let normal_job_error = reused_normal_job["error"]
        .as_str()
        .expect("ordinary job reuse should fail");
    assert!(
        normal_job_error.contains("not an agent writeback job"),
        "expected origin guard error, got {reused_normal_job}"
    );

    let created = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "candidate_id": "duplicate-agent-writeback-candidate",
            "job_id": "duplicate-agent-writeback-job",
            "proposed_content": "Agent writeback with duplicate candidate guard.",
            "source_memory_ids": [source_id],
            "dry_run": false,
            "confirm": true
        }),
    );
    assert_eq!(created.get("status").unwrap(), "success");

    let duplicate_candidate = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "candidate_id": "duplicate-agent-writeback-candidate",
            "job_id": "duplicate-agent-writeback-job-2",
            "proposed_content": "Duplicate candidate should return a clean conflict.",
            "source_memory_ids": [source_id],
            "dry_run": false,
            "confirm": true
        }),
    );
    let duplicate_error = duplicate_candidate["error"]
        .as_str()
        .expect("duplicate candidate should fail");
    assert!(
        duplicate_error.contains("dream candidate already exists"),
        "duplicate error should be domain-level, got {duplicate_candidate}"
    );
    assert!(
        !duplicate_error.contains("UNIQUE constraint") && !duplicate_error.contains("SQLITE"),
        "duplicate error should not leak raw SQL details: {duplicate_candidate}"
    );

    let reused_completed_job = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "candidate_id": "reuse-completed-job-candidate",
            "job_id": "duplicate-agent-writeback-job",
            "proposed_content": "Completed synthetic jobs must not be reused.",
            "source_memory_ids": [source_id],
            "dry_run": false,
            "confirm": true
        }),
    );
    let completed_job_error = reused_completed_job["error"]
        .as_str()
        .expect("completed job reuse should fail");
    assert!(
        completed_job_error.contains("must be pending"),
        "expected status guard error, got {reused_completed_job}"
    );

    let spoofed_metadata = dispatch(
        &ctx,
        "memory_agent_writeback",
        json!({
            "proposed_content": "Reserved metadata keys must not be spoofed by casing.",
            "source_memory_ids": [source_id],
            "metadata": {
                "Origin": "user-controlled",
                "review_required": false
            }
        }),
    );
    let spoof_error = spoofed_metadata["error"]
        .as_str()
        .expect("reserved metadata spoofing should fail");
    assert!(
        spoof_error.contains("reserved metadata key"),
        "expected reserved metadata error, got {spoofed_metadata}"
    );
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

#[cfg(feature = "dream-phase")]
#[test]
fn test_mcp_merge_candidate_uses_derived_from_edges_without_superseding_sources() {
    use engram::mcp::handlers::dispatch;
    use engram::storage::{
        create_dream_candidate, create_dream_job, NewDreamCandidate, NewDreamJob, Storage,
    };
    use rusqlite::params;
    use serde_json::json;

    let storage = Storage::open_in_memory().unwrap();
    let (source_a, source_b): (i64, i64) = storage
        .with_connection(|conn| {
            conn.execute(
                "INSERT INTO memories (content, workspace, importance, created_at, updated_at)
                 VALUES ('Source memory A.', 'default', 0.7, datetime('now'), datetime('now'))",
                [],
            )?;
            let source_a = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO memories (content, workspace, importance, created_at, updated_at)
                 VALUES ('Source memory B.', 'default', 0.7, datetime('now'), datetime('now'))",
                [],
            )?;
            Ok((source_a, conn.last_insert_rowid()))
        })
        .unwrap();

    storage
        .with_connection(|conn| {
            create_dream_job(
                conn,
                &NewDreamJob {
                    id: Some("merge-edge-job"),
                    workspace: "default",
                    instructions: Some("merge edge regression"),
                    model_profile: None,
                    input_summary: &json!({}),
                },
            )?;
            create_dream_candidate(
                conn,
                &NewDreamCandidate {
                    id: Some("merge-edge-candidate"),
                    job_id: "merge-edge-job",
                    workspace: "default",
                    kind: "merge",
                    proposed_action: "merge",
                    confidence: 0.86,
                    freshness_state: "current",
                    content_preview: "Merged source memory.",
                    proposed_content: Some("Merged source memory."),
                    reason_codes: &json!(["merge_related_context"]),
                    policy_explanation: &json!({}),
                    metadata: &json!({"target_memory_ids": [source_a, source_b]}),
                },
            )?;
            Ok(())
        })
        .unwrap();

    let ctx = test_handler_context(storage.clone());
    let reviewed = dispatch(
        &ctx,
        "dream_candidate_review",
        json!({"id": "merge-edge-candidate", "review_state": "accepted"}),
    );
    assert_eq!(reviewed.get("status").unwrap(), "success");

    let applied = dispatch(
        &ctx,
        "dream_candidate_apply",
        json!({"id": "merge-edge-candidate", "confirm": true}),
    );
    assert_eq!(applied.get("status").unwrap(), "completed");
    let merged_id = applied["application"]["canonical_memory_ids"][0]
        .as_i64()
        .expect("merged canonical memory id");

    storage
        .with_connection(|conn| {
            let derived_edges: i64 = conn.query_row(
                "SELECT COUNT(*)
                 FROM crossrefs
                 WHERE from_id = ?1
                   AND to_id IN (?2, ?3)
                   AND edge_type = 'derived_from'
                   AND valid_to IS NULL",
                params![merged_id, source_a, source_b],
                |row| row.get(0),
            )?;
            assert_eq!(derived_edges, 2);

            let supersedes_edges: i64 = conn.query_row(
                "SELECT COUNT(*)
                 FROM crossrefs
                 WHERE from_id = ?1
                   AND to_id IN (?2, ?3)
                   AND edge_type = 'supersedes'
                   AND valid_to IS NULL",
                params![merged_id, source_a, source_b],
                |row| row.get(0),
            )?;
            assert_eq!(supersedes_edges, 0);

            let active_sources: i64 = conn.query_row(
                "SELECT COUNT(*)
                 FROM memories
                 WHERE id IN (?1, ?2)
                   AND valid_to IS NULL
                   AND COALESCE(lifecycle_state, 'active') = 'active'",
                params![source_a, source_b],
                |row| row.get(0),
            )?;
            assert_eq!(active_sources, 2);
            Ok(())
        })
        .unwrap();
}
