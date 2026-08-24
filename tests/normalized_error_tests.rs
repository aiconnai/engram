//! Integration tests for Normalized Tool Error Contract (RFC 0006).

use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::error::{ToolError, ToolErrorCode};
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::mcp::permission::{permission_denial_for_mode, PermissionMode};
use engram::mcp::protocol::ToolCallResult;
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn test_ctx() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    HandlerContext {
        storage,
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
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        progress_reporter: None,
        principal: None,
    }
}

#[test]
fn test_unknown_tool_returns_normalized_error() {
    let ctx = test_ctx();
    let res = dispatch(&ctx, "nonexistent_custom_tool", json!({}));

    assert!(ToolError::is_error_response(&res));
    assert_eq!(res["error"]["code"], "tool_not_found");
    assert_eq!(res["error"]["tool"], "nonexistent_custom_tool");
    assert!(res["error"]["message"]
        .as_str()
        .unwrap()
        .contains("nonexistent_custom_tool"));
}

#[test]
fn test_missing_argument_returns_normalized_error() {
    let ctx = test_ctx();

    // 1. memory_get without id
    let res = dispatch(&ctx, "memory_get", json!({}));
    assert!(ToolError::is_error_response(&res));
    assert_eq!(res["error"]["code"], "missing_argument");
    assert_eq!(res["error"]["details"]["argument"], "id");

    // 2. agent_register without agent_id
    let res_agent = dispatch(&ctx, "agent_register", json!({}));
    assert!(ToolError::is_error_response(&res_agent));
    assert_eq!(res_agent["error"]["code"], "missing_argument");
    assert_eq!(res_agent["error"]["details"]["argument"], "agent_id");
}

#[test]
fn test_not_found_returns_normalized_error() {
    let ctx = test_ctx();
    let res = dispatch(&ctx, "memory_get", json!({"id": 999999}));

    assert!(ToolError::is_error_response(&res));
    assert_eq!(res["error"]["code"], "not_found");
    assert_eq!(res["error"]["details"]["entity"], "memory");
    assert_eq!(res["error"]["details"]["id"], "999999");
}

#[test]
fn test_permission_denial_returns_normalized_error() {
    let denial = permission_denial_for_mode("memory_delete", PermissionMode::ReadOnly)
        .expect("should deny memory_delete in read_only");

    assert!(ToolError::is_error_response(&denial));
    assert_eq!(denial["error"]["code"], "permission_denied");
    assert_eq!(denial["error"]["tool"], "memory_delete");
    assert_eq!(denial["error"]["current_mode"], "read_only");
    assert_eq!(denial["error"]["required_mode"], "admin");
}

#[test]
fn test_tool_call_result_detects_error_envelope() {
    let error_val = ToolError::invalid_params("Invalid parameter format").into_value();
    let tool_res = ToolCallResult::from_tool_output(&error_val);

    assert_eq!(tool_res.is_error, Some(true));
    assert_eq!(tool_res.content.len(), 1);

    let success_val = json!({"status": "success", "memory_id": 123});
    let success_res = ToolCallResult::from_tool_output(&success_val);
    assert_eq!(success_res.is_error, None);
}

#[test]
fn test_typed_tool_error_constructors_and_display() {
    let err = ToolError::version_mismatch(2, 1);
    assert_eq!(err.code(), ToolErrorCode::VersionMismatch.as_str());
    assert_eq!(
        format!("{err}"),
        "[version_mismatch] Version mismatch: expected 2, but found 1"
    );
    assert_eq!(err.error.details.unwrap()["expected_version"], 2);

    let audit_err = ToolError::conflict("concurrent edit conflict").with_audit_id("audit-xyz-123");
    assert_eq!(audit_err.error.audit_id.as_deref(), Some("audit-xyz-123"));
}
