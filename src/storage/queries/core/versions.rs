use super::*;

/// Get memory versions
pub fn get_memory_versions(conn: &Connection, memory_id: i64) -> Result<Vec<MemoryVersion>> {
    let mut stmt = conn.prepare_cached(
        "SELECT version, content, tags, metadata, created_at, created_by, change_summary
         FROM memory_versions WHERE memory_id = ? ORDER BY version DESC",
    )?;

    let versions: Vec<MemoryVersion> = stmt
        .query_map([memory_id], |row| {
            let tags_str: String = row.get("tags")?;
            let metadata_str: String = row.get("metadata")?;
            let created_at_str: String = row.get("created_at")?;

            Ok(MemoryVersion {
                version: row.get("version")?,
                content: row.get("content")?,
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                created_by: row.get("created_by")?,
                change_summary: row.get("change_summary")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(versions)
}
