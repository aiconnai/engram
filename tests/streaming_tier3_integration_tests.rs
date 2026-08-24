//! Streaming Response Protocol Tier 3 Integration Tests.
//!
//! Verifies:
//! - Multi-stage progress updates for `memory_smart_retrieve` and `memory_digest`.
//! - `CallbackProgressReporter` for transport and bridge integration.
//! - `RealtimeEvent::progress` construction and SSE serialization.
//! - Event type conversions for `progress`.

use std::sync::mpsc;
use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, HandlerContext};
use engram::mcp::progress::{
    CallbackProgressReporter, ChannelProgressReporter, ProgressNotification, ProgressReporter,
    ProgressToken,
};
use engram::realtime::{EventType, RealtimeEvent};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn create_test_context(reporter: Option<Arc<dyn ProgressReporter>>) -> (Storage, HandlerContext) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("embedder");
    let ctx = HandlerContext {
        storage: storage.clone(),
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(
                embedder.dimensions(),
                engram::search::VectorMetric::Cosine,
            ),
        ))),
        progress_reporter: reporter,
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        principal: None,
    };
    (storage, ctx)
}

fn seed_memory(ctx: &HandlerContext, content: &str, mem_type: &str, workspace: &str) -> i64 {
    let params = json!({
        "content": content,
        "memory_type": mem_type,
        "workspace": workspace,
        "importance": 0.85,
    });
    let res = handlers::dispatch(ctx, "memory_create", params);
    res["id"].as_i64().expect("created memory id")
}

#[test]
fn test_smart_retrieve_emits_stage_progress() {
    let (tx, rx) = mpsc::channel::<ProgressNotification>();
    let token = ProgressToken::String("smart-retrieve-token-1".to_string());
    let reporter = Arc::new(ChannelProgressReporter::from_sender(token.clone(), tx));

    let (_, ctx) = create_test_context(Some(reporter));

    seed_memory(
        &ctx,
        "PostgreSQL max connections is 100 on production replica.",
        "decision",
        "infra",
    );

    let params = json!({
        "query": "what is the postgresql max connections setting?",
        "workspace": "infra",
    });

    let resp = handlers::smart_retrieve::memory_smart_retrieve(&ctx, params);
    assert!(resp.get("error").is_none());

    let notifications: Vec<ProgressNotification> = rx.try_iter().collect();
    assert!(
        !notifications.is_empty(),
        "Expected at least one progress notification from smart_retrieve"
    );

    // Verify progress token is matched
    for n in &notifications {
        assert_eq!(n.params.progress_token, token);
        assert!(n.params.total.is_some());
    }

    // Verify the final notification indicates completion
    let last = notifications.last().unwrap();
    assert_eq!(last.params.progress, last.params.total.unwrap());
}

#[test]
fn test_memory_digest_emits_multi_stage_progress() {
    let (tx, rx) = mpsc::channel::<ProgressNotification>();
    let token = ProgressToken::Integer(8842);
    let reporter = Arc::new(ChannelProgressReporter::from_sender(token.clone(), tx));

    let (_, ctx) = create_test_context(Some(reporter));

    seed_memory(
        &ctx,
        "Redis cluster cache policy: LRU eviction with 2GB limit.",
        "decision",
        "cache-ws",
    );
    seed_memory(
        &ctx,
        "Redis key prefixes must use snake_case formatting.",
        "context",
        "cache-ws",
    );

    let params = json!({
        "topic": "Redis cluster cache policy",
        "workspace": "cache-ws",
        "include_graph": true,
        "include_operational_context": false,
    });

    let resp = handlers::digest::memory_digest(&ctx, params);
    assert!(resp.get("error").is_none());

    let notifications: Vec<ProgressNotification> = rx.try_iter().collect();
    assert!(
        notifications.len() >= 4,
        "Expected at least 4 stage notifications for memory_digest, got {}",
        notifications.len()
    );

    for n in &notifications {
        assert_eq!(n.params.progress_token, token);
    }

    // Stage 1: Retrieval
    assert!(notifications.iter().any(|n| n
        .params
        .message
        .as_deref()
        .unwrap_or("")
        .contains("Retrieving")
        || n.params.progress == 1));

    // Stage 4: Synthesis & Complete
    let last = notifications.last().unwrap();
    assert_eq!(last.params.progress, 4);
    assert_eq!(last.params.total, Some(4));
    assert!(last
        .params
        .message
        .as_deref()
        .unwrap_or("")
        .contains("complete"));
}

#[test]
fn test_callback_progress_reporter_integration() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();

    let token = ProgressToken::String("callback-test".to_string());
    let reporter = Arc::new(CallbackProgressReporter::new(token.clone(), move |notif| {
        captured_clone.lock().push(notif);
    }));

    let (_, ctx) = create_test_context(Some(reporter));

    seed_memory(
        &ctx,
        "Kafka partition count is set to 12 for high-throughput events.",
        "decision",
        "streaming-ws",
    );

    let params = json!({
        "query": "Kafka partition count",
        "workspace": "streaming-ws",
    });

    let _ = handlers::smart_retrieve::memory_smart_retrieve(&ctx, params);

    let list = captured.lock().clone();
    assert!(
        !list.is_empty(),
        "CallbackProgressReporter must capture events"
    );
    assert_eq!(list[0].params.progress_token, token);
}

#[test]
fn test_realtime_event_progress_serialization() {
    let event = RealtimeEvent::progress(
        "req-pt-99",
        2,
        Some(5),
        Some("Processing stage 2 of 5".to_string()),
        Some("my-ws".to_string()),
    );

    assert_eq!(event.event_type, EventType::Progress);
    assert_eq!(event.preview.as_deref(), Some("Processing stage 2 of 5"));

    let json = serde_json::to_value(&event).expect("serialize RealtimeEvent");
    assert_eq!(json["type"], "progress");
    assert_eq!(json["data"]["progress_token"], "req-pt-99");
    assert_eq!(json["data"]["progress"], 2);
    assert_eq!(json["data"]["total"], 5);
    assert_eq!(json["data"]["message"], "Processing stage 2 of 5");
    assert_eq!(json["data"]["workspace"], "my-ws");
}

#[test]
fn test_sse_event_type_parsing() {
    let json_type = serde_json::to_string(&EventType::Progress).unwrap();
    assert_eq!(json_type, "\"progress\"");

    let parsed: EventType = serde_json::from_str("\"progress\"").unwrap();
    assert_eq!(parsed, EventType::Progress);
}
