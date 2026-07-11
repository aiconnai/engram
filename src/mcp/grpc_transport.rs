//! gRPC transport for the MCP server.
//!
//! Exposes the same `McpHandler` trait used by stdio and HTTP transports through
//! a tonic-based gRPC server, enabling strongly-typed, bidirectional-streaming
//! access to all 200+ Engram MCP tools.
//!
//! # Feature gate
//! This module is compiled only when the `grpc` feature is active.
//!
//! # Design
//! - [`GrpcMcpService`] bridges the generated tonic stubs to `McpHandler`.
//! - Params/results travel as JSON strings (`params_json`, `result_json`) so
//!   the protobuf schema remains stable as the tool catalogue grows.
//! - Auth is checked via the gRPC metadata `authorization` header
//!   (`Bearer <token>`), mirroring the HTTP transport.
//! - Unauthenticated operation is allowed only on loopback and is limited to
//!   the anonymous loopback principal's default read scope.
//! - Native TLS is not configured by this transport; terminate TLS at a trusted
//!   reverse proxy when exposing gRPC outside the host.
//! - Streaming events are sourced from `RealtimeManager::subscribe()` and
//!   pushed through a `tokio_stream::wrappers::BroadcastStream`.

use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{metadata::MetadataMap, transport::Server, Request, Response, Status};

use super::methods;
use super::permission::permission_denial_for_principal;
use super::protocol::{McpHandler, McpRequest, McpResponse};
use crate::auth::{TransportPrincipal, TransportPrincipalError};
use crate::realtime::{EventType, RealtimeManager};

// Include generated tonic stubs.
pub mod proto {
    tonic::include_proto!("engram.mcp");
}

use proto::mcp_service_server::{McpService, McpServiceServer};
use proto::{
    mcp_response, McpError as ProtoMcpError, McpEvent, McpRequest as ProtoRequest,
    McpResponse as ProtoResponse, SubscribeRequest,
};

// ---------------------------------------------------------------------------
// Service implementation
// ---------------------------------------------------------------------------

/// gRPC service that bridges tonic to the `McpHandler` trait.
pub struct GrpcMcpService {
    handler: Arc<dyn McpHandler>,
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
}

impl GrpcMcpService {
    /// Create a new service.
    pub fn new(
        handler: Arc<dyn McpHandler>,
        api_key: Option<String>,
        realtime: Option<RealtimeManager>,
    ) -> Self {
        Self {
            handler,
            api_key,
            realtime,
        }
    }
}

// ---------------------------------------------------------------------------
// Auth helper
// ---------------------------------------------------------------------------

#[allow(clippy::result_large_err)]
fn authenticate(
    metadata: &MetadataMap,
    expected: &Option<String>,
) -> Result<TransportPrincipal, Status> {
    let Some(ref key) = expected else {
        return Ok(TransportPrincipal::anonymous_loopback());
    };

    let authorization = metadata
        .get("authorization")
        .map(|value| {
            value
                .to_str()
                .map_err(|_| TransportPrincipalError::MalformedBearer)
        })
        .transpose()
        .map_err(auth_status)?;

    TransportPrincipal::from_process_bearer(authorization, key).map_err(auth_status)
}

fn auth_status(_: TransportPrincipalError) -> Status {
    Status::unauthenticated("Invalid or missing Bearer token")
}

#[allow(clippy::result_large_err)]
fn principal_from_request<T>(
    request: &Request<T>,
    expected: &Option<String>,
) -> Result<TransportPrincipal, Status> {
    match request.extensions().get::<TransportPrincipal>() {
        Some(principal) => Ok(principal.clone()),
        None if expected.is_some() => authenticate(request.metadata(), expected),
        None => Err(Status::unauthenticated("Missing authenticated principal")),
    }
}

#[allow(clippy::result_large_err)]
fn authorize_request(principal: &TransportPrincipal, request: &McpRequest) -> Result<(), Status> {
    let is_anonymous = matches!(principal, TransportPrincipal::AnonymousLoopback(_));
    if is_anonymous {
        match request.method.as_str() {
            methods::INITIALIZE | methods::LIST_TOOLS => return Ok(()),
            methods::CALL_TOOL => {}
            _ => return Err(Status::permission_denied("permission denied")),
        }
    }

    if request.method != methods::CALL_TOOL {
        return Ok(());
    }

    let tool_name = request.params.get("name").and_then(|v| v.as_str());
    let Some(tool_name) = tool_name else {
        return if is_anonymous {
            Err(Status::permission_denied("permission denied"))
        } else {
            Ok(())
        };
    };

    if is_anonymous && !is_anonymous_default_list(tool_name, &request.params) {
        return Err(Status::permission_denied("permission denied"));
    }

    let requested_workspaces = requested_workspaces(&request.params);
    let first_workspace = requested_workspaces.first().copied();
    if permission_denial_for_principal(tool_name, principal, first_workspace).is_some()
        || requested_workspaces
            .iter()
            .any(|workspace| !principal.allows_workspace(Some(workspace)))
        || (requests_all_workspaces(&request.params) && !allows_all_workspaces(principal))
    {
        return Err(Status::permission_denied("permission denied"));
    }

    Ok(())
}

fn requested_workspaces(params: &serde_json::Value) -> Vec<&str> {
    let mut workspaces = Vec::new();
    collect_requested_workspaces(params, &mut workspaces);
    workspaces
}

fn collect_requested_workspaces<'a>(value: &'a serde_json::Value, workspaces: &mut Vec<&'a str>) {
    let Some(object) = value.as_object() else {
        return;
    };

    for (key, child) in object {
        if matches!(key.as_str(), "workspace" | "workspaces") {
            collect_workspace_values(child, workspaces);
        } else {
            collect_requested_workspaces(child, workspaces);
        }
    }
}

fn collect_workspace_values<'a>(value: &'a serde_json::Value, workspaces: &mut Vec<&'a str>) {
    match value {
        serde_json::Value::String(workspace) => workspaces.push(workspace),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_workspace_values(item, workspaces);
            }
        }
        serde_json::Value::Object(object) => {
            for child in object.values() {
                collect_workspace_values(child, workspaces);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn is_anonymous_default_list(tool_name: &str, params: &serde_json::Value) -> bool {
    if tool_name != "memory_list" {
        return false;
    }

    let Some(params) = params.as_object() else {
        return false;
    };
    let Some(arguments) = params.get("arguments").and_then(|value| value.as_object()) else {
        return false;
    };

    params.len() == 2
        && arguments.len() == 1
        && arguments.get("workspace").and_then(|value| value.as_str()) == Some("default")
}

fn requests_all_workspaces(params: &serde_json::Value) -> bool {
    let Some(object) = params.as_object() else {
        return false;
    };

    object.iter().any(|(key, value)| {
        (key == "global" && value.as_bool() == Some(true)) || requests_all_workspaces(value)
    })
}

fn allows_all_workspaces(principal: &TransportPrincipal) -> bool {
    !matches!(principal, TransportPrincipal::AnonymousLoopback(_))
        && principal.allows_workspace(None)
}

#[allow(clippy::result_large_err)]
fn authorize_subscription(
    principal: &TransportPrincipal,
    workspace: Option<&str>,
) -> Result<(), Status> {
    if permission_denial_for_principal("memory_get", principal, workspace).is_some()
        || (workspace.is_none() && !allows_all_workspaces(principal))
    {
        return Err(Status::permission_denied("permission denied"));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

/// Convert a protobuf `McpRequest` to the protocol-layer `McpRequest`.
///
/// The `id` field is stored as a JSON string. An empty `id` represents a
/// JSON-RPC notification (no id).
fn proto_to_handler_request(req: ProtoRequest) -> McpRequest {
    let id = if req.id.is_empty() {
        None
    } else {
        Some(serde_json::Value::String(req.id))
    };

    let params = serde_json::from_str::<serde_json::Value>(&req.params_json)
        .unwrap_or(serde_json::Value::Null);

    McpRequest {
        jsonrpc: "2.0".to_string(),
        id,
        method: req.method,
        params,
    }
}

/// Convert a protocol-layer `McpResponse` to a protobuf `McpResponse`.
fn handler_to_proto_response(resp: McpResponse) -> ProtoResponse {
    let id = resp
        .id
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(result) = resp.result {
        let result_json = serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string());
        ProtoResponse {
            id,
            result: Some(mcp_response::Result::ResultJson(result_json)),
        }
    } else if let Some(err) = resp.error {
        let error = ProtoMcpError {
            code: err.code as i32,
            message: err.message,
            data_json: err
                .data
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default())
                .unwrap_or_default(),
        };
        ProtoResponse {
            id,
            result: Some(mcp_response::Result::Error(error)),
        }
    } else {
        // Notification response — empty result
        ProtoResponse { id, result: None }
    }
}

/// Parse event type string into `EventType`.
fn parse_event_type(s: &str) -> Option<EventType> {
    match s {
        "memory_created" => Some(EventType::MemoryCreated),
        "memory_updated" => Some(EventType::MemoryUpdated),
        "memory_deleted" => Some(EventType::MemoryDeleted),
        "crossref_created" => Some(EventType::CrossrefCreated),
        "crossref_deleted" => Some(EventType::CrossrefDeleted),
        "sync_started" => Some(EventType::SyncStarted),
        "sync_completed" => Some(EventType::SyncCompleted),
        "sync_failed" => Some(EventType::SyncFailed),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tonic service trait implementation
// ---------------------------------------------------------------------------

type EventStream = Pin<Box<dyn Stream<Item = Result<McpEvent, Status>> + Send>>;

#[tonic::async_trait]
impl McpService for GrpcMcpService {
    /// Handle a unary MCP call — mirrors JSON-RPC request/response semantics.
    async fn call(
        &self,
        request: Request<ProtoRequest>,
    ) -> Result<Response<ProtoResponse>, Status> {
        let principal = principal_from_request(&request, &self.api_key)?;

        let handler_req = proto_to_handler_request(request.into_inner());
        authorize_request(&principal, &handler_req)?;
        let handler_resp = self.handler.handle_request(handler_req);
        let proto_resp = handler_to_proto_response(handler_resp);
        Ok(Response::new(proto_resp))
    }

    type SubscribeStream = EventStream;

    /// Open a server-streaming subscription — events are filtered by
    /// `event_types` and `workspace`, then forwarded as `McpEvent` messages.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let principal = principal_from_request(&request, &self.api_key)?;

        let sub_req = request.into_inner();
        let workspace = if sub_req.workspace.is_empty() {
            None
        } else {
            Some(sub_req.workspace.as_str())
        };
        authorize_subscription(&principal, workspace)?;

        let realtime = self.realtime.as_ref().ok_or_else(|| {
            Status::unavailable("Real-time events are not enabled on this server")
        })?;

        // Parse requested event type filters (empty = all).
        let type_filters: Vec<EventType> = sub_req
            .event_types
            .iter()
            .filter_map(|s| parse_event_type(s))
            .collect();

        let workspace_filter = if sub_req.workspace.is_empty() {
            None
        } else {
            Some(sub_req.workspace.clone())
        };

        let rx = realtime.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(move |result| {
            let type_filters = type_filters.clone();
            let workspace_filter = workspace_filter.clone();

            match result {
                Err(_) => None, // Lagged — skip
                Ok(event) => {
                    // Apply event-type filter
                    if !type_filters.is_empty() && !type_filters.contains(&event.event_type) {
                        return None;
                    }

                    // Apply workspace filter (events carry workspace in `data.workspace`)
                    if let Some(ref ws) = workspace_filter {
                        let event_workspace = event
                            .data
                            .as_ref()
                            .and_then(|d| d.get("workspace"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if event_workspace != ws.as_str() {
                            return None;
                        }
                    }

                    let event_type = format!("{:?}", event.event_type)
                        .chars()
                        .enumerate()
                        .map(|(i, c)| {
                            if c.is_uppercase() && i > 0 {
                                format!("_{}", c.to_lowercase())
                            } else {
                                c.to_lowercase().to_string()
                            }
                        })
                        .collect::<String>();

                    let data_json =
                        serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());

                    Some(Ok(McpEvent {
                        event_type,
                        data_json,
                        sequence_id: event.seq_id.unwrap_or(0),
                    }))
                }
            }
        });

        Ok(Response::new(Box::pin(stream)))
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Start the gRPC server.
///
/// Binds to `addr` and serves until an error occurs or the process
/// is interrupted. Mirrors the signature of `serve_http()` in `http_transport`.
#[allow(clippy::result_large_err)]
pub async fn serve_grpc(
    handler: Arc<dyn McpHandler>,
    addr: SocketAddr,
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
) -> crate::error::Result<()> {
    let api_key = api_key.filter(|key| !key.is_empty());
    if !addr.ip().is_loopback() && api_key.is_none() {
        return Err(crate::error::EngramError::Internal(
            "gRPC non-loopback bind requires --grpc-api-key or ENGRAM_GRPC_API_KEY; terminate TLS at a trusted reverse proxy when exposing gRPC"
                .to_string(),
        ));
    }

    let service = GrpcMcpService::new(handler, api_key.clone(), realtime);
    let auth_api_key = api_key;
    let service = McpServiceServer::with_interceptor(service, move |mut request: Request<()>| {
        let principal = authenticate(request.metadata(), &auth_api_key)?;
        request.extensions_mut().insert(principal);
        Ok(request)
    });

    tracing::info!("gRPC transport listening on {}", addr);

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await
        .map_err(|e| crate::error::EngramError::Internal(e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoHandler;

    impl McpHandler for EchoHandler {
        fn handle_request(&self, request: McpRequest) -> McpResponse {
            McpResponse::success(request.id, serde_json::json!({"method": request.method}))
        }
    }

    fn make_service() -> GrpcMcpService {
        GrpcMcpService::new(Arc::new(EchoHandler), None, None)
    }

    // --- proto_to_handler_request ---

    #[test]
    fn converts_proto_request_with_id() {
        let proto_req = ProtoRequest {
            id: "42".to_string(),
            method: "tools/list".to_string(),
            params_json: r#"{"cursor":null}"#.to_string(),
        };
        let req = proto_to_handler_request(proto_req);
        assert_eq!(req.id, Some(serde_json::Value::String("42".to_string())));
        assert_eq!(req.method, "tools/list");
        assert_eq!(req.params["cursor"], serde_json::Value::Null);
    }

    #[test]
    fn converts_proto_notification_empty_id() {
        let proto_req = ProtoRequest {
            id: "".to_string(),
            method: "notifications/initialized".to_string(),
            params_json: "{}".to_string(),
        };
        let req = proto_to_handler_request(proto_req);
        assert!(req.id.is_none(), "empty id should map to None");
    }

    #[test]
    fn handles_invalid_params_json_gracefully() {
        let proto_req = ProtoRequest {
            id: "1".to_string(),
            method: "tools/call".to_string(),
            params_json: "not valid json {{".to_string(),
        };
        let req = proto_to_handler_request(proto_req);
        assert_eq!(req.params, serde_json::Value::Null);
    }

    // --- handler_to_proto_response ---

    #[test]
    fn converts_success_response() {
        let resp = McpResponse::success(
            Some(serde_json::Value::String("1".to_string())),
            serde_json::json!({"ok": true}),
        );
        let proto = handler_to_proto_response(resp);
        assert_eq!(proto.id, "1");
        match proto.result {
            Some(proto::mcp_response::Result::ResultJson(json)) => {
                assert!(json.contains("ok"));
            }
            other => panic!("expected ResultJson, got {:?}", other),
        }
    }

    #[test]
    fn converts_error_response() {
        let resp = McpResponse::error(
            Some(serde_json::Value::String("2".to_string())),
            -32601,
            "Method not found".to_string(),
        );
        let proto = handler_to_proto_response(resp);
        match proto.result {
            Some(proto::mcp_response::Result::Error(err)) => {
                assert_eq!(err.code, -32601);
                assert_eq!(err.message, "Method not found");
            }
            other => panic!("expected Error variant, got {:?}", other),
        }
    }

    #[test]
    fn auth_returns_anonymous_loopback_when_no_key_configured() {
        let metadata = MetadataMap::new();
        let principal = authenticate(&metadata, &None).expect("anonymous principal");
        assert!(matches!(
            principal,
            TransportPrincipal::AnonymousLoopback(_)
        ));
    }

    #[test]
    fn auth_fails_when_token_missing() {
        let metadata = MetadataMap::new();
        let key = Some("secret".to_string());
        assert_eq!(
            authenticate(&metadata, &key).unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
    }

    #[test]
    fn auth_fails_when_token_malformed() {
        let mut metadata = MetadataMap::new();
        metadata.insert("authorization", "Basic secret".parse().unwrap());
        let key = Some("secret".to_string());
        assert_eq!(
            authenticate(&metadata, &key).unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
    }

    #[test]
    fn auth_passes_with_correct_bearer_token() {
        let mut metadata = MetadataMap::new();
        metadata.insert("authorization", "Bearer secret".parse().unwrap());
        let key = Some("secret".to_string());
        let principal = authenticate(&metadata, &key).expect("process principal");
        assert!(matches!(principal, TransportPrincipal::ProcessBearer(_)));
    }

    #[test]
    fn auth_fails_with_wrong_bearer_token() {
        let mut metadata = MetadataMap::new();
        metadata.insert("authorization", "Bearer wrong".parse().unwrap());
        let key = Some("secret".to_string());
        let result = authenticate(&metadata, &key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    // --- parse_event_type ---

    #[test]
    fn parses_all_known_event_types() {
        let cases = [
            ("memory_created", EventType::MemoryCreated),
            ("memory_updated", EventType::MemoryUpdated),
            ("memory_deleted", EventType::MemoryDeleted),
            ("crossref_created", EventType::CrossrefCreated),
            ("crossref_deleted", EventType::CrossrefDeleted),
            ("sync_started", EventType::SyncStarted),
            ("sync_completed", EventType::SyncCompleted),
            ("sync_failed", EventType::SyncFailed),
        ];
        for (input, expected) in cases {
            assert_eq!(
                parse_event_type(input),
                Some(expected),
                "failed for {input}"
            );
        }
        assert_eq!(parse_event_type("unknown"), None);
    }

    // --- integration: round-trip through service.call() ---

    #[tokio::test]
    async fn grpc_call_round_trip() {
        let svc = make_service();
        let proto_req = ProtoRequest {
            id: "99".to_string(),
            method: "initialize".to_string(),
            params_json: "{}".to_string(),
        };
        let mut tonic_req = Request::new(proto_req);
        tonic_req
            .extensions_mut()
            .insert(TransportPrincipal::anonymous_loopback());
        let resp = svc.call(tonic_req).await.expect("call failed");
        let inner = resp.into_inner();
        assert_eq!(inner.id, "99");
        match inner.result {
            Some(proto::mcp_response::Result::ResultJson(json)) => {
                assert!(
                    json.contains("initialize"),
                    "expected method echo in result"
                );
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[tokio::test]
    async fn grpc_call_rejects_wrong_token() {
        let svc = GrpcMcpService::new(
            Arc::new(EchoHandler),
            Some("correct-token".to_string()),
            None,
        );
        let proto_req = ProtoRequest {
            id: "1".to_string(),
            method: "initialize".to_string(),
            params_json: "{}".to_string(),
        };
        let mut req = Request::new(proto_req);
        req.metadata_mut()
            .insert("authorization", "Bearer wrong-token".parse().unwrap());
        let err = svc.call(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn grpc_call_rejects_anonymous_request_without_interceptor_or_peer() {
        let svc = make_service();
        let req = Request::new(ProtoRequest {
            id: "2".to_string(),
            method: methods::INITIALIZE.to_string(),
            params_json: "{}".to_string(),
        });

        let err = svc.call(req).await.unwrap_err();

        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }
}
