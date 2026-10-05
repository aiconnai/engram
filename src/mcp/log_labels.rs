//! Bounded label values for logs and counters (task O1).
//!
//! JSON-RPC methods and tool names arrive from the caller. Logging them raw
//! would let a client put arbitrary text (or an unbounded label set) into logs
//! and metrics, so only names this server actually defines are passed through.

use super::protocol::methods;
use super::tools::TOOL_DEFINITIONS;

const KNOWN_METHODS: &[&str] = &[
    methods::INITIALIZE,
    methods::INITIALIZED,
    methods::LIST_TOOLS,
    methods::CALL_TOOL,
    methods::LIST_RESOURCES,
    methods::READ_RESOURCE,
    methods::SUBSCRIBE_RESOURCE,
    methods::UNSUBSCRIBE_RESOURCE,
    methods::LIST_PROMPTS,
    methods::GET_PROMPT,
    methods::PROGRESS,
    methods::NOTIFY_RESOURCE_UPDATED,
    methods::NOTIFY_RESOURCE_LIST_CHANGED,
];

/// The JSON-RPC method when it is one this server defines, else `"other"`.
pub fn bounded_method(method: &str) -> &'static str {
    KNOWN_METHODS
        .iter()
        .copied()
        .find(|known| *known == method)
        .unwrap_or("other")
}

/// The tool name when it is in the catalog, else `"unknown"`.
pub fn known_tool(name: &str) -> &'static str {
    TOOL_DEFINITIONS
        .iter()
        .map(|def| def.name)
        .find(|known| *known == name)
        .unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_supplied_names_are_bounded() {
        assert_eq!(bounded_method("tools/call"), "tools/call");
        assert_eq!(bounded_method("tools/call\nINJECTED"), "other");
        assert_eq!(known_tool("memory_search"), "memory_search");
        assert_eq!(known_tool("secret-tool-name-from-caller"), "unknown");
    }
}
