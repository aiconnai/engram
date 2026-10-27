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

/// Write-flag parameters of `tool_name` (aliases resolved as in enforcement).
fn write_flag_params(tool_name: &str) -> Vec<&'static str> {
    let canonical = canonical_tool_name(tool_name);
    WRITE_FLAG_PARAMS
        .iter()
        .filter(|(tool, _)| *tool == canonical)
        .map(|(_, flag)| *flag)
        .collect()
}

/// Bare arguments that set every given write flag, to ask enforcement for the
/// mode such a call requires.
fn flag_probe(flags: &[&str]) -> Value {
    Value::Object(
        flags
            .iter()
            .map(|flag| ((*flag).to_string(), Value::Bool(true)))
            .collect(),
    )
}

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
/// read from the bare arguments.
pub fn required_mode_for_call(tool_name: &str, params: &Value) -> PermissionMode {
    let base = required_mode(tool_name).unwrap_or(PermissionMode::Admin);
    let canonical = canonical_tool_name(tool_name);
    let writes = WRITE_FLAG_PARAMS
        .iter()
        .filter(|(tool, _)| *tool == canonical)
        .any(|(_, flag)| {
            params
                .get(*flag)
                .is_some_and(|value| !matches!(value, Value::Null | Value::Bool(false)))
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

/// Denial of `tool_name` under `current`, or `None` when the mode allows it.
///
/// Fails closed: a tool the catalog does not know requires `admin`, exactly as
/// in [`required_mode_for_call`], so an unknown name is never waved through.
pub fn permission_denial_for_mode(tool_name: &str, current: PermissionMode) -> Option<Value> {
    let required = required_mode_for_call(tool_name, &Value::Null);
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
    status_report_for(active_permission_mode(), target_tool)
}

fn status_report_for(active: Option<PermissionMode>, target_tool: Option<&str>) -> Value {
    let active_str = active.map(|m| m.as_str()).unwrap_or("unconstrained");

    if let Some(tool) = target_tool {
        let req = required_mode(tool);
        let allowed = match (active, req) {
            (Some(act), Some(r)) => act.allows(r),
            (None, _) => true,
            _ => false,
        };
        let mut report = json!({
            "active_mode": active_str,
            "configured_via": if active.is_some() { "env" } else { "default_unconstrained" },
            "tool": tool,
            "required_mode": req.map(|r| r.as_str()),
            "allowed": allowed
        });
        // `required_mode`/`allowed` describe a call without write flags. A call
        // that sets one is enforced at `scoped_write` or above, so report that
        // too (additive fields; the existing ones keep their meaning).
        let flags = write_flag_params(tool);
        if !flags.is_empty() {
            let escalated = required_mode_for_call(tool, &flag_probe(&flags));
            report["write_flag_params"] = json!(flags);
            report["required_mode_with_write_flags"] = json!(escalated.as_str());
            report["allowed_with_write_flags"] =
                json!(active.is_none_or(|act| act.allows(escalated)));
        }
        report
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
    let workspace_allowed = match requested_workspace {
        Some(raw) => crate::types::normalize_workspace(raw)
            .map(|normalized| principal.allows_workspace(Some(&normalized)))
            .unwrap_or(false),
        None => principal.allows_workspace(None),
    };
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

        if !allows_all_workspaces(p)
            && has_disallowed_workspace_claim_or_filter(tool_name, params, p)
        {
            record_permission_denied(PermissionDeniedReason::WorkspaceNotAllowed);
            return Some(permission_denied(
                tool_name,
                principal_permission_mode(&p.auth_context().permissions),
                required_mode_for_call(tool_name, params),
            ));
        }

        let workspaces = requested_workspaces(tool_name, params);
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

/// Split a `tools/call` params envelope (`{"name", "arguments"}`) into the tool
/// name and its bare arguments. Missing arguments yield `Value::Null`.
///
/// Transports call this exactly once, at the boundary, and hand everything
/// downstream (this module, the dispatcher, handlers) the bare arguments plus
/// the tool name. Nothing below the transport unwraps again, so a `name` or
/// `arguments` key inside real arguments is plain data, never an envelope.
pub fn tool_call_parts(envelope: &Value) -> Option<(&str, &Value)> {
    static NO_ARGUMENTS: Value = Value::Null;
    let name = envelope.get("name")?.as_str()?;
    Some((name, envelope.get("arguments").unwrap_or(&NO_ARGUMENTS)))
}

/// The bare arguments object a call's workspace claims live in.
///
/// `params` is always the bare tool arguments (see [`tool_call_parts`]); it is
/// never an envelope, so no key of it is interpreted as one.
fn argument_views(params: &Value) -> Vec<&serde_json::Map<String, Value>> {
    params.as_object().into_iter().collect()
}

/// Every workspace `params` claims for `tool_name`: the top-level `workspace`
/// and `workspaces`, `filters.workspace`, and the `workspace` of each item of
/// the tool's batch arrays (see
/// [`workspace_item_arrays`](crate::mcp::workspace_guard::workspace_item_arrays)).
pub(crate) fn requested_workspaces<'a>(tool_name: &str, params: &'a Value) -> Vec<&'a str> {
    let mut workspaces = Vec::new();
    for object in argument_views(params) {
        if let Some(Value::String(ws)) = object.get("workspace") {
            workspaces.push(ws.as_str());
        }
        if let Some(Value::Array(items)) = object.get("workspaces") {
            if !items.is_empty() && items.iter().all(Value::is_string) {
                for item in items {
                    if let Value::String(ws) = item {
                        workspaces.push(ws.as_str());
                    }
                }
            }
        }
        if let Some(filters) = object.get("filters").and_then(Value::as_object) {
            if let Some(Value::String(ws)) = filters.get("workspace") {
                workspaces.push(ws.as_str());
            }
        }
        for item in batch_items(tool_name, object) {
            if let Some(Value::String(ws)) = item.get("workspace") {
                workspaces.push(ws.as_str());
            }
        }
    }
    workspaces
}

/// Items of the tool's batch arrays (arrays whose items carry a `workspace`).
fn batch_items<'a>(
    tool_name: &str,
    object: &'a serde_json::Map<String, Value>,
) -> impl Iterator<Item = &'a Value> {
    crate::mcp::workspace_guard::workspace_item_arrays(tool_name)
        .iter()
        .filter_map(|key| object.get(key.as_str()).and_then(Value::as_array))
        .flatten()
}

/// Structured denial for a claim the principal may not make (malformed or
/// outside its workspaces) that a handler detected itself.
pub(crate) fn workspace_claim_denied(tool_name: &str, principal: &TransportPrincipal) -> Value {
    record_permission_denied(PermissionDeniedReason::WorkspaceNotAllowed);
    permission_denied(
        tool_name,
        principal_permission_mode(&principal.auth_context().permissions),
        required_mode_for_call(tool_name, &Value::Null),
    )
}

fn has_disallowed_workspace_claim_or_filter(
    tool_name: &str,
    params: &Value,
    principal: &TransportPrincipal,
) -> bool {
    argument_views(params)
        .into_iter()
        .any(|object| object_has_disallowed_workspace_claim(tool_name, object, principal))
}

fn object_has_disallowed_workspace_claim(
    tool_name: &str,
    object: &serde_json::Map<String, Value>,
    principal: &TransportPrincipal,
) -> bool {
    if let Some(ws) = object.get("workspace") {
        if !ws.is_null() && !ws.is_string() {
            return true;
        }
    }
    if let Some(wss) = object.get("workspaces") {
        match wss {
            Value::Null => {}
            Value::Array(items) if !items.is_empty() && items.iter().all(Value::is_string) => {}
            _ => return true,
        }
    }
    if let Some(filters) = object.get("filters").and_then(Value::as_object) {
        if let Some(ws) = filters.get("workspace") {
            if !ws.is_null() && !ws.is_string() {
                return true;
            }
        }
    }
    if let Some(filter) = object.get("filter") {
        if filter_has_disallowed_workspace(filter, principal) {
            return true;
        }
    }
    batch_items(tool_name, object).any(|item| {
        item.get("workspace")
            .is_some_and(|ws| !ws.is_null() && !ws.is_string())
    })
}

fn filter_has_disallowed_workspace(value: &Value, principal: &TransportPrincipal) -> bool {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if matches!(key.as_str(), "workspace" | "workspaces") {
                    if !workspace_filter_condition_allowed(child, principal) {
                        return true;
                    }
                } else if filter_has_disallowed_workspace(child, principal) {
                    return true;
                }
            }
            false
        }
        Value::Array(items) => items
            .iter()
            .any(|item| filter_has_disallowed_workspace(item, principal)),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => false,
    }
}

fn workspace_filter_condition_allowed(value: &Value, principal: &TransportPrincipal) -> bool {
    let check_str = |raw: &str| -> bool {
        crate::types::normalize_workspace(raw)
            .map(|norm| principal.allows_workspace(Some(&norm)))
            .unwrap_or(false)
    };
    match value {
        Value::String(ws) => check_str(ws),
        Value::Object(ops) => {
            !ops.is_empty()
                && ops.iter().all(|(op, operand)| match op.as_str() {
                    "eq" => operand.as_str().is_some_and(check_str),
                    "in" => operand.as_array().is_some_and(|arr| {
                        !arr.is_empty() && arr.iter().all(|v| v.as_str().is_some_and(check_str))
                    }),
                    _ => false,
                })
        }
        _ => false,
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
    fn unknown_tool_fails_closed_below_admin_mode() {
        for mode in [
            PermissionMode::ReadOnly,
            PermissionMode::ScopedWrite,
            PermissionMode::Maintenance,
        ] {
            let denial = permission_denial_for_mode("no_such_tool", mode)
                .unwrap_or_else(|| panic!("unknown tool allowed in {mode:?} mode"));
            assert_eq!(denial["error"]["code"], "permission_denied");
            assert_eq!(denial["error"]["required_mode"], "admin");
        }
        assert!(permission_denial_for_mode("no_such_tool", PermissionMode::Admin).is_none());
    }

    #[test]
    fn dispatch_aliases_are_classified_like_their_canonical_tool() {
        for (alias, canonical) in DISPATCH_ALIASES {
            for mode in [PermissionMode::ReadOnly, PermissionMode::ScopedWrite] {
                assert_eq!(
                    permission_denial_for_mode(alias, mode).is_some(),
                    permission_denial_for_mode(canonical, mode).is_some(),
                    "{alias} must be classified like {canonical} in {mode:?} mode"
                );
            }
        }
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

    fn alpha_writer() -> TransportPrincipal {
        principal(Some("alpha"), PermissionSet::standard_user())
    }

    #[test]
    fn bare_arguments_with_name_and_arguments_keys_are_not_unwrapped() {
        // `name`/`arguments` inside real arguments are plain data: they are not
        // an envelope, so the nested object is neither read as claims nor able
        // to change which workspace the call targets.
        let p = alpha_writer();
        let args = json!({"workspace": "alpha", "name": "x", "arguments": {"workspace": "beta"}});
        assert_eq!(requested_workspaces("memory_list", &args), vec!["alpha"]);
        assert!(check_tool_authorization(None, "memory_list", &args, Some(&p)).is_none());

        // The top-level claim is the only one that counts: a foreign top-level
        // workspace is denied no matter what `arguments` holds.
        let foreign =
            json!({"workspace": "beta", "name": "x", "arguments": {"workspace": "alpha"}});
        assert_eq!(requested_workspaces("memory_list", &foreign), vec!["beta"]);
        assert!(check_tool_authorization(None, "memory_list", &foreign, Some(&p)).is_some());
    }

    #[test]
    fn write_flag_is_read_from_bare_arguments_only() {
        let nested = json!({"name": "x", "arguments": {"auto_apply": true}});
        assert_eq!(
            required_mode_for_call("memory_predict_links", &nested),
            required_mode("memory_predict_links").unwrap()
        );
        assert_eq!(
            required_mode_for_call("memory_predict_links", &json!({"auto_apply": true})),
            PermissionMode::ScopedWrite
        );
    }

    #[test]
    fn tool_call_parts_unwraps_the_envelope_exactly_once() {
        let envelope =
            json!({"name": "memory_list", "arguments": {"name": "inner", "arguments": {"a": 1}}});
        let (name, arguments) = tool_call_parts(&envelope).unwrap();
        assert_eq!(name, "memory_list");
        assert_eq!(arguments, &json!({"name": "inner", "arguments": {"a": 1}}));

        let no_arguments = json!({"name": "memory_list"});
        assert_eq!(tool_call_parts(&no_arguments).unwrap().1, &Value::Null);
        assert!(tool_call_parts(&json!({"arguments": {}})).is_none());
        assert!(tool_call_parts(&json!({"name": 7})).is_none());
    }

    #[test]
    fn status_report_reflects_write_flag_escalation() {
        // Under read_only the default call of a flagged tool is allowed, but the
        // call that sets the write flag is not: the report must say so.
        let report =
            status_report_for(Some(PermissionMode::ReadOnly), Some("memory_predict_links"));
        assert_eq!(report["required_mode"], "read_only");
        assert_eq!(report["allowed"], true);
        assert_eq!(report["write_flag_params"], json!(["auto_apply"]));
        assert_eq!(report["required_mode_with_write_flags"], "scoped_write");
        assert_eq!(report["allowed_with_write_flags"], false);

        // Aliases resolve like enforcement does.
        let alias = status_report_for(Some(PermissionMode::ReadOnly), Some("graph_predict_links"));
        assert_eq!(alias["required_mode_with_write_flags"], "scoped_write");
        assert_eq!(alias["allowed_with_write_flags"], false);

        // Once the mode permits writes, both agree.
        let writer = status_report_for(
            Some(PermissionMode::ScopedWrite),
            Some("memory_predict_links"),
        );
        assert_eq!(writer["allowed"], true);
        assert_eq!(writer["allowed_with_write_flags"], true);

        // Unconstrained process: nothing is denied.
        let open = status_report_for(None, Some("memory_predict_links"));
        assert_eq!(open["allowed_with_write_flags"], true);
    }

    #[test]
    fn status_report_without_write_flags_is_unchanged() {
        let report = status_report_for(Some(PermissionMode::ReadOnly), Some("memory_get"));
        assert_eq!(report["required_mode"], "read_only");
        assert_eq!(report["allowed"], true);
        assert!(report.get("write_flag_params").is_none());
        assert!(report.get("allowed_with_write_flags").is_none());
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
