use super::privacy::strip_private_content;
use super::render::render_copy_block;
use super::retrieval::{collect_open_items, collect_recent_decisions, push_source_ids};
use super::types::{HandoffItem, SessionHandoffPacket, SessionHandoffRequest};
use crate::error::Result;
use crate::intelligence::session_indexing::list_sessions;
use crate::Storage;

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

    match collect_open_items(storage, &packet.workspace) {
        Ok(open_items) => packet.open_items = open_items,
        Err(err) => packet
            .warnings
            .push(format!("Open item retrieval failed: {err}")),
    }

    match collect_recent_decisions(storage, &packet.workspace) {
        Ok(mut inferred_decisions) => {
            if packet.decisions.is_empty() {
                packet.decisions = inferred_decisions;
            } else {
                packet.decisions.append(&mut inferred_decisions);
            }
        }
        Err(err) => packet
            .warnings
            .push(format!("Decision retrieval failed: {err}")),
    }

    push_source_ids(&mut packet);
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
