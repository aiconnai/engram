use super::*;

/// Query episodic memories ordered by event_time within a time range.
pub fn get_episodic_timeline(
    conn: &Connection,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    workspace: Option<&str>,
    tags: Option<&[String]>,
    limit: i64,
) -> Result<Vec<Memory>> {
    let now = Utc::now().to_rfc3339();

    let mut sql = String::from(
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url,
                m.event_time, m.event_duration_seconds, m.trigger_pattern,
                m.procedure_success_count, m.procedure_failure_count, m.summary_of_id,
                m.lifecycle_state
         FROM memories m",
    );

    let mut conditions = vec![
        "m.valid_to IS NULL".to_string(),
        "(m.expires_at IS NULL OR m.expires_at > ?)".to_string(),
        "m.memory_type = 'episodic'".to_string(),
        "m.event_time IS NOT NULL".to_string(),
    ];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    if let Some(start) = start_time {
        conditions.push("m.event_time >= ?".to_string());
        params.push(Box::new(start.to_rfc3339()));
    }

    if let Some(end) = end_time {
        conditions.push("m.event_time <= ?".to_string());
        params.push(Box::new(end.to_rfc3339()));
    }

    if let Some(ws) = workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(ws.to_string()));
    }

    if let Some(tag_list) = tags {
        if !tag_list.is_empty() {
            sql.push_str(
                " JOIN memory_tags mt ON m.id = mt.memory_id
                  JOIN tags t ON mt.tag_id = t.id",
            );
            let placeholders: Vec<String> = tag_list.iter().map(|_| "?".to_string()).collect();
            conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
            for tag in tag_list {
                params.push(Box::new(tag.clone()));
            }
        }
    }

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));
    sql.push_str(" ORDER BY m.event_time ASC");
    sql.push_str(&format!(" LIMIT {}", limit));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}

/// Query procedural memories, optionally filtered by trigger pattern and success rate.
pub fn get_procedural_memories(
    conn: &Connection,
    trigger_pattern: Option<&str>,
    workspace: Option<&str>,
    min_success_rate: Option<f32>,
    limit: i64,
) -> Result<Vec<Memory>> {
    let now = Utc::now().to_rfc3339();

    let sql_base = "SELECT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url,
                m.event_time, m.event_duration_seconds, m.trigger_pattern,
                m.procedure_success_count, m.procedure_failure_count, m.summary_of_id,
                m.lifecycle_state
         FROM memories m";

    let mut conditions = vec![
        "m.valid_to IS NULL".to_string(),
        "(m.expires_at IS NULL OR m.expires_at > ?)".to_string(),
        "m.memory_type = 'procedural'".to_string(),
    ];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    if let Some(pattern) = trigger_pattern {
        conditions.push("m.trigger_pattern LIKE ?".to_string());
        params.push(Box::new(format!("%{}%", pattern)));
    }

    if let Some(ws) = workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(ws.to_string()));
    }

    if let Some(min_rate) = min_success_rate {
        // Filter: success / (success + failure) >= min_rate
        // Only apply when there's at least one execution
        conditions.push("(m.procedure_success_count + m.procedure_failure_count) > 0".to_string());
        conditions.push(
            "CAST(m.procedure_success_count AS REAL) / (m.procedure_success_count + m.procedure_failure_count) >= ?"
                .to_string(),
        );
        params.push(Box::new(min_rate as f64));
    }

    let sql = format!(
        "{} WHERE {} ORDER BY m.procedure_success_count DESC LIMIT {}",
        sql_base,
        conditions.join(" AND "),
        limit
    );

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}

/// Record a success or failure outcome for a procedural memory.
pub fn record_procedure_outcome(
    conn: &Connection,
    memory_id: i64,
    success: bool,
) -> Result<Memory> {
    let column = if success {
        "procedure_success_count"
    } else {
        "procedure_failure_count"
    };

    let now = Utc::now().to_rfc3339();

    // Verify the memory exists and is procedural
    let memory_type: String = conn
        .query_row(
            "SELECT memory_type FROM memories WHERE id = ? AND valid_to IS NULL",
            params![memory_id],
            |row| row.get(0),
        )
        .map_err(|_| EngramError::NotFound(memory_id))?;

    if memory_type != "procedural" {
        return Err(EngramError::InvalidInput(format!(
            "Memory {} is type '{}', not 'procedural'",
            memory_id, memory_type
        )));
    }

    conn.execute(
        &format!(
            "UPDATE memories SET {} = {} + 1, updated_at = ? WHERE id = ?",
            column, column
        ),
        params![now, memory_id],
    )?;

    get_memory(conn, memory_id)
}
