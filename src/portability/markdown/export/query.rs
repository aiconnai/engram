use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::error::EngramError;
use crate::portability::markdown::MarkdownContext;

pub(super) type RelatedMap = HashMap<i64, Vec<(i64, String)>>;

/// Read memories and their eligible relations from one consistent SQLite snapshot.
/// The transaction ends before rendering or filesystem I/O.
pub(super) fn query_export_snapshot(
    ctx: &MarkdownContext<'_>,
    workspace: &str,
    include_links: bool,
) -> Result<(Vec<Value>, RelatedMap), EngramError> {
    if let Some(principal) = ctx
        .principal
        .filter(|p| !crate::auth::access::allows_all_workspaces(p))
    {
        let allowed = crate::types::normalize_workspace(workspace)
            .is_ok_and(|normalized| principal.allows_workspace(Some(&normalized)));
        if !allowed {
            return Err(EngramError::Unauthorized(
                "Markdown target workspace is not allowed".into(),
            ));
        }
    }
    ctx.storage.with_connection(|conn| {
        let tx = conn.unchecked_transaction()?;
        let cutoff = chrono::Utc::now().to_rfc3339();
        let memories = query_workspace_memories_at(&tx, workspace, &cutoff)?;
        let related = if include_links && !memories.is_empty() {
            let ids: HashSet<i64> = memories
                .iter()
                .map(|m| {
                    m["id"].as_i64().ok_or_else(|| {
                        EngramError::InvalidInput("invalid exported memory id".into())
                    })
                })
                .collect::<Result<_, _>>()?;
            build_related_map(&tx, workspace, &ids)?
        } else {
            RelatedMap::new()
        };
        tx.commit()?;
        Ok((memories, related))
    })
}

pub(super) fn query_workspace_memories_at(
    conn: &Connection,
    workspace: &str,
    cutoff: &str,
) -> Result<Vec<Value>, EngramError> {
    let cutoff = chrono::DateTime::parse_from_rfc3339(cutoff)
        .map_err(|_| EngramError::InvalidInput("invalid export cutoff".into()))?;
    let mut stmt = conn.prepare(
        "SELECT m.id,m.content,m.memory_type,m.importance,m.workspace,m.tier,
                m.created_at,m.updated_at,
                (SELECT json_group_array(name) FROM (
                    SELECT t.name AS name FROM memory_tags mt
                    JOIN tags t ON mt.tag_id=t.id
                    WHERE mt.memory_id=m.id ORDER BY t.name,t.id
                )) AS tags,
                m.scope_type,m.version,m.metadata,m.content_hash,m.expires_at
         FROM memories m WHERE m.workspace=?1
           AND COALESCE(m.lifecycle_state,'active') != 'archived'
           AND m.valid_to IS NULL
         ORDER BY m.memory_type,m.created_at,m.id",
    )?;
    let mut rows = stmt.query([workspace])?;
    let mut memories = Vec::new();
    while let Some(row) = rows.next()? {
        let expiration: Option<String> = row.get(13)?;
        if let Some(raw) = expiration {
            // Compare instants, not RFC3339 strings: Z, offsets and fractions
            // have different textual ordering. Keep nanosecond precision.
            let expiry = chrono::DateTime::parse_from_rfc3339(&raw).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    13,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;
            if expiry <= cutoff {
                continue;
            }
        }
        let metadata_str: Option<String> = row.get(11)?;
        let metadata: Value = match metadata_str {
            Some(raw) => serde_json::from_str(&raw).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?,
            None => Value::Null,
        };
        memories.push(json!({
            "id": row.get::<_,i64>(0)?, "content": row.get::<_,String>(1)?,
            "memory_type": row.get::<_,String>(2)?, "importance": row.get::<_,Option<f64>>(3)?,
            "workspace": row.get::<_,String>(4)?, "tier": row.get::<_,Option<String>>(5)?,
            "created_at": row.get::<_,String>(6)?, "updated_at": row.get::<_,Option<String>>(7)?,
            "tags": row.get::<_,String>(8)?, "scope": row.get::<_,Option<String>>(9)?,
            "version": row.get::<_,Option<i64>>(10)?, "metadata": metadata,
            "content_hash": row.get::<_,Option<String>>(12)?
        }));
    }
    Ok(memories)
}

fn build_related_map(
    conn: &Connection,
    workspace: &str,
    eligible: &HashSet<i64>,
) -> Result<RelatedMap, EngramError> {
    let mut stmt = conn.prepare(
        "SELECT c.from_id,c.to_id,c.edge_type FROM crossrefs c
         JOIN memories a ON a.id=c.from_id JOIN memories b ON b.id=c.to_id
         WHERE a.workspace=?1 AND b.workspace=?1 AND c.valid_to IS NULL
         ORDER BY c.edge_type,c.from_id,c.to_id",
    )?;
    let rows = stmt.query_map([workspace], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut map = RelatedMap::new();
    for row in rows {
        let (from, to, kind) = row?;
        if !eligible.contains(&from) || !eligible.contains(&to) {
            continue;
        }
        map.entry(from).or_default().push((to, kind.clone()));
        if to != from {
            map.entry(to)
                .or_default()
                .push((from, format!("incoming {kind}")));
        }
    }
    for links in map.values_mut() {
        links.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    }
    Ok(map)
}
