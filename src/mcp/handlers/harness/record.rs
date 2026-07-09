//! `harness_record` — durable cross-session memory for harness events.

use serde_json::{json, Value};
use std::collections::HashMap;

use super::super::HandlerContext;
use super::{kind_to_memory_type, VALID_KINDS};

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
    let details = match params.get("details").and_then(|v| v.as_str()) {
        Some(d) if d.len() > 8000 => {
            return json!({"error": "details must be ≤ 8000 characters"});
        }
        Some(d) => Some(d.to_string()),
        None => None,
    };

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
    metadata.insert("source_paths".to_string(), json!(source_paths));
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
