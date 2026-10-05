//! Failure, replay and bound tests for the lifecycle hooks (C6).
//!
//! Delivery semantics under test (see `docs/OPERATIONS.md`-style summary in
//! each handler): the SessionEnd -> SessionStart queue is at-most-once, a
//! replayed SessionEnd is de-duplicated while its row is still queued, and
//! every oversize input is truncated or capped with an explicit marker rather
//! than silently accepted.

use std::collections::HashMap;

use serde_json::json;

use super::post_tool_use::{PostToolUseHandler, MAX_REINFORCED_MEMORIES_PER_EVENT};
use super::session_end::{SessionEndHandler, SessionEndPayload, MAX_NOTES_BYTES};
use super::session_start::SessionStartHandler;
use super::{HookContext, HookManager, HookResult, LifecycleHook};
use crate::storage::pending_injections::{self, MAX_DRAIN_ROWS};
use crate::storage::queries::{create_memory, get_policy_record};
use crate::storage::Storage;
use crate::types::{CreateMemoryInput, MemoryType};

fn ctx(session: &str, workspace: &str) -> HookContext {
    HookContext {
        session_id: Some(session.to_string()),
        workspace: Some(workspace.to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: HashMap::new(),
    }
}

fn memory_input(content: &str, workspace: &str) -> CreateMemoryInput {
    CreateMemoryInput {
        content: content.to_string(),
        memory_type: MemoryType::Note,
        tags: vec![],
        metadata: HashMap::new(),
        importance: None,
        scope: Default::default(),
        workspace: Some(workspace.to_string()),
        tier: Default::default(),
        defer_embedding: true,
        ttl_seconds: None,
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    }
}

fn queued_payloads(storage: &Storage, workspace: &str) -> Vec<String> {
    storage
        .with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT payload FROM pending_injections WHERE workspace = ? ORDER BY id",
            )?;
            let rows = stmt
                .query_map([workspace], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .unwrap()
}

#[test]
fn oversize_notes_are_truncated_with_explicit_marker() {
    let storage = Storage::open_in_memory().unwrap();
    let handler = SessionEndHandler::new(storage.clone());
    let mut c = ctx("s-big", "ws");
    // Multi-byte characters: a naive byte slice would split one.
    let notes = "é".repeat(MAX_NOTES_BYTES);
    c.metadata.insert("notes".to_string(), json!(notes));

    handler.handle(LifecycleHook::SessionEnd, &c).unwrap();

    let payloads = queued_payloads(&storage, "ws");
    assert_eq!(payloads.len(), 1, "oversize notes must still enqueue");
    let parsed: SessionEndPayload = serde_json::from_str(&payloads[0]).unwrap();
    let kept = parsed.notes.expect("truncated notes kept");
    assert!(
        kept.len() <= MAX_NOTES_BYTES,
        "notes not capped: {}",
        kept.len()
    );
    assert!(kept.chars().all(|ch| ch == 'é'), "truncation split a char");
    assert!(parsed.notes_truncated, "partial result must be flagged");
}

#[test]
fn notes_within_limit_are_untouched_and_unflagged() {
    let storage = Storage::open_in_memory().unwrap();
    let handler = SessionEndHandler::new(storage.clone());
    let mut c = ctx("s-ok", "ws");
    c.metadata
        .insert("notes".to_string(), json!("short handoff"));
    handler.handle(LifecycleHook::SessionEnd, &c).unwrap();
    let parsed: SessionEndPayload =
        serde_json::from_str(&queued_payloads(&storage, "ws")[0]).unwrap();
    assert_eq!(parsed.notes.as_deref(), Some("short handoff"));
    assert!(!parsed.notes_truncated);
}

#[test]
fn replayed_session_end_does_not_duplicate_the_injection() {
    let storage = Storage::open_in_memory().unwrap();
    let handler = SessionEndHandler::new(storage.clone());
    let c = ctx("sess-replay", "ws");

    handler.handle(LifecycleHook::SessionEnd, &c).unwrap();
    handler.handle(LifecycleHook::SessionEnd, &c).unwrap();
    handler.handle(LifecycleHook::SessionEnd, &c).unwrap();

    let n = storage
        .with_connection(|conn| pending_injections::pending_count(conn, "ws"))
        .unwrap();
    assert_eq!(n, 1, "retried SessionEnd must be idempotent while queued");
}

#[test]
fn session_start_reports_the_backlog_it_left_queued() {
    let storage = Storage::open_in_memory().unwrap();
    let total = MAX_DRAIN_ROWS + 7;
    storage
        .with_connection(|conn| {
            for i in 0..total {
                pending_injections::enqueue(conn, "ws", &format!("p{i}"), None, None)?;
            }
            Ok(())
        })
        .unwrap();

    let result = SessionStartHandler::new(storage.clone())
        .handle(LifecycleHook::SessionStart, &ctx("next", "ws"))
        .unwrap();
    let HookResult::Modify(value) = result else {
        panic!("expected Modify");
    };
    assert_eq!(value["count"], MAX_DRAIN_ROWS);
    assert_eq!(value["remaining"], 7, "partial delivery must be reported");

    // The rest is delivered by the next session, nothing is lost or repeated.
    let HookResult::Modify(second) = SessionStartHandler::new(storage.clone())
        .handle(LifecycleHook::SessionStart, &ctx("next-2", "ws"))
        .unwrap()
    else {
        panic!("expected Modify");
    };
    assert_eq!(second["count"], 7);
    assert_eq!(second["remaining"], 0);
}

#[test]
fn a_panicking_handler_does_not_take_down_the_dispatch() {
    let mut manager = HookManager::new();
    manager.register(LifecycleHook::PostToolUse, |_, _| {
        panic!("handler bug");
    });
    manager.register(LifecycleHook::PostToolUse, |_, _| Ok(HookResult::Continue));

    let results = manager
        .trigger(LifecycleHook::PostToolUse, &ctx("s", "ws"))
        .expect("dispatch must survive a handler panic");
    assert_eq!(
        results.len(),
        1,
        "later handlers still run; the panicking one yields no result"
    );
}

#[test]
fn post_tool_use_reinforcement_is_bounded_per_event() {
    let storage = Storage::open_in_memory().unwrap();
    let total = MAX_REINFORCED_MEMORIES_PER_EVENT + 10;
    let ids: Vec<i64> = storage
        .with_connection(|conn| {
            (0..total)
                .map(|i| {
                    create_memory(conn, &memory_input(&format!("bounded {i}"), "default"))
                        .map(|m| m.id)
                })
                .collect()
        })
        .unwrap();

    let handler = PostToolUseHandler::new(storage.clone());
    let mut c = ctx("s", "default");
    c.metadata
        .insert("tool_name".to_string(), json!("memory_search"));
    c.metadata
        .insert("returned_memory_ids".to_string(), json!(ids));
    handler.handle(LifecycleHook::PostToolUse, &c).unwrap();

    let reinforced = storage
        .with_connection(|conn| {
            let mut n = 0usize;
            for id in &ids {
                if get_policy_record(conn, *id)?.is_some_and(|p| p.reinforcement_count > 0) {
                    n += 1;
                }
            }
            Ok(n)
        })
        .unwrap();
    assert_eq!(reinforced, MAX_REINFORCED_MEMORIES_PER_EVENT);
}

#[test]
fn post_tool_use_does_not_reinforce_memories_of_another_workspace() {
    let storage = Storage::open_in_memory().unwrap();
    let (mine, theirs) = storage
        .with_connection(|conn| {
            let mine = create_memory(conn, &memory_input("mine", "ws-a"))?.id;
            let theirs = create_memory(conn, &memory_input("theirs", "ws-b"))?.id;
            Ok((mine, theirs))
        })
        .unwrap();

    let handler = PostToolUseHandler::new(storage.clone());
    let mut c = ctx("s", "ws-a");
    c.metadata
        .insert("tool_name".to_string(), json!("memory_search"));
    c.metadata
        .insert("returned_memory_ids".to_string(), json!([mine, theirs]));
    handler.handle(LifecycleHook::PostToolUse, &c).unwrap();

    let count = |id: i64| {
        storage
            .with_connection(|conn| get_policy_record(conn, id))
            .unwrap()
            .map(|p| p.reinforcement_count)
            .unwrap_or(0)
    };
    assert_eq!(count(mine), 1);
    assert_eq!(count(theirs), 0, "cross-workspace id must be ignored");
}

#[test]
fn workspace_filter_is_applied_before_the_per_event_cap() {
    let storage = Storage::open_in_memory().unwrap();
    // More foreign memories (smaller ids) than the cap, then one of ours.
    let mine = storage
        .with_connection(|conn| {
            for i in 0..(MAX_REINFORCED_MEMORIES_PER_EVENT + 5) {
                create_memory(conn, &memory_input(&format!("foreign {i}"), "ws-b"))?;
            }
            Ok(create_memory(conn, &memory_input("mine", "ws-a"))?.id)
        })
        .unwrap();
    let all_ids: Vec<i64> = (1..=mine).collect();

    let handler = PostToolUseHandler::new(storage.clone());
    let mut c = ctx("s", "ws-a");
    c.metadata
        .insert("tool_name".to_string(), json!("memory_search"));
    c.metadata
        .insert("returned_memory_ids".to_string(), json!(all_ids));
    handler.handle(LifecycleHook::PostToolUse, &c).unwrap();

    let count = storage
        .with_connection(|conn| get_policy_record(conn, mine))
        .unwrap()
        .map(|p| p.reinforcement_count)
        .unwrap_or(0);
    assert_eq!(count, 1, "foreign ids consumed the cap before filtering");
}

#[test]
fn unknown_backlog_size_is_reported_as_null_not_zero() {
    use super::session_start::remaining_value;
    assert_eq!(remaining_value(Ok(7)), json!(7));
    let err = crate::error::EngramError::Storage("boom".to_string());
    assert_eq!(remaining_value(Err(err)), serde_json::Value::Null);
}
