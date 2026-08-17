//! MCP tool definition registry, aggregating modular domain catalogs.

use std::sync::LazyLock;

use super::catalog;
use super::ToolDef;

/// All tool definitions for Engram, aggregated across domain catalogs.
pub static TOOL_DEFINITIONS: LazyLock<Vec<ToolDef>> = LazyLock::new(|| {
    let mut tools = Vec::with_capacity(300);
    tools.extend_from_slice(catalog::memory_crud::TOOLS);
    tools.extend_from_slice(catalog::search::TOOLS);
    tools.extend_from_slice(catalog::context::TOOLS);
    tools.extend_from_slice(catalog::graph::TOOLS);
    tools.extend_from_slice(catalog::policy::TOOLS);
    tools.extend_from_slice(catalog::admin::TOOLS);
    tools.extend_from_slice(catalog::multimodal::TOOLS);
    tools.extend_from_slice(catalog::misc::TOOLS);
    tools
});
