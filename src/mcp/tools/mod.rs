// MCP tool definitions for Engram

use serde_json::json;

use super::protocol::{ToolAnnotations, ToolDefinition};

/// Tool exposure tier for progressive discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTier {
    /// Core tools every agent needs. Always exposed.
    Essential,
    /// Common tools for standard workflows.
    Standard,
    /// Advanced/specialized tools.
    Advanced,
}

/// Structured tool definition with MCP 2025-11-25 annotations.
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: &'static str,
    pub annotations: ToolAnnotations,
    pub tier: ToolTier,
}

/// All tool definitions for Engram
pub const TOOL_DEFINITIONS: &[ToolDef] = include!("registry.rs");

/// Returns `false` for tool names whose feature flag is not compiled in.
///
/// Called by both `get_tool_definitions` and `get_tool_definitions_tiered` so
/// that `tools/list` never advertises a tool that `tools/call` would reject with
/// "Unknown tool".
fn tool_feature_available(name: &str) -> bool {
    match name {
        // langfuse feature
        "langfuse_connect"
        | "langfuse_sync"
        | "langfuse_sync_status"
        | "langfuse_extract_patterns"
        | "memory_from_trace" => cfg!(feature = "langfuse"),

        // meilisearch feature
        "meilisearch_search"
        | "meilisearch_reindex"
        | "meilisearch_status"
        | "meilisearch_config" => cfg!(feature = "meilisearch"),

        // emergent-graph feature
        "memory_auto_link"
        | "memory_list_auto_links"
        | "memory_auto_link_stats"
        | "memory_cluster"
        | "memory_get_cluster"
        | "memory_list_clusters" => cfg!(feature = "emergent-graph"),

        // multimodal feature (memory_sync_media also needs cloud)
        "memory_sync_media" => cfg!(all(feature = "multimodal", feature = "cloud")),
        "memory_describe_image"
        | "memory_transcribe_audio"
        | "memory_capture_screenshot"
        | "memory_process_video"
        | "memory_list_media"
        | "memory_search_by_image" => cfg!(feature = "multimodal"),

        // duckdb-graph feature
        "memory_graph_path" | "memory_temporal_snapshot" | "memory_scope_snapshot" => {
            cfg!(feature = "duckdb-graph")
        }

        // dream-phase feature
        "dream_run_now"
        | "dream_create"
        | "dream_get"
        | "dream_list"
        | "dream_cancel"
        | "dream_archive"
        | "dream_candidates_list"
        | "dream_candidate_get"
        | "dream_candidate_review"
        | "dream_candidate_apply"
        | "dream_eval_run" => cfg!(feature = "dream-phase"),

        // agent-portability feature
        "attestation_log"
        | "attestation_verify"
        | "attestation_chain_verify"
        | "attestation_list"
        | "snapshot_create"
        | "snapshot_load"
        | "snapshot_inspect" => cfg!(feature = "agent-portability"),

        _ => true,
    }
}

/// Get all tool definitions as ToolDefinition structs.
///
/// Only returns tools whose feature flag is compiled in so that `tools/list`
/// never advertises a tool that `tools/call` would reject.
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    iter_tool_definitions()
        .map(|def| ToolDefinition {
            name: def.name.to_string(),
            description: def.description.to_string(),
            input_schema: serde_json::from_str(def.schema).unwrap_or(json!({})),
            annotations: Some(def.annotations.clone()),
        })
        .collect()
}

/// Get tool definitions filtered by maximum tier level.
///
/// - `Some("essential")` → only Essential tools + discover_tools
/// - `Some("standard")` → Essential + Standard tools + discover_tools
/// - `None` → Essential + Standard tools + discover_tools
/// - `Some("advanced")` or `Some("all")` → all tools
pub fn get_tool_definitions_tiered(max_tier: Option<&str>) -> Vec<ToolDefinition> {
    let max = match max_tier {
        Some("essential") => ToolTier::Essential,
        Some("standard") | None => ToolTier::Standard,
        Some("advanced") | Some("all") => ToolTier::Advanced,
        Some(unknown) => {
            tracing::warn!(
                target: "engram::mcp::tools",
                tool_tier = %unknown,
                "unknown ENGRAM_TOOL_TIER; falling back to standard tool tier"
            );
            ToolTier::Standard
        }
    };

    iter_tool_definitions()
        .filter(|def| {
            // discover_tools is always included regardless of tier
            if def.name == "discover_tools" {
                return true;
            }
            // Tier filtering
            matches!(
                (max, def.tier),
                (ToolTier::Essential, ToolTier::Essential)
                    | (ToolTier::Standard, ToolTier::Essential | ToolTier::Standard)
                    | (ToolTier::Advanced, _)
            )
        })
        .map(|def| ToolDefinition {
            name: def.name.to_string(),
            description: def.description.to_string(),
            input_schema: serde_json::from_str(def.schema).unwrap_or(json!({})),
            annotations: Some(def.annotations.clone()),
        })
        .collect()
}

pub(crate) fn iter_tool_definitions() -> impl Iterator<Item = &'static ToolDef> {
    TOOL_DEFINITIONS
        .iter()
        .filter(|def| tool_feature_available(def.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions_all_parseable() {
        let tools = get_tool_definitions();
        assert!(!tools.is_empty(), "TOOL_DEFINITIONS must not be empty");
        for tool in &tools {
            assert!(!tool.name.is_empty(), "tool name must not be empty");
            assert!(
                !tool.description.is_empty(),
                "tool description must not be empty"
            );
            assert!(
                tool.input_schema.is_object(),
                "tool '{}' schema must be a JSON object",
                tool.name
            );
        }
    }

    #[test]
    fn test_read_only_tools_have_annotation() {
        let tools = get_tool_definitions();
        let read_only_names = ["memory_get", "memory_list", "memory_search", "memory_stats"];
        for name in read_only_names {
            let tool = tools
                .iter()
                .find(|t| t.name == name)
                .unwrap_or_else(|| panic!("tool '{}' not found", name));
            let ann = tool
                .annotations
                .as_ref()
                .expect("annotations must be present");
            assert_eq!(
                ann.read_only_hint,
                Some(true),
                "tool '{}' should have readOnlyHint=true",
                name
            );
        }
    }

    #[test]
    fn test_destructive_tools_have_annotation() {
        let tools = get_tool_definitions();
        let destructive_names = [
            "memory_delete",
            "memory_cleanup_expired",
            "embedding_cache_clear",
        ];
        for name in destructive_names {
            let tool = tools
                .iter()
                .find(|t| t.name == name)
                .unwrap_or_else(|| panic!("tool '{}' not found", name));
            let ann = tool
                .annotations
                .as_ref()
                .expect("annotations must be present");
            assert_eq!(
                ann.destructive_hint,
                Some(true),
                "tool '{}' should have destructiveHint=true",
                name
            );
        }
    }

    #[test]
    fn test_idempotent_tools_have_annotation() {
        let tools = get_tool_definitions();
        let idempotent_names = [
            "memory_extract_entities",
            "memory_rebuild_embeddings",
            "memory_rebuild_crossrefs",
            "lifecycle_run",
            "retention_policy_apply",
        ];
        for name in idempotent_names {
            let tool = tools
                .iter()
                .find(|t| t.name == name)
                .unwrap_or_else(|| panic!("tool '{}' not found", name));
            let ann = tool
                .annotations
                .as_ref()
                .expect("annotations must be present");
            assert_eq!(
                ann.idempotent_hint,
                Some(true),
                "tool '{}' should have idempotentHint=true",
                name
            );
        }
    }

    #[test]
    fn test_tier_distribution() {
        let essential = TOOL_DEFINITIONS
            .iter()
            .filter(|t| t.tier == ToolTier::Essential)
            .count();
        let standard = TOOL_DEFINITIONS
            .iter()
            .filter(|t| t.tier == ToolTier::Standard)
            .count();
        let advanced = TOOL_DEFINITIONS
            .iter()
            .filter(|t| t.tier == ToolTier::Advanced)
            .count();
        // Ranges widened after the dispatch/registry reconciliation that
        // advertised ~59 previously-hidden tools (registry grew to ~250).
        assert!((18..=25).contains(&essential), "essential: {}", essential);
        assert!((60..=130).contains(&standard), "standard: {}", standard);
        // Advanced count depends on feature flags (feature-gated tools are Advanced)
        assert!(advanced >= 80, "advanced: {}", advanced);
        assert_eq!(essential + standard + advanced, TOOL_DEFINITIONS.len());
    }

    #[test]
    fn test_default_tier_excludes_advanced_tools() {
        for tier in [None, Some("advnaced")] {
            let tools = get_tool_definitions_tiered(tier);
            assert!(
                tools.iter().any(|t| t.name == "memory_search"),
                "default tools/list should include essential tools"
            );
            assert!(
                tools.iter().any(|t| t.name == "quality_report"),
                "default tools/list should include standard tools"
            );
            assert!(
                tools.iter().any(|t| t.name == "discover_tools"),
                "default tools/list should always include discover_tools"
            );
            assert!(
                tools.iter().all(|t| t.name != "memory_archive_old"),
                "default tools/list should not expose advanced tools"
            );
        }
    }

    #[test]
    fn test_advanced_tiers_include_advanced_tools() {
        for tier in ["all", "advanced"] {
            let tools = get_tool_definitions_tiered(Some(tier));
            assert!(
                tools.iter().any(|t| t.name == "memory_archive_old"),
                "ENGRAM_TOOL_TIER={tier} should expose advanced tools"
            );
        }
    }

    /// Guard against the dispatch/registry drift bug class: every tool routed
    /// in `handlers::dispatch` must have a `TOOL_DEFINITIONS` entry, and every
    /// advertised tool must be routable. A new dispatch arm without a registry
    /// entry (or vice-versa) fails CI here.
    ///
    /// Text-level + feature-independent: both sources are parsed as text, so
    /// `#[cfg]`-gated arms and entries appear in both and cancel out.
    #[test]
    fn test_dispatch_registry_parity() {
        use std::collections::BTreeSet;

        let mod_src = include_str!("../handlers/mod.rs");

        // Dispatch arm names: inside `pub fn dispatch`, up to the catch-all.
        let d_start = mod_src
            .find("pub fn dispatch")
            .expect("dispatch fn present");
        let d_end = mod_src[d_start..]
            .find(r#"_ => json!({"error": format!("Unknown tool"#)
            .map(|i| d_start + i)
            .expect("dispatch catch-all present");
        let mut dispatch: BTreeSet<&str> = BTreeSet::new();
        for line in mod_src[d_start..d_end].lines() {
            let t = line.trim();
            if t.starts_with('"') && t.contains("=>") {
                if let Some(name) = t[1..].split('"').next() {
                    dispatch.insert(name);
                }
            }
        }

        // Registry names: from the composed source, mirroring old behavior for cfg-gated tools.
        let registry_src = include_str!("registry.rs");
        let mut registry: BTreeSet<&str> = BTreeSet::new();
        for line in registry_src.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("name:") {
                if let Some(inner) = rest.trim().strip_prefix('"') {
                    if let Some(name) = inner.split('"').next() {
                        registry.insert(name);
                    }
                }
            }
        }

        // Legitimate asymmetries — keep empty; document any future addition.
        const DISPATCH_ONLY_OK: &[&str] = &[];
        const REGISTRY_ONLY_OK: &[&str] = &[];

        // Sanity: the parser actually found the tables.
        assert!(
            dispatch.len() > 100,
            "parsed too few dispatch arms: {}",
            dispatch.len()
        );
        assert!(
            registry.len() > 100,
            "parsed too few registry defs: {}",
            registry.len()
        );

        let d_only: Vec<&str> = dispatch
            .difference(&registry)
            .filter(|n| !DISPATCH_ONLY_OK.contains(n))
            .copied()
            .collect();
        let r_only: Vec<&str> = registry
            .difference(&dispatch)
            .filter(|n| !REGISTRY_ONLY_OK.contains(n))
            .copied()
            .collect();

        assert!(
            d_only.is_empty(),
            "Tools CALLABLE but NOT advertised — add a TOOL_DEFINITIONS entry: {:?}",
            d_only
        );
        assert!(
            r_only.is_empty(),
            "Tools ADVERTISED but NOT callable — add a dispatch arm or remove from registry: {:?}",
            r_only
        );
    }

    #[test]
    fn test_essential_tools_present() {
        let essential_names = [
            "memory_create",
            "context_seed",
            "memory_get",
            "memory_update",
            "memory_delete",
            "memory_list",
            "memory_search",
            "memory_search_compact",
            "memory_expand",
            "memory_get_injection_prompt",
            "memory_link",
            "memory_related",
            "memory_traverse",
            "memory_stats",
            "workspace_list",
            "session_index",
            "session_list",
            "identity_create",
            "identity_resolve",
        ];
        for name in essential_names {
            let tool = TOOL_DEFINITIONS
                .iter()
                .find(|t| t.name == name)
                .unwrap_or_else(|| panic!("tool '{}' not found", name));
            assert_eq!(
                tool.tier,
                ToolTier::Essential,
                "tool '{}' should be Essential",
                name
            );
        }
    }

    #[test]
    fn test_annotations_serialize_with_camel_case_keys() {
        let tools = get_tool_definitions();
        let memory_get = tools.iter().find(|t| t.name == "memory_get").unwrap();
        let json = serde_json::to_string(memory_get).expect("serialization must succeed");
        assert!(
            json.contains("readOnlyHint"),
            "should serialize as readOnlyHint"
        );
        assert!(
            !json.contains("read_only_hint"),
            "must not use snake_case key"
        );
    }

    #[test]
    fn test_recent_activity_tool_is_defined_as_read_only() {
        let tools = get_tool_definitions();
        let tool = tools
            .iter()
            .find(|t| t.name == "recent_activity")
            .expect("recent_activity tool must be defined");
        let ann = tool
            .annotations
            .as_ref()
            .expect("annotations must be present");
        assert_eq!(
            ann.read_only_hint,
            Some(true),
            "recent_activity should have readOnlyHint=true"
        );
    }

    #[test]
    fn test_mutating_tool_has_no_hints() {
        let tools = get_tool_definitions();
        let memory_create = tools.iter().find(|t| t.name == "memory_create").unwrap();
        let ann = memory_create
            .annotations
            .as_ref()
            .expect("annotations must be present");
        assert!(ann.read_only_hint.is_none());
        assert!(ann.destructive_hint.is_none());
        assert!(ann.idempotent_hint.is_none());
        // Serialized form should omit None fields
        let json = serde_json::to_string(ann).expect("serialization must succeed");
        // mutating annotations serialize as empty object since all fields are None
        assert_eq!(json, "{}");
    }
}
