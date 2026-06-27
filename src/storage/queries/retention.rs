use super::*;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// A per-workspace retention policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: i64,
    pub workspace: String,
    pub max_age_days: Option<i64>,
    pub max_memories: Option<i64>,
    pub compress_after_days: Option<i64>,
    pub compress_max_importance: f32,
    pub compress_min_access: i32,
    pub auto_delete_after_days: Option<i64>,
    pub exclude_types: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Get retention policy for a workspace.
pub fn get_retention_policy(conn: &Connection, workspace: &str) -> Result<Option<RetentionPolicy>> {
    let workspace = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    conn.query_row(
        "SELECT id, workspace, max_age_days, max_memories, compress_after_days,
                compress_max_importance, compress_min_access, auto_delete_after_days,
                exclude_types, created_at, updated_at
         FROM retention_policies WHERE workspace = ?",
        params![workspace],
        |row| {
            let exclude_str: Option<String> = row.get(8)?;
            Ok(RetentionPolicy {
                id: row.get(0)?,
                workspace: row.get(1)?,
                max_age_days: row.get(2)?,
                max_memories: row.get(3)?,
                compress_after_days: row.get(4)?,
                compress_max_importance: row.get::<_, f32>(5).unwrap_or(0.3),
                compress_min_access: row.get::<_, i32>(6).unwrap_or(3),
                auto_delete_after_days: row.get(7)?,
                exclude_types: exclude_str
                    .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                    .unwrap_or_default(),
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
    .optional()
    .map_err(EngramError::from)
}

/// List all retention policies.
pub fn list_retention_policies(conn: &Connection) -> Result<Vec<RetentionPolicy>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace, max_age_days, max_memories, compress_after_days,
                compress_max_importance, compress_min_access, auto_delete_after_days,
                exclude_types, created_at, updated_at
         FROM retention_policies ORDER BY workspace",
    )?;

    let policies = stmt
        .query_map([], |row| {
            let exclude_str: Option<String> = row.get(8)?;
            Ok(RetentionPolicy {
                id: row.get(0)?,
                workspace: row.get(1)?,
                max_age_days: row.get(2)?,
                max_memories: row.get(3)?,
                compress_after_days: row.get(4)?,
                compress_max_importance: row.get::<_, f32>(5).unwrap_or(0.3),
                compress_min_access: row.get::<_, i32>(6).unwrap_or(3),
                auto_delete_after_days: row.get(7)?,
                exclude_types: exclude_str
                    .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                    .unwrap_or_default(),
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(policies)
}

/// Upsert a retention policy for a workspace.
pub fn set_retention_policy(
    conn: &Connection,
    workspace: &str,
    max_age_days: Option<i64>,
    max_memories: Option<i64>,
    compress_after_days: Option<i64>,
    compress_max_importance: Option<f32>,
    compress_min_access: Option<i32>,
    auto_delete_after_days: Option<i64>,
    exclude_types: Option<Vec<String>>,
) -> Result<RetentionPolicy> {
    let workspace = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    let now = Utc::now().to_rfc3339();
    let exclude_str = exclude_types.map(|v| v.join(","));

    conn.execute(
        "INSERT INTO retention_policies (workspace, max_age_days, max_memories, compress_after_days,
            compress_max_importance, compress_min_access, auto_delete_after_days, exclude_types,
            created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
         ON CONFLICT(workspace) DO UPDATE SET
            max_age_days = COALESCE(?2, max_age_days),
            max_memories = COALESCE(?3, max_memories),
            compress_after_days = COALESCE(?4, compress_after_days),
            compress_max_importance = COALESCE(?5, compress_max_importance),
            compress_min_access = COALESCE(?6, compress_min_access),
            auto_delete_after_days = COALESCE(?7, auto_delete_after_days),
            exclude_types = COALESCE(?8, exclude_types),
            updated_at = ?9",
        params![
            workspace,
            max_age_days,
            max_memories,
            compress_after_days,
            compress_max_importance.unwrap_or(0.3),
            compress_min_access.unwrap_or(3),
            auto_delete_after_days,
            exclude_str,
            now,
        ],
    )?;

    get_retention_policy(conn, &workspace)?.ok_or_else(|| EngramError::NotFound(0))
}

/// Delete a retention policy for a workspace.
pub fn delete_retention_policy(conn: &Connection, workspace: &str) -> Result<bool> {
    let workspace = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    let affected = conn.execute(
        "DELETE FROM retention_policies WHERE workspace = ?",
        params![workspace],
    )?;
    Ok(affected > 0)
}

/// Apply all retention policies. Returns total memories affected across all workspaces.
pub fn apply_retention_policies(conn: &Connection) -> Result<i64> {
    let policies = list_retention_policies(conn)?;
    let mut total_affected = 0i64;

    for policy in &policies {
        if let Some(compress_days) = policy.compress_after_days {
            let compressed = compress_old_memories(
                conn,
                compress_days,
                policy.compress_max_importance,
                policy.compress_min_access,
                100,
            )?;
            total_affected += compressed;
        }

        // 2. Enforce max_memories limit per workspace
        if let Some(max_mem) = policy.max_memories {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM memories WHERE workspace = ? AND valid_to IS NULL
                     AND COALESCE(lifecycle_state, 'active') = 'active'",
                    params![policy.workspace],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if count > max_mem {
                // Archive excess (oldest, lowest importance first)
                let excess = count - max_mem;
                let archived = conn.execute(
                    "UPDATE memories SET lifecycle_state = 'archived'
                     WHERE id IN (
                        SELECT id FROM memories
                        WHERE workspace = ? AND valid_to IS NULL
                          AND COALESCE(lifecycle_state, 'active') = 'active'
                          AND memory_type NOT IN ('summary', 'checkpoint')
                        ORDER BY importance ASC, access_count ASC, created_at ASC
                        LIMIT ?
                     )",
                    params![policy.workspace, excess],
                )?;
                total_affected += archived as i64;
            }
        }

        // 3. Auto-delete very old archived memories
        if let Some(delete_days) = policy.auto_delete_after_days {
            let cutoff = (Utc::now() - chrono::Duration::days(delete_days)).to_rfc3339();
            let now = Utc::now().to_rfc3339();
            let deleted = conn.execute(
                "UPDATE memories SET valid_to = ?
                 WHERE workspace = ? AND valid_to IS NULL
                   AND lifecycle_state = 'archived'
                   AND created_at < ?",
                params![now, policy.workspace, cutoff],
            )?;
            total_affected += deleted as i64;
        }
    }

    Ok(total_affected)
}

/// Compress already-archived memories by creating summary rows.
/// Returns the number of memories compressed.
pub fn compress_old_memories(
    conn: &Connection,
    max_age_days: i64,
    max_importance: f32,
    min_access_count: i32,
    batch_limit: usize,
) -> Result<i64> {
    let cutoff = (Utc::now() - chrono::Duration::days(max_age_days)).to_rfc3339();
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare(
        "SELECT id, content, memory_type, importance, tags, workspace
         FROM (
            SELECT m.id, m.content, m.memory_type, m.importance, m.access_count, m.workspace,
                   COALESCE(m.lifecycle_state, 'active') as lifecycle_state,
                   (SELECT GROUP_CONCAT(t.name, ',') FROM memory_tags mt JOIN tags t ON mt.tag_id = t.id WHERE mt.memory_id = m.id) as tags
            FROM memories m
            WHERE m.valid_to IS NULL
              AND (m.expires_at IS NULL OR m.expires_at > ?1)
              AND m.created_at < ?2
              AND m.importance <= ?3
              AND m.access_count < ?4
              AND m.memory_type NOT IN ('summary', 'checkpoint')
              AND COALESCE(m.lifecycle_state, 'active') = 'archived'
              AND NOT EXISTS (
                  SELECT 1
                  FROM memories s
                  WHERE s.summary_of_id = m.id
                    AND s.memory_type = 'summary'
                    AND s.valid_to IS NULL
              )
            ORDER BY m.created_at ASC
            LIMIT ?5
         )",
    )?;

    let candidates: Vec<(i64, String, String, f32, Option<String>, String)> = stmt
        .query_map(
            params![
                now,
                cutoff,
                max_importance,
                min_access_count,
                batch_limit as i64
            ],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get::<_, String>(5)
                        .unwrap_or_else(|_| "default".to_string()),
                ))
            },
        )?
        .filter_map(|r| r.ok())
        .collect();

    let mut compressed = 0i64;

    for (id, content, memory_type, importance, tags_csv, workspace) in &candidates {
        // Create compressed summary
        let summary_text = if content.len() > 200 {
            let head: String = content.chars().take(120).collect();
            let tail: String = content
                .chars()
                .rev()
                .take(60)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            format!("{}...{}", head, tail)
        } else {
            content.clone()
        };

        let tags: Vec<String> = tags_csv
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let input = CreateMemoryInput {
            content: format!("[Compressed {}] {}", memory_type, summary_text),
            memory_type: MemoryType::Summary,
            importance: Some(*importance),
            tags,
            workspace: Some(workspace.clone()),
            tier: MemoryTier::Permanent,
            summary_of_id: Some(*id),
            ..Default::default()
        };

        if create_memory(conn, &input).is_ok() {
            compressed += 1;
        }
    }

    Ok(compressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn seed_old_memory(conn: &Connection, lifecycle_state: &str) -> Result<i64> {
        let memory = create_memory(
            conn,
            &CreateMemoryInput {
                content: format!("old {lifecycle_state} retention memory"),
                memory_type: MemoryType::Note,
                importance: Some(0.0),
                workspace: Some("default".to_string()),
                tier: MemoryTier::Permanent,
                ..Default::default()
            },
        )?;
        let old_ts = (Utc::now() - chrono::Duration::days(120)).to_rfc3339();
        conn.execute(
            "UPDATE memories
             SET created_at = ?1,
                 updated_at = ?1,
                 last_accessed_at = ?1,
                 access_count = 0,
                 lifecycle_state = ?2
             WHERE id = ?3",
            params![old_ts, lifecycle_state, memory.id],
        )?;
        Ok(memory.id)
    }

    fn summary_count_for(conn: &Connection, memory_id: i64) -> Result<i64> {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM memories
             WHERE summary_of_id = ?1 AND memory_type = 'summary' AND valid_to IS NULL",
            params![memory_id],
            |row| row.get(0),
        )?)
    }

    fn lifecycle_state(conn: &Connection, memory_id: i64) -> Result<String> {
        Ok(conn.query_row(
            "SELECT lifecycle_state FROM memories WHERE id = ?1",
            params![memory_id],
            |row| row.get(0),
        )?)
    }

    #[test]
    fn compress_old_memories_skips_active_rows() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let id = seed_old_memory(conn, "active")?;

                let compressed = compress_old_memories(conn, 90, 0.5, 5, 10)?;

                assert_eq!(compressed, 0);
                assert_eq!(summary_count_for(conn, id)?, 0);
                assert_eq!(lifecycle_state(conn, id)?, "active");
                Ok(())
            })
            .expect("compression should skip active rows");
    }

    #[test]
    fn compress_old_memories_summarizes_archived_rows_without_rewriting_lifecycle() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let id = seed_old_memory(conn, "archived")?;

                let compressed = compress_old_memories(conn, 90, 0.5, 5, 10)?;

                assert_eq!(compressed, 1);
                assert_eq!(summary_count_for(conn, id)?, 1);
                assert_eq!(lifecycle_state(conn, id)?, "archived");
                Ok(())
            })
            .expect("compression should summarize archived rows");
    }

    #[test]
    fn compress_old_memories_is_idempotent_for_archived_rows() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let id = seed_old_memory(conn, "archived")?;

                let first = compress_old_memories(conn, 90, 0.5, 5, 10)?;
                let second = compress_old_memories(conn, 90, 0.5, 5, 10)?;

                assert_eq!(first, 1);
                assert_eq!(second, 0);
                assert_eq!(summary_count_for(conn, id)?, 1);
                assert_eq!(lifecycle_state(conn, id)?, "archived");
                Ok(())
            })
            .expect("compression should be idempotent");
    }

    #[test]
    fn apply_retention_compression_only_summarizes_archived_rows() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let active_id = seed_old_memory(conn, "active")?;
                let archived_id = seed_old_memory(conn, "archived")?;
                set_retention_policy(
                    conn,
                    "default",
                    None,
                    None,
                    Some(90),
                    Some(0.5),
                    Some(5),
                    None,
                    None,
                )?;

                let affected = apply_retention_policies(conn)?;

                assert_eq!(affected, 1);
                assert_eq!(summary_count_for(conn, active_id)?, 0);
                assert_eq!(summary_count_for(conn, archived_id)?, 1);
                assert_eq!(lifecycle_state(conn, active_id)?, "active");
                assert_eq!(lifecycle_state(conn, archived_id)?, "archived");
                Ok(())
            })
            .expect("retention compression should be archived-only");
    }

    #[test]
    fn retention_auto_delete_still_soft_deletes_archived_rows() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let id = seed_old_memory(conn, "archived")?;
                set_retention_policy(
                    conn,
                    "default",
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(90),
                    None,
                )?;

                let affected = apply_retention_policies(conn)?;

                assert_eq!(affected, 1);
                let valid_to: Option<String> = conn.query_row(
                    "SELECT valid_to FROM memories WHERE id = ?1",
                    params![id],
                    |row| row.get(0),
                )?;
                assert!(valid_to.is_some(), "archived row should be soft-deleted");
                Ok(())
            })
            .expect("auto-delete should soft-delete archived rows");
    }
}
