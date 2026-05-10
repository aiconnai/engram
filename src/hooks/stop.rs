// Stop hook handler
// Triggered when the agent stops (before session end)

use super::{HookContext, HookResult};
use crate::Result;

/// Handler for Stop hook
pub struct StopHandler;

impl StopHandler {
    pub fn handle(&self, _hook: super::LifecycleHook, context: &HookContext) -> Result<HookResult> {
        eprintln!(
            "[Hook] Stop: session_id={:?}, workspace={:?}",
            context.session_id, context.workspace
        );

        // TODO: Implement stop logic
        // - Finalize any pending operations
        // - Prepare session summary if needed
        // - Cleanup temporary resources

        Ok(HookResult::Continue)
    }
}

pub fn create_handler(
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = StopHandler;
        handler.handle(hook, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_stop_handler() {
        let handler = StopHandler;
        let context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        let result = handler.handle(crate::hooks::LifecycleHook::Stop, &context);
        assert!(result.is_ok());
    }
}
