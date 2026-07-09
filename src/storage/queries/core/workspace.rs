use super::*;

/// Move a memory to a different workspace.
///
/// # Arguments
/// - `id`: Memory ID
/// - `workspace`: New workspace name (will be normalized)
///
/// # Errors
/// - Returns `NotFound` if memory doesn't exist
/// - Returns `Validation` if workspace name is invalid
pub fn move_to_workspace(conn: &Connection, id: i64, workspace: &str) -> Result<Memory> {
    // Validate workspace exists (by checking the memory exists first)
    let _memory = get_memory_internal(conn, id, false)?;

    // Normalize the workspace name
    let normalized = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE memories SET workspace = ?, updated_at = ?, version = version + 1 WHERE id = ?",
        params![normalized, now, id],
    )?;

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": ["workspace"],
            "action": "move_to_workspace",
            "new_workspace": normalized,
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    tracing::info!(memory_id = id, workspace = %normalized, "Moved memory to workspace");

    get_memory_internal(conn, id, false)
}

/// List all workspaces with their statistics.
///
/// Returns computed stats for each workspace that has at least one memory.
/// Stats are computed on-demand (not cached at the database level).
pub fn list_workspaces(conn: &Connection) -> Result<Vec<WorkspaceStats>> {
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare(
        r#"
        SELECT
            workspace,
            COUNT(*) as memory_count,
            SUM(CASE WHEN tier = 'permanent' THEN 1 ELSE 0 END) as permanent_count,
            SUM(CASE WHEN tier = 'daily' THEN 1 ELSE 0 END) as daily_count,
            MIN(created_at) as first_memory_at,
            MAX(created_at) as last_memory_at,
            AVG(importance) as avg_importance
        FROM memories
        WHERE valid_to IS NULL AND (expires_at IS NULL OR expires_at > ?)
        GROUP BY workspace
        ORDER BY memory_count DESC
        "#,
    )?;

    let workspaces: Vec<WorkspaceStats> = stmt
        .query_map(params![now], |row| {
            let workspace: String = row.get(0)?;
            let memory_count: i64 = row.get(1)?;
            let permanent_count: i64 = row.get(2)?;
            let daily_count: i64 = row.get(3)?;
            let first_memory_at: Option<String> = row.get(4)?;
            let last_memory_at: Option<String> = row.get(5)?;
            let avg_importance: Option<f64> = row.get(6)?;

            Ok(WorkspaceStats {
                workspace,
                memory_count,
                permanent_count,
                daily_count,
                first_memory_at: first_memory_at.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                last_memory_at: last_memory_at.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                top_tags: vec![], // Loaded separately if needed
                avg_importance: avg_importance.map(|v| v as f32),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(workspaces)
}

/// Get statistics for a specific workspace.
pub fn get_workspace_stats(conn: &Connection, workspace: &str) -> Result<WorkspaceStats> {
    let normalized = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    let now = Utc::now().to_rfc3339();

    let stats = conn
        .query_row(
            r#"
        SELECT
            workspace,
            COUNT(*) as memory_count,
            SUM(CASE WHEN tier = 'permanent' THEN 1 ELSE 0 END) as permanent_count,
            SUM(CASE WHEN tier = 'daily' THEN 1 ELSE 0 END) as daily_count,
            MIN(created_at) as first_memory_at,
            MAX(created_at) as last_memory_at,
            AVG(importance) as avg_importance
        FROM memories
        WHERE workspace = ? AND valid_to IS NULL AND (expires_at IS NULL OR expires_at > ?)
        GROUP BY workspace
        "#,
            params![normalized, now],
            |row| {
                let workspace: String = row.get(0)?;
                let memory_count: i64 = row.get(1)?;
                let permanent_count: i64 = row.get(2)?;
                let daily_count: i64 = row.get(3)?;
                let first_memory_at: Option<String> = row.get(4)?;
                let last_memory_at: Option<String> = row.get(5)?;
                let avg_importance: Option<f64> = row.get(6)?;

                Ok(WorkspaceStats {
                    workspace,
                    memory_count,
                    permanent_count,
                    daily_count,
                    first_memory_at: first_memory_at.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    last_memory_at: last_memory_at.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    top_tags: vec![],
                    avg_importance: avg_importance.map(|v| v as f32),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                EngramError::NotFound(0) // Workspace doesn't exist
            }
            _ => EngramError::Database(e),
        })?;

    Ok(stats)
}

/// Delete a workspace by moving all its memories to the default workspace or deleting them.
///
/// # Arguments
/// - `workspace`: Workspace to delete
/// - `move_to_default`: If true, moves memories to "default" workspace. If false, deletes them.
///
/// # Returns
/// Number of memories affected.
pub fn delete_workspace(conn: &Connection, workspace: &str, move_to_default: bool) -> Result<i64> {
    let normalized = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    if normalized == "default" {
        return Err(EngramError::InvalidInput(
            "Cannot delete the default workspace".to_string(),
        ));
    }

    let now = Utc::now().to_rfc3339();

    // First, get the IDs of all affected memories so we can record individual events
    let affected_ids: Vec<i64> = {
        let mut stmt =
            conn.prepare("SELECT id FROM memories WHERE workspace = ? AND valid_to IS NULL")?;
        let rows = stmt.query_map(params![&normalized], |row| row.get(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };

    let affected = affected_ids.len() as i64;

    if affected > 0 {
        if move_to_default {
            // Move all memories to the default workspace
            conn.execute(
                "UPDATE memories SET workspace = 'default', updated_at = ?, version = version + 1 WHERE workspace = ? AND valid_to IS NULL",
                params![&now, &normalized],
            )?;
        } else {
            // Soft delete all memories in the workspace
            conn.execute(
                "UPDATE memories SET valid_to = ? WHERE workspace = ? AND valid_to IS NULL",
                params![&now, &normalized],
            )?;
        }

        // Record individual events for each affected memory (for proper sync delta tracking)
        let event_type = if move_to_default {
            MemoryEventType::Updated
        } else {
            MemoryEventType::Deleted
        };

        for memory_id in &affected_ids {
            record_event(
                conn,
                event_type.clone(),
                Some(*memory_id),
                None,
                serde_json::json!({
                    "action": "delete_workspace",
                    "workspace": normalized,
                    "move_to_default": move_to_default,
                }),
            )?;
        }
    }

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + ?, version = (SELECT COALESCE(MAX(id), 0) FROM memory_events) WHERE id = 1",
        params![affected],
    )?;

    tracing::info!(
        workspace = %normalized,
        move_to_default,
        affected,
        "Deleted workspace"
    );

    Ok(affected)
}
