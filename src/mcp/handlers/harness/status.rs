//! `harness_status` — structured project-state summary for a fresh agent.

use serde_json::{json, Value};

use super::super::HandlerContext;
use super::run_command;

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
                let commit_sha = mem
                    .metadata
                    .get("commit_sha")
                    .cloned()
                    .unwrap_or(Value::Null);
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
            "verification_result" if last_verification.is_none() => {
                let command = mem.metadata.get("command").cloned().unwrap_or(Value::Null);
                last_verification = Some(json!({
                    "memory_id": mem.id,
                    "summary": summary,
                    "created_at": created_at,
                    "command": command,
                }));
            }
            "verification_result" => {}
            "handoff" if last_handoff.is_none() => {
                let handoff_summary = mem
                    .metadata
                    .get("current_goal")
                    .and_then(|value| value.as_str())
                    .unwrap_or(&summary)
                    .to_string();
                last_handoff = Some(json!({
                    "memory_id": mem.id,
                    "summary": handoff_summary,
                    "created_at": created_at,
                    "metadata": mem.metadata.clone(),
                }));
            }
            "handoff" => {}
            "issue_update" => {
                let issue_number = mem
                    .metadata
                    .get("issue_number")
                    .cloned()
                    .unwrap_or(Value::Null);
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
        let dirty_files: Option<Vec<String>> =
            dirty_raw.map(|s| s.lines().take(10).map(|l| l.to_string()).collect());
        let commits_raw = run_command("git", &["log", "--oneline", "-5"]);
        let recent_commits: Option<Vec<String>> =
            commits_raw.map(|s| s.lines().map(|l| l.to_string()).collect());
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
        format!(
            "Resolve {} known blocker(s) before proceeding.",
            blockers.len()
        )
    } else if let Some(ref h) = last_handoff {
        let s = h["summary"].as_str().unwrap_or("");
        let preview: String = s.chars().take(60).collect();
        format!("Continue from last handoff: {}.", preview)
    } else if !decisions.is_empty() {
        format!(
            "Review {} recent decision(s) and confirm alignment.",
            decisions.len()
        )
    } else {
        "No harness context found. Run harness_record to start tracking.".to_string()
    };

    // ── Assemble and apply token budget ───────────────────────────────────────
    let generated_at = chrono::Utc::now().to_rfc3339();

    // Build response, truncating from bottom if over budget
    loop {
        // current_objective: extracted from the most recent handoff's current_goal
        let current_objective = last_handoff
            .as_ref()
            .and_then(|h| {
                h["metadata"]["current_goal"]
                    .as_str()
                    .or_else(|| h["summary"].as_str())
            })
            .map(|s| s.chars().take(200).collect::<String>());

        let candidate = json!({
            "workspace": workspace,
            "generated_at": generated_at,
            "current_objective": current_objective,
            "active_issues": recent_issue_updates,
            "recent_decisions": decisions,
            "known_blockers": blockers,
            "last_verification": last_verification,
            "last_handoff": last_handoff,
            "git_state": git_state,
            "suggested_next_action": suggested_next_action,
        });
        let serialized = candidate.to_string();
        // Token budget enforced with chars/4 heuristic (not BPE).
        // Actual tiktoken count may differ by ~20-30%.
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
    let current_objective = last_handoff
        .as_ref()
        .and_then(|h| {
            h["metadata"]["current_goal"]
                .as_str()
                .or_else(|| h["summary"].as_str())
        })
        .map(|s| s.chars().take(200).collect::<String>());

    json!({
        "workspace": workspace,
        "generated_at": generated_at,
        "current_objective": current_objective,
        "active_issues": recent_issue_updates,
        "recent_decisions": decisions,
        "known_blockers": blockers,
        "last_verification": last_verification,
        "last_handoff": last_handoff,
        "git_state": git_state,
        "suggested_next_action": suggested_next_action,
        "token_estimate": 0,
    })
}
