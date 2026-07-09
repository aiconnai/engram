//! `harness_verify` — record verification command outcomes.

use serde_json::{json, Value};
use std::collections::HashMap;

use super::super::HandlerContext;

/// Record a verification command outcome with exit code, output summary, and optional evidence.
///
/// Params:
/// - `command` (string, required, ≤200 chars): the command that was run
/// - `exit_code` (integer, required): 0 = success, non-zero = failure
/// - `passed` (bool, optional): explicit pass/fail; derived from exit_code == 0 if absent
/// - `output_summary` (string, required, ≤500 chars): concise summary
/// - `evidence_path` (string, optional): path to full output file or log
/// - `evidence_hash` (string, optional): SHA256 of full output for integrity
/// - `skipped_reason` (string, optional): if skipped, why
/// - `issue_numbers` (array of integers, optional)
/// - `memory_ids` (array of integers, optional)
/// - `workspace` (string, optional, defaults to "default")
/// - `importance` (float 0.0–1.0, optional, default 0.8)
pub fn handle_harness_verify(ctx: &HandlerContext, params: Value) -> Value {
    // ── Validate command ─────────────────────────────────────────────────────
    let command = match params.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return json!({"error": "command is required"}),
    };
    if command.is_empty() {
        return json!({"error": "command must not be empty"});
    }
    if command.len() > 200 {
        return json!({"error": "command must be 200 characters or fewer"});
    }

    // ── Validate exit_code ───────────────────────────────────────────────────
    let exit_code = match params.get("exit_code").and_then(|v| v.as_i64()) {
        Some(c) => c,
        None => return json!({"error": "exit_code is required and must be an integer"}),
    };

    // ── Validate output_summary ──────────────────────────────────────────────
    let output_summary = match params.get("output_summary").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "output_summary is required"}),
    };
    if output_summary.is_empty() {
        return json!({"error": "output_summary must not be empty"});
    }
    if output_summary.len() > 500 {
        return json!({"error": "output_summary must be 500 characters or fewer"});
    }

    // ── Validate importance ──────────────────────────────────────────────────
    let importance: f32 = if let Some(v) = params.get("importance") {
        match v.as_f64() {
            Some(f) if (0.0..=1.0).contains(&f) => f as f32,
            Some(_) => return json!({"error": "importance must be between 0.0 and 1.0"}),
            None => return json!({"error": "importance must be a number"}),
        }
    } else {
        0.8
    };

    // ── Extract optional params ──────────────────────────────────────────────
    let skipped_reason = params
        .get("skipped_reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let skipped = skipped_reason.is_some();

    let passed = if skipped {
        // When skipped, passed is false (not a true pass)
        params
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        params
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(exit_code == 0)
    };

    let evidence_path = params
        .get("evidence_path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let evidence_hash = params
        .get("evidence_hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let issue_numbers: Vec<i64> = params
        .get("issue_numbers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    let memory_ids: Vec<i64> = params
        .get("memory_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    // ── Build content ────────────────────────────────────────────────────────
    let result_label = if skipped {
        "SKIP"
    } else if passed {
        "PASS"
    } else {
        "FAIL"
    };
    let content = format!(
        "{}\n\nResult: {}\n{}",
        command, result_label, output_summary
    );

    // ── Build tags ───────────────────────────────────────────────────────────
    let mut tags = vec!["harness".to_string(), "verification_result".to_string()];
    if !passed && !skipped {
        tags.push("verification_failed".to_string());
    }
    if skipped {
        tags.push("verification_skipped".to_string());
    }

    // ── Build metadata ───────────────────────────────────────────────────────
    let mut metadata: HashMap<String, Value> = HashMap::new();
    metadata.insert("harness_kind".to_string(), json!("verification_result"));
    metadata.insert("command".to_string(), json!(command));
    metadata.insert("exit_code".to_string(), json!(exit_code));
    metadata.insert("passed".to_string(), json!(passed));
    metadata.insert("skipped".to_string(), json!(skipped));
    metadata.insert("skipped_reason".to_string(), json!(skipped_reason));
    metadata.insert("evidence_path".to_string(), json!(evidence_path));
    metadata.insert("evidence_hash".to_string(), json!(evidence_hash));
    metadata.insert("issue_numbers".to_string(), json!(issue_numbers));
    metadata.insert("memory_ids".to_string(), json!(memory_ids));

    // ── Create memory ────────────────────────────────────────────────────────
    let input = crate::types::CreateMemoryInput {
        content,
        memory_type: crate::types::MemoryType::Checkpoint,
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
            "command": command,
            "exit_code": exit_code,
            "passed": passed,
            "skipped": skipped,
            "output_summary": output_summary,
            "evidence_path": evidence_path,
            "evidence_hash": evidence_hash,
            "tags": tags,
            "workspace": workspace,
            "created_at": memory.created_at.to_rfc3339(),
        }),
        Err(e) => json!({"error": format!("Failed to create memory: {}", e)}),
    }
}
