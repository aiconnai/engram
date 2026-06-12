//! Permission-mode classification for MCP tools.
//!
//! This is an opt-in guard. When no explicit permission mode is configured, MCP
//! dispatch preserves the existing local-first behavior.

use serde_json::{json, Value};

use super::tools::TOOL_DEFINITIONS;

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
    json!({
        "error": {
            "code": "permission_denied",
            "tool": tool_name,
            "current_mode": current.as_str(),
            "required_mode": required.as_str(),
            "message": format!("{tool_name} requires {} mode", required.as_str()),
            "audit_id": null
        }
    })
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
}
