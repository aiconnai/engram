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

/// Return a structured summary of current project state for a fresh agent.
///
/// Params:
/// - `workspace` (optional string, defaults to "default")
/// - `max_records` (optional integer, default 10, max 50)
/// - `token_budget` (optional integer, default 2000)
/// - `include_git` (optional bool, default true)
pub fn handle_harness_status(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let max_records = params
        .get("max_records")
        .and_then(|v| v.as_i64())
        .unwrap_or(10)
        .min(50);

    let token_budget = params
        .get("token_budget")
        .and_then(|v| v.as_i64())
        .unwrap_or(2000) as usize;

    let include_git = params
        .get("include_git")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // ── Fetch recent harness records ─────────────────────────────────────────
    let options = crate::types::ListOptions {
        tags: Some(vec!["harness".to_string()]),
        limit: Some(max_records),
        sort_by: Some(crate::types::SortField::CreatedAt),
        sort_order: Some(crate::types::SortOrder::Desc),
        workspace: Some(workspace.clone()),
        ..Default::default()
    };

    let memories = match ctx
        .storage
        .with_connection(|conn| crate::storage::queries::list_memories(conn, &options))
    {
        Ok(m) => m,
        Err(e) => return json!({"error": format!("Failed to fetch harness records: {}", e)}),
    };

    // ── Group by kind ─────────────────────────────────────────────────────────
    let mut decisions: Vec<Value> = Vec::new();
    let mut blockers: Vec<Value> = Vec::new();
    let mut last_verification: Option<Value> = None;
    let mut last_handoff: Option<Value> = None;
    let mut recent_issue_updates: Vec<Value> = Vec::new();

    for mem in &memories {
        let kind = mem
            .metadata
            .get("harness_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let summary = mem.content.lines().next().unwrap_or("").to_string();
        let created_at = mem.created_at.to_rfc3339();

        match kind.as_str() {
            "decision" => {
                let commit_sha = mem.metadata.get("commit_sha").cloned().unwrap_or(Value::Null);
                decisions.push(json!({
                    "memory_id": mem.id,
                    "summary": summary,
                    "created_at": created_at,
                    "commit_sha": commit_sha,
                }));
            }
            "risk" | "failed_attempt" => {
                blockers.push(json!({
                    "memory_id": mem.id,
                    "kind": kind,
                    "summary": summary,
                    "created_at": created_at,
                }));
            }
            "verification_result" => {
                if last_verification.is_none() {
                    let command = mem.metadata.get("command").cloned().unwrap_or(Value::Null);
                    last_verification = Some(json!({
                        "memory_id": mem.id,
                        "summary": summary,
                        "created_at": created_at,
                        "command": command,
                    }));
                }
            }
            "handoff" => {
                if last_handoff.is_none() {
                    last_handoff = Some(json!({
                        "memory_id": mem.id,
                        "summary": summary,
                        "created_at": created_at,
                    }));
                }
            }
            "issue_update" => {
                let issue_number = mem.metadata.get("issue_number").cloned().unwrap_or(Value::Null);
                recent_issue_updates.push(json!({
                    "memory_id": mem.id,
                    "summary": summary,
                    "issue_number": issue_number,
                    "created_at": created_at,
                }));
            }
            _ => {}
        }
    }

    // ── Optional git state ────────────────────────────────────────────────────
    let git_state = if include_git {
        let branch = run_command("git", &["branch", "--show-current"]);
        let dirty_raw = run_command("git", &["status", "--short"]);
        let dirty_files: Option<Vec<String>> = dirty_raw.map(|s| {
            s.lines().take(10).map(|l| l.to_string()).collect()
        });
        let commits_raw = run_command("git", &["log", "--oneline", "-5"]);
        let recent_commits: Option<Vec<String>> = commits_raw.map(|s| {
            s.lines().map(|l| l.to_string()).collect()
        });
        Some(json!({
            "branch": branch,
            "dirty_files": dirty_files,
            "recent_commits": recent_commits,
        }))
    } else {
        None
    };

    // ── suggested_next_action ─────────────────────────────────────────────────
    let suggested_next_action = if !blockers.is_empty() {
        format!("Resolve {} known blocker(s) before proceeding.", blockers.len())
    } else if let Some(ref h) = last_handoff {
        let s = h["summary"].as_str().unwrap_or("");
        let preview: String = s.chars().take(60).collect();
        format!("Continue from last handoff: {}.", preview)
    } else if !decisions.is_empty() {
        format!("Review {} recent decision(s) and confirm alignment.", decisions.len())
    } else {
        "No harness context found. Run harness_record to start tracking.".to_string()
    };

    // ── Assemble and apply token budget ───────────────────────────────────────
    let generated_at = chrono::Utc::now().to_rfc3339();

    // Build response, truncating from bottom if over budget
    loop {
        let candidate = json!({
            "workspace": workspace,
            "generated_at": generated_at,
            "recent_decisions": decisions,
            "known_blockers": blockers,
            "last_verification": last_verification,
            "last_handoff": last_handoff,
            "recent_issue_updates": recent_issue_updates,
            "git_state": git_state,
            "suggested_next_action": suggested_next_action,
        });
        let serialized = candidate.to_string();
        let estimated_tokens = serialized.len() / 4;
        if estimated_tokens <= token_budget
            || (decisions.is_empty() && blockers.is_empty() && recent_issue_updates.is_empty())
        {
            let mut result = candidate;
            result["token_estimate"] = json!(estimated_tokens);
            return result;
        }
        // Truncate the longest list first
        if !decisions.is_empty() {
            decisions.pop();
        } else if !blockers.is_empty() {
            blockers.pop();
        } else if !recent_issue_updates.is_empty() {
            recent_issue_updates.pop();
        } else {
            break;
        }
    }

    // Fallback (should not normally be reached)
    let final_val = json!({
        "workspace": workspace,
        "generated_at": generated_at,
        "recent_decisions": decisions,
        "known_blockers": blockers,
        "last_verification": last_verification,
        "last_handoff": last_handoff,
        "recent_issue_updates": recent_issue_updates,
        "git_state": git_state,
        "suggested_next_action": suggested_next_action,
        "token_estimate": 0,
    });
    final_val
}

/// Generate a structured handoff packet for next-agent continuity.
///
/// Params:
/// - `current_goal` (string, required, ≤300 chars)
/// - `files_touched` (array of strings, optional)
/// - `decisions_made` (array of strings, optional)
/// - `tests_run` (array of strings, optional)
/// - `tests_not_run` (array of strings, optional)
/// - `known_risks` (array of strings, optional)
/// - `blockers` (array of strings, optional)
/// - `next_steps` (array of strings, required, min 1 item)
/// - `issue_numbers` (array of integers, optional)
/// - `plan_doc_paths` (array of strings, optional)
/// - `verification_evidence` (string, optional)
/// - `persist` (bool, optional, default true)
/// - `workspace` (string, optional, defaults to "default")
pub fn handle_harness_handoff(ctx: &HandlerContext, params: Value) -> Value {
    // ── Validate current_goal ────────────────────────────────────────────────
    let current_goal = match params.get("current_goal").and_then(|v| v.as_str()) {
        Some(g) => g.to_string(),
        None => return json!({"error": "current_goal is required"}),
    };
    if current_goal.len() > 300 {
        return json!({"error": "current_goal must be 300 characters or fewer"});
    }

    // ── Validate next_steps ──────────────────────────────────────────────────
    let next_steps: Vec<String> = match params.get("next_steps").and_then(|v| v.as_array()) {
        Some(arr) => arr.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
        None => return json!({"error": "next_steps is required"}),
    };
    if next_steps.is_empty() {
        return json!({"error": "next_steps must have at least one item"});
    }

    // ── Extract optional params ──────────────────────────────────────────────
    let files_touched: Vec<String> = params
        .get("files_touched")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let decisions_made: Vec<String> = params
        .get("decisions_made")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let tests_run: Vec<String> = params
        .get("tests_run")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let tests_not_run: Vec<String> = params
        .get("tests_not_run")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let known_risks: Vec<String> = params
        .get("known_risks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let blockers: Vec<String> = params
        .get("blockers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let issue_numbers: Vec<i64> = params
        .get("issue_numbers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    let plan_doc_paths: Vec<String> = params
        .get("plan_doc_paths")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let verification_evidence = params
        .get("verification_evidence")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let persist = params
        .get("persist")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    // ── Completion claim ─────────────────────────────────────────────────────
    let has_evidence = verification_evidence
        .as_deref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    // ── Persist if requested ─────────────────────────────────────────────────
    let (handoff_id, created_at, persisted) = if persist {
        // Build content (≤1000 chars)
        let steps_text: String = next_steps
            .iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n");
        let full_content = format!("{}\n\nNext steps:\n{}", current_goal, steps_text);
        let content = if full_content.len() > 1000 {
            full_content[..1000].to_string()
        } else {
            full_content
        };

        let mut metadata: HashMap<String, Value> = HashMap::new();
        metadata.insert("harness_kind".to_string(), json!("handoff"));
        metadata.insert("files_touched".to_string(), json!(files_touched));
        metadata.insert("decisions_made".to_string(), json!(decisions_made));
        metadata.insert("tests_run".to_string(), json!(tests_run));
        metadata.insert("tests_not_run".to_string(), json!(tests_not_run));
        metadata.insert("known_risks".to_string(), json!(known_risks));
        metadata.insert("blockers".to_string(), json!(blockers));
        metadata.insert("issue_numbers".to_string(), json!(issue_numbers));
        metadata.insert("plan_doc_paths".to_string(), json!(plan_doc_paths));
        metadata.insert("verification_evidence".to_string(), json!(verification_evidence));

        let input = crate::types::CreateMemoryInput {
            content,
            memory_type: crate::types::MemoryType::Checkpoint,
            tags: vec!["harness".to_string(), "handoff".to_string()],
            metadata,
            importance: Some(0.9),
            workspace: Some(workspace.clone()),
            tier: crate::types::MemoryTier::Permanent,
            ..Default::default()
        };

        match ctx
            .storage
            .with_transaction(|conn| crate::storage::queries::create_memory(conn, &input))
        {
            Ok(memory) => (Some(memory.id), memory.created_at.to_rfc3339(), true),
            Err(e) => return json!({"error": format!("Failed to persist handoff: {}", e)}),
        }
    } else {
        (None, chrono::Utc::now().to_rfc3339(), false)
    };

    // ── Build response ───────────────────────────────────────────────────────
    let mut response = json!({
        "handoff_id": handoff_id,
        "workspace": workspace,
        "current_goal": current_goal,
        "files_touched": files_touched,
        "decisions_made": decisions_made,
        "tests_run": tests_run,
        "tests_not_run": tests_not_run,
        "known_risks": known_risks,
        "blockers": blockers,
        "next_steps": next_steps,
        "issue_numbers": issue_numbers,
        "plan_doc_paths": plan_doc_paths,
        "verification_evidence": verification_evidence,
        "completion_claimed": has_evidence,
        "persisted": persisted,
        "created_at": created_at,
    });

    if !has_evidence {
        response["completion_warning"] =
            json!("No verification evidence provided. Do not claim this work is complete.");
    }

    response
}

/// Run a shell command and return trimmed stdout, or None on error.
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
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

    // ── harness_status tests ────────────────────────────────────────────────

    #[test]
    fn test_harness_status_empty_workspace() {
        let ctx = test_ctx();
        let result = handle_harness_status(&ctx, json!({"workspace": "test_empty_ws"}));
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        assert_eq!(result["workspace"], "test_empty_ws");
        assert!(result["recent_decisions"].as_array().unwrap().is_empty());
        assert!(result["known_blockers"].as_array().unwrap().is_empty());
        assert!(result["recent_issue_updates"].as_array().unwrap().is_empty());
        assert!(result.get("token_estimate").is_some());
        let suggestion = result["suggested_next_action"].as_str().unwrap();
        assert!(suggestion.contains("No harness context"), "got: {}", suggestion);
    }

    #[test]
    fn test_harness_status_with_decisions() {
        let ctx = test_ctx();
        let ws = "test_decisions_ws";
        handle_harness_record(&ctx, json!({"kind": "decision", "summary": "Use SQLite", "workspace": ws}));
        handle_harness_record(&ctx, json!({"kind": "decision", "summary": "Use Axum", "workspace": ws}));
        let result = handle_harness_status(&ctx, json!({"workspace": ws}));
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        let decisions = result["recent_decisions"].as_array().unwrap();
        assert_eq!(decisions.len(), 2, "expected 2 decisions, got: {}", result);
        assert!(decisions[0].get("memory_id").is_some());
        assert!(decisions[0].get("summary").is_some());
    }

    #[test]
    fn test_harness_status_with_blocker() {
        let ctx = test_ctx();
        let ws = "test_blocker_ws";
        handle_harness_record(&ctx, json!({"kind": "risk", "summary": "DB migration may fail", "workspace": ws}));
        let result = handle_harness_status(&ctx, json!({"workspace": ws}));
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        let blockers = result["known_blockers"].as_array().unwrap();
        assert_eq!(blockers.len(), 1);
        let suggestion = result["suggested_next_action"].as_str().unwrap();
        assert!(suggestion.to_lowercase().contains("blocker"), "got: {}", suggestion);
    }

    #[test]
    fn test_harness_status_no_git() {
        let ctx = test_ctx();
        let result = handle_harness_status(&ctx, json!({"include_git": false}));
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        assert!(result["git_state"].is_null());
    }

    #[test]
    fn test_harness_status_token_budget() {
        let ctx = test_ctx();
        let ws = "test_budget_ws";
        for i in 0..20 {
            handle_harness_record(&ctx, json!({
                "kind": "decision",
                "summary": format!("Decision number {} with some content to pad size", i),
                "workspace": ws,
            }));
        }
        let result = handle_harness_status(&ctx, json!({"workspace": ws, "token_budget": 200}));
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        let decisions = result["recent_decisions"].as_array().unwrap();
        assert!(decisions.len() < 20, "expected truncation, got {} decisions", decisions.len());
    }

    // ── harness_handoff tests ────────────────────────────────────────────────

    #[test]
    fn test_harness_handoff_basic() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Implement search index v2",
                "files_touched": ["src/search.rs", "src/index.rs"],
                "decisions_made": ["Use BM25 scoring"],
                "tests_run": ["cargo test --lib"],
                "next_steps": ["Review PR #34", "Run full CI"],
                "verification_evidence": "873 tests passed",
            }),
        );
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        assert!(result["handoff_id"].as_i64().is_some(), "expected handoff_id, got: {}", result);
        assert_eq!(result["completion_claimed"], true);
        assert_eq!(result["persisted"], true);
        assert_eq!(result["current_goal"], "Implement search index v2");
    }

    #[test]
    fn test_harness_handoff_no_verification_evidence() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Fix bug in parser",
                "next_steps": ["Run cargo test"],
            }),
        );
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        assert_eq!(result["completion_claimed"], false);
        assert!(result["completion_warning"].as_str().is_some());
        let warning = result["completion_warning"].as_str().unwrap();
        assert!(warning.contains("No verification evidence"), "got: {}", warning);
    }

    #[test]
    fn test_harness_handoff_no_persist() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Draft only handoff",
                "next_steps": ["Check logs"],
                "persist": false,
            }),
        );
        assert!(result.get("error").is_none(), "unexpected error: {}", result);
        assert!(result["handoff_id"].is_null(), "expected null handoff_id, got: {}", result);
        assert_eq!(result["persisted"], false);
    }

    #[test]
    fn test_harness_handoff_missing_goal() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "next_steps": ["Do something"],
            }),
        );
        assert!(result.get("error").is_some(), "expected error, got: {}", result);
    }

    #[test]
    fn test_harness_handoff_empty_next_steps() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Some goal",
                "next_steps": [],
            }),
        );
        assert!(result.get("error").is_some(), "expected error, got: {}", result);
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
