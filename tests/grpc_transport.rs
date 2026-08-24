//! Integration tests for the gRPC transport.
//!
//! Starts an in-process `serve_grpc()` server on a random port, then exercises
//! all 7 test scenarios via a real tonic gRPC client.
//!
//! Run with:
//!   cargo test --test grpc_transport --features grpc -- --nocapture
//!
//! The tests share a single server spawned by `server_addr()`.

#![cfg(feature = "grpc")]

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
#[cfg(feature = "langfuse")]
use std::sync::OnceLock;

use parking_lot::Mutex;
use serde_json::{json, Value};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Code, Request};

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::grpc_transport::proto::mcp_service_client::McpServiceClient;
use engram::mcp::grpc_transport::proto::McpRequest as ProtoRequest;
use engram::mcp::grpc_transport::proto::SubscribeRequest;
use engram::mcp::grpc_transport::serve_grpc;
use engram::mcp::{
    get_tool_definitions, handlers, methods, InitializeResult, McpHandler, McpRequest, McpResponse,
    ServerCapabilities, ToolsCapability, MCP_PROTOCOL_VERSION,
};
use engram::realtime::RealtimeManager;
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

// ---------------------------------------------------------------------------
// Test handler
// ---------------------------------------------------------------------------

/// A complete `McpHandler` backed by in-memory storage, suitable for tests.
struct TestHandler {
    // Storage is kept alive here to prevent the in-memory DB from being dropped
    // while the handler is still in use.
    #[allow(dead_code)]
    storage: Storage,
    ctx: handlers::HandlerContext,
}

impl TestHandler {
    fn new() -> Self {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        let ctx = handlers::HandlerContext {
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
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: test_langfuse_runtime(),
            progress_reporter: None,
            principal: None,
        };
        Self { storage, ctx }
    }
}

#[cfg(feature = "langfuse")]
fn test_langfuse_runtime() -> Arc<tokio::runtime::Runtime> {
    static RUNTIME: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();
    Arc::clone(
        RUNTIME.get_or_init(|| Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime"))),
    )
}

impl McpHandler for TestHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            methods::INITIALIZE => {
                let result = InitializeResult {
                    protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability {
                            list_changed: false,
                        }),
                        resources: None,
                        prompts: None,
                    },
                    ..InitializeResult::default()
                };
                McpResponse::success(request.id, json!(result))
            }
            methods::LIST_TOOLS => {
                let tools = get_tool_definitions();
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
                let result = handlers::dispatch(&self.ctx, name, arguments);
                use engram::mcp::ToolCallResult;
                let tool_result = ToolCallResult::json(&result);
                McpResponse::success(request.id, json!(tool_result))
            }
            _ => McpResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Server fixture
// ---------------------------------------------------------------------------

/// Pick an ephemeral port by binding to port 0, then release it.
fn pick_free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    listener.local_addr().expect("local addr").port()
}

/// Spawn a gRPC server in the background and return its address.
///
/// The server runs for the lifetime of the test process.
async fn start_server(api_key: Option<String>) -> SocketAddr {
    start_server_with_realtime(api_key, None).await
}

async fn start_server_with_realtime(
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
) -> SocketAddr {
    for _ in 0..10 {
        let port = pick_free_port();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("valid addr");
        let handler: Arc<dyn McpHandler> = Arc::new(TestHandler::new());
        let api_key_clone = api_key.clone();
        let realtime_clone = realtime.clone();

        let server =
            tokio::spawn(
                async move { serve_grpc(handler, addr, api_key_clone, realtime_clone).await },
            );

        for _ in 0..20 {
            if server.is_finished() {
                break;
            }

            if tokio::net::TcpStream::connect(addr).await.is_ok() {
                return addr;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        }

        server.abort();
    }

    panic!("failed to start test grpc server after retries")
}

/// Connect a tonic client to `addr`.
async fn connect(addr: SocketAddr) -> McpServiceClient<Channel> {
    let endpoint = format!("http://{addr}");
    McpServiceClient::connect(endpoint)
        .await
        .expect("connect to test grpc server")
}

// ---------------------------------------------------------------------------
// Helper: build a plain ProtoRequest
// ---------------------------------------------------------------------------

fn req(id: &str, method: &str, params: Value) -> ProtoRequest {
    ProtoRequest {
        id: id.to_string(),
        method: method.to_string(),
        params_json: serde_json::to_string(&params).unwrap_or_default(),
    }
}

fn bearer_req(id: &str, method: &str, params: Value) -> Request<ProtoRequest> {
    let mut request = Request::new(req(id, method, params));
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_static("Bearer secret-token"),
    );
    request
}

// ---------------------------------------------------------------------------
// Scenario a: Call `initialize` — returns server info
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_a_initialize_returns_server_info() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let resp = client
        .call(Request::new(req(
            "1",
            methods::INITIALIZE,
            json!({"protocolVersion": MCP_PROTOCOL_VERSION}),
        )))
        .await
        .expect("call initialize")
        .into_inner();

    assert_eq!(resp.id, "1");

    let result_json = match resp.result.expect("result present") {
        engram::mcp::grpc_transport::proto::mcp_response::Result::ResultJson(j) => j,
        other => panic!("expected ResultJson, got {:?}", other),
    };

    let parsed: Value = serde_json::from_str(&result_json).expect("valid json");
    assert_eq!(
        parsed["protocolVersion"].as_str(),
        Some(MCP_PROTOCOL_VERSION),
        "server must echo the current protocol version"
    );
    assert!(
        parsed["capabilities"]["tools"].is_object(),
        "capabilities.tools must be present"
    );
}

// ---------------------------------------------------------------------------
// Scenario b: Call `tools/list` — returns tool list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_b_tools_list_returns_tools() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let resp = client
        .call(Request::new(req("2", methods::LIST_TOOLS, json!({}))))
        .await
        .expect("call tools/list")
        .into_inner();

    assert_eq!(resp.id, "2");

    let result_json = match resp.result.expect("result present") {
        engram::mcp::grpc_transport::proto::mcp_response::Result::ResultJson(j) => j,
        other => panic!("expected ResultJson, got {:?}", other),
    };

    let parsed: Value = serde_json::from_str(&result_json).expect("valid json");
    let tools = parsed["tools"].as_array().expect("tools must be array");
    assert!(
        !tools.is_empty(),
        "tool list must contain at least one tool"
    );
    // Verify at least memory_create is present
    let has_memory_create = tools
        .iter()
        .any(|t| t["name"].as_str() == Some("memory_create"));
    assert!(has_memory_create, "tools/list must include memory_create");
}

// ---------------------------------------------------------------------------
// Scenario c: Call `tools/call` + `memory_create` — creates a memory
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_c_memory_create_returns_id() {
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    let resp = client
        .call(bearer_req(
            "3",
            methods::CALL_TOOL,
            json!({
                "name": "memory_create",
                "arguments": {
                    "content": "gRPC integration test memory",
                    "memory_type": "note"
                }
            }),
        ))
        .await
        .expect("call memory_create")
        .into_inner();

    assert_eq!(resp.id, "3");

    let result_json = match resp.result.expect("result present") {
        engram::mcp::grpc_transport::proto::mcp_response::Result::ResultJson(j) => j,
        other => panic!("expected ResultJson, got {:?}", other),
    };

    let parsed: Value = serde_json::from_str(&result_json).expect("valid json");
    // ToolCallResult wraps output in content[0].text
    let text = parsed["content"][0]["text"]
        .as_str()
        .expect("content[0].text must be a string");
    let inner: Value = serde_json::from_str(text).expect("content text must be JSON");
    assert!(
        inner["id"].is_number() || inner["id"].is_string(),
        "memory_create must return an id"
    );
}

// ---------------------------------------------------------------------------
// Scenario d: Call `tools/call` + `memory_search` — returns results
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_d_memory_search_returns_results() {
    // Use a fresh server with a pre-created memory
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    // Create a memory first
    client
        .call(bearer_req(
            "seed",
            methods::CALL_TOOL,
            json!({
                "name": "memory_create",
                "arguments": {
                    "content": "searchable gRPC test content alpha",
                    "memory_type": "note"
                }
            }),
        ))
        .await
        .expect("seed memory");

    // Now search for it
    let resp = client
        .call(bearer_req(
            "4",
            methods::CALL_TOOL,
            json!({
                "name": "memory_search",
                "arguments": {
                    "query": "gRPC test content alpha"
                }
            }),
        ))
        .await
        .expect("call memory_search")
        .into_inner();

    assert_eq!(resp.id, "4");

    let result_json = match resp.result.expect("result present") {
        engram::mcp::grpc_transport::proto::mcp_response::Result::ResultJson(j) => j,
        other => panic!("expected ResultJson, got {:?}", other),
    };

    let parsed: Value = serde_json::from_str(&result_json).expect("valid json");
    let text = parsed["content"][0]["text"]
        .as_str()
        .expect("content[0].text must be a string");
    let inner: Value = serde_json::from_str(text).expect("content text must be JSON");
    // memory_search returns either:
    // - a top-level JSON array of match objects, OR
    // - an object with a `results` or `memories` key
    let results: &Vec<Value> = if let Some(arr) = inner.as_array() {
        arr
    } else if let Some(arr) = inner.get("results").and_then(|v| v.as_array()) {
        arr
    } else if let Some(arr) = inner.get("memories").and_then(|v| v.as_array()) {
        arr
    } else {
        panic!(
            "memory_search response must be an array or have results/memories key, got: {}",
            inner
        );
    };
    assert!(
        !results.is_empty(),
        "memory_search must return at least one result for a seeded memory"
    );
}

// ---------------------------------------------------------------------------
// Scenario e: Auth test — call without token when token required → UNAUTHENTICATED
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_e_missing_token_is_unauthenticated() {
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req("5", methods::LIST_TOOLS, json!({}))))
        .await
        .expect_err("call without token should fail");

    assert_eq!(
        err.code(),
        Code::Unauthenticated,
        "missing token must return UNAUTHENTICATED, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Scenario f: Auth test — call with correct token → succeeds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_f_correct_token_succeeds() {
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    let mut request = Request::new(req("6", methods::LIST_TOOLS, json!({})));
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_static("Bearer secret-token"),
    );

    let resp = client.call(request).await.expect("call with correct token");
    assert_eq!(resp.into_inner().id, "6");
}

#[tokio::test]
async fn scenario_h_malformed_token_is_unauthenticated() {
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    let mut request = Request::new(req("8", methods::LIST_TOOLS, json!({})));
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_static("Basic secret-token"),
    );

    let err = client
        .call(request)
        .await
        .expect_err("malformed token should fail");

    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn scenario_i_wrong_token_is_unauthenticated() {
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    let mut request = Request::new(req("9", methods::LIST_TOOLS, json!({})));
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_static("Bearer wrong-token"),
    );

    let err = client
        .call(request)
        .await
        .expect_err("wrong token should fail");

    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn scenario_j_loopback_no_key_allows_default_read_tool() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let resp = client
        .call(Request::new(req(
            "10",
            methods::CALL_TOOL,
            json!({
                "name": "memory_list",
                "arguments": {
                    "workspace": "default"
                }
            }),
        )))
        .await
        .expect("loopback anonymous read tool should succeed")
        .into_inner();

    assert_eq!(resp.id, "10");
}

#[tokio::test]
async fn scenario_j2_loopback_empty_key_uses_no_key_compatibility() {
    let addr = start_server(Some(String::new())).await;
    let mut client = connect(addr).await;

    let resp = client
        .call(Request::new(req(
            "10-empty",
            methods::CALL_TOOL,
            json!({
                "name": "memory_list",
                "arguments": {
                    "workspace": "default"
                }
            }),
        )))
        .await
        .expect("an empty configured key should behave as no key on loopback")
        .into_inner();

    assert_eq!(resp.id, "10-empty");
}

#[tokio::test]
async fn scenario_k_loopback_no_key_rejects_private_workspace() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "11",
            methods::CALL_TOOL,
            json!({
                "name": "memory_list",
                "arguments": {
                    "workspace": "private"
                }
            }),
        )))
        .await
        .expect_err("anonymous loopback must not read private workspaces");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_k2_loopback_no_key_rejects_nested_filter_workspace() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "11-filter",
            methods::CALL_TOOL,
            json!({
                "name": "memory_list",
                "arguments": {
                    "filter": {
                        "workspace": {"eq": "private"}
                    }
                }
            }),
        )))
        .await
        .expect_err("anonymous loopback must not bypass scope through an advanced filter");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_k3_loopback_no_key_rejects_implicit_all_workspace_search() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "11-implicit",
            methods::CALL_TOOL,
            json!({
                "name": "memory_search",
                "arguments": {"query": "private"}
            }),
        )))
        .await
        .expect_err("anonymous loopback must not search without an explicit workspace");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_k4_loopback_no_key_rejects_malformed_scoped_list() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "11-malformed",
            methods::CALL_TOOL,
            json!({
                "name": "memory_list",
                "arguments": {
                    "workspace": "default",
                    "sort_order": "bogus"
                }
            }),
        )))
        .await
        .expect_err("malformed arguments must not erase the authorized workspace");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_k5_loopback_no_key_rejects_resource_read() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "11-resource",
            methods::READ_RESOURCE,
            json!({"uri": "engram://memory/1"}),
        )))
        .await
        .expect_err("anonymous loopback must not bypass workspace scope through resources/read");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_k6_loopback_no_key_rejects_call_without_tool_name() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "11-missing-name",
            methods::CALL_TOOL,
            json!({"arguments": {"workspace": "default"}}),
        )))
        .await
        .expect_err("anonymous tools/call without a string name must fail closed");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_l_loopback_no_key_rejects_private_workspaces_array() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "12",
            methods::CALL_TOOL,
            json!({
                "name": "memory_search",
                "arguments": {
                    "query": "anything",
                    "workspaces": ["private"]
                }
            }),
        )))
        .await
        .expect_err("anonymous loopback must not bypass scope with workspaces[]");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_m_loopback_no_key_rejects_global_workspace() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "13",
            methods::CALL_TOOL,
            json!({
                "name": "memory_search",
                "arguments": {
                    "query": "anything",
                    "global": true
                }
            }),
        )))
        .await
        .expect_err("anonymous loopback must not search all workspaces");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_n_loopback_no_key_rejects_write_scope() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .call(Request::new(req(
            "14",
            methods::CALL_TOOL,
            json!({
                "name": "memory_create",
                "arguments": {
                    "content": "anonymous write should not dispatch",
                    "memory_type": "note",
                    "workspace": "default"
                }
            }),
        )))
        .await
        .expect_err("anonymous loopback must not write");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_o_stream_rejects_private_workspace_before_realtime() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .subscribe(Request::new(SubscribeRequest {
            event_types: Vec::new(),
            workspace: "private".to_string(),
        }))
        .await
        .expect_err("stream auth must reject private workspace before realtime lookup");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_p_anonymous_stream_rejects_unfiltered_workspace_before_realtime() {
    let addr = start_server(None).await;
    let mut client = connect(addr).await;

    let err = client
        .subscribe(Request::new(SubscribeRequest {
            event_types: Vec::new(),
            workspace: String::new(),
        }))
        .await
        .expect_err("anonymous stream must not bypass workspace scope with an empty filter");

    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn scenario_p2_authenticated_stream_accepts_default_workspace() {
    let addr = start_server_with_realtime(
        Some("secret-token".to_string()),
        Some(RealtimeManager::new()),
    )
    .await;
    let mut client = connect(addr).await;
    let mut request = Request::new(SubscribeRequest {
        event_types: Vec::new(),
        workspace: "default".to_string(),
    });
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_static("Bearer secret-token"),
    );

    client
        .subscribe(request)
        .await
        .expect("authenticated default-workspace stream should be accepted");
}

#[tokio::test]
async fn scenario_q_public_bind_without_key_refuses_before_accept() {
    let port = pick_free_port();
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse().expect("valid addr");
    let handler: Arc<dyn McpHandler> = Arc::new(TestHandler::new());

    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(200),
        serve_grpc(handler, addr, None, None),
    )
    .await;

    let err = match result {
        Ok(Err(err)) => err,
        Ok(Ok(())) => panic!("public unauthenticated gRPC bind unexpectedly exited OK"),
        Err(_) => panic!("public unauthenticated gRPC bind did not fail before accept"),
    };

    assert!(
        err.to_string().contains("non-loopback"),
        "error should explain non-loopback auth requirement, got: {err}"
    );

    assert!(
        tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_err(),
        "server must not accept connections after refusing startup"
    );
}

#[derive(Clone)]
struct CountingHandler {
    calls: Arc<AtomicUsize>,
}

impl McpHandler for CountingHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        self.calls.fetch_add(1, Ordering::SeqCst);
        McpResponse::success(request.id, json!({"reached_handler": true}))
    }
}

#[tokio::test]
async fn scenario_r_invalid_scope_never_reaches_dispatch() {
    let port = pick_free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("valid addr");
    let calls = Arc::new(AtomicUsize::new(0));
    let handler: Arc<dyn McpHandler> = Arc::new(CountingHandler {
        calls: Arc::clone(&calls),
    });

    let server = tokio::spawn(async move { serve_grpc(handler, addr, None, None).await });
    for _ in 0..20 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
    }

    let mut client = connect(addr).await;
    let err = client
        .call(Request::new(req(
            "15",
            methods::CALL_TOOL,
            json!({
                "name": "memory_create",
                "arguments": {
                    "content": "must not dispatch",
                    "memory_type": "note",
                    "workspace": "default"
                }
            }),
        )))
        .await
        .expect_err("invalid scope should fail before handler");

    assert_eq!(err.code(), Code::PermissionDenied);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "handler must not be called for invalid scope"
    );

    server.abort();
}

// ---------------------------------------------------------------------------
// Scenario g: Unknown method → error response (not a transport error)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_g_unknown_method_returns_error_response() {
    let addr = start_server(Some("secret-token".to_string())).await;
    let mut client = connect(addr).await;

    let resp = client
        .call(bearer_req("7", "unknown/method/xyz", json!({})))
        .await
        .expect("transport should succeed even for unknown method")
        .into_inner();

    assert_eq!(resp.id, "7");

    match resp.result.expect("result present") {
        engram::mcp::grpc_transport::proto::mcp_response::Result::Error(err) => {
            assert_eq!(err.code, -32601, "unknown method should return -32601");
            assert!(
                err.message.contains("Method not found"),
                "error message should mention Method not found, got: {}",
                err.message
            );
        }
        other => panic!("expected Error variant for unknown method, got {:?}", other),
    }
}
