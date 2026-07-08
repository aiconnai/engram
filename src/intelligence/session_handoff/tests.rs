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
