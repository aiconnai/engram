// SessionEnd hook handler
// Triggered when a session ends - generates summaries and finalizes session.

use super::{HookContext, HookResult};
use crate::Result;

/// Handler for SessionEnd hook
/// Generates session summaries and persists session data
pub struct SessionEndHandler {
    /// Whether to generate AI-powered session summaries
    pub generate_summary: bool,
}

impl Default for SessionEndHandler {
    fn default() -> Self {
        Self {
            generate_summary: true,
        }
    }
}

impl SessionEndHandler {
    pub fn handle(
        &self,
        _hook: super::LifecycleHook,
        context: &HookContext,
    ) -> Result<HookResult> {
        eprintln!(
            "[Hook] SessionEnd: session_id={:?}, workspace={:?}",
            context.session_id, context.workspace
        );

        // TODO: Implement session end logic
        // - Generate session summary using AI
        // - Persist session memories
        // - Update session statistics
        // - Cleanup session resources.

        if self.generate_summary {
            eprintln!(
                "[Hook] Would generate session summary for session: {:?}",
                context.session_id
            );
            // TODO: Call intelligence layer to generate summary
        }

        Ok(HookResult::Continue)
    }
}

pub fn create_handler(
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = SessionEndHandler::default();
        handler.handle(hook, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_session_end_handler() {
        let handler = SessionEndHandler::default();
        let context = HookContext {
            session_id: Some("test-session-123".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        let result = handler.handle(crate::hooks::LifecycleHook::SessionEnd, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_end_without_summary() {
        let handler = SessionEndHandler {
            generate_summary: false,
        };
        let context = HookContext {
            session_id: Some("test-session-456".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        let result = handler.handle(crate::hooks::LifecycleHook::SessionEnd, &context);
        assert!(result.is_ok());
    }
}
