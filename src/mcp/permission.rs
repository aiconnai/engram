//! Permission-mode classification for MCP tools.
//!
//! This is an opt-in guard. When no explicit permission mode is configured, MCP
//! dispatch preserves the existing local-first behavior.

use serde_json::{json, Value};

use super::tools::TOOL_DEFINITIONS;
use crate::auth::{Permission, PermissionSet, ResourceType, TransportPrincipal};
use crate::observability::{record_permission_denied, PermissionDeniedReason};

const MODE_ENV: &str = "ENGRAM_PERMISSION_MODE";

const ADMIN_TOOLS: &[&str] = &[
    "agent_deregister",
    "agent_register",
    "embedding_cache_clear",
    "identity_delete",
    "memory_cache_clear",
    "memory_delete",
    "memory_delete_batch",
    "memory_events_clear",
    "memory_grant_access",
    "memory_revoke_access",
    "retention_policy_delete",
    "search_cache_clear",
    "session_delete",
    "workspace_delete",
];

const MAINTENANCE_TOOLS: &[&str] = &[
    "lifecycle_run",
    "meilisearch_reindex",
    "memory_archive_old",
    "memory_cleanup_expired",
    "memory_embedding_migrate",
    "memory_migrate_images",
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

/// Names the dispatcher routes to the same handler as a catalog tool. They must
/// be classified exactly like their canonical tool (never fail open).
const DISPATCH_ALIASES: &[(&str, &str)] = &[
    ("graph_predict_links", "memory_predict_links"),
    ("graph_cluster_concepts", "memory_cluster_concepts"),
];

/// Parameters that make an otherwise read-only tool write. A call carrying a
/// truthy value requires `scoped_write`.
const WRITE_FLAG_PARAMS: &[(&str, &str)] = &[
    ("memory_predict_links", "auto_apply"),
    ("sync_state", "update_version"),
];

/// Resolve a dispatcher alias to the catalog tool it executes.
pub fn canonical_tool_name(tool_name: &str) -> &str {
    DISPATCH_ALIASES
        .iter()
        .find(|(alias, _)| *alias == tool_name)
        .map(|(_, canonical)| *canonical)
        .unwrap_or(tool_name)
}

pub fn required_mode(tool_name: &str) -> Option<PermissionMode> {
    let tool_name = canonical_tool_name(tool_name);
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

/// Mode required for one concrete call. Unknown tools require `admin` (fail
/// closed); write flags raise a read-only tool to `scoped_write`. Flags are
/// read from the arguments or, for transport pre-checks, `params.arguments`.
pub fn required_mode_for_call(tool_name: &str, params: &Value) -> PermissionMode {
    let base = required_mode(tool_name).unwrap_or(PermissionMode::Admin);
    let canonical = canonical_tool_name(tool_name);
    let writes = WRITE_FLAG_PARAMS
        .iter()
        .filter(|(tool, _)| *tool == canonical)
        .any(|(_, flag)| {
            [
                params.get(*flag),
                params.get("arguments").and_then(|a| a.get(*flag)),
            ]
            .into_iter()
            .flatten()
            .any(|value| !matches!(value, Value::Null | Value::Bool(false)))
        });
    if writes && base < PermissionMode::ScopedWrite {
        PermissionMode::ScopedWrite
    } else {
        base
    }
}

fn permission_denial_for_call(
    tool_name: &str,
    params: &Value,
    current: PermissionMode,
) -> Option<Value> {
    let required = required_mode_for_call(tool_name, params);
    if current.allows(required) {
        return None;
    }
    record_permission_denied(PermissionDeniedReason::ModeInsufficient);
    Some(permission_denied(tool_name, current, required))
}

pub fn permission_denial_for_mode(tool_name: &str, current: PermissionMode) -> Option<Value> {
    let required = required_mode(tool_name)?;
    if current.allows(required) {
        return None;
    }
    record_permission_denied(PermissionDeniedReason::ModeInsufficient);
    Some(permission_denied(tool_name, current, required))
}

pub fn permission_denial_from_env(tool_name: &str) -> Option<Value> {
    permission_denial_from_env_for_call(tool_name, &Value::Null)
}

fn permission_denial_from_env_for_call(tool_name: &str, params: &Value) -> Option<Value> {
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

    permission_denial_for_call(tool_name, params, mode)
}

/// Returns the active permission mode configured in the process environment, if any.
pub fn active_permission_mode() -> Option<PermissionMode> {
    std::env::var(MODE_ENV)
        .ok()
        .and_then(|raw| PermissionMode::parse(&raw))
}

/// Inspect permission modes and tool requirements report (RFC 0010).
pub fn permission_mode_status_report(target_tool: Option<&str>) -> Value {
    let active = active_permission_mode();
    let active_str = active.map(|m| m.as_str()).unwrap_or("unconstrained");

    if let Some(tool) = target_tool {
        let req = required_mode(tool);
        let allowed = match (active, req) {
            (Some(act), Some(r)) => act.allows(r),
            (None, _) => true,
            _ => false,
        };
        json!({
            "active_mode": active_str,
            "configured_via": if active.is_some() { "env" } else { "default_unconstrained" },
            "tool": tool,
            "required_mode": req.map(|r| r.as_str()),
            "allowed": allowed
        })
    } else {
        let all_tools = TOOL_DEFINITIONS.len();
        let allowed_count = if let Some(act) = active {
            TOOL_DEFINITIONS
                .iter()
                .filter(|t| {
                    required_mode(t.name)
                        .map(|r| act.allows(r))
                        .unwrap_or(false)
                })
                .count()
        } else {
            all_tools
        };

        json!({
            "active_mode": active_str,
            "configured_via": if active.is_some() { "env" } else { "default_unconstrained" },
            "modes_hierarchy": ["read_only", "scoped_write", "maintenance", "admin"],
            "total_tools_count": all_tools,
            "allowed_tools_count": allowed_count
        })
    }
}

fn permission_denied(tool_name: &str, current: PermissionMode, required: PermissionMode) -> Value {
    crate::mcp::error::ToolError::permission_denied(tool_name, current.as_str(), required.as_str())
        .into_value()
}

/// Conservative denial for a workspace-restricted principal calling a tool that
/// a workspace claim cannot scope. Same envelope as other permission denials;
/// `details.reason` distinguishes it from a permission-mode mismatch.
fn workspace_unscoped_tool_denied(tool_name: &str, principal: &TransportPrincipal) -> Value {
    record_permission_denied(PermissionDeniedReason::ToolNotWorkspaceScoped);
    crate::mcp::error::ToolError::permission_denied(
        tool_name,
        principal_permission_mode(&principal.auth_context().permissions).as_str(),
        required_mode(tool_name)
            .unwrap_or(PermissionMode::Admin)
            .as_str(),
    )
    .with_details(json!({"reason": "tool_not_workspace_scoped"}))
    .into_value()
}

pub fn permission_denial_for_principal(
    tool_name: &str,
    principal: &TransportPrincipal,
    requested_workspace: Option<&str>,
) -> Option<Value> {
    principal_denial_for_call(tool_name, &Value::Null, principal, requested_workspace)
}

fn principal_denial_for_call(
    tool_name: &str,
    params: &Value,
    principal: &TransportPrincipal,
    requested_workspace: Option<&str>,
) -> Option<Value> {
    let required = required_mode_for_call(tool_name, params);
    let current = principal_permission_mode(&principal.auth_context().permissions);
    let workspace_allowed = principal.allows_workspace(requested_workspace);
    if !workspace_allowed || !principal_allows_mode(principal, required) {
        record_permission_denied(if workspace_allowed {
            PermissionDeniedReason::ModeInsufficient
        } else {
            PermissionDeniedReason::WorkspaceNotAllowed
        });
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
    // Always prefer the verified transport principal identity to prevent spoofing
    if let Some(p) = principal {
        let user_str = p.auth_context().user_id.as_str();
        if !user_str.is_empty() && user_str != "system" && user_str != "anonymous" {
            return Some(user_str);
        }
    }
    // Fallback to params only when no authenticated principal (e.g., stdio transport)
    if let Some(id) = params.get("agent_id").and_then(|v| v.as_str()) {
        if !id.trim().is_empty() {
            return Some(id);
        }
    }
    None
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
    // Admins bypass granular scope grant checks
    if principal
        .map(|p| p.auth_context().permissions.is_admin())
        .unwrap_or(false)
    {
        return None;
    }

    let agent_id = extract_agent_id(params, principal)?;
    let scopes = extract_requested_scopes(params);
    if scopes.is_empty() {
        return None;
    }

    let required_perm = required_scope_permission(tool_name);
    for scope in scopes {
        match crate::storage::scope_grants::check_scope_access(conn, agent_id, scope, required_perm)
        {
            Ok(true) => continue,
            Ok(false) => {
                let current_mode = principal
                    .map(|p| principal_permission_mode(&p.auth_context().permissions).as_str())
                    .unwrap_or("unauthorized_scope");
                record_permission_denied(PermissionDeniedReason::ScopeGrantMissing);
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
                record_permission_denied(PermissionDeniedReason::AuthorizationCheckFailed);
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
    // Denominator for the permission-denied alert rule.
    crate::observability::record_authorization_check();

    // 0. Per-call explicit permission mode override (RFC 0010 per-request override)
    if let Some(mode_str) = params
        .get("_permission_mode")
        .or_else(|| params.get("permission_mode"))
        .and_then(|v| v.as_str())
    {
        if let Some(mode) = PermissionMode::parse(mode_str) {
            if let Some(denial) = permission_denial_for_call(tool_name, params, mode) {
                return Some(denial);
            }
        }
    }

    // 1. Env-level permission mode
    if let Some(denial) = permission_denial_from_env_for_call(tool_name, params) {
        return Some(denial);
    }

    // 2. Principal workspace & permission mode
    let metadata_only = crate::mcp::workspace_guard::is_catalog_metadata_tool(tool_name);
    if let Some(p) = principal.filter(|_| metadata_only) {
        // Catalog metadata carries no workspace data: only the mode applies.
        let required = required_mode_for_call(tool_name, params);
        if !principal_allows_mode(p, required) {
            record_permission_denied(PermissionDeniedReason::ModeInsufficient);
            return Some(permission_denied(
                tool_name,
                principal_permission_mode(&p.auth_context().permissions),
                required,
            ));
        }
    } else if let Some(p) = principal {
        if requests_all_workspaces(params) && !allows_all_workspaces(p) {
            record_permission_denied(PermissionDeniedReason::GlobalScopeRequested);
            return Some(permission_denied(
                tool_name,
                principal_permission_mode(&p.auth_context().permissions),
                PermissionMode::Admin,
            ));
        }

        // A workspace claim only scopes tools that accept a workspace argument
        // or that are limited to verified memory IDs (see `workspace_guard`).
        if !allows_all_workspaces(p)
            && !crate::mcp::workspace_guard::tool_honors_workspace_claim(tool_name)
        {
            return Some(workspace_unscoped_tool_denied(tool_name, p));
        }

        let workspaces = requested_workspaces(params);
        if workspaces.is_empty() {
            if matches!(p, crate::auth::TransportPrincipal::AnonymousLoopback(_)) {
                record_permission_denied(PermissionDeniedReason::WorkspaceClaimMissing);
                return Some(permission_denied(
                    tool_name,
                    principal_permission_mode(&p.auth_context().permissions),
                    required_mode_for_call(tool_name, params),
                ));
            }
            if let Some(denial) = principal_denial_for_call(tool_name, params, p, None) {
                return Some(denial);
            }
        } else {
            for ws in workspaces {
                if let Some(denial) = principal_denial_for_call(tool_name, params, p, Some(ws)) {
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
    record_permission_denied(PermissionDeniedReason::InvalidModeConfig);
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

    #[test]
    fn test_principal_overrides_params_agent_id() {
        use serde_json::json;
        // When a principal is present, params agent_id should be ignored
        let params = json!({"agent_id": "spoofed_agent"});

        let p = principal(None, PermissionSet::standard_user());

        let result = extract_agent_id(&params, Some(&p));
        assert_eq!(result, Some("user-1"));

        let result_none = extract_agent_id(&params, None);
        assert_eq!(result_none, Some("spoofed_agent"));
    }
}
