//! Permission-mode classification for MCP tools.
//!
//! This is an opt-in guard. When no explicit permission mode is configured, MCP
//! dispatch preserves the existing local-first behavior.

use serde_json::{json, Value};

use super::tools::TOOL_DEFINITIONS;
use crate::auth::{Permission, PermissionSet, ResourceType, TransportPrincipal};

const MODE_ENV: &str = "ENGRAM_PERMISSION_MODE";

const ADMIN_TOOLS: &[&str] = &[
    "agent_deregister",
    "agent_register",
    "embedding_cache_clear",
    "identity_delete",
    "memory_delete",
    "memory_delete_batch",
    "memory_grant_access",
    "memory_revoke_access",
    "retention_policy_delete",
    "session_delete",
    "workspace_delete",
];

const MAINTENANCE_TOOLS: &[&str] = &[
    "lifecycle_run",
    "meilisearch_reindex",
    "memory_archive_old",
    "memory_cleanup_expired",
    "memory_embedding_migrate",
    "memory_rebuild_crossrefs",
    "memory_rebuild_embeddings",
    "pending_injections_cleanup",
    "retention_policy_apply",
    "sync_cleanup",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionMode {
    ReadOnly,
    ScopedWrite,
    Maintenance,
    Admin,
}

impl PermissionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ScopedWrite => "scoped_write",
            Self::Maintenance => "maintenance",
            Self::Admin => "admin",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "read_only" => Some(Self::ReadOnly),
            "scoped_write" => Some(Self::ScopedWrite),
            "maintenance" => Some(Self::Maintenance),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn allows(self, required: Self) -> bool {
        self >= required
    }
}

pub fn required_mode(tool_name: &str) -> Option<PermissionMode> {
    let tool = TOOL_DEFINITIONS
        .iter()
        .find(|tool| tool.name == tool_name)?;

    if ADMIN_TOOLS.contains(&tool_name) {
        return Some(PermissionMode::Admin);
    }
    if MAINTENANCE_TOOLS.contains(&tool_name) {
        return Some(PermissionMode::Maintenance);
    }
    if tool.annotations.read_only_hint == Some(true) {
        return Some(PermissionMode::ReadOnly);
    }
    if tool.annotations.destructive_hint == Some(true) {
        return Some(PermissionMode::Admin);
    }

    Some(PermissionMode::ScopedWrite)
}

pub fn permission_denial_for_mode(tool_name: &str, current: PermissionMode) -> Option<Value> {
    let required = required_mode(tool_name)?;
    if current.allows(required) {
        return None;
    }
    Some(permission_denied(tool_name, current, required))
}

pub fn permission_denial_from_env(tool_name: &str) -> Option<Value> {
    let raw = match std::env::var(MODE_ENV) {
        Ok(value) if !value.trim().is_empty() => value,
        Ok(_) | Err(std::env::VarError::NotPresent) => return None,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Some(invalid_permission_mode(tool_name, "<non-unicode>"));
        }
    };

    let Some(mode) = PermissionMode::parse(&raw) else {
        return Some(invalid_permission_mode(tool_name, &raw));
    };

    permission_denial_for_mode(tool_name, mode)
}

fn permission_denied(tool_name: &str, current: PermissionMode, required: PermissionMode) -> Value {
    crate::mcp::error::ToolError::permission_denied(tool_name, current.as_str(), required.as_str())
        .into_value()
}

pub fn permission_denial_for_principal(
    tool_name: &str,
    principal: &TransportPrincipal,
    requested_workspace: Option<&str>,
) -> Option<Value> {
    let required = required_mode(tool_name)?;
    let current = principal_permission_mode(&principal.auth_context().permissions);
    if !principal.allows_workspace(requested_workspace)
        || !principal_allows_mode(principal, required)
    {
        return Some(permission_denied(tool_name, current, required));
    }
    None
}

pub fn extract_requested_scopes(params: &Value) -> Vec<&str> {
    let mut scopes = Vec::new();
    collect_requested_scopes(params, &mut scopes);
    scopes
}

fn collect_requested_scopes<'a>(value: &'a Value, scopes: &mut Vec<&'a str>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if matches!(key.as_str(), "scope" | "scope_path") {
                    if let Some(s) = child.as_str() {
                        if !s.trim().is_empty() {
                            scopes.push(s);
                        }
                    } else if let Some(arr) = child.as_array() {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                if !s.trim().is_empty() {
                                    scopes.push(s);
                                }
                            }
                        }
                    }
                } else {
                    collect_requested_scopes(child, scopes);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_requested_scopes(item, scopes);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

pub fn extract_agent_id<'a>(
    params: &'a Value,
    principal: Option<&'a TransportPrincipal>,
) -> Option<&'a str> {
    if let Some(id) = params.get("agent_id").and_then(|v| v.as_str()) {
        if !id.trim().is_empty() {
            return Some(id);
        }
    }
    principal.and_then(|p| {
        let user_str = p.auth_context().user_id.as_str();
        if !user_str.is_empty() && user_str != "system" && user_str != "anonymous" {
            Some(user_str)
        } else {
            None
        }
    })
}

pub fn required_scope_permission(tool_name: &str) -> &'static str {
    if ADMIN_TOOLS.contains(&tool_name) {
        return "admin";
    }
    match required_mode(tool_name) {
        Some(PermissionMode::Admin) | Some(PermissionMode::Maintenance) => "admin",
        Some(PermissionMode::ScopedWrite) => "write",
        Some(PermissionMode::ReadOnly) | None => "read",
    }
}

pub fn check_scope_authorization(
    conn: &rusqlite::Connection,
    tool_name: &str,
    params: &Value,
    principal: Option<&TransportPrincipal>,
) -> Option<Value> {
    let agent_id = extract_agent_id(params, principal)?;
    let scopes = extract_requested_scopes(params);
    if scopes.is_empty() {
        return None;
    }

    let required_perm = required_scope_permission(tool_name);
    for scope in scopes {
        // Global root or unparameterized scope does not require specific path grant
        if scope.eq_ignore_ascii_case("global") {
            continue;
        }

        match crate::storage::scope_grants::check_scope_access(conn, agent_id, scope, required_perm)
        {
            Ok(true) => continue,
            Ok(false) => {
                let current_mode = principal
                    .map(|p| principal_permission_mode(&p.auth_context().permissions).as_str())
                    .unwrap_or("unauthorized_scope");
                return Some(
                    crate::mcp::error::ToolError::permission_denied(
                        tool_name,
                        current_mode,
                        required_perm,
                    )
                    .with_details(json!({
                        "agent_id": agent_id,
                        "scope": scope,
                        "required_permission": required_perm,
                        "reason": format!("Agent '{agent_id}' does not have '{required_perm}' access to scope '{scope}'")
                    }))
                    .into_value(),
                );
            }
            Err(e) => {
                return Some(crate::mcp::error::ToolError::from(e).into_value());
            }
        }
    }

    None
}

/// Central authorization guard checking:
/// 1. Environment permission modes (ENGRAM_PERMISSION_MODE)
/// 2. Transport principal modes and workspace boundaries
/// 3. Hierarchical scope access grants (when connection and scope parameters are available)
pub fn check_tool_authorization(
    conn: Option<&rusqlite::Connection>,
    tool_name: &str,
    params: &Value,
    principal: Option<&TransportPrincipal>,
) -> Option<Value> {
    // 1. Env-level permission mode
    if let Some(denial) = permission_denial_from_env(tool_name) {
        return Some(denial);
    }

    // 2. Principal workspace & permission mode
    if let Some(p) = principal {
        if requests_all_workspaces(params) && !allows_all_workspaces(p) {
            return Some(permission_denied(
                tool_name,
                principal_permission_mode(&p.auth_context().permissions),
                PermissionMode::Admin,
            ));
        }

        let workspaces = requested_workspaces(params);
        if workspaces.is_empty() {
            if matches!(p, crate::auth::TransportPrincipal::AnonymousLoopback(_)) {
                return Some(permission_denied(
                    tool_name,
                    principal_permission_mode(&p.auth_context().permissions),
                    required_mode(tool_name).unwrap_or(PermissionMode::ReadOnly),
                ));
            }
            if let Some(denial) = permission_denial_for_principal(tool_name, p, None) {
                return Some(denial);
            }
        } else {
            for ws in workspaces {
                if let Some(denial) = permission_denial_for_principal(tool_name, p, Some(ws)) {
                    return Some(denial);
                }
            }
        }
    }

    // 3. Hierarchical scope access grant verification
    if let Some(connection) = conn {
        if let Some(denial) = check_scope_authorization(connection, tool_name, params, principal) {
            return Some(denial);
        }
    }

    None
}

pub(crate) fn requested_workspaces(params: &Value) -> Vec<&str> {
    let mut workspaces = Vec::new();
    collect_requested_workspaces(params, &mut workspaces);
    workspaces
}

fn collect_requested_workspaces<'a>(value: &'a Value, workspaces: &mut Vec<&'a str>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if matches!(key.as_str(), "workspace" | "workspaces") {
                    collect_workspace_values(child, workspaces);
                } else {
                    collect_requested_workspaces(child, workspaces);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_requested_workspaces(item, workspaces);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn collect_workspace_values<'a>(value: &'a Value, workspaces: &mut Vec<&'a str>) {
    match value {
        Value::String(workspace) => workspaces.push(workspace),
        Value::Array(items) => {
            for item in items {
                collect_workspace_values(item, workspaces);
            }
        }
        Value::Object(object) => {
            for child in object.values() {
                collect_workspace_values(child, workspaces);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

pub(crate) fn requests_all_workspaces(params: &Value) -> bool {
    match params {
        Value::Object(object) => object.iter().any(|(key, value)| {
            (key == "global" && value.as_bool() == Some(true)) || requests_all_workspaces(value)
        }),
        Value::Array(items) => items.iter().any(requests_all_workspaces),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => false,
    }
}

pub(crate) fn allows_all_workspaces(principal: &TransportPrincipal) -> bool {
    !matches!(principal, TransportPrincipal::AnonymousLoopback(_))
        && principal.allows_workspace(None)
}

fn principal_allows_mode(principal: &TransportPrincipal, required: PermissionMode) -> bool {
    let permissions = &principal.auth_context().permissions;
    match required {
        PermissionMode::ReadOnly => {
            permissions.has_permission(Permission::Read, ResourceType::Memory)
        }
        PermissionMode::ScopedWrite => {
            permissions.has_permission(Permission::Write, ResourceType::Memory)
        }
        PermissionMode::Maintenance | PermissionMode::Admin => {
            permissions.has_permission(Permission::Admin, ResourceType::System)
        }
    }
}

fn principal_permission_mode(permissions: &PermissionSet) -> PermissionMode {
    if permissions.has_permission(Permission::Admin, ResourceType::System) {
        PermissionMode::Admin
    } else if permissions.has_permission(Permission::Write, ResourceType::Memory) {
        PermissionMode::ScopedWrite
    } else {
        PermissionMode::ReadOnly
    }
}

fn invalid_permission_mode(tool_name: &str, raw: &str) -> Value {
    json!({
        "error": {
            "code": "invalid_permission_mode",
            "tool": tool_name,
            "current_mode": raw,
            "required_mode": null,
            "message": format!("{MODE_ENV} must be one of read_only, scoped_write, maintenance, admin"),
            "audit_id": null
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{TokenClaims, UserId};
    use chrono::Utc;

    fn principal(namespace: Option<&str>, permissions: PermissionSet) -> TransportPrincipal {
        TransportPrincipal::from_token_claims(TokenClaims {
            user_id: UserId::from_string("user-1"),
            key_id: "key-1".to_string(),
            permissions,
            namespace: namespace.map(str::to_string),
            issued_at: Utc::now(),
            expires_at: None,
        })
        .unwrap()
    }

    #[test]
    fn classifies_representative_tools() {
        assert_eq!(required_mode("memory_get"), Some(PermissionMode::ReadOnly));
        assert_eq!(
            required_mode("memory_create"),
            Some(PermissionMode::ScopedWrite)
        );
        assert_eq!(
            required_mode("lifecycle_run"),
            Some(PermissionMode::Maintenance)
        );
        assert_eq!(required_mode("memory_delete"), Some(PermissionMode::Admin));
        assert_eq!(required_mode("nonexistent_tool"), None);
    }

    #[test]
    fn read_only_mode_denies_admin_tool_with_structured_error() {
        let denial = permission_denial_for_mode("memory_delete", PermissionMode::ReadOnly).unwrap();
        assert_eq!(denial["error"]["code"], "permission_denied");
        assert_eq!(denial["error"]["tool"], "memory_delete");
        assert_eq!(denial["error"]["current_mode"], "read_only");
        assert_eq!(denial["error"]["required_mode"], "admin");
    }

    #[test]
    fn read_only_mode_allows_read_only_tool() {
        let denial = permission_denial_for_mode("memory_get", PermissionMode::ReadOnly);
        assert!(denial.is_none());
    }

    #[test]
    fn scoped_write_mode_denies_maintenance_tool() {
        let denial =
            permission_denial_for_mode("lifecycle_run", PermissionMode::ScopedWrite).unwrap();
        assert_eq!(denial["error"]["code"], "permission_denied");
        assert_eq!(denial["error"]["required_mode"], "maintenance");
    }

    #[test]
    fn stored_scope_allows_read_tool() {
        let principal = principal(Some("alpha"), PermissionSet::read_only());

        let denial = permission_denial_for_principal("memory_get", &principal, Some("alpha"));

        assert!(denial.is_none());
    }

    #[test]
    fn stored_scope_denies_write_tool() {
        let principal = principal(Some("alpha"), PermissionSet::read_only());

        let denial =
            permission_denial_for_principal("memory_create", &principal, Some("alpha")).unwrap();

        assert_eq!(denial["error"]["code"], "permission_denied");
        assert_eq!(denial["error"]["required_mode"], "scoped_write");
    }

    #[test]
    fn stored_scope_denies_workspace_mismatch() {
        let principal = principal(Some("alpha"), PermissionSet::standard_user());

        let denial =
            permission_denial_for_principal("memory_create", &principal, Some("beta")).unwrap();

        assert_eq!(denial["error"]["code"], "permission_denied");
        assert_eq!(denial["error"]["current_mode"], "scoped_write");
    }

    #[test]
    fn anonymous_loopback_allows_only_read_default_workspace() {
        let principal = TransportPrincipal::anonymous_loopback();

        let allowed = permission_denial_for_principal("memory_get", &principal, Some("default"));
        let denied = permission_denial_for_principal("memory_create", &principal, Some("default"));
        let private = permission_denial_for_principal("memory_get", &principal, Some("private"));

        assert!(allowed.is_none());
        assert!(denied.is_some());
        assert!(private.is_some());
    }

    #[test]
    fn principal_denial_helper_allows_read_and_denies_write() {
        let principal = principal(Some("alpha"), PermissionSet::read_only());

        let allowed = permission_denial_for_principal("memory_get", &principal, Some("alpha"));
        let denied = permission_denial_for_principal("memory_create", &principal, Some("alpha"));

        assert!(allowed.is_none());
        assert!(denied.is_some());
    }
}
