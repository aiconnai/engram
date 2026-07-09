use super::*;

/// Find all potential duplicate memory pairs
///
/// Returns pairs of memories that are either:
/// 1. Exact duplicates (same content hash within same scope)
/// 2. High similarity (crossref score >= threshold within same scope)
///
/// Duplicates are scoped - memories in different scopes are not considered duplicates.
pub fn find_duplicates(conn: &Connection, threshold: f64) -> Result<Vec<DuplicatePair>> {
    find_duplicates_in_workspace(conn, threshold, None)
}

/// Find duplicate memories within a specific workspace (or all if None)
pub fn find_duplicates_in_workspace(
    conn: &Connection,
    threshold: f64,
    workspace: Option<&str>,
) -> Result<Vec<DuplicatePair>> {
    let now = Utc::now().to_rfc3339();
    let mut duplicates = Vec::new();

    // First, find exact hash duplicates (same content_hash within same scope AND workspace)
    let (hash_sql, hash_params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ws) = workspace
    {
        (
            "SELECT content_hash, scope_type, scope_id, GROUP_CONCAT(id) as ids
             FROM memories
             WHERE content_hash IS NOT NULL
               AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
               AND workspace = ?
             GROUP BY content_hash, scope_type, scope_id, workspace
             HAVING COUNT(*) > 1",
            vec![Box::new(now.clone()), Box::new(ws.to_string())],
        )
    } else {
        (
            "SELECT content_hash, scope_type, scope_id, GROUP_CONCAT(id) as ids
             FROM memories
             WHERE content_hash IS NOT NULL
               AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
             GROUP BY content_hash, scope_type, scope_id, workspace
             HAVING COUNT(*) > 1",
            vec![Box::new(now.clone())],
        )
    };

    let mut hash_stmt = conn.prepare_cached(hash_sql)?;
    let hash_rows = hash_stmt.query_map(
        rusqlite::params_from_iter(hash_params.iter().map(|p| p.as_ref())),
        |row| {
            let ids_str: String = row.get(3)?;
            Ok(ids_str)
        },
    )?;

    for ids_result in hash_rows {
        let ids_str = ids_result?;
        let ids: Vec<i64> = ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        // Create pairs from all IDs with same hash
        // Use get_memory_internal with track_access=false to avoid inflating access stats
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let memory_a = get_memory_internal(conn, ids[i], false)?;
                let memory_b = get_memory_internal(conn, ids[j], false)?;
                duplicates.push(DuplicatePair {
                    memory_a,
                    memory_b,
                    similarity_score: 1.0, // Exact match
                    match_type: DuplicateMatchType::ExactHash,
                });
            }
        }
    }

    // Second, find high-similarity pairs from crossrefs (within same scope AND workspace)
    let (sim_sql, sim_params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ws) = workspace {
        (
            "SELECT DISTINCT c.from_id, c.to_id, c.score
             FROM crossrefs c
             JOIN memories m1 ON c.from_id = m1.id
             JOIN memories m2 ON c.to_id = m2.id
             WHERE c.score >= ?
               AND m1.valid_to IS NULL
               AND m2.valid_to IS NULL
               AND (m1.expires_at IS NULL OR m1.expires_at > ?)
               AND (m2.expires_at IS NULL OR m2.expires_at > ?)
               AND c.from_id < c.to_id
               AND m1.scope_type = m2.scope_type
               AND (m1.scope_id = m2.scope_id OR (m1.scope_id IS NULL AND m2.scope_id IS NULL))
               AND m1.workspace = ?
               AND m2.workspace = ?
             ORDER BY c.score DESC",
            vec![
                Box::new(threshold),
                Box::new(now.clone()),
                Box::new(now.clone()),
                Box::new(ws.to_string()),
                Box::new(ws.to_string()),
            ],
        )
    } else {
        (
            "SELECT DISTINCT c.from_id, c.to_id, c.score
             FROM crossrefs c
             JOIN memories m1 ON c.from_id = m1.id
             JOIN memories m2 ON c.to_id = m2.id
             WHERE c.score >= ?
               AND m1.valid_to IS NULL
               AND m2.valid_to IS NULL
               AND (m1.expires_at IS NULL OR m1.expires_at > ?)
               AND (m2.expires_at IS NULL OR m2.expires_at > ?)
               AND c.from_id < c.to_id
               AND m1.scope_type = m2.scope_type
               AND (m1.scope_id = m2.scope_id OR (m1.scope_id IS NULL AND m2.scope_id IS NULL))
               AND m1.workspace = m2.workspace
             ORDER BY c.score DESC",
            vec![
                Box::new(threshold),
                Box::new(now.clone()),
                Box::new(now.clone()),
            ],
        )
    };

    let mut sim_stmt = conn.prepare_cached(sim_sql)?;
    let sim_rows = sim_stmt.query_map(
        rusqlite::params_from_iter(sim_params.iter().map(|p| p.as_ref())),
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        },
    )?;

    for row_result in sim_rows {
        let (from_id, to_id, score) = row_result?;

        // Skip if this pair was already found as exact hash match
        let already_found = duplicates.iter().any(|d| {
            (d.memory_a.id == from_id && d.memory_b.id == to_id)
                || (d.memory_a.id == to_id && d.memory_b.id == from_id)
        });

        if !already_found {
            // Use get_memory_internal with track_access=false to avoid inflating access stats
            let memory_a = get_memory_internal(conn, from_id, false)?;
            let memory_b = get_memory_internal(conn, to_id, false)?;
            duplicates.push(DuplicatePair {
                memory_a,
                memory_b,
                similarity_score: score,
                match_type: DuplicateMatchType::HighSimilarity,
            });
        }
    }

    Ok(duplicates)
}
