use std::time::Instant;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

use super::super::permission::permission_denial_for_principal;
use super::super::protocol::{methods, McpRequest, McpResponse};
use super::rate_limit::{is_rate_limit_allowed, rate_limited_response};
use super::{authenticate_transport_principal, AppState, MICROSECONDS_PER_MILLISECOND};

pub(super) async fn handle_mcp(
    State(state): State<AppState>,
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
            } else if !is_rate_limit_allowed(&state, &headers).await {
                is_rate_limited = true;
                decision = "rate_limited";
                include_retry_after = true;
                rate_limited_response(request.id, is_notification)
            } else if is_notification {
                (StatusCode::ACCEPTED, serde_json::Value::Null)
            } else {
                let response = state.handler.handle_request(request);
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
    let requested_workspace = request
        .params
        .get("arguments")
        .and_then(|arguments| arguments.get("workspace"))
        .and_then(|workspace| workspace.as_str());

    permission_denial_for_principal(tool_name, principal, requested_workspace)
}
