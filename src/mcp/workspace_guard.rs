//! Persisted-workspace authorization for workspace-restricted principals.
//!
//! `permission::check_tool_authorization` validates the workspaces a request
//! *claims*. A claim alone never authorizes access to a stored row. For a
//! principal restricted to a subset of workspaces (anonymous loopback, or a
//! token bound to a namespace):
//!
//! - every memory referenced by ID is authorized against the workspace
//!   persisted on that row ([`ensure_memory_access`]); a foreign row and a
//!   missing row produce the same `not_found` error, so neither content nor
//!   existence leaks;
//! - a workspace claim only scopes tools whose schema accepts a workspace
//!   argument, or tools whose whole effect derives from verified memory IDs
//!   ([`tool_honors_workspace_claim`]); any other tool is denied.
//!
//! Principals that may access every workspace (process bearer, unscoped
//! tokens) and the stdio transport (no principal: the local process owner)
//! are not restricted here; see `docs/security/workspace-operation-matrix.md`.

use std::collections::HashSet;
use std::sync::LazyLock;

use rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

use crate::auth::TransportPrincipal;
use crate::error::{EngramError, Result};
use crate::mcp::error::ToolError;
use crate::mcp::tools::TOOL_DEFINITIONS;
use crate::storage::Storage;

/// Top-level argument keys whose values reference `memories.id`.
///
/// Every integer `id`/`*_id`/`*_ids` parameter in the tool catalog that
/// names a memory is listed; non-memory IDs (`since_id`, `share_id`,
/// `conflict_id`, `archive_id`, `version_id`, ...) are not.
pub const MEMORY_ID_ARGUMENT_KEYS: &[&str] = &[
    "id",
    "ids",
    "memory_id",
    "memory_ids",
    "from_id",
    "to_id",
    "parent_id",
    "summary_of_id",
    "focus_id",
    "source_memory_ids",
];

/// Tools without a workspace parameter that a restricted principal may call
/// because their reads and writes are limited to the memory IDs verified by
/// [`authorize_memory_arguments`] (and re-verified inside the handler).
///
/// Tools that follow references to other rows (graph traversal, summary
/// originals, focus-less exports) are deliberately absent.
pub const ID_SCOPED_TOOLS: &[&str] = &[
    "memory_get",
    "memory_get_public",
    "memory_versions",
    "memory_update",
    "memory_delete",
    "memory_delete_batch",
    "memory_link",
    "memory_unlink",
];

/// Tools whose `workspace` parameter is NOT the memory workspace: in the
/// Operational Context tools it is an alias for `workspace_path_hash`, a
/// caller-asserted scope that the artifact policy compares as given. A claim
/// therefore cannot bind them for a restricted principal. Audited against
/// every `workspace` property in the catalog (C1 review round 1).
pub const WORKSPACE_ALIAS_TOOLS: &[&str] = &[
    "context_record",
    "context_record_artifact",
    "context_get_artifact",
    "context_search",
    "context_build_bundle",
];

/// Tools that return only catalog or permission metadata and never read
/// workspace data. Restricted principals may call them without a claim.
pub const CATALOG_METADATA_TOOLS: &[&str] = &["discover_tools", "permission_mode_status"];

const WORKSPACE_PARAMETER_KEYS: &[&str] = &["workspace", "workspaces"];

/// Tools whose JSON schema declares a `workspace`/`workspaces` parameter at any
/// depth (e.g. `filters.workspace`).
static WORKSPACE_PARAMETER_TOOLS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    TOOL_DEFINITIONS
        .iter()
        .filter(|tool| {
            serde_json::from_str::<Value>(tool.schema)
                .map(|schema| schema_declares_workspace(&schema))
                .unwrap_or(false)
        })
        .map(|tool| tool.name)
        .collect()
});

fn schema_declares_workspace(schema: &Value) -> bool {
    match schema {
        Value::Object(object) => {
            let declared = object
                .get("properties")
                .and_then(Value::as_object)
                .is_some_and(|properties| {
                    WORKSPACE_PARAMETER_KEYS
                        .iter()
                        .any(|key| properties.contains_key(*key))
                });
            declared || object.values().any(schema_declares_workspace)
        }
        Value::Array(items) => items.iter().any(schema_declares_workspace),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => false,
    }
}

/// True when the principal may not access every workspace.
pub fn is_workspace_restricted(principal: &TransportPrincipal) -> bool {
    !crate::mcp::permission::allows_all_workspaces(principal)
}

/// Whether a workspace claim can scope `tool_name` for a restricted principal.
pub fn tool_honors_workspace_claim(tool_name: &str) -> bool {
    if WORKSPACE_ALIAS_TOOLS.contains(&tool_name) {
        return false;
    }
    WORKSPACE_PARAMETER_TOOLS.contains(tool_name) || ID_SCOPED_TOOLS.contains(&tool_name)
}

/// Whether `tool_name` returns only catalog/permission metadata.
pub fn is_catalog_metadata_tool(tool_name: &str) -> bool {
    CATALOG_METADATA_TOOLS.contains(&tool_name)
}

/// Memory IDs referenced by the top-level tool arguments.
///
/// Integers, integer strings, and arrays of either are collected so a value
/// a handler might coerce into an ID is never skipped by the guard.
pub fn memory_id_arguments(arguments: &Value) -> Vec<i64> {
    let Some(object) = arguments.as_object() else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for key in MEMORY_ID_ARGUMENT_KEYS {
        match object.get(*key) {
            Some(Value::Array(items)) => ids.extend(items.iter().filter_map(id_value)),
            Some(value) => ids.extend(id_value(value)),
            None => {}
        }
    }
    ids
}

fn id_value(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse().ok(),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => None,
    }
}

/// Authorize one memory row against the workspace persisted on it.
///
/// Returns `EngramError::NotFound(id)` when the row is missing *or* belongs to
/// a workspace the principal may not access, so callers cannot tell the two
/// apart. Unrestricted principals and `None` (stdio owner) always pass. Call it
/// on the same connection/transaction that reads or mutates the row.
pub fn ensure_memory_access(
    conn: &Connection,
    principal: Option<&TransportPrincipal>,
    id: i64,
) -> Result<()> {
    let Some(principal) = principal.filter(|p| is_workspace_restricted(p)) else {
        return Ok(());
    };
    let workspace: Option<String> = conn
        .query_row(
            "SELECT workspace FROM memories WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()?;
    match workspace {
        Some(workspace) if principal.allows_workspace(Some(&workspace)) => Ok(()),
        Some(_) | None => Err(EngramError::NotFound(id)),
    }
}

/// Authorize every memory ID in `arguments` before a tool runs.
pub fn authorize_memory_arguments(
    conn: &Connection,
    principal: Option<&TransportPrincipal>,
    arguments: &Value,
) -> Result<()> {
    for id in memory_id_arguments(arguments) {
        ensure_memory_access(conn, principal, id)?;
    }
    Ok(())
}

/// Dispatcher guard: the structured denial for a restricted principal whose
/// arguments reference a memory outside its workspaces, or `None` to proceed.
///
/// Fails closed: a storage error during the check is a denial.
pub fn denial_for_memory_arguments(
    storage: &Storage,
    principal: Option<&TransportPrincipal>,
    tool_name: &str,
    arguments: &Value,
) -> Option<Value> {
    let principal = principal.filter(|p| is_workspace_restricted(p))?;
    if memory_id_arguments(arguments).is_empty() {
        return None;
    }
    match storage
        .with_connection(|conn| authorize_memory_arguments(conn, Some(principal), arguments))
    {
        Ok(()) => None,
        Err(err @ EngramError::NotFound(_)) => {
            crate::observability::record_permission_denied(
                crate::observability::PermissionDeniedReason::ForeignOrMissingMemory,
            );
            Some(ToolError::from(err).into_value())
        }
        Err(err) => {
            crate::observability::record_permission_denied(
                crate::observability::PermissionDeniedReason::AuthorizationCheckFailed,
            );
            tracing::error!(
                tool = crate::mcp::log_labels::known_tool(tool_name),
                error = %crate::observability::redact::redacted(&err),
                "workspace authorization lookup failed; denying"
            );
            Some(ToolError::internal("workspace authorization check failed").into_value())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn workspace_parameter_detection_matches_catalog() {
        for tool in [
            "memory_list",
            "memory_search",
            "memory_create",
            "workspace_move",
        ] {
            assert!(
                tool_honors_workspace_claim(tool),
                "{tool} declares workspace"
            );
        }
        for tool in [
            "memory_export_graph",
            "memory_related",
            "memory_traverse",
            "memory_get_full",
            "graph_predict_links",
            "unknown_tool",
        ]
        .into_iter()
        .chain(WORKSPACE_ALIAS_TOOLS.iter().copied())
        {
            assert!(
                !tool_honors_workspace_claim(tool),
                "{tool} must not be scoped"
            );
        }
        for tool in ID_SCOPED_TOOLS {
            assert!(
                TOOL_DEFINITIONS.iter().any(|def| def.name == *tool),
                "{tool} must exist in the catalog"
            );
        }
    }

    #[test]
    fn memory_id_arguments_collects_known_keys_only() {
        let ids = memory_id_arguments(&json!({
            "id": 1,
            "ids": [2, "3", "x"],
            "from_id": "4",
            "to_id": 5.5,
            "since_id": 6,
            "metadata": {"id": 7}
        }));
        assert_eq!(ids, vec![1, 2, 3, 4]);
        assert!(memory_id_arguments(&json!([1, 2])).is_empty());
    }
}
