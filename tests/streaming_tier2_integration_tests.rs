//! Integration tests for Streaming Response Protocol Tier 2 (RFC 0001 Tier 2).
//!
//! Verifies granular progress reporting across batch operations (memory_create_batch,
//! memory_delete_batch, context_seed).

use std::sync::mpsc;
use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::mcp::progress::{
    ChannelProgressReporter, ProgressNotification, ProgressReporter, ProgressToken,
};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn create_test_context(reporter: Option<Arc<dyn ProgressReporter>>) -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    HandlerContext {
        storage,
        embedder,
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        progress_reporter: reporter,
    }
}

#[test]
fn test_memory_create_batch_emits_step_progress() {
    let (tx, rx) = mpsc::channel::<ProgressNotification>();
    let token = ProgressToken::String("batch-create-test".to_string());
    let reporter = Arc::new(ChannelProgressReporter::from_sender(token, tx));

    let ctx = create_test_context(Some(reporter));

    let memories = vec![
        json!({"content": "Batch item 1", "memory_type": "context"}),
        json!({"content": "Batch item 2", "memory_type": "context"}),
        json!({"content": "Batch item 3", "memory_type": "context"}),
        json!({"content": "Batch item 4", "memory_type": "context"}),
    ];

    let result = dispatch(&ctx, "memory_create_batch", json!({ "memories": memories }));

    assert_eq!(
        result.get("total_created").and_then(|v| v.as_u64()),
        Some(4)
    );

    let notifications: Vec<ProgressNotification> = rx.try_iter().collect();
    assert_eq!(
        notifications.len(),
        6,
        "Expected 6 notifications: 1 start + 4 items + 1 complete"
    );

    // Notification 0: Start
    assert_eq!(notifications[0].params.progress, 0);
    assert_eq!(notifications[0].params.total, Some(4));
    assert!(notifications[0]
        .params
        .message
        .as_deref()
        .unwrap()
        .contains("Starting batch creation"));

    // Notifications 1-4: Incremental Steps
    for (idx, notification) in notifications[1..=4].iter().enumerate() {
        let step = (idx + 1) as u64;
        assert_eq!(notification.params.progress, step);
        assert_eq!(notification.params.total, Some(4));
        assert!(notification
            .params
            .message
            .as_deref()
            .unwrap()
            .contains(&format!("Created batch memory {step}/4")));
    }

    // Notification 5: Completion
    assert_eq!(notifications[5].params.progress, 4);
    assert_eq!(notifications[5].params.total, Some(4));
    assert!(notifications[5]
        .params
        .message
        .as_deref()
        .unwrap()
        .contains("Batch creation completed: 4 created, 0 failed"));
}

#[test]
fn test_memory_delete_batch_emits_step_progress() {
    let (tx, rx) = mpsc::channel::<ProgressNotification>();
    let token = ProgressToken::Integer(42);
    let reporter = Arc::new(ChannelProgressReporter::from_sender(token, tx));

    let ctx = create_test_context(Some(reporter));

    // Pre-create 3 memories
    let create_res = dispatch(
        &ctx,
        "memory_create_batch",
        json!({
            "memories": [
                {"content": "Delete target 1"},
                {"content": "Delete target 2"},
                {"content": "Delete target 3"}
            ]
        }),
    );
    let created_arr = create_res
        .get("created")
        .and_then(|v| v.as_array())
        .unwrap();
    let ids: Vec<i64> = created_arr
        .iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_i64()))
        .collect();
    assert_eq!(ids.len(), 3);

    // Drain previous notifications
    let _: Vec<_> = rx.try_iter().collect();

    // Now delete the batch
    let del_res = dispatch(
        &ctx,
        "memory_delete_batch",
        json!({ "ids": ids, "cascade_chain": false }),
    );

    assert_eq!(
        del_res.get("total_deleted").and_then(|v| v.as_u64()),
        Some(3)
    );

    let notifications: Vec<ProgressNotification> = rx.try_iter().collect();
    assert!(
        notifications.len() >= 2,
        "Expected at least start and completion notifications"
    );

    // Verify start
    assert_eq!(notifications[0].params.progress, 0);
    assert_eq!(notifications[0].params.total, Some(3));

    // Verify completion
    let last = notifications.last().unwrap();
    assert_eq!(last.params.progress, 3);
    assert_eq!(last.params.total, Some(3));
    assert!(last
        .params
        .message
        .as_deref()
        .unwrap()
        .contains("Successfully deleted 3 memories"));
}

#[test]
fn test_context_seed_emits_step_progress() {
    let (tx, rx) = mpsc::channel::<ProgressNotification>();
    let token = ProgressToken::String("seed-progress-token".to_string());
    let reporter = Arc::new(ChannelProgressReporter::from_sender(token, tx));

    let ctx = create_test_context(Some(reporter));

    let seed_payload = json!({
        "entity_context": "Project Omega",
        "facts": [
            {"content": "Fact 1: uses PostgreSQL", "confidence": 0.95},
            {"content": "Fact 2: requires Redis for cache", "confidence": 0.85},
            {"content": "Fact 3: runs on port 8080", "confidence": 0.75}
        ]
    });

    let res = dispatch(&ctx, "context_seed", seed_payload);
    assert_eq!(res.get("status").and_then(|v| v.as_str()), Some("success"));
    assert_eq!(res.get("seeded_count").and_then(|v| v.as_u64()), Some(3));

    let notifications: Vec<ProgressNotification> = rx.try_iter().collect();
    assert_eq!(
        notifications.len(),
        3,
        "Expected 3 notifications: start, step, complete"
    );

    assert_eq!(notifications[0].params.progress, 0);
    assert_eq!(notifications[0].params.total, Some(3));
    assert!(notifications[0]
        .params
        .message
        .as_deref()
        .unwrap()
        .contains("Starting context seed of 3 facts"));

    assert_eq!(notifications[1].params.progress, 3);
    assert_eq!(notifications[1].params.total, Some(3));

    assert_eq!(notifications[2].params.progress, 3);
    assert_eq!(notifications[2].params.total, Some(3));
    assert!(notifications[2]
        .params
        .message
        .as_deref()
        .unwrap()
        .contains("Context seed completed: 3 created, 0 failed"));
}

#[test]
fn test_noop_progress_reporter_handles_all_operations_cleanly() {
    // When progress_reporter is None, HandlerContext returns NoopProgressReporter.
    let ctx = create_test_context(None);

    let memories = vec![
        json!({"content": "Noop test 1"}),
        json!({"content": "Noop test 2"}),
    ];

    let create_res = dispatch(&ctx, "memory_create_batch", json!({ "memories": memories }));
    assert_eq!(
        create_res.get("total_created").and_then(|v| v.as_u64()),
        Some(2)
    );

    let delete_res = dispatch(
        &ctx,
        "memory_delete_batch",
        json!({ "ids": [1, 2], "cascade_chain": false }),
    );
    assert!(delete_res.get("total_deleted").is_some());
}
