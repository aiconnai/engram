// PostToolUse hook handler
// Triggered after a tool use completes - inspired by claude-mem's automatic observation capture

use super::{HookContext, HookResult};
use crate::Result;

/// Handler for PostToolUse hook
/// Automatically captures tool usage and creates memory observations
pub struct PostToolUseHandler {
    /// Whether to automatically create memories from tool outputs
    pub auto_memory: bool,
}

impl Default for PostToolUseHandler {
    fn default() -> Self {
        Self { auto_memory: true }
    }
}

impl PostToolUseHandler {
    pub fn handle(&self, _hook: super::LifecycleHook, context: &HookContext) -> Result<HookResult> {
        eprintln!(
            "[Hook] PostToolUse: tool={:?}",
            context.metadata.get("tool_name")
        );

        // If auto-memory is enabled, create a memory from tool output
        if self.auto_memory {
            if let Some(tool_name) = context.metadata.get("tool_name") {
                if let Some(_tool_output) = context.metadata.get("tool_output") {
                    // TODO: Call storage layer to create memory
                    // This would integrate with engram's memory creation flow
                    eprintln!(
                        "[Hook] Would create memory for tool: {}",
                        tool_name.as_str().unwrap_or("unknown")
                    );
                }
            }
        }

        Ok(HookResult::Continue)
    }
}

pub fn create_handler(
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = PostToolUseHandler::default();
        handler.handle(hook, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_post_tool_use_handler() {
        let handler = PostToolUseHandler::default();
        let mut context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        context
            .metadata
            .insert("tool_name".to_string(), json!("memory_create"));
        context
            .metadata
            .insert("tool_output".to_string(), json!({"status": "success"}));

        let result = handler.handle(crate::hooks::LifecycleHook::PostToolUse, &context);
        assert!(result.is_ok());
    }
}
