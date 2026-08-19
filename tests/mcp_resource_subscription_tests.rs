//! MCP Dynamic Resource Subscriptions Integration Tests.
//!
//! Verifies:
//! - Capability advertising for resource subscriptions (MCP 2025-11-25).
//! - Subscribing and unsubscribing from valid/invalid resource URIs.
//! - Invalidation matching for memory, workspace, entities, and stats resources.
//! - Serialization of `notifications/resources/updated` and `notifications/resources/list_changed`.

use engram::mcp::{
    validate_resource_uri, InitializeResult, ResourceListChangedNotification,
    ResourceSubscriptionManager, ResourceUpdatedNotification,
};

#[test]
fn test_initialize_advertises_resource_subscription_capabilities() {
    let init = InitializeResult::default();
    let res_caps = init
        .capabilities
        .resources
        .expect("resources capability must be present");
    assert!(res_caps.subscribe, "subscribe capability must be true");
    assert!(
        res_caps.list_changed,
        "list_changed capability must be true"
    );
}

#[test]
fn test_resources_subscribe_and_unsubscribe_manager() {
    let mgr = ResourceSubscriptionManager::new();
    assert_eq!(mgr.count(), 0);

    // Subscribe to valid URIs
    assert!(mgr.subscribe("engram://stats").is_ok());
    assert!(mgr.subscribe("engram://entities").is_ok());
    assert!(mgr.subscribe("engram://memory/42").is_ok());
    assert!(mgr.subscribe("engram://workspace/default").is_ok());
    assert!(mgr
        .subscribe("engram://workspace/default/memories?limit=10")
        .is_ok());

    assert_eq!(mgr.count(), 5);
    assert!(mgr.is_subscribed("engram://stats"));
    assert!(mgr.is_subscribed("engram://memory/42"));
    assert!(mgr.is_subscribed("engram://workspace/default/memories?limit=10"));
    assert!(!mgr.is_subscribed("engram://memory/999"));

    // Unsubscribe
    assert!(mgr.unsubscribe("engram://stats").is_ok());
    assert_eq!(mgr.count(), 4);
    assert!(!mgr.is_subscribed("engram://stats"));

    // Clear
    mgr.clear();
    assert_eq!(mgr.count(), 0);
}

#[test]
fn test_invalid_resource_uris_rejected() {
    let mgr = ResourceSubscriptionManager::new();

    // Invalid format
    assert!(!validate_resource_uri("https://example.com"));
    assert!(!validate_resource_uri("engram://unknown/foo"));
    assert!(!validate_resource_uri("engram://memory/not-a-number"));
    assert!(!validate_resource_uri("engram://workspace/"));

    assert!(mgr.subscribe("https://example.com").is_err());
    assert!(mgr.subscribe("engram://invalid").is_err());
    assert!(mgr.subscribe("engram://memory/abc").is_err());
    assert_eq!(mgr.count(), 0);
}

#[test]
fn test_match_affected_uris_routing() {
    let mgr = ResourceSubscriptionManager::new();

    mgr.subscribe("engram://stats").unwrap();
    mgr.subscribe("engram://entities").unwrap();
    mgr.subscribe("engram://memory/10").unwrap();
    mgr.subscribe("engram://memory/20").unwrap();
    mgr.subscribe("engram://workspace/infra/memories").unwrap();
    mgr.subscribe("engram://workspace/frontend/memories")
        .unwrap();

    // Mutation on workspace "infra", memory ID 10
    let affected = mgr.match_affected_uris(Some("infra"), Some(10));

    assert!(affected.contains(&"engram://stats".to_string()));
    assert!(affected.contains(&"engram://entities".to_string()));
    assert!(affected.contains(&"engram://memory/10".to_string()));
    assert!(affected.contains(&"engram://workspace/infra/memories".to_string()));
    assert!(!affected.contains(&"engram://memory/20".to_string()));
    assert!(!affected.contains(&"engram://workspace/frontend/memories".to_string()));

    // Mutation on workspace "frontend", memory ID 20
    let affected2 = mgr.match_affected_uris(Some("frontend"), Some(20));
    assert!(affected2.contains(&"engram://memory/20".to_string()));
    assert!(affected2.contains(&"engram://workspace/frontend/memories".to_string()));
    assert!(!affected2.contains(&"engram://memory/10".to_string()));
    assert!(!affected2.contains(&"engram://workspace/infra/memories".to_string()));
}

#[test]
fn test_resource_updated_notification_serialization() {
    let notif = ResourceUpdatedNotification::new("engram://workspace/dev/memories");
    let json = serde_json::to_value(&notif).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["method"], "notifications/resources/updated");
    assert_eq!(json["params"]["uri"], "engram://workspace/dev/memories");
}

#[test]
fn test_resource_list_changed_notification_serialization() {
    let notif = ResourceListChangedNotification::default();
    let json = serde_json::to_value(&notif).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["method"], "notifications/resources/list_changed");
}
