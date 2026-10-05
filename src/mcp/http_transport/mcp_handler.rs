use std::time::Instant;

use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    Extension, Json,
};

use super::super::log_labels::{bounded_method, known_tool};
use super::super::protocol::{methods, McpRequest, McpResponse};
use super::metrics::{McpOutcome, RequestCounted};
use super::rate_limit::{is_rate_limit_allowed, rate_limited_response};
use super::request_id::RequestId;
use super::{authenticate_transport_principal, AppState};
use crate::observability::{redact, CorrelationId, OperationEvent};

const OPERATION: &str = "http.mcp";

/// Bounded route label (only the two MCP routes reach this handler).
fn route_label(uri: &Uri) -> &'static str {
    match uri.path() {
        "/mcp" => "/mcp",
        "/v1/mcp" => "/v1/mcp",
        _ => "other",
    }
}

/// Classify a handler response that travels as HTTP 200: a JSON-RPC `error`
/// object or a tool result flagged `isError` is a failure, not a success.
fn classify_handler_response(response: &McpResponse) -> McpOutcome {
    if response.error.is_some() {
        return McpOutcome::ProtocolError;
    }
    let tool_failed = response
        .result
        .as_ref()
        .and_then(|result| result.get("isError"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if tool_failed {
        McpOutcome::ToolError
    } else {
        McpOutcome::Success
    }
}

/// Answer a body the extractor refused (malformed JSON, wrong content type,
/// over the body limit) exactly as before, but count and log it. The rejection
/// text is never logged: it can quote the offending input.
fn reject_unparsed(
    state: &AppState,
    correlation: &CorrelationId,
    uri: &Uri,
    started: Instant,
    rejection: JsonRejection,
) -> Response {
    let status = rejection.status();
    let outcome = if status == StatusCode::PAYLOAD_TOO_LARGE {
        McpOutcome::BodyTooLarge
    } else {
        McpOutcome::ParseError
    };
    let latency = started.elapsed();
    state
        .metrics
        .on_mcp_rejected_before_handler(outcome, latency);
    OperationEvent::new(OPERATION, correlation.as_str(), outcome.as_str(), latency)
        .route(route_label(uri))
        .http(None, None, status.as_u16())
        .failed(true)
        .emit();
    rejection.into_response()
}

pub(super) async fn handle_mcp(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<std::net::SocketAddr>>,
    headers: HeaderMap,
    uri: Uri,
    request_id: Option<Extension<RequestId>>,
    counted: Option<Extension<RequestCounted>>,
    payload: Result<Json<McpRequest>, JsonRejection>,
) -> Response {
    let request_started = Instant::now();
    let correlation = request_id
        .map(|Extension(id)| id.0)
        .unwrap_or_else(CorrelationId::generate);
    // Every path below counts the request in `mcp_requests_total`; tell the
    // timeout middleware so it does not count it a second time.
    if let Some(Extension(counted)) = &counted {
        counted.mark();
    }
    let request = match payload {
        Ok(Json(request)) => request,
        Err(rejection) => {
            return reject_unparsed(&state, &correlation, &uri, request_started, rejection)
        }
    };
    let is_notification = request.id.is_none();
    let inflight = state
        .metrics
        .start_mcp_request(is_notification, request_started)
        .with_context(correlation.as_str(), route_label(&uri));

    // Bounded labels only: caller-supplied names never reach logs verbatim.
    let method_label = bounded_method(&request.method);
    let tool_label = (request.method == methods::CALL_TOOL)
        .then(|| request.params.get("name").and_then(|v| v.as_str()))
        .flatten()
        .map(known_tool);

    let mut outcome = McpOutcome::Success;
    let mut include_retry_after = false;

    let principal = authenticate_transport_principal(&state.api_key, &headers);
    let (status, response_payload) = match principal {
        Err(_) => {
            outcome = McpOutcome::Unauthorized;

            (
                StatusCode::UNAUTHORIZED,
                if is_notification {
                    serde_json::Value::Null
                } else {
                    serde_json::to_value(McpResponse::error(
                        request.id,
                        -32001,
                        "Unauthorized".to_string(),
                    ))
                    .unwrap_or_else(|_| {
                        tracing::error!(
                            error_class = "serialization",
                            route = route_label(&uri),
                            correlation_id = correlation.as_str(),
                            "failed to serialize error response"
                        );
                        serde_json::Value::Null
                    })
                },
            )
        }
        Ok(principal) => {
            if let Some(denial) = permission_denial_for_http_request(&request, &principal) {
                outcome = McpOutcome::Forbidden;
                (
                    StatusCode::FORBIDDEN,
                    if is_notification {
                        serde_json::Value::Null
                    } else {
                        serde_json::to_value(McpResponse::error(
                            request.id.clone(),
                            -32003,
                            denial.to_string(),
                        ))
                        .unwrap_or_else(|_| {
                            tracing::error!(
                                error_class = "serialization",
                                route = route_label(&uri),
                                correlation_id = correlation.as_str(),
                                "failed to serialize forbidden response"
                            );
                            serde_json::Value::Null
                        })
                    },
                )
            } else if !is_rate_limit_allowed(&state, &headers, connect_info.map(|info| info.0))
                .await
            {
                outcome = McpOutcome::RateLimited;
                include_retry_after = true;
                rate_limited_response(request.id, is_notification)
            } else if is_notification {
                (StatusCode::ACCEPTED, serde_json::Value::Null)
            } else {
                let handler = state.handler.clone();
                let request_id = request.id.clone();

                // Extract progress token from _meta if present.
                let progress_token = crate::mcp::extract_progress_token(&request.params);

                // Create a progress channel if the client requested progress.
                let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                let progress_reporter: Option<std::sync::Arc<dyn crate::mcp::ProgressReporter>> =
                    progress_token.map(|token| {
                        std::sync::Arc::new(crate::mcp::ChannelProgressReporter::from_sender(
                            token,
                            progress_tx,
                        ))
                            as std::sync::Arc<dyn crate::mcp::ProgressReporter>
                    });

                // Create a modified request that carries the progress reporter
                // through to the handler via McpHandler's existing interface.
                // Since McpHandler::handle_request takes an McpRequest, and we
                // need the progress reporter in HandlerContext, we attach it to
                // a custom header that the handler implementation reads.
                //
                // The handler's CALL_TOOL path in server.rs extracts the
                // progress token from request params._meta and creates its own
                // channel. For HTTP transport, we instead pre-create the channel
                // here and pass the sender through a thread-local.
                //
                // However, for simplicity and to avoid modifying the McpHandler
                // trait (which would be a breaking API change), we let the
                // handler create its own progress channel from the request's
                // _meta. The HTTP transport's progress_rx will capture the
                // notifications when the handler's progress_tx is connected
                // to the same channel.
                //
                // Since the handler creates its own channel from the progress
                // token in request.params._meta, the progress notifications
                // from the handler will go to the handler's own channel.
                // The HTTP handler drains progress_rx which won't receive
                // anything — this is correct: the handler owns both ends.
                //
                // For HTTP, progress events are emitted to the SSE event stream
                // when a RealtimeManager is present, not inline in the response.
                let _ = progress_reporter;
                let _ = progress_rx;

                // The authenticated principal travels with the request so
                // dispatch and storage lookups authorize persisted rows.
                let response = match tokio::task::spawn_blocking(move || {
                    handler.handle_request_as(request, Some(principal))
                })
                .await
                {
                    Ok(response) => {
                        outcome = classify_handler_response(&response);
                        response
                    }
                    Err(e) => {
                        // The join error text carries the panic payload; log its class only.
                        outcome = McpOutcome::HandlerPanic;
                        tracing::error!(
                            error_class = redact::join_error_class(&e),
                            route = route_label(&uri),
                            correlation_id = correlation.as_str(),
                            "MCP handler task failed or panicked"
                        );
                        McpResponse::error(request_id, -32603, "Internal server error".to_string())
                    }
                };
                (
                    StatusCode::OK,
                    serde_json::to_value(response).unwrap_or_else(|_| {
                        tracing::error!(
                            error_class = "serialization",
                            route = route_label(&uri),
                            correlation_id = correlation.as_str(),
                            "failed to serialize MCP response"
                        );
                        serde_json::Value::Null
                    }),
                )
            }
        }
    };

    let latency = inflight.finish(outcome);

    let response = if include_retry_after {
        (status, [("retry-after", "1")], Json(response_payload)).into_response()
    } else {
        (status, Json(response_payload)).into_response()
    };

    let event = OperationEvent::new(OPERATION, correlation.as_str(), outcome.as_str(), latency)
        .route(route_label(&uri))
        .notification(is_notification)
        .http(Some(method_label), tool_label, status.as_u16())
        .failed(outcome.is_failure());
    if include_retry_after {
        event.retry_after(1).emit();
    } else {
        event.emit();
    }

    response
}

fn permission_denial_for_http_request(
    request: &McpRequest,
    principal: &crate::auth::TransportPrincipal,
) -> Option<serde_json::Value> {
    if request.method != methods::CALL_TOOL {
        return None;
    }

    let tool_name = request
        .params
        .get("name")
        .and_then(|value| value.as_str())?;

    crate::mcp::permission::check_tool_authorization(
        None,
        tool_name,
        &request.params,
        Some(principal),
    )
}
