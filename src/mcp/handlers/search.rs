use std::collections::HashMap;

use serde_json::{json, Value};

use crate::intelligence::memory_policy::{
    blend_retrieval_priority, extract_features, score_policy, PolicyFeatureInput,
};
use crate::mcp::error::ToolError;
use crate::search::{RerankConfig, RerankStrategy, Reranker};
use crate::storage::queries::get_policy_record;
use crate::types::*;

use super::HandlerContext;

#[derive(Clone)]
struct PolicyRerankInfo {
    score: f32,
    blended_score: f32,
    reason: String,
    version: String,
    source: &'static str,
}

impl PolicyRerankInfo {
    fn to_json(&self) -> Value {
        json!({
            "score": self.score,
            "blended_score": self.blended_score,
            "reason": self.reason,
            "version": self.version,
            "source": self.source
        })
    }
}

fn validate_non_negative_int(
    params: &Value,
    name: &str,
) -> std::result::Result<Option<i64>, Value> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => match v.as_i64() {
            Some(val) if val >= 0 => Ok(Some(val)),
            _ => Err(
                ToolError::invalid_params(format!("{name} must be a non-negative integer"))
                    .into_value(),
            ),
        },
    }
}

fn validate_threshold(params: &Value, default: f64) -> std::result::Result<f64, Value> {
    match params.get("threshold") {
        None | Some(Value::Null) => Ok(default),
        Some(v) => match v.as_f64() {
            Some(val) if val.is_finite() && (0.0..=1.0).contains(&val) => Ok(val),
            _ => Err(
                ToolError::invalid_params("threshold must be a number between 0.0 and 1.0")
                    .into_value(),
            ),
        },
    }
}

/// Extract, normalize, and authorize an optional `workspace` parameter against `ctx.principal`.
fn extract_optional_workspace(
    tool_name: &str,
    params: &Value,
    ctx: &HandlerContext,
) -> std::result::Result<Option<String>, Value> {
    if let Some(p) = ctx.principal.as_ref() {
        if let Some(denial) =
            crate::mcp::permission::check_tool_authorization(None, tool_name, params, Some(p))
        {
            return Err(denial);
        }
    }

    match params.get("workspace") {
        None | Some(Value::Null) => {
            if let Some(p) = ctx.principal.as_ref() {
                if !crate::mcp::permission::allows_all_workspaces(p) {
                    if let Some(ns) = p.auth_context().namespace.as_deref() {
                        return Ok(Some(ns.to_string()));
                    }
                    return Ok(Some("default".to_string()));
                }
            }
            Ok(None)
        }
        Some(Value::String(raw)) => {
            let normalized = crate::types::normalize_workspace(raw)
                .map_err(|e| ToolError::invalid_params(e.to_string()).into_value())?;
            if let Some(p) = ctx.principal.as_ref() {
                if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
                    tool_name,
                    p,
                    Some(&normalized),
                ) {
                    return Err(denial);
                }
            }
            Ok(Some(normalized))
        }
        Some(_) => Err(ToolError::invalid_params("workspace must be a string").into_value()),
    }
}

/// Normalize and enforce workspace bounds on `SearchOptions` against `ctx.principal`.
fn enforce_search_options_workspace(
    tool_name: &str,
    params: &Value,
    options: &mut SearchOptions,
    ctx: &HandlerContext,
) -> std::result::Result<(), Value> {
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

    if let Some(p) = ctx.principal.as_ref() {
        if let Some(denial) =
            crate::mcp::permission::check_tool_authorization(None, tool_name, params, Some(p))
        {
            return Err(denial);
        }
    }

    if let Some(raw_ws) = options.workspace.take() {
        let normalized = crate::types::normalize_workspace(&raw_ws)
            .map_err(|e| ToolError::invalid_params(e.to_string()).into_value())?;
        options.workspace = Some(normalized);
    }

    if let Some(raw_list) = options.workspaces.take() {
        let mut normalized_list = Vec::with_capacity(raw_list.len());
        for raw_ws in raw_list {
            let normalized = crate::types::normalize_workspace(&raw_ws)
                .map_err(|e| ToolError::invalid_params(e.to_string()).into_value())?;
            normalized_list.push(normalized);
        }
        options.workspaces = Some(normalized_list);
    }

    if let Some(p) = ctx.principal.as_ref() {
        if !crate::mcp::permission::allows_all_workspaces(p) {
            options.global = false;
            if options.workspace.is_none()
                && options
                    .workspaces
                    .as_ref()
                    .is_none_or(|list| list.is_empty())
            {
                if let Some(ns) = p.auth_context().namespace.as_deref() {
                    options.workspace = Some(ns.to_string());
                } else {
                    options.workspace = Some("default".to_string());
                }
            }
        }
    }

    Ok(())
}

pub fn memory_search(ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::result_cache::CacheFilterParams;

    if let Err(err) = validate_non_negative_int(&params, "limit") {
        return err;
    }
    if let Err(err) = validate_non_negative_int(&params, "offset") {
        return err;
    }

    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let mut options: SearchOptions = match serde_json::from_value(params.clone()) {
        Ok(opts) => opts,
        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
    };
    let policy_rerank = params
        .get("policy_rerank")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let policy_explain = params
        .get("policy_explain")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Global search opt-in: when `global` is true, ignore workspace filters.
    let global_param = params
        .get("global")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if global_param {
        options.global = true;
        options.workspace = None;
        options.workspaces = None;
    }

    if let Err(err) = enforce_search_options_workspace("memory_search", &params, &mut options, ctx)
    {
        return err;
    }
    let global = options.global;

    let rerank_enabled = params
        .get("rerank")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let rerank_strategy = match params.get("rerank_strategy").and_then(|v| v.as_str()) {
        Some("none") => RerankStrategy::None,
        Some("multi_signal") => RerankStrategy::MultiSignal,
        _ => RerankStrategy::Heuristic,
    };

    let effective_rerank_strategy = if rerank_enabled && rerank_strategy != RerankStrategy::None {
        Some(format!("{:?}", rerank_strategy))
    } else {
        None
    };

    let cache_filters = CacheFilterParams {
        workspace: options.workspace.clone(),
        tier: options.tier.map(|t| t.as_str().to_string()),
        memory_types: options.memory_type.map(|t| vec![t]),
        include_archived: options.include_archived,
        include_transcripts: options.include_transcripts,
        tags: options.tags.clone(),
        global,
        rerank_strategy: effective_rerank_strategy,
        policy_rerank,
        limit: options.limit,
        min_score_bits: options.min_score.map(f32::to_bits),
        strategy: options.strategy.map(|s| format!("{s:?}")),
        scope: options
            .scope
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok()),
        workspaces: options.workspaces.clone(),
        scope_path: options.scope_path.clone(),
        filter: options.filter.as_ref().map(|f| f.to_string()),
        offset: options.offset,
    };

    let skip_cache = params
        .get("skip_cache")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || options.explain
        || (policy_rerank && policy_explain);

    // Tier 1 Fast Path: Exact query match without embedding overhead
    if !skip_cache {
        if let Some(cached_results) = ctx.search_cache.get_exact(query, &cache_filters) {
            if global {
                let results_with_ws: Vec<Value> = cached_results
                    .iter()
                    .map(|r| {
                        json!({
                            "memory": r.memory,
                            "score": r.score,
                            "match_info": r.match_info,
                            "workspace": r.memory.workspace
                        })
                    })
                    .collect();
                return json!(results_with_ws);
            }
            return json!(cached_results
                .iter()
                .map(|r| {
                    json!({
                        "memory": r.memory,
                        "score": r.score,
                        "match_info": r.match_info
                    })
                })
                .collect::<Vec<_>>());
        }
    }

    // Generate query embedding for similarity matching and hybrid search
    let query_embedding = ctx.embedder.embed(query).ok();
    let embedding_ref = query_embedding.as_deref();

    // Tier 2 Fast Path: Semantic similarity matching above adaptive threshold
    if !skip_cache {
        if let Some(emb) = embedding_ref {
            if let Some(cached_results) = ctx.search_cache.get_semantic(emb, &cache_filters) {
                if global {
                    let results_with_ws: Vec<Value> = cached_results
                        .iter()
                        .map(|r| {
                            json!({
                                "memory": r.memory,
                                "score": r.score,
                                "match_info": r.match_info,
                                "workspace": r.memory.workspace
                            })
                        })
                        .collect();
                    return json!(results_with_ws);
                }
                return json!(cached_results
                    .iter()
                    .map(|r| {
                        json!({
                            "memory": r.memory,
                            "score": r.score,
                            "match_info": r.match_info
                        })
                    })
                    .collect::<Vec<_>>());
            }
        }
    }

    let mut search_config = ctx.search_config.clone();
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(canonical) = cwd.canonicalize() {
            search_config.project_context_path = Some(canonical.to_string_lossy().to_string());
        }
    }

    let requested_limit = options.limit.unwrap_or(20).max(0) as usize;
    let requested_offset = options.offset.unwrap_or(0).max(0) as usize;
    let mut retrieval_options = options.clone();
    retrieval_options.limit = Some(requested_limit.saturating_add(requested_offset) as i64);
    retrieval_options.offset = None;

    ctx.storage
        .with_connection(|conn| {
            let mut results = crate::search::hybrid_search_with_hnsw(
                conn,
                query,
                embedding_ref,
                &retrieval_options,
                &search_config,
                Some(&*ctx.hnsw_index.read()),
            )?;

            let compute_policy_info =
                |memory: &Memory, base_score: f32| -> crate::error::Result<PolicyRerankInfo> {
                    let stored_policy = get_policy_record(conn, memory.id)?;
                    let (policy_score, source) = if let Some(policy) = stored_policy.as_ref() {
                        (
                            crate::intelligence::memory_policy::PolicyScore {
                                salience_score: policy.salience_score,
                                retention_score: policy.retention_score,
                                retrieval_priority: policy.retrieval_priority,
                                policy_version: policy.policy_version.clone(),
                                policy_reason: policy.policy_reason.clone(),
                            },
                            "stored",
                        )
                    } else {
                        let features = extract_features(PolicyFeatureInput {
                            memory,
                            existing_policy: None,
                            event: None,
                            hybrid_search_score: Some(base_score),
                            session_relevance: None,
                        });
                        (score_policy(&features), "heuristic-v1")
                    };
                    let blended_score =
                        blend_retrieval_priority(base_score, policy_score.retrieval_priority);
                    Ok(PolicyRerankInfo {
                        score: policy_score.retrieval_priority,
                        blended_score,
                        reason: policy_score.policy_reason,
                        version: policy_score.policy_version,
                        source,
                    })
                };

            let mut policy_info_by_memory_id = HashMap::new();

            if rerank_enabled && rerank_strategy != RerankStrategy::None {
                let config = RerankConfig {
                    enabled: true,
                    strategy: rerank_strategy,
                    ..Default::default()
                };
                let reranker = Reranker::with_config(config);
                let mut reranked = reranker.rerank(results, query, Some(conn));

                if policy_rerank {
                    for item in &mut reranked {
                        let info =
                            compute_policy_info(&item.result.memory, item.rerank_info.final_score)?;
                        item.rerank_info.final_score = info.blended_score;
                        item.result.score = info.blended_score;
                        policy_info_by_memory_id.insert(item.result.memory.id, info);
                    }
                    reranked.sort_by(|a, b| {
                        b.rerank_info
                            .final_score
                            .partial_cmp(&a.rerank_info.final_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }

                if requested_offset > 0 {
                    let skip = requested_offset.min(reranked.len());
                    reranked.drain(..skip);
                }
                reranked.truncate(requested_limit);

                if !skip_cache {
                    let cacheable_results: Vec<SearchResult> = reranked
                        .iter()
                        .map(|r| SearchResult {
                            memory: r.result.memory.clone(),
                            score: r.rerank_info.final_score,
                            match_info: r.result.match_info.clone(),
                        })
                        .collect();
                    ctx.search_cache.put(
                        query,
                        query_embedding.clone(),
                        cache_filters.clone(),
                        cacheable_results,
                    );
                }

                if options.explain {
                    Ok(json!({
                        "results": reranked.iter().map(|r| {
                            let mut obj = json!({
                                "memory": r.result.memory,
                                "score": r.rerank_info.final_score,
                                "match_info": r.result.match_info,
                                "rerank_info": r.rerank_info
                            });
                            if global {
                                obj["workspace"] = json!(r.result.memory.workspace);
                            }
                            if policy_rerank && policy_explain {
                                if let Some(policy) = policy_info_by_memory_id.get(&r.result.memory.id) {
                                    obj["policy"] = policy.to_json();
                                }
                            }
                            obj
                        }).collect::<Vec<_>>(),
                        "reranked": true,
                        "strategy": format!("{:?}", rerank_strategy)
                    }))
                } else {
                    Ok(json!(reranked
                        .iter()
                        .map(|r| {
                            let mut obj = json!({
                                "memory": r.result.memory,
                                "score": r.rerank_info.final_score,
                                "match_info": r.result.match_info
                            });
                            if global {
                                obj["workspace"] = json!(r.result.memory.workspace);
                            }
                            if policy_rerank && policy_explain {
                                if let Some(policy) = policy_info_by_memory_id.get(&r.result.memory.id) {
                                    obj["policy"] = policy.to_json();
                                }
                            }
                            obj
                        })
                        .collect::<Vec<_>>()))
                }
            } else {
                if policy_rerank {
                    for result in &mut results {
                        let info = compute_policy_info(&result.memory, result.score)?;
                        result.score = info.blended_score;
                        policy_info_by_memory_id.insert(result.memory.id, info);
                    }
                    results.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }

                if requested_offset > 0 {
                    let skip = requested_offset.min(results.len());
                    results.drain(..skip);
                }
                results.truncate(requested_limit);

                if !skip_cache {
                    ctx.search_cache.put(
                        query,
                        query_embedding.clone(),
                        cache_filters.clone(),
                        results.clone(),
                    );
                }

                let formatted: Vec<Value> = results
                    .iter()
                    .map(|r| {
                        let mut obj = json!({
                            "memory": r.memory,
                            "score": r.score,
                            "match_info": r.match_info
                        });
                        if global {
                            obj["workspace"] = json!(r.memory.workspace);
                        }
                        if policy_rerank && policy_explain {
                            if let Some(policy) = policy_info_by_memory_id.get(&r.memory.id) {
                                obj["policy"] = policy.to_json();
                            }
                        }
                        obj
                    })
                    .collect();

                if options.explain {
                    Ok(json!({
                        "results": formatted,
                        "reranked": false,
                        "strategy": "None"
                    }))
                } else {
                    Ok(json!(formatted))
                }
            }
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn search_suggest(ctx: &HandlerContext, params: Value) -> Value {
    if let Err(err) = validate_non_negative_int(&params, "limit") {
        return err;
    }
    if let Err(err) = extract_optional_workspace("memory_search_suggest", &params, ctx) {
        return err;
    }
    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let fuzzy = ctx.fuzzy_engine.lock();
    let result = fuzzy.correct_query(query);
    json!(result)
}

pub fn memory_search_by_identity(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::search_by_identity;

    let limit = match validate_non_negative_int(&params, "limit") {
        Ok(val) => val.map(|v| v as usize),
        Err(err) => return err,
    };

    let identity = match params.get("identity").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return ToolError::missing_argument("identity").into_value(),
    };

    let workspace = match extract_optional_workspace("memory_search_by_identity", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| {
            let memories = search_by_identity(conn, identity, workspace.as_deref(), limit)?;
            Ok(json!({"memories": memories}))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_session_search(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::search_sessions;

    let limit = match validate_non_negative_int(&params, "limit") {
        Ok(val) => val.map(|v| v as usize),
        Err(err) => return err,
    };

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return ToolError::missing_argument("query").into_value(),
    };

    let session_id = params.get("session_id").and_then(|v| v.as_str());
    let workspace = match extract_optional_workspace("memory_session_search", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| {
            let memories = search_sessions(conn, query, session_id, workspace.as_deref(), limit)?;
            Ok(json!({"memories": memories}))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn find_duplicates(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::find_duplicates_in_workspace;

    let threshold = match validate_threshold(&params, 0.9) {
        Ok(t) => t,
        Err(err) => return err,
    };
    let workspace = match extract_optional_workspace("memory_find_duplicates", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| {
            let duplicates = find_duplicates_in_workspace(conn, threshold, workspace.as_deref())?;
            Ok(json!({
                "count": duplicates.len(),
                "threshold": threshold,
                "duplicates": duplicates
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn find_semantic_duplicates(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::find_duplicates_by_embedding;

    let limit = match validate_non_negative_int(&params, "limit") {
        Ok(val) => val.unwrap_or(50) as usize,
        Err(err) => return err,
    };
    let threshold = match validate_threshold(&params, 0.92) {
        Ok(t) => t as f32,
        Err(err) => return err,
    };
    let workspace =
        match extract_optional_workspace("memory_find_semantic_duplicates", &params, ctx) {
            Ok(ws) => ws,
            Err(err) => return err,
        };

    ctx.storage
        .with_connection(|conn| {
            let duplicates =
                find_duplicates_by_embedding(conn, threshold, workspace.as_deref(), limit)?;
            Ok(json!({
                "count": duplicates.len(),
                "threshold": threshold,
                "method": "embedding_cosine_similarity",
                "duplicates": duplicates
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn search_cache_feedback(ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::CacheFilterParams;

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return ToolError::missing_argument("query").into_value(),
    };

    let positive = match params.get("positive").and_then(|v| v.as_bool()) {
        Some(p) => p,
        None => return ToolError::missing_argument("positive").into_value(),
    };

    let workspace = match extract_optional_workspace("search_cache_feedback", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    let filters = CacheFilterParams {
        workspace,
        rerank_strategy: Some("Heuristic".to_string()),
        ..Default::default()
    };

    ctx.search_cache.record_feedback(query, &filters, positive);
    let new_threshold = ctx.search_cache.current_threshold();

    json!({
        "recorded": true,
        "feedback": if positive { "positive" } else { "negative" },
        "current_threshold": new_threshold
    })
}

pub fn search_cache_stats(ctx: &HandlerContext, _params: Value) -> Value {
    let stats = ctx.search_cache.stats();
    json!(stats)
}

pub fn search_cache_clear(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = match extract_optional_workspace("search_cache_clear", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    if let Some(ws) = workspace.as_deref() {
        ctx.search_cache.invalidate_for_workspace(Some(ws));
        json!({"cleared": true, "scope": "workspace", "workspace": ws})
    } else {
        ctx.search_cache.clear();
        json!({"cleared": true, "scope": "all"})
    }
}

// ── Search Explainability (RML-1242) ────────────────────────────────────────

pub fn memory_explain_search(_ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::explain::SearchExplainer;

    let results = match params.get("results").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            return ToolError::invalid_params("results array is required (each with memory_id, bm25, vector, fuzzy, recency, importance, final_score, and optional rerank_score)").into_value();
        }
    };

    let reranking_active = params
        .get("reranking_active")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let rrf_k = params.get("rrf_k").and_then(|v| v.as_f64()).unwrap_or(60.0) as f32;

    let explainer = SearchExplainer::new(rrf_k, reranking_active);

    let mut batch = Vec::with_capacity(results.len());
    for r in results {
        let Some(memory_id) = r.get("memory_id").and_then(|v| v.as_i64()) else {
            return ToolError::invalid_params(
                "Each result entry must contain an integer 'memory_id'",
            )
            .into_value();
        };
        let bm25 = r.get("bm25").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let vector = r.get("vector").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let fuzzy = r.get("fuzzy").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let recency = r.get("recency").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let importance = r.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let rerank = r
            .get("rerank_score")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);
        let final_score = r.get("final_score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        batch.push((
            memory_id,
            bm25,
            vector,
            fuzzy,
            recency,
            importance,
            rerank,
            final_score,
        ));
    }

    let explanations = explainer.explain_batch(batch);
    json!({
        "count": explanations.len(),
        "explanations": explanations
    })
}

// ── Relevance Feedback (RML-1243) ───────────────────────────────────────────

pub fn memory_feedback(ctx: &HandlerContext, params: Value) -> Value {
    use crate::error::EngramError;
    use crate::search::feedback::{record_feedback, FeedbackSignal};
    use crate::storage::feedback::FeedbackProcessor;
    use rusqlite::OptionalExtension;

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return ToolError::missing_argument("query").into_value(),
    };

    let memory_id = match crate::mcp::handlers::memory_crud::extract_id(&params, "memory_id") {
        Ok(id) => id,
        Err(err) => return err,
    };

    // Accept both legacy aliases (helpful/not_helpful) and canonical names
    // (useful/irrelevant/outdated/conflict). All four canonical signals are
    // distinct in storage and produce different boost magnitudes downstream.
    let (feedback_signal, signal_str): (FeedbackSignal, &str) = match params
        .get("signal")
        .and_then(|v| v.as_str())
    {
        Some("useful") | Some("helpful") => (FeedbackSignal::Useful, "helpful"),
        Some("irrelevant") | Some("not_helpful") => (FeedbackSignal::Irrelevant, "not_helpful"),
        Some("outdated") => (FeedbackSignal::Outdated, "outdated"),
        Some("conflict") => (FeedbackSignal::Conflict, "conflict"),
        _ => {
            return ToolError::invalid_params("signal must be 'helpful'/'useful', 'not_helpful'/'irrelevant', 'outdated', or 'conflict'").into_value();
        }
    };

    let rank_position = params
        .get("rank_position")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let original_score = params
        .get("original_score")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let explicit_workspace: Option<String> = match params.get("workspace") {
        None | Some(Value::Null) => None,
        Some(Value::String(raw)) => {
            let normalized = match crate::types::normalize_workspace(raw) {
                Ok(ws) => ws,
                Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
            };
            if let Some(p) = ctx.principal.as_ref() {
                if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
                    "memory_feedback",
                    p,
                    Some(&normalized),
                ) {
                    return denial;
                }
            }
            Some(normalized)
        }
        Some(_) => return ToolError::invalid_params("workspace must be a string").into_value(),
    };

    ctx.storage
        .with_transaction(|conn| {
            crate::mcp::workspace_guard::ensure_memory_access(
                conn,
                ctx.principal.as_ref(),
                memory_id,
            )?;

            let persisted_workspace: Option<String> = conn
                .query_row(
                    "SELECT workspace FROM memories WHERE id = ?1 AND valid_to IS NULL",
                    [memory_id],
                    |row| row.get(0),
                )
                .optional()?;

            let effective_workspace = match (&persisted_workspace, &explicit_workspace) {
                (Some(persisted), Some(explicit)) => {
                    if persisted != explicit {
                        return Err(EngramError::NotFound(memory_id));
                    }
                    persisted.clone()
                }
                (Some(persisted), None) => persisted.clone(),
                (None, Some(explicit)) => explicit.clone(),
                (None, None) => "default".to_string(),
            };

            // 1. Record the feedback in the search feedback table
            let fb = record_feedback(
                conn,
                query,
                memory_id,
                feedback_signal,
                rank_position,
                original_score,
                &effective_workspace,
            )?;

            // 2. Process feedback through the feedback loop. Attach the
            // process-wide queueing consolidator so low-utility memories are
            // enqueued for the auto-consolidation scheduler to pick up.
            let consolidator =
                std::sync::Arc::new(crate::mcp::handlers::auto_consolidate::QueueingConsolidator);
            let processor = FeedbackProcessor::new().with_consolidator(consolidator);
            let (new_score, scheduled) =
                processor.process_feedback_with_query(memory_id, signal_str, Some(query), conn)?;

            // 3. Spacing-effect stability reinforcement on positive signal
            if feedback_signal == FeedbackSignal::Useful && persisted_workspace.is_some() {
                crate::intelligence::stability::record_reinforcement(
                    conn,
                    memory_id,
                    chrono::Utc::now(),
                )?;
            }

            // 4. Return enriched response
            let mut result = json!(fb);
            if let Some(obj) = result.as_object_mut() {
                obj.insert("utility_score_after".to_string(), json!(new_score));
                obj.insert("scheduled_for_consolidation".to_string(), json!(scheduled));
            }

            Ok(result)
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_feedback_stats(ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::feedback::feedback_stats;

    let workspace = match extract_optional_workspace("memory_feedback_stats", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| {
            let stats = feedback_stats(conn, workspace.as_deref())?;
            Ok(json!(stats))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_explain_utility(ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::utility::UtilityTracker;

    let memory_id = match crate::mcp::handlers::memory_crud::extract_id(&params, "memory_id") {
        Ok(id) => id,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| {
            crate::mcp::workspace_guard::ensure_memory_access(
                conn,
                ctx.principal.as_ref(),
                memory_id,
            )?;
            let tracker = UtilityTracker::new();
            let explanation = tracker.explain_utility(conn, memory_id)?;
            Ok(json!(explanation))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

// ── Compact Search + Expand ──────────────────────────────────────────────────

/// Return a compact summary of search results (id, title, created_at, tags).
///
/// Parameters:
///   - `query` (String, required)
///   - `limit` (u64, optional, default 10)
///   - `workspace` (String, optional)
///
/// The `title` field is the first line of `content` truncated to at most 80 UTF-8
/// bytes on a character boundary, with `"..."` appended if truncated or if
/// additional lines follow.
pub fn memory_search_compact(ctx: &HandlerContext, params: Value) -> Value {
    if let Err(err) = validate_non_negative_int(&params, "limit") {
        return err;
    }
    if let Err(err) = validate_non_negative_int(&params, "offset") {
        return err;
    }

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return ToolError::missing_argument("query").into_value(),
    };

    let mut options: SearchOptions = match serde_json::from_value(params.clone()) {
        Ok(opts) => opts,
        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
    };

    // Global search opt-in: when `global` is true, ignore workspace filters.
    let global_param = params
        .get("global")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if global_param {
        options.global = true;
        options.workspace = None;
        options.workspaces = None;
    }

    if let Err(err) =
        enforce_search_options_workspace("memory_search_compact", &params, &mut options, ctx)
    {
        return err;
    }
    let global = options.global;

    // Apply default limit of 10 when not supplied
    if options.limit.is_none() {
        options.limit = Some(10);
    }

    let query_embedding = ctx.embedder.embed(query).ok();
    let embedding_ref = query_embedding.as_deref();

    let mut search_config = ctx.search_config.clone();
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(canonical) = cwd.canonicalize() {
            search_config.project_context_path = Some(canonical.to_string_lossy().to_string());
        }
    }

    ctx.storage
        .with_connection(|conn| {
            let results = crate::search::hybrid_search_with_hnsw(
                conn,
                query,
                embedding_ref,
                &options,
                &search_config,
                Some(&*ctx.hnsw_index.read()),
            )?;

            let compact: Vec<Value> = results
                .iter()
                .map(|r| {
                    let memory = &r.memory;
                    // Build title: first line of content, capped at 80 UTF-8 bytes on a
                    // char boundary, with "..." appended if the first line was truncated
                    // or if additional lines follow (ignoring a single trailing newline).
                    let first_line = memory.content.lines().next().unwrap_or("");
                    let has_more_lines = memory.content.lines().nth(1).is_some();
                    let title_str = if first_line.len() > 80 {
                        let truncated = crate::text_util::truncate_bytes(first_line, 80);
                        format!("{truncated}...")
                    } else if has_more_lines {
                        format!("{first_line}...")
                    } else {
                        first_line.to_string()
                    };
                    let mut obj = json!({
                        "id": memory.id,
                        "title": title_str,
                        "created_at": memory.created_at,
                        "tags": memory.tags
                    });
                    if global {
                        obj["workspace"] = json!(memory.workspace);
                    }
                    obj
                })
                .collect();

            Ok(json!({
                "results": compact,
                "count": compact.len()
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Return recently created or updated memories for discovery.
///
/// Params:
/// - `workspace` (string, optional) — filter by workspace
/// - `timeframe` (string, optional) — "1h", "24h", "7d", "30d" (default: "24h")
/// - `limit` (integer, optional, default 20, max 100)
/// - `include_types` (array of strings, optional) — filter by memory type
pub fn recent_activity(ctx: &HandlerContext, params: Value) -> Value {
    let workspace_owned = match extract_optional_workspace("recent_activity", &params, ctx) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    let timeframe = match params.get("timeframe") {
        None | Some(Value::Null) => "24h",
        Some(Value::String(s)) if matches!(s.as_str(), "1h" | "24h" | "7d" | "30d") => s.as_str(),
        Some(_) => {
            return ToolError::invalid_params("timeframe must be one of '1h', '24h', '7d', '30d'")
                .into_value();
        }
    };

    let limit = match params.get("limit") {
        None | Some(Value::Null) => 20_i64,
        Some(v) => match v.as_i64() {
            Some(val) if val >= 1 => val.min(100),
            _ => {
                return ToolError::invalid_params("limit must be a positive integer").into_value();
            }
        },
    };

    let include_types: Option<Vec<String>> = match params.get("include_types") {
        None | Some(Value::Null) => None,
        Some(Value::Array(arr)) => {
            let mut types = Vec::with_capacity(arr.len());
            for item in arr {
                let Some(s) = item.as_str() else {
                    return ToolError::invalid_params("include_types must be an array of strings")
                        .into_value();
                };
                types.push(s.to_string());
            }
            Some(types)
        }
        Some(_) => {
            return ToolError::invalid_params("include_types must be an array of strings")
                .into_value();
        }
    };

    let now = chrono::Utc::now();
    let duration = match timeframe {
        "1h" => chrono::Duration::hours(1),
        "24h" => chrono::Duration::hours(24),
        "7d" => chrono::Duration::days(7),
        "30d" => chrono::Duration::days(30),
        _ => chrono::Duration::hours(24),
    };
    let cutoff_rfc3339 = (now - duration).to_rfc3339();
    let now_rfc3339 = now.to_rfc3339();
    let timeframe_owned = timeframe.to_string();

    ctx.storage
        .with_connection(|conn| {
            let mut sql = String::from(
                "SELECT m.id, m.content, m.memory_type, m.importance, m.workspace, \
                 m.created_at, m.updated_at, \
                 (SELECT GROUP_CONCAT(t.name, ',') FROM memory_tags mt \
                  JOIN tags t ON mt.tag_id = t.id WHERE mt.memory_id = m.id) AS tags \
                 FROM memories m \
                 WHERE m.valid_to IS NULL \
                   AND COALESCE(m.updated_at, m.created_at) > ?1 \
                   AND (m.expires_at IS NULL OR m.expires_at > ?2) \
                   AND (m.lifecycle_state IS NULL OR m.lifecycle_state != 'archived') ",
            );

            let mut param_values: Vec<Box<dyn rusqlite::ToSql>> =
                vec![Box::new(cutoff_rfc3339), Box::new(now_rfc3339)];

            if let Some(ref ws) = workspace_owned {
                sql.push_str(&format!("AND m.workspace = ?{} ", param_values.len() + 1));
                param_values.push(Box::new(ws.clone()));
            }

            if let Some(ref types) = include_types {
                if !types.is_empty() {
                    let placeholders: Vec<String> = types
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format!("?{}", param_values.len() + i + 1))
                        .collect();
                    sql.push_str(&format!(
                        "AND m.memory_type IN ({}) ",
                        placeholders.join(",")
                    ));
                    for t in types {
                        param_values.push(Box::new(t.clone()));
                    }
                } else {
                    sql.push_str("AND m.memory_type != 'transcript_chunk' ");
                }
            } else {
                sql.push_str("AND m.memory_type != 'transcript_chunk' ");
            }

            sql.push_str(&format!(
                "ORDER BY COALESCE(m.updated_at, m.created_at) DESC LIMIT ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(limit));

            let mut stmt = conn.prepare(&sql)?;

            let rows = stmt.query_map(
                rusqlite::params_from_iter(param_values.iter().map(|p| p.as_ref())),
                |row| {
                    let content: String = row.get(1)?;
                    let char_count = content.chars().count();
                    let preview: String = content.chars().take(100).collect();
                    let preview = if char_count > 100 {
                        format!("{}...", preview)
                    } else {
                        preview
                    };

                    Ok(json!({
                        "id": row.get::<_, i64>(0)?,
                        "preview": preview,
                        "memory_type": row.get::<_, String>(2)?,
                        "importance": row.get::<_, Option<f64>>(3)?.unwrap_or(0.5),
                        "workspace": row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "default".to_string()),
                        "created_at": row.get::<_, String>(5)?,
                        "updated_at": row.get::<_, Option<String>>(6)?,
                        "tags": row.get::<_, Option<String>>(7)?
                    }))
                },
            )?;

            let activities: Vec<Value> = rows.collect::<rusqlite::Result<Vec<Value>>>()?;
            let count = activities.len();

            Ok(json!({
                "activities": activities,
                "count": count,
                "timeframe": timeframe_owned,
                "workspace": workspace_owned
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Fetch full Memory objects for a list of IDs.
///
/// Parameters:
///   - `ids` (array of integers, required)
///
/// IDs that do not exist are silently skipped.
/// Returns `{memories: [...], found: N, requested: N}`.
pub fn memory_expand(ctx: &HandlerContext, params: Value) -> Value {
    use crate::error::EngramError;
    use crate::storage::queries::get_memory_readonly;

    let ids_arr = match params.get("ids") {
        None | Some(Value::Null) => return ToolError::missing_argument("ids").into_value(),
        Some(Value::Array(arr)) => arr,
        Some(_) => {
            return ToolError::invalid_params("ids must be an array of positive integers")
                .into_value();
        }
    };

    let mut ids = Vec::with_capacity(ids_arr.len());
    for item in ids_arr {
        match item.as_i64() {
            Some(id) if id > 0 => ids.push(id),
            _ => {
                return ToolError::invalid_params("ids must be an array of positive integers")
                    .into_value();
            }
        }
    }

    let requested = ids.len();

    ctx.storage
        .with_connection(|conn| {
            let mut memories: Vec<Value> = Vec::with_capacity(ids.len());
            for id in &ids {
                if let Err(e) = crate::mcp::workspace_guard::ensure_memory_access(
                    conn,
                    ctx.principal.as_ref(),
                    *id,
                ) {
                    match e {
                        EngramError::NotFound(_) => continue,
                        other => return Err(other),
                    }
                }
                match get_memory_readonly(conn, *id) {
                    Ok(memory) => memories.push(json!(memory)),
                    Err(EngramError::NotFound(_)) => {
                        // Skip silently
                    }
                    Err(e) => return Err(e),
                }
            }
            let found = memories.len();
            Ok(json!({
                "memories": memories,
                "found": found,
                "requested": requested
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}
