use super::*;

/// Create a cross-reference between memories
pub fn create_crossref(conn: &Connection, input: &CreateCrossRefInput) -> Result<CrossReference> {
    let now = Utc::now().to_rfc3339();

    // Verify both memories exist
    let _ = get_memory_internal(conn, input.from_id, false)?;
    let _ = get_memory_internal(conn, input.to_id, false)?;

    let strength = input.strength.unwrap_or(1.0);

    conn.execute(
        "INSERT INTO crossrefs (from_id, to_id, edge_type, score, strength, source, source_context, pinned, created_at, valid_from)
         VALUES (?, ?, ?, 1.0, ?, 'manual', ?, ?, ?, ?)
         ON CONFLICT(from_id, to_id, edge_type) DO UPDATE SET
            strength = excluded.strength,
            source_context = COALESCE(excluded.source_context, crossrefs.source_context),
            pinned = excluded.pinned",
        params![
            input.from_id,
            input.to_id,
            input.edge_type.as_str(),
            strength,
            input.source_context,
            input.pinned,
            now,
            now,
        ],
    )?;

    get_crossref(conn, input.from_id, input.to_id, input.edge_type)
}

/// Get a cross-reference
pub fn get_crossref(
    conn: &Connection,
    from_id: i64,
    to_id: i64,
    edge_type: EdgeType,
) -> Result<CrossReference> {
    let mut stmt = conn.prepare_cached(
        "SELECT from_id, to_id, edge_type, score, confidence, strength, source,
                source_context, created_at, valid_from, valid_to, pinned, metadata
         FROM crossrefs
         WHERE from_id = ? AND to_id = ? AND edge_type = ? AND valid_to IS NULL",
    )?;

    let crossref = stmt.query_row(params![from_id, to_id, edge_type.as_str()], |row| {
        let edge_type_str: String = row.get("edge_type")?;
        let source_str: String = row.get("source")?;
        let created_at_str: String = row.get("created_at")?;
        let valid_from_str: String = row.get("valid_from")?;
        let valid_to_str: Option<String> = row.get("valid_to")?;
        let metadata_str: String = row.get("metadata")?;

        Ok(CrossReference {
            from_id: row.get("from_id")?,
            to_id: row.get("to_id")?,
            edge_type: edge_type_str.parse().unwrap_or(EdgeType::RelatedTo),
            score: row.get("score")?,
            confidence: row.get("confidence")?,
            strength: row.get("strength")?,
            source: match source_str.as_str() {
                "manual" => RelationSource::Manual,
                "llm" => RelationSource::Llm,
                _ => RelationSource::Auto,
            },
            source_context: row.get("source_context")?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            valid_from: DateTime::parse_from_rfc3339(&valid_from_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            valid_to: valid_to_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            pinned: row.get::<_, i32>("pinned")? != 0,
            metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
        })
    })?;

    Ok(crossref)
}

/// Get all cross-references for a memory
pub fn get_related(conn: &Connection, memory_id: i64) -> Result<Vec<CrossReference>> {
    let mut stmt = conn.prepare_cached(
        "SELECT from_id, to_id, edge_type, score, confidence, strength, source,
                source_context, created_at, valid_from, valid_to, pinned, metadata
         FROM crossrefs
         WHERE (from_id = ? OR to_id = ?) AND valid_to IS NULL
         ORDER BY score DESC",
    )?;

    let crossrefs: Vec<CrossReference> = stmt
        .query_map(params![memory_id, memory_id], |row| {
            let edge_type_str: String = row.get("edge_type")?;
            let source_str: String = row.get("source")?;
            let created_at_str: String = row.get("created_at")?;
            let valid_from_str: String = row.get("valid_from")?;
            let valid_to_str: Option<String> = row.get("valid_to")?;
            let metadata_str: String = row.get("metadata")?;

            Ok(CrossReference {
                from_id: row.get("from_id")?,
                to_id: row.get("to_id")?,
                edge_type: edge_type_str.parse().unwrap_or(EdgeType::RelatedTo),
                score: row.get("score")?,
                confidence: row.get("confidence")?,
                strength: row.get("strength")?,
                source: match source_str.as_str() {
                    "manual" => RelationSource::Manual,
                    "llm" => RelationSource::Llm,
                    _ => RelationSource::Auto,
                },
                source_context: row.get("source_context")?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                valid_from: DateTime::parse_from_rfc3339(&valid_from_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                valid_to: valid_to_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                pinned: row.get::<_, i32>("pinned")? != 0,
                metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(crossrefs)
}

/// Delete a cross-reference (soft delete)
pub fn delete_crossref(
    conn: &Connection,
    from_id: i64,
    to_id: i64,
    edge_type: EdgeType,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    let affected = conn.execute(
        "UPDATE crossrefs SET valid_to = ?
         WHERE from_id = ? AND to_id = ? AND edge_type = ? AND valid_to IS NULL",
        params![now, from_id, to_id, edge_type.as_str()],
    )?;

    if affected == 0 {
        return Err(EngramError::NotFound(from_id));
    }

    Ok(())
}
