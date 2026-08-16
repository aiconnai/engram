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

use crate::storage::queries::create_memory;
use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};

fn seed_memory(storage: &Storage, workspace: &str, content: &str, memory_type: MemoryType) -> i64 {
    storage
        .with_transaction(|conn| {
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: content.to_string(),
                    memory_type,
                    workspace: Some(workspace.to_string()),
                    tier: MemoryTier::Permanent,
                    ..Default::default()
                },
            )?;
            Ok::<_, crate::error::EngramError>(memory.id)
        })
        .expect("seed memory")
}

#[test]
fn explicit_fields_are_rendered_and_override_empty_inference() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("explicit-workspace".to_string()),
            current_goal: Some("Finish the shared handoff builder".to_string()),
            files_touched: vec!["src/intelligence/session_handoff/builder.rs".to_string()],
            decisions_made: vec!["Use one shared builder for MCP and CLI".to_string()],
            tests_run: vec!["rtk cargo test session_handoff --lib".to_string()],
            tests_not_run: vec!["full make ci not run in focused task".to_string()],
            known_risks: vec!["MCP schema still needs migration".to_string()],
            blockers: vec!["No blockers".to_string()],
            next_steps: vec!["Wire session_land to the builder".to_string()],
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert!(packet
        .copy_block
        .contains("Finish the shared handoff builder"));
    assert!(packet
        .copy_block
        .contains("Use one shared builder for MCP and CLI"));
    assert!(packet
        .copy_block
        .contains("rtk cargo test session_handoff --lib"));
    assert!(packet
        .copy_block
        .contains("full make ci not run in focused task"));
    assert!(packet
        .copy_block
        .contains("Wire session_land to the builder"));
    assert!(packet
        .copy_block
        .contains("## What changed\n- src/intelligence/session_handoff/builder.rs"));
}

#[test]
fn rendered_content_strips_private_tags_from_memory_previews() {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    seed_memory(
        &storage,
        "safe-workspace",
        "Public decision <private>SECRET_TOKEN</private> after text",
        MemoryType::Decision,
    );

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("safe-workspace".to_string()),
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert!(packet.copy_block.contains("Public decision"));
    assert!(!packet.copy_block.contains("SECRET_TOKEN"));
}

#[test]
fn memory_retrieval_populates_open_items_decisions_and_sorted_source_ids() {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let issue_id = seed_memory(
        &storage,
        "memory-workspace",
        "Investigate MCP schema update",
        MemoryType::Issue,
    );
    let decision_id = seed_memory(
        &storage,
        "memory-workspace",
        "Reuse the shared handoff builder",
        MemoryType::Decision,
    );
    let todo_id = seed_memory(
        &storage,
        "memory-workspace",
        "Wire CLI wrapper next",
        MemoryType::Todo,
    );

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("memory-workspace".to_string()),
            decisions_made: vec!["Explicit reviewer decision".to_string()],
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert!(packet.copy_block.contains("## Open items"));
    assert!(packet.copy_block.contains("Investigate MCP schema update"));
    assert!(packet.copy_block.contains("Wire CLI wrapper next"));
    assert!(packet.copy_block.contains("Explicit reviewer decision"));
    assert!(packet
        .copy_block
        .contains("Reuse the shared handoff builder"));
    assert_eq!(
        packet.source_memory_ids,
        vec![issue_id, decision_id, todo_id]
    );
}

#[test]
fn topic_digest_enriches_handoff_packet_with_goal_memories() {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let goal_doc_id = seed_memory(
        &storage,
        "digest-workspace",
        "Architecture overview for Context Rotation and Session Handoff",
        MemoryType::Note,
    );

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("digest-workspace".to_string()),
            current_goal: Some("Context Rotation".to_string()),
            include_digest: true,
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet with digest");

    assert!(packet.source_memory_ids.contains(&goal_doc_id));
    assert!(packet
        .open_items
        .iter()
        .any(|item| item.source_memory_id == Some(goal_doc_id)));
    assert!(packet
        .copy_block
        .contains("Architecture overview for Context Rotation"));
}
