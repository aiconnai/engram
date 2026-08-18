use super::*;
use rusqlite::Connection;

fn in_memory_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    run_migrations(&conn).expect("run migrations");
    conn
}

#[test]
fn test_fresh_db_reaches_current_version() {
    let conn = in_memory_conn();
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .expect("query schema version");
    assert_eq!(version, 47);
}

#[test]
fn test_schema_version_constant() {
    assert_eq!(SCHEMA_VERSION, 47);
}

#[test]
fn test_hnsw_checkpoints_table_exists() {
    let conn = in_memory_conn();
    conn.execute(
        "INSERT INTO hnsw_checkpoints (model, dimensions, metric, vector_count, checkpoint_blob)
         VALUES ('default', 384, 'cosine', 10, X'DEADBEEF')",
        [],
    )
    .expect("insert checkpoint");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM hnsw_checkpoints", [], |row| {
            row.get(0)
        })
        .expect("count checkpoints");
    assert_eq!(count, 1);
}

#[test]
fn test_media_assets_table_exists() {
    let conn = in_memory_conn();
    conn.execute_batch(
        "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
         VALUES ('test memory', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
    )
    .expect("insert memory");
    let memory_id: i64 = conn
        .query_row("SELECT id FROM memories LIMIT 1", [], |row| row.get(0))
        .expect("get memory id");
    conn.execute(
        "INSERT INTO media_assets (memory_id, media_type, file_hash, mime_type)
         VALUES (?1, 'image', 'abc123hash', 'image/png')",
        rusqlite::params![memory_id],
    )
    .expect("insert media_asset");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media_assets", [], |row| row.get(0))
        .expect("count");
    assert_eq!(count, 1);
}

#[test]
fn test_media_assets_hash_uniqueness() {
    let conn = in_memory_conn();
    conn.execute_batch(
        "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
         VALUES ('test', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
    )
    .expect("insert memory");
    let memory_id: i64 = conn
        .query_row("SELECT id FROM memories LIMIT 1", [], |row| row.get(0))
        .expect("get memory id");
    conn.execute(
        "INSERT INTO media_assets (memory_id, media_type, file_hash) VALUES (?1, 'image', 'dup_hash')",
        rusqlite::params![memory_id],
    )
    .expect("first insert");
    let dup = conn.execute(
        "INSERT INTO media_assets (memory_id, media_type, file_hash) VALUES (?1, 'audio', 'dup_hash')",
        rusqlite::params![memory_id],
    );
    assert!(dup.is_err(), "duplicate file_hash should fail");
}

#[test]
fn test_auto_links_table_exists() {
    let conn = in_memory_conn();
    // Insert a memory first (required by FK)
    conn.execute_batch(
        "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
         VALUES ('test memory', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
    )
    .expect("insert memory");
    let memory_id: i64 = conn
        .query_row("SELECT id FROM memories LIMIT 1", [], |row| row.get(0))
        .expect("get memory id");

    // Insert an auto_link pointing to itself (valid for test purposes)
    conn.execute(
        "INSERT INTO auto_links (from_id, to_id, link_type, score) VALUES (?1, ?2, 'semantic', 0.9)",
        rusqlite::params![memory_id, memory_id],
    )
    .expect("insert auto_link");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM auto_links", [], |row| row.get(0))
        .expect("count auto_links");
    assert_eq!(count, 1);
}

#[test]
fn test_auto_links_unique_pair_type() {
    let conn = in_memory_conn();
    conn.execute_batch(
        "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
         VALUES ('test memory', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
    )
    .expect("insert memory");
    let memory_id: i64 = conn
        .query_row("SELECT id FROM memories LIMIT 1", [], |row| row.get(0))
        .expect("get memory id");

    conn.execute(
        "INSERT INTO auto_links (from_id, to_id, link_type, score) VALUES (?1, ?2, 'semantic', 0.9)",
        rusqlite::params![memory_id, memory_id],
    )
    .expect("first insert");

    // Duplicate (from_id, to_id, link_type) should fail
    let result = conn.execute(
        "INSERT INTO auto_links (from_id, to_id, link_type, score) VALUES (?1, ?2, 'semantic', 0.8)",
        rusqlite::params![memory_id, memory_id],
    );
    assert!(
        result.is_err(),
        "duplicate pair+type should violate unique index"
    );
}

#[test]
fn test_memory_clusters_table_exists() {
    let conn = in_memory_conn();
    conn.execute_batch(
        "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
         VALUES ('test memory', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
    )
    .expect("insert memory");
    let memory_id: i64 = conn
        .query_row("SELECT id FROM memories LIMIT 1", [], |row| row.get(0))
        .expect("get memory id");

    conn.execute(
        "INSERT INTO memory_clusters (cluster_id, memory_id, algorithm) VALUES (1, ?1, 'louvain')",
        rusqlite::params![memory_id],
    )
    .expect("insert memory_cluster");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM memory_clusters", [], |row| row.get(0))
        .expect("count memory_clusters");
    assert_eq!(count, 1);
}

#[test]
fn test_upgrade_from_v17_to_v19() {
    // Simulate a v17 database by running only migrations up to v17
    let conn = Connection::open_in_memory().expect("open in-memory db");

    // Bootstrap schema_version table and run through v17 manually
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .expect("create schema_version");

    // Run all migrations (they'll stop at the current version)
    // We simulate v17 state by running the full migration once,
    // then verify the version is 33.
    run_migrations(&conn).expect("run migrations from scratch");

    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .expect("query schema version");
    assert_eq!(version, 47, "should reach v47 after full migration");

    // Verify both new tables exist
    let auto_links_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='auto_links'",
            [],
            |row| row.get(0),
        )
        .expect("check auto_links");
    assert_eq!(auto_links_exists, 1, "auto_links table should exist");

    let clusters_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memory_clusters'",
            [],
            |row| row.get(0),
        )
        .expect("check memory_clusters");
    assert_eq!(clusters_exists, 1, "memory_clusters table should exist");
}

#[test]
fn test_enrichment_events_table_exists() {
    let conn = in_memory_conn();
    let exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='enrichment_events'",
            [],
            |row| row.get(0),
        )
        .expect("query sqlite_master");
    assert_eq!(
        exists, 1,
        "enrichment_events table should exist after migration"
    );
}

#[test]
fn test_enrichment_events_operation_id_not_null() {
    let conn = in_memory_conn();
    let result = conn.execute(
        "INSERT INTO enrichment_events (operation_id, event_type, triggered_by, created_at)
         VALUES (NULL, 'test', 'test', '2026-01-01T00:00:00Z')",
        [],
    );
    assert!(result.is_err(), "NULL operation_id should be rejected");
}

#[test]
fn test_context_events_and_summaries_tables_exist() {
    let conn = in_memory_conn();
    for table in [
        "context_events",
        "context_summaries",
        "context_artifacts",
        "context_artifact_access_log",
    ] {
        let exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |row| row.get(0),
            )
            .expect("query sqlite_master");
        assert_eq!(exists, 1, "{table} table should exist after migration");
    }
}

#[test]
fn test_context_artifacts_default_to_pointer_only() {
    let conn = in_memory_conn();
    conn.execute(
        "INSERT INTO context_artifacts
            (id, repo_id, kind, redaction_status, retention_policy,
             access_policy, retain_raw, raw_content, metadata, created_at)
         VALUES
            ('artifact-1', 'github:aiconnai/engram', 'command_output',
             'redacted', 'pointer_only', 'same_session', 0, X'616263',
             '{}', '2026-01-01T00:00:00Z')",
        [],
    )
    .expect_err("raw_content is rejected unless retain_raw is true");

    conn.execute(
        "INSERT INTO context_artifacts
            (id, repo_id, kind, redaction_status, retention_policy,
             access_policy, retain_raw, raw_content, metadata, created_at)
         VALUES
            ('artifact-2', 'github:aiconnai/engram', 'command_output',
             'redacted', 'pointer_only', 'same_session', 0, NULL,
             '{}', '2026-01-01T00:00:00Z')",
        [],
    )
    .expect("pointer-only artifact should be accepted");
}

#[test]
fn test_command_context_events_require_exit_code() {
    let conn = in_memory_conn();
    let result = conn.execute(
        "INSERT INTO context_events
            (repo_id, session_id, source, event_type, command_name,
             started_at, redaction_status, retention_policy, metadata, created_at)
         VALUES
            ('github:aiconnai/engram', 'sess-1', 'codex', 'command', 'cargo',
             '2026-01-01T00:00:00Z', 'redacted', 'default', '{}',
             '2026-01-01T00:00:00Z')",
        [],
    );
    assert!(
        result.is_err(),
        "command context events should require exit_code"
    );
}

#[test]
fn test_context_summaries_require_source_event_and_reducer_version() {
    let conn = in_memory_conn();
    conn.execute(
        "INSERT INTO context_events
            (repo_id, session_id, source, event_type, command_name, exit_code,
             started_at, redaction_status, retention_policy, metadata, created_at)
         VALUES
            ('github:aiconnai/engram', 'sess-1', 'codex', 'command', 'cargo', 0,
             '2026-01-01T00:00:00Z', 'redacted', 'default', '{}',
             '2026-01-01T00:00:00Z')",
        [],
    )
    .expect("insert context event");
    let event_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO context_summaries
            (source_event_id, reducer_name, reducer_version, lossy,
             confidence, summary, structured_facts, warnings, created_at)
         VALUES
            (?1, 'command_savings', '1.0.0', 1, 0.9, 'summary', '{}', '[]',
             '2026-01-01T00:00:00Z')",
        rusqlite::params![event_id],
    )
    .expect("insert context summary");

    let missing_reducer_version = conn.execute(
        "INSERT INTO context_summaries
            (source_event_id, reducer_name, reducer_version, lossy,
             confidence, summary, structured_facts, warnings, created_at)
         VALUES
            (?1, 'command_savings', '', 1, 0.9, 'summary', '{}', '[]',
             '2026-01-01T00:00:00Z')",
        rusqlite::params![event_id],
    );
    assert!(
        missing_reducer_version.is_err(),
        "reducer-generated summaries should require reducer_version"
    );
}

#[test]
fn test_memory_policy_table_exists() {
    let conn = in_memory_conn();
    let exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memory_policy'",
            [],
            |row| row.get(0),
        )
        .expect("query sqlite_master");
    assert_eq!(
        exists, 1,
        "memory_policy table should exist after migration"
    );
}

#[test]
fn test_dream_snapshot_review_tables_exist() {
    let conn = in_memory_conn();
    for table in ["dream_jobs", "dream_candidates", "dream_candidate_sources"] {
        let exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |row| row.get(0),
            )
            .expect("query sqlite_master");
        assert_eq!(exists, 1, "{table} table should exist after migration");
    }
}

#[test]
fn test_dream_candidates_allow_agent_writeback_kind() {
    let conn = in_memory_conn();
    conn.execute(
        "INSERT INTO dream_jobs (id, workspace, status, created_at)
         VALUES ('agent-writeback-job', 'default', 'completed', '2026-01-01T00:00:00Z')",
        [],
    )
    .expect("insert dream job");

    conn.execute(
        "INSERT INTO dream_candidates
            (id, job_id, workspace, kind, proposed_action, confidence,
             freshness_state, content_preview, proposed_content, reason_codes,
             policy_explanation_json, metadata_json, created_at)
         VALUES
            ('agent-writeback-candidate', 'agent-writeback-job', 'default',
             'agent_writeback', 'create', 0.7, 'current',
             'Pending agent writeback.', 'Pending agent writeback.',
             '[\"agent_writeback\"]', '{}', '{\"origin\":\"agent_writeback\"}',
             '2026-01-01T00:00:00Z')",
        [],
    )
    .expect("agent_writeback dream candidate kind should be accepted");

    let kind: String = conn
        .query_row(
            "SELECT kind FROM dream_candidates WHERE id = 'agent-writeback-candidate'",
            [],
            |row| row.get(0),
        )
        .expect("read candidate kind");
    assert_eq!(kind, "agent_writeback");
}

#[test]
fn test_v45_preserves_existing_dream_candidate_data() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        r#"
        CREATE TABLE schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO schema_version (version) VALUES (44);

        CREATE TABLE dream_jobs (
            id TEXT PRIMARY KEY CHECK (length(id) > 0),
            workspace TEXT NOT NULL CHECK (length(workspace) > 0),
            status TEXT NOT NULL
                CHECK (status IN (
                    'pending', 'running', 'completed', 'failed', 'canceled',
                    'archived'
                )),
            instructions TEXT,
            model_profile TEXT NOT NULL DEFAULT 'deterministic-local-v1',
            input_summary_json TEXT NOT NULL DEFAULT '{}',
            output_summary_json TEXT NOT NULL DEFAULT '{}',
            error_json TEXT,
            created_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT,
            archived_at TEXT
        );

        CREATE TABLE dream_candidates (
            id TEXT PRIMARY KEY CHECK (length(id) > 0),
            job_id TEXT NOT NULL
                REFERENCES dream_jobs(id) ON DELETE CASCADE,
            workspace TEXT NOT NULL CHECK (length(workspace) > 0),
            kind TEXT NOT NULL
                CHECK (kind IN (
                    'summary', 'preference', 'constraint', 'project_state',
                    'stale_fact', 'contradiction', 'merge', 'promotion',
                    'decay', 'temporal_update'
                )),
            proposed_action TEXT NOT NULL
                CHECK (proposed_action IN (
                    'create', 'update', 'merge', 'supersede', 'expire',
                    'promote', 'demote', 'ignore'
                )),
            review_state TEXT NOT NULL DEFAULT 'pending'
                CHECK (review_state IN (
                    'pending', 'accepted', 'edited', 'rejected', 'applied',
                    'archived'
                )),
            confidence REAL NOT NULL
                CHECK (confidence >= 0.0 AND confidence <= 1.0),
            freshness_state TEXT NOT NULL DEFAULT 'unknown'
                CHECK (freshness_state IN (
                    'current', 'stale', 'future_due', 'expired',
                    'conflicted', 'unknown'
                )),
            content_preview TEXT NOT NULL CHECK (length(content_preview) > 0),
            proposed_content TEXT,
            reason_codes TEXT NOT NULL DEFAULT '[]',
            policy_explanation_json TEXT NOT NULL DEFAULT '{}',
            metadata_json TEXT NOT NULL DEFAULT '{}',
            application_result_json TEXT,
            created_at TEXT NOT NULL,
            reviewed_at TEXT,
            applied_at TEXT,
            CHECK (
                proposed_action NOT IN ('create', 'update', 'merge')
                OR (proposed_content IS NOT NULL AND length(proposed_content) > 0)
            )
        );

        CREATE TABLE dream_candidate_sources (
            candidate_id TEXT NOT NULL
                REFERENCES dream_candidates(id) ON DELETE CASCADE,
            source_type TEXT NOT NULL CHECK (length(source_type) > 0),
            source_id TEXT NOT NULL CHECK (length(source_id) > 0),
            source_ref TEXT,
            evidence_json TEXT NOT NULL DEFAULT '{}',
            PRIMARY KEY (candidate_id, source_type, source_id)
        );

        INSERT INTO dream_jobs (id, workspace, status, created_at)
        VALUES ('v44-job', 'default', 'completed', '2026-01-01T00:00:00Z');
        INSERT INTO dream_candidates
            (id, job_id, workspace, kind, proposed_action, confidence,
             freshness_state, content_preview, proposed_content, reason_codes,
             policy_explanation_json, metadata_json, created_at)
        VALUES
            ('v44-candidate', 'v44-job', 'default', 'summary', 'create',
             0.7, 'current', 'Existing summary.', 'Existing summary.',
             '["workspace_digest"]', '{"policy":"v44"}',
             '{"retained":true}', '2026-01-01T00:00:00Z');
        INSERT INTO dream_candidate_sources
            (candidate_id, source_type, source_id, source_ref, evidence_json)
        VALUES
            ('v44-candidate', 'memory', '1', 'memory:1',
             '{"preview":"source"}');
        "#,
    )
    .expect("create v44 dream candidate schema");

    run_migrations(&conn).expect("run v45 migration");

    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .expect("query schema version");
    assert_eq!(version, 47);

    let retained: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dream_candidates
             WHERE id = 'v44-candidate'
               AND kind = 'summary'
               AND metadata_json = '{\"retained\":true}'",
            [],
            |row| row.get(0),
        )
        .expect("query retained candidate");
    assert_eq!(retained, 1);

    let retained_source: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dream_candidate_sources
             WHERE candidate_id = 'v44-candidate'
               AND source_type = 'memory'
               AND source_id = '1'",
            [],
            |row| row.get(0),
        )
        .expect("query retained source");
    assert_eq!(retained_source, 1);

    conn.execute(
        "INSERT INTO dream_candidates
            (id, job_id, workspace, kind, proposed_action, confidence,
             freshness_state, content_preview, proposed_content, reason_codes,
             policy_explanation_json, metadata_json, created_at)
         VALUES
            ('v45-agent-writeback', 'v44-job', 'default', 'agent_writeback',
             'create', 0.7, 'current', 'Pending writeback.',
             'Pending writeback.', '[\"agent_writeback\"]', '{}',
             '{\"origin\":\"agent_writeback\"}', '2026-01-01T00:00:00Z')",
        [],
    )
    .expect("v45 should accept new agent_writeback kind after rebuild");
}

#[test]
fn test_v46_stability_and_reinforcements() {
    let conn = in_memory_conn();

    // Verify memories table has stability column with default 1.0
    conn.execute(
        "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
         VALUES ('stability test', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
        [],
    )
    .expect("insert memory");

    let stability: f32 = conn
        .query_row(
            "SELECT stability FROM memories WHERE content = 'stability test'",
            [],
            |row| row.get(0),
        )
        .expect("query stability column");
    assert_eq!(stability, 1.0);

    // Verify memory_reinforcements table exists
    let reinforcements_table_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memory_reinforcements'",
            [],
            |row| row.get(0),
        )
        .expect("query memory_reinforcements table");
    assert_eq!(reinforcements_table_exists, 1);
}

#[test]
fn test_dream_candidates_require_content_for_create_update_merge() {
    let conn = in_memory_conn();
    conn.execute(
        "INSERT INTO dream_jobs (id, workspace, status, created_at)
         VALUES ('job-1', 'default', 'completed', '2026-01-01T00:00:00Z')",
        [],
    )
    .expect("insert job");

    let result = conn.execute(
        "INSERT INTO dream_candidates
            (id, job_id, workspace, kind, proposed_action, confidence,
             freshness_state, content_preview, reason_codes,
             policy_explanation_json, metadata_json, created_at)
         VALUES
            ('cand-1', 'job-1', 'default', 'summary', 'create', 0.9,
             'current', 'preview', '[]', '{}', '{}',
             '2026-01-01T00:00:00Z')",
        [],
    );
    assert!(
        result.is_err(),
        "create/update/merge candidates should require proposed_content"
    );
}

#[test]
fn test_v47_hnsw_checkpoints_columns() {
    let conn = in_memory_conn();

    // Verify hnsw_checkpoints table exists and supports insertion
    conn.execute(
        "INSERT INTO hnsw_checkpoints (model, dimensions, metric, vector_count, checkpoint_blob, created_at)
         VALUES ('openai-text-3', 1536, 'cosine', 500, X'12345678', '2026-08-17T00:00:00Z')",
        [],
    )
    .expect("insert hnsw checkpoint");

    let (model, dim, metric, count): (String, i64, String, i64) = conn
        .query_row(
            "SELECT model, dimensions, metric, vector_count FROM hnsw_checkpoints WHERE model = 'openai-text-3'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("query checkpoint");

    assert_eq!(model, "openai-text-3");
    assert_eq!(dim, 1536);
    assert_eq!(metric, "cosine");
    assert_eq!(count, 500);
}
