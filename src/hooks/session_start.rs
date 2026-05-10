// SessionStart hook handler
// Triggered when a new agent session starts.

use super::{HookContext, HookResult};
use crate::Result;

/// Handler for SessionStart hook
pub struct SessionStartHandler;

impl SessionStartHandler {
    pub fn handle(&self, _hook: super::LifecycleHook, context: &HookContext) -> Result<HookResult> {
        // Log session start
        eprintln!(
            "[Hook] SessionStart: session_id={:?}, workspace={:?}",
            context.session_id, context.workspace
        );

        // TODO: Implement session initialization logic
        // - Load session context from storage
        // - Prepare injection prompt for the session
        // - Initialize session-specific state

        Ok(HookResult::Continue)
    }
}

pub fn create_handler(
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = SessionStartHandler;
        handler.handle(hook, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_session_start_handler() {
        let handler = SessionStartHandler;
        let context = HookContext {
            session_id: Some("test-session-123".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        let result = handler.handle(crate::hooks::LifecycleHook::SessionStart, &context);
        assert!(result.is_ok());
    }
}
