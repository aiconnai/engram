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
    list_resources, methods, read_resource_as, InitializeResult, McpHandler, McpRequest,
    McpResponse, McpServer, ServerCapabilities, ToolCallResult, ToolsCapability,
    MCP_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION_LEGACY,
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

    /// Enable the in-memory HNSW approximate-nearest-neighbour index for
    /// semantic search. Opt-in (RFC 0003): when off, semantic search uses the
    /// exact SQLite scan and no index is built, warmed up or maintained.
    #[arg(long, env = "ENGRAM_HNSW_ENABLED")]
    hnsw_enabled: bool,

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

    /// API key for WebSocket real-time events authentication (optional Bearer token)
    #[arg(long, env = "ENGRAM_WS_AUTH_KEY")]
    ws_auth_key: Option<String>,

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
    /// In-memory HNSW vector index (RFC 0005)
    hnsw_index: Arc<parking_lot::RwLock<engram::search::HnswIndex<i64>>>,
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
    /// Sender for progress notifications emitted during tool calls.
    /// The receiver is owned by the transport loop (stdio or HTTP).
    progress_tx: std::sync::mpsc::Sender<engram::mcp::ProgressNotification>,
    /// Active MCP resource subscriptions (MCP 2025-11-25).
    subscriptions: Arc<engram::mcp::ResourceSubscriptionManager>,
}

impl EngramHandler {
    fn new(
        storage: Storage,
        embedder: Arc<dyn engram::embedding::Embedder>,
        progress_tx: std::sync::mpsc::Sender<engram::mcp::ProgressNotification>,
        search_config: SearchConfig,
    ) -> Self {
        let hnsw_index = Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(
                embedder.dimensions(),
                engram::search::VectorMetric::Cosine,
            ),
        )));

        // HNSW is opt-in: skip the warmup (full DB replay) when it is disabled.
        if search_config.hnsw_enabled {
            let _ = storage.with_connection(|conn| {
                engram::search::warmup_hnsw_from_db(conn, &mut hnsw_index.write())
            });
        }

        Self {
            storage,
            embedder,
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config,
            realtime: None,
            embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(engram::search::SearchResultCache::new(
                engram::search::AdaptiveCacheConfig::default(),
            )),
            hnsw_index,
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
            progress_tx,
            subscriptions: Arc::new(engram::mcp::ResourceSubscriptionManager::new()),
        }
    }

    fn with_realtime(mut self, manager: RealtimeManager) -> Self {
        self.realtime = Some(manager);
        self
    }

    #[cfg(feature = "meilisearch")]
    fn with_meilisearch(
        mut self,
        meili: Option<Arc<MeilisearchBackend>>,
        meili_indexer: Option<Arc<MeilisearchIndexer>>,
        meili_sync_interval: u64,
    ) -> Self {
        self.meili = meili;
        self.meili_indexer = meili_indexer;
        self.meili_sync_interval = meili_sync_interval;
        self
    }

    /// Enable lifecycle hooks (Phase L - ENG-78 / issue #11).
    ///
    /// Registers the default `SessionStart` / `PostToolUse` / `Stop` /
    /// `SessionEnd` handlers. After this is called, `initialize` drains
    /// workspace-scoped pending injections via `SessionStart` and every
    /// successful `tools/call` dispatch fires `PostToolUse`.
    #[cfg(feature = "hooks")]
    fn enable_hooks(&mut self) {
        use engram::hooks::{
            HookManager, LifecycleHook, PostToolUseHandler, SessionEndHandler, SessionStartHandler,
            StopHandler,
        };

        let mut hm = HookManager::new();
        let storage = self.storage.clone();

        let session_start_handler = SessionStartHandler::new(storage.clone());
        hm.register(LifecycleHook::SessionStart, move |hook, ctx| {
            session_start_handler.handle(hook, ctx)
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

    /// Fire a hook and return its results if hooks are enabled; empty vec otherwise.
    #[cfg(feature = "hooks")]
    fn trigger_hook_results(
        &self,
        hook: engram::hooks::LifecycleHook,
        ctx: &engram::hooks::HookContext,
    ) -> Vec<engram::hooks::HookResult> {
        self.hook_manager
            .as_ref()
            .and_then(|hm| {
                hm.trigger(hook, ctx)
                    .map_err(|e| {
                        tracing::warn!(
                            target = "engram::hooks",
                            error = %engram::observability::redact::opaque(&e),
                            "hook dispatch failed"
                        );
                    })
                    .ok()
            })
            .unwrap_or_default()
    }

    /// Fire a hook if hooks are enabled; no-op otherwise.
    #[cfg(feature = "hooks")]
    fn trigger_hook(&self, hook: engram::hooks::LifecycleHook, ctx: engram::hooks::HookContext) {
        let _ = self.trigger_hook_results(hook, &ctx);
    }

    /// Build a `HandlerContext` from this handler's shared state and delegate
    /// to the domain-module dispatch function.
    fn handle_tool_call(
        &self,
        name: &str,
        params: Value,
        progress_reporter: Option<Arc<dyn engram::mcp::ProgressReporter>>,
        principal: Option<engram::auth::TransportPrincipal>,
    ) -> Value {
        let mut ctx = self.make_context().with_principal(principal);
        ctx.progress_reporter = progress_reporter;
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
            hnsw_index: self.hnsw_index.clone(),
            #[cfg(feature = "meilisearch")]
            meili: self.meili.clone(),
            #[cfg(feature = "meilisearch")]
            meili_indexer: self.meili_indexer.clone(),
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: self.meili_sync_interval,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: self.langfuse_runtime.clone(),
            progress_reporter: None,
            principal: None,
        }
    }
}

#[cfg(feature = "hooks")]
fn resolve_hook_workspace(
    params: &Value,
    principal: Option<&engram::auth::TransportPrincipal>,
) -> Option<String> {
    if let Some(p) = principal {
        if !p.has_permission(
            engram::auth::Permission::Write,
            engram::auth::ResourceType::Memory,
        ) {
            return None;
        }
        if let Some(ns) = p
            .auth_context()
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let norm = engram::types::normalize_workspace(ns).ok()?;
            return p.allows_workspace(Some(&norm)).then_some(norm);
        }
    }
    let candidate = match params.get("workspace") {
        None | Some(Value::Null) => "default".to_string(),
        Some(Value::String(ws)) if ws.trim().is_empty() => "default".to_string(),
        Some(Value::String(ws)) => engram::types::normalize_workspace(ws).ok()?,
        Some(_) => return None,
    };
    if let Some(p) = principal {
        if !p.allows_workspace(Some(&candidate)) {
            return None;
        }
    }
    Some(candidate)
}

#[cfg(feature = "hooks")]
fn resolve_hook_session_id(params: &Value) -> Option<String> {
    params
        .get("session_id")
        .or_else(|| params.get("sessionId"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
}

impl McpHandler for EngramHandler {
    /// stdio entry point: no transport principal, i.e. the local process owner.
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        self.handle_request_as(request, None)
    }

    fn handle_request_as(
        &self,
        request: McpRequest,
        principal: Option<engram::auth::TransportPrincipal>,
    ) -> McpResponse {
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
                    // Current mode: 2025-11-25 with full capabilities (including resource subscribe)
                    InitializeResult::default()
                };

                #[allow(unused_mut)]
                let mut resp_json = json!(result);
                #[cfg(feature = "hooks")]
                if let Some(workspace) = resolve_hook_workspace(&request.params, principal.as_ref())
                {
                    let session_id = resolve_hook_session_id(&request.params);
                    let ctx = engram::hooks::HookContext::new(session_id, Some(workspace));
                    for hook_res in
                        self.trigger_hook_results(engram::hooks::LifecycleHook::SessionStart, &ctx)
                    {
                        if let engram::hooks::HookResult::Modify(injection) = hook_res {
                            if let Some(obj) = resp_json.as_object_mut() {
                                obj.insert("pending_injections".to_string(), injection);
                            }
                        }
                    }
                }

                McpResponse::success(request.id, resp_json)
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

                // Extract progress token from _meta if present.
                let progress_token = engram::mcp::extract_progress_token(&request.params);
                let progress_reporter: Option<Arc<dyn engram::mcp::ProgressReporter>> =
                    progress_token.map(|token| {
                        Arc::new(engram::mcp::ChannelProgressReporter::from_sender(
                            token,
                            self.progress_tx.clone(),
                        )) as Arc<dyn engram::mcp::ProgressReporter>
                    });

                #[cfg(feature = "hooks")]
                let hook_workspace = resolve_hook_workspace(&arguments, principal.as_ref());
                #[cfg(feature = "hooks")]
                let hook_session_id = resolve_hook_session_id(&arguments);

                let result = self.handle_tool_call(name, arguments, progress_reporter, principal);

                #[cfg(feature = "hooks")]
                if !engram::mcp::ToolError::is_error_response(&result) {
                    if let Some(hook_workspace) = hook_workspace {
                        let mut ctx =
                            engram::hooks::HookContext::new(hook_session_id, Some(hook_workspace));
                        ctx.metadata.insert("tool_name".to_string(), json!(name));
                        ctx.metadata
                            .insert("tool_output".to_string(), result.clone());
                        self.trigger_hook(engram::hooks::LifecycleHook::PostToolUse, ctx);
                    }
                }

                let tool_result = ToolCallResult::from_tool_output(&result);
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

                match read_resource_as(&self.storage, &uri, principal.as_ref()) {
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
            methods::SUBSCRIBE_RESOURCE => {
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

                match self
                    .subscriptions
                    .subscribe_as(&self.storage, &uri, principal.as_ref())
                {
                    Ok(()) => McpResponse::success(request.id, json!({})),
                    Err(msg) => McpResponse::error(request.id, -32602, msg),
                }
            }
            methods::UNSUBSCRIBE_RESOURCE => {
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

                match self
                    .subscriptions
                    .unsubscribe_as(&self.storage, &uri, principal.as_ref())
                {
                    Ok(()) => McpResponse::success(request.id, json!({})),
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

/// Drain cycles between two HNSW-vs-SQLite reconciliations.
const HNSW_SYNC_EVERY_N_CYCLES: u32 = 10;

/// Search configuration for the server: defaults plus the explicit HNSW opt-in.
fn search_config_for(hnsw_enabled: bool) -> SearchConfig {
    SearchConfig {
        hnsw_enabled,
        ..SearchConfig::default()
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

    // Panic payloads can hold request content; keep them out of stderr logs.
    engram::observability::redact::install_redacting_panic_hook();

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
        // The warning embeds the database path; keep it out of the log stream.
        tracing::warn!(
            warning = %engram::observability::redact::opaque(&warning),
            "storage mode warning"
        );
    }

    #[cfg(feature = "meilisearch")]
    let mut meili_backend_for_handler: Option<Arc<MeilisearchBackend>> = None;
    #[cfg(feature = "meilisearch")]
    let mut meili_indexer_for_handler: Option<Arc<MeilisearchIndexer>> = None;
    #[cfg(feature = "meilisearch")]
    let meili_sync_interval = args.meilisearch_sync_interval;

    #[cfg(feature = "meilisearch")]
    {
        if let Some(url) = args.meilisearch_url.as_deref() {
            let meili = Arc::new(MeilisearchBackend::new(
                url,
                args.meilisearch_api_key.as_deref(),
            )?);
            meili_backend_for_handler = Some(meili.clone());

            if args.meilisearch_indexer {
                let sqlite_backend = SqliteBackend::new(config.clone())?;
                let indexer = Arc::new(MeilisearchIndexer::new(
                    Arc::new(sqlite_backend),
                    meili.clone(),
                    args.meilisearch_sync_interval,
                ));
                meili_indexer_for_handler = Some(indexer.clone());

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
    let (progress_tx, progress_rx) = std::sync::mpsc::channel();
    let mut handler_state = EngramHandler::new(
        storage.clone(),
        embedder.clone(),
        progress_tx,
        search_config_for(args.hnsw_enabled),
    );
    if let Some(ref manager) = realtime_manager {
        handler_state = handler_state.with_realtime(manager.clone());
    }
    #[cfg(feature = "meilisearch")]
    {
        handler_state = handler_state.with_meilisearch(
            meili_backend_for_handler,
            meili_indexer_for_handler,
            meili_sync_interval,
        );
    }
    #[cfg(feature = "hooks")]
    handler_state.enable_hooks();
    let handler = Arc::new(handler_state);
    let server = McpServer::new(handler.clone(), progress_rx);

    // Start background cleanup thread if enabled
    if args.cleanup_interval_seconds > 0 {
        let cleanup_storage = storage.clone();
        let cleanup_hnsw = handler.hnsw_index.clone();
        let cleanup_hnsw_enabled = handler.search_config.hnsw_enabled;
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
                            if cleanup_hnsw_enabled {
                                engram::search::hnsw_sync::sync_hnsw_logged(
                                    &cleanup_storage,
                                    &cleanup_hnsw,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %engram::observability::redact::redacted(&e),
                            "Error cleaning up expired memories"
                        );
                    }
                }
            }
        });
    }

    // Start background embedding drain thread if enabled. This drains the
    // SQL `embedding_queue` table — without it, memories accumulate with
    // status='pending' forever and never get embeddings (issue #10).
    //
    // Each cycle first recovers `processing` jobs whose lease expired (for
    // example after a crash/restart; see `run_embedding_drain_cycle`), then
    // drains until empty. The cycle runs once on start and then every interval.
    if args.embedding_drain_interval_seconds > 0 {
        let drain_storage = storage.clone();
        let drain_embedder = embedder.clone();
        let drain_hnsw = handler.hnsw_index.clone();
        let sync_hnsw = handler.hnsw_index.clone();
        let hnsw_enabled = handler.search_config.hnsw_enabled;
        let interval = std::time::Duration::from_secs(args.embedding_drain_interval_seconds);
        let batch_size = args.embedding_drain_batch_size;

        std::thread::spawn(move || {
            tracing::info!(
                "Embedding drain thread started (interval: {}s, batch: {})",
                interval.as_secs(),
                batch_size,
            );
            let hygiene = engram::embedding::EmbeddingQueueHygieneConfig::default();
            // Drained embeddings are mirrored into the in-memory vector index
            // only after their transaction committed.
            let index_persisted = move |memory_id: i64, embedding: &[f32]| {
                if hnsw_enabled {
                    drain_hnsw.write().insert(memory_id, embedding)
                }
            };

            let mut cycles_since_hnsw_sync: u32 = 0;
            loop {
                // run_embedding_drain_cycle owns its lock discipline — it
                // releases the connection lock around the embed_batch call
                // so other DB ops aren't blocked by the network round-trip.
                // Logs the outcome with the error class only (provider
                // messages can echo memory text); see `observability::redact`.
                engram::embedding::run_embedding_drain_cycle_logged(
                    &drain_storage,
                    drain_embedder.as_ref(),
                    batch_size,
                    &hygiene,
                    &index_persisted,
                );
                // Memories also leave the searchable set through paths that
                // never touch the index (expiry, supersede, merge, GC, content
                // rewrites): reconcile against SQLite and bound tombstones.
                // Search re-validates every candidate against SQLite, so this
                // only reclaims memory and beam width: run it every few cycles
                // rather than scanning the whole index twice a minute.
                if hnsw_enabled {
                    if cycles_since_hnsw_sync == 0 {
                        engram::search::hnsw_sync::sync_hnsw_logged(&drain_storage, &sync_hnsw);
                    }
                    cycles_since_hnsw_sync =
                        (cycles_since_hnsw_sync + 1) % HNSW_SYNC_EVERY_N_CYCLES;
                }
                std::thread::sleep(interval);
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
                        tracing::error!(
                            error = %engram::observability::redact::redacted(&e),
                            "Compression scheduler error"
                        );
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

    // Start background cloud sync worker if configured
    #[cfg(feature = "cloud")]
    if let Some(ref cloud_uri) = config.cloud_uri {
        if config.auto_sync && config.sync_debounce_ms > 0 && config.db_path != ":memory:" {
            let sync_config = config.clone();
            let cloud_uri = cloud_uri.clone();
            std::thread::spawn(move || {
                let pool = match engram::storage::StoragePool::new(sync_config.clone(), 1) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!(
                            error = %engram::observability::redact::redacted(&e),
                            "Failed to open storage pool for cloud sync worker"
                        );
                        return;
                    }
                };
                let conn = pool.get();
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::error!(
                            error = %engram::observability::redact::io_class(&e),
                            "Failed to create cloud sync runtime"
                        );
                        return;
                    }
                };
                rt.block_on(async move {
                    let db_path = std::path::PathBuf::from(&sync_config.db_path);
                    let interval =
                        std::time::Duration::from_millis(sync_config.sync_debounce_ms.max(1000));
                    let mut worker: Option<engram::sync::SyncWorker> = None;
                    match engram::sync::SyncWorker::start(
                        db_path.clone(),
                        cloud_uri.clone(),
                        sync_config.encrypt_cloud,
                        sync_config.sync_debounce_ms,
                        conn.clone(),
                    )
                    .await
                    {
                        Ok(w) => {
                            tracing::info!(
                                "Cloud sync worker started (interval: {}ms)",
                                interval.as_millis()
                            );
                            worker = Some(w);
                        }
                        Err(e) => {
                            tracing::error!(
                                error = %engram::observability::redact::redacted(&e),
                                "Failed to start cloud sync worker; will retry"
                            );
                        }
                    }
                    loop {
                        tokio::time::sleep(interval).await;
                        if worker.is_none() {
                            match engram::sync::SyncWorker::start(
                                db_path.clone(),
                                cloud_uri.clone(),
                                sync_config.encrypt_cloud,
                                sync_config.sync_debounce_ms,
                                conn.clone(),
                            )
                            .await
                            {
                                Ok(w) => {
                                    tracing::info!("Cloud sync worker recovered");
                                    worker = Some(w);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        error = %engram::observability::redact::redacted(&e),
                                        "Cloud sync worker restart failed; will retry"
                                    );
                                    continue;
                                }
                            }
                        }
                        if let Some(ref w) = worker {
                            if let Err(e) = w.sync(engram::sync::SyncDirection::Push, false).await {
                                tracing::error!(
                                    error = %engram::observability::redact::redacted(&e),
                                    "Cloud sync worker dispatch error; resetting worker"
                                );
                                worker = None;
                            }
                        }
                    }
                });
            });
        }
    }

    let ws_addr = SocketAddr::new(args.ws_bind_address, args.ws_port);

    // Start WebSocket server in background if ws_port > 0.
    // Clone the manager so it can also be shared with the HTTP transport SSE endpoint.
    if args.ws_port > 0 {
        if let Some(ref manager) = realtime_manager {
            let ws_manager = manager.clone();
            let ws_auth_key = args.ws_auth_key.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(async {
                    let ws_server =
                        RealtimeServer::new(ws_manager, ws_addr).with_auth_key(ws_auth_key);
                    tracing::info!("WebSocket server starting on {}...", ws_addr);
                    if let Err(e) = ws_server.start().await {
                        tracing::error!(
                            error = %engram::observability::redact::io_class(&e),
                            "WebSocket server error"
                        );
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

    let res = match args.transport {
        TransportMode::Stdio => server.run(),
        TransportMode::Http => {
            let http_addr = SocketAddr::new(args.http_bind_address, args.http_port);
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))?;
            rt.block_on(async {
                http_transport::serve_http(
                    handler.clone(),
                    http_addr,
                    args.http_api_key,
                    realtime_manager,
                    args.http_rate_limit_rps,
                    args.http_rate_limit_burst,
                    args.http_rate_limit_key,
                )
                .await
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))
            })
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
                        tracing::error!(
                            error = %engram::observability::redact::opaque(&e),
                            "HTTP transport error"
                        );
                    }
                });
            });

            // Run stdio in the main thread
            server.run()
        }
        #[cfg(feature = "grpc")]
        TransportMode::Grpc => {
            use engram::mcp::grpc_transport;

            let grpc_addr = SocketAddr::new(args.grpc_bind_address, args.grpc_port);
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))?;
            rt.block_on(async {
                grpc_transport::serve_grpc(
                    handler.clone(),
                    grpc_addr,
                    args.grpc_api_key,
                    realtime_manager,
                )
                .await
                .map_err(|e| engram::error::EngramError::Internal(e.to_string()))
            })
        }
    };

    // Save final HNSW checkpoint on shutdown if vectors exist (lock order: storage -> hnsw_index)
    let _ = storage.with_connection(|conn| {
        let hnsw = handler.hnsw_index.read();
        if !hnsw.is_empty() {
            engram::search::checkpoint_hnsw_to_db(conn, &hnsw, "default")?;
        }
        Ok(())
    });

    res
}

#[cfg(test)]
macro_rules! engram_server_only {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[cfg(test)]
pub(crate) use engram_server_only;

#[cfg(test)]
#[path = "server/tests.rs"]
mod tests;
