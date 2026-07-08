//! Tool discovery: list available tools by tier, category, or search query.

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

/// Level of per-tool detail returned by `discover_tools`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DiscoverDetail {
    /// Only the tool name.
    Names,
    /// Name, description, and tier (default; backward-compatible).
    Summary,
    /// Summary plus the full input schema as a JSON object.
    Schema,
}

/// List available tools by tier, category, or search query.
pub fn discover_tools(_ctx: &HandlerContext, params: Value) -> Value {
    use crate::mcp::tools::{
        catalog::{required_feature_summary, required_features, tool_group},
        iter_tool_definitions, tool_feature_available, ToolTier, TOOL_DEFINITIONS,
    };

    // Validate filters at the boundary: `as_str()` returns None both for a
    // missing key and for a wrong-typed value, so distinguish the two and
    // reject invalid input loudly instead of silently ignoring the filter.
    let tier_filter = match params.get("tier") {
        None => None,
        Some(value) => match value.as_str() {
            Some(t @ ("essential" | "standard" | "advanced" | "all")) => Some(t),
            Some(other) => {
                return json!({
                    "error": format!(
                        "invalid tier '{other}': expected one of 'essential', 'standard', 'advanced', 'all'"
                    )
                });
            }
            None => {
                return json!({
                    "error": "invalid tier type: expected string 'essential', 'standard', 'advanced', or 'all'"
                });
            }
        },
    };
    let group_filter = match params.get("group").or_else(|| params.get("category")) {
        None => None,
        Some(value) => match value.as_str() {
            Some(group) => Some(group),
            None => {
                return json!({
                    "error": "invalid group type: expected string group name"
                });
            }
        },
    };
    let search = match params.get("search") {
        None => None,
        Some(value) => match value.as_str() {
            Some(q) => Some(q),
            None => {
                return json!({
                    "error": "invalid search type: expected string query"
                });
            }
        },
    };

    // Validate `detail` at the boundary: reject unknown values loudly rather
    // than silently defaulting, so agents get clear feedback on typos.
    let detail = match params.get("detail") {
        None => DiscoverDetail::Summary,
        Some(value) => match value.as_str() {
            Some("summary") => DiscoverDetail::Summary,
            Some("names") => DiscoverDetail::Names,
            Some("schema") => DiscoverDetail::Schema,
            Some(other) => {
                return json!({
                    "error": format!(
                        "invalid detail '{other}': expected one of 'names', 'summary', 'schema'"
                    )
                });
            }
            None => {
                return json!({
                    "error": "invalid detail type: expected string value 'names', 'summary', or 'schema'"
                });
            }
        },
    };

    let tools: Vec<Value> = TOOL_DEFINITIONS
        .iter()
        .filter(|def| {
            if let Some(t) = tier_filter {
                match t {
                    "essential" if def.tier != ToolTier::Essential => {
                        return false;
                    }
                    "standard" if def.tier != ToolTier::Standard => {
                        return false;
                    }
                    "advanced" if def.tier != ToolTier::Advanced => {
                        return false;
                    }
                    _ => {}
                }
            }
            if let Some(group) = group_filter {
                let group_lower = group.to_lowercase();
                if !tool_group(def.name).contains(&group_lower)
                    && !def.name.contains(&group_lower)
                    && !def.description.to_lowercase().contains(&group_lower)
                {
                    return false;
                }
            }
            if let Some(q) = search {
                let q_lower = q.to_lowercase();
                if !def.name.to_lowercase().contains(&q_lower)
                    && !def.description.to_lowercase().contains(&q_lower)
                {
                    return false;
                }
            }
            true
        })
        .map(|def| {
            let tier_str = match def.tier {
                ToolTier::Essential => "essential",
                ToolTier::Standard => "standard",
                ToolTier::Advanced => "advanced",
            };
            let features = required_features(def.name);
            let feature = required_feature_summary(def.name);
            let available = tool_feature_available(def.name);
            let availability = if available {
                "available"
            } else {
                "feature_disabled"
            };
            let enable_with = (!features.is_empty())
                .then(|| format!("cargo build --features {}", features.join(",")));
            let base = json!({
                "name": def.name,
                "description": def.description,
                "tier": tier_str,
                "group": tool_group(def.name),
                "availability": availability,
                "feature": feature,
                "required_features": features,
                "enable_with": enable_with
            });
            match detail {
                DiscoverDetail::Names => json!({ "name": def.name }),
                DiscoverDetail::Summary => base,
                DiscoverDetail::Schema => {
                    let mut tool = base;
                    if let Value::Object(ref mut object) = tool {
                        object.insert(
                            "schema".to_string(),
                            serde_json::from_str::<Value>(def.schema).unwrap_or(json!({})),
                        );
                    }
                    tool
                }
            }
        })
        .collect();

    let essential_count = iter_tool_definitions()
        .filter(|d| d.tier == ToolTier::Essential)
        .count();
    let standard_count = iter_tool_definitions()
        .filter(|d| d.tier == ToolTier::Standard)
        .count();
    let advanced_count = iter_tool_definitions()
        .filter(|d| d.tier == ToolTier::Advanced)
        .count();
    let count = tools.len();

    json!({
        "tools": tools,
        "count": count,
        "total_available": iter_tool_definitions().count(),
        "total_defined": TOOL_DEFINITIONS.len(),
        "tier_summary": {
            "essential": essential_count,
            "standard": standard_count,
            "advanced": advanced_count
        }
    })
}
