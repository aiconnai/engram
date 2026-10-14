//! MCP tool handlers for multimodal features: vision, audio, screenshot, and video.
//!
//! All handlers are feature-gated with `#[cfg(feature = "multimodal")]`.
//! Async provider calls are executed via a short-lived `tokio::runtime::Runtime`
//! so that the synchronous MCP dispatch loop is not affected.

#[cfg(feature = "multimodal")]
use serde_json::{json, Value};

#[cfg(feature = "multimodal")]
use super::HandlerContext;

// ── memory_describe_image ─────────────────────────────────────────────────────

/// Describe an image file using the configured vision provider.
///
/// Required params:
/// - `image_path` (string) — absolute or relative path to the image file
///
/// Optional params:
/// - `prompt` (string) — custom prompt passed to the vision model
///
/// Returns: `{ text, model, provider }`
#[cfg(feature = "multimodal")]
pub fn memory_describe_image(ctx: &HandlerContext, params: Value) -> Value {
    use crate::multimodal::vision::{VisionInput, VisionOptions, VisionProviderFactory};

    let image_path = match params.get("image_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "image_path is required"}),
    };

    let prompt = params
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let provider = match VisionProviderFactory::from_env() {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Vision provider not configured: {}", e)}),
    };

    let validated_path = match validate_media_path(&image_path) {
        Ok(p) => p,
        Err(e) => return json!({"error": e.to_string()}),
    };
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(&validated_path) {
        return json!({"error": e.to_string()});
    }

    let image_bytes = match read_media_file_bounded(&validated_path, MAX_MEDIA_FILE_BYTES) {
        Ok(bytes) => bytes,
        Err(e) => {
            return json!({"error": format!("Failed to read image file '{}': {}", image_path, e)})
        }
    };

    let mime_type = infer_mime_type(&image_path);

    let input = VisionInput {
        image_bytes,
        mime_type,
    };

    let opts = VisionOptions {
        prompt,
        max_tokens: None,
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return json!({"error": format!("Failed to create async runtime: {}", e)}),
    };

    match rt.block_on(provider.describe_image(input, opts)) {
        Ok(desc) => json!({
            "text": desc.text,
            "model": desc.model,
            "provider": desc.provider,
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── memory_transcribe_audio ───────────────────────────────────────────────────

/// Transcribe an audio file using the configured audio transcription provider.
///
/// Required params:
/// - `audio_path` (string) — path to the audio file
///
/// Returns: `{ text, language, duration_secs, segments }`
#[cfg(feature = "multimodal")]
pub fn memory_transcribe_audio(ctx: &HandlerContext, params: Value) -> Value {
    use crate::multimodal::audio::AudioTranscriberFactory;

    let audio_path = match params.get("audio_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "audio_path is required"}),
    };

    let validated_audio = match validate_media_path(&audio_path) {
        Ok(p) => p,
        Err(e) => return json!({"error": e.to_string()}),
    };
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(&validated_audio) {
        return json!({"error": e.to_string()});
    }

    let transcriber = match AudioTranscriberFactory::from_env() {
        Ok(t) => t,
        Err(e) => {
            return json!({"error": format!("Audio transcription provider not configured: {}", e)})
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return json!({"error": format!("Failed to create async runtime: {}", e)}),
    };

    match rt.block_on(transcriber.transcribe(&validated_audio)) {
        Ok(result) => {
            let segments: Vec<Value> = result
                .segments
                .iter()
                .map(|s| {
                    json!({
                        "start_secs": s.start_secs,
                        "end_secs": s.end_secs,
                        "text": s.text,
                    })
                })
                .collect();

            json!({
                "text": result.text,
                "language": result.language,
                "duration_secs": result.duration_secs,
                "segments": segments,
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── memory_capture_screenshot ─────────────────────────────────────────────────

/// Capture a screenshot of the full screen or a specific application window.
///
/// Optional params:
/// - `app_name` (string) — if provided, captures that app's window; otherwise captures full screen
///
/// Returns: `{ image_path, width, height, file_size, file_hash }`
#[cfg(feature = "multimodal")]
pub fn memory_capture_screenshot(_ctx: &HandlerContext, params: Value) -> Value {
    use crate::multimodal::screenshot::ScreenshotCapture;

    let capture = match ScreenshotCapture::new() {
        Ok(c) => c,
        Err(e) => {
            return json!({"error": format!("Failed to initialize screenshot capture: {}", e)})
        }
    };

    let app_name = params
        .get("app_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let result = if let Some(app) = app_name {
        capture.capture_window(&app)
    } else {
        capture.capture()
    };

    match result {
        Ok(screenshot) => json!({
            "image_path": screenshot.image_path.to_string_lossy(),
            "width": screenshot.width,
            "height": screenshot.height,
            "file_size": screenshot.file_size,
            "file_hash": screenshot.file_hash,
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── memory_process_video ──────────────────────────────────────────────────────

/// Process a video file: extract metadata and keyframe descriptions.
///
/// Required params:
/// - `video_path` (string) — path to the video file
///
/// Returns: `{ metadata, keyframe_descriptions, summary }`
#[cfg(feature = "multimodal")]
pub fn memory_process_video(ctx: &HandlerContext, params: Value) -> Value {
    use crate::multimodal::video::VideoProcessor;
    use crate::multimodal::vision::VisionProviderFactory;

    let video_path = match params.get("video_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "video_path is required"}),
    };

    let vision = match VisionProviderFactory::from_env() {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Vision provider not configured: {}", e)}),
    };

    let validated_video = match validate_media_path(&video_path) {
        Ok(p) => p,
        Err(e) => return json!({"error": e.to_string()}),
    };
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(&validated_video) {
        return json!({"error": e.to_string()});
    }

    let processor = VideoProcessor::new();

    if let Err(e) = processor.check_availability() {
        return json!({"error": format!("Video processing unavailable: {}", e)});
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return json!({"error": format!("Failed to create async runtime: {}", e)}),
    };

    match rt.block_on(processor.create_video_memory(&validated_video, vision.as_ref())) {
        Ok(video_memory) => {
            let meta = &video_memory.metadata;
            json!({
                "metadata": {
                    "duration_secs": meta.duration_secs,
                    "width": meta.width,
                    "height": meta.height,
                    "codec": meta.codec,
                    "file_size": meta.file_size,
                    "file_hash": meta.file_hash,
                },
                "keyframe_descriptions": video_memory.keyframe_descriptions,
                "summary": video_memory.summary,
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── memory_list_media ─────────────────────────────────────────────────────────

/// List media assets stored in the media_assets table.
///
/// Optional params:
/// - `media_type` (string) — filter by type: "image", "audio", "video"
/// - `limit` (integer) — maximum number of results (default 50)
///
/// Returns: `{ assets: [...], count }`
#[cfg(feature = "multimodal")]
pub fn memory_list_media(ctx: &HandlerContext, params: Value) -> Value {
    let media_type = params
        .get("media_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    ctx.storage
        .with_connection(|conn| {
            let assets = query_media_assets(conn, media_type.as_deref(), limit)?;
            Ok(json!({
                "assets": assets,
                "count": assets.len(),
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Query media_assets table, optionally filtered by media_type.
#[cfg(feature = "multimodal")]
fn query_media_assets(
    conn: &rusqlite::Connection,
    media_type: Option<&str>,
    limit: usize,
) -> crate::error::Result<Vec<serde_json::Value>> {
    let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
        if let Some(mt) = media_type {
            (
                "SELECT id, memory_id, media_type, file_hash, file_path, file_size, \
                 mime_type, duration_secs, width, height, transcription, description, \
                 provider, model, created_at \
                 FROM media_assets WHERE media_type = ?1 \
                 ORDER BY created_at DESC LIMIT ?2"
                    .to_string(),
                vec![Box::new(mt.to_string()), Box::new(limit as i64)],
            )
        } else {
            (
                "SELECT id, memory_id, media_type, file_hash, file_path, file_size, \
                 mime_type, duration_secs, width, height, transcription, description, \
                 provider, model, created_at \
                 FROM media_assets ORDER BY created_at DESC LIMIT ?1"
                    .to_string(),
                vec![Box::new(limit as i64)],
            )
        };

    let mut stmt = conn.prepare(&sql)?;

    let rows: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "memory_id": row.get::<_, i64>(1)?,
                "media_type": row.get::<_, String>(2)?,
                "file_hash": row.get::<_, String>(3)?,
                "file_path": row.get::<_, Option<String>>(4)?,
                "file_size": row.get::<_, Option<i64>>(5)?,
                "mime_type": row.get::<_, Option<String>>(6)?,
                "duration_secs": row.get::<_, Option<f64>>(7)?,
                "width": row.get::<_, Option<i64>>(8)?,
                "height": row.get::<_, Option<i64>>(9)?,
                "transcription": row.get::<_, Option<String>>(10)?,
                "description": row.get::<_, Option<String>>(11)?,
                "provider": row.get::<_, Option<String>>(12)?,
                "model": row.get::<_, Option<String>>(13)?,
                "created_at": row.get::<_, String>(14)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

// ── memory_search_by_image ────────────────────────────────────────────────────

/// Search memories by image similarity.
///
/// Uses multimodal embeddings (CLIP-style description-mediated) to embed the
/// query image, then searches the vector index for nearest neighbours. Falls
/// back to describing the image with a vision model and searching by text if
/// no multimodal embedder is available.
///
/// Required params:
/// - `image_path` (string) — path to the local image file
///
/// Optional params:
/// - `limit` (integer, default 10) — maximum results
/// - `min_score` (number, default 0.0) — minimum similarity score
/// - `workspace` (string) — restrict search to workspace
/// - `strategy` (string: "clip" | "description" | "auto") — embedding strategy
///
/// Returns: `{ results: [...], query_description, strategy_used }`
#[cfg(feature = "multimodal")]
pub fn memory_search_by_image(ctx: &HandlerContext, params: Value) -> Value {
    use crate::search::{hybrid_search, Reranker};
    use crate::types::SearchOptions;

    let image_path = match params.get("image_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "image_path is required"}),
    };

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as i64;
    let min_score = params
        .get("min_score")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let strategy = params
        .get("strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");
    #[cfg(not(feature = "clip-embeddings"))]
    let _ = strategy;

    // Step 1: Validate and read the image file
    let validated_image = match validate_media_path(&image_path) {
        Ok(p) => p,
        Err(e) => return json!({"error": e.to_string()}),
    };
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(&validated_image) {
        return json!({"error": e.to_string()});
    }

    let image_bytes = match read_media_file_bounded(&validated_image, MAX_MEDIA_FILE_BYTES) {
        Ok(b) => b,
        Err(e) => {
            return json!({"error": format!("Failed to read image file '{}': {}", image_path, e)})
        }
    };

    let mime_type = infer_mime_type(&image_path);

    // Step 2: Generate a text description of the image
    // This is the universal fallback path — works even without a CLIP embedder.
    let vision_provider = crate::multimodal::vision::VisionProviderFactory::from_env().ok();

    let description = if let Some(ref provider) = vision_provider {
        let input = crate::multimodal::vision::VisionInput {
            image_bytes: image_bytes.clone(),
            mime_type: mime_type.clone(),
        };
        let opts = crate::multimodal::vision::VisionOptions {
            prompt: Some(
                "Describe this image in detail, including all visual elements, text, colors, and context. Be comprehensive.".to_string(),
            ),
            max_tokens: Some(512),
        };
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => return json!({"error": format!("Failed to create async runtime: {}", e)}),
        };
        match rt.block_on(provider.describe_image(input, opts)) {
            Ok(desc) => Some(desc.text),
            Err(e) => {
                tracing::warn!(
                    error = %crate::observability::redact::redacted(&e),
                    "Vision model failed, falling back to filename hint"
                );
                None
            }
        }
    } else {
        None
    };

    // Step 3: Build the query text for embedding
    // Prefer the vision description; fall back to the file name
    let query_text = description.clone().unwrap_or_else(|| {
        std::path::Path::new(&image_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image")
            .replace(['-', '_'], " ")
    });

    // Step 4: Determine strategy
    let strategy_used;

    #[cfg(feature = "clip-embeddings")]
    let query_embedding: Option<Vec<f32>> = if strategy == "clip" || strategy == "auto" {
        use crate::embedding::clip::{ClipEmbedder, MultimodalEmbedder};
        if let Ok(clip) = ClipEmbedder::from_env() {
            match clip.embed_image_sync(&image_bytes, &mime_type) {
                Ok(v) => {
                    strategy_used = "clip";
                    Some(v)
                }
                Err(e) => {
                    tracing::warn!(
                        error = %crate::observability::redact::redacted(&e),
                        "CLIP embedding failed, falling back to description"
                    );
                    strategy_used = "description";
                    ctx.embedder.embed(&query_text).ok()
                }
            }
        } else {
            strategy_used = "description";
            ctx.embedder.embed(&query_text).ok()
        }
    } else {
        strategy_used = "description";
        ctx.embedder.embed(&query_text).ok()
    };

    #[cfg(not(feature = "clip-embeddings"))]
    let query_embedding: Option<Vec<f32>> = {
        strategy_used = "description";
        ctx.embedder.embed(&query_text).ok()
    };

    // Step 5: Run hybrid search with the generated embedding
    let options = SearchOptions {
        limit: Some(limit),
        min_score,
        workspace,
        ..Default::default()
    };

    let search_config = ctx.search_config.clone();
    let embedding_ref = query_embedding.as_deref();

    ctx.storage
        .with_connection(|conn| {
            let results =
                hybrid_search(conn, &query_text, embedding_ref, &options, &search_config)?;
            Ok(results)
        })
        .map(|results| {
            let reranker = Reranker::new();
            let reranked = reranker.rerank(results, &query_text, None);
            json!({
                "results": reranked,
                "query_description": description,
                "strategy_used": strategy_used,
            })
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── memory_sync_media ─────────────────────────────────────────────────────────

/// Sync local media assets to S3/R2 cloud storage.
///
/// Reads the `media_assets` table for files that have not yet been uploaded to
/// cloud storage, uploads each one, and updates the `file_path` column in place
/// with the resulting cloud URL.
///
/// Requires both `multimodal` AND `cloud` features.
///
/// Optional params:
/// - `dry_run` (bool) — if true, report what would be synced without uploading
///
/// Returns: `{ assets_examined, assets_already_synced, assets_uploaded, assets_failed, errors, dry_run }`
#[cfg(all(feature = "multimodal", feature = "cloud"))]
pub fn memory_sync_media(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::image_storage::{sync_to_cloud, ImageStorageConfig, MediaSyncReport};

    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Build config from environment variables (same approach as existing cloud sync)
    let config = ImageStorageConfig {
        local_dir: ImageStorageConfig::default().local_dir,
        s3_bucket: std::env::var("ENGRAM_S3_BUCKET")
            .or_else(|_| std::env::var("R2_BUCKET"))
            .ok(),
        s3_endpoint: std::env::var("AWS_ENDPOINT_URL")
            .or_else(|_| std::env::var("R2_ENDPOINT"))
            .ok(),
        public_domain: std::env::var("ENGRAM_MEDIA_PUBLIC_DOMAIN").ok(),
    };

    if config.s3_bucket.is_none() {
        return json!({
            "error": "S3/R2 bucket not configured. Set ENGRAM_S3_BUCKET or R2_BUCKET environment variable."
        });
    }

    ctx.storage
        .with_connection(|conn| sync_to_cloud(conn, &config, dry_run))
        .map(|report: MediaSyncReport| json!(report))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── memory_ingest_media ───────────────────────────────────────────────────────

/// Ingest and analyze a media asset (image, audio, or video) into a durable memory.
#[cfg(feature = "multimodal")]
pub fn memory_ingest_media(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::create_memory;
    use crate::types::{CreateMemoryInput, MemoryType};
    use rusqlite::OptionalExtension;
    use sha2::{Digest, Sha256};

    let media_path = match params
        .get("media_path")
        .or_else(|| params.get("path"))
        .or_else(|| params.get("file_path"))
        .and_then(|v| v.as_str())
    {
        Some(p) => p.to_string(),
        None => return json!({"error": "media_path is required"}),
    };

    let validated_path = match validate_media_path(&media_path) {
        Ok(p) => p,
        Err(e) => return json!({"error": e.to_string()}),
    };
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(&validated_path) {
        return json!({"error": e.to_string()});
    }

    let file_bytes = match read_media_file_bounded(&validated_path, MAX_MEDIA_FILE_BYTES) {
        Ok(bytes) => bytes,
        Err(e) => {
            return json!({"error": format!("Failed to read media file '{}': {}", media_path, e)})
        }
    };

    let file_size = file_bytes.len() as i64;
    let mut hasher = Sha256::new();
    hasher.update(&file_bytes);
    let file_hash = format!("{:x}", hasher.finalize());

    let mime_type = infer_mime_type(&media_path);
    let explicit_type = params.get("media_type").and_then(|v| v.as_str());
    let media_type_str = match explicit_type {
        Some("image") | Some("audio") | Some("video") => explicit_type.unwrap().to_string(),
        _ => {
            if mime_type.starts_with("image/") {
                "image".to_string()
            } else if mime_type.starts_with("audio/") {
                "audio".to_string()
            } else if mime_type.starts_with("video/") {
                "video".to_string()
            } else {
                "image".to_string()
            }
        }
    };

    let memory_type = match media_type_str.as_str() {
        "image" => MemoryType::Image,
        "audio" => MemoryType::Audio,
        "video" => MemoryType::Video,
        _ => MemoryType::Note,
    };

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let importance = params
        .get("importance")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);

    let default_content = format!("[Media: {} ({})] {}", media_type_str, mime_type, media_path);
    let content = params
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_content)
        .to_string();

    let media_url = format!("local://{}", media_path);

    // Storage keys memories by the normalised workspace ("WS-A" -> "ws-a"), so
    // the dedup lookup must use the same form or a case/whitespace variant
    // would create a duplicate and orphan the first asset.
    let requested_ws = match workspace.as_deref() {
        Some(ws) => match crate::types::normalize_workspace(ws) {
            Ok(normalized) => normalized,
            Err(e) => return json!({"error": format!("Invalid workspace: {e}")}),
        },
        None => "default".to_string(),
    };

    let res = ctx.storage.with_transaction(|conn| {
        // Retry idempotence: the same bytes already ingested into the same
        // (normalised) workspace and still backed by a live memory return that memory
        // instead of creating a second one and re-pointing the asset row.
        // Only live rows are matched, so a deleted memory is never revived, and
        // only the requested workspace, so another workspace's memory is never
        // returned. Not exactly-once: it is a content-hash dedupe.
        let existing: Option<(i64, i64, String, String)> = conn
            .query_row(
                "SELECT a.id, m.id, m.workspace, m.created_at
                 FROM media_assets a JOIN memories m ON m.id = a.memory_id
                 WHERE a.workspace = ?1 AND a.file_hash = ?2 AND m.valid_to IS NULL AND m.workspace = ?1",
                rusqlite::params![requested_ws, file_hash],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        if let Some((asset_id, memory_id, ws, created_at)) = existing {
            // Same response shape as a first ingest; the perceptual hash is
            // derived from the bytes, not stored.
            let phash = (media_type_str == "image").then(|| {
                crate::multimodal::hashing::format_phash(
                    crate::multimodal::hashing::compute_perceptual_hash(&file_bytes),
                )
            });
            return Ok(json!({
                "memory_id": memory_id,
                "asset_id": asset_id,
                "media_type": media_type_str,
                "media_url": media_url,
                "file_hash": file_hash,
                "perceptual_hash": phash,
                "file_size": file_size,
                "mime_type": mime_type,
                "workspace": ws,
                "created_at": created_at,
                "deduplicated": true
            }));
        }

        let input = CreateMemoryInput {
            content,
            memory_type,
            tags,
            workspace: workspace.clone(),
            importance,
            media_url: Some(media_url.clone()),
            ..Default::default()
        };

        let memory = create_memory(conn, &input)?;

        conn.execute(
            "INSERT INTO media_assets (
                memory_id, workspace, media_type, file_hash, file_path, file_size, mime_type
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(workspace, file_hash) DO UPDATE SET
                memory_id = excluded.memory_id,
                media_type = excluded.media_type,
                file_path = excluded.file_path,
                file_size = excluded.file_size,
                mime_type = excluded.mime_type",
            rusqlite::params![
                memory.id,
                &memory.workspace,
                media_type_str,
                file_hash,
                media_path,
                file_size,
                mime_type
            ],
        )?;

        // `last_insert_rowid` is stale after an upsert that updated; ask the row.
        let asset_id: i64 = conn.query_row(
            "SELECT id FROM media_assets WHERE workspace = ?1 AND file_hash = ?2",
            rusqlite::params![memory.workspace, file_hash],
            |row| row.get(0),
        )?;

        let phash = if media_type_str == "image" {
            let h = crate::multimodal::hashing::compute_perceptual_hash(&file_bytes);
            Some(crate::multimodal::hashing::format_phash(h))
        } else {
            None
        };

        Ok(json!({
            "memory_id": memory.id,
            "asset_id": asset_id,
            "media_type": media_type_str,
            "media_url": media_url,
            "file_hash": file_hash,
            "perceptual_hash": phash,
            "file_size": file_size,
            "mime_type": mime_type,
            "workspace": memory.workspace,
            "created_at": memory.created_at,
            "deduplicated": false
        }))
    });

    res.unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Maximum media file size read into memory (50 MiB).
#[cfg(feature = "multimodal")]
pub(crate) const MAX_MEDIA_FILE_BYTES: u64 = 50 * 1024 * 1024;

/// Read a media file with a strict upper bound on byte size to prevent OOM.
#[cfg(feature = "multimodal")]
fn read_media_file_bounded(
    path: &std::path::Path,
    max_bytes: u64,
) -> crate::error::Result<Vec<u8>> {
    use crate::error::EngramError;
    use std::io::Read;

    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > max_bytes {
        return Err(EngramError::InvalidInput(format!(
            "media file '{}' exceeds maximum allowed size of {} bytes",
            path.display(),
            max_bytes
        )));
    }
    let mut buf = Vec::new();
    file.take(max_bytes + 1).read_to_end(&mut buf)?;
    if buf.len() as u64 > max_bytes {
        return Err(EngramError::InvalidInput(format!(
            "media file '{}' exceeds maximum allowed size of {} bytes",
            path.display(),
            max_bytes
        )));
    }
    Ok(buf)
}

/// Validate a user-supplied media file path.
///
/// Rejects empty paths and null bytes. If `ENGRAM_MEDIA_BASE_DIR` is set,
/// canonicalizes the path and rejects any path that escapes the base directory.
#[cfg(feature = "multimodal")]
fn validate_media_path(path: &str) -> crate::error::Result<std::path::PathBuf> {
    use crate::error::EngramError;

    if path.is_empty() {
        return Err(EngramError::InvalidInput(
            "media path must not be empty".to_string(),
        ));
    }
    if path.contains('\0') {
        return Err(EngramError::InvalidInput(
            "media path must not contain null bytes".to_string(),
        ));
    }

    let canonical = std::fs::canonicalize(path)
        .map_err(|e| EngramError::InvalidInput(format!("media path is not accessible: {}", e)))?;

    if let Ok(base_str) = std::env::var("ENGRAM_MEDIA_BASE_DIR") {
        let base = std::fs::canonicalize(&base_str).map_err(|e| {
            EngramError::InvalidInput(format!("ENGRAM_MEDIA_BASE_DIR is not accessible: {}", e))
        })?;
        if !canonical.starts_with(&base) {
            return Err(EngramError::InvalidInput(format!(
                "media path '{}' is outside the allowed base directory",
                path
            )));
        }
    }

    Ok(canonical)
}

/// Validate a user-supplied media file path (non-multimodal fallback, always errors).
#[cfg(not(feature = "multimodal"))]
#[allow(dead_code)]
fn validate_media_path(_path: &str) -> crate::error::Result<std::path::PathBuf> {
    Err(crate::error::EngramError::InvalidInput(
        "multimodal feature not enabled".to_string(),
    ))
}

/// Infer MIME type from file extension.
#[cfg(feature = "multimodal")]
fn infer_mime_type(path: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/m4a",
        "flac" => "audio/flac",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[cfg(not(feature = "multimodal"))]
pub fn memory_ingest_media(_ctx: &HandlerContext, _params: Value) -> Value {
    json!({"error": "memory_ingest_media requires the `multimodal` feature"})
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(feature = "multimodal")]
#[path = "multimodal_tests.rs"]
mod tests;
