use rusqlite::Connection;

use crate::error::Result;

/// Schema v16: Retention policies per workspace
pub(super) fn migrate_v16(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v16: Adding retention policies table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS retention_policies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace TEXT NOT NULL,
            max_age_days INTEGER,
            max_memories INTEGER,
            compress_after_days INTEGER,
            compress_max_importance REAL DEFAULT 0.3,
            compress_min_access INTEGER DEFAULT 3,
            auto_delete_after_days INTEGER,
            exclude_types TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(workspace)
        );

        CREATE INDEX IF NOT EXISTS idx_retention_policies_workspace
            ON retention_policies(workspace);
        "#,
    )?;

    conn.execute("INSERT INTO schema_version (version) VALUES (16)", [])?;

    tracing::info!("Migration v16 complete: retention policies table added");

    Ok(())
}

/// Schema v17: Agent registry
///
/// Adds the `agents` table for tracking registered AI agents with their
/// capabilities, namespaces, heartbeat status, and lifecycle state.
pub(super) fn migrate_v17(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v17: Adding agent registry table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS agents (
            agent_id TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            capabilities TEXT NOT NULL DEFAULT '[]',
            namespaces TEXT NOT NULL DEFAULT '["default"]',
            last_heartbeat TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            metadata TEXT NOT NULL DEFAULT '{}',
            registered_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
        CREATE INDEX IF NOT EXISTS idx_agents_heartbeat ON agents(last_heartbeat);
        "#,
    )?;

    conn.execute("INSERT INTO schema_version (version) VALUES (17)", [])?;

    tracing::info!("Migration v17 complete: agent registry table added");

    Ok(())
}

/// Schema v18: Auto-links and memory clusters
///
/// Adds two tables for the emergent-graph feature:
/// - `auto_links`: auto-generated links (semantic, temporal, co-occurrence)
/// - `memory_clusters`: memory cluster assignments from community detection
pub(super) fn migrate_v18(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v18: Adding auto_links and memory_clusters tables...");

    conn.execute_batch(
        r#"
        -- Auto-generated links (semantic, temporal, co-occurrence)
        CREATE TABLE IF NOT EXISTS auto_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
            to_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
            link_type TEXT NOT NULL,
            score REAL NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT DEFAULT '{}'
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_auto_links_pair_type ON auto_links(from_id, to_id, link_type);
        CREATE INDEX IF NOT EXISTS idx_auto_links_type ON auto_links(link_type);
        CREATE INDEX IF NOT EXISTS idx_auto_links_from ON auto_links(from_id);
        CREATE INDEX IF NOT EXISTS idx_auto_links_to ON auto_links(to_id);

        -- Memory clusters from community detection
        CREATE TABLE IF NOT EXISTS memory_clusters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cluster_id INTEGER NOT NULL,
            memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
            algorithm TEXT NOT NULL,
            modularity REAL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_memory_clusters_cluster ON memory_clusters(cluster_id);
        CREATE INDEX IF NOT EXISTS idx_memory_clusters_memory ON memory_clusters(memory_id);
        CREATE INDEX IF NOT EXISTS idx_memory_clusters_algorithm ON memory_clusters(algorithm);
        "#,
    )?;

    conn.execute("INSERT INTO schema_version (version) VALUES (18)", [])?;

    tracing::info!("Migration v18 complete: auto_links and memory_clusters tables added");

    Ok(())
}

pub(super) fn migrate_v19(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v19: Adding media_assets table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS media_assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
            media_type TEXT NOT NULL,
            file_hash TEXT NOT NULL,
            file_path TEXT,
            file_size INTEGER,
            mime_type TEXT,
            duration_secs REAL,
            width INTEGER,
            height INTEGER,
            transcription TEXT,
            description TEXT,
            provider TEXT,
            model TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_media_assets_memory ON media_assets(memory_id);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_assets_hash ON media_assets(file_hash);
        CREATE INDEX IF NOT EXISTS idx_media_assets_type ON media_assets(media_type);
        "#,
    )?;

    conn.execute("INSERT INTO schema_version (version) VALUES (19)", [])?;

    tracing::info!("Migration v19 complete: media_assets table added");

    Ok(())
}

/// Schema v20: Embedding model tracking
///
/// Adds `embedding_model` column to `memories` so we can track which backend
/// generated a particular embedding and support multi-model environments.
pub(super) fn migrate_v20(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v20: Adding embedding_model column to memories...");

    conn.execute_batch(
        r#"
        ALTER TABLE memories ADD COLUMN embedding_model TEXT DEFAULT 'tfidf';

        INSERT INTO schema_version (version) VALUES (20);
        "#,
    )?;

    tracing::info!("Migration v20 complete: embedding_model column added");

    Ok(())
}

/// Schema v21: Facts table for SPO triples
///
/// Stores structured subject-predicate-object facts extracted from memories.
pub(super) fn migrate_v21(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v21: Adding facts table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS facts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            subject TEXT NOT NULL,
            predicate TEXT NOT NULL,
            object TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.8,
            source_memory_id INTEGER,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_facts_subject ON facts(subject);
        CREATE INDEX IF NOT EXISTS idx_facts_source ON facts(source_memory_id);

        INSERT INTO schema_version (version) VALUES (21);
        "#,
    )?;

    tracing::info!("Migration v21 complete: facts table added");

    Ok(())
}

/// Schema v22: Memory blocks + edit log
///
/// Adds Letta/MemGPT-inspired self-editing memory blocks with full revision
/// history in `block_edit_log`.
pub(super) fn migrate_v22(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v22: Adding memory_blocks and block_edit_log tables...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_blocks (
            name TEXT PRIMARY KEY,
            content TEXT NOT NULL DEFAULT '',
            version INTEGER NOT NULL DEFAULT 1,
            max_tokens INTEGER NOT NULL DEFAULT 4096,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS block_edit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            block_name TEXT NOT NULL,
            old_content TEXT NOT NULL,
            new_content TEXT NOT NULL,
            edit_reason TEXT NOT NULL DEFAULT '',
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            FOREIGN KEY (block_name) REFERENCES memory_blocks(name) ON DELETE CASCADE
        );

        INSERT INTO schema_version (version) VALUES (22);
        "#,
    )?;

    tracing::info!("Migration v22 complete: memory_blocks and block_edit_log tables added");

    Ok(())
}

/// Schema v23: Temporal knowledge graph edges
///
/// Adds `temporal_edges` with bi-temporal validity intervals for tracking how
/// relationships change over time.
pub(super) fn migrate_v23(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v23: Adding temporal_edges table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS temporal_edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_id INTEGER NOT NULL,
            to_id INTEGER NOT NULL,
            relation TEXT NOT NULL,
            properties TEXT NOT NULL DEFAULT '{}',
            valid_from TEXT NOT NULL,
            valid_to TEXT,
            confidence REAL NOT NULL DEFAULT 1.0,
            source TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_temporal_edges_from ON temporal_edges(from_id);
        CREATE INDEX IF NOT EXISTS idx_temporal_edges_to ON temporal_edges(to_id);
        CREATE INDEX IF NOT EXISTS idx_temporal_edges_valid ON temporal_edges(valid_from, valid_to);

        INSERT INTO schema_version (version) VALUES (23);
        "#,
    )?;

    tracing::info!("Migration v23 complete: temporal_edges table added");

    Ok(())
}

/// Schema v24: Scope path column on memories
///
/// Enables hierarchical scoping (Global > Org > User > Session > Agent) for
/// fine-grained multi-tenant memory isolation.
pub(super) fn migrate_v24(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v24: Adding scope_path column to memories...");

    conn.execute_batch(
        r#"
        ALTER TABLE memories ADD COLUMN scope_path TEXT DEFAULT 'global';

        CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope_path);

        INSERT INTO schema_version (version) VALUES (24);
        "#,
    )?;

    tracing::info!("Migration v24 complete: scope_path column added");

    Ok(())
}

pub(super) fn migrate_v25(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v25: Creating search_feedback table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS search_feedback (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query TEXT NOT NULL,
            query_embedding_hash TEXT,
            memory_id INTEGER NOT NULL,
            signal TEXT NOT NULL CHECK(signal IN ('useful', 'irrelevant')),
            rank_position INTEGER,
            original_score REAL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            workspace TEXT DEFAULT 'default'
        );

        CREATE INDEX IF NOT EXISTS idx_feedback_memory ON search_feedback(memory_id);
        CREATE INDEX IF NOT EXISTS idx_feedback_query ON search_feedback(query);
        CREATE INDEX IF NOT EXISTS idx_feedback_workspace ON search_feedback(workspace);

        INSERT INTO schema_version (version) VALUES (25);
        "#,
    )?;

    tracing::info!("Migration v25 complete: search_feedback table created");

    Ok(())
}
