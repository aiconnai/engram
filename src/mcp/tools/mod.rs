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
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: &'static str,
    pub annotations: ToolAnnotations,
    pub tier: ToolTier,
}

pub(crate) mod catalog;
pub mod registry;

pub use registry::TOOL_DEFINITIONS;

/// Compile-time probe for every Cargo feature that gates an MCP tool.
///
/// `catalog::required_features` names the features each tool needs; this table
/// says which of them are compiled into the current build. Add a row here when
/// a new feature starts gating tools (a unit test fails otherwise).
const GATE_FEATURES: &[(&str, bool)] = &[
    ("langfuse", cfg!(feature = "langfuse")),
    ("meilisearch", cfg!(feature = "meilisearch")),
    ("emergent-graph", cfg!(feature = "emergent-graph")),
    ("multimodal", cfg!(feature = "multimodal")),
    ("cloud", cfg!(feature = "cloud")),
    ("duckdb-graph", cfg!(feature = "duckdb-graph")),
    ("dream-phase", cfg!(feature = "dream-phase")),
    ("attestation", cfg!(feature = "attestation")),
    ("snapshot", cfg!(feature = "snapshot")),
];

/// Whether a gate feature is compiled in. Unknown features fail closed.
fn gate_feature_compiled(feature: &str) -> bool {
    GATE_FEATURES
        .iter()
        .any(|(name, compiled)| *name == feature && *compiled)
}

/// Returns `false` for tool names whose feature flags are not all compiled in.
///
/// Derived from `catalog::required_features`, the single source of truth for
/// feature gating (also used by `discover_tools` and parsed by
/// `scripts/generate_mcp_reference.py`). Called by both `get_tool_definitions`
/// and `get_tool_definitions_tiered` so that `tools/list` never advertises a
/// tool that `tools/call` would reject with "Unknown tool".
pub(crate) fn tool_feature_available(name: &str) -> bool {
    catalog::required_features(name)
        .iter()
        .all(|feature| gate_feature_compiled(feature))
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
/// - `None` (default) → only Essential tools + discover_tools
/// - `Some("essential")` → only Essential tools + discover_tools
/// - `Some("standard")` → Essential + Standard tools + discover_tools
/// - `Some("advanced")` or `Some("all")` → all tools
pub fn get_tool_definitions_tiered(max_tier: Option<&str>) -> Vec<ToolDefinition> {
    let max = match max_tier {
        None | Some("essential") => ToolTier::Essential,
        Some("standard") => ToolTier::Standard,
        Some("advanced") | Some("all") => ToolTier::Advanced,
        Some(unknown) => {
            tracing::warn!(
                target: "engram::mcp::tools",
                tool_tier = %unknown,
                "unknown ENGRAM_TOOL_TIER; falling back to essential tool tier"
            );
            ToolTier::Essential
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
        assert!((12..=18).contains(&essential), "essential: {}", essential);
        assert!((120..=150).contains(&standard), "standard: {}", standard);
        // Advanced count depends on feature flags (feature-gated tools are Advanced)
        assert!(advanced >= 80, "advanced: {}", advanced);
        assert_eq!(essential + standard + advanced, TOOL_DEFINITIONS.len());
    }

    #[test]
    fn test_default_tier_excludes_advanced_tools() {
        for tier in [None, Some("advnaced")] {
            let tools = get_tool_definitions_tiered(tier);
            assert!(
                tools.iter().any(|t| t.name == "memory_digest"),
                "default tools/list should include essential tools"
            );
            assert!(
                tools.iter().all(|t| t.name != "memory_search"),
                "default tools/list should not expose standard tools"
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

    /// Regression guard for the Essential default surface: the default
    /// `tools/list` must stay small and curated. Any tool promoted into (or
    /// leaking out of) the default tier fails here, independent of which
    /// optional features are compiled in.
    #[test]
    fn test_default_tier_surface_regression() {
        let tools = get_tool_definitions_tiered(None);

        let expected = TOOL_DEFINITIONS
            .iter()
            .filter(|t| t.tier == ToolTier::Essential || t.name == "discover_tools")
            .count();
        assert_eq!(
            tools.len(),
            expected,
            "default surface must be exactly the Essential tier plus discover_tools"
        );

        for name in ["discover_tools", "memory_digest", "memory_create"] {
            assert!(
                tools.iter().any(|t| t.name == name),
                "default tools/list must include {name}"
            );
        }
        for name in [
            "memory_search",
            "memory_delete",
            "session_index",
            "identity_create",
        ] {
            assert!(
                tools.iter().all(|t| t.name != name),
                "default tools/list must not expose {name}"
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

        // Dispatch arm names: inside `pub fn dispatch`, up to the catch-all.
        let mut dispatch: BTreeSet<&str> = BTreeSet::new();
        for line in dispatch_source().lines() {
            let t = line.trim();
            if t.starts_with('"') && t.contains("=>") {
                if let Some(name) = t[1..].split('"').next() {
                    dispatch.insert(name);
                }
            }
        }

        // Registry names: from the aggregated tool definitions.
        let registry: BTreeSet<&str> = TOOL_DEFINITIONS.iter().map(|d| d.name).collect();

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

    /// Source text of the `dispatch` match, from `pub fn dispatch` up to the
    /// `tool_not_found` catch-all. Shared by the text-level parity guards.
    fn dispatch_source() -> &'static str {
        let mod_src = include_str!("../handlers/mod.rs");
        let start = mod_src
            .find("pub fn dispatch")
            .expect("dispatch fn present");
        let end = mod_src[start..]
            .find("tool_not_found")
            .map(|i| start + i)
            .expect("dispatch catch-all present");
        &mod_src[start..end]
    }

    /// Independent test oracle: is this gate feature compiled into the test
    /// build? Deliberately NOT shared with production code so a wrong
    /// `cfg!` mapping there is caught here.
    fn gate_feature_compiled_in_test_build(feature: &str) -> bool {
        match feature {
            "langfuse" => cfg!(feature = "langfuse"),
            "meilisearch" => cfg!(feature = "meilisearch"),
            "emergent-graph" => cfg!(feature = "emergent-graph"),
            "multimodal" => cfg!(feature = "multimodal"),
            "cloud" => cfg!(feature = "cloud"),
            "duckdb-graph" => cfg!(feature = "duckdb-graph"),
            "dream-phase" => cfg!(feature = "dream-phase"),
            "attestation" => cfg!(feature = "attestation"),
            "snapshot" => cfg!(feature = "snapshot"),
            other => panic!("test oracle does not know gate feature `{other}`"),
        }
    }

    /// `tool_feature_available` (what `tools/list` advertises) must be exactly
    /// "all `required_features` are compiled in". This is the contract that
    /// keeps `discover_tools`, `tools/list` and the generated reference in
    /// agreement; it regressed once when the two lists drifted (the dream
    /// status/insights tools were gated in dispatch but advertised as always
    /// available).
    #[test]
    fn test_feature_availability_follows_required_features() {
        for def in TOOL_DEFINITIONS.iter() {
            let expected = catalog::required_features(def.name)
                .iter()
                .all(|feature| gate_feature_compiled_in_test_build(feature));
            assert_eq!(
                tool_feature_available(def.name),
                expected,
                "tool_feature_available({}) disagrees with required_features {:?}",
                def.name,
                catalog::required_features(def.name)
            );
        }
    }

    /// Every `#[cfg(feature = ...)]`-gated dispatch arm must be listed in
    /// `required_features` with the same feature set, and no ungated arm may
    /// appear there. Otherwise a default build advertises (or documents) a
    /// tool that `tools/call` answers with `tool_not_found`.
    #[test]
    fn test_cfg_gated_dispatch_arms_match_required_features() {
        use std::collections::BTreeSet;

        let mut gated: Vec<(&str, BTreeSet<&str>)> = Vec::new();
        let mut ungated: Vec<&str> = Vec::new();
        let mut pending: Option<BTreeSet<&str>> = None;

        for line in dispatch_source().lines() {
            let t = line.trim();
            if t.starts_with("#[cfg(") && !t.contains("not(") {
                pending = Some(
                    t.split("feature = \"")
                        .skip(1)
                        .filter_map(|rest| rest.split('"').next())
                        .collect(),
                );
            } else if t.starts_with('"') && t.contains("=>") {
                let name = t[1..].split('"').next().expect("quoted arm name");
                match pending.take() {
                    Some(features) => gated.push((name, features)),
                    None => ungated.push(name),
                }
            } else if !t.is_empty() && !t.starts_with("//") {
                pending = None;
            }
        }

        assert!(
            gated.len() >= 40,
            "parsed too few gated arms: {}",
            gated.len()
        );
        for (name, features) in &gated {
            let declared: BTreeSet<&str> =
                catalog::required_features(name).iter().copied().collect();
            assert_eq!(
                &declared, features,
                "dispatch cfg gate for `{name}` disagrees with catalog::required_features"
            );
        }
        for name in ungated {
            assert!(
                catalog::required_features(name).is_empty(),
                "`{name}` is dispatched unconditionally but catalog::required_features gates it"
            );
        }
    }

    /// `required_features` may only name cataloged tools and probed gate
    /// features; a typo there would silently leave a tool ungated.
    #[test]
    fn test_required_features_names_are_cataloged_and_probed() {
        let src = include_str!("catalog/mod.rs");
        let start = src
            .find("pub(crate) fn required_features")
            .expect("required_features present");
        let end = src[start..]
            .find("pub(crate) fn required_feature(")
            .map(|i| start + i)
            .expect("required_feature present");
        let registry: std::collections::BTreeSet<&str> =
            TOOL_DEFINITIONS.iter().map(|d| d.name).collect();

        let mut names = 0;
        for quoted in src[start..end].split('"').skip(1).step_by(2) {
            if registry.contains(quoted) {
                names += 1;
            } else {
                // Not a tool: must be a probed gate feature, and the production
                // probe must agree with the independent oracle.
                let probe = GATE_FEATURES
                    .iter()
                    .find(|(name, _)| *name == quoted)
                    .unwrap_or_else(|| {
                        panic!("`{quoted}` in required_features is neither a cataloged tool nor in GATE_FEATURES")
                    });
                assert_eq!(
                    probe.1,
                    gate_feature_compiled_in_test_build(quoted),
                    "GATE_FEATURES probe for `{quoted}` disagrees with the cfg oracle"
                );
            }
        }
        assert!(names >= 40, "parsed too few gated tool names: {names}");
    }

    #[test]
    fn test_essential_tools_present() {
        let essential_names = [
            "memory_create",
            "context_seed",
            "memory_get",
            "memory_update",
            "memory_search_compact",
            "memory_expand",
            "memory_digest",
            "memory_get_injection_prompt",
            "memory_list_compact",
            "memory_smart_retrieve",
            "memory_agent_contract",
            "recent_activity",
            "workspace_list",
            "session_land",
            "discover_tools",
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
