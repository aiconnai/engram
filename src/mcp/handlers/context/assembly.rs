//! Prompt-context assembly tools (build_context, injection prompt, prepare_context).
use std::collections::HashSet;

use serde_json::{json, Value};

use super::super::HandlerContext;
use super::safe_truncate;
use crate::auth::TransportPrincipal;
use crate::mcp::ToolError;
use crate::types::SearchOptions;

#[derive(Debug, Clone, Default)]
struct ContextWorkspaceScope {
    allowed: Option<HashSet<String>>,
}

impl ContextWorkspaceScope {
    fn allows(&self, workspace: &str, principal: Option<&TransportPrincipal>) -> bool {
        self.allowed
            .as_ref()
            .is_none_or(|allowed| allowed.contains(workspace))
            && principal.is_none_or(|p| p.allows_workspace(Some(workspace)))
    }
}

/// Extract, normalize, and authorize `workspace`, `workspaces`, and `filters.workspace`
/// claims against `ctx.principal`, matching `permission::requested_workspaces`.
fn resolve_context_workspace_scope(
    tool_name: &str,
    params: &Value,
    limit: i64,
    ctx: &HandlerContext,
) -> std::result::Result<(SearchOptions, ContextWorkspaceScope), Value> {
    if let Some(ws_val) = params.get("workspace") {
        if !ws_val.is_null() && !ws_val.is_string() {
            return Err(ToolError::invalid_params("workspace must be a string").into_value());
        }
    }
    if let Some(wss_val) = params.get("workspaces") {
        if !wss_val.is_null()
            && !wss_val
                .as_array()
                .is_some_and(|arr| !arr.is_empty() && arr.iter().all(Value::is_string))
        {
            return Err(ToolError::invalid_params(
                "workspaces must be a non-empty array of strings",
            )
            .into_value());
        }
    }
    if let Some(filter_ws_val) = params
        .get("filters")
        .and_then(Value::as_object)
        .and_then(|f| f.get("workspace"))
    {
        if !filter_ws_val.is_null() && !filter_ws_val.is_string() {
            return Err(
                ToolError::invalid_params("filters.workspace must be a string").into_value(),
            );
        }
    }

    if let Some(p) = ctx.principal.as_ref() {
        if let Some(denial) =
            crate::mcp::permission::check_tool_authorization(None, tool_name, params, Some(p))
        {
            return Err(denial);
        }
    }

    let top_ws = match params.get("workspace").and_then(Value::as_str) {
        Some(raw) => Some(
            crate::types::normalize_workspace(raw)
                .map_err(|e| ToolError::invalid_params(e.to_string()).into_value())?,
        ),
        None => None,
    };

    let filter_ws = match params
        .get("filters")
        .and_then(Value::as_object)
        .and_then(|f| f.get("workspace"))
        .and_then(Value::as_str)
    {
        Some(raw) => Some(
            crate::types::normalize_workspace(raw)
                .map_err(|e| ToolError::invalid_params(e.to_string()).into_value())?,
        ),
        None => None,
    };

    if let (Some(ref a), Some(ref b)) = (&top_ws, &filter_ws) {
        if a != b {
            return Err(ToolError::invalid_params(
                "workspace and filters.workspace must match when both are provided",
            )
            .into_value());
        }
    }
    let single_ws = top_ws.or(filter_ws);

    let multi_ws = if let Some(items) = params.get("workspaces").and_then(Value::as_array) {
        let mut list = Vec::with_capacity(items.len());
        for item in items {
            if let Some(raw) = item.as_str() {
                let normalized = crate::types::normalize_workspace(raw)
                    .map_err(|e| ToolError::invalid_params(e.to_string()).into_value())?;
                if !list.contains(&normalized) {
                    list.push(normalized);
                }
            }
        }
        Some(list)
    } else {
        None
    };

    let mut effective_list = match (single_ws, multi_ws) {
        (Some(ws), Some(list)) => {
            if !list.contains(&ws) {
                return Err(
                    ToolError::invalid_params("workspace is not included in workspaces")
                        .into_value(),
                );
            }
            Some(vec![ws])
        }
        (Some(ws), None) => Some(vec![ws]),
        (None, Some(list)) => Some(list),
        (None, None) => None,
    };

    if let Some(p) = ctx.principal.as_ref() {
        if !crate::mcp::permission::allows_all_workspaces(p) {
            if effective_list.as_ref().is_none_or(Vec::is_empty) {
                let default_ws = p
                    .auth_context()
                    .namespace
                    .as_deref()
                    .map(|ns| {
                        crate::types::normalize_workspace(ns).unwrap_or_else(|_| ns.to_string())
                    })
                    .unwrap_or_else(|| "default".to_string());
                effective_list = Some(vec![default_ws]);
            }
            if let Some(ref list) = effective_list {
                for ws in list {
                    if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
                        tool_name,
                        p,
                        Some(ws),
                    ) {
                        return Err(denial);
                    }
                }
            }
        }
    }

    let mut search_opts = SearchOptions {
        limit: Some(limit),
        global: false,
        ..Default::default()
    };
    let scope = match effective_list {
        Some(list) if list.len() == 1 => {
            let ws = list
                .into_iter()
                .next()
                .expect("invariant: checked list length == 1");
            let mut set = HashSet::with_capacity(1);
            set.insert(ws.clone());
            search_opts.workspace = Some(ws);
            ContextWorkspaceScope { allowed: Some(set) }
        }
        Some(list) => {
            let set: HashSet<String> = list.iter().cloned().collect();
            search_opts.workspaces = Some(list);
            ContextWorkspaceScope { allowed: Some(set) }
        }
        None => ContextWorkspaceScope { allowed: None },
    };

    Ok((search_opts, scope))
}

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
    use chrono::{Duration, Utc};

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

    let (search_opts, scope) =
        match resolve_context_workspace_scope("memory_build_context", &params, limit as i64, ctx) {
            Ok(resolved) => resolved,
            Err(err) => return err,
        };

    let query_embedding = ctx.embedder.embed(&query).ok();
    let embedding_ref = query_embedding.as_deref();

    let search_result = ctx.storage.with_connection(|conn| {
        crate::search::hybrid_search_with_hnsw(
            conn,
            &query,
            embedding_ref,
            &search_opts,
            &ctx.search_config,
            Some(&*ctx.hnsw_index.read()),
        )
    });

    let mut memories = match search_result {
        Ok(results) => results,
        Err(e) => return json!({"error": e.to_string()}),
    };

    // Enforce workspace and principal confinement on initial search hits as well.
    memories.retain(|r| scope.allows(&r.memory.workspace, ctx.principal.as_ref()));

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

            // Fetch the actual memories for the new frontier and advance the next hop
            // ONLY from memories that pass workspace, principal, timeframe, and type checks.
            let mut accepted_next_frontier = Vec::new();
            for id in &next_frontier {
                if let Ok(mem) = ctx
                    .storage
                    .with_connection(|conn| crate::storage::queries::get_memory_readonly(conn, *id))
                {
                    if !scope.allows(&mem.workspace, ctx.principal.as_ref()) {
                        continue;
                    }
                    if let Some(cutoff) = time_cutoff {
                        if mem.created_at < cutoff {
                            continue;
                        }
                    }
                    if let Some(ref types) = include_types {
                        if !types.contains(&mem.memory_type.as_str().to_string()) {
                            continue;
                        }
                    }
                    all_memory_contents.push((mem.content.clone(), mem.created_at));
                    all_memory_ids.push(mem.id);
                    accepted_next_frontier.push(mem.id);
                }
            }

            if accepted_next_frontier.is_empty() {
                break;
            }
            frontier = accepted_next_frontier;
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

    // Build graph data if requested (restricting edges so both endpoints are live and in scope).
    let graph = if include_graph {
        let now_rfc3339 = Utc::now().to_rfc3339();
        ctx.storage
            .with_connection(|conn| {
                let mut seen_edges: HashSet<(i64, i64, String)> = HashSet::new();
                let mut edges = Vec::new();
                for id in &all_memory_ids {
                    let mut stmt = conn.prepare(
                        "SELECT c.from_id, c.to_id, c.edge_type, mf.workspace, mt.workspace
                         FROM crossrefs c
                         JOIN memories mf ON mf.id = c.from_id
                         JOIN memories mt ON mt.id = c.to_id
                         WHERE (c.from_id = ?1 OR c.to_id = ?1)
                           AND mf.valid_to IS NULL
                           AND (mf.expires_at IS NULL OR mf.expires_at > ?2)
                           AND mt.valid_to IS NULL
                           AND (mt.expires_at IS NULL OR mt.expires_at > ?2)",
                    )?;
                    let rows: Vec<(i64, i64, String, String, String)> = stmt
                        .query_map(rusqlite::params![id, &now_rfc3339], |row| {
                            Ok((
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                            ))
                        })?
                        .filter_map(|r| r.ok())
                        .collect();
                    for (source, target, relation, from_ws, to_ws) in rows {
                        if scope.allows(&from_ws, ctx.principal.as_ref())
                            && scope.allows(&to_ws, ctx.principal.as_ref())
                            && seen_edges.insert((source, target, relation.clone()))
                        {
                            edges.push(json!({
                                "source": source,
                                "target": target,
                                "relation": relation
                            }));
                        }
                    }
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

    let (search_opts, scope) =
        match resolve_context_workspace_scope("memory_get_injection_prompt", &params, 20, ctx) {
            Ok(resolved) => resolved,
            Err(err) => return err,
        };

    let query_embedding = ctx.embedder.embed(&query).ok();
    let embedding_ref = query_embedding.as_deref();

    let search_result = ctx.storage.with_connection(|conn| {
        crate::search::hybrid_search_with_hnsw(
            conn,
            &query,
            embedding_ref,
            &search_opts,
            &ctx.search_config,
            Some(&*ctx.hnsw_index.read()),
        )
    });

    let mut memories = match search_result {
        Ok(results) => results,
        Err(e) => return json!({"error": e.to_string()}),
    };
    memories.retain(|r| scope.allows(&r.memory.workspace, ctx.principal.as_ref()));

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

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return json!({"error": "query is required"}),
    };

    let budget = params
        .get("budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(4000) as usize;

    let (search_opts, scope) =
        match resolve_context_workspace_scope("memory_prepare_context", &params, 50, ctx) {
            Ok(resolved) => resolved,
            Err(err) => return err,
        };

    let query_embedding = ctx.embedder.embed(&query).ok();
    let embedding_ref = query_embedding.as_deref();

    let search_result = ctx.storage.with_connection(|conn| {
        crate::search::hybrid_search_with_hnsw(
            conn,
            &query,
            embedding_ref,
            &search_opts,
            &ctx.search_config,
            Some(&*ctx.hnsw_index.read()),
        )
    });

    let memories: Vec<_> = match search_result {
        Ok(results) => results
            .into_iter()
            .map(|r| r.memory)
            .filter(|m| scope.allows(&m.workspace, ctx.principal.as_ref()))
            .collect(),
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
