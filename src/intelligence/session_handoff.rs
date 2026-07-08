use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::intelligence::session_indexing::list_sessions;
use crate::Storage;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SessionHandoffRequest {
    pub session_id: Option<String>,
    pub workspace: Option<String>,
    pub summary: Option<String>,
    pub current_goal: Option<String>,
    pub next_session_hints: Vec<String>,
    pub files_touched: Vec<String>,
    pub decisions_made: Vec<String>,
    pub tests_run: Vec<String>,
    pub tests_not_run: Vec<String>,
    pub known_risks: Vec<String>,
    pub blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub verification_evidence: Option<String>,
    pub issue_numbers: Vec<i64>,
    pub plan_doc_paths: Vec<String>,
    pub persist: bool,
    pub include_operational_context: bool,
    pub include_digest: bool,
}

impl Default for SessionHandoffRequest {
    fn default() -> Self {
        Self {
            session_id: None,
            workspace: None,
            summary: None,
            current_goal: None,
            next_session_hints: Vec::new(),
            files_touched: Vec::new(),
            decisions_made: Vec::new(),
            tests_run: Vec::new(),
            tests_not_run: Vec::new(),
            known_risks: Vec::new(),
            blockers: Vec::new(),
            next_steps: Vec::new(),
            verification_evidence: None,
            issue_numbers: Vec::new(),
            plan_doc_paths: Vec::new(),
            persist: true,
            include_operational_context: true,
            include_digest: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HandoffItem {
    pub title: String,
    pub detail: Option<String>,
    pub source_memory_id: Option<i64>,
    pub source_context_event_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionHandoffPacket {
    pub session_id: Option<String>,
    pub workspace: String,
    pub created_at: String,
    pub summary: String,
    pub current_goal: Option<String>,
    pub open_items: Vec<HandoffItem>,
    pub decisions: Vec<HandoffItem>,
    pub verification: Vec<HandoffItem>,
    pub risks: Vec<HandoffItem>,
    pub blockers: Vec<HandoffItem>,
    pub files_touched: Vec<String>,
    pub tests_run: Vec<String>,
    pub tests_not_run: Vec<String>,
    pub next_steps: Vec<String>,
    pub source_memory_ids: Vec<i64>,
    pub source_context_event_ids: Vec<i64>,
    pub warnings: Vec<String>,
    pub checkpoint_id: Option<i64>,
    pub copy_block: String,
}

pub fn build_session_handoff(
    storage: &Storage,
    request: SessionHandoffRequest,
) -> Result<SessionHandoffPacket> {
    let workspace = workspace_name(request.workspace.as_deref());
    let (session_id, warnings) =
        resolve_session_id(storage, request.session_id.as_deref(), &workspace)?;
    let summary = handoff_summary(
        request.summary.as_deref(),
        session_id.as_deref(),
        &workspace,
    );

    let mut packet = SessionHandoffPacket {
        session_id,
        workspace,
        created_at: chrono::Utc::now().to_rfc3339(),
        summary,
        current_goal: request.current_goal.clone(),
        open_items: Vec::new(),
        decisions: item_list(&request.decisions_made, None),
        verification: verification_items(&request),
        risks: item_list(&request.known_risks, None),
        blockers: item_list(&request.blockers, None),
        files_touched: request.files_touched.clone(),
        tests_run: request.tests_run.clone(),
        tests_not_run: request.tests_not_run.clone(),
        next_steps: merged_next_steps(&request.next_steps, &request.next_session_hints),
        source_memory_ids: Vec::new(),
        source_context_event_ids: Vec::new(),
        warnings,
        checkpoint_id: None,
        copy_block: String::new(),
    };

    add_completeness_warnings(&mut packet);
    packet.copy_block = render_copy_block(&packet);
    Ok(packet)
}

fn workspace_name(requested: Option<&str>) -> String {
    requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string()
}

fn resolve_session_id(
    storage: &Storage,
    requested: Option<&str>,
    workspace: &str,
) -> Result<(Option<String>, Vec<String>)> {
    if let Some(session_id) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok((Some(session_id.to_string()), Vec::new()));
    }

    let sessions = storage.with_connection(|conn| list_sessions(conn, Some(workspace), 1))?;
    if let Some(session) = sessions.into_iter().next() {
        let warning = format!(
            "session_id omitted; using most recent session '{}' in workspace '{}'.",
            session.session_id, workspace
        );
        Ok((Some(session.session_id), vec![warning]))
    } else {
        let warning = format!(
            "No concrete session resolved for workspace '{workspace}'; generated a workspace-level handoff."
        );
        Ok((None, vec![warning]))
    }
}

fn handoff_summary(requested: Option<&str>, session_id: Option<&str>, workspace: &str) -> String {
    if let Some(summary) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        return summary.to_string();
    }

    match session_id {
        Some(id) => format!("Session {id} handoff"),
        None => format!("Workspace {workspace} handoff"),
    }
}

fn item_list(values: &[String], detail: Option<&str>) -> Vec<HandoffItem> {
    values
        .iter()
        .map(|value| handoff_item(strip_private_content(value), detail))
        .collect()
}

fn handoff_item(title: String, detail: Option<&str>) -> HandoffItem {
    HandoffItem {
        title,
        detail: detail.map(str::to_string),
        source_memory_id: None,
        source_context_event_id: None,
    }
}

fn verification_items(request: &SessionHandoffRequest) -> Vec<HandoffItem> {
    let mut items = item_list(&request.tests_run, Some("test_run"));
    if let Some(evidence) = request
        .verification_evidence
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        items.push(handoff_item(
            strip_private_content(evidence),
            Some("verification_evidence"),
        ));
    }
    items
}

fn merged_next_steps(next_steps: &[String], hints: &[String]) -> Vec<String> {
    let mut merged = next_steps.to_vec();
    for hint in hints {
        if !hint.trim().is_empty() && !merged.iter().any(|step| step == hint) {
            merged.push(hint.clone());
        }
    }
    merged
}

fn add_completeness_warnings(packet: &mut SessionHandoffPacket) {
    if packet
        .current_goal
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        packet
            .warnings
            .push("No current_goal was provided or inferred.".to_string());
    }
    if packet.next_steps.is_empty() {
        packet
            .warnings
            .push("No next steps were provided or inferred.".to_string());
    }
    if packet.verification.is_empty() {
        packet.warnings.push(
            "No verification evidence provided. Do not claim this work is complete.".to_string(),
        );
    }
}

fn strip_private_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(start) = remaining.find("<private>") {
        result.push_str(&remaining[..start]);
        if let Some(end_offset) = remaining[start..].find("</private>") {
            remaining = &remaining[start + end_offset + "</private>".len()..];
        } else {
            return result;
        }
    }
    result.push_str(remaining);
    result
}

fn render_copy_block(packet: &SessionHandoffPacket) -> String {
    let mut output = String::new();
    output.push_str("# Continue this work in a new AI session\n\n");
    output.push_str("## Essential context\n");
    output.push_str(&format!("Workspace: {}\n", packet.workspace));
    if let Some(session_id) = &packet.session_id {
        output.push_str(&format!("Session: {session_id}\n"));
    }
    output.push_str(&format!("Summary: {}\n\n", packet.summary));

    output.push_str("## Current goal\n");
    output.push_str(
        packet
            .current_goal
            .as_deref()
            .unwrap_or("No current goal captured."),
    );
    output.push_str("\n\n## Decisions\n");
    push_items(&mut output, &packet.decisions);
    output.push_str("\n## Verification\n");
    push_items(&mut output, &packet.verification);
    push_string_list(&mut output, "Tests not run", &packet.tests_not_run);

    output.push_str("\n## Risks and blockers\n");
    push_items(&mut output, &packet.risks);
    push_items(&mut output, &packet.blockers);

    output.push_str("\n## Next steps\n");
    if packet.next_steps.is_empty() {
        output.push_str("- No next steps captured.\n");
    } else {
        for (index, step) in packet.next_steps.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", index + 1, strip_private_content(step)));
        }
    }

    output.push_str("\n## Source references\n");
    output.push_str(&format!("- Memory IDs: {:?}\n", packet.source_memory_ids));
    output.push_str(&format!(
        "- Context event IDs: {:?}\n",
        packet.source_context_event_ids
    ));
    output.push_str(&format!("- Files: {:?}\n", packet.files_touched));
    push_string_list(&mut output, "Warnings", &packet.warnings);
    output
}

fn push_items(output: &mut String, items: &[HandoffItem]) {
    if items.is_empty() {
        output.push_str("- None captured.\n");
        return;
    }

    for item in items {
        output.push_str(&format!("- {}\n", item.title));
    }
}

fn push_string_list(output: &mut String, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }

    output.push('\n');
    output.push_str(title);
    output.push_str(":\n");
    for item in items {
        output.push_str(&format!("- {}\n", strip_private_content(item)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::session_indexing::{index_conversation, ChunkingConfig, Message};
    use crate::Storage;
    use chrono::{Duration, Utc};

    fn message_at(minutes_ago: i64, content: &str) -> Message {
        Message {
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Utc::now() - Duration::minutes(minutes_ago),
            id: None,
        }
    }

    fn seed_session(storage: &Storage, session_id: &str, workspace: &str, minutes_ago: i64) {
        let messages = vec![message_at(minutes_ago, &format!("seed {session_id}"))];
        storage
            .with_connection(|conn| {
                index_conversation(
                    conn,
                    session_id,
                    &messages,
                    &ChunkingConfig::default(),
                    Some(workspace),
                    Some(session_id),
                    Some("test-agent"),
                )?;
                Ok::<_, crate::error::EngramError>(())
            })
            .expect("seed session");
    }

    #[test]
    fn omitted_session_id_uses_latest_session_in_workspace() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        seed_session(&storage, "older-session", "handoff-test", 20);
        seed_session(&storage, "newer-session", "handoff-test", 1);

        let packet = build_session_handoff(
            &storage,
            SessionHandoffRequest {
                workspace: Some("handoff-test".to_string()),
                persist: false,
                ..SessionHandoffRequest::default()
            },
        )
        .expect("handoff packet");

        assert_eq!(packet.session_id.as_deref(), Some("newer-session"));
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("session_id omitted")),
            "fallback warning missing: {:?}",
            packet.warnings
        );
        assert!(packet
            .copy_block
            .contains("# Continue this work in a new AI session"));
    }

    #[test]
    fn omitted_session_id_without_sessions_returns_workspace_packet_with_warning() {
        let storage = Storage::open_in_memory().expect("in-memory storage");

        let packet = build_session_handoff(
            &storage,
            SessionHandoffRequest {
                workspace: Some("empty-workspace".to_string()),
                persist: false,
                ..SessionHandoffRequest::default()
            },
        )
        .expect("workspace handoff packet");

        assert_eq!(packet.session_id, None);
        assert_eq!(packet.workspace, "empty-workspace");
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("No concrete session resolved")),
            "workspace warning missing: {:?}",
            packet.warnings
        );
        assert!(packet.copy_block.contains("## Source references"));
    }
}
