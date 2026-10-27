//! End-to-end workspace authorization over the real gRPC transport.
//!
//! A tonic client talks to an in-process `GrpcMcpService` whose interceptor
//! authenticates every call as a workspace-restricted principal (the shape a
//! namespaced token produces). The handler enforces the principal through the
//! real dispatcher, so these tests cover the transport pre-check AND the
//! dispatcher, against in-memory storage.
//!
//! Every denial asserts nothing was written; positive controls prove the
//! legitimate paths still work.

#![cfg(feature = "grpc")]
// `tonic::Status` is large by design; test helpers pass it through unchanged.
#![allow(clippy::result_large_err)]

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::{json, Value};
use tonic::transport::{Channel, Server};
use tonic::{Code, Request};

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::grpc_transport::proto::mcp_response::Result as ProtoResult;
use engram::mcp::grpc_transport::proto::mcp_service_client::McpServiceClient;
use engram::mcp::grpc_transport::proto::mcp_service_server::McpServiceServer;
use engram::mcp::grpc_transport::proto::McpRequest as ProtoRequest;
use engram::mcp::grpc_transport::GrpcMcpService;
use engram::mcp::handlers::{self, HandlerContext};
use engram::mcp::permission::tool_call_parts;
use engram::mcp::{methods, McpHandler, McpRequest, McpResponse};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

const OWN_WS: &str = "tenant-a";
const FOREIGN_WS: &str = "tenant-b";

#[cfg(feature = "langfuse")]
fn langfuse_runtime() -> Arc<tokio::runtime::Runtime> {
    static RUNTIME: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();
    Arc::clone(
        RUNTIME.get_or_init(|| Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime"))),
    )
}

fn handler_context(storage: Storage) -> HandlerContext {
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
        langfuse_runtime: langfuse_runtime(),
        progress_reporter: None,
        principal: None,
    }
}

/// Handler that runs `tools/call` through the real dispatcher as the
/// transport-authenticated principal (what `EngramHandler` does in the server).
struct DispatchingHandler {
    storage: Storage,
}

impl McpHandler for DispatchingHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        self.handle_request_as(request, None)
    }

    fn handle_request_as(
        &self,
        request: McpRequest,
        principal: Option<TransportPrincipal>,
    ) -> McpResponse {
        if request.method != methods::CALL_TOOL {
            return McpResponse::error(request.id, -32601, "Method not found".to_string());
        }
        let (name, arguments) = tool_call_parts(&request.params).expect("tools/call has a name");
        let ctx = handler_context(self.storage.clone()).with_principal(principal);
        let result = handlers::dispatch(&ctx, name, arguments.clone());
        McpResponse::success(request.id, result)
    }
}

fn restricted_writer(namespace: &str) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("grpc-e2e"),
        key_id: "key-grpc-e2e".to_string(),
        permissions: PermissionSet::standard_user(),
        namespace: Some(namespace.to_string()),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

struct Fixture {
    client: McpServiceClient<Channel>,
    storage: Storage,
}

async fn start(principal: TransportPrincipal) -> Fixture {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    for _ in 0..10 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind port 0");
        let addr: SocketAddr = listener.local_addr().expect("local addr");
        drop(listener);

        let handler: Arc<dyn McpHandler> = Arc::new(DispatchingHandler {
            storage: storage.clone(),
        });
        let service = GrpcMcpService::new(handler, Some("unused-process-key".to_string()), None);
        let principal = principal.clone();
        let service =
            McpServiceServer::with_interceptor(service, move |mut request: Request<()>| {
                request.extensions_mut().insert(principal.clone());
                Ok(request)
            });
        let server = tokio::spawn(Server::builder().add_service(service).serve(addr));

        for _ in 0..20 {
            if server.is_finished() {
                break;
            }
            if let Ok(client) = McpServiceClient::connect(format!("http://{addr}")).await {
                return Fixture { client, storage };
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        server.abort();
    }
    panic!("failed to start test grpc server after retries")
}

impl Fixture {
    async fn call(&mut self, tool: &str, arguments: Value) -> Result<Value, tonic::Status> {
        let params = json!({"name": tool, "arguments": arguments});
        let response = self
            .client
            .call(Request::new(ProtoRequest {
                id: "1".to_string(),
                method: methods::CALL_TOOL.to_string(),
                params_json: params.to_string(),
            }))
            .await?
            .into_inner();
        match response.result.expect("result present") {
            ProtoResult::ResultJson(json) => Ok(serde_json::from_str(&json).expect("valid json")),
            other => panic!("expected ResultJson, got {other:?}"),
        }
    }

    fn memories_in(&self, workspace: &str) -> i64 {
        self.storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM memories WHERE workspace = ?1",
                    [workspace],
                    |row| row.get(0),
                )?)
            })
            .expect("workspace count")
    }
}

fn assert_denied(result: Result<Value, tonic::Status>, what: &str) {
    match result {
        Err(status) => assert_eq!(status.code(), Code::PermissionDenied, "{what}: {status}"),
        Ok(body) => {
            // A dispatcher-level denial is a normal response carrying the error.
            assert_eq!(
                body["error"]["code"], "permission_denied",
                "{what} must be denied, got {body}"
            );
        }
    }
}

fn assert_allowed(result: Result<Value, tonic::Status>, what: &str) {
    let body = result.unwrap_or_else(|status| panic!("{what} denied: {status}"));
    assert!(
        body.get("error").is_none(),
        "{what} must be allowed, got {body}"
    );
}

#[tokio::test]
async fn foreign_workspace_is_denied_and_own_is_allowed_over_grpc() {
    let mut fx = start(restricted_writer(OWN_WS)).await;

    assert_denied(
        fx.call(
            "memory_create",
            json!({"content": "foreign", "workspace": FOREIGN_WS}),
        )
        .await,
        "foreign memory_create",
    );
    assert_denied(
        fx.call("memory_list", json!({"workspace": FOREIGN_WS}))
            .await,
        "foreign memory_list",
    );
    assert_eq!(fx.memories_in(FOREIGN_WS), 0);

    assert_allowed(
        fx.call(
            "memory_create",
            json!({"content": "own", "workspace": OWN_WS}),
        )
        .await,
        "own memory_create",
    );
    assert_eq!(fx.memories_in(OWN_WS), 1);
}

#[tokio::test]
async fn foreign_batch_item_is_denied_and_own_batch_is_allowed_over_grpc() {
    let mut fx = start(restricted_writer(OWN_WS)).await;

    // A foreign item hidden behind an own top-level workspace.
    assert_denied(
        fx.call(
            "memory_create_batch",
            json!({"workspace": OWN_WS, "memories": [
                {"content": "own item", "workspace": OWN_WS},
                {"content": "foreign item", "workspace": FOREIGN_WS}
            ]}),
        )
        .await,
        "batch with a foreign item",
    );
    assert_denied(
        fx.call(
            "memory_ingest_fact_batch",
            json!({"workspace": OWN_WS, "facts": [
                {"fact": "foreign fact", "workspace": FOREIGN_WS}
            ]}),
        )
        .await,
        "fact batch with a foreign item",
    );
    assert_eq!(fx.memories_in(FOREIGN_WS), 0);
    assert_eq!(fx.memories_in(OWN_WS), 0, "a denied batch writes nothing");

    assert_allowed(
        fx.call(
            "memory_create_batch",
            json!({"memories": [
                {"content": "own one", "workspace": OWN_WS},
                {"content": "own two", "workspace": OWN_WS}
            ]}),
        )
        .await,
        "batch with only own items",
    );
    assert_eq!(fx.memories_in(OWN_WS), 2);
    assert_eq!(fx.memories_in(FOREIGN_WS), 0);
}

#[tokio::test]
async fn envelope_keys_inside_arguments_cannot_smuggle_a_workspace_over_grpc() {
    let mut fx = start(restricted_writer(OWN_WS)).await;

    // The real (top-level) claim is foreign; an own-looking `arguments` key
    // inside the arguments must not cover for it.
    assert_denied(
        fx.call(
            "memory_create",
            json!({
                "content": "smuggled",
                "workspace": FOREIGN_WS,
                "name": "memory_create",
                "arguments": {"workspace": OWN_WS}
            }),
        )
        .await,
        "foreign claim covered by an own `arguments` decoy",
    );
    assert_eq!(fx.memories_in(FOREIGN_WS), 0);

    // A foreign-looking decoy inside the arguments is data, not a claim.
    assert_allowed(
        fx.call(
            "memory_create",
            json!({
                "content": "own with decoy",
                "workspace": OWN_WS,
                "name": "x",
                "arguments": {"workspace": FOREIGN_WS}
            }),
        )
        .await,
        "own claim with a foreign `arguments` decoy",
    );
    assert_eq!(fx.memories_in(OWN_WS), 1);
    assert_eq!(fx.memories_in(FOREIGN_WS), 0);
}
