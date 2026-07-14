//! WebSocket principal extraction and subscription authorization.

use axum::http::{header::AUTHORIZATION, HeaderMap};

use crate::auth::{TransportPrincipal, TransportPrincipalError};

pub(super) fn principal_can_subscribe(principal: &TransportPrincipal, workspace: &str) -> bool {
    principal.allows_workspace(Some(workspace))
        && principal.has_permission(
            crate::auth::Permission::Read,
            crate::auth::ResourceType::Memory,
        )
}

pub(super) fn websocket_principal(
    headers: &HeaderMap,
    auth_key: Option<&str>,
) -> Result<TransportPrincipal, TransportPrincipalError> {
    match auth_key {
        Some(expected) => {
            let authorization = headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok());
            TransportPrincipal::from_process_bearer(authorization, expected)
        }
        None => Ok(TransportPrincipal::anonymous_loopback()),
    }
}
