// Lifecycle Hooks for Engram
// Inspired by claude-mem's hook system, adapted for Rust/Engram architecture.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle hooks that can be triggered during agent sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleHook {
    /// Triggered when a new session starts
    SessionStart,
    /// Triggered when user submits a prompt
    UserPromptSubmit,
    /// Triggered after a tool use completes
    PostToolUse,
    /// Triggered when the agent stops (before session end)
    Stop,
    /// Triggered when a session ends
    SessionEnd,
}

impl LifecycleHook {
    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleHook::SessionStart => "session_start",
            LifecycleHook::UserPromptSubmit => "user_prompt_submit",
            LifecycleHook::PostToolUse => "post_tool_use",
            LifecycleHook::Stop => "stop",
            LifecycleHook::SessionEnd => "session_end",
        }
    }
}

/// Context passed to hook handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub session_id: Option<String>,
    pub workspace: Option<String>,
    pub timestamp: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HookContext {
    pub fn new(session_id: Option<String>, workspace: Option<String>) -> Self {
        Self {
            session_id,
            workspace,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Result of hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookResult {
    /// Hook executed successfully, continue normal flow
    Continue,
    /// Hook requests modification of the input/output
    Modify(serde_json::Value),
    /// Hook requests to stop the current operation
    Abort { reason: String },
}

/// Hook manager that dispatches events to registered handlers
pub struct HookManager {
    handlers: HashMap<
        LifecycleHook,
        Vec<Box<dyn Fn(LifecycleHook, &HookContext) -> crate::Result<HookResult> + Send + Sync>>,
    >,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a specific hook
    pub fn register<F>(&mut self, hook: LifecycleHook, handler: F)
    where
        F: Fn(LifecycleHook, &HookContext) -> crate::Result<HookResult> + Send + Sync + 'static,
    {
        self.handlers
            .entry(hook)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    /// Trigger a hook and collect results
    pub fn trigger(
        &self,
        hook: LifecycleHook,
        context: &HookContext,
    ) -> crate::Result<Vec<HookResult>> {
        let mut results = Vec::new();

        if let Some(handlers) = self.handlers.get(&hook) {
            for handler in handlers {
                match handler(hook, context) {
                    Ok(HookResult::Abort { reason }) => {
                        return Ok(vec![HookResult::Abort { reason }]);
                    }
                    Ok(result) => results.push(result),
                    Err(e) => {
                        // Log error but continue with other handlers
                        eprintln!("Hook handler error for {:?}: {}", hook, e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Convenience method to trigger hook without collecting results
    pub fn trigger_and_forget(&self, hook: LifecycleHook, context: HookContext) {
        let _ = self.trigger(hook, &context);
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

// Module declarations for individual hook implementations
pub mod post_tool_use;
pub mod session_end;
pub mod session_start;
pub mod stop;

// Re-exports
pub use post_tool_use::PostToolUseHandler;
pub use session_end::SessionEndHandler;
pub use session_start::SessionStartHandler;
pub use stop::StopHandler;
