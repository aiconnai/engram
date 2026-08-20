//! Spatial navigation handlers ("Method of Loci").
//! Implements palace_navigate, room_search, and drawer_open.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

use super::HandlerContext;
use crate::error::EngramError;
use crate::storage::queries::get_memory;

/// Navigate the Memory Palace: discover available wings and rooms with drawer counts.
pub fn palace_navigate(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let target_wing = params.get("wing").and_then(|v| v.as_str());

    ctx.storage
        .with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT scope_path, COUNT(*) as cnt FROM memories
                 WHERE workspace = ?1 AND lifecycle_state != 'archived'
                 GROUP BY scope_path",
            )?;

            let rows = stmt.query_map([workspace], |row| {
                let scope_path: Option<String> = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((scope_path.unwrap_or_else(|| "global".to_string()), count))
            })?;

            let mut wings_map: BTreeMap<String, (i64, BTreeSet<String>)> = BTreeMap::new();
            let mut total_drawers = 0;

            for row in rows {
                let (path, count) = row?;
                total_drawers += count;

                // Normalize path into wing / room
                let clean_path = path.trim_start_matches("wing:").trim_start_matches('/');
                let mut parts = clean_path.split('/');
                let wing = parts.next().unwrap_or("general").to_string();
                let room = parts
                    .next()
                    .map(|r| r.trim_start_matches("room:").to_string());

                if let Some(tw) = target_wing {
                    if !wing.eq_ignore_ascii_case(tw) && !path.contains(tw) {
                        continue;
                    }
                }

                let entry = wings_map.entry(wing).or_insert((0, BTreeSet::new()));
                entry.0 += count;
                if let Some(r) = room {
                    if !r.is_empty() {
                        entry.1.insert(r);
                    }
                }
            }

            let wings_list: Vec<Value> = wings_map
                .into_iter()
                .map(|(wing_name, (count, rooms))| {
                    json!({
                        "wing": wing_name,
                        "drawers_count": count,
                        "rooms": rooms.into_iter().collect::<Vec<_>>()
                    })
                })
                .collect();

            Ok(json!({
                "palace": workspace,
                "wings_count": wings_list.len(),
                "total_drawers": total_drawers,
                "wings": wings_list
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Scoped hybrid search within a spatial room and wing.
pub fn room_search(ctx: &HandlerContext, params: Value) -> Value {
    let wing = match params.get("wing").and_then(|v| v.as_str()) {
        Some(w) => w,
        None => return json!({"error": "wing is required"}),
    };
    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return json!({"error": "query is required"}),
    };
    let room = params.get("room").and_then(|v| v.as_str());
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
    let workspace = params.get("workspace").and_then(|v| v.as_str());

    // Build the scoped search parameters delegating to memory_search
    let mut search_params = json!({
        "query": query,
        "limit": limit,
    });

    if let Some(ws) = workspace {
        search_params["workspace"] = json!(ws);
    }

    // Set scope path filter for hierarchical retrieval
    let scope_prefix = if let Some(r) = room {
        format!("wing:{}/room:{}", wing, r)
    } else {
        format!("wing:{}", wing)
    };
    search_params["scope_path"] = json!(scope_prefix);

    super::search::memory_search(ctx, search_params)
}

/// Open a specific memory drawer by ID.
pub fn drawer_open(ctx: &HandlerContext, params: Value) -> Value {
    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };

    ctx.storage
        .with_connection(|conn| match get_memory(conn, id) {
            Ok(memory) => Ok(json!({
                "id": memory.id,
                "content": memory.content,
                "memory_type": memory.memory_type.as_str(),
                "importance": memory.importance,
                "tags": memory.tags,
                "workspace": memory.workspace,
                "scope": memory.scope,
                "created_at": memory.created_at,
                "updated_at": memory.updated_at,
                "metadata": memory.metadata,
            })),
            Err(EngramError::NotFound(_)) => {
                Ok(json!({"error": format!("Drawer with ID {} not found", id)}))
            }
            Err(e) => Err(e),
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::create_memory;
    use crate::types::{CreateMemoryInput, MemoryType};
    use parking_lot::{Mutex, RwLock};
    use std::sync::Arc;

    fn test_context() -> HandlerContext {
        HandlerContext {
            storage: crate::Storage::open_in_memory().expect("in-memory storage"),
            embedder: crate::embedding::create_embedder(&crate::types::EmbeddingConfig::default())
                .expect("tfidf embedder"),
            fuzzy_engine: Arc::new(Mutex::new(crate::search::FuzzyEngine::new())),
            search_config: crate::search::SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(crate::search::SearchResultCache::new(
                crate::search::AdaptiveCacheConfig::default(),
            )),
            hnsw_index: Arc::new(RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
        }
    }

    #[test]
    fn test_palace_navigate_and_drawer_open() {
        let ctx = test_context();

        let id1 = ctx
            .storage
            .with_transaction(|conn| {
                let m = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "JWT auth tokens expire in 15 minutes".to_string(),
                        memory_type: MemoryType::Verbatim,
                        workspace: Some("default".to_string()),
                        tags: vec!["wing:backend".to_string(), "room:auth".to_string()],
                        ..Default::default()
                    },
                )?;
                // Set scope_path directly in SQLite
                conn.execute(
                    "UPDATE memories SET scope_path = 'wing:backend/room:auth' WHERE id = ?",
                    [m.id],
                )?;
                Ok(m.id)
            })
            .unwrap();

        let _id2 = ctx
            .storage
            .with_transaction(|conn| {
                let m = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "Database pool size is 20 connections".to_string(),
                        memory_type: MemoryType::Decision,
                        workspace: Some("default".to_string()),
                        tags: vec!["wing:backend".to_string(), "room:db".to_string()],
                        ..Default::default()
                    },
                )?;
                conn.execute(
                    "UPDATE memories SET scope_path = 'wing:backend/room:db' WHERE id = ?",
                    [m.id],
                )?;
                Ok(m.id)
            })
            .unwrap();

        // Navigate the palace
        let nav = palace_navigate(&ctx, json!({"workspace": "default"}));
        assert_eq!(nav["palace"], "default");
        assert!(nav["wings_count"].as_u64().unwrap() >= 1);
        assert_eq!(nav["total_drawers"], 2);

        // Open drawer
        let drawer = drawer_open(&ctx, json!({"id": id1}));
        assert_eq!(drawer["id"], id1);
        assert_eq!(drawer["memory_type"], "verbatim");
        assert_eq!(drawer["workspace"], "default");
        assert!(drawer["tags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t == "wing:backend"));

        // Open non-existent drawer
        let drawer_missing = drawer_open(&ctx, json!({"id": 999999}));
        assert!(drawer_missing["error"]
            .as_str()
            .unwrap()
            .contains("not found"));
    }
}
