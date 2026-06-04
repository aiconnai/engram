// MCP tool definitions by domain.

    ToolDef {
        name: "memory_search_by_image",
        description: "Search memories using an image as the query. Uses multimodal embeddings (CLIP-style) or falls back to describing the image via vision model and searching by description. Returns semantically similar memories — text or media — ranked by relevance.",
        schema: r#"{
            "type": "object",
            "properties": {
                "image_path": {"type": "string", "description": "Path to the local image file to use as the search query"},
                "limit": {"type": "integer", "default": 10, "description": "Maximum number of results to return"},
                "min_score": {"type": "number", "minimum": 0, "maximum": 1, "description": "Minimum similarity score (0.0-1.0)"},
                "workspace": {"type": "string", "description": "Restrict search to a specific workspace"},
                "strategy": {"type": "string", "enum": ["clip", "description", "auto"], "default": "auto", "description": "Embedding strategy: clip (requires CLIP embedder), description (vision model + text embedding), auto (use CLIP if available, else description)"}
            },
            "required": ["image_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Image Handling
    ToolDef {
        name: "memory_upload_image",
        description: "Upload an image file and attach it to a memory. The image will be stored locally and linked to the memory's metadata.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "ID of the memory to attach the image to"},
                "file_path": {"type": "string", "description": "Path to the image file to upload"},
                "image_index": {"type": "integer", "default": 0, "description": "Index for ordering multiple images (0-based)"},
                "caption": {"type": "string", "description": "Optional caption for the image"}
            },
            "required": ["memory_id", "file_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_migrate_images",
        description: "Migrate existing base64-encoded images in memories to file storage. Scans all memories and uploads any embedded data URIs to storage, replacing them with file references.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": false, "description": "If true, only report what would be migrated without making changes"}
            }
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "multimodal")]
    ToolDef {
        name: "memory_capture_screenshot",
        description: "Capture a screenshot of the full screen or a specific application window and save it to a local file.",
        schema: r#"{
            "type": "object",
            "properties": {
                "app_name": {"type": "string", "description": "Name of the application window to capture; omit to capture the full screen"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
#[cfg(feature = "multimodal")]
    ToolDef {
        name: "memory_describe_image",
        description: "Describe the contents of an image file using the configured vision provider (requires VISION_PROVIDER env).",
        schema: r#"{
            "type": "object",
            "properties": {
                "image_path": {
                    "type": "string",
                    "description": "Absolute filesystem path to the image file (JPEG, PNG, WebP, etc.)."
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional custom prompt to guide the image description."
                }
            },
            "required": ["image_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "multimodal")]
    // 9. memory_list_media
    ToolDef {
        name: "memory_list_media",
        description: "List media assets stored in the media_assets table, optionally filtered by type (image, audio, video).",
        schema: r#"{
            "type": "object",
            "properties": {
                "media_type": {"type": "string", "description": "Filter by media type: \"image\", \"audio\", or \"video\". Omit for all types."},
                "limit": {"type": "integer", "description": "Maximum number of assets to return (default: 50)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    #[cfg(feature = "multimodal")]
    // 10. memory_process_video
    ToolDef {
        name: "memory_process_video",
        description: "Process a video file: extract metadata and keyframe descriptions via the configured vision provider, and create a memory record for the result.",
        schema: r#"{
            "type": "object",
            "properties": {
                "video_path": {"type": "string", "description": "Absolute path to the video file to process."}
            },
            "required": ["video_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "multimodal")]
    // ── multimodal.rs ────────────────────────────────────────────────────────
    ToolDef {
        name: "memory_transcribe_audio",
        description: "Transcribe an audio file to text using the configured audio transcription provider.",
        schema: r#"{
            "type": "object",
            "properties": {
                "audio_path": {
                    "type": "string",
                    "description": "Absolute or relative path to the audio file to transcribe."
                }
            },
            "required": ["audio_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
