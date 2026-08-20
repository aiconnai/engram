//! `harness_handoff` — structured handoff packet for next-agent continuity.

use serde_json::{json, Value};

use super::super::HandlerContext;

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
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        None => return json!({"error": "next_steps is required"}),
    };
    if next_steps.is_empty() {
        return json!({"error": "next_steps must have at least one item"});
    }

    // ── Extract optional params ──────────────────────────────────────────────
    let files_touched: Vec<String> = params
        .get("files_touched")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let decisions_made: Vec<String> = params
        .get("decisions_made")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let tests_run: Vec<String> = params
        .get("tests_run")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let tests_not_run: Vec<String> = params
        .get("tests_not_run")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let known_risks: Vec<String> = params
        .get("known_risks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let blockers: Vec<String> = params
        .get("blockers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let issue_numbers: Vec<i64> = params
        .get("issue_numbers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    let plan_doc_paths: Vec<String> = params
        .get("plan_doc_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
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

    let mut packet = match crate::intelligence::build_session_handoff(
        &ctx.storage,
        crate::intelligence::SessionHandoffRequest {
            workspace: Some(workspace.clone()),
            current_goal: Some(current_goal.clone()),
            files_touched: files_touched.clone(),
            decisions_made: decisions_made.clone(),
            tests_run: tests_run.clone(),
            tests_not_run: tests_not_run.clone(),
            known_risks: known_risks.clone(),
            blockers: blockers.clone(),
            next_steps: next_steps.clone(),
            verification_evidence: verification_evidence.clone(),
            issue_numbers: issue_numbers.clone(),
            plan_doc_paths: plan_doc_paths.clone(),
            persist,
            ..Default::default()
        },
    ) {
        Ok(packet) => packet,
        Err(e) => return json!({"error": format!("Failed to build handoff: {e}")}),
    };

    if persist {
        if let Some(checkpoint_id) = packet.checkpoint_id {
            if let Err(err) =
                mark_harness_handoff_checkpoint(ctx, checkpoint_id, &params, &current_goal)
            {
                packet
                    .warnings
                    .push(format!("Harness status indexing failed: {err}"));
            }
        }
    }

    let has_evidence = verification_evidence
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    // ── Build response ───────────────────────────────────────────────────────
    let mut response = json!({
        "handoff_id": packet.checkpoint_id,
        "workspace": packet.workspace,
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
        "persisted": persist && packet.checkpoint_id.is_some(),
        "created_at": packet.created_at,
        "warnings": packet.warnings,
        "copy_block": packet.copy_block,
    });

    if !has_evidence {
        response["completion_warning"] =
            json!("No verification evidence provided. Do not claim this work is complete.");
    }

    response
}

fn mark_harness_handoff_checkpoint(
    ctx: &HandlerContext,
    checkpoint_id: i64,
    params: &Value,
    current_goal: &str,
) -> crate::error::Result<()> {
    ctx.storage.with_transaction(|conn| {
        let memory = crate::storage::queries::get_memory(conn, checkpoint_id)?;
        let mut tags = memory.tags.clone();
        for tag in ["harness", "handoff"] {
            if !tags.iter().any(|existing| existing == tag) {
                tags.push(tag.to_string());
            }
        }

        let mut metadata = memory.metadata.clone();
        metadata.insert("harness_kind".to_string(), json!("handoff"));
        metadata.insert(
            "current_goal".to_string(),
            serde_json::Value::String(current_goal.to_string()),
        );
        metadata.insert("source".to_string(), json!("session_handoff_builder"));
        for key in [
            "files_touched",
            "decisions_made",
            "tests_run",
            "tests_not_run",
            "known_risks",
            "blockers",
            "next_steps",
            "issue_numbers",
            "plan_doc_paths",
            "verification_evidence",
        ] {
            if let Some(value) = params.get(key) {
                metadata.insert(key.to_string(), value.clone());
            }
        }

        crate::storage::queries::update_memory(
            conn,
            checkpoint_id,
            &crate::types::UpdateMemoryInput {
                content: None,
                memory_type: None,
                tags: Some(tags),
                metadata: Some(metadata),
                importance: None,
                scope: None,
                ttl_seconds: None,
                event_time: None,
                trigger_pattern: None,
                media_url: None,
            },
        )?;
        Ok(())
    })
}
