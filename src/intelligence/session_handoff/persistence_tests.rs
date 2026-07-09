use super::*;
use crate::storage::queries::list_memories;
use crate::storage::{create_context_event, NewContextEvent};
use crate::types::{ListOptions, MemoryType};
use crate::Storage;
use chrono::Utc;
use serde_json::json;

#[test]
fn persist_true_creates_checkpoint_memory() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("persist-workspace".to_string()),
            current_goal: Some("Persist packet".to_string()),
            next_steps: vec!["Inspect checkpoint".to_string()],
            persist: true,
            include_operational_context: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    let checkpoint_id = packet.checkpoint_id.expect("checkpoint id");
    let memories = storage
        .with_connection(|conn| {
            list_memories(
                conn,
                &ListOptions {
                    workspace: Some("persist-workspace".to_string()),
                    memory_type: Some(MemoryType::Checkpoint),
                    ..Default::default()
                },
            )
        })
        .expect("list checkpoints");

    assert!(memories.iter().any(|memory| memory.id == checkpoint_id));
    assert!(packet.copy_block.contains("Persist packet"));
}

#[test]
fn persist_false_does_not_create_checkpoint_memory() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("no-persist-workspace".to_string()),
            persist: false,
            include_operational_context: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert_eq!(packet.checkpoint_id, None);
    let memories = storage
        .with_connection(|conn| {
            list_memories(
                conn,
                &ListOptions {
                    workspace: Some("no-persist-workspace".to_string()),
                    memory_type: Some(MemoryType::Checkpoint),
                    ..Default::default()
                },
            )
        })
        .expect("list checkpoints");
    assert!(memories.is_empty());
}

#[test]
fn persist_failure_adds_warning_without_losing_packet() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("invalid workspace".to_string()),
            current_goal: Some("Keep packet despite persistence failure".to_string()),
            next_steps: vec!["Read warning".to_string()],
            persist: true,
            include_operational_context: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet survives persistence failure");

    assert_eq!(packet.checkpoint_id, None);
    assert!(packet
        .warnings
        .iter()
        .any(|warning| warning.contains("Checkpoint persistence failed")));
    assert!(packet
        .copy_block
        .contains("Keep packet despite persistence failure"));
}

#[test]
fn operational_context_bundle_adds_source_context_event_ids() {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let metadata = json!({"decision": "Use Operational Context bundle"});
    let event_id = storage
        .with_transaction(|conn| {
            create_context_event(
                conn,
                &NewContextEvent {
                    repo_id: None,
                    workspace_path_hash: Some("oc-workspace"),
                    git_branch: None,
                    worktree_name: None,
                    commit_hash: None,
                    session_id: "oc-session",
                    task_id: None,
                    agent_id: Some("codex"),
                    source: "codex",
                    event_type: "decision",
                    command_name: None,
                    tool_name: None,
                    cwd: None,
                    exit_code: None,
                    started_at: Utc::now(),
                    finished_at: None,
                    redaction_status: "redacted",
                    retention_policy: "default",
                    raw_artifact_id: None,
                    raw_payload: None,
                    metadata: &metadata,
                },
            )
        })
        .expect("seed context event");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("oc-workspace".to_string()),
            session_id: Some("oc-session".to_string()),
            current_goal: Some("Use Operational Context bundle".to_string()),
            persist: false,
            include_operational_context: true,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert_eq!(packet.source_context_event_ids, vec![event_id]);
    assert!(packet.copy_block.contains(&format!("{event_id}")));
}
