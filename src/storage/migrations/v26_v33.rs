use rusqlite::Connection;

use crate::error::Result;

/// v26: Compression columns on memories table
pub(super) fn migrate_v26(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v26: Adding compression columns to memories...");

    conn.execute_batch(
        r#"
        ALTER TABLE memories ADD COLUMN compressed_content TEXT;
        ALTER TABLE memories ADD COLUMN compression_ratio REAL;
        ALTER TABLE memories ADD COLUMN compression_method TEXT;

        INSERT INTO schema_version (version) VALUES (26);
        "#,
    )?;

    tracing::info!("Migration v26 complete: compression columns added");

    Ok(())
}

/// v27: Consolidated memories table
pub(super) fn migrate_v27(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v27: Creating consolidated_memories table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS consolidated_memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_ids TEXT NOT NULL DEFAULT '[]',
            summary TEXT NOT NULL,
            strategy_used TEXT NOT NULL DEFAULT 'content_overlap',
            tokens_before INTEGER NOT NULL DEFAULT 0,
            tokens_after INTEGER NOT NULL DEFAULT 0,
            workspace TEXT DEFAULT 'default',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        INSERT INTO schema_version (version) VALUES (27);
        "#,
    )?;

    tracing::info!("Migration v27 complete: consolidated_memories table created");

    Ok(())
}

/// v28: Utility feedback + update log tables
pub(super) fn migrate_v28(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v28: Creating utility_feedback and update_log tables...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS utility_feedback (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id INTEGER NOT NULL,
            was_useful INTEGER NOT NULL DEFAULT 1,
            query TEXT,
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );
        CREATE INDEX IF NOT EXISTS idx_utility_memory ON utility_feedback(memory_id);

        CREATE TABLE IF NOT EXISTS update_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            old_content_hash TEXT,
            new_content_hash TEXT,
            reason TEXT NOT NULL DEFAULT '',
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        INSERT INTO schema_version (version) VALUES (28);
        "#,
    )?;

    tracing::info!("Migration v28 complete: utility_feedback and update_log tables created");

    Ok(())
}

/// v29: Sentiment columns + reflections + query_log tables
pub(super) fn migrate_v29(conn: &Connection) -> Result<()> {
    tracing::info!(
        "Migration v29: Adding sentiment columns and creating reflections/query_log tables..."
    );

    conn.execute_batch(
        r#"
        ALTER TABLE memories ADD COLUMN sentiment_score REAL;
        ALTER TABLE memories ADD COLUMN sentiment_label TEXT;

        CREATE TABLE IF NOT EXISTS reflections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            source_ids TEXT NOT NULL DEFAULT '[]',
            depth TEXT NOT NULL DEFAULT 'surface',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS query_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query TEXT NOT NULL,
            workspace TEXT DEFAULT 'default',
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        INSERT INTO schema_version (version) VALUES (29);
        "#,
    )?;

    tracing::info!("Migration v29 complete: sentiment columns, reflections and query_log created");

    Ok(())
}

/// v30: Coactivation edges + graph conflicts + garden log tables
pub(super) fn migrate_v30(conn: &Connection) -> Result<()> {
    tracing::info!(
        "Migration v30: Creating coactivation_edges, graph_conflicts, and garden_log tables..."
    );

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS coactivation_edges (
            from_id INTEGER NOT NULL,
            to_id INTEGER NOT NULL,
            strength REAL NOT NULL DEFAULT 0.1,
            coactivation_count INTEGER NOT NULL DEFAULT 1,
            last_coactivated TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            PRIMARY KEY (from_id, to_id)
        );

        CREATE TABLE IF NOT EXISTS graph_conflicts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conflict_type TEXT NOT NULL,
            edge_ids TEXT NOT NULL DEFAULT '[]',
            description TEXT NOT NULL DEFAULT '',
            severity TEXT NOT NULL DEFAULT 'medium',
            resolved_at TEXT,
            resolution_strategy TEXT
        );

        CREATE TABLE IF NOT EXISTS garden_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace TEXT DEFAULT 'default',
            actions TEXT NOT NULL DEFAULT '[]',
            memories_pruned INTEGER NOT NULL DEFAULT 0,
            memories_merged INTEGER NOT NULL DEFAULT 0,
            memories_archived INTEGER NOT NULL DEFAULT 0,
            tokens_freed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        INSERT INTO schema_version (version) VALUES (30);
        "#,
    )?;

    tracing::info!(
        "Migration v30 complete: coactivation_edges, graph_conflicts, garden_log created"
    );

    Ok(())
}

/// v31: Scope grants table for multi-agent memory sharing access control
pub(super) fn migrate_v31(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v31: Creating scope_grants table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS scope_grants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            agent_id TEXT NOT NULL,
            scope_path TEXT NOT NULL,
            permissions TEXT NOT NULL DEFAULT 'read',
            granted_by TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            UNIQUE(agent_id, scope_path)
        );

        CREATE INDEX IF NOT EXISTS idx_scope_grants_agent ON scope_grants(agent_id);
        CREATE INDEX IF NOT EXISTS idx_scope_grants_scope ON scope_grants(scope_path);

        INSERT INTO schema_version (version) VALUES (31);
        "#,
    )?;

    tracing::info!("Migration v31 complete: scope_grants table created");

    Ok(())
}

/// v32: Agent portability support for snapshots and attestation
pub(super) fn migrate_v32(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v32: Creating snapshot provenance and attestation tables...");

    conn.execute_batch(
        r#"
        -- Phase L: Agent Portability (v0.13.0)
        -- Snapshot provenance tracking
        ALTER TABLE memories ADD COLUMN snapshot_origin TEXT;
        ALTER TABLE memories ADD COLUMN snapshot_loaded_at TEXT;

        -- Knowledge attestation log
        CREATE TABLE IF NOT EXISTS attestation_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            document_hash TEXT NOT NULL,
            document_name TEXT NOT NULL,
            document_size INTEGER NOT NULL,
            ingested_at TEXT NOT NULL,
            agent_id TEXT,
            memory_ids TEXT NOT NULL,
            previous_hash TEXT NOT NULL,
            record_hash TEXT NOT NULL,
            signature TEXT,
            metadata TEXT DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_attestation_document_hash ON attestation_log(document_hash);
        CREATE INDEX IF NOT EXISTS idx_attestation_agent_id ON attestation_log(agent_id);
        CREATE INDEX IF NOT EXISTS idx_attestation_ingested_at ON attestation_log(ingested_at);

        INSERT INTO schema_version (version) VALUES (32);
        "#,
    )?;

    tracing::info!("Migration v32 complete: snapshot provenance and attestation tables created");

    Ok(())
}

/// v33: DuckDB CQRS Graph support with scope_path and graph_entities table
pub(super) fn migrate_v33(conn: &Connection) -> Result<()> {
    tracing::info!(
        "Migration v33: Adding scope_path to temporal_edges and creating graph_entities table..."
    );

    conn.execute_batch(
        r#"
        -- DuckDB CQRS Graph: scope_path for tenant isolation
        ALTER TABLE temporal_edges ADD COLUMN scope_path TEXT NOT NULL DEFAULT 'global';
        CREATE INDEX IF NOT EXISTS idx_temporal_edges_scope_path ON temporal_edges(scope_path);

        -- Graph entities table for DuckDB property graph vertex table
        CREATE TABLE IF NOT EXISTS graph_entities (
            id TEXT PRIMARY KEY,
            scope_path TEXT NOT NULL DEFAULT 'global',
            name TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            metadata TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );
        CREATE INDEX IF NOT EXISTS idx_graph_entities_scope ON graph_entities(scope_path);
        CREATE INDEX IF NOT EXISTS idx_graph_entities_type ON graph_entities(entity_type);

        INSERT INTO schema_version (version) VALUES (33);
        "#,
    )?;

    tracing::info!("Migration v33 complete: scope_path and graph_entities table created");

    Ok(())
}
