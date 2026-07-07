use super::*;

/// Get storage statistics
pub fn get_stats(conn: &Connection) -> Result<StorageStats> {
    let total_memories: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL",
        [],
        |row| row.get(0),
    )?;

    let total_tags: i64 = conn.query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;

    let total_crossrefs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM crossrefs WHERE valid_to IS NULL",
        [],
        |row| row.get(0),
    )?;

    let total_versions: i64 =
        conn.query_row("SELECT COUNT(*) FROM memory_versions", [], |row| row.get(0))?;

    let _total_identities: i64 =
        conn.query_row("SELECT COUNT(*) FROM identities", [], |row| row.get(0))?;

    let _total_entities: i64 =
        conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;

    let db_size_bytes: i64 = conn.query_row(
        "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
        [],
        |row| row.get(0),
    )?;

    let _schema_version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let mut workspace_stmt = conn.prepare(
        "SELECT workspace, COUNT(*) FROM memories WHERE valid_to IS NULL GROUP BY workspace",
    )?;
    let workspaces: HashMap<String, i64> = workspace_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut type_stmt = conn.prepare(
        "SELECT memory_type, COUNT(*) FROM memories WHERE valid_to IS NULL GROUP BY memory_type",
    )?;
    let type_counts: HashMap<String, i64> = type_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut tier_stmt = conn.prepare(
        "SELECT COALESCE(tier, 'permanent'), COUNT(*) FROM memories GROUP BY COALESCE(tier, 'permanent')",
    )?;
    let tier_counts: HashMap<String, i64> = tier_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let memories_with_embeddings: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE has_embedding = 1 AND valid_to IS NULL",
        [],
        |row| row.get(0),
    )?;

    let memories_pending_embedding: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;

    let (last_sync, sync_pending): (Option<String>, i64) = conn.query_row(
        "SELECT last_sync, pending_changes FROM sync_state WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let schema_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(StorageStats {
        total_memories,
        total_tags,
        total_crossrefs,
        total_versions,
        total_identities: 0,
        total_entities: 0,
        db_size_bytes,
        memories_with_embeddings,
        memories_pending_embedding,
        last_sync: last_sync.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        sync_pending: sync_pending > 0,
        storage_mode: "sqlite".to_string(),
        schema_version,
        workspaces,
        type_counts,
        tier_counts,
    })
}
