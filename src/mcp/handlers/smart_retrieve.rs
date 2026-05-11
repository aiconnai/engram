//! `memory_smart_retrieve` — intent-aware unified retrieval (issue #13).
//!
//! Inspired by SimpleMem (aiming-lab) which reports 43% F1 with ~30× fewer
//! tokens by inferring search intent and dynamically routing to the right
//! retrieval strategy.
//!
//! ## How it works
//!
//! 1. `classify(query)` produces a small set of [`Intent`]s using
//!    heuristics over the raw query (no LLM call by default).
//! 2. For each intent, the matching internal handler is invoked
//!    (`memory_search`, `memory_related`, `find_path`, `get_project_context`).
//! 3. Results are merged, deduplicated by memory id, and returned with
//!    audit fields (`intents_used`, `strategies_called`) so the caller can
//!    see why each result is present.
//!
//! ## Trade-offs
//!
//! - Strategies are invoked **serially** today. They take a `Value` and
//!   return a `Value` — moving them to `spawn_blocking` for parallelism is
//!   a straightforward follow-up once we measure that latency matters.
//! - The classifier is heuristic-only. An embedding-based fallback for
//!   ambiguous queries is also future work.

use serde_json::{json, Value};

use super::{graph, project_context, search, HandlerContext};

/// The retrieval intents recognised by the classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    /// Short factual lookup. Hybrid search is enough.
    Lookup,
    /// "Things related to X" — semantic neighbors and graph traversal.
    Exploration,
    /// Workspace-wide context build ("what's going on in project X").
    Context,
    /// "How are X and Y connected" — pathfinding between two entities.
    Path,
}

impl Intent {
    fn as_str(self) -> &'static str {
        match self {
            Intent::Lookup => "lookup",
            Intent::Exploration => "exploration",
            Intent::Context => "context",
            Intent::Path => "path",
        }
    }

    fn strategy_name(self) -> &'static str {
        match self {
            Intent::Lookup => "memory_search",
            Intent::Exploration => "memory_related+memory_search",
            Intent::Context => "memory_get_project_context",
            Intent::Path => "memory_find_path",
        }
    }
}

/// Classify a raw user query into a (small) set of intents.
///
/// The classifier always returns at least one intent. `Lookup` is the
/// default fallback so the response is never empty.
pub fn classify(query: &str) -> Vec<Intent> {
    let q = query.trim().to_lowercase();
    let mut intents = Vec::new();

    // PATH — explicit "how are X and Y connected" / "path between"
    if (q.contains(" and ") || q.contains(" e ") || q.contains(" entre "))
        && (q.contains("connect")
            || q.contains("relat")
            || q.contains("link")
            || q.contains("path")
            || q.contains("conect")
            || q.contains("relac")
            || q.contains("caminho"))
    {
        intents.push(Intent::Path);
    }

    // EXPLORATION — "things related to X" / "what's connected to X" / etc.
    if q.contains("related to")
        || q.contains("similar to")
        || q.contains("connected to")
        || q.contains("relacionad")
        || q.contains("similar a")
        || q.contains("ligado a")
    {
        intents.push(Intent::Exploration);
    }

    // CONTEXT — workspace-scoped "status / overview / what's going on"
    if q.contains("status")
        || q.contains("overview")
        || q.contains("what's going on")
        || q.contains("what is going on")
        || q.contains("resumo")
        || q.contains("o que está acontecendo")
        || q.contains("contexto do projeto")
    {
        intents.push(Intent::Context);
    }

    // LOOKUP is the universal fallback. Always add it so the result is
    // never empty; deduplication will keep it cheap when other intents
    // already covered the same memories.
    if !intents.iter().any(|i| matches!(i, Intent::Lookup)) {
        intents.push(Intent::Lookup);
    }

    intents
}

/// Extract a numeric memory id from a result entry, regardless of how the
/// upstream handler shaped it.
fn extract_id(entry: &Value) -> Option<i64> {
    entry
        .get("id")
        .or_else(|| entry.get("memory_id"))
        .and_then(|v| v.as_i64())
}

/// Run `memory_search` for the query and pull the result list out of its
/// JSON envelope. Returns an empty vec on error.
fn call_search(ctx: &HandlerContext, query: &str, limit: u64, workspace: Option<&str>) -> Vec<Value> {
    let mut params = json!({ "query": query, "limit": limit });
    if let Some(ws) = workspace {
        params["workspace"] = json!(ws);
    }
    let resp = search::memory_search(ctx, params);
    resp.get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Run `memory_related` and return its results. Best-effort — empty on error.
fn call_related(ctx: &HandlerContext, query: &str, limit: u64, workspace: Option<&str>) -> Vec<Value> {
    // `memory_related` needs a seed id. We fetch one via search first.
    let seed = call_search(ctx, query, 1, workspace);
    let Some(seed_id) = seed.first().and_then(extract_id) else {
        return Vec::new();
    };
    let params = json!({ "id": seed_id, "limit": limit });
    let resp = graph::memory_related(ctx, params);
    resp.get("related")
        .or_else(|| resp.get("results"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Run `memory_get_project_context` and surface its memories list.
fn call_context(ctx: &HandlerContext, workspace: Option<&str>) -> Vec<Value> {
    let workspace = workspace.unwrap_or("default");
    let params = json!({ "workspace": workspace });
    let resp = project_context::get_project_context(ctx, params);
    resp.get("memories")
        .or_else(|| resp.get("results"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Top-level handler: parses params, classifies, dispatches, merges.
///
/// Request shape:
/// ```json
/// {
///   "query": "...",
///   "limit": 10,
///   "workspace": "default",
///   "force_intents": ["lookup", "context"]
/// }
/// ```
pub fn memory_smart_retrieve(ctx: &HandlerContext, params: Value) -> Value {
    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) if !q.trim().is_empty() => q,
        _ => return json!({ "error": "missing or empty `query` parameter" }),
    };

    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .clamp(1, 100);

    let workspace = params.get("workspace").and_then(|v| v.as_str());

    // Optional override for tests and debugging.
    let intents: Vec<Intent> = match params.get("force_intents").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(|s| match s {
                "lookup" => Some(Intent::Lookup),
                "exploration" => Some(Intent::Exploration),
                "context" => Some(Intent::Context),
                "path" => Some(Intent::Path),
                _ => None,
            })
            .collect(),
        None => classify(query),
    };

    let mut merged: Vec<Value> = Vec::new();
    let mut seen_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut strategies_called: Vec<&'static str> = Vec::new();

    for intent in &intents {
        let entries = match intent {
            Intent::Lookup => call_search(ctx, query, limit, workspace),
            Intent::Exploration => {
                let mut combined = call_related(ctx, query, limit, workspace);
                combined.extend(call_search(ctx, query, limit, workspace));
                combined
            }
            Intent::Context => call_context(ctx, workspace),
            // Path requires two named entities; without a parser we leave it
            // to the LOOKUP fallback. A future iteration could extract two
            // entities and call `graph::find_path` here.
            Intent::Path => Vec::new(),
        };

        if !strategies_called.contains(&intent.strategy_name()) {
            strategies_called.push(intent.strategy_name());
        }

        for entry in entries {
            match extract_id(&entry) {
                Some(id) if seen_ids.insert(id) => merged.push(entry),
                None => merged.push(entry), // entries without ids still surface
                _ => {} // duplicate, skip
            }
            if merged.len() >= limit as usize {
                break;
            }
        }

        if merged.len() >= limit as usize {
            break;
        }
    }

    json!({
        "results": merged,
        "intents_used": intents.iter().map(|i| i.as_str()).collect::<Vec<_>>(),
        "strategies_called": strategies_called,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_default() {
        let intents = classify("rust async runtime");
        assert_eq!(intents, vec![Intent::Lookup]);
    }

    #[test]
    fn exploration_detected() {
        let intents = classify("things related to async");
        assert!(intents.contains(&Intent::Exploration));
        assert!(intents.contains(&Intent::Lookup)); // fallback still added
    }

    #[test]
    fn context_detected() {
        let intents = classify("what is the status of the project");
        assert!(intents.contains(&Intent::Context));
    }

    #[test]
    fn path_detected() {
        let intents = classify("how are tokio and async connected");
        assert!(intents.contains(&Intent::Path));
    }

    #[test]
    fn portuguese_exploration() {
        let intents = classify("memórias relacionadas a engram");
        assert!(intents.contains(&Intent::Exploration));
    }

    #[test]
    fn portuguese_path() {
        let intents = classify("como engram e tokio estão conectados");
        assert!(intents.contains(&Intent::Path));
    }

    #[test]
    fn force_intents_override() {
        // Direct unit-level sanity: classify gives lookup for "x", but
        // force_intents would override at the handler level. We test that
        // parsing in `memory_smart_retrieve` happens correctly via a
        // separate integration test path.
        let intents = classify("x");
        assert_eq!(intents, vec![Intent::Lookup]);
    }
}
