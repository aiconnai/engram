use crate::auth::{AuthContext, Permission, ResourceType, TokenClaims};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportPrincipalError {
    MissingBearer,
    MalformedBearer,
    InvalidBearer,
    ExpiredToken,
}

#[derive(Debug, Clone)]
pub enum TransportPrincipal {
    ProcessBearer(AuthContext),
    StoredToken(AuthContext),
    AnonymousLoopback(AuthContext),
}

impl TransportPrincipal {
    pub fn from_process_bearer(
        authorization: Option<&str>,
        expected_secret: &str,
    ) -> Result<Self, TransportPrincipalError> {
        let header = authorization.ok_or(TransportPrincipalError::MissingBearer)?;
        let token = header
            .strip_prefix(BEARER_PREFIX)
            .ok_or(TransportPrincipalError::MalformedBearer)?;
        if token.is_empty() || expected_secret.is_empty() {
            return Err(TransportPrincipalError::MalformedBearer);
        }
        if constant_time_secret_eq(token, expected_secret) {
            Ok(Self::ProcessBearer(AuthContext::system()))
        } else {
            Err(TransportPrincipalError::InvalidBearer)
        }
    }

    pub fn from_token_claims(claims: TokenClaims) -> Result<Self, TransportPrincipalError> {
        if claims.is_expired() {
            return Err(TransportPrincipalError::ExpiredToken);
        }
        Ok(Self::StoredToken(AuthContext {
            user_id: claims.user_id,
            permissions: claims.permissions,
            namespace: claims.namespace,
        }))
    }

    pub fn anonymous_loopback() -> Self {
        Self::AnonymousLoopback(AuthContext::anonymous())
    }

    pub fn auth_context(&self) -> &AuthContext {
        match self {
            Self::ProcessBearer(context)
            | Self::StoredToken(context)
            | Self::AnonymousLoopback(context) => context,
        }
    }

    pub fn allows_workspace(&self, requested_workspace: Option<&str>) -> bool {
        if matches!(self, Self::AnonymousLoopback(_)) {
            return matches!(requested_workspace, None | Some("default"));
        }

        match (
            self.auth_context().namespace.as_deref(),
            requested_workspace,
        ) {
            (Some(namespace), Some(requested)) => namespace == requested,
            (Some(_), None) => false,
            (None, _) => true,
        }
    }

    pub fn has_permission(&self, permission: Permission, resource: ResourceType) -> bool {
        self.auth_context().has_permission(permission, resource)
    }
}

pub fn constant_time_secret_eq(candidate: &str, expected: &str) -> bool {
    let candidate_hash = Sha256::digest(candidate.as_bytes());
    let expected_hash = Sha256::digest(expected.as_bytes());
    bool::from(candidate_hash[..].ct_eq(&expected_hash[..]))
}

impl std::fmt::Display for TransportPrincipalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingBearer => "missing bearer token",
            Self::MalformedBearer => "malformed bearer token",
            Self::InvalidBearer => "invalid bearer token",
            Self::ExpiredToken => "expired stored token",
        };
        f.write_str(message)
    }
}

impl std::error::Error for TransportPrincipalError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{PermissionSet, UserId};
    use chrono::Utc;

    fn claims(namespace: Option<&str>, permissions: PermissionSet) -> TokenClaims {
        TokenClaims {
            user_id: UserId::from_string("user-1"),
            key_id: "key-1".to_string(),
            permissions,
            namespace: namespace.map(str::to_string),
            issued_at: Utc::now(),
            expires_at: None,
        }
    }

    #[test]
    fn process_bearer_accepts_valid_secret() {
        let principal =
            TransportPrincipal::from_process_bearer(Some("Bearer secret"), "secret").unwrap();

        assert!(principal.has_permission(Permission::Admin, ResourceType::System));
    }

    #[test]
    fn process_bearer_rejects_malformed_header() {
        let err =
            TransportPrincipal::from_process_bearer(Some("Basic secret"), "secret").unwrap_err();

        assert_eq!(err, TransportPrincipalError::MalformedBearer);
    }

    #[test]
    fn process_bearer_rejects_wrong_secret() {
        let err =
            TransportPrincipal::from_process_bearer(Some("Bearer wrong!"), "secret").unwrap_err();

        assert_eq!(err, TransportPrincipalError::InvalidBearer);
    }

    #[test]
    fn process_bearer_rejects_missing_token() {
        let err = TransportPrincipal::from_process_bearer(None, "secret").unwrap_err();

        assert_eq!(err, TransportPrincipalError::MissingBearer);
    }

    #[test]
    fn constant_time_secret_comparison_keeps_valid_and_wrong_paths_distinct() {
        assert!(constant_time_secret_eq(
            "abcdefghijklmnop",
            "abcdefghijklmnop"
        ));
        assert!(!constant_time_secret_eq(
            "abcdefghijklmnox",
            "abcdefghijklmnop"
        ));
    }

    #[test]
    fn stored_token_preserves_scope_and_workspace() {
        let principal = TransportPrincipal::from_token_claims(claims(
            Some("alpha"),
            PermissionSet::read_only(),
        ))
        .unwrap();

        assert!(principal.has_permission(Permission::Read, ResourceType::Memory));
        assert!(principal.allows_workspace(Some("alpha")));
        assert!(!principal.allows_workspace(Some("beta")));
    }

    #[test]
    fn anonymous_loopback_is_limited_to_default_read_scope() {
        let principal = TransportPrincipal::anonymous_loopback();

        assert!(principal.has_permission(Permission::Read, ResourceType::Memory));
        assert!(!principal.has_permission(Permission::Write, ResourceType::Memory));
        assert!(principal.allows_workspace(None));
        assert!(!principal.allows_workspace(Some("private")));
    }
}
