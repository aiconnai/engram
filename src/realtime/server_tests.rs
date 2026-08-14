//! Unit tests for the realtime WebSocket server.

use super::super::metrics::RealtimeMetrics;
use super::super::origin::parse_origin_allowlist;
use super::super::socket::{CLOSE_REASON_MESSAGE_TOO_LARGE, CLOSE_REASON_READ_IDLE};
use super::*;
use axum::extract::ws::close_code;
use base64::Engine as _;
use rand::RngCore;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn websocket_handshake(
    manager: RealtimeManager,
    path: &str,
    auth_key: Option<&str>,
    authorization: Option<&str>,
    origin: Option<&str>,
    allowed_origins: &[&str],
) -> (
    tokio::net::TcpStream,
    String,
    tokio::task::JoinHandle<Result<(), std::io::Error>>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = RealtimeServer::router_with_security(
        manager,
        auth_key.map(str::to_string),
        allowed_origins
            .iter()
            .map(|origin| (*origin).to_string())
            .collect(),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    let mut nonce = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let websocket_key = base64::engine::general_purpose::STANDARD.encode(nonce);
    let authorization = authorization
        .map(|value| format!("Authorization: {value}\r\n"))
        .unwrap_or_default();
    let origin = origin
        .map(|value| format!("Origin: {value}\r\n"))
        .unwrap_or_default();

    client
            .write_all(
                format!(
                    "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {websocket_key}\r\n{authorization}{origin}\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    let mut response = Vec::new();
    while !response.ends_with(b"\r\n\r\n") {
        response.push(client.read_u8().await.unwrap());
    }

    (client, String::from_utf8(response).unwrap(), server)
}

async fn spawn_resource_server(
    manager: RealtimeManager,
    auth_key: Option<&str>,
    config: RealtimeConfig,
    metrics: Arc<RealtimeMetrics>,
) -> (
    SocketAddr,
    tokio::task::JoinHandle<Result<(), std::io::Error>>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = RealtimeServer::router_with_resources(
        manager,
        auth_key.map(str::to_string),
        HashSet::new(),
        config,
        metrics,
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    (address, server)
}

async fn websocket_handshake_to(
    address: SocketAddr,
    path: &str,
    authorization: Option<&str>,
) -> (tokio::net::TcpStream, String) {
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    let mut nonce = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let websocket_key = base64::engine::general_purpose::STANDARD.encode(nonce);
    let authorization = authorization
        .map(|value| format!("Authorization: {value}\r\n"))
        .unwrap_or_default();
    client
            .write_all(
                format!(
                    "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {websocket_key}\r\n{authorization}\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    let mut response = Vec::new();
    while !response.ends_with(b"\r\n\r\n") {
        response.push(client.read_u8().await.unwrap());
    }
    (client, String::from_utf8(response).unwrap())
}

async fn read_server_text(client: &mut tokio::net::TcpStream) -> String {
    let first = client.read_u8().await.unwrap();
    assert_eq!(first & 0x0f, 1, "expected a text websocket frame");
    let second = client.read_u8().await.unwrap();
    assert_eq!(second & 0x80, 0, "server frames must not be masked");
    let len = match second & 0x7f {
        value @ 0..=125 => usize::from(value),
        126 => usize::from(client.read_u16().await.unwrap()),
        127 => usize::try_from(client.read_u64().await.unwrap()).unwrap(),
        _ => unreachable!(),
    };
    let mut payload = vec![0; len];
    client.read_exact(&mut payload).await.unwrap();
    String::from_utf8(payload).unwrap()
}

async fn write_masked_text(client: &mut tokio::net::TcpStream, payload: &[u8]) {
    assert!(payload.len() <= 125);
    let mut mask = [0_u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut mask);
    client.write_u8(0x81).await.unwrap();
    client
        .write_u8(0x80 | u8::try_from(payload.len()).unwrap())
        .await
        .unwrap();
    client.write_all(&mask).await.unwrap();
    let masked = payload
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ mask[index % mask.len()])
        .collect::<Vec<_>>();
    client.write_all(&masked).await.unwrap();
}

async fn read_server_close(client: &mut tokio::net::TcpStream) -> (u16, String) {
    let first = client.read_u8().await.unwrap();
    assert_eq!(first & 0x0f, 8, "expected a websocket close frame");
    let second = client.read_u8().await.unwrap();
    assert_eq!(second & 0x80, 0, "server frames must not be masked");
    let len = usize::from(second & 0x7f);
    assert!((2..=125).contains(&len));
    let code = client.read_u16().await.unwrap();
    let mut reason = vec![0; len - 2];
    client.read_exact(&mut reason).await.unwrap();
    (code, String::from_utf8(reason).unwrap())
}

async fn wait_for_active(metrics: &RealtimeMetrics, expected: u64) {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if metrics.active_connections.load(Ordering::Relaxed) == expected {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("active websocket gauge must converge");
}

#[test]
fn test_realtime_manager() {
    let manager = RealtimeManager::new();
    assert_eq!(manager.client_count(), 0);

    manager.register_client("test".to_string(), SubscriptionFilter::default());
    assert_eq!(manager.client_count(), 1);

    manager.unregister_client("test");
    assert_eq!(manager.client_count(), 0);
}

#[test]
fn authenticated_principal_survives_subscription_filter_updates() {
    // Given: an authenticated websocket connection registered with the manager.
    let manager = RealtimeManager::new();
    let principal =
        TransportPrincipal::from_process_bearer(Some("Bearer secret"), "secret").unwrap();
    manager.register_client_with_principal(
        "authenticated".to_string(),
        SubscriptionFilter::default(),
        principal,
    );

    // When: the connected client changes its event subscription filter.
    manager.update_client_filter(
        "authenticated",
        SubscriptionFilter {
            event_types: Some(vec![super::super::events::EventType::MemoryCreated]),
            memory_ids: None,
            tags: None,
        },
    );

    // Then: the connection retains the authenticated process principal.
    let attached = manager
        .get_client_principal("authenticated")
        .expect("authenticated principal remains attached");
    assert!(attached.has_permission(
        crate::auth::Permission::Admin,
        crate::auth::ResourceType::System
    ));
}

#[tokio::test]
async fn authenticated_handshake_attaches_process_principal() {
    // Given: a real loopback websocket listener with process bearer auth.
    let manager = RealtimeManager::new();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = RealtimeServer::router_with_auth(manager.clone(), Some("secret".to_string()));
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    let mut nonce = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let websocket_key = base64::engine::general_purpose::STANDARD.encode(nonce);

    // When: a client completes the upgrade with the configured bearer.
    client
            .write_all(
                format!(
                    "GET /ws HTTP/1.1\r\nHost: {address}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {websocket_key}\r\nAuthorization: Bearer secret\r\n\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    let mut response = Vec::new();
    while !response.ends_with(b"\r\n\r\n") {
        response.push(client.read_u8().await.unwrap());
    }

    // Then: the upgraded connection is registered with its process principal.
    assert!(String::from_utf8(response)
        .unwrap()
        .starts_with("HTTP/1.1 101"));
    for _ in 0..100 {
        if manager.client_count() == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }
    let clients = manager.clients.read();
    let attached = &clients
        .values()
        .next()
        .expect("upgraded client is registered")
        .principal;
    assert!(attached.has_permission(
        crate::auth::Permission::Admin,
        crate::auth::ResourceType::System
    ));

    server.abort();
}

#[tokio::test]
async fn real_client_receives_only_its_authorized_workspace() {
    let manager = RealtimeManager::new();
    let (mut client, response, server) = websocket_handshake(
        manager.clone(),
        "/ws?workspace=default",
        None,
        None,
        None,
        &[],
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 101"));
    for _ in 0..100 {
        if manager.client_count() == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }

    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "private sentinel".to_string(),
        "private",
    ));
    manager.broadcast(RealtimeEvent::memory_created(
        2,
        "visible sentinel".to_string(),
        "default",
    ));

    let payload = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        read_server_text(&mut client),
    )
    .await
    .expect("authorized workspace event must arrive");
    assert!(payload.contains("visible sentinel"));
    assert!(!payload.contains("private sentinel"));
    server.abort();
}

#[tokio::test]
async fn anonymous_client_cannot_subscribe_to_private_workspace() {
    let manager = RealtimeManager::new();
    let (_client, response, server) = websocket_handshake(
        manager.clone(),
        "/ws?workspace=private",
        None,
        None,
        None,
        &[],
    )
    .await;

    assert!(response.starts_with("HTTP/1.1 403"));
    assert_eq!(manager.client_count(), 0);
    server.abort();
}

#[tokio::test]
async fn origin_allowlist_is_exact_and_missing_origin_is_non_browser() {
    let allowed = "https://app.example.com";
    let (_client, response, server) = websocket_handshake(
        RealtimeManager::new(),
        "/ws",
        None,
        None,
        Some(allowed),
        &[allowed],
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 101"));
    server.abort();

    let (_client, response, server) = websocket_handshake(
        RealtimeManager::new(),
        "/ws",
        None,
        None,
        Some("https://evil.example.com"),
        &[allowed],
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 403"));
    server.abort();

    let (_client, response, server) =
        websocket_handshake(RealtimeManager::new(), "/ws", None, None, None, &[allowed]).await;
    assert!(response.starts_with("HTTP/1.1 101"));
    server.abort();
}

#[tokio::test]
async fn connection_cap_rejects_cap_plus_one_and_cleanup_restores_gauge() {
    let manager = RealtimeManager::new();
    let metrics = Arc::new(RealtimeMetrics::default());
    let config = RealtimeConfig {
        max_connections: 1,
        max_message_bytes: 1024,
        read_idle_timeout: std::time::Duration::from_secs(5),
    };
    let (address, server) = spawn_resource_server(manager, None, config, metrics.clone()).await;

    let (first, response) = websocket_handshake_to(address, "/ws", None).await;
    assert!(response.starts_with("HTTP/1.1 101"));
    wait_for_active(&metrics, 1).await;

    let (_second, response) = websocket_handshake_to(address, "/ws", None).await;
    assert!(response.starts_with("HTTP/1.1 503"));
    assert_eq!(metrics.connection_cap_rejections.load(Ordering::Relaxed), 1);

    drop(first);
    wait_for_active(&metrics, 0).await;
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.accepted_connections, 1);
    assert_eq!(snapshot.completed_disconnects, 1);
    server.abort();
}

#[tokio::test]
async fn oversized_message_closes_with_1009_without_disturbing_healthy_client() {
    let manager = RealtimeManager::new();
    let metrics = Arc::new(RealtimeMetrics::default());
    let config = RealtimeConfig {
        max_connections: 2,
        max_message_bytes: 8,
        read_idle_timeout: std::time::Duration::from_secs(5),
    };
    let (address, server) =
        spawn_resource_server(manager.clone(), Some("secret"), config, metrics.clone()).await;
    let (mut oversized, response) =
        websocket_handshake_to(address, "/ws", Some("Bearer secret")).await;
    assert!(response.starts_with("HTTP/1.1 101"));
    let (mut healthy, response) =
        websocket_handshake_to(address, "/ws", Some("Bearer secret")).await;
    assert!(response.starts_with("HTTP/1.1 101"));
    wait_for_active(&metrics, 2).await;

    write_masked_text(&mut oversized, b"123456789").await;
    let (code, reason) = read_server_close(&mut oversized).await;
    assert_eq!(code, close_code::SIZE);
    assert_eq!(reason, CLOSE_REASON_MESSAGE_TOO_LARGE);
    wait_for_active(&metrics, 1).await;
    assert_eq!(metrics.oversized_messages.load(Ordering::Relaxed), 1);

    manager.broadcast(RealtimeEvent::memory_created(
        7,
        "healthy stream".to_string(),
        "default",
    ));
    let payload = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        read_server_text(&mut healthy),
    )
    .await
    .expect("healthy authenticated stream remains available");
    assert!(payload.contains("healthy stream"));
    server.abort();
}

#[tokio::test]
async fn idle_client_closes_with_1008_and_records_cleanup_metrics() {
    let metrics = Arc::new(RealtimeMetrics::default());
    let config = RealtimeConfig {
        max_connections: 1,
        max_message_bytes: 1024,
        read_idle_timeout: std::time::Duration::from_millis(30),
    };
    let (address, server) =
        spawn_resource_server(RealtimeManager::new(), None, config, metrics.clone()).await;
    let (mut client, response) = websocket_handshake_to(address, "/ws", None).await;
    assert!(response.starts_with("HTTP/1.1 101"));

    let (code, reason) = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        read_server_close(&mut client),
    )
    .await
    .expect("idle client must be disconnected");
    assert_eq!(code, close_code::POLICY);
    assert_eq!(reason, CLOSE_REASON_READ_IDLE);
    wait_for_active(&metrics, 0).await;
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.idle_disconnects, 1);
    assert_eq!(snapshot.completed_disconnects, 1);
    server.abort();
}

#[test]
fn origin_allowlist_rejects_wildcards_and_non_origins() {
    assert!(parse_origin_allowlist(Some("*")).is_err());
    assert!(parse_origin_allowlist(Some("app.example.com")).is_err());
    assert!(parse_origin_allowlist(Some("https://app.example.com/path")).is_err());
    assert_eq!(
        parse_origin_allowlist(Some("https://app.example.com, https://admin.example.com/"))
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn scoped_principal_cannot_register_cross_workspace() {
    use crate::auth::{PermissionSet, TokenClaims, UserId};
    use chrono::Utc;

    let manager = RealtimeManager::new();
    let principal = TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("user-1"),
        key_id: "key-1".to_string(),
        permissions: PermissionSet::read_only(),
        namespace: Some("alpha".to_string()),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .unwrap();
    assert!(manager.try_register_client_with_principal(
        "scoped".to_string(),
        SubscriptionFilter::default(),
        principal.clone(),
        "alpha".to_string(),
    ));

    assert!(!manager.try_register_client_with_principal(
        "cross-workspace".to_string(),
        SubscriptionFilter::default(),
        principal,
        "beta".to_string(),
    ));
    assert!(manager.get_client_filter("scoped").is_some());
    assert!(manager.get_client_filter("cross-workspace").is_none());
}

#[test]
fn test_subscription_filter() {
    let filter = SubscriptionFilter {
        event_types: Some(vec![super::super::events::EventType::MemoryCreated]),
        memory_ids: None,
        tags: None,
    };

    let event = RealtimeEvent::memory_created(1, "test".to_string(), "default");
    assert!(filter.matches(&event));

    let event = RealtimeEvent::memory_deleted(1, "default");
    assert!(!filter.matches(&event));
}

// --- Sequential event ID tests ------------------------------------------

#[test]
fn test_broadcast_stamps_sequential_ids() {
    let manager = RealtimeManager::new();
    let _rx = manager.subscribe(); // keep channel alive

    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "first".to_string(),
        "default",
    ));
    manager.broadcast(RealtimeEvent::memory_created(
        2,
        "second".to_string(),
        "default",
    ));
    manager.broadcast(RealtimeEvent::memory_deleted(3, "default"));

    // IDs should be 1, 2, 3 (counter starts at 1)
    let buf = manager.buffer.read();
    let ids: Vec<u64> = buf.iter().filter_map(|e| e.seq_id).collect();
    assert_eq!(ids, vec![1, 2, 3]);
}

#[test]
fn test_seq_id_starts_at_one() {
    let manager = RealtimeManager::new();
    assert_eq!(manager.current_seq(), 1);

    let _rx = manager.subscribe();
    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "hello".to_string(),
        "default",
    ));
    assert_eq!(manager.current_seq(), 2); // next id to be issued
}

// --- Ring buffer eviction tests -----------------------------------------

#[test]
fn test_ring_buffer_evicts_oldest_when_full() {
    let max = 3;
    let manager = RealtimeManager::with_buffer_size(max);
    let _rx = manager.subscribe();

    for i in 1..=5u64 {
        manager.broadcast(RealtimeEvent::memory_created(
            i as i64,
            format!("m{i}"),
            "default",
        ));
    }

    let buf = manager.buffer.read();
    assert_eq!(buf.len(), max, "buffer should be at capacity");
    // The first two events (seq 1, 2) should have been evicted
    let ids: Vec<u64> = buf.iter().filter_map(|e| e.seq_id).collect();
    assert_eq!(ids, vec![3, 4, 5]);
}

#[test]
fn test_ring_buffer_does_not_exceed_max_size() {
    let max = 10;
    let manager = RealtimeManager::with_buffer_size(max);
    let _rx = manager.subscribe();

    for i in 1..=20u64 {
        manager.broadcast(RealtimeEvent::memory_deleted(i as i64, "default"));
    }

    assert_eq!(manager.buffer.read().len(), max);
}

// --- Replay / get_events_after tests ------------------------------------

#[test]
fn test_get_events_after_returns_correct_subset() {
    let manager = RealtimeManager::new();
    let _rx = manager.subscribe();

    manager.broadcast(RealtimeEvent::memory_created(1, "a".to_string(), "default")); // seq 1
    manager.broadcast(RealtimeEvent::memory_created(2, "b".to_string(), "default")); // seq 2
    manager.broadcast(RealtimeEvent::memory_deleted(3, "default")); // seq 3

    let replayed = manager.get_events_after(1);
    assert_eq!(replayed.len(), 2);
    let ids: Vec<u64> = replayed.iter().filter_map(|e| e.seq_id).collect();
    assert_eq!(ids, vec![2, 3]);
}

#[test]
fn test_get_events_after_zero_returns_all() {
    let manager = RealtimeManager::new();
    let _rx = manager.subscribe();

    manager.broadcast(RealtimeEvent::memory_created(1, "x".to_string(), "default"));
    manager.broadcast(RealtimeEvent::memory_created(2, "y".to_string(), "default"));

    let replayed = manager.get_events_after(0);
    assert_eq!(replayed.len(), 2);
}

#[test]
fn test_get_events_after_last_id_returns_empty() {
    let manager = RealtimeManager::new();
    let _rx = manager.subscribe();

    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "only".to_string(),
        "default",
    )); // seq 1

    // Requesting events after the last known ID → nothing new
    let replayed = manager.get_events_after(1);
    assert!(replayed.is_empty());
}

#[test]
fn test_get_events_after_large_id_returns_empty() {
    let manager = RealtimeManager::new();
    let _rx = manager.subscribe();

    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "ev".to_string(),
        "default",
    ));

    let replayed = manager.get_events_after(9999);
    assert!(replayed.is_empty());
}

// --- Clone shares same state --------------------------------------------

#[test]
fn test_clone_shares_buffer() {
    let manager = RealtimeManager::new();
    let cloned = manager.clone();
    let _rx = manager.subscribe();

    manager.broadcast(RealtimeEvent::memory_created(
        1,
        "shared".to_string(),
        "default",
    ));

    // cloned should see the same buffer
    assert_eq!(cloned.buffer.read().len(), 1);
    let replayed = cloned.get_events_after(0);
    assert_eq!(replayed.len(), 1);
}
