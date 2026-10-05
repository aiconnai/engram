//! Fuzz target: workspace / scope identifier boundary handling.
//!
//! The engine has no public `normalize_workspace` function (the only one is
//! private to the writeback-plan handler), so this target covers the public
//! surface that interprets caller-supplied workspace and scope identifiers:
//!
//! - `storage::scoping::MemoryScope::{parse, new, parent, ancestors, contains}`
//! - `auth::TransportPrincipal::allows_workspace` (anonymous / namespaced /
//!   unscoped principals)
//! - `mcp::permission::extract_requested_scopes` and
//!   `mcp::workspace_guard::memory_id_arguments` on arbitrary JSON.
#![no_main]

use engram::auth::{AuthContext, PermissionSet, TransportPrincipal, UserId};
use engram::mcp::permission::extract_requested_scopes;
use engram::mcp::workspace_guard::memory_id_arguments;
use engram::storage::scoping::{MemoryScope, ScopeLevel};
use libfuzzer_sys::fuzz_target;

const LEVELS: [ScopeLevel; 5] = [
    ScopeLevel::Global,
    ScopeLevel::Org,
    ScopeLevel::User,
    ScopeLevel::Session,
    ScopeLevel::Agent,
];

fn check_scope(scope: &MemoryScope) {
    assert!(scope.contains(scope), "contains is reflexive");
    let ancestors = scope.ancestors();
    assert_eq!(ancestors.len(), scope.level as usize);
    let mut previous_level = scope.level;
    for ancestor in &ancestors {
        assert!(ancestor.level < previous_level, "levels strictly coarser");
        previous_level = ancestor.level;
        assert!(ancestor.contains(scope), "ancestor must contain descendant");
        assert!(
            scope.path.starts_with(&ancestor.path),
            "ancestor path is a prefix"
        );
    }
}

// Input layout: byte 0 = selector, rest = `workspace` bytes, an optional 0xFF
// separator (never valid UTF-8), then `other` bytes. Invalid UTF-8 is replaced.
fuzz_target!(|data: &[u8]| {
    let [selector, rest @ ..] = data else {
        return;
    };
    let selector = *selector;
    let (workspace_bytes, other_bytes) = match rest.iter().position(|b| *b == 0xFF) {
        Some(split) => (&rest[..split], &rest[split + 1..]),
        None => (rest, &rest[..0]),
    };
    let workspace = String::from_utf8_lossy(workspace_bytes);
    let other = String::from_utf8_lossy(other_bytes);
    let (workspace, other) = (workspace.as_ref(), other.as_ref());

    // Scope paths: `parse` infers the level, `new` validates a declared one.
    if let Ok(scope) = MemoryScope::parse(workspace) {
        assert_eq!(scope.path, workspace, "parse must not rewrite the path");
        assert_eq!(scope.path.split('/').count(), scope.level as usize + 1);
        assert!(MemoryScope::new(scope.level, workspace).is_ok());
        check_scope(&scope);
    }
    let level = LEVELS[usize::from(selector) % LEVELS.len()];
    if let Ok(scope) = MemoryScope::new(level, workspace) {
        assert_eq!(scope.level, level);
        let _ = scope.ancestors();
        if let Ok(other_scope) = MemoryScope::parse(other) {
            let _ = scope.contains(&other_scope);
            let _ = other_scope.contains(&scope);
        }
    }

    // Transport principals: namespace binding must be exact string equality,
    // anonymous loopback is limited to the default workspace.
    let anonymous = TransportPrincipal::anonymous_loopback();
    assert_eq!(
        anonymous.allows_workspace(Some(workspace)),
        workspace == "default"
    );
    assert!(anonymous.allows_workspace(None));

    let bound = TransportPrincipal::StoredToken(AuthContext::with_namespace(
        UserId::system(),
        PermissionSet::read_only(),
        workspace.to_string(),
    ));
    assert!(bound.allows_workspace(Some(workspace)));
    assert_eq!(bound.allows_workspace(Some(other)), workspace == other);
    assert!(!bound.allows_workspace(None), "bound token needs a claim");

    let unscoped = TransportPrincipal::StoredToken(AuthContext::new(
        UserId::system(),
        PermissionSet::read_only(),
    ));
    assert!(unscoped.allows_workspace(Some(workspace)));
    assert!(unscoped.allows_workspace(None));

    // Arbitrary JSON arguments (the first byte selects whether the second
    // string is treated as JSON or wrapped as a scope value).
    let value = if selector & 1 == 0 {
        serde_json::from_str::<serde_json::Value>(other)
            .unwrap_or_else(|_| serde_json::json!({ "scope": other, "id": workspace }))
    } else {
        serde_json::json!({ "scope_path": [workspace, other], "ids": [workspace, other] })
    };
    for scope in extract_requested_scopes(&value) {
        assert!(!scope.trim().is_empty());
    }
    let _ = memory_id_arguments(&value);
});
