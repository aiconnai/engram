//! MCP Resource definitions and handlers for engram
//!
//! Implements the `resources/list` and `resources/read` MCP methods.
//! Resources expose engram data as addressable URIs that MCP clients can browse.
//!
//! Supported URI patterns:
//! - `engram://stats` — global storage statistics
//! - `engram://entities` — known entities (top 100 by mention count)
//! - `engram://memory/{id}` — a single memory by numeric ID
//! - `engram://workspace/{name}` — workspace statistics
//! - `engram://workspace/{name}/memories` — paginated memories in a workspace

use serde_json::{json, Value};

use crate::mcp::protocol::ResourceTemplate;
use crate::storage::queries::{get_memory, get_stats, get_workspace_stats, list_memories};
use crate::storage::{entity_queries::list_entities, Storage};
use crate::types::ListOptions;

/// Return all resource URI templates that engram exposes.
///
/// These are returned to MCP clients via `resources/list`.
pub fn list_resources() -> Vec<ResourceTemplate> {
    vec![
        ResourceTemplate {
            uri_template: "engram://stats".to_string(),
            name: "Global Statistics".to_string(),
            description: Some("Storage statistics across all workspaces".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        ResourceTemplate {
            uri_template: "engram://entities".to_string(),
            name: "Entities".to_string(),
            description: Some(
                "Known entities extracted from memories (top 100 by mention count)".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
        ResourceTemplate {
            uri_template: "engram://memory/{id}".to_string(),
            name: "Memory".to_string(),
            description: Some("A single memory by numeric ID".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        ResourceTemplate {
            uri_template: "engram://workspace/{name}".to_string(),
            name: "Workspace Statistics".to_string(),
            description: Some("Statistics for a named workspace".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        ResourceTemplate {
            uri_template: "engram://workspace/{name}/memories".to_string(),
            name: "Workspace Memories".to_string(),
            description: Some(
                "Paginated memories in a workspace. Supports ?limit=N&offset=N query params."
                    .to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

/// Read a resource by URI and return its JSON content.
///
/// Returns `Ok(Value)` on success, or `Err(String)` with a human-readable
/// error message that will be forwarded to the MCP client.
///
/// Supported URIs:
/// - `engram://stats`
/// - `engram://entities`
/// - `engram://memory/{id}`
/// - `engram://workspace/{name}`
/// - `engram://workspace/{name}/memories[?limit=N&offset=N]`
pub fn read_resource(storage: &Storage, uri: &str) -> Result<Value, String> {
    // Strip optional query string before routing
    let (path, query) = split_uri(uri);

    if path == "engram://stats" {
        read_stats(storage)
    } else if path == "engram://entities" {
        read_entities(storage)
    } else if let Some(rest) = path.strip_prefix("engram://memory/") {
        let id: i64 = rest
            .parse()
            .map_err(|_| format!("Invalid memory ID: {}", rest))?;
        read_memory(storage, id)
    } else if let Some(rest) = path.strip_prefix("engram://workspace/") {
        // Distinguish `workspace/{name}` from `workspace/{name}/memories`
        if let Some(name) = rest.strip_suffix("/memories") {
            read_workspace_memories(storage, name, query.as_deref())
        } else {
            read_workspace(storage, rest)
        }
    } else {
        Err(format!("Unknown resource URI: {}", uri))
    }
}

/// Validate whether a given URI is a supported engram resource URI.
pub fn validate_resource_uri(uri: &str) -> bool {
    let (path, _) = split_uri(uri);
    if path == "engram://stats" || path == "engram://entities" {
        return true;
    }
    if let Some(rest) = path.strip_prefix("engram://memory/") {
        return rest.parse::<i64>().is_ok();
    }
    if let Some(rest) = path.strip_prefix("engram://workspace/") {
        if let Some(name) = rest.strip_suffix("/memories") {
            return !name.trim().is_empty();
        }
        return !rest.trim().is_empty();
    }
    false
}

/// Thread-safe registry and matcher for active MCP resource subscriptions.
#[derive(Debug, Clone, Default)]
pub struct ResourceSubscriptionManager {
    subscriptions: std::sync::Arc<parking_lot::RwLock<std::collections::HashSet<String>>>,
}

impl ResourceSubscriptionManager {
    /// Create a new empty subscription manager.
    pub fn new() -> Self {
        Self {
            subscriptions: std::sync::Arc::new(parking_lot::RwLock::new(
                std::collections::HashSet::new(),
            )),
        }
    }

    /// Subscribe to a resource URI. Returns `Ok(())` or `Err(String)` if URI is invalid.
    pub fn subscribe(&self, uri: &str) -> Result<(), String> {
        let uri = uri.trim();
        if !validate_resource_uri(uri) {
            return Err(format!("Invalid resource URI: {}", uri));
        }
        let mut subs = self.subscriptions.write();
        subs.insert(uri.to_string());
        Ok(())
    }

    /// Unsubscribe from a resource URI.
    pub fn unsubscribe(&self, uri: &str) -> Result<(), String> {
        let uri = uri.trim();
        let mut subs = self.subscriptions.write();
        subs.remove(uri);
        Ok(())
    }

    /// Check if a specific URI is subscribed.
    pub fn is_subscribed(&self, uri: &str) -> bool {
        self.subscriptions.read().contains(uri.trim())
    }

    /// Count total active subscriptions.
    pub fn count(&self) -> usize {
        self.subscriptions.read().len()
    }

    /// List all currently subscribed URIs.
    pub fn subscribed_uris(&self) -> Vec<String> {
        self.subscriptions.read().iter().cloned().collect()
    }

    /// Clear all subscriptions.
    pub fn clear(&self) {
        self.subscriptions.write().clear();
    }

    /// Given an optional workspace and/or memory ID that changed,
    /// compute all subscribed URIs that should be invalidated or notified.
    pub fn match_affected_uris(
        &self,
        workspace: Option<&str>,
        memory_id: Option<i64>,
    ) -> Vec<String> {
        let subs = self.subscriptions.read();
        if subs.is_empty() {
            return Vec::new();
        }

        let mut affected = Vec::new();

        for sub in subs.iter() {
            let (path, _) = split_uri(sub);

            // Global stats are always affected by memory changes
            if path == "engram://stats" {
                affected.push(sub.clone());
                continue;
            }

            // Entities may change on memory insert/update/delete
            if path == "engram://entities" {
                affected.push(sub.clone());
                continue;
            }

            // Specific memory URI
            if let Some(target_id) = memory_id {
                if let Some(rest) = path.strip_prefix("engram://memory/") {
                    if let Ok(id) = rest.parse::<i64>() {
                        if id == target_id {
                            affected.push(sub.clone());
                            continue;
                        }
                    }
                }
            }

            // Workspace-level resources
            if let Some(ws) = workspace {
                if let Some(rest) = path.strip_prefix("engram://workspace/") {
                    let ws_name = rest.strip_suffix("/memories").unwrap_or(rest);
                    if ws_name == ws {
                        affected.push(sub.clone());
                        continue;
                    }
                }
            }
        }

        affected
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Split a URI into (path, query_string).
/// `engram://workspace/foo/memories?limit=10` → `("engram://workspace/foo/memories", Some("limit=10"))`
fn split_uri(uri: &str) -> (String, Option<String>) {
    match uri.find('?') {
        Some(pos) => (uri[..pos].to_string(), Some(uri[pos + 1..].to_string())),
        None => (uri.to_string(), None),
    }
}

/// Parse `limit` and `offset` from a query string of the form `key=value&key=value`.
fn parse_pagination(query: Option<&str>) -> (Option<i64>, Option<i64>) {
    let mut limit = None;
    let mut offset = None;

    if let Some(q) = query {
        for part in q.split('&') {
            if let Some((key, val)) = part.split_once('=') {
                match key {
                    "limit" => limit = val.parse().ok(),
                    "offset" => offset = val.parse().ok(),
                    _ => {}
                }
            }
        }
    }

    (limit, offset)
}

fn read_stats(storage: &Storage) -> Result<Value, String> {
    storage
        .with_connection(|conn| {
            let stats = get_stats(conn)?;
            Ok(json!(stats))
        })
        .map_err(|e| e.to_string())
}

fn read_entities(storage: &Storage) -> Result<Value, String> {
    storage
        .with_connection(|conn| {
            let entities = list_entities(conn, None, 100, 0)?;
            Ok(json!({
                "count": entities.len(),
                "entities": entities,
            }))
        })
        .map_err(|e| e.to_string())
}

fn read_memory(storage: &Storage, id: i64) -> Result<Value, String> {
    storage
        .with_connection(|conn| {
            let memory = get_memory(conn, id)?;
            Ok(json!(memory))
        })
        .map_err(|e| e.to_string())
}

fn read_workspace(storage: &Storage, name: &str) -> Result<Value, String> {
    storage
        .with_connection(|conn| {
            let stats = get_workspace_stats(conn, name)?;
            Ok(json!(stats))
        })
        .map_err(|e| e.to_string())
}

fn read_workspace_memories(
    storage: &Storage,
    name: &str,
    query: Option<&str>,
) -> Result<Value, String> {
    let (limit, offset) = parse_pagination(query);

    storage
        .with_connection(|conn| {
            let opts = ListOptions {
                workspace: Some(name.to_string()),
                limit: Some(limit.unwrap_or(50)),
                offset,
                ..Default::default()
            };
            let memories = list_memories(conn, &opts)?;
            Ok(json!({
                "workspace": name,
                "count": memories.len(),
                "limit": limit.unwrap_or(50),
                "offset": offset.unwrap_or(0),
                "memories": memories,
            }))
        })
        .map_err(|e| e.to_string())
}
