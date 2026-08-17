use rusqlite::Connection;

use crate::error::Result;

pub(super) fn migrate_v34(conn: &Connection) -> Result<()> {
    tracing::info!(
        "Migration v34: Adding media_url column to memories table for multimodal support..."
    );

    conn.execute_batch(
        r#"
        -- Multimodal: URL or local path to the primary media asset
        -- Nullable, additive column. Used by Image, Audio, Video memory types.
        ALTER TABLE memories ADD COLUMN media_url TEXT;
        CREATE INDEX IF NOT EXISTS idx_memories_media_url ON memories(media_url) WHERE media_url IS NOT NULL;

        INSERT INTO schema_version (version) VALUES (34);
        "#,
    )?;

    tracing::info!("Migration v34 complete: media_url column added to memories");

    Ok(())
}

pub(super) fn migrate_v35(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v35: Creating dream_runs table...");

    conn.execute_batch(
        r#"
        -- Dream Phase: History of background consolidation runs
        CREATE TABLE IF NOT EXISTS dream_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TEXT NOT NULL,
            finished_at TEXT NOT NULL,
            report_json TEXT NOT NULL,
            error_count INTEGER NOT NULL DEFAULT 0,
            workspace_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_dream_runs_started ON dream_runs(started_at DESC);

        INSERT INTO schema_version (version) VALUES (35);
        "#,
    )?;

    tracing::info!("Migration v35 complete: dream_runs table created");

    Ok(())
}

pub(super) fn migrate_v36(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v36: Creating dream_locks table...");

    conn.execute_batch(
        r#"
        -- Advisory locks for background processes (Dream Phase)
        CREATE TABLE IF NOT EXISTS dream_locks (
            lock_id TEXT PRIMARY KEY,
            acquired_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            owner_id TEXT NOT NULL
        );

        INSERT INTO schema_version (version) VALUES (36);
        "#,
    )?;

    tracing::info!("Migration v36 complete: dream_locks table created");

    Ok(())
}

pub(super) fn migrate_v37(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v37: Creating consolidation_runs table...");

    conn.execute_batch(
        r#"
        -- Audit log of auto-consolidation passes. One row per run regardless
        -- of how many actions were taken. The `report` column holds the full
        -- `ConsolidationReport` as JSON for forensic queries; counters are
        -- denormalised into top-level columns for fast charting and ceiling
        -- checks.
        CREATE TABLE IF NOT EXISTS consolidation_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT NOT NULL,
            dry_run INTEGER NOT NULL,
            duplicates_merged INTEGER NOT NULL DEFAULT 0,
            conflicts_resolved INTEGER NOT NULL DEFAULT 0,
            summarized INTEGER NOT NULL DEFAULT 0,
            skipped INTEGER NOT NULL DEFAULT 0,
            report TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_consolidation_runs_workspace
            ON consolidation_runs(workspace, started_at DESC);

        INSERT INTO schema_version (version) VALUES (37);
        "#,
    )?;

    tracing::info!("Migration v37 complete: consolidation_runs table created");

    Ok(())
}

pub(super) fn migrate_v38(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v38: Creating pending_injections table...");

    conn.execute_batch(
        r#"
        -- Queue of injection payloads to be consumed at the next SessionStart
        -- for a given workspace. The producer (SessionEnd hook) writes one row
        -- per relevant outgoing session; the consumer (SessionStart hook)
        -- reads-and-deletes oldest-first.
        --
        -- This decouples the two hooks: SessionEnd cannot know the *next*
        -- session id, so it cannot target a specific session. Workspace is
        -- the routing key, and FIFO within a workspace preserves intent
        -- order when multiple agents end overlapping sessions.
        CREATE TABLE IF NOT EXISTS pending_injections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace TEXT NOT NULL,
            payload TEXT NOT NULL,
            source_session_id TEXT,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_pending_injections_workspace
            ON pending_injections(workspace, created_at);

        INSERT INTO schema_version (version) VALUES (38);
        "#,
    )?;

    tracing::info!("Migration v38 complete: pending_injections table created");

    Ok(())
}

/// Widen `search_feedback.signal` CHECK constraint from {useful, irrelevant}
/// to {useful, irrelevant, outdated, conflict} so the new feedback handler
/// can persist all four signal types with distinct semantics.
///
/// SQLite cannot ALTER a CHECK constraint in place; we recreate the table.
pub(super) fn migrate_v39(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v39: Widening search_feedback.signal CHECK constraint...");

    conn.execute_batch(
        r#"
        CREATE TABLE search_feedback_v39 (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query TEXT NOT NULL,
            query_embedding_hash TEXT,
            memory_id INTEGER NOT NULL,
            signal TEXT NOT NULL CHECK(signal IN ('useful', 'irrelevant', 'outdated', 'conflict')),
            rank_position INTEGER,
            original_score REAL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            workspace TEXT DEFAULT 'default'
        );

        INSERT INTO search_feedback_v39
            (id, query, query_embedding_hash, memory_id, signal,
             rank_position, original_score, created_at, workspace)
        SELECT id, query, query_embedding_hash, memory_id, signal,
               rank_position, original_score, created_at, workspace
        FROM search_feedback;

        DROP TABLE search_feedback;
        ALTER TABLE search_feedback_v39 RENAME TO search_feedback;

        CREATE INDEX IF NOT EXISTS idx_feedback_memory ON search_feedback(memory_id);
        CREATE INDEX IF NOT EXISTS idx_feedback_query ON search_feedback(query);
        CREATE INDEX IF NOT EXISTS idx_feedback_workspace ON search_feedback(workspace);

        INSERT INTO schema_version (version) VALUES (39);
        "#,
    )?;

    tracing::info!("Migration v39 complete: search_feedback signal types widened");

    Ok(())
}

pub(super) fn migrate_v40(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v40: Creating enrichment_events table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS enrichment_events (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            operation_id TEXT NOT NULL,
            event_type   TEXT NOT NULL,
            memory_id    INTEGER,          -- no FK: preserved for audit even after hard delete
            version_id   INTEGER REFERENCES memory_versions(id) ON DELETE SET NULL,
            triggered_by TEXT NOT NULL,
            agent_id     TEXT,
            workspace    TEXT,
            params       TEXT NOT NULL DEFAULT '{}',
            outcome      TEXT NOT NULL DEFAULT '{}',
            status       TEXT NOT NULL DEFAULT 'completed'
                             CHECK (status IN ('completed', 'failed', 'skipped')),
            dry_run      INTEGER NOT NULL DEFAULT 0
                             CHECK (dry_run IN (0, 1)),
            created_at   TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_enrichment_by_memory
            ON enrichment_events(memory_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_type
            ON enrichment_events(event_type, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_operation
            ON enrichment_events(operation_id);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_triggered_by
            ON enrichment_events(triggered_by, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_workspace
            ON enrichment_events(workspace, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_time
            ON enrichment_events(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_version
            ON enrichment_events(version_id)
            WHERE version_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_agent
            ON enrichment_events(agent_id, created_at DESC)
            WHERE agent_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_enrichment_by_status
            ON enrichment_events(status, created_at DESC);

        INSERT INTO schema_version (version) VALUES (40);
        "#,
    )?;

    tracing::info!("Migration v40 complete: enrichment_events table created");
    Ok(())
}

pub(super) fn migrate_v41(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v41: Creating operational context tables...");

    conn.execute_batch(
        r#"
        -- Observed operational facts from agents, commands, and tools.
        -- Raw artifact fields are optional references/payloads; this schema
        -- does not imply raw artifact retention.
        CREATE TABLE IF NOT EXISTS context_events (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_id             TEXT,
            workspace_path_hash TEXT,
            git_branch          TEXT,
            worktree_name       TEXT,
            commit_hash         TEXT,
            session_id          TEXT NOT NULL CHECK (length(session_id) > 0),
            task_id             TEXT,
            agent_id            TEXT,
            source              TEXT NOT NULL CHECK (length(source) > 0),
            event_type          TEXT NOT NULL CHECK (length(event_type) > 0),
            command_name        TEXT,
            tool_name           TEXT,
            cwd                 TEXT,
            exit_code           INTEGER,
            started_at          TEXT NOT NULL,
            finished_at         TEXT,
            redaction_status    TEXT NOT NULL DEFAULT 'unknown'
                                      CHECK (length(redaction_status) > 0),
            retention_policy    TEXT NOT NULL DEFAULT 'default'
                                      CHECK (length(retention_policy) > 0),
            raw_artifact_id     TEXT,
            raw_payload         TEXT,
            metadata            TEXT NOT NULL DEFAULT '{}',
            created_at          TEXT NOT NULL,
            CHECK (
                (repo_id IS NOT NULL AND length(repo_id) > 0)
                OR (workspace_path_hash IS NOT NULL AND length(workspace_path_hash) > 0)
            ),
            CHECK (
                lower(event_type) <> 'command'
                OR (command_name IS NOT NULL AND length(command_name) > 0 AND exit_code IS NOT NULL)
            ),
            CHECK (
                lower(event_type) <> 'tool'
                OR (tool_name IS NOT NULL AND length(tool_name) > 0)
            )
        );

        CREATE INDEX IF NOT EXISTS idx_context_events_scope_time
            ON context_events(repo_id, workspace_path_hash, started_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_events_session
            ON context_events(session_id, started_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_events_task
            ON context_events(task_id, started_at DESC)
            WHERE task_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_events_agent
            ON context_events(agent_id, started_at DESC)
            WHERE agent_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_events_source
            ON context_events(source, started_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_events_type
            ON context_events(event_type, started_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_events_command
            ON context_events(command_name, started_at DESC)
            WHERE command_name IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_events_tool
            ON context_events(tool_name, started_at DESC)
            WHERE tool_name IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_events_commit
            ON context_events(commit_hash)
            WHERE commit_hash IS NOT NULL;

        -- Derived operational summaries. Every row points back to a source
        -- event; optional source_artifact_id is a provenance pointer, not a
        -- raw retention guarantee.
        CREATE TABLE IF NOT EXISTS context_summaries (
            id                 INTEGER PRIMARY KEY AUTOINCREMENT,
            source_event_id    INTEGER NOT NULL
                                   REFERENCES context_events(id) ON DELETE CASCADE,
            source_artifact_id TEXT,
            reducer_name       TEXT NOT NULL CHECK (length(reducer_name) > 0),
            reducer_version    TEXT NOT NULL CHECK (length(reducer_version) > 0),
            lossy              INTEGER NOT NULL DEFAULT 1 CHECK (lossy IN (0, 1)),
            confidence         REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
            summary            TEXT NOT NULL CHECK (length(summary) > 0),
            structured_facts   TEXT NOT NULL DEFAULT '{}',
            warnings           TEXT NOT NULL DEFAULT '[]',
            tokens_raw_est     INTEGER CHECK (tokens_raw_est IS NULL OR tokens_raw_est >= 0),
            tokens_compact_est INTEGER CHECK (tokens_compact_est IS NULL OR tokens_compact_est >= 0),
            created_at         TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_context_summaries_source_event
            ON context_summaries(source_event_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_summaries_artifact
            ON context_summaries(source_artifact_id)
            WHERE source_artifact_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_summaries_reducer
            ON context_summaries(reducer_name, reducer_version, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_summaries_created_at
            ON context_summaries(created_at DESC);

        INSERT INTO schema_version (version) VALUES (41);
        "#,
    )?;

    tracing::info!("Migration v41 complete: operational context tables created");
    Ok(())
}

pub(super) fn migrate_v42(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v42: Creating operational context artifact tables...");

    conn.execute_batch(
        r#"
        -- Policy-controlled raw artifacts for operational context.
        -- Metadata is queryable, but raw_content is retrieved only through an
        -- explicit artifact-id path that enforces retention, TTL, staleness,
        -- access policy, and audit logging.
        CREATE TABLE IF NOT EXISTS context_artifacts (
            id                  TEXT PRIMARY KEY CHECK (length(id) > 0),
            source_event_id     INTEGER
                                    REFERENCES context_events(id) ON DELETE SET NULL,
            repo_id             TEXT,
            workspace_path_hash TEXT,
            session_id          TEXT,
            task_id             TEXT,
            agent_id            TEXT,
            kind                TEXT NOT NULL CHECK (length(kind) > 0),
            label               TEXT,
            uri                 TEXT,
            media_type          TEXT,
            content_sha256      TEXT,
            byte_len            INTEGER CHECK (byte_len IS NULL OR byte_len >= 0),
            redaction_status    TEXT NOT NULL DEFAULT 'not_required'
                                    CHECK (
                                        redaction_status IN (
                                            'passed',
                                            'redacted',
                                            'not_required'
                                        )
                                    ),
            retention_policy    TEXT NOT NULL DEFAULT 'pointer_only'
                                    CHECK (length(retention_policy) > 0),
            access_policy       TEXT NOT NULL DEFAULT 'same_session'
                                    CHECK (
                                        access_policy IN (
                                            'same_session',
                                            'same_task',
                                            'same_agent',
                                            'repo',
                                            'public'
                                        )
                                    ),
            retain_raw          INTEGER NOT NULL DEFAULT 0 CHECK (retain_raw IN (0, 1)),
            raw_content         BLOB,
            stale_at            TEXT,
            expires_at          TEXT,
            metadata            TEXT NOT NULL DEFAULT '{}',
            created_at          TEXT NOT NULL,
            CHECK (retain_raw = 1 OR raw_content IS NULL),
            CHECK (raw_content IS NULL OR content_sha256 IS NOT NULL),
            CHECK (
                source_event_id IS NOT NULL
                OR (repo_id IS NOT NULL AND length(repo_id) > 0)
                OR (
                    workspace_path_hash IS NOT NULL
                    AND length(workspace_path_hash) > 0
                )
            )
        );

        CREATE INDEX IF NOT EXISTS idx_context_artifacts_event
            ON context_artifacts(source_event_id, created_at DESC)
            WHERE source_event_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_artifacts_scope_time
            ON context_artifacts(repo_id, workspace_path_hash, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_artifacts_session
            ON context_artifacts(session_id, created_at DESC)
            WHERE session_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_artifacts_task
            ON context_artifacts(task_id, created_at DESC)
            WHERE task_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_artifacts_hash
            ON context_artifacts(content_sha256)
            WHERE content_sha256 IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_artifacts_expiry
            ON context_artifacts(expires_at)
            WHERE expires_at IS NOT NULL;

        -- Durable access attempts for explicit raw retrieval. No FK is used so
        -- denied/not-found/deleted-artifact attempts remain auditable.
        CREATE TABLE IF NOT EXISTS context_artifact_access_log (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            artifact_id         TEXT NOT NULL CHECK (length(artifact_id) > 0),
            requester_agent_id  TEXT,
            session_id          TEXT,
            task_id             TEXT,
            repo_id             TEXT,
            workspace_path_hash TEXT,
            access_result       TEXT NOT NULL CHECK (length(access_result) > 0),
            reason              TEXT NOT NULL CHECK (length(reason) > 0),
            max_bytes           INTEGER CHECK (max_bytes IS NULL OR max_bytes >= 0),
            returned_bytes      INTEGER CHECK (returned_bytes IS NULL OR returned_bytes >= 0),
            truncated           INTEGER NOT NULL DEFAULT 0 CHECK (truncated IN (0, 1)),
            created_at          TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_context_artifact_access_artifact
            ON context_artifact_access_log(artifact_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_context_artifact_access_agent
            ON context_artifact_access_log(requester_agent_id, created_at DESC)
            WHERE requester_agent_id IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_context_artifact_access_result
            ON context_artifact_access_log(access_result, created_at DESC);

        INSERT INTO schema_version (version) VALUES (42);
        "#,
    )?;

    tracing::info!("Migration v42 complete: operational context artifact tables created");
    Ok(())
}

pub(super) fn migrate_v43(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v43: Creating memory_policy table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_policy (
            memory_id INTEGER PRIMARY KEY,
            salience_score REAL NOT NULL DEFAULT 0.5 CHECK (salience_score >= 0.0 AND salience_score <= 1.0),
            retention_score REAL NOT NULL DEFAULT 0.5 CHECK (retention_score >= 0.0 AND retention_score <= 1.0),
            retrieval_priority REAL NOT NULL DEFAULT 0.5 CHECK (retrieval_priority >= 0.0 AND retrieval_priority <= 1.0),
            last_reinforced_at TEXT,
            reinforcement_count INTEGER NOT NULL DEFAULT 0 CHECK (reinforcement_count >= 0),
            contradiction_count INTEGER NOT NULL DEFAULT 0 CHECK (contradiction_count >= 0),
            policy_version TEXT NOT NULL DEFAULT 'heuristic-v1',
            policy_reason TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(memory_id) REFERENCES memories(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_memory_policy_retrieval_priority
            ON memory_policy(retrieval_priority DESC);

        CREATE INDEX IF NOT EXISTS idx_memory_policy_retention_score
            ON memory_policy(retention_score ASC);

        INSERT INTO schema_version (version) VALUES (43);
        "#,
    )?;

    tracing::info!("Migration v43 complete: memory_policy table created");
    Ok(())
}

pub(super) fn migrate_v44(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v44: Creating dream snapshot review tables...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS dream_jobs (
            id TEXT PRIMARY KEY CHECK (length(id) > 0),
            workspace TEXT NOT NULL CHECK (length(workspace) > 0),
            status TEXT NOT NULL
                CHECK (status IN (
                    'pending',
                    'running',
                    'completed',
                    'failed',
                    'canceled',
                    'archived'
                )),
            instructions TEXT,
            model_profile TEXT NOT NULL DEFAULT 'deterministic-local-v1'
                CHECK (length(model_profile) > 0),
            input_summary_json TEXT NOT NULL DEFAULT '{}',
            output_summary_json TEXT NOT NULL DEFAULT '{}',
            error_json TEXT,
            created_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT,
            archived_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_dream_jobs_workspace_status
            ON dream_jobs(workspace, status, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_dream_jobs_created
            ON dream_jobs(created_at DESC);

        CREATE TABLE IF NOT EXISTS dream_candidates (
            id TEXT PRIMARY KEY CHECK (length(id) > 0),
            job_id TEXT NOT NULL
                REFERENCES dream_jobs(id) ON DELETE CASCADE,
            workspace TEXT NOT NULL CHECK (length(workspace) > 0),
            kind TEXT NOT NULL
                CHECK (kind IN (
                    'summary',
                    'preference',
                    'constraint',
                    'project_state',
                    'stale_fact',
                    'contradiction',
                    'merge',
                    'promotion',
                    'decay',
                    'temporal_update'
                )),
            proposed_action TEXT NOT NULL
                CHECK (proposed_action IN (
                    'create',
                    'update',
                    'merge',
                    'supersede',
                    'expire',
                    'promote',
                    'demote',
                    'ignore'
                )),
            review_state TEXT NOT NULL DEFAULT 'pending'
                CHECK (review_state IN (
                    'pending',
                    'accepted',
                    'edited',
                    'rejected',
                    'applied',
                    'archived'
                )),
            confidence REAL NOT NULL
                CHECK (confidence >= 0.0 AND confidence <= 1.0),
            freshness_state TEXT NOT NULL DEFAULT 'unknown'
                CHECK (freshness_state IN (
                    'current',
                    'stale',
                    'future_due',
                    'expired',
                    'conflicted',
                    'unknown'
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

        CREATE INDEX IF NOT EXISTS idx_dream_candidates_job
            ON dream_candidates(job_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_dream_candidates_workspace_review
            ON dream_candidates(workspace, review_state, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_dream_candidates_kind
            ON dream_candidates(kind, freshness_state, created_at DESC);

        CREATE TABLE IF NOT EXISTS dream_candidate_sources (
            candidate_id TEXT NOT NULL
                REFERENCES dream_candidates(id) ON DELETE CASCADE,
            source_type TEXT NOT NULL CHECK (length(source_type) > 0),
            source_id TEXT NOT NULL CHECK (length(source_id) > 0),
            source_ref TEXT,
            evidence_json TEXT NOT NULL DEFAULT '{}',
            PRIMARY KEY (candidate_id, source_type, source_id)
        );

        CREATE INDEX IF NOT EXISTS idx_dream_candidate_sources_source
            ON dream_candidate_sources(source_type, source_id);

        INSERT INTO schema_version (version) VALUES (44);
        "#,
    )?;

    tracing::info!("Migration v44 complete: dream snapshot review tables created");
    Ok(())
}

pub(super) fn migrate_v45(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v45: Allowing agent writeback dream candidates...");

    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;

        CREATE TABLE dream_candidates_new (
            id TEXT PRIMARY KEY CHECK (length(id) > 0),
            job_id TEXT NOT NULL
                REFERENCES dream_jobs(id) ON DELETE CASCADE,
            workspace TEXT NOT NULL CHECK (length(workspace) > 0),
            kind TEXT NOT NULL
                CHECK (kind IN (
                    'summary',
                    'preference',
                    'constraint',
                    'project_state',
                    'stale_fact',
                    'contradiction',
                    'merge',
                    'promotion',
                    'decay',
                    'temporal_update',
                    'agent_writeback'
                )),
            proposed_action TEXT NOT NULL
                CHECK (proposed_action IN (
                    'create',
                    'update',
                    'merge',
                    'supersede',
                    'expire',
                    'promote',
                    'demote',
                    'ignore'
                )),
            review_state TEXT NOT NULL DEFAULT 'pending'
                CHECK (review_state IN (
                    'pending',
                    'accepted',
                    'edited',
                    'rejected',
                    'applied',
                    'archived'
                )),
            confidence REAL NOT NULL
                CHECK (confidence >= 0.0 AND confidence <= 1.0),
            freshness_state TEXT NOT NULL DEFAULT 'unknown'
                CHECK (freshness_state IN (
                    'current',
                    'stale',
                    'future_due',
                    'expired',
                    'conflicted',
                    'unknown'
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

        INSERT INTO dream_candidates_new (
            id, job_id, workspace, kind, proposed_action, review_state,
            confidence, freshness_state, content_preview, proposed_content,
            reason_codes, policy_explanation_json, metadata_json,
            application_result_json, created_at, reviewed_at, applied_at
        )
        SELECT
            id, job_id, workspace, kind, proposed_action, review_state,
            confidence, freshness_state, content_preview, proposed_content,
            reason_codes, policy_explanation_json, metadata_json,
            application_result_json, created_at, reviewed_at, applied_at
        FROM dream_candidates;

        DROP TABLE dream_candidates;
        ALTER TABLE dream_candidates_new RENAME TO dream_candidates;

        CREATE INDEX IF NOT EXISTS idx_dream_candidates_job
            ON dream_candidates(job_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_dream_candidates_workspace_review
            ON dream_candidates(workspace, review_state, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_dream_candidates_kind
            ON dream_candidates(kind, freshness_state, created_at DESC);

        PRAGMA foreign_keys = ON;

        INSERT INTO schema_version (version) VALUES (45);
        "#,
    )?;

    tracing::info!("Migration v45 complete: agent writeback dream candidates enabled");
    Ok(())
}

pub(super) fn migrate_v46(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v46: Adding memory stability and reinforcements table...");

    // Add stability column to memories table if memories table exists
    let memories_table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memories'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if memories_table_exists {
        let column_exists: bool = conn
            .prepare("PRAGMA table_info(memories)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|col| col == "stability");

        if !column_exists {
            conn.execute(
                "ALTER TABLE memories ADD COLUMN stability REAL NOT NULL DEFAULT 1.0",
                [],
            )?;
        }
    }

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_reinforcements (
            memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
            reinforced_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_reinforcements_mem_time
            ON memory_reinforcements(memory_id, reinforced_at);

        INSERT INTO schema_version (version) VALUES (46);
        "#,
    )?;

    tracing::info!("Migration v46 complete: memory stability and reinforcements table created");
    Ok(())
}
