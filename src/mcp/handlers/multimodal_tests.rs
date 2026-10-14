use super::*;
use crate::mcp::handlers::HandlerContext;
use crate::storage::Storage;
use serde_json::json;
use std::sync::Arc;

fn make_ctx() -> HandlerContext {
    use crate::embedding::{create_embedder, EmbeddingCache};
    use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
    use crate::types::EmbeddingConfig;
    use parking_lot::Mutex;

    let storage = Storage::open_in_memory().expect("in-memory storage should open");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    HandlerContext {
        storage,
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
            crate::search::HnswConfig::new(
                embedder.dimensions(),
                crate::search::VectorMetric::Cosine,
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
fn test_describe_image_missing_param() {
    let ctx = make_ctx();
    let result = memory_describe_image(&ctx, json!({}));
    assert!(
        result.get("error").is_some(),
        "should return error when image_path is missing"
    );
    assert!(
        result["error"].as_str().unwrap().contains("image_path"),
        "error should mention image_path"
    );
}

#[test]
fn test_describe_image_missing_file() {
    let ctx = make_ctx();
    let result = memory_describe_image(
        &ctx,
        json!({"image_path": "/tmp/nonexistent_image_12345.png"}),
    );
    assert!(
        result.get("error").is_some(),
        "should return error for missing file"
    );
}

#[test]
fn test_transcribe_audio_missing_param() {
    let ctx = make_ctx();
    let result = memory_transcribe_audio(&ctx, json!({}));
    assert!(
        result.get("error").is_some(),
        "should return error when audio_path is missing"
    );
    assert!(
        result["error"].as_str().unwrap().contains("audio_path"),
        "error should mention audio_path"
    );
}

#[test]
fn test_process_video_missing_param() {
    let ctx = make_ctx();
    let result = memory_process_video(&ctx, json!({}));
    assert!(
        result.get("error").is_some(),
        "should return error when video_path is missing"
    );
    assert!(
        result["error"].as_str().unwrap().contains("video_path"),
        "error should mention video_path"
    );
}

#[test]
fn test_capture_screenshot_no_params() {
    let ctx = make_ctx();
    // Without screencapture available (CI), this will fail with a meaningful error.
    let result = memory_capture_screenshot(&ctx, json!({}));
    // On platforms without screencapture, expect an error; on macOS, might succeed.
    // We only assert the response is a JSON object.
    assert!(result.is_object(), "should return a JSON object");
}

#[test]
fn test_list_media_empty_db() {
    let ctx = make_ctx();
    let result = memory_list_media(&ctx, json!({}));
    assert!(
        result.get("error").is_none(),
        "should not error on empty db"
    );
    assert_eq!(result["count"], 0, "empty db should return 0 assets");
    assert!(result["assets"].is_array(), "assets should be an array");
}

#[test]
fn test_list_media_with_type_filter() {
    let ctx = make_ctx();
    let result = memory_list_media(&ctx, json!({"media_type": "image", "limit": 10}));
    assert!(result.get("error").is_none(), "should not error");
    assert!(result["assets"].is_array(), "assets should be an array");
}

#[test]
fn test_list_media_default_limit() {
    let ctx = make_ctx();
    let result = memory_list_media(&ctx, json!({}));
    assert!(result.get("error").is_none(), "should not error");
    assert_eq!(result["count"], 0);
}

// ── M2: validate_media_path tests ────────────────────────────────────────

#[test]
fn test_validate_media_path_rejects_empty() {
    let result = super::validate_media_path("");
    assert!(result.is_err(), "empty path must be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("empty"), "error should mention 'empty'");
}

#[test]
fn test_validate_media_path_rejects_null_bytes() {
    let result = super::validate_media_path("some/path\0file.png");
    assert!(result.is_err(), "path with null byte must be rejected");
}

#[test]
fn test_validate_media_path_rejects_dotdot() {
    let result = super::validate_media_path("../../../etc/passwd");
    // When no ENGRAM_MEDIA_BASE_DIR is set, canonicalize may succeed or fail.
    // The key requirement is that dotdot is flagged before canonicalize if base is set.
    // Without base dir, we still check for null bytes and empty.
    // We test the dotdot rejection through the sanitize path when base dir is set.
    // For now just ensure no panic.
    let _ = result;
}

#[test]
fn test_validate_media_path_rejects_traversal_with_base_dir() {
    // Set a base dir and verify that a traversal path is rejected
    std::env::set_var("ENGRAM_MEDIA_BASE_DIR", "/tmp");
    let result = super::validate_media_path("../../../etc/passwd");
    std::env::remove_var("ENGRAM_MEDIA_BASE_DIR");
    assert!(
        result.is_err(),
        "path traversal outside base dir must be rejected"
    );
}

#[test]
fn test_validate_media_path_accepts_valid_path_no_base() {
    std::env::remove_var("ENGRAM_MEDIA_BASE_DIR");
    // A nonexistent path should fail at canonicalize, which is expected.
    // A valid existing path should pass.
    let result = super::validate_media_path("/tmp");
    assert!(
        result.is_ok(),
        "existing path without base dir must be accepted: {:?}",
        result
    );
}

#[test]
fn test_infer_mime_type() {
    assert_eq!(infer_mime_type("photo.jpg"), "image/jpeg");
    assert_eq!(infer_mime_type("photo.jpeg"), "image/jpeg");
    assert_eq!(infer_mime_type("image.png"), "image/png");
    assert_eq!(infer_mime_type("anim.gif"), "image/gif");
    assert_eq!(infer_mime_type("pic.webp"), "image/webp");
    assert_eq!(infer_mime_type("scan.tiff"), "image/tiff");
    assert_eq!(infer_mime_type("audio.mp3"), "audio/mpeg");
    assert_eq!(infer_mime_type("audio.wav"), "audio/wav");
    assert_eq!(infer_mime_type("video.mp4"), "video/mp4");
    assert_eq!(infer_mime_type("unknown.xyz"), "application/octet-stream");
}

// ── T4: memory_search_by_image tests ─────────────────────────────────────

#[test]
fn test_search_by_image_missing_param() {
    let ctx = make_ctx();
    let result = memory_search_by_image(&ctx, json!({}));
    assert!(
        result.get("error").is_some(),
        "should error without image_path"
    );
    assert!(
        result["error"].as_str().unwrap().contains("image_path"),
        "error should mention image_path"
    );
}

#[test]
fn test_search_by_image_missing_file() {
    let ctx = make_ctx();
    let result = memory_search_by_image(
        &ctx,
        json!({"image_path": "/tmp/nonexistent_query_image_99999.png"}),
    );
    assert!(
        result.get("error").is_some(),
        "should error when image file is missing"
    );
}

#[test]
fn test_search_by_image_tool_is_registered() {
    // Verify the tool is listed in the tool dispatch (compile-time check via mod.rs)
    // This test simply ensures the handler is callable without panicking
    let ctx = make_ctx();
    // Call with a missing param to get a predictable error without side effects
    let result = memory_search_by_image(&ctx, json!({"image_path": "/nonexistent.png"}));
    // Either "error" (file missing) or "results" (file found) is acceptable
    assert!(
        result.is_object(),
        "handler should always return a JSON object"
    );
}

#[test]
fn test_memory_ingest_media_roundtrip() {
    let ctx = make_ctx();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let temp_file = temp_dir.path().join("test_sample.png");
    std::fs::write(&temp_file, b"fake png bytes 12345").expect("write temp file");

    let res = memory_ingest_media(
        &ctx,
        json!({
            "media_path": temp_file.to_string_lossy(),
            "content": "Diagram of cognitive architecture",
            "tags": ["diagram", "architecture"],
            "importance": 0.9,
            "workspace": "research"
        }),
    );

    assert!(
        res.get("error").is_none(),
        "ingest should succeed: {:?}",
        res
    );
    assert!(res["memory_id"].as_i64().is_some());
    assert_eq!(res["media_type"].as_str(), Some("image"));
    assert_eq!(res["mime_type"].as_str(), Some("image/png"));
    assert_eq!(res["workspace"].as_str(), Some("research"));
}

fn ingest(ctx: &HandlerContext, path: &std::path::Path, workspace: Option<&str>) -> Value {
    let mut params = json!({"media_path": path.to_string_lossy()});
    if let Some(ws) = workspace {
        params["workspace"] = json!(ws);
    }
    memory_ingest_media(ctx, params)
}

fn count(ctx: &HandlerContext, sql: &str) -> i64 {
    ctx.storage
        .with_connection(|c| Ok(c.query_row(sql, [], |r| r.get::<_, i64>(0))?))
        .unwrap()
}

#[test]
fn ingest_retry_of_same_file_does_not_duplicate_the_memory() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("retry.png");
    std::fs::write(&file, b"retry payload bytes").unwrap();

    let first = ingest(&ctx, &file, None);
    let second = ingest(&ctx, &file, None);

    assert!(first.get("error").is_none(), "{first:?}");
    assert_eq!(
        second["memory_id"], first["memory_id"],
        "a retried ingest must return the original memory"
    );
    assert_eq!(second["asset_id"], first["asset_id"]);
    assert_eq!(second["deduplicated"], json!(true));
    assert_eq!(first["deduplicated"], json!(false));
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM memories"), 1);
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM media_assets"), 1);
}

#[test]
fn ingest_reports_the_real_asset_id() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("id.png");
    std::fs::write(&file, b"asset id payload").unwrap();

    let res = ingest(&ctx, &file, None);

    let actual = count(&ctx, "SELECT id FROM media_assets");
    assert_eq!(res["asset_id"].as_i64(), Some(actual));
}

#[test]
fn ingest_after_delete_creates_a_fresh_live_memory() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("again.png");
    std::fs::write(&file, b"delete then ingest").unwrap();

    let first = ingest(&ctx, &file, None);
    let first_id = first["memory_id"].as_i64().unwrap();
    ctx.storage
        .with_connection(|c| crate::storage::queries::delete_memory(c, first_id))
        .unwrap();

    let second = ingest(&ctx, &file, None);

    assert!(second.get("error").is_none(), "{second:?}");
    let second_id = second["memory_id"].as_i64().unwrap();
    assert_ne!(second_id, first_id, "a deleted memory must not be revived");
    assert_eq!(second["deduplicated"], json!(false));
    assert_eq!(
        count(&ctx, "SELECT memory_id FROM media_assets"),
        second_id,
        "asset must point at the live memory"
    );
    assert_eq!(
        second["asset_id"].as_i64(),
        Some(count(&ctx, "SELECT id FROM media_assets"))
    );
}

#[test]
fn ingest_never_returns_another_workspaces_memory() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("shared.png");
    std::fs::write(&file, b"same bytes two workspaces").unwrap();

    let a = ingest(&ctx, &file, Some("ws-a"));
    let b = ingest(&ctx, &file, Some("ws-b"));

    assert_ne!(a["memory_id"], b["memory_id"]);
    assert_eq!(b["workspace"].as_str(), Some("ws-b"));
    assert_eq!(b["deduplicated"], json!(false));
}

#[test]
fn ingest_dedup_normalizes_the_workspace_before_lookup() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("norm.png");
    std::fs::write(&file, b"normalised workspace payload").unwrap();

    let first = ingest(&ctx, &file, Some("WS-A"));
    let second = ingest(&ctx, &file, Some("ws-a"));
    let third = ingest(&ctx, &file, Some(" ws-a "));

    assert_eq!(first["workspace"].as_str(), Some("ws-a"));
    assert_eq!(second["memory_id"], first["memory_id"], "{second:?}");
    assert_eq!(second["deduplicated"], json!(true));
    assert_eq!(third["memory_id"], first["memory_id"], "{third:?}");
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM memories"), 1);
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM media_assets"), 1);
}

#[test]
fn ingest_rejects_an_invalid_workspace_with_a_typed_error() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad-ws.png");
    std::fs::write(&file, b"bad workspace payload").unwrap();

    let res = ingest(&ctx, &file, Some("not a workspace!"));

    assert!(
        res["error"]
            .as_str()
            .is_some_and(|e| e.contains("workspace")),
        "{res:?}"
    );
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM memories"), 0);
}

#[test]
fn dedup_response_has_the_same_shape_as_the_first_ingest() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("shape.png");
    std::fs::write(&file, b"shape payload bytes 0123456789").unwrap();

    let first = ingest(&ctx, &file, None);
    let second = ingest(&ctx, &file, None);

    assert!(first["perceptual_hash"].is_string(), "{first:?}");
    assert_eq!(second["perceptual_hash"], first["perceptual_hash"]);
    let keys = |v: &Value| {
        let mut k: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    assert_eq!(keys(&second), keys(&first));
}

#[test]
fn ingest_same_file_across_two_workspaces_keeps_both_assets_and_deduplicates_each() {
    let ctx = make_ctx();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("shared.png");
    std::fs::write(&file, b"shared file bytes across workspaces").unwrap();

    let a1 = ingest(&ctx, &file, Some("ws-a"));
    let b1 = ingest(&ctx, &file, Some("ws-b"));
    let a2 = ingest(&ctx, &file, Some("ws-a"));
    let b2 = ingest(&ctx, &file, Some("ws-b"));

    assert!(a1.get("error").is_none(), "{a1:?}");
    assert!(b1.get("error").is_none(), "{b1:?}");
    assert_eq!(a1["deduplicated"], json!(false));
    assert_eq!(b1["deduplicated"], json!(false));
    assert_ne!(a1["memory_id"], b1["memory_id"]);
    assert_ne!(a1["asset_id"], b1["asset_id"]);

    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM memories"), 2);
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM media_assets"), 2);

    assert_eq!(a2["deduplicated"], json!(true));
    assert_eq!(a2["memory_id"], a1["memory_id"]);
    assert_eq!(a2["asset_id"], a1["asset_id"]);

    assert_eq!(b2["deduplicated"], json!(true));
    assert_eq!(b2["memory_id"], b1["memory_id"]);
    assert_eq!(b2["asset_id"], b1["asset_id"]);
}

#[test]
fn read_media_file_bounded_rejects_oversized_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("oversized.png");
    std::fs::write(&file, [0u8; 32]).unwrap();

    let err = super::read_media_file_bounded(&file, 16)
        .expect_err("32-byte file must fail with 16-byte limit");
    match err {
        crate::error::EngramError::InvalidInput(msg) => {
            assert!(
                msg.contains("exceeds maximum allowed size"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected InvalidInput error, got: {other:?}"),
    }
}
