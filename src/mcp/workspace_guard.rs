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

use std::collections::{HashMap, HashSet};
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
    "memory_expand",
    "memory_explain_utility",
    "memory_versions",
    "memory_update",
    "memory_delete",
    "memory_delete_batch",
    "memory_link",
    "memory_unlink",
    "memory_set_lifecycle",
    "memory_lifecycle_update",
    "memory_score",
    "memory_promote",
    "memory_explain",
    "memory_reconcile_conflict",
    "memory_promote_to_permanent",
    "memory_boost",
    "memory_set_expiration",
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

/// Workspace the storage layer assigns to a memory created without one.
pub const DEFAULT_WORKSPACE: &str = "default";

/// Where a tool's JSON schema declares `workspace`/`workspaces` parameters.
///
/// Authorization reads claims only from the places listed here, so a schema
/// that declares a claim anywhere else must not be treated as scoped.
#[derive(Debug, Default)]
struct WorkspaceSites {
    /// Declared directly on the tool arguments.
    top_level: bool,
    /// Top-level array arguments whose object items declare their own
    /// `workspace` (batch tools such as `memory_create_batch`).
    item_arrays: Vec<String>,
    /// Declared in any other position (nested objects, `oneOf`, deeper arrays).
    /// Claims there are not extracted, so the tool fails closed.
    unsupported: bool,
}

impl WorkspaceSites {
    fn declares_any(&self) -> bool {
        self.top_level || !self.item_arrays.is_empty() || self.unsupported
    }

    fn is_enforceable(&self) -> bool {
        !self.unsupported && (self.top_level || !self.item_arrays.is_empty())
    }
}

#[derive(Debug, Clone, Copy)]
enum SchemaStep<'a> {
    Property(&'a str),
    Items,
    Other,
}

fn collect_workspace_sites<'a>(
    node: &'a Value,
    path: &mut Vec<SchemaStep<'a>>,
    sites: &mut WorkspaceSites,
) {
    match node {
        Value::Array(items) => {
            for item in items {
                collect_workspace_sites(item, path, sites);
            }
        }
        Value::Object(object) => {
            for (key, child) in object {
                match key.as_str() {
                    "properties" => collect_property_sites(child, path, sites),
                    "items" => {
                        path.push(SchemaStep::Items);
                        collect_workspace_sites(child, path, sites);
                        path.pop();
                    }
                    _ => {
                        path.push(SchemaStep::Other);
                        collect_workspace_sites(child, path, sites);
                        path.pop();
                    }
                }
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn collect_property_sites<'a>(
    properties: &'a Value,
    path: &mut Vec<SchemaStep<'a>>,
    sites: &mut WorkspaceSites,
) {
    let Some(properties) = properties.as_object() else {
        return;
    };
    for (name, child) in properties {
        if WORKSPACE_PARAMETER_KEYS.contains(&name.as_str()) {
            match path.as_slice() {
                [] => sites.top_level = true,
                [SchemaStep::Property(array_key), SchemaStep::Items] => {
                    if !sites.item_arrays.iter().any(|key| key == array_key) {
                        sites.item_arrays.push((*array_key).to_string());
                    }
                }
                _ => sites.unsupported = true,
            }
        }
        path.push(SchemaStep::Property(name));
        collect_workspace_sites(child, path, sites);
        path.pop();
    }
}

fn analyze_workspace_sites(schema: &Value) -> WorkspaceSites {
    let mut sites = WorkspaceSites::default();
    collect_workspace_sites(schema, &mut Vec::new(), &mut sites);
    sites
}

/// Workspace parameter sites declared by each tool's schema (tools without any
/// declaration are absent). A schema that fails to parse is absent, so the tool
/// is not scoped and restricted principals are denied.
static WORKSPACE_SITES: LazyLock<HashMap<&'static str, WorkspaceSites>> = LazyLock::new(|| {
    TOOL_DEFINITIONS
        .iter()
        .filter_map(|tool| {
            let schema = serde_json::from_str::<Value>(tool.schema).ok()?;
            let sites = analyze_workspace_sites(&schema);
            sites.declares_any().then_some((tool.name, sites))
        })
        .collect()
});

/// Top-level array arguments of `tool_name` whose object items carry their own
/// `workspace` (e.g. `memories` for `memory_create_batch`). Derived from the
/// catalog schema, so a new batch tool is covered the moment it is declared.
pub fn workspace_item_arrays(tool_name: &str) -> &'static [String] {
    WORKSPACE_SITES
        .get(tool_name)
        .map(|sites| sites.item_arrays.as_slice())
        .unwrap_or(&[])
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
    WORKSPACE_SITES
        .get(tool_name)
        .is_some_and(WorkspaceSites::is_enforceable)
        || ID_SCOPED_TOOLS.contains(&tool_name)
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
    let raw = match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse().ok(),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => None,
    };
    raw.filter(|&id| id > 0)
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

/// Handler guard for batch tools whose items carry their own `workspace`.
///
/// For a workspace-restricted principal, every item is authorized against the
/// workspace it will actually be written to: its own `workspace`, or
/// `inherited_workspace` when it names none (the handler default). Returns the
/// structured denial for the first item outside the principal's workspaces, so
/// the caller can refuse the whole batch before writing anything. A
/// non-string `workspace` is denied rather than silently falling back.
/// Unrestricted principals and `None` (stdio owner) are never denied here.
pub fn denial_for_batch_item_workspaces(
    principal: Option<&TransportPrincipal>,
    tool_name: &str,
    items: &[Value],
    inherited_workspace: &str,
) -> Option<Value> {
    let principal = principal.filter(|p| is_workspace_restricted(p))?;
    let mut checked: HashSet<&str> = HashSet::new();
    for item in items {
        let workspace = match item.get("workspace") {
            None | Some(Value::Null) => inherited_workspace,
            Some(Value::String(workspace)) => workspace.as_str(),
            Some(_) => {
                return Some(crate::mcp::permission::workspace_claim_denied(
                    tool_name, principal,
                ))
            }
        };
        if !checked.insert(workspace) {
            continue;
        }
        if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
            tool_name,
            principal,
            Some(workspace),
        ) {
            return Some(denial);
        }
    }
    None
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

    /// Batch tools whose handler calls [`denial_for_batch_item_workspaces`].
    const BATCH_TOOLS_WITH_HANDLER_GUARD: &[&str] =
        &["memory_create_batch", "memory_ingest_fact_batch"];

    #[test]
    fn catalog_declares_workspace_only_where_claims_are_extracted() {
        let unsupported: Vec<&str> = WORKSPACE_SITES
            .iter()
            .filter(|(_, sites)| sites.unsupported)
            .map(|(name, _)| *name)
            .collect();
        assert!(
            unsupported.is_empty(),
            "tools declare `workspace` where the authorization layer does not read \
             it (restricted principals are denied until it does): {unsupported:?}"
        );
    }

    #[test]
    fn batch_tools_are_derived_from_the_catalog_and_guarded_in_their_handlers() {
        let mut batch: Vec<&str> = WORKSPACE_SITES
            .iter()
            .filter(|(_, sites)| !sites.item_arrays.is_empty())
            .map(|(name, _)| *name)
            .collect();
        batch.sort_unstable();
        let mut guarded = BATCH_TOOLS_WITH_HANDLER_GUARD.to_vec();
        guarded.sort_unstable();
        assert_eq!(
            batch, guarded,
            "a tool whose schema puts `workspace` on array items needs \
             denial_for_batch_item_workspaces in its handler plus tests in \
             tests/workspace_batch_auth_tests.rs"
        );
        assert_eq!(workspace_item_arrays("memory_create_batch"), ["memories"]);
        assert_eq!(workspace_item_arrays("memory_ingest_fact_batch"), ["facts"]);
        assert!(workspace_item_arrays("memory_create").is_empty());
        assert!(workspace_item_arrays("no_such_tool").is_empty());
        for tool in BATCH_TOOLS_WITH_HANDLER_GUARD {
            assert!(tool_honors_workspace_claim(tool), "{tool} must stay scoped");
        }
    }

    #[test]
    fn workspace_site_analysis_fails_closed_on_unreadable_positions() {
        let top = analyze_workspace_sites(&json!({
            "type": "object",
            "properties": {"workspace": {"type": "string"}}
        }));
        assert!(top.top_level && top.is_enforceable() && !top.unsupported);

        let items = analyze_workspace_sites(&json!({
            "type": "object",
            "properties": {"rows": {"type": "array", "items": {
                "type": "object", "properties": {"workspace": {"type": "string"}}
            }}}
        }));
        assert_eq!(items.item_arrays, vec!["rows".to_string()]);
        assert!(items.is_enforceable());

        for unreadable in [
            // nested object
            json!({"properties": {"opts": {"type": "object", "properties": {
                "workspace": {"type": "string"}}}}}),
            // array of arrays
            json!({"properties": {"rows": {"type": "array", "items": {
                "type": "array", "items": {"type": "object", "properties": {
                    "workspace": {"type": "string"}}}}}}}),
            // behind a combinator
            json!({"oneOf": [{"properties": {"workspace": {"type": "string"}}}]}),
            // array items nested one level too deep
            json!({"properties": {"outer": {"type": "object", "properties": {
                "rows": {"type": "array", "items": {"type": "object", "properties": {
                    "workspace": {"type": "string"}}}}}}}}),
        ] {
            let sites = analyze_workspace_sites(&unreadable);
            assert!(sites.unsupported, "must be unsupported: {unreadable}");
            assert!(!sites.is_enforceable(), "must fail closed: {unreadable}");
        }
    }

    #[test]
    fn batch_item_workspace_denial_only_applies_to_restricted_principals() {
        use crate::auth::{PermissionSet, TokenClaims, UserId};

        let principal = |namespace: Option<&str>| {
            TransportPrincipal::from_token_claims(TokenClaims {
                user_id: UserId::from_string("u"),
                key_id: "k".to_string(),
                permissions: PermissionSet::standard_user(),
                namespace: namespace.map(str::to_string),
                issued_at: chrono::Utc::now(),
                expires_at: None,
            })
            .unwrap()
        };
        let restricted = principal(Some("alpha"));
        let unrestricted = principal(None);
        let own = [json!({"workspace": "alpha"}), json!({"workspace": "ALPHA"})];
        let foreign = [json!({"workspace": "alpha"}), json!({"workspace": "beta"})];
        let unscoped = [json!({"content": "x"})];
        let weird = [json!({"workspace": 7})];

        let denial = |p: Option<&TransportPrincipal>, items: &[Value], inherited: &str| {
            denial_for_batch_item_workspaces(p, "memory_create_batch", items, inherited)
        };
        assert!(denial(Some(&restricted), &own, "default").is_none());
        assert!(denial(Some(&restricted), &foreign, "default").is_some());
        assert!(denial(Some(&restricted), &unscoped, "default").is_some());
        assert!(denial(Some(&restricted), &unscoped, "alpha").is_none());
        assert!(denial(Some(&restricted), &weird, "alpha").is_some());
        for items in [&own[..], &foreign[..], &unscoped[..], &weird[..]] {
            assert!(denial(Some(&unrestricted), items, "default").is_none());
            assert!(denial(None, items, "default").is_none());
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
