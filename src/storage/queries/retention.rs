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

fn parse_exclude_types(raw: Option<String>) -> Vec<String> {
    raw.map(|s| {
        s.split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect()
    })
    .unwrap_or_default()
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
                exclude_types: parse_exclude_types(exclude_str),
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
                exclude_types: parse_exclude_types(exclude_str),
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(policies)
}

/// Day fields must be in `1..=MAX_OFFSET_DAYS`, `max_memories` positive,
/// `compress_max_importance` finite in `0.0..=1.0`, `compress_min_access >= 0`,
/// and `exclude_types` valid `MemoryType` names.
#[allow(clippy::too_many_arguments)]
fn validate_policy_fields(
    max_age_days: Option<i64>,
    max_memories: Option<i64>,
    compress_after_days: Option<i64>,
    compress_max_importance: Option<f32>,
    compress_min_access: Option<i32>,
    auto_delete_after_days: Option<i64>,
    exclude_types: Option<&[String]>,
) -> Result<()> {
    let max_days = crate::storage::queries::MAX_OFFSET_DAYS;
    for (field, value) in [
        ("max_age_days", max_age_days),
        ("compress_after_days", compress_after_days),
        ("auto_delete_after_days", auto_delete_after_days),
    ] {
        if let Some(days) = value.filter(|days| !(1..=max_days).contains(days)) {
            return Err(EngramError::InvalidInput(format!(
                "{field} {days} is out of range (must be within 1..={max_days})"
            )));
        }
    }
    if let Some(count) = max_memories.filter(|count| *count <= 0) {
        return Err(EngramError::InvalidInput(format!(
            "max_memories {count} must be greater than 0"
        )));
    }
    if let Some(imp) =
        compress_max_importance.filter(|imp| !imp.is_finite() || !(0.0..=1.0).contains(imp))
    {
        return Err(EngramError::InvalidInput(format!(
            "compress_max_importance {imp} must be finite and within 0.0..=1.0"
        )));
    }
    if let Some(acc) = compress_min_access.filter(|acc| *acc < 0) {
        return Err(EngramError::InvalidInput(format!(
            "compress_min_access {acc} must be >= 0"
        )));
    }
    if let Some(types) = exclude_types {
        for raw in types {
            let normalized = raw.trim().to_lowercase();
            if normalized.is_empty() || normalized.parse::<MemoryType>().is_err() {
                return Err(EngramError::InvalidInput(format!(
                    "Invalid memory_type in exclude_types: '{raw}'"
                )));
            }
        }
    }
    Ok(())
}

/// Upsert a retention policy for a workspace.
#[allow(clippy::too_many_arguments)]
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
    validate_policy_fields(
        max_age_days,
        max_memories,
        compress_after_days,
        compress_max_importance,
        compress_min_access,
        auto_delete_after_days,
        exclude_types.as_deref(),
    )?;

    let now = Utc::now().to_rfc3339();
    let exclude_str = exclude_types.map(|v| {
        v.into_iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(",")
    });

    conn.execute(
        "INSERT INTO retention_policies (workspace, max_age_days, max_memories, compress_after_days,
            compress_max_importance, compress_min_access, auto_delete_after_days, exclude_types,
            created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, COALESCE(?5, 0.3), COALESCE(?6, 3), ?7, ?8, ?9, ?9)
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
            compress_max_importance,
            compress_min_access,
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

fn build_exclude_type_clause(
    column: &str,
    always_excluded: &[&str],
    exclude_types: &[String],
) -> (String, Vec<String>) {
    let mut values: Vec<String> = always_excluded.iter().map(|s| (*s).to_string()).collect();
    for raw in exclude_types {
        let norm = raw.trim().to_lowercase();
        if !norm.is_empty() && !values.contains(&norm) {
            values.push(norm);
        }
    }
    if values.is_empty() {
        return (String::new(), Vec::new());
    }
    let placeholders = std::iter::repeat_n("?", values.len())
        .collect::<Vec<_>>()
        .join(", ");
    (
        format!(" AND LOWER({column}) NOT IN ({placeholders})"),
        values,
    )
}

/// Apply all retention policies. Returns total memories affected across all workspaces.
pub fn apply_retention_policies(conn: &Connection) -> Result<i64> {
    let policies = list_retention_policies(conn)?;
    let mut total_affected = 0i64;

    for policy in &policies {
        // Rows stored before validation existed may be out of range; skip them
        // loudly instead of letting one workspace block every other one.
        if let Err(error) = validate_policy_fields(
            policy.max_age_days,
            policy.max_memories,
            policy.compress_after_days,
            Some(policy.compress_max_importance),
            Some(policy.compress_min_access),
            policy.auto_delete_after_days,
            Some(&policy.exclude_types),
        ) {
            tracing::warn!(
                workspace = %crate::observability::redact::opaque(&policy.workspace),
                error = %crate::observability::redact::redacted(&error),
                "skipping retention policy with out-of-range fields"
            );
            continue;
        }

        // 1. Hard age limit (max_age_days): soft-delete expired memories first so
        //    they are neither compressed nor counted against max_memories.
        if let Some(max_age) = policy.max_age_days {
            let cutoff =
                crate::storage::queries::cutoff_days_ago(Utc::now(), max_age)?.to_rfc3339();
            let now = Utc::now().to_rfc3339();
            let (exclude_sql, exclude_vals) =
                build_exclude_type_clause("memory_type", &[], &policy.exclude_types);
            let sql = format!(
                "UPDATE memories SET valid_to = ?
                 WHERE workspace = ? AND valid_to IS NULL
                   AND julianday(created_at) < julianday(?){exclude_sql}"
            );
            let mut bind_vals: Vec<rusqlite::types::Value> =
                vec![now.into(), policy.workspace.clone().into(), cutoff.into()];
            bind_vals.extend(exclude_vals.into_iter().map(Into::into));
            let deleted = conn.execute(&sql, rusqlite::params_from_iter(bind_vals))?;
            total_affected += deleted as i64;
        }

        // 2. Compress already-archived memories scoped to this workspace
        if let Some(compress_days) = policy.compress_after_days {
            let compressed = compress_old_memories_scoped(
                conn,
                Some(&policy.workspace),
                &policy.exclude_types,
                compress_days,
                policy.compress_max_importance,
                policy.compress_min_access,
                100,
            )?;
            total_affected += compressed;
        }

        // 3. Enforce max_memories limit per workspace (excluding summary, checkpoint, and exclude_types)
        if let Some(max_mem) = policy.max_memories {
            let (exclude_sql, exclude_vals) = build_exclude_type_clause(
                "memory_type",
                &["summary", "checkpoint"],
                &policy.exclude_types,
            );
            let count_sql = format!(
                "SELECT COUNT(*) FROM memories
                 WHERE workspace = ? AND valid_to IS NULL
                   AND COALESCE(lifecycle_state, 'active') = 'active'{exclude_sql}"
            );
            let mut count_params: Vec<rusqlite::types::Value> =
                vec![policy.workspace.clone().into()];
            count_params.extend(exclude_vals.iter().cloned().map(Into::into));
            let count: i64 = conn
                .query_row(
                    &count_sql,
                    rusqlite::params_from_iter(count_params),
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if count > max_mem {
                // Archive excess (oldest, lowest importance first)
                let excess = count - max_mem;
                let archive_sql = format!(
                    "UPDATE memories SET lifecycle_state = 'archived'
                     WHERE id IN (
                        SELECT id FROM memories
                        WHERE workspace = ? AND valid_to IS NULL
                          AND COALESCE(lifecycle_state, 'active') = 'active'{exclude_sql}
                        ORDER BY importance ASC, access_count ASC, created_at ASC
                        LIMIT ?
                     )"
                );
                let mut archive_params: Vec<rusqlite::types::Value> =
                    vec![policy.workspace.clone().into()];
                archive_params.extend(exclude_vals.into_iter().map(Into::into));
                archive_params.push(excess.into());
                let archived =
                    conn.execute(&archive_sql, rusqlite::params_from_iter(archive_params))?;
                total_affected += archived as i64;
            }
        }

        // 4. Auto-delete very old archived memories (respecting exclude_types)
        if let Some(delete_days) = policy.auto_delete_after_days {
            let cutoff =
                crate::storage::queries::cutoff_days_ago(Utc::now(), delete_days)?.to_rfc3339();
            let now = Utc::now().to_rfc3339();
            let (exclude_sql, exclude_vals) =
                build_exclude_type_clause("memory_type", &[], &policy.exclude_types);
            let sql = format!(
                "UPDATE memories SET valid_to = ?
                 WHERE workspace = ? AND valid_to IS NULL
                   AND lifecycle_state = 'archived'
                   AND julianday(created_at) < julianday(?){exclude_sql}"
            );
            let mut bind_vals: Vec<rusqlite::types::Value> =
                vec![now.into(), policy.workspace.clone().into(), cutoff.into()];
            bind_vals.extend(exclude_vals.into_iter().map(Into::into));
            let deleted = conn.execute(&sql, rusqlite::params_from_iter(bind_vals))?;
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
    compress_old_memories_scoped(
        conn,
        None,
        &[],
        max_age_days,
        max_importance,
        min_access_count,
        batch_limit,
    )
}

pub(crate) fn compress_old_memories_scoped(
    conn: &Connection,
    workspace: Option<&str>,
    exclude_types: &[String],
    max_age_days: i64,
    max_importance: f32,
    min_access_count: i32,
    batch_limit: usize,
) -> Result<i64> {
    let cutoff = crate::storage::queries::cutoff_days_ago(Utc::now(), max_age_days)?.to_rfc3339();
    let now = Utc::now().to_rfc3339();
    let workspace_sql = if workspace.is_some() {
        " AND m.workspace = ?"
    } else {
        ""
    };
    let (exclude_sql, exclude_vals) =
        build_exclude_type_clause("m.memory_type", &["summary", "checkpoint"], exclude_types);
    let query = format!(
        "SELECT id, content, memory_type, importance, tags, workspace
         FROM (
            SELECT m.id, m.content, m.memory_type, m.importance, m.access_count, m.workspace,
                   COALESCE(m.lifecycle_state, 'active') as lifecycle_state,
                   (SELECT GROUP_CONCAT(t.name, ',') FROM memory_tags mt JOIN tags t ON mt.tag_id = t.id WHERE mt.memory_id = m.id) as tags
            FROM memories m
            WHERE m.valid_to IS NULL
              AND (m.expires_at IS NULL OR julianday(m.expires_at) > julianday(?))
              AND julianday(m.created_at) < julianday(?)
              AND m.importance <= ?
              AND m.access_count < ?{workspace_sql}{exclude_sql}
              AND COALESCE(m.lifecycle_state, 'active') = 'archived'
              AND NOT EXISTS (
                  SELECT 1
                  FROM memories s
                  WHERE s.summary_of_id = m.id
                    AND s.memory_type = 'summary'
                    AND s.valid_to IS NULL
              )
            ORDER BY m.created_at ASC
            LIMIT ?
         )"
    );

    let mut query_params: Vec<rusqlite::types::Value> = vec![
        now.into(),
        cutoff.into(),
        (max_importance as f64).into(),
        min_access_count.into(),
    ];
    if let Some(ws) = workspace {
        query_params.push(ws.to_string().into());
    }
    query_params.extend(exclude_vals.into_iter().map(Into::into));
    query_params.push((batch_limit as i64).into());

    let mut stmt = conn.prepare(&query)?;
    let candidates: Vec<(i64, String, String, f32, Option<String>, String)> = stmt
        .query_map(rusqlite::params_from_iter(query_params), |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get::<_, String>(5)
                    .unwrap_or_else(|_| "default".to_string()),
            ))
        })?
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
    fn set_retention_policy_rejects_out_of_range_day_and_count_fields() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let too_far = crate::storage::queries::MAX_OFFSET_DAYS + 1;
                let cases: [(Option<i64>, Option<i64>, Option<i64>, Option<i64>); 10] = [
                    (Some(too_far), None, None, None),
                    (Some(-1), None, None, None),
                    (Some(0), None, None, None),
                    (None, Some(-1), None, None),
                    (None, Some(0), None, None),
                    (None, None, Some(too_far), None),
                    (None, None, Some(0), None),
                    (None, None, None, Some(99_999)),
                    (None, None, None, Some(-1)),
                    (None, None, None, Some(0)),
                ];
                for (max_age, max_memories, compress_after, auto_delete) in cases {
                    let result = set_retention_policy(
                        conn,
                        "default",
                        max_age,
                        max_memories,
                        compress_after,
                        None,
                        None,
                        auto_delete,
                        None,
                    );
                    assert!(
                        matches!(result, Err(EngramError::InvalidInput(_))),
                        "accepted {max_age:?}/{max_memories:?}/{compress_after:?}/{auto_delete:?}"
                    );
                }
                assert!(get_retention_policy(conn, "default")?.is_none());
                Ok(())
            })
            .expect("out-of-range policies must be rejected");
    }

    #[test]
    fn apply_retention_skips_a_stored_out_of_range_policy_and_runs_the_rest() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let id = seed_old_memory(conn, "archived")?;
                // A row written before validation existed; sorts before "default".
                conn.execute(
                    "INSERT INTO retention_policies
                        (workspace, auto_delete_after_days, created_at, updated_at)
                     VALUES ('aaa-legacy', 99999, ?1, ?1)",
                    params![Utc::now().to_rfc3339()],
                )?;
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

                assert_eq!(affected, 1, "the valid workspace must still be processed");
                let valid_to: Option<String> = conn.query_row(
                    "SELECT valid_to FROM memories WHERE id = ?1",
                    params![id],
                    |row| row.get(0),
                )?;
                assert!(valid_to.is_some());
                Ok(())
            })
            .expect("one bad stored policy must not block the others");
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

    #[test]
    fn set_retention_policy_preserves_custom_thresholds_and_validates_extra_fields() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let created = set_retention_policy(
                    conn,
                    "ws-partial",
                    None,
                    None,
                    Some(60),
                    Some(0.75),
                    Some(9),
                    None,
                    Some(Vec::new()),
                )?;
                assert!((created.compress_max_importance - 0.75).abs() < 1e-6);
                assert_eq!(created.compress_min_access, 9);
                assert!(
                    created.exclude_types.is_empty(),
                    "empty exclude_types must deserialize as empty Vec, got {:?}",
                    created.exclude_types
                );

                // Partial update omitting compress_max_importance / compress_min_access must preserve 0.75 and 9.
                let updated = set_retention_policy(
                    conn,
                    "ws-partial",
                    None,
                    Some(50),
                    None,
                    None,
                    None,
                    None,
                    None,
                )?;
                assert_eq!(updated.max_memories, Some(50));
                assert_eq!(updated.compress_after_days, Some(60));
                assert!((updated.compress_max_importance - 0.75).abs() < 1e-6);
                assert_eq!(updated.compress_min_access, 9);

                // Invalid importance, access count, or unknown memory_type must fail closed.
                for bad_imp in [1.5_f32, -0.1_f32, f32::NAN, f32::INFINITY] {
                    let err = set_retention_policy(
                        conn,
                        "ws-partial",
                        None,
                        None,
                        None,
                        Some(bad_imp),
                        None,
                        None,
                        None,
                    );
                    assert!(matches!(err, Err(EngramError::InvalidInput(_))));
                }
                let bad_acc = set_retention_policy(
                    conn,
                    "ws-partial",
                    None,
                    None,
                    None,
                    None,
                    Some(-1),
                    None,
                    None,
                );
                assert!(matches!(bad_acc, Err(EngramError::InvalidInput(_))));

                let bad_type = set_retention_policy(
                    conn,
                    "ws-partial",
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(vec!["not_a_valid_memory_type".to_string()]),
                );
                assert!(matches!(bad_type, Err(EngramError::InvalidInput(_))));
                Ok(())
            })
            .expect("partial update and validation");
    }

    #[test]
    fn apply_retention_policies_enforces_max_age_workspace_scoping_and_exclude_types() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        storage
            .with_transaction(|conn| {
                let seed_custom =
                    |ws: &str, mtype: MemoryType, age_days: i64, state: &str| -> Result<i64> {
                        let mem = create_memory(
                            conn,
                            &CreateMemoryInput {
                                content: format!("{ws} {mtype:?} {age_days}d {state}"),
                                memory_type: mtype,
                                importance: Some(0.1),
                                workspace: Some(ws.to_string()),
                                tier: MemoryTier::Permanent,
                                ..Default::default()
                            },
                        )?;
                        let ts = (Utc::now() - chrono::Duration::days(age_days)).to_rfc3339();
                        conn.execute(
                            "UPDATE memories
                         SET created_at = ?1, updated_at = ?1, last_accessed_at = ?1,
                             access_count = 0, lifecycle_state = ?2
                         WHERE id = ?3",
                            params![ts, state, mem.id],
                        )?;
                        Ok(mem.id)
                    };

                // ws-a has policy: max_age=90, compress_after=60, max_memories=1, exclude_types=["decision"]
                let ws_a_expired_note = seed_custom("ws-a", MemoryType::Note, 120, "active")?;
                let ws_a_expired_decision =
                    seed_custom("ws-a", MemoryType::Decision, 120, "active")?;
                let ws_a_archived_note = seed_custom("ws-a", MemoryType::Note, 75, "archived")?;
                let ws_a_archived_decision =
                    seed_custom("ws-a", MemoryType::Decision, 75, "archived")?;
                let ws_a_fresh_note_1 = seed_custom("ws-a", MemoryType::Note, 10, "active")?;
                let ws_a_fresh_note_2 = seed_custom("ws-a", MemoryType::Note, 5, "active")?;

                // ws-b has NO retention policy; its archived memory must not be compressed by ws-a's policy.
                let ws_b_archived_note = seed_custom("ws-b", MemoryType::Note, 75, "archived")?;

                set_retention_policy(
                    conn,
                    "ws-a",
                    Some(90),
                    Some(1),
                    Some(60),
                    Some(0.5),
                    Some(5),
                    None,
                    Some(vec!["decision".to_string()]),
                )?;

                let affected = apply_retention_policies(conn)?;
                assert!(
                    affected >= 3,
                    "expected max_age + compress + max_memories actions"
                );

                let is_deleted = |id: i64| -> Result<bool> {
                    let vt: Option<String> = conn.query_row(
                        "SELECT valid_to FROM memories WHERE id = ?1",
                        params![id],
                        |r| r.get(0),
                    )?;
                    Ok(vt.is_some())
                };

                assert!(
                    is_deleted(ws_a_expired_note)?,
                    "120d note in ws-a must be soft-deleted by max_age_days=90"
                );
                assert!(
                    !is_deleted(ws_a_expired_decision)?,
                    "120d decision in ws-a must be spared by exclude_types"
                );
                assert_eq!(
                    summary_count_for(conn, ws_a_archived_note)?,
                    1,
                    "75d archived note in ws-a must be compressed"
                );
                assert_eq!(
                    summary_count_for(conn, ws_a_archived_decision)?,
                    0,
                    "75d archived decision in ws-a must be spared from compression by exclude_types"
                );
                assert_eq!(
                    summary_count_for(conn, ws_b_archived_note)?,
                    0,
                    "ws-b archived note must NOT be compressed by ws-a policy"
                );

                // Between the two remaining active fresh notes in ws-a, max_memories=1 archives one.
                let active_fresh = [ws_a_fresh_note_1, ws_a_fresh_note_2]
                    .into_iter()
                    .filter(|id| lifecycle_state(conn, *id).unwrap() == "active")
                    .count();
                assert_eq!(
                    active_fresh, 1,
                    "max_memories=1 must keep 1 active note in ws-a"
                );
                assert_eq!(
                    lifecycle_state(conn, ws_a_expired_decision)?,
                    "active",
                    "excluded decision must not count toward or be archived by max_memories"
                );
                Ok(())
            })
            .expect("retention scoping and exclude_types");
    }
}
