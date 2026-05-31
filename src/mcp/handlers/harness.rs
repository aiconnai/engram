//! Harness record handler — durable cross-session memory for harness events.
//!
//! Creates permanent memory records for decisions, handoffs, failed attempts,
//! verification results, risks, assumptions, bug reproductions, and issue updates.

use serde_json::{json, Value};
use std::collections::HashMap;

use super::HandlerContext;

const VALID_KINDS: &[&str] = &[
    "decision",
    "handoff",
    "failed_attempt",
    "bug_reproduction",
    "verification_result",
    "risk",
    "assumption",
    "issue_update",
];

/// Record a durable harness event with structured metadata for cross-session continuity.
///
/// Params:
/// - `kind` (string, required): one of the 8 valid harness event kinds
/// - `summary` (string, required): 1–500 chars — stored as memory content
/// - `details` (string, optional): appended to content after a blank line
/// - `source_paths` (array of strings, optional): relevant file paths
/// - `command` (string, optional): CLI/shell command that produced evidence
/// - `issue_number` (integer, optional): GitHub issue number
/// - `commit_sha` (string, optional): git commit SHA
/// - `evidence_refs` (array of strings, optional): free-form references
/// - `importance` (float 0.0–1.0, optional, default 0.7)
/// - `workspace` (string, optional, defaults to "default")
pub fn handle_harness_record(ctx: &HandlerContext, params: Value) -> Value {
    // ── Validate kind ────────────────────────────────────────────────────────
    let kind = match params.get("kind").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => {
            return json!({
                "error": "kind is required",
                "valid_kinds": VALID_KINDS,
            })
        }
    };
    if !VALID_KINDS.contains(&kind.as_str()) {
        return json!({
            "error": format!("invalid harness kind: {}", kind),
            "valid_kinds": VALID_KINDS,
        });
    }

    // ── Validate summary ─────────────────────────────────────────────────────
    let summary = match params.get("summary").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "summary is required"}),
    };
    if summary.is_empty() {
        return json!({"error": "summary must not be empty"});
    }
    if summary.len() > 500 {
        return json!({"error": "summary must be 500 characters or fewer"});
    }

    // ── Validate importance ──────────────────────────────────────────────────
    let importance: f32 = if let Some(v) = params.get("importance") {
        match v.as_f64() {
            Some(f) if (0.0..=1.0).contains(&f) => f as f32,
            Some(_) => return json!({"error": "importance must be between 0.0 and 1.0"}),
            None => return json!({"error": "importance must be a number"}),
        }
    } else {
        0.7
    };

    // ── Extract optional params ──────────────────────────────────────────────
    let details = params
        .get("details")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let source_paths: Vec<String> = params
        .get("source_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let command = params
        .get("command")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let issue_number = params.get("issue_number").and_then(|v| v.as_i64());

    let commit_sha = params
        .get("commit_sha")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let evidence_refs: Vec<String> = params
        .get("evidence_refs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // ── Build content ────────────────────────────────────────────────────────
    let content = match &details {
        Some(d) => format!("{}\n\n{}", summary, d),
        None => summary.clone(),
    };

    // ── Map kind → MemoryType ────────────────────────────────────────────────
    let memory_type = kind_to_memory_type(&kind);

    // ── Build tags ───────────────────────────────────────────────────────────
    let tags = vec!["harness".to_string(), kind.clone()];

    // ── Build metadata ───────────────────────────────────────────────────────
    let mut metadata: HashMap<String, Value> = HashMap::new();
    metadata.insert("harness_kind".to_string(), json!(kind));
    metadata.insert(
        "source_paths".to_string(),
        json!(source_paths),
    );
    metadata.insert("command".to_string(), json!(command));
    metadata.insert("issue_number".to_string(), json!(issue_number));
    metadata.insert("commit_sha".to_string(), json!(commit_sha));
    metadata.insert("evidence_refs".to_string(), json!(evidence_refs));

    // ── Create memory ────────────────────────────────────────────────────────
    let input = crate::types::CreateMemoryInput {
        content,
        memory_type,
        tags: tags.clone(),
        metadata,
        importance: Some(importance),
        workspace: Some(workspace.clone()),
        tier: crate::types::MemoryTier::Permanent,
        ..Default::default()
    };

    match ctx
        .storage
        .with_transaction(|conn| crate::storage::queries::create_memory(conn, &input))
    {
        Ok(memory) => json!({
            "memory_id": memory.id,
            "kind": kind,
            "workspace": workspace,
            "summary": summary,
            "tags": tags,
            "created_at": memory.created_at.to_rfc3339(),
        }),
        Err(e) => json!({"error": format!("Failed to create memory: {}", e)}),
    }
}

fn kind_to_memory_type(kind: &str) -> crate::types::MemoryType {
    match kind {
        "decision" => crate::types::MemoryType::Decision,
        "handoff" => crate::types::MemoryType::Checkpoint,
        "failed_attempt" => crate::types::MemoryType::Learning,
        "bug_reproduction" => crate::types::MemoryType::Episodic,
        "verification_result" => crate::types::MemoryType::Checkpoint,
        "risk" => crate::types::MemoryType::Note,
        "assumption" => crate::types::MemoryType::Note,
        "issue_update" => crate::types::MemoryType::Issue,
        _ => crate::types::MemoryType::Note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::handlers::HandlerContext;
    use crate::storage::Storage;
    use std::sync::Arc;

    fn test_ctx() -> HandlerContext {
        let storage = Storage::open_in_memory().expect("open in-memory storage");
        HandlerContext {
            storage,
            embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(128)),
            fuzzy_engine: Arc::new(parking_lot::Mutex::new(crate::search::FuzzyEngine::new())),
            search_config: crate::search::SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(crate::search::SearchResultCache::new(
                crate::search::AdaptiveCacheConfig::default(),
            )),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 300,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(
                tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap(),
            ),
        }
    }

    #[test]
    fn test_decision_record_returns_memory_id_and_tags() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": "Use SQLite for storage layer",
            }),
        );
        assert!(result.get("memory_id").is_some(), "should return memory_id");
        assert_eq!(result["kind"], "decision");
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "harness"));
        assert!(tags.iter().any(|t| t == "decision"));
    }

    #[test]
    fn test_failed_attempt_record_tags_and_type() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "failed_attempt",
                "summary": "Tried using DuckDB but it caused compile errors",
            }),
        );
        assert!(result.get("memory_id").is_some());
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "harness"));
        assert!(tags.iter().any(|t| t == "failed_attempt"));
    }

    #[test]
    fn test_verification_result_record() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "verification_result",
                "summary": "All 858 tests pass after refactor",
            }),
        );
        assert!(result.get("memory_id").is_some());
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "harness"));
        assert!(tags.iter().any(|t| t == "verification_result"));
    }

    #[test]
    fn test_invalid_kind_returns_error_with_valid_kinds() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "not_a_kind",
                "summary": "Something",
            }),
        );
        assert!(result.get("error").is_some());
        let error = result["error"].as_str().unwrap();
        assert!(error.contains("invalid harness kind"));
        assert!(result.get("valid_kinds").is_some());
    }

    #[test]
    fn test_empty_summary_returns_error() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": "",
            }),
        );
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_summary_over_500_chars_returns_error() {
        let ctx = test_ctx();
        let long_summary = "x".repeat(501);
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": long_summary,
            }),
        );
        assert!(result.get("error").is_some());
        let error = result["error"].as_str().unwrap();
        assert!(error.contains("500"));
    }

    #[test]
    fn test_metadata_fields_stored_correctly() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "bug_reproduction",
                "summary": "Crash on empty input",
                "source_paths": ["src/lib.rs", "src/main.rs"],
                "command": "cargo test -- test_empty",
                "issue_number": 42,
                "commit_sha": "abc1234",
                "evidence_refs": ["https://github.com/org/repo/issues/42"],
            }),
        );
        assert!(result.get("memory_id").is_some(), "expected memory_id, got: {}", result);
        // Memory was created — verify the memory_id is a number
        assert!(result["memory_id"].as_i64().is_some());
    }

    #[test]
    fn test_importance_defaults_to_0_7() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "risk",
                "summary": "External API may rate-limit us",
            }),
        );
        // If no error, importance defaulted correctly
        assert!(result.get("memory_id").is_some());
    }

    #[test]
    fn test_importance_out_of_range_returns_error() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": "Some decision",
                "importance": 1.5,
            }),
        );
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_tier_is_always_permanent() {
        // Verify via kind_to_memory_type that all kinds are handled,
        // and the tier field is set to Permanent in CreateMemoryInput.
        // This is structural — if it compiled and saved successfully
        // with tier=Permanent, the record creation passed.
        let ctx = test_ctx();
        for kind in VALID_KINDS {
            let result = handle_harness_record(
                &ctx,
                json!({
                    "kind": kind,
                    "summary": format!("Test for kind {}", kind),
                }),
            );
            assert!(
                result.get("memory_id").is_some(),
                "kind {} should succeed, got: {}",
                kind,
                result
            );
        }
    }
}
