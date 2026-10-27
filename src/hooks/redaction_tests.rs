//! Hook logs must not carry the workspace name (free-form, often a project
//! path or customer name).

use super::{HookContext, LifecycleHook, SessionStartHandler};
use crate::observability::test_capture::{assert_logs_exclude, captured, install};
use crate::storage::Storage;

#[test]
fn session_start_log_omits_workspace_name() {
    install();
    const WORKSPACE: &str = "workspace-sentinel-wt12-5d20";
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let handler = SessionStartHandler::new(storage);
    let context = HookContext::new(
        Some("session-wt12".to_string()),
        Some(WORKSPACE.to_string()),
    );

    handler
        .handle(LifecycleHook::SessionStart, &context)
        .expect("session start handles an empty queue");

    assert!(captured().contains("SessionStart"));
    assert_logs_exclude(&[WORKSPACE]);
}
