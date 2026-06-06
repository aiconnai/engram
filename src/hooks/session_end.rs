//! SessionEnd hook handler.
//!
//! Producer side of the cross-session injection flow: when a session ends,
//! enqueue a small payload onto `pending_injections`. The next `SessionStart`
//! for the same workspace will drain it and shape it into an injection
//! prompt.
//!
//! The handler is intentionally minimal in this slice: it produces a
//! lightweight payload (session id + workspace + timestamp + opaque
//! `notes`) without doing memory search. Search-driven payload building
//! lives in a follow-up slice so we can ship the queue plumbing and the
//! hook wiring independently.

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{HookContext, HookResult, LifecycleHook};
use crate::storage::{
    enrichment_events::{self, EnrichmentEvent},
    pending_injections, Storage,
};
use crate::Result;

/// Default payload shape written to `pending_injections`. Tools that
/// consume the queue can parse this struct; future producers may write
/// richer payloads — the consumer should treat unknown fields as opaque
/// passthrough.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionEndPayload {
    pub kind: String,
    pub source_session_id: Option<String>,
    pub workspace: String,
    pub ended_at: String,
    pub notes: Option<String>,
}

pub struct SessionEndHandler {
    pub generate_summary: bool,
    /// Storage handle for enqueueing payloads. `None` keeps the handler
    /// observably inert — useful for tests of unrelated hooks and during
    /// the transition while the server wires this up.
    pub storage: Option<Storage>,
    /// Master switch for the cross-session injection feature. When false,
    /// the handler still logs the session end but does not write to the
    /// queue.
    pub auto_inject_next_session: bool,
    /// Emit policy audit summary without enqueueing a next-session payload.
    pub policy_summary_only: bool,
}

impl Default for SessionEndHandler {
    fn default() -> Self {
        Self {
            generate_summary: true,
            storage: None,
            auto_inject_next_session: false,
            policy_summary_only: false,
        }
    }
}

impl SessionEndHandler {
    pub fn new(storage: Storage) -> Self {
        Self {
            generate_summary: true,
            storage: Some(storage),
            auto_inject_next_session: true,
            policy_summary_only: false,
        }
    }

    pub fn policy_summary_only(storage: Storage) -> Self {
        Self {
            generate_summary: false,
            storage: Some(storage),
            auto_inject_next_session: false,
            policy_summary_only: true,
        }
    }

    pub fn handle(&self, _hook: LifecycleHook, context: &HookContext) -> Result<HookResult> {
        tracing::info!(
            target = "engram::hooks::session_end",
            session_id = ?context.session_id,
            workspace = ?context.workspace,
            "SessionEnd"
        );

        let Some(storage) = self.storage.as_ref() else {
            tracing::debug!(
                target = "engram::hooks::session_end",
                "auto_inject_next_session enabled but no storage handle attached"
            );
            return Ok(HookResult::Continue);
        };

        let Some(workspace) = context.workspace.as_deref() else {
            // Without a workspace we cannot route the payload to the
            // next session — drop silently rather than enqueue something
            // unaddressable.
            return Ok(HookResult::Continue);
        };

        if self.policy_summary_only {
            emit_session_end_policy_summary(storage, workspace, context, false);
            return Ok(HookResult::Continue);
        }

        if !self.auto_inject_next_session {
            return Ok(HookResult::Continue);
        }

        let payload = SessionEndPayload {
            kind: "session_end".to_string(),
            source_session_id: context.session_id.clone(),
            workspace: workspace.to_string(),
            ended_at: context.timestamp.clone(),
            notes: context
                .metadata
                .get("notes")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        };
        let payload_json = serde_json::to_string(&payload).unwrap_or_else(|_| {
            // Falling back to a hand-rolled JSON keeps the queue write
            // best-effort: a serialisation failure on a tiny struct is
            // not worth aborting session-end for.
            json!({
                "kind": "session_end",
                "workspace": workspace,
                "ended_at": context.timestamp,
            })
            .to_string()
        });

        let result = storage.with_connection(|conn| {
            pending_injections::enqueue(
                conn,
                workspace,
                &payload_json,
                context.session_id.as_deref(),
                None,
            )
        });
        let queued = match result {
            Ok(id) => {
                tracing::debug!(
                    target = "engram::hooks::session_end",
                    queue_id = id,
                    "enqueued session-end payload"
                );
                true
            }
            Err(e) => {
                tracing::warn!(
                    target = "engram::hooks::session_end",
                    error = %e,
                    "failed to enqueue session-end payload; continuing"
                );
                false
            }
        };

        emit_session_end_policy_summary(storage, workspace, context, queued);

        Ok(HookResult::Continue)
    }
}

fn emit_session_end_policy_summary(
    storage: &Storage,
    workspace: &str,
    context: &HookContext,
    queued_next_session_payload: bool,
) {
    let result = storage.with_connection(|conn| {
        let now = chrono::Utc::now();
        let operation_id = format!("memory-policy-session-end-{}", now.timestamp_micros());
        let event = EnrichmentEvent {
            operation_id: &operation_id,
            event_type: "memory_policy_session_end",
            memory_id: None,
            version_id: None,
            triggered_by: "session_end",
            agent_id: None,
            workspace: Some(workspace),
            params: json!({
                "source_session_id": context.session_id,
                "ended_at": context.timestamp,
                "has_notes": context.metadata.contains_key("notes")
            }),
            outcome: json!({
                "queued_next_session_payload": queued_next_session_payload,
                "hidden_facts_written": false
            }),
            status: "completed",
            dry_run: false,
        };
        enrichment_events::emit_best_effort(conn, &event);
        Ok(())
    });

    if let Err(e) = result {
        tracing::warn!(
            target = "engram::hooks::session_end",
            error = %e,
            "failed to emit session-end policy summary; continuing"
        );
    }
}

/// Build a stateless handler suitable for tests and code paths that do
/// not (yet) want cross-session injection wired up.
pub fn create_handler() -> impl Fn(LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync
{
    move |hook, context| SessionEndHandler::default().handle(hook, context)
}

/// Build a handler that owns a `Storage` clone and enqueues injection
/// payloads on every session end.
pub fn create_handler_with_storage(
    storage: Storage,
) -> impl Fn(LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| SessionEndHandler::new(storage.clone()).handle(hook, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::pending_injections::pending_count;
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
    fn default_handler_is_inert_for_queue() {
        let handler = SessionEndHandler::default();
        let c = ctx("s1", "default");
        let result = handler.handle(LifecycleHook::SessionEnd, &c);
        assert!(result.is_ok());
    }

    #[test]
    fn handler_with_storage_enqueues_payload() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = SessionEndHandler::new(storage.clone());
        let c = ctx("sess-42", "ws-a");

        handler
            .handle(LifecycleHook::SessionEnd, &c)
            .expect("handle");

        let count = storage
            .with_connection(|conn| pending_count(conn, "ws-a"))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn handler_skips_when_workspace_missing() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = SessionEndHandler::new(storage.clone());
        let c = HookContext {
            session_id: Some("orphan".to_string()),
            workspace: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        handler
            .handle(LifecycleHook::SessionEnd, &c)
            .expect("handle");

        // No workspace means no row should land anywhere.
        let count = storage
            .with_connection(|conn| {
                let n: i64 = conn
                    .query_row("SELECT COUNT(*) FROM pending_injections", [], |row| {
                        row.get(0)
                    })
                    .unwrap();
                Ok(n)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn handler_skips_when_feature_disabled() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = SessionEndHandler {
            storage: Some(storage.clone()),
            auto_inject_next_session: false,
            generate_summary: false,
            policy_summary_only: false,
        };
        let c = ctx("s1", "ws");

        handler
            .handle(LifecycleHook::SessionEnd, &c)
            .expect("handle");

        let count = storage
            .with_connection(|conn| pending_count(conn, "ws"))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn payload_serialises_with_notes() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = SessionEndHandler::new(storage.clone());
        let mut c = ctx("s1", "ws");
        c.metadata
            .insert("notes".to_string(), serde_json::json!("manual handoff"));

        handler
            .handle(LifecycleHook::SessionEnd, &c)
            .expect("handle");

        let drained = storage
            .with_connection(|conn| pending_injections::drain_for_workspace(conn, "ws"))
            .unwrap();
        assert_eq!(drained.len(), 1);
        let parsed: SessionEndPayload = serde_json::from_str(&drained[0].payload).unwrap();
        assert_eq!(parsed.source_session_id.as_deref(), Some("s1"));
        assert_eq!(parsed.notes.as_deref(), Some("manual handoff"));
    }

    #[test]
    fn handler_with_storage_emits_policy_summary_without_hidden_notes() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = SessionEndHandler::new(storage.clone());
        let mut c = ctx("s1", "ws");
        c.metadata.insert(
            "notes".to_string(),
            serde_json::json!("do not persist this as policy"),
        );

        handler
            .handle(LifecycleHook::SessionEnd, &c)
            .expect("handle");

        let (count, outcome): (i64, String) = storage
            .with_connection(|conn| {
                let row = conn.query_row(
                    "SELECT COUNT(*), COALESCE(MAX(outcome), '{}')
                     FROM enrichment_events
                     WHERE event_type = 'memory_policy_session_end'
                       AND memory_id IS NULL
                       AND workspace = 'ws'",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                Ok(row)
            })
            .unwrap();
        assert_eq!(count, 1);
        assert!(!outcome.contains("do not persist this as policy"));
        assert!(outcome.contains("hidden_facts_written"));
    }
}
