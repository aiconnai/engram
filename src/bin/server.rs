//! Engram MCP Server
//!
//! Run with: engram-server

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use clap::Parser;
use parking_lot::Mutex;
use serde_json::{json, Value};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use engram::embedding::create_embedder;
use engram::error::Result;
use engram::mcp::{
    get_prompt, get_tool_definitions_tiered, handlers, http_transport, list_prompts,
    list_resources, methods, read_resource, InitializeResult, McpHandler, McpRequest, McpResponse,
    McpServer, PromptCapabilities, ResourceCapabilities, ServerCapabilities, ToolCallResult,
    ToolsCapability, MCP_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION_LEGACY,
};
use engram::realtime::{RealtimeManager, RealtimeServer};
use engram::search::{FuzzyEngine, SearchConfig};
use engram::storage::Storage;
#[cfg(feature = "meilisearch")]
use engram::storage::{MeilisearchBackend, MeilisearchIndexer, SqliteBackend};
use engram::types::*;

/// Transport mode for the MCP server.
#[derive(Debug, Clone, clap::ValueEnum)]
enum TransportMode {
    /// JSON-RPC over stdio (default, for MCP clients like Claude)
    Stdio,
    /// Streamable HTTP transport (JSON-RPC over HTTP)
    Http,
    /// Both stdio and HTTP transports simultaneously
    Both,
    /// gRPC transport only (requires the `grpc` feature)
    #[cfg(feature = "grpc")]
    Grpc,
}

#[derive(Parser, Debug)]
#[command(name = "engram-server")]
#[command(about = "Engram MCP server for AI memory")]
struct Args {
    /// Database path
    #[arg(
        long,
        env = "ENGRAM_DB_PATH",
        default_value = "~/.local/share/engram/memories.db"
    )]
    db_path: String,

    /// Storage mode (local or cloud-safe)
    #[arg(long, env = "ENGRAM_STORAGE_MODE", default_value = "local")]
    storage_mode: String,

    /// Cloud storage URI (s3://bucket/path)
    #[arg(long, env = "ENGRAM_STORAGE_URI")]
    cloud_uri: Option<String>,

    /// Enable cloud encryption
    #[arg(long, env = "ENGRAM_CLOUD_ENCRYPT")]
    encrypt: bool,

    /// Embedding model (tfidf, local, openai)
    #[arg(long, env = "ENGRAM_EMBEDDING_MODEL", default_value = "tfidf")]
    embedding_model: String,

    /// Local ONNX model directory (contains model.onnx and tokenizer.json)
    #[arg(long, env = "ENGRAM_ONNX_MODEL_DIR")]
    onnx_model_dir: Option<String>,

    /// OpenAI API key
    #[arg(long, env = "OPENAI_API_KEY")]
    openai_key: Option<String>,

    /// OpenAI-compatible API base URL (for OpenRouter, Azure, etc.)
    #[arg(
        long,
        env = "OPENAI_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    openai_base_url: String,

    /// Embedding model name (e.g., text-embedding-3-small, openai/text-embedding-3-small for OpenRouter)
    #[arg(
        long,
        env = "OPENAI_EMBEDDING_MODEL",
        default_value = "text-embedding-3-small"
    )]
    openai_embedding_model: String,

    /// Embedding dimensions (must match model output; 1536 for text-embedding-3-small)
    #[arg(long, env = "OPENAI_EMBEDDING_DIMENSIONS")]
    openai_embedding_dimensions: Option<usize>,

    /// Sync debounce in ms
    #[arg(long, env = "ENGRAM_SYNC_DEBOUNCE_MS", default_value = "5000")]
    sync_debounce_ms: u64,

    /// Confidence decay half-life in days
    #[arg(long, env = "ENGRAM_CONFIDENCE_HALF_LIFE", default_value = "30")]
    half_life_days: f32,

    /// Memory cleanup interval in seconds (0 = disabled)
    /// When enabled, expired memories are automatically cleaned up at this interval
    #[arg(long, env = "ENGRAM_CLEANUP_INTERVAL", default_value = "3600")]
    cleanup_interval_seconds: u64,

    /// Embedding drain interval in seconds (0 = disabled)
    /// When enabled, pending entries in the embedding_queue table are drained
    /// at this interval. Without this, memories accumulate without embeddings.
    #[arg(long, env = "ENGRAM_EMBEDDING_DRAIN_INTERVAL", default_value = "30")]
    embedding_drain_interval_seconds: u64,

    /// Max embeddings to compute per drain cycle (one batch call to the
    /// embedder). Larger values reduce API overhead but increase latency
    /// per cycle.
    #[arg(long, env = "ENGRAM_EMBEDDING_DRAIN_BATCH", default_value = "32")]
    embedding_drain_batch_size: usize,

    /// Compression scheduler interval in seconds (0 = disabled)
    /// Auto-summarizes old, rarely-accessed memories at this interval
    #[arg(long, env = "ENGRAM_COMPRESSION_INTERVAL", default_value = "0")]
    compression_interval_seconds: u64,

    /// Max age in days before a memory is eligible for auto-compression
    #[arg(long, env = "ENGRAM_COMPRESSION_MAX_AGE_DAYS", default_value = "90")]
    compression_max_age_days: i64,

    /// Max importance for auto-compression eligibility (0.0-1.0)
    #[arg(long, env = "ENGRAM_COMPRESSION_MAX_IMPORTANCE", default_value = "0.3")]
    compression_max_importance: f32,

    /// Min access count to skip auto-compression
    #[arg(long, env = "ENGRAM_COMPRESSION_MIN_ACCESS", default_value = "3")]
    compression_min_access: i32,

    /// WebSocket server port for real-time events (0 = disabled)
    #[arg(long, env = "ENGRAM_WS_PORT", default_value = "0")]
    ws_port: u16,

    /// WebSocket bind address for real-time events
    #[arg(long, env = "ENGRAM_WS_BIND_ADDRESS", default_value = "127.0.0.1")]
    ws_bind_address: IpAddr,

    /// Transport mode: stdio (default), http, or both
    #[arg(long, env = "ENGRAM_TRANSPORT", value_enum, default_value = "stdio")]
    transport: TransportMode,

    /// HTTP transport port (used when --transport is http or both)
    #[arg(long, env = "ENGRAM_HTTP_PORT", default_value = "3100")]
    http_port: u16,

    /// HTTP transport bind address
    #[arg(long, env = "ENGRAM_HTTP_BIND_ADDRESS", default_value = "127.0.0.1")]
    http_bind_address: IpAddr,

    /// API key for HTTP transport authentication (optional)
    #[arg(long, env = "ENGRAM_HTTP_API_KEY")]
    http_api_key: Option<String>,

    /// HTTP requests per second for MCP HTTP rate limiting
    #[arg(long, env = "ENGRAM_HTTP_RATE_LIMIT_RPS", default_value = "120")]
    http_rate_limit_rps: u64,

    /// HTTP burst size for MCP HTTP rate limiting
    #[arg(long, env = "ENGRAM_HTTP_RATE_LIMIT_BURST", default_value = "240")]
    http_rate_limit_burst: u64,

    /// HTTP rate-limit key source header (e.g., x-api-key, x-tenant-id). Empty disables header keying.
    #[arg(long, env = "ENGRAM_HTTP_RATE_LIMIT_KEY")]
    http_rate_limit_key: Option<String>,

    /// gRPC transport port (used when --transport is grpc)
    #[cfg(feature = "grpc")]
    #[arg(long, env = "ENGRAM_GRPC_PORT", default_value = "50051")]
    grpc_port: u16,

    /// gRPC transport bind address
    #[cfg(feature = "grpc")]
    #[arg(long, env = "ENGRAM_GRPC_BIND_ADDRESS", default_value = "127.0.0.1")]
    grpc_bind_address: IpAddr,

    /// API key for gRPC transport authentication (optional Bearer token)
    #[cfg(feature = "grpc")]
    #[arg(long, env = "ENGRAM_GRPC_API_KEY")]
    grpc_api_key: Option<String>,

    /// Meilisearch URL for optional search indexing
    #[cfg(feature = "meilisearch")]
    #[arg(long, env = "MEILISEARCH_URL")]
    meilisearch_url: Option<String>,

    /// Meilisearch API key (optional)
    #[cfg(feature = "meilisearch")]
    #[arg(long, env = "MEILISEARCH_API_KEY")]
    meilisearch_api_key: Option<String>,

    /// Enable Meilisearch indexer service
    #[cfg(feature = "meilisearch")]
    #[arg(long, env = "MEILISEARCH_INDEXER", default_value_t = false)]
    meilisearch_indexer: bool,

    /// Meilisearch sync interval in seconds
    #[cfg(feature = "meilisearch")]
    #[arg(long, env = "MEILISEARCH_SYNC_INTERVAL", default_value = "60")]
    meilisearch_sync_interval: u64,

    /// Dream Phase interval in seconds (0 = disabled)
    /// Periodic background consolidation of memories
    #[cfg(feature = "dream-phase")]
    #[arg(long, env = "ENGRAM_DREAM_INTERVAL", default_value = "0")]
    dream_interval_seconds: u64,
}

/// MCP request handler
struct EngramHandler {
    storage: Storage,
    embedder: Arc<dyn engram::embedding::Embedder>,
    fuzzy_engine: Arc<Mutex<FuzzyEngine>>,
    search_config: SearchConfig,
    /// Real-time event manager for WebSocket broadcasting
    realtime: Option<RealtimeManager>,
    /// Embedding cache for performance optimization
    embedding_cache: Arc<engram::embedding::EmbeddingCache>,
    /// Search result cache (Phase 4 - ENG-36)
    search_cache: Arc<engram::search::SearchResultCache>,
    /// Meilisearch backend for Phase 7 MCP tools
    #[cfg(feature = "meilisearch")]
    meili: Option<Arc<engram::storage::MeilisearchBackend>>,
    /// Meilisearch indexer for reindex operations
    #[cfg(feature = "meilisearch")]
    meili_indexer: Option<Arc<MeilisearchIndexer>>,
    /// Meilisearch sync interval config
    #[cfg(feature = "meilisearch")]
    meili_sync_interval: u64,
    /// Dedicated Tokio runtime for async operations (Langfuse sync).
    /// Kept alive here so the runtime outlives the handler; wiring to
    /// actual Langfuse calls lands in a follow-up task.
    #[cfg(feature = "langfuse")]
    #[allow(dead_code)]
    langfuse_runtime: Arc<tokio::runtime::Runtime>,
    /// Lifecycle hooks (Phase L - ENG-78). None unless `enable_hooks()` is called.
    #[cfg(feature = "hooks")]
    hook_manager: Option<Arc<engram::hooks::HookManager>>,
}

impl EngramHandler {
    fn new(storage: Storage, embedder: Arc<dyn engram::embedding::Embedder>) -> Self {
        Self {
            storage,
            embedder,
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(engram::search::SearchResultCache::new(
                engram::search::AdaptiveCacheConfig::default(),
            )),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("Failed to create Langfuse runtime"),
            ),
            #[cfg(feature = "hooks")]
            hook_manager: None,
        }
    }

    // Preserved for the WebSocket feature that wires realtime events to the handler;
    // not yet called from `main` while WebSocket transport is gated behind ws_port > 0
    // and uses a separate path.
    #[allow(dead_code)]
    fn with_realtime(mut self, manager: RealtimeManager) -> Self {
        self.realtime = Some(manager);
        self
    }

    /// Enable lifecycle hooks (Phase L - ENG-78 / issue #11).
    ///
    /// Registers the default `SessionStart` / `PostToolUse` / `Stop` /
    /// `SessionEnd` handlers. After this is called, every successful
    /// `tools/call` dispatch will fire `PostToolUse`.
    #[cfg(feature = "hooks")]
    fn enable_hooks(&mut self) {
        use engram::hooks::{
            HookManager, HookResult, LifecycleHook, PostToolUseHandler, SessionEndHandler,
            StopHandler,
        };

        let mut hm = HookManager::new();
        let storage = self.storage.clone();

        hm.register(LifecycleHook::SessionStart, |_hook, ctx| {
            tracing::info!(
                target = "engram::hooks",
                session_id = ?ctx.session_id,
                workspace = ?ctx.workspace,
                "SessionStart"
            );
            Ok(HookResult::Continue)
        });

        let post_tool_use_handler = PostToolUseHandler::new(storage.clone());
        hm.register(LifecycleHook::PostToolUse, move |hook, ctx| {
            post_tool_use_handler.handle(hook, ctx)
        });

        let stop_handler = StopHandler;
        hm.register(LifecycleHook::Stop, move |hook, ctx| {
            stop_handler.handle(hook, ctx)
        });
        let session_end_handler = SessionEndHandler::policy_summary_only(storage);
        hm.register(LifecycleHook::SessionEnd, move |hook, ctx| {
            session_end_handler.handle(hook, ctx)
        });

        self.hook_manager = Some(Arc::new(hm));
        tracing::info!("Lifecycle hooks enabled");
    }

    /// Fire a hook if hooks are enabled; no-op otherwise.
    #[cfg(feature = "hooks")]
    fn trigger_hook(&self, hook: engram::hooks::LifecycleHook, ctx: engram::hooks::HookContext) {
        if let Some(ref hm) = self.hook_manager {
            if let Err(e) = hm.trigger(hook, &ctx) {
                tracing::warn!(target = "engram::hooks", error = %e, "hook dispatch failed");
            }
        }
    }

    /// Build a `HandlerContext` from this handler's shared state and delegate
    /// to the domain-module dispatch function.
    fn handle_tool_call(&self, name: &str, params: Value) -> Value {
        let ctx = self.make_context();
        handlers::dispatch(&ctx, name, params)
    }

    /// Construct a `HandlerContext` from this handler's shared state.
    fn make_context(&self) -> handlers::HandlerContext {
        handlers::HandlerContext {
            storage: self.storage.clone(),
            embedder: self.embedder.clone(),
            fuzzy_engine: self.fuzzy_engine.clone(),
            search_config: self.search_config.clone(),
            realtime: self.realtime.clone(),
            embedding_cache: self.embedding_cache.clone(),
            search_cache: self.search_cache.clone(),
            #[cfg(feature = "meilisearch")]
            meili: self.meili.clone(),
            #[cfg(feature = "meilisearch")]
            meili_indexer: self.meili_indexer.clone(),
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: self.meili_sync_interval,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: self.langfuse_runtime.clone(),
        }
    }
}

impl McpHandler for EngramHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            methods::INITIALIZE => {
                // Negotiate protocol version: if the client requests the legacy version, respond
                // with that version and omit resources/prompts from capabilities.
                let client_version = request
                    .params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or(MCP_PROTOCOL_VERSION);

                let result = if client_version == MCP_PROTOCOL_VERSION_LEGACY {
                    // Legacy mode: respond with 2024-11-05, no resources/prompts capabilities
                    InitializeResult {
                        protocol_version: MCP_PROTOCOL_VERSION_LEGACY.to_string(),
                        capabilities: ServerCapabilities {
                            tools: Some(ToolsCapability {
                                list_changed: false,
                            }),
                            resources: None,
                            prompts: None,
                        },
                        ..InitializeResult::default()
                    }
                } else {
                    // Current mode: 2025-11-25 with full capabilities
                    InitializeResult {
                        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                        capabilities: ServerCapabilities {
                            tools: Some(ToolsCapability {
                                list_changed: false,
                            }),
                            resources: Some(ResourceCapabilities {
                                subscribe: false,
                                list_changed: false,
                            }),
                            prompts: Some(PromptCapabilities {
                                list_changed: false,
                            }),
                        },
                        ..InitializeResult::default()
                    }
                };

                McpResponse::success(request.id, json!(result))
            }
            methods::INITIALIZED => {
                // Notification — MCP spec says no response should be sent.
                // Return a response with id=None so the server loop can skip it.
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: None,
                }
            }
            methods::LIST_TOOLS => {
                let tier = std::env::var("ENGRAM_TOOL_TIER").ok();
                let tools = get_tool_definitions_tiered(tier.as_deref());
                McpResponse::success(request.id, json!({"tools": tools}))
            }
            methods::CALL_TOOL => {
                let name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));

                let result = self.handle_tool_call(name, arguments);

                #[cfg(feature = "hooks")]
                {
                    let mut ctx = engram::hooks::HookContext::new(None, None);
                    ctx.metadata.insert("tool_name".to_string(), json!(name));
                    ctx.metadata
                        .insert("tool_output".to_string(), result.clone());
                    self.trigger_hook(engram::hooks::LifecycleHook::PostToolUse, ctx);
                }

                let tool_result = ToolCallResult::json(&result);
                McpResponse::success(request.id, json!(tool_result))
            }
            methods::LIST_RESOURCES => {
                let templates = list_resources();
                let resources: Vec<Value> = templates
                    .into_iter()
                    .map(|t| {
                        json!({
                            "uri": t.uri_template,
                            "name": t.name,
                            "description": t.description,
                            "mimeType": t.mime_type,
                        })
                    })
                    .collect();
                McpResponse::success(request.id, json!({"resources": resources}))
            }
            methods::READ_RESOURCE => {
                let uri = match request.params.get("uri").and_then(|v| v.as_str()) {
                    Some(u) => u.to_string(),
                    None => {
                        return McpResponse::error(
                            request.id,
                            -32602,
                            "Missing required parameter: uri".to_string(),
                        )
                    }
                };

                match read_resource(&self.storage, &uri) {
                    Ok(content) => {
                        let text = serde_json::to_string_pretty(&content)
                            .unwrap_or_else(|_| content.to_string());
                        McpResponse::success(
                            request.id,
                            json!({
                                "contents": [{
                                    "uri": uri,
                                    "mimeType": "application/json",
                                    "text": text,
                                }]
                            }),
                        )
                    }
                    Err(msg) => McpResponse::error(request.id, -32602, msg),
                }
            }
            methods::LIST_PROMPTS => {
                let prompts = list_prompts();
                McpResponse::success(request.id, json!({"prompts": prompts}))
            }
            methods::GET_PROMPT => {
                let name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));
                match get_prompt(name, &arguments) {
                    Ok(messages) => McpResponse::success(request.id, json!({"messages": messages})),
                    Err(e) => McpResponse::error(request.id, -32002, e),
                }
            }
            _ => McpResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is for MCP protocol)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Expand ~ in path
    let db_path = shellexpand::tilde(&args.db_path).to_string();

    // Determine storage mode
    let storage_mode = match args.storage_mode.as_str() {
        "cloud-safe" => StorageMode::CloudSafe,
        _ => StorageMode::Local,
    };

    let config = StorageConfig {
        db_path,
        storage_mode,
        cloud_uri: args.cloud_uri,
        encrypt_cloud: args.encrypt,
        confidence_half_life_days: args.half_life_days,
        auto_sync: true,
        sync_debounce_ms: args.sync_debounce_ms,
    };

    // Open storage
    let storage = Storage::open(config.clone())?;

    // Singleton lock: only one mutating worker/server may own a storage path
    // (#24). Held for the lifetime of `main`; released automatically on exit.
    // A second server on the same path exits here with a clear error.
    let _storage_lock = engram::storage::StorageLock::acquire(&config.db_path, "engram-server")?;

    // Check for storage mode warning
    if let Some(warning) = storage.storage_mode_warning() {
        tracing::warn!("{}", warning);
    }

    #[cfg(feature = "meilisearch")]
    let mut _meili_backend_for_handler: Option<Arc<MeilisearchBackend>> = None;
    #[cfg(feature = "meilisearch")]
    let mut _meili_indexer_for_handler: Option<Arc<MeilisearchIndexer>> = None;
    #[cfg(feature = "meilisearch")]
    let _meili_sync_interval = args.meilisearch_sync_interval;

    #[cfg(feature = "meilisearch")]
    {
        if let Some(url) = args.meilisearch_url.as_deref() {
            let meili = Arc::new(MeilisearchBackend::new(
                url,
                args.meilisearch_api_key.as_deref(),
            )?);
            _meili_backend_for_handler = Some(meili.clone());

            if args.meilisearch_indexer {
                let sqlite_backend = SqliteBackend::new(config.clone())?;
                let indexer = Arc::new(MeilisearchIndexer::new(
                    Arc::new(sqlite_backend),
                    meili.clone(),
                    args.meilisearch_sync_interval,
                ));
                _meili_indexer_for_handler = Some(indexer.clone());

                let indexer_bg = indexer.clone();
                std::thread::spawn(move || {
                    let rt =
                        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(indexer_bg.start());
                });
            } else {
                tracing::info!(
                    "Meilisearch URL provided but indexer disabled. Set --meilisearch-indexer to enable."
                );
            }
        }
    }

    // Create embedder
    // Determine dimensions: use explicit config, or default based on model
    let dimensions = args.openai_embedding_dimensions.unwrap_or_else(|| {
        if args.embedding_model == "openai" {
            1536 // Default for text-embedding-3-small
        } else {
            384 // Default for TF-IDF and local MiniLM
        }
    });

    let embedding_config = EmbeddingConfig {
        model: args.embedding_model,
        api_key: args.openai_key,
        base_url: if args.openai_base_url == "https://api.openai.com/v1" {
            None // Use default
        } else {
            Some(args.openai_base_url)
        },
        embedding_model: Some(args.openai_embedding_model),
        model_path: args.onnx_model_dir,
        dimensions,
        batch_size: 100,
    };
    let embedder = create_embedder(&embedding_config)?;

    // Create real-time manager.
    // Always created so both the WebSocket server (when ws_port > 0) and
    // the HTTP SSE endpoint (GET /v1/events) can share the same broadcast channel.
    let realtime_manager = Some(RealtimeManager::new());

    // Create MCP request handler.
    //
    // Note: a parallel `engram::app_state::AppState` type exists in the lib
    // and is intended to eventually replace `EngramHandler` once the lifecycle
    // hook wiring lands (see issue #11). For now we still construct
    // `EngramHandler` directly since that's the type that implements
    // `McpHandler`.
    #[allow(unused_mut)]
    let mut handler_state = EngramHandler::new(storage.clone(), embedder.clone());
    #[cfg(feature = "hooks")]
    handler_state.enable_hooks();
    let handler = Arc::new(handler_state);
    let server = McpServer::new(handler.clone());

    // Start background cleanup thread if enabled
    if args.cleanup_interval_seconds > 0 {
        let cleanup_storage = storage.clone();
        let interval = std::time::Duration::from_secs(args.cleanup_interval_seconds);

        std::thread::spawn(move || {
            tracing::info!(
                "Memory cleanup thread started (interval: {}s)",
                interval.as_secs()
            );

            loop {
                std::thread::sleep(interval);

                match cleanup_storage.with_transaction(|conn| {
                    engram::storage::queries::cleanup_expired_memories(conn)
                }) {
                    Ok(deleted) => {
                        if deleted > 0 {
                            tracing::info!("Cleaned up {} expired memories", deleted);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error cleaning up expired memories: {}", e);
                    }
                }
            }
        });
    }

    // Start background embedding drain thread if enabled. This drains the
    // SQL `embedding_queue` table — without it, memories accumulate with
    // status='pending' forever and never get embeddings (issue #10).
    if args.embedding_drain_interval_seconds > 0 {
        let drain_storage = storage.clone();
        let drain_embedder = embedder.clone();
        let interval = std::time::Duration::from_secs(args.embedding_drain_interval_seconds);
        let batch_size = args.embedding_drain_batch_size;

        std::thread::spawn(move || {
            tracing::info!(
                "Embedding drain thread started (interval: {}s, batch: {})",
                interval.as_secs(),
                batch_size,
            );

            loop {
                std::thread::sleep(interval);

                // Drain in a loop until the queue is empty (or we hit one
                // batch that returns 0). This catches up faster after a
                // backlog without waiting `interval` between batches.
                // drain_pending_embeddings owns its lock discipline — it
                // releases the connection lock around the embed_batch call
                // so other DB ops aren't blocked by the network round-trip.
                loop {
                    let result = engram::embedding::drain_pending_embeddings(
                        &drain_storage,
                        drain_embedder.as_ref(),
                        batch_size,
                    );

                    match result {
                        Ok(0) => break,
                        Ok(n) => {
                            tracing::info!("Embedding drain processed {} memories", n);
                            if n < batch_size {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Embedding drain error: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }

    // Start background compression scheduler if enabled
    if args.compression_interval_seconds > 0 {
        let compression_storage = storage.clone();
        let interval = std::time::Duration::from_secs(args.compression_interval_seconds);
        let max_age = args.compression_max_age_days;
        let max_imp = args.compression_max_importance;
        let min_acc = args.compression_min_access;

        std::thread::spawn(move || {
            tracing::info!(
                "Compression scheduler started (interval: {}s, max_age: {}d, max_importance: {}, min_access: {})",
                interval.as_secs(),
                max_age,
                max_imp,
                min_acc,
            );

            loop {
                std::thread::sleep(interval);

                match compression_storage.with_transaction(|conn| {
                    engram::storage::queries::compress_old_memories(
                        conn, max_age, max_imp, min_acc, 100, // batch limit per cycle
                    )
                }) {
                    Ok(compressed) => {
                        if compressed > 0 {
                            tracing::info!(
                                "Compression scheduler compressed {} archived memories",
                                compressed
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Compression scheduler error: {}", e);
                    }
                }
            }
        });
    }

    // Start Dream Phase scheduler if enabled
    #[cfg(feature = "dream-phase")]
    if args.dream_interval_seconds > 0 {
        let dream_storage = Arc::new(storage.clone());
        let dream_config = engram::dream::DreamConfig {
            interval: std::time::Duration::from_secs(args.dream_interval_seconds),
            ..Default::default()
        };

        // The scheduler runs in its own Tokio task
        engram::dream::spawn_scheduler(dream_storage, dream_config);
        tracing::info!(
            "Dream Phase scheduler started (interval: {}s)",
            args.dream_interval_seconds
        );
    }

    let ws_addr = SocketAddr::new(args.ws_bind_address, args.ws_port);

    // Start WebSocket server in background if ws_port > 0.
    // Clone the manager so it can also be shared with the HTTP transport SSE endpoint.
    if args.ws_port > 0 {
        if let Some(ref manager) = realtime_manager {
            let ws_manager = manager.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(async {
                    let ws_server = RealtimeServer::new(ws_manager, ws_addr);
                    tracing::info!("WebSocket server starting on {}...", ws_addr);
                    if let Err(e) = ws_server.start().await {
                        tracing::error!("WebSocket server error: {}", e);
                    }
                });
            });
        }
    }

    tracing::info!("Engram MCP server starting...");

    // Log RTK-inspired features
    tracing::info!("Engram with RTK-inspired features loaded:");
    tracing::info!("  - OutputFilter: Active");
    tracing::info!("  - ContextGrouper: Active");
    tracing::info!("  - TruncationEngine: Active");
    tracing::info!("  - IntegrationOrchestrator: Active");

    match args.transport {
        TransportMode::Stdio => {
            server.run()?;
        }
        TransportMode::Http => {
            let http_addr = SocketAddr::new(args.http_bind_address, args.http_port);
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))?;
            rt.block_on(async {
                http_transport::serve_http(
                    handler,
                    http_addr,
                    args.http_api_key,
                    realtime_manager,
                    args.http_rate_limit_rps,
                    args.http_rate_limit_burst,
                    args.http_rate_limit_key,
                )
                .await
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))
            })?;
        }
        TransportMode::Both => {
            let http_handler = handler.clone();
            let http_addr = SocketAddr::new(args.http_bind_address, args.http_port);
            let http_api_key = args.http_api_key.clone();
            let http_realtime = realtime_manager.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create HTTP transport runtime");
                rt.block_on(async {
                    if let Err(e) = http_transport::serve_http(
                        http_handler,
                        http_addr,
                        http_api_key,
                        http_realtime,
                        args.http_rate_limit_rps,
                        args.http_rate_limit_burst,
                        args.http_rate_limit_key,
                    )
                    .await
                    {
                        tracing::error!("HTTP transport error: {}", e);
                    }
                });
            });

            // Run stdio in the main thread
            server.run()?;
        }
        #[cfg(feature = "grpc")]
        TransportMode::Grpc => {
            use engram::mcp::grpc_transport;

            let grpc_addr = SocketAddr::new(args.grpc_bind_address, args.grpc_port);
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))?;
            rt.block_on(async {
                grpc_transport::serve_grpc(handler, grpc_addr, args.grpc_api_key, realtime_manager)
                    .await
                    .map_err(|e| engram::error::EngramError::Internal(e.to_string()))
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handler() -> EngramHandler {
        let storage = Storage::open_in_memory().unwrap();
        let embedder = create_embedder(&EmbeddingConfig::default()).unwrap();
        EngramHandler {
            storage: storage.clone(),
            search_cache: Arc::new(engram::search::result_cache::SearchResultCache::new(
                Default::default(),
            )),
            embedder,
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("Failed to create Langfuse runtime"),
            ),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 300,
            #[cfg(feature = "hooks")]
            hook_manager: None,
        }
    }

    #[test]
    fn test_tool_ingest_document_idempotent() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        std::fs::write(&file_path, "# Title\n\nHello world.\n").unwrap();

        let handler = test_handler();

        let first = handler.handle_tool_call(
            "memory_ingest_document",
            json!({
                "path": file_path.to_string_lossy(),
                "format": "md"
            }),
        );
        assert!(first.get("error").is_none(), "first ingest error: {first}");
        assert!(
            first
                .get("chunks_created")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );

        let second = handler.handle_tool_call(
            "memory_ingest_document",
            json!({
                "path": file_path.to_string_lossy(),
                "format": "md"
            }),
        );
        assert!(
            second.get("error").is_none(),
            "second ingest error: {second}"
        );
        assert_eq!(
            second
                .get("chunks_created")
                .and_then(|v| v.as_u64())
                .unwrap_or(1),
            0
        );
    }

    /// Verifies that `enable_hooks()` populates the manager and that
    /// `trigger_hook(PostToolUse, …)` succeeds with hooks enabled and is a
    /// silent no-op when disabled. Indirectly validates the wiring used by
    /// the `CALL_TOOL` dispatch path.
    #[cfg(feature = "hooks")]
    #[test]
    fn test_hook_wiring() {
        use engram::hooks::{HookContext, HookResult, LifecycleHook};

        let mut handler = test_handler();
        assert!(
            handler.hook_manager.is_none(),
            "hooks should start disabled"
        );

        handler.enable_hooks();
        assert!(
            handler.hook_manager.is_some(),
            "enable_hooks should populate the manager"
        );

        let mut ctx = HookContext::new(Some("test-session".into()), Some("default".into()));
        ctx.metadata
            .insert("tool_name".into(), json!("memory_create"));
        handler.trigger_hook(LifecycleHook::PostToolUse, ctx);

        let stop_context = HookContext::new(Some("test-session".into()), Some("default".into()));
        let stop_results = handler
            .hook_manager
            .as_ref()
            .expect("hook manager should be enabled")
            .trigger(LifecycleHook::Stop, &stop_context)
            .expect("Stop hook should run without error");
        assert!(
            matches!(stop_results.as_slice(), [HookResult::Continue]),
            "Stop hook should be registered and continue, got {stop_results:?}"
        );
    }
}
