use std::collections::HashMap;

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

pub(super) fn query_workspace_memories(
    ctx: &HandlerContext,
    workspace: &str,
) -> Result<Vec<Value>, crate::error::EngramError> {
    ctx.storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.content, m.memory_type, m.importance, m.workspace, m.tier,
                    m.created_at, m.updated_at,
                    (SELECT GROUP_CONCAT(t.name, ',')
                     FROM memory_tags mt
                     JOIN tags t ON mt.tag_id = t.id
                     WHERE mt.memory_id = m.id) as tags,
                    m.scope_type as scope, m.version, m.metadata, m.content_hash
             FROM memories m
             WHERE m.workspace = ?1
               AND COALESCE(m.lifecycle_state, 'active') != 'archived'
               AND m.valid_to IS NULL
             ORDER BY m.memory_type, m.created_at",
        )?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(std::iter::once(workspace)),
            |row| {
                let metadata_str: Option<String> = row.get(11)?;
                let metadata: Value = metadata_str
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or(Value::Null);
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "content": row.get::<_, String>(1)?,
                    "memory_type": row.get::<_, String>(2)?,
                    "importance": row.get::<_, Option<f64>>(3)?,
                    "workspace": row.get::<_, String>(4)?,
                    "tier": row.get::<_, Option<String>>(5)?,
                    "created_at": row.get::<_, String>(6)?,
                    "updated_at": row.get::<_, Option<String>>(7)?,
                    "tags": row.get::<_, Option<String>>(8)?,
                    "scope": row.get::<_, Option<String>>(9)?,
                    "version": row.get::<_, Option<i64>>(10)?,
                    "metadata": metadata,
                    "content_hash": row.get::<_, Option<String>>(12)?
                }))
            },
        )?;
        let memories: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
        Ok(memories)
    })
}

/// Build a map of memory_id -> [(related_id, relation_type)].
///
/// Issues a single batched query over all `memory_ids` instead of one per ID.
/// DB errors are logged and result in an empty map rather than being silently
/// discarded.
pub(super) fn build_related_map(
    ctx: &HandlerContext,
    memory_ids: &[i64],
) -> HashMap<i64, Vec<(i64, String)>> {
    if memory_ids.is_empty() {
        return HashMap::new();
    }

    // Build a single IN-list query for all IDs at once.
    let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{i}")).collect();
    let in_list = placeholders.join(", ");
    let sql = format!(
        "SELECT from_id, to_id, edge_type FROM crossrefs
          WHERE from_id IN ({in_list}) OR to_id IN ({in_list})"
    );

    let result = ctx.storage.with_connection(|conn| {
        let mut stmt = conn.prepare(&sql)?;
        let rows: Vec<(i64, i64, String)> = stmt
            .query_map(rusqlite::params_from_iter(memory_ids.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(crate::error::EngramError::Database)?;
        Ok(rows)
    });

    match result {
        Ok(rows) => {
            let id_set: std::collections::HashSet<i64> = memory_ids.iter().copied().collect();
            let mut map: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
            for (from_id, to_id, rel_type) in rows {
                if id_set.contains(&from_id) {
                    map.entry(from_id)
                        .or_default()
                        .push((to_id, rel_type.clone()));
                }
                if id_set.contains(&to_id) && to_id != from_id {
                    map.entry(to_id).or_default().push((from_id, rel_type));
                }
            }
            map
        }
        Err(e) => {
            eprintln!("[markdown_export] build_related_map DB error: {e}");
            HashMap::new()
        }
    }
}
