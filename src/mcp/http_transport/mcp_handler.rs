use std::time::Instant;

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

use super::super::permission::{
    allows_all_workspaces, permission_denial_for_principal, requested_workspaces,
    requests_all_workspaces,
};
use super::super::protocol::{methods, McpRequest, McpResponse};
use super::rate_limit::{is_rate_limit_allowed, rate_limited_response};
use super::{authenticate_transport_principal, AppState, MICROSECONDS_PER_MILLISECOND};

pub(super) async fn handle_mcp(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<std::net::SocketAddr>>,
    headers: HeaderMap,
    uri: Uri,
    Json(request): Json<McpRequest>,
) -> Response {
    let request_started = Instant::now();
    let is_notification = request.id.is_none();
    state.metrics.on_mcp_request_start(is_notification);

    let duration_ms = || {
        request_started
            .elapsed()
            .as_micros()
            .saturating_div(MICROSECONDS_PER_MILLISECOND as u128)
    };

    let mut decision = "success";
    let mut is_unauthorized = false;
    let mut is_forbidden = false;
    let mut is_rate_limited = false;
    let mut include_retry_after = false;

    let principal = authenticate_transport_principal(&state.api_key, &headers);
    let (status, response_payload) = match principal {
        Err(_) => {
            is_unauthorized = true;
            decision = "unauthorized";

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
                    .unwrap_or_else(|e| {
                        tracing::error!(
                            error = %e,
                            route = %uri.path(),
                            "failed to serialize error response"
                        );
                        serde_json::Value::Null
                    })
                },
            )
        }
        Ok(principal) => {
            if let Some(denial) = permission_denial_for_http_request(&request, &principal) {
                is_forbidden = true;
                decision = "forbidden";
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
                        .unwrap_or_else(|e| {
                            tracing::error!(
                                error = %e,
                                route = %uri.path(),
                                "failed to serialize forbidden response"
                            );
                            serde_json::Value::Null
                        })
                    },
                )
            } else if !is_rate_limit_allowed(&state, &headers, connect_info.map(|info| info.0))
                .await
            {
                is_rate_limited = true;
                decision = "rate_limited";
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

                let response = tokio::task::spawn_blocking(move || handler.handle_request(request))
                    .await
                    .unwrap_or_else(|e| {
                        tracing::error!(error = %e, route = %uri.path(), "MCP handler task failed or panicked");
                        McpResponse::error(request_id, -32603, "Internal server error".to_string())
                    });
                (
                    StatusCode::OK,
                    serde_json::to_value(response).unwrap_or_else(|e| {
                        tracing::error!(error = %e, route = %uri.path(), "failed to serialize MCP response");
                        serde_json::Value::Null
                    }),
                )
            }
        }
    };

    state.metrics.on_mcp_request_complete(
        !is_unauthorized && !is_forbidden && !is_rate_limited,
        is_unauthorized,
        is_rate_limited,
        request_started.elapsed(),
    );

    let response = if include_retry_after {
        (status, [("retry-after", "1")], Json(response_payload)).into_response()
    } else {
        (status, Json(response_payload)).into_response()
    };

    if is_rate_limited || is_unauthorized || is_forbidden {
        tracing::warn!(
            route = %uri.path(),
            status = %status,
            decision = decision,
            notification = is_notification,
            duration_ms = duration_ms(),
            "mcp_http_request"
        );
    } else {
        tracing::info!(
            route = %uri.path(),
            status = %status,
            decision = decision,
            notification = is_notification,
            duration_ms = duration_ms(),
            "mcp_http_request"
        );
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
    let requested_workspaces = requested_workspaces(&request.params);
    if let Some(denial) = requested_workspaces.iter().find_map(|workspace| {
        permission_denial_for_principal(tool_name, principal, Some(workspace))
    }) {
        return Some(denial);
    }

    if requests_all_workspaces(&request.params) && !allows_all_workspaces(principal) {
        return Some(workspace_scope_denial(tool_name));
    }

    if requested_workspaces.is_empty()
        && matches!(
            principal,
            crate::auth::TransportPrincipal::AnonymousLoopback(_)
        )
    {
        return Some(workspace_scope_denial(tool_name));
    }

    permission_denial_for_principal(tool_name, principal, requested_workspaces.first().copied())
}

fn workspace_scope_denial(tool_name: &str) -> serde_json::Value {
    serde_json::json!({
        "error": {
            "code": "permission_denied",
            "tool": tool_name,
            "message": "permission denied"
        }
    })
}
