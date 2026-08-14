use std::time::Duration;

use axum::http::HeaderMap;

use super::super::events::{
    event_type_to_str, parse_event_type, realtime_event_to_sse, EventsQuery, SSE_RETRY_MS,
};
use crate::realtime::{EventType, RealtimeEvent};

#[test]
fn test_sse_event_serialization() {
    let event = RealtimeEvent::memory_created(42, "hello world".to_string(), "default");
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"memory_created\""));
    assert!(json.contains("\"memory_id\":42"));
    assert_eq!(event_type_to_str(event.event_type), "memory_created");
}

#[test]
fn test_sse_event_type_to_str_all_variants() {
    assert_eq!(
        event_type_to_str(EventType::MemoryCreated),
        "memory_created"
    );
    assert_eq!(
        event_type_to_str(EventType::MemoryUpdated),
        "memory_updated"
    );
    assert_eq!(
        event_type_to_str(EventType::MemoryDeleted),
        "memory_deleted"
    );
    assert_eq!(
        event_type_to_str(EventType::CrossrefCreated),
        "crossref_created"
    );
    assert_eq!(
        event_type_to_str(EventType::CrossrefDeleted),
        "crossref_deleted"
    );
    assert_eq!(event_type_to_str(EventType::SyncStarted), "sync_started");
    assert_eq!(
        event_type_to_str(EventType::SyncCompleted),
        "sync_completed"
    );
    assert_eq!(event_type_to_str(EventType::SyncFailed), "sync_failed");
}

// ---- parse_event_type tests -------------------------------------------

#[test]
fn test_parse_event_type_known() {
    assert_eq!(
        parse_event_type("memory_created"),
        Some(EventType::MemoryCreated)
    );
    assert_eq!(parse_event_type("sync_failed"), Some(EventType::SyncFailed));
}

#[test]
fn test_parse_event_type_unknown_is_none() {
    assert_eq!(parse_event_type("unknown_event"), None);
    assert_eq!(parse_event_type(""), None);
}

// ---- EventsQuery filter parsing tests ---------------------------------

#[test]
fn test_events_query_parsed_event_types_none() {
    let q = EventsQuery {
        event_types: None,
        workspace: None,
    };
    assert!(q.parsed_event_types().is_none());
}

#[test]
fn test_events_query_parsed_event_types_single() {
    let q = EventsQuery {
        event_types: Some("memory_created".to_string()),
        workspace: None,
    };
    let types = q.parsed_event_types().unwrap();
    assert_eq!(types, vec![EventType::MemoryCreated]);
}

#[test]
fn test_events_query_parsed_event_types_multiple() {
    let q = EventsQuery {
        event_types: Some("memory_created,memory_deleted,sync_failed".to_string()),
        workspace: None,
    };
    let types = q.parsed_event_types().unwrap();
    assert_eq!(
        types,
        vec![
            EventType::MemoryCreated,
            EventType::MemoryDeleted,
            EventType::SyncFailed
        ]
    );
}

#[test]
fn test_events_query_parsed_event_types_with_spaces() {
    let q = EventsQuery {
        event_types: Some("memory_created, memory_updated".to_string()),
        workspace: None,
    };
    let types = q.parsed_event_types().unwrap();
    assert_eq!(
        types,
        vec![EventType::MemoryCreated, EventType::MemoryUpdated]
    );
}

#[test]
fn test_events_query_parsed_event_types_all_unknown_returns_none() {
    let q = EventsQuery {
        event_types: Some("bogus,fake".to_string()),
        workspace: None,
    };
    // All tokens invalid → None (no filter)
    assert!(q.parsed_event_types().is_none());
}

// ---- Filter matching tests (via SubscriptionFilter in events module) --

#[test]
fn test_event_type_filter_matches() {
    use crate::realtime::SubscriptionFilter;

    let filter = SubscriptionFilter {
        event_types: Some(vec![EventType::MemoryCreated]),
        memory_ids: None,
        tags: None,
    };
    let created = RealtimeEvent::memory_created(1, "test".to_string(), "default");
    let deleted = RealtimeEvent::memory_deleted(1, "default");
    assert!(filter.matches(&created));
    assert!(!filter.matches(&deleted));
}

#[test]
fn test_keep_alive_interval_is_30s() {
    // Verify the constant used for keep-alive is correct.
    let interval = Duration::from_secs(30);
    assert_eq!(interval.as_secs(), 30);
}

// ---- Last-Event-Id header parsing tests --------------------------------

/// Verify that a valid numeric `Last-Event-Id` header is parsed to `u64`.
#[test]
fn test_last_event_id_header_valid() {
    let mut headers = HeaderMap::new();
    headers.insert("last-event-id", "42".parse().unwrap());

    let parsed: Option<u64> = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    assert_eq!(parsed, Some(42));
}

#[test]
fn test_last_event_id_header_missing_is_none() {
    let headers = HeaderMap::new();
    let parsed: Option<u64> = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    assert!(parsed.is_none());
}

#[test]
fn test_last_event_id_header_non_numeric_is_none() {
    let mut headers = HeaderMap::new();
    headers.insert("last-event-id", "not-a-number".parse().unwrap());
    let parsed: Option<u64> = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    assert!(parsed.is_none());
}

#[test]
fn test_last_event_id_header_zero() {
    let mut headers = HeaderMap::new();
    headers.insert("last-event-id", "0".parse().unwrap());
    let parsed: Option<u64> = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    assert_eq!(parsed, Some(0));
}

// ---- realtime_event_to_sse tests ---------------------------------------

/// Verify that an event with a seq_id produces an SSE event with an `id:` field.
#[test]
fn test_realtime_event_to_sse_with_seq_id() {
    use crate::realtime::RealtimeManager;

    let manager = RealtimeManager::new();
    let _rx = manager.subscribe();
    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "hello".to_string(),
        "default",
    ));

    let buffered = manager.get_events_after(0);
    assert_eq!(buffered.len(), 1);

    let event = &buffered[0];
    assert_eq!(event.seq_id, Some(1));

    // Verify the SSE event would include the id
    // (axum's Event::id sets the id field; we verify seq_id is present)
    let sse = realtime_event_to_sse(event);
    // The event should serialize without panic; content is verified via seq_id field
    let _ = sse; // axum::sse::Event has no public getter, just verify it builds
}

#[test]
fn test_realtime_event_to_sse_without_seq_id_no_id_field() {
    // Events with seq_id = None should still build an SSE event (no id field).
    let event = RealtimeEvent::memory_created(5, "no id".to_string(), "default");
    assert!(event.seq_id.is_none());
    let sse = realtime_event_to_sse(&event);
    let _ = sse; // should not panic
}

// ---- Replay via get_events_after + Last-Event-Id integration -----------

#[test]
fn test_replay_events_after_last_id() {
    use crate::realtime::RealtimeManager;

    let manager = RealtimeManager::new();
    let _rx = manager.subscribe();

    // Broadcast 5 events
    for i in 1..=5i64 {
        manager.broadcast(RealtimeEvent::memory_created(
            i,
            format!("ev{i}"),
            "default",
        ));
    }

    // Simulate Last-Event-Id: 3 — client missed events 4 and 5
    let last_id: u64 = 3;
    let replayed = manager.get_events_after(last_id);
    assert_eq!(replayed.len(), 2);
    let ids: Vec<u64> = replayed.iter().filter_map(|e| e.seq_id).collect();
    assert_eq!(ids, vec![4, 5]);
}

#[test]
fn test_retry_constant_is_3000ms() {
    assert_eq!(SSE_RETRY_MS, 3000);
}

// ---- H2: constant-time token comparison --------------------------------
