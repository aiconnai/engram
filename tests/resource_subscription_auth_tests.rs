//! Principal-aware `resources/subscribe` and `resources/unsubscribe`.
//!
//! A workspace-restricted principal may subscribe only to the resources it may
//! read (see `read_resource_as`): its own workspaces and memories stored in
//! them. A subscription to anything else would later report activity (or
//! its absence) in workspaces the principal cannot see.

use chrono::Utc;

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::mcp::ResourceSubscriptionManager;
use engram::storage::{create_memory, Storage};
use engram::types::CreateMemoryInput;

const OWN_WS: &str = "tenant-a";
const FOREIGN_WS: &str = "tenant-b";

fn principal(namespace: Option<&str>) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("agent-subscriber"),
        key_id: "key-agent-subscriber".to_string(),
        permissions: PermissionSet::standard_user(),
        namespace: namespace.map(str::to_string),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

fn seed(storage: &Storage, content: &str, workspace: &str) -> i64 {
    let input: CreateMemoryInput =
        serde_json::from_value(serde_json::json!({"content": content, "workspace": workspace}))
            .expect("create input");
    storage
        .with_connection(|conn| Ok(create_memory(conn, &input)?.id))
        .expect("seed memory")
}

struct Fixture {
    storage: Storage,
    own_id: i64,
    foreign_id: i64,
    missing_id: i64,
}

fn fixture() -> Fixture {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let own_id = seed(&storage, "own roadmap note", OWN_WS);
    let foreign_id = seed(&storage, "foreign payroll secret", FOREIGN_WS);
    Fixture {
        storage,
        own_id,
        foreign_id,
        missing_id: foreign_id + 10_000,
    }
}

#[test]
fn restricted_principal_subscribes_only_to_its_own_resources() {
    let fx = fixture();
    let who = principal(Some(OWN_WS));
    let mgr = ResourceSubscriptionManager::new();

    for uri in [
        format!("engram://workspace/{OWN_WS}"),
        format!("engram://workspace/{OWN_WS}/memories?limit=5"),
        format!("engram://memory/{}", fx.own_id),
    ] {
        mgr.subscribe_as(&fx.storage, &uri, Some(&who))
            .unwrap_or_else(|e| panic!("own resource {uri} denied: {e}"));
        assert!(mgr.is_subscribed(&uri));
    }
    let own_count = mgr.count();

    for uri in [
        format!("engram://workspace/{FOREIGN_WS}"),
        format!("engram://workspace/{FOREIGN_WS}/memories"),
        format!("engram://memory/{}", fx.foreign_id),
        "engram://stats".to_string(),
        "engram://entities".to_string(),
    ] {
        let err = mgr
            .subscribe_as(&fx.storage, &uri, Some(&who))
            .expect_err(&format!("{uri} must be denied"));
        assert!(!err.contains("payroll"), "{uri}: leaked content: {err}");
        assert!(!mgr.is_subscribed(&uri), "{uri}: denied but registered");
    }
    assert_eq!(
        mgr.count(),
        own_count,
        "denied calls must not register anything"
    );
}

#[test]
fn foreign_and_missing_memory_subscriptions_are_indistinguishable() {
    let fx = fixture();
    let who = principal(Some(OWN_WS));
    let mgr = ResourceSubscriptionManager::new();

    let foreign = mgr
        .subscribe_as(
            &fx.storage,
            &format!("engram://memory/{}", fx.foreign_id),
            Some(&who),
        )
        .expect_err("foreign memory");
    let missing = mgr
        .subscribe_as(
            &fx.storage,
            &format!("engram://memory/{}", fx.missing_id),
            Some(&who),
        )
        .expect_err("missing memory");

    assert_eq!(
        foreign.replace(&fx.foreign_id.to_string(), "<ID>"),
        missing.replace(&fx.missing_id.to_string(), "<ID>"),
        "a foreign row must fail exactly like a missing one"
    );
}

#[test]
fn invalid_uris_are_still_rejected_for_every_caller() {
    let fx = fixture();
    let mgr = ResourceSubscriptionManager::new();
    for who in [Some(principal(Some(OWN_WS))), Some(principal(None)), None] {
        for uri in [
            "engram://nothing",
            "engram://memory/abc",
            "https://example.com",
        ] {
            let err = mgr
                .subscribe_as(&fx.storage, uri, who.as_ref())
                .expect_err(uri);
            assert!(err.contains("Invalid resource URI"), "{uri}: {err}");
        }
    }
    assert_eq!(mgr.count(), 0);
}

#[test]
fn unrestricted_principal_and_stdio_subscribe_like_before() {
    let fx = fixture();
    let unrestricted = principal(None);
    for who in [Some(&unrestricted), None] {
        let mgr = ResourceSubscriptionManager::new();
        for uri in [
            "engram://stats".to_string(),
            "engram://entities".to_string(),
            format!("engram://workspace/{FOREIGN_WS}"),
            format!("engram://memory/{}", fx.foreign_id),
            format!("engram://memory/{}", fx.missing_id),
        ] {
            mgr.subscribe_as(&fx.storage, &uri, who)
                .unwrap_or_else(|e| panic!("{uri} denied: {e}"));
        }
        assert_eq!(mgr.count(), 5);
    }
}

#[test]
fn restricted_principal_cannot_unsubscribe_resources_it_could_not_subscribe_to() {
    let fx = fixture();
    let who = principal(Some(OWN_WS));
    let mgr = ResourceSubscriptionManager::new();
    let foreign_uri = format!("engram://workspace/{FOREIGN_WS}");
    let own_uri = format!("engram://workspace/{OWN_WS}");
    mgr.subscribe(&foreign_uri).unwrap();
    mgr.subscribe(&own_uri).unwrap();

    assert!(mgr
        .unsubscribe_as(&fx.storage, &foreign_uri, Some(&who))
        .is_err());
    assert!(
        mgr.is_subscribed(&foreign_uri),
        "foreign subscription removed"
    );

    mgr.unsubscribe_as(&fx.storage, &own_uri, Some(&who))
        .expect("own subscription can be removed");
    assert!(!mgr.is_subscribed(&own_uri));

    // Unrestricted callers keep the old behaviour.
    mgr.unsubscribe_as(&fx.storage, &foreign_uri, Some(&principal(None)))
        .expect("unrestricted unsubscribe");
    assert_eq!(mgr.count(), 0);
}
