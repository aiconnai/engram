//! Correlation id for every request to the transport's MCP and SSE routes.
//!
//! A well-formed caller `x-request-id` is kept (so client and server logs
//! join); anything else is replaced by a generated id. The id is stored in the
//! request extensions for handlers and logs, and echoed on every response,
//! including auth, parse and timeout rejections.

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

use crate::observability::CorrelationId;

pub(super) const REQUEST_ID_HEADER: &str = "x-request-id";

/// Correlation id of the current request (request extension).
#[derive(Clone)]
pub(super) struct RequestId(pub(super) CorrelationId);

pub(super) async fn attach_request_id(mut request: Request, next: Next) -> Response {
    let id = CorrelationId::from_header_or_generate(
        request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
    );
    request.extensions_mut().insert(RequestId(id.clone()));
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(id.as_str()) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }
    response
}
