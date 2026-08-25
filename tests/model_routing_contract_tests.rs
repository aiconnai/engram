//! Integration tests for RFC 0011 Model Routing Contract.

use parking_lot::{Mutex, RwLock};
use serde_json::json;
use std::sync::Arc;

use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::routing::{ModelPurpose, ModelRouter, OfflinePolicy, RouteStatus};
use engram::Storage;

fn setup_test_context() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory database");
    let embedder = engram::embedding::create_embedder(&engram::types::EmbeddingConfig::default())
        .expect("tfidf embedder");

    HandlerContext {
        storage,
        embedder,
        fuzzy_engine: Arc::new(Mutex::new(engram::search::FuzzyEngine::new())),
        search_config: engram::search::SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
        search_cache: Arc::new(engram::search::SearchResultCache::new(
            engram::search::AdaptiveCacheConfig::default(),
        )),
        hnsw_index: Arc::new(RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(128, engram::search::VectorMetric::Cosine),
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
fn test_default_model_routes_registration() {
    let router = ModelRouter::new();
    let routes = router.list_routes();

    assert!(routes.len() >= 12);

    let purposes: Vec<ModelPurpose> = routes.iter().map(|r| r.purpose).collect();
    assert!(purposes.contains(&ModelPurpose::EmbeddingText));
    assert!(purposes.contains(&ModelPurpose::EmbeddingImage));
    assert!(purposes.contains(&ModelPurpose::Rerank));
    assert!(purposes.contains(&ModelPurpose::VisionDescribeImage));
    assert!(purposes.contains(&ModelPurpose::AudioTranscribe));
    assert!(purposes.contains(&ModelPurpose::LlmCouncil));
    assert!(purposes.contains(&ModelPurpose::TokenCount));
    assert!(purposes.contains(&ModelPurpose::DeterministicEval));
}

#[test]
fn test_deterministic_local_route_resolution() {
    let router = ModelRouter::new();

    // 1. Text embedding default route -> TF-IDF (local, offline)
    let res = router.resolve(ModelPurpose::EmbeddingText, None);
    assert_eq!(res.status, RouteStatus::Ok);
    assert_eq!(res.provider_id, "tfidf");
    assert_eq!(res.model_id, "tfidf-128");
    assert_eq!(res.offline_policy, OfflinePolicy::WorksOffline);
    assert!(!res.fallback_used);
    assert!(res.warnings.is_empty());

    // 2. Token counting -> deterministic local rule-based
    let res_tok = router.resolve(ModelPurpose::TokenCount, None);
    assert_eq!(res_tok.status, RouteStatus::Ok);
    assert_eq!(res_tok.provider_id, "tiktoken_local");
    assert_eq!(res_tok.offline_policy, OfflinePolicy::WorksOffline);
    assert!(!res_tok.fallback_used);

    // 3. Deterministic eval -> local engine
    let res_eval = router.resolve(ModelPurpose::DeterministicEval, None);
    assert_eq!(res_eval.status, RouteStatus::Ok);
    assert_eq!(res_eval.provider_id, "eval_engine");
    assert_eq!(res_eval.offline_policy, OfflinePolicy::WorksOffline);
    assert!(!res_eval.fallback_used);

    // 4. Reranking default -> local rule-based
    let res_rerank = router.resolve(ModelPurpose::Rerank, None);
    assert_eq!(res_rerank.status, RouteStatus::Ok);
    assert_eq!(res_rerank.provider_id, "rule_based");
    assert_eq!(res_rerank.offline_policy, OfflinePolicy::WorksOffline);
    assert!(!res_rerank.fallback_used);
}

#[test]
fn test_missing_secret_fallback_and_reporting() {
    // Ensure OPENAI_API_KEY is temporarily unset for deterministic testing
    let orig_openai = std::env::var("OPENAI_API_KEY").ok();
    std::env::remove_var("OPENAI_API_KEY");

    let router = ModelRouter::new();

    // 1. Text embedding with preferred "openai" (feature is active, secret is missing)
    let res_openai = router.resolve(ModelPurpose::EmbeddingText, Some("openai"));
    assert_eq!(res_openai.status, RouteStatus::MissingSecret);
    assert_eq!(res_openai.provider_id, "openai");
    assert_eq!(res_openai.model_id, "text-embedding-3-small");
    assert_eq!(
        res_openai.required_secret,
        Some("OPENAI_API_KEY".to_string())
    );
    assert_eq!(res_openai.fallback_available, Some("tfidf".to_string()));
    assert!(!res_openai.fallback_used);

    // 2. Text embedding with preferred "voyage" (feature status depends on compilation)
    let res_voyage = router.resolve(ModelPurpose::EmbeddingText, Some("voyage"));
    #[cfg(feature = "voyage")]
    assert_eq!(res_voyage.status, RouteStatus::MissingSecret);
    #[cfg(not(feature = "voyage"))]
    assert_eq!(res_voyage.status, RouteStatus::FeatureDisabled);
    assert_eq!(res_voyage.provider_id, "voyage");
    assert_eq!(res_voyage.fallback_available, Some("tfidf".to_string()));
    assert!(!res_voyage.fallback_used);

    // Restore environment
    if let Some(val) = orig_openai {
        std::env::set_var("OPENAI_API_KEY", val);
    }
}

#[test]
fn test_unknown_provider_resolution() {
    let router = ModelRouter::new();
    let res = router.resolve(
        ModelPurpose::EmbeddingText,
        Some("non_existent_provider_xyz"),
    );
    assert_eq!(res.status, RouteStatus::Unavailable);
    assert!(res.reason.is_some());
}

#[test]
fn test_mcp_model_route_resolve_and_list_tools_dispatch() {
    let ctx = setup_test_context();

    // 1. Dispatch model_route_resolve for text embedding
    let resolve_val = dispatch(
        &ctx,
        "model_route_resolve",
        json!({
            "purpose": "embedding_text"
        }),
    );
    assert_eq!(resolve_val["status"], "ok");
    assert_eq!(resolve_val["provider_id"], "tfidf");
    assert_eq!(resolve_val["model_id"], "tfidf-128");

    // 2. Dispatch model_route_resolve with missing required purpose
    let err_val = dispatch(&ctx, "model_route_resolve", json!({}));
    assert!(err_val["error"].is_string());

    // 3. Dispatch model_routes_list (all)
    let list_val = dispatch(&ctx, "model_routes_list", json!({}));
    let count = list_val["routes_count"].as_u64().unwrap();
    assert!(count >= 12);
    let routes = list_val["routes"].as_array().unwrap();
    assert_eq!(routes.len() as u64, count);

    // 4. Dispatch model_routes_list filtered by purpose
    let filtered_val = dispatch(
        &ctx,
        "model_routes_list",
        json!({
            "purpose": "rerank"
        }),
    );
    let filtered_routes = filtered_val["routes"].as_array().unwrap();
    for r in filtered_routes {
        assert_eq!(r["purpose"], "rerank");
    }
}
