//! SessionStart hook handler.
//!
//! Consumer side of the cross-session injection flow. When a session
//! starts in a workspace that has queued payloads, drain them oldest-first
//! and emit an injection prompt via `HookResult::Modify`. The handler
//! never persists anything itself — drain is atomic at the storage layer.

use serde_json::{json, Value};

use super::{HookContext, HookResult, LifecycleHook};
use crate::hooks::session_end::SessionEndPayload;
use crate::storage::{pending_injections, Storage};
use crate::Result;

pub struct SessionStartHandler {
    pub storage: Option<Storage>,
    pub auto_inject: bool,
}

impl Default for SessionStartHandler {
    fn default() -> Self {
        Self {
            storage: None,
            auto_inject: false,
        }
    }
}

impl SessionStartHandler {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Some(storage),
            auto_inject: true,
        }
    }

    pub fn handle(&self, _hook: LifecycleHook, context: &HookContext) -> Result<HookResult> {
        tracing::info!(
            target = "engram::hooks::session_start",
            session_id = ?context.session_id,
            workspace = ?context.workspace,
            "SessionStart"
        );

        if !self.auto_inject {
            return Ok(HookResult::Continue);
        }
        let Some(storage) = self.storage.as_ref() else {
            return Ok(HookResult::Continue);
        };
        let Some(workspace) = context.workspace.as_deref() else {
            // Without a workspace we cannot pick which queue to drain.
            return Ok(HookResult::Continue);
        };

        let drained = match storage
            .with_connection(|conn| pending_injections::drain_for_workspace(conn, workspace))
        {
            Ok(items) => items,
            Err(e) => {
                tracing::warn!(
                    target = "engram::hooks::session_start",
                    error = %e,
                    "failed to drain pending_injections; continuing"
                );
                return Ok(HookResult::Continue);
            }
        };
        if drained.is_empty() {
            return Ok(HookResult::Continue);
        }

        let injection_value = build_injection(workspace, &drained);
        Ok(HookResult::Modify(injection_value))
    }
}

/// Render the drained queue rows into an injection payload that the
/// caller (the MCP server hook plumbing) can hand to whatever rendering
/// path applies. Kept as structured JSON rather than a pre-rendered
/// string so the consumer can choose a Markdown/plain/structured shape.
fn build_injection(workspace: &str, items: &[pending_injections::PendingInjection]) -> Value {
    let parsed: Vec<Value> = items
        .iter()
        .map(|row| {
            match serde_json::from_str::<SessionEndPayload>(&row.payload) {
                Ok(p) => json!({
                    "queue_id": row.id,
                    "source_session_id": p.source_session_id,
                    "ended_at": p.ended_at,
                    "notes": p.notes,
                }),
                Err(_) => {
                    // Unknown payload shape — surface raw so the next
                    // session at least sees it; do not drop signal.
                    json!({
                        "queue_id": row.id,
                        "raw_payload": row.payload,
                    })
                }
            }
        })
        .collect();

    json!({
        "kind": "session_start_injection",
        "workspace": workspace,
        "count": parsed.len(),
        "items": parsed,
    })
}

pub fn create_handler(
) -> impl Fn(LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| SessionStartHandler::default().handle(hook, context)
}

pub fn create_handler_with_storage(
    storage: Storage,
) -> impl Fn(LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| SessionStartHandler::new(storage.clone()).handle(hook, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::session_end::SessionEndHandler;
    use std::collections::HashMap;

    fn ctx(session: &str, workspace: &str) -> HookContext {
        HookContext {
            session_id: Some(session.to_string()),
            workspace: Some(workspace.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn default_handler_is_inert() {
        let handler = SessionStartHandler::default();
        let result = handler.handle(LifecycleHook::SessionStart, &ctx("s1", "ws"));
        assert!(matches!(result, Ok(HookResult::Continue)));
    }

    #[test]
    fn empty_queue_returns_continue() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = SessionStartHandler::new(storage);
        let result = handler.handle(LifecycleHook::SessionStart, &ctx("s1", "ws"));
        assert!(matches!(result, Ok(HookResult::Continue)));
    }

    #[test]
    fn end_to_start_round_trip_yields_modify() {
        let storage = Storage::open_in_memory().unwrap();

        // Producer side: SessionEnd handler enqueues a row.
        let end_handler = SessionEndHandler::new(storage.clone());
        end_handler
            .handle(LifecycleHook::SessionEnd, &ctx("prev-session", "ws-x"))
            .unwrap();

        // Consumer side: SessionStart for the same workspace drains it.
        let start_handler = SessionStartHandler::new(storage.clone());
        let result = start_handler
            .handle(LifecycleHook::SessionStart, &ctx("next-session", "ws-x"))
            .unwrap();

        match result {
            HookResult::Modify(value) => {
                assert_eq!(value["kind"], "session_start_injection");
                assert_eq!(value["count"], 1);
                assert_eq!(value["items"][0]["source_session_id"], "prev-session");
            }
            other => panic!("expected Modify, got {:?}", other),
        }

        // Queue is empty after the drain.
        let leftover = storage
            .with_connection(|conn| pending_injections::pending_count(conn, "ws-x"))
            .unwrap();
        assert_eq!(leftover, 0);
    }

    #[test]
    fn drain_respects_workspace_routing() {
        let storage = Storage::open_in_memory().unwrap();
        // Producer enqueues for ws-a.
        SessionEndHandler::new(storage.clone())
            .handle(LifecycleHook::SessionEnd, &ctx("s1", "ws-a"))
            .unwrap();
        // Consumer started in ws-b sees nothing.
        let handler = SessionStartHandler::new(storage.clone());
        let result = handler
            .handle(LifecycleHook::SessionStart, &ctx("s2", "ws-b"))
            .unwrap();
        assert!(matches!(result, HookResult::Continue));
        // The ws-a row is still there.
        let n = storage
            .with_connection(|conn| pending_injections::pending_count(conn, "ws-a"))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn drain_returns_items_in_fifo_order() {
        let storage = Storage::open_in_memory().unwrap();
        SessionEndHandler::new(storage.clone())
            .handle(LifecycleHook::SessionEnd, &ctx("a", "ws"))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        SessionEndHandler::new(storage.clone())
            .handle(LifecycleHook::SessionEnd, &ctx("b", "ws"))
            .unwrap();

        let result = SessionStartHandler::new(storage.clone())
            .handle(LifecycleHook::SessionStart, &ctx("next", "ws"))
            .unwrap();
        match result {
            HookResult::Modify(v) => {
                assert_eq!(v["count"], 2);
                assert_eq!(v["items"][0]["source_session_id"], "a");
                assert_eq!(v["items"][1]["source_session_id"], "b");
            }
            other => panic!("expected Modify, got {:?}", other),
        }
    }
}
