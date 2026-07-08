//! Prompt-context assembly tools (build_context, injection prompt, prepare_context).
use serde_json::{json, Value};

use super::super::HandlerContext;
use super::safe_truncate;

/// Build a structured prompt context from memories.
///
/// Params:
/// - `query` (string, required) — search query to retrieve relevant memories
/// - `total_budget` (u64, optional) — max tokens for the entire prompt (default: 4096)
/// - `strategy` (string, optional) — "greedy" | "balanced" | "recency" (default: "greedy")
/// - `workspace` (string, optional) — workspace to search in
/// - `limit` (u64, optional) — max memories to retrieve (default: 20)
/// - `depth` (u64, optional) — graph traversal depth 1-3 (default: 1, search only)
/// - `timeframe` (string, optional) — "1h"|"24h"|"7d"|"30d"|"all" (default: "all")
/// - `include_types` (array of string, optional) — filter to these memory types
/// - `include_graph` (bool, optional) — include relationship graph in response (default: false)
pub fn memory_build_context(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::context_builder::{
        ContextBuilder, MemoryEntry, PromptTemplate, Section, SimpleTokenCounter, Strategy,
    };
    use crate::search::hybrid_search;
    use crate::types::SearchOptions;
    use chrono::{Duration, Utc};
    use std::collections::HashSet;

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return json!({"error": "query is required"}),
    };

    let total_budget = params
        .get("total_budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(4096) as usize;

    let strategy = match params.get("strategy").and_then(|v| v.as_str()) {
        Some("balanced") => Strategy::Balanced,
        Some("recency") => Strategy::Recency,
        _ => Strategy::Greedy,
    };

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    // New params: depth, timeframe, include_types, include_graph
    let depth = params
        .get("depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(1)
        .clamp(1, 3) as usize;

    let timeframe = params
        .get("timeframe")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let include_types: Option<Vec<String>> = params
        .get("include_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    let include_graph = params
        .get("include_graph")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Compute the timeframe cutoff
    let time_cutoff = match timeframe {
        "1h" => Some(Utc::now() - Duration::hours(1)),
        "24h" => Some(Utc::now() - Duration::hours(24)),
        "7d" => Some(Utc::now() - Duration::days(7)),
        "30d" => Some(Utc::now() - Duration::days(30)),
        _ => None, // "all" or unrecognized
    };

    let search_opts = SearchOptions {
        workspace: params
            .get("workspace")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        limit: Some(limit as i64),
        ..Default::default()
    };

    let query_embedding = ctx.embedder.embed(&query).ok();
    let embedding_ref = query_embedding.as_deref();

    let search_result = ctx.storage.with_connection(|conn| {
        hybrid_search(
            conn,
            &query,
            embedding_ref,
            &search_opts,
            &ctx.search_config,
        )
    });

    let mut memories = match search_result {
        Ok(results) => results,
        Err(e) => return json!({"error": e.to_string()}),
    };

    // Apply timeframe filter
    if let Some(cutoff) = time_cutoff {
        memories.retain(|r| r.memory.created_at >= cutoff);
    }

    // Apply type filter
    if let Some(ref types) = include_types {
        memories.retain(|r| types.contains(&r.memory.memory_type.as_str().to_string()));
    }

    // Depth expansion: follow crossref links to pull in related memories
    let mut all_memory_contents: Vec<(String, chrono::DateTime<Utc>)> = memories
        .iter()
        .map(|r| (r.memory.content.clone(), r.memory.created_at))
        .collect();

    let mut all_memory_ids: Vec<i64> = memories.iter().map(|r| r.memory.id).collect();

    if depth > 1 {
        let mut seen_ids: HashSet<i64> = all_memory_ids.iter().copied().collect();
        let mut frontier: Vec<i64> = all_memory_ids.clone();

        for _hop in 1..depth {
            let mut next_frontier = Vec::new();
            for id in &frontier {
                if let Ok(related) = ctx.storage.with_connection(|conn| {
                    let mut stmt = conn.prepare(
                        "SELECT DISTINCT to_id FROM crossrefs WHERE from_id = ?1
                         UNION
                         SELECT DISTINCT from_id FROM crossrefs WHERE to_id = ?1",
                    )?;
                    let ids: Vec<i64> = stmt
                        .query_map(rusqlite::params![id], |row| row.get(0))?
                        .filter_map(|r| r.ok())
                        .collect();
                    Ok(ids)
                }) {
                    for rid in related {
                        if seen_ids.insert(rid) {
                            next_frontier.push(rid);
                        }
                    }
                }
            }

            if next_frontier.is_empty() {
                break;
            }

            // Fetch the actual memories for the new frontier
            for id in &next_frontier {
                if let Ok(mem) = ctx
                    .storage
                    .with_connection(|conn| crate::storage::queries::get_memory(conn, *id))
                {
                    // Apply timeframe filter to expanded memories too
                    if let Some(cutoff) = time_cutoff {
                        if mem.created_at < cutoff {
                            continue;
                        }
                    }
                    // Apply type filter to expanded memories too
                    if let Some(ref types) = include_types {
                        if !types.contains(&mem.memory_type.as_str().to_string()) {
                            continue;
                        }
                    }
                    all_memory_contents.push((mem.content.clone(), mem.created_at));
                    all_memory_ids.push(mem.id);
                }
            }

            frontier = next_frontier;
        }
    }

    // Convert to MemoryEntry items.
    let entries: Vec<MemoryEntry> = all_memory_contents
        .iter()
        .map(|(content, created_at)| MemoryEntry::new(content.clone(), *created_at))
        .collect();

    let template = PromptTemplate {
        sections: vec![Section {
            name: "Memories".to_string(),
            content: String::new(),
            max_tokens: total_budget,
            priority: 0,
        }],
        total_budget,
        separator: "\n\n---\n\n".to_string(),
    };

    let builder = ContextBuilder::new(Box::new(SimpleTokenCounter));
    let prompt = builder.build(&template, &entries, strategy);
    let token_estimate = builder.estimate_tokens(&prompt);

    // Build graph data if requested
    let graph = if include_graph {
        ctx.storage
            .with_connection(|conn| {
                let mut edges = Vec::new();
                for id in &all_memory_ids {
                    let mut stmt = conn.prepare(
                        "SELECT from_id, to_id, edge_type FROM crossrefs
                         WHERE from_id = ?1 OR to_id = ?1",
                    )?;
                    let rows: Vec<Value> = stmt
                        .query_map(rusqlite::params![id], |row| {
                            Ok(json!({
                                "source": row.get::<_, i64>(0)?,
                                "target": row.get::<_, i64>(1)?,
                                "relation": row.get::<_, String>(2)?
                            }))
                        })?
                        .filter_map(|r| r.ok())
                        .collect();
                    edges.extend(rows);
                }
                Ok(json!({"edges": edges, "node_count": all_memory_ids.len()}))
            })
            .unwrap_or_else(|_| json!({"edges": [], "node_count": 0}))
    } else {
        Value::Null
    };

    let mut response = json!({
        "prompt": prompt,
        "token_estimate": token_estimate,
        "memories_used": entries.len(),
        "total_budget": total_budget,
        "depth": depth,
        "timeframe": timeframe
    });

    if include_graph {
        response
            .as_object_mut()
            .expect("response is an object")
            .insert("graph".to_string(), graph);
    }

    response
}
/// Build a ready-to-inject prompt string from memories relevant to a query.
///
/// Params:
/// - `query` (string, required) — search query to retrieve relevant memories
/// - `token_budget` (u64, optional, default: 2000) — maximum tokens for the output prompt
/// - `workspace` (string, optional) — workspace to search in
/// - `include_types` (array of string, optional) — filter by memory type (e.g. ["note","episodic"])
pub fn memory_get_injection_prompt(ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::hybrid_search;
    use crate::types::SearchOptions;

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return json!({"error": "query is required"}),
    };

    let token_budget = params
        .get("token_budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(2000) as usize;

    let include_types: Vec<String> = params
        .get("include_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let search_opts = SearchOptions {
        workspace: params
            .get("workspace")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        limit: Some(20),
        ..Default::default()
    };

    let query_embedding = ctx.embedder.embed(&query).ok();
    let embedding_ref = query_embedding.as_deref();

    let search_result = ctx.storage.with_connection(|conn| {
        hybrid_search(
            conn,
            &query,
            embedding_ref,
            &search_opts,
            &ctx.search_config,
        )
    });

    let memories = match search_result {
        Ok(results) => results,
        Err(e) => return json!({"error": e.to_string()}),
    };

    // Filter by memory type if include_types is specified.
    let memories: Vec<_> = if include_types.is_empty() {
        memories
    } else {
        memories
            .into_iter()
            .filter(|r| include_types.contains(&r.memory.memory_type.as_str().to_string()))
            .collect()
    };

    if memories.is_empty() {
        return json!({
            "prompt": "# Relevant Context\n\n*(No memories found)*",
            "memory_count": 0,
            "tokens_used": 0
        });
    }

    // Build per-memory markdown blocks.
    let blocks: Vec<String> = memories
        .iter()
        .map(|r| {
            let m = &r.memory;
            let tags_str = m.tags.join(", ");
            format!(
                "## [{}] Memory #{}\nCreated: {} | Tags: {}\n\n{}\n\n---",
                m.memory_type.as_str(),
                m.id,
                m.created_at.to_rfc3339(),
                tags_str,
                m.content
            )
        })
        .collect();

    // Estimate tokens for the full prompt.
    let header = "# Relevant Context\n\n";
    let joined = blocks.join("\n\n");
    let full_prompt = format!("{}{}", header, joined);
    let total_chars = full_prompt.len();
    let estimated_tokens = total_chars / 4;

    if estimated_tokens <= token_budget {
        return json!({
            "prompt": full_prompt,
            "memory_count": memories.len(),
            "tokens_used": estimated_tokens
        });
    }

    // Budget exceeded — proportionally truncate each memory's content.
    let count = memories.len();
    let budget_chars = token_budget * 4;
    let header_chars = header.len();
    let separator_chars = "\n\n".len() * (count.saturating_sub(1));
    let overhead_per_block = 80usize;
    let total_overhead = header_chars + separator_chars + overhead_per_block * count;
    let available_content_chars = budget_chars.saturating_sub(total_overhead);
    let chars_per_content = available_content_chars.checked_div(count).unwrap_or(0);

    let truncated_blocks: Vec<String> = memories
        .iter()
        .map(|r| {
            let m = &r.memory;
            let tags_str = m.tags.join(", ");
            let content = if m.content.len() > chars_per_content && chars_per_content > 0 {
                format!("{}…", safe_truncate(&m.content, chars_per_content))
            } else {
                m.content.clone()
            };
            format!(
                "## [{}] Memory #{}\nCreated: {} | Tags: {}\n\n{}\n\n---",
                m.memory_type.as_str(),
                m.id,
                m.created_at.to_rfc3339(),
                tags_str,
                content
            )
        })
        .collect();

    let final_prompt = format!("{}{}", header, truncated_blocks.join("\n\n"));
    let tokens_used = final_prompt.len() / 4;

    json!({
        "prompt": final_prompt,
        "memory_count": count,
        "tokens_used": tokens_used
    })
}

// ── Tool-use observation ───────────────────────────────────────────────────────

/// Prepare optimized context for an LLM using the RTK-inspired pipeline.
///
/// Pipeline: hybrid search → relevance filter → grouping → token-budget
/// truncation. Returns the assembled context string plus metadata about
/// token usage and group count.
///
/// Params:
/// - `query` (string, required) — search query to retrieve memories for
/// - `budget` (u64, optional, default: 4000) — token budget for the context
/// - `workspace` (string, optional) — workspace filter
pub fn memory_prepare_context(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::integration_orchestrator::IntegrationOrchestrator;
    use crate::search::hybrid_search;
    use crate::types::SearchOptions;

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return json!({"error": "query is required"}),
    };

    let budget = params
        .get("budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(4000) as usize;

    let search_opts = SearchOptions {
        workspace: params
            .get("workspace")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        limit: Some(50),
        ..Default::default()
    };

    let query_embedding = ctx.embedder.embed(&query).ok();
    let embedding_ref = query_embedding.as_deref();

    let search_result = ctx.storage.with_connection(|conn| {
        hybrid_search(
            conn,
            &query,
            embedding_ref,
            &search_opts,
            &ctx.search_config,
        )
    });

    let memories: Vec<_> = match search_result {
        Ok(results) => results.into_iter().map(|r| r.memory).collect(),
        Err(e) => return json!({"error": e.to_string()}),
    };

    let orchestrator = IntegrationOrchestrator::new();
    match orchestrator.prepare_context_for_llm(&query, &memories, budget) {
        Ok(prepared) => json!({
            "context": prepared.context,
            "token_count": prepared.token_count,
            "groups_count": prepared.groups_count,
            "memory_count": memories.len(),
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}
