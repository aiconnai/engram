//! WebSocket server for real-time updates

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::{TransportPrincipal, TransportPrincipalError};

use super::events::{RealtimeEvent, SubscriptionFilter};

/// Connection ID
pub type ConnectionId = String;

struct RealtimeClient {
    filter: SubscriptionFilter,
    principal: TransportPrincipal,
}

/// Default maximum number of events retained in the replay ring buffer.
const DEFAULT_MAX_BUFFERED_EVENTS: usize = 500;

/// Manages WebSocket connections and SSE subscriptions.
///
/// Each event broadcast through [`RealtimeManager::broadcast`] is:
/// 1. Assigned a monotonically-increasing `seq_id`.
/// 2. Pushed into an in-memory ring buffer (capacity 500).
/// 3. Sent over the tokio broadcast channel for live subscribers.
///
/// Clients that reconnect with a `Last-Event-Id` header can call
/// [`RealtimeManager::get_events_after`] to retrieve buffered events they missed.
pub struct RealtimeManager {
    /// Broadcast channel for live delivery
    tx: broadcast::Sender<RealtimeEvent>,
    /// Connected clients with their filters and authenticated principals
    clients: Arc<RwLock<HashMap<ConnectionId, RealtimeClient>>>,
    /// Monotonically-increasing sequence counter (starts at 1)
    next_seq_id: Arc<AtomicU64>,
    /// In-memory ring buffer for replay
    buffer: Arc<RwLock<VecDeque<RealtimeEvent>>>,
    /// Maximum number of events kept in the buffer
    max_buffered_events: usize,
}

impl RealtimeManager {
    /// Create a new realtime manager with the default buffer size (500 events).
    pub fn new() -> Self {
        Self::with_buffer_size(DEFAULT_MAX_BUFFERED_EVENTS)
    }

    /// Create a realtime manager with a custom ring-buffer size.
    pub fn with_buffer_size(max_buffered_events: usize) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_seq_id: Arc::new(AtomicU64::new(1)),
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(
                max_buffered_events.min(4096),
            ))),
            max_buffered_events,
        }
    }

    /// Broadcast an event to all matching clients.
    ///
    /// The event is stamped with a sequential `seq_id`, pushed into the ring
    /// buffer, and sent over the broadcast channel.
    pub fn broadcast(&self, mut event: RealtimeEvent) {
        // Stamp with sequential ID (fetch-and-increment, wraps at u64::MAX which
        // is effectively never for any real-world workload).
        let seq = self.next_seq_id.fetch_add(1, Ordering::Relaxed);
        event.seq_id = Some(seq);

        // Push into ring buffer, evicting the oldest entry when full.
        {
            let mut buf = self.buffer.write();
            if buf.len() >= self.max_buffered_events {
                buf.pop_front();
            }
            buf.push_back(event.clone());
        }

        // Deliver to live subscribers (errors are expected when no subscriber
        // is registered yet — ignore them).
        let _ = self.tx.send(event);
    }

    /// Return all buffered events whose `seq_id` is strictly greater than
    /// `last_seq_id`, in ascending order. Used to replay missed events for
    /// reconnecting clients.
    pub fn get_events_after(&self, last_seq_id: u64) -> Vec<RealtimeEvent> {
        self.buffer
            .read()
            .iter()
            .filter(|e| e.seq_id.is_some_and(|id| id > last_seq_id))
            .cloned()
            .collect()
    }

    /// Return the current value of the sequence counter (next ID to be issued).
    /// Mainly useful for tests.
    pub fn current_seq(&self) -> u64 {
        self.next_seq_id.load(Ordering::Relaxed)
    }

    /// Get number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.read().len()
    }

    /// Subscribe to live events
    pub fn subscribe(&self) -> broadcast::Receiver<RealtimeEvent> {
        self.tx.subscribe()
    }

    /// Register a new client
    pub fn register_client(&self, id: ConnectionId, filter: SubscriptionFilter) {
        self.register_client_with_principal(id, filter, TransportPrincipal::anonymous_loopback());
    }

    /// Register a new client with its authenticated principal
    pub fn register_client_with_principal(
        &self,
        id: ConnectionId,
        filter: SubscriptionFilter,
        principal: TransportPrincipal,
    ) {
        self.clients
            .write()
            .insert(id, RealtimeClient { filter, principal });
    }

    /// Unregister a client
    pub fn unregister_client(&self, id: &str) {
        self.clients.write().remove(id);
    }

    /// Get client filter
    pub fn get_client_filter(&self, id: &str) -> Option<SubscriptionFilter> {
        self.clients
            .read()
            .get(id)
            .map(|client| client.filter.clone())
    }

    /// Update a connected client's subscription without replacing its principal
    pub fn update_client_filter(&self, id: &str, filter: SubscriptionFilter) {
        if let Some(client) = self.clients.write().get_mut(id) {
            client.filter = filter;
        }
    }

    /// Get client principal
    pub fn get_client_principal(&self, id: &str) -> Option<TransportPrincipal> {
        self.clients
            .read()
            .get(id)
            .map(|client| client.principal.clone())
    }
}

impl Default for RealtimeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RealtimeManager {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            clients: self.clients.clone(),
            next_seq_id: self.next_seq_id.clone(),
            buffer: self.buffer.clone(),
            max_buffered_events: self.max_buffered_events,
        }
    }
}

/// WebSocket server
pub struct RealtimeServer {
    manager: RealtimeManager,
    addr: SocketAddr,
    auth_key: Option<String>,
}

#[derive(Clone)]
struct RealtimeServerState {
    manager: RealtimeManager,
    auth_key: Option<String>,
}

impl RealtimeServer {
    /// Create a new WebSocket server
    pub fn new(manager: RealtimeManager, addr: SocketAddr) -> Self {
        Self {
            manager,
            addr,
            auth_key: None,
        }
    }

    /// Attach a process auth key required for WebSocket upgrades
    pub fn with_auth_key(mut self, auth_key: Option<String>) -> Self {
        self.auth_key = auth_key;
        self
    }

    /// Build the router
    pub fn router(manager: RealtimeManager) -> Router {
        Self::router_with_auth(manager, None)
    }

    fn router_with_auth(manager: RealtimeManager, auth_key: Option<String>) -> Router {
        let state = RealtimeServerState { manager, auth_key };
        Router::new()
            .route("/ws", get(ws_handler))
            .route("/health", get(health_handler))
            .with_state(state)
    }

    /// Start the server
    pub async fn start(self) -> std::io::Result<()> {
        if !self.addr.ip().is_loopback() && self.auth_key.as_deref().is_none_or(str::is_empty) {
            let message = "refusing websocket listener on non-loopback address without auth key";
            tracing::warn!(target = "engram::realtime", %message);
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                message,
            ));
        }
        let app = Self::router_with_auth(self.manager, self.auth_key);

        tracing::info!("WebSocket server listening on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
async fn health_handler(State(state): State<RealtimeServerState>) -> impl IntoResponse {
    serde_json::json!({
        "status": "ok",
        "clients": state.manager.client_count(),
    })
    .to_string()
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<RealtimeServerState>,
) -> axum::response::Response {
    let principal = match websocket_principal(&headers, state.auth_key.as_deref()) {
        Ok(principal) => principal,
        Err(error) => {
            tracing::warn!(
                target = "engram::realtime",
                reason = %error,
                "websocket auth rejected"
            );
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket, state.manager, principal))
        .into_response()
}

fn websocket_principal(
    headers: &HeaderMap,
    auth_key: Option<&str>,
) -> Result<TransportPrincipal, TransportPrincipalError> {
    match auth_key {
        Some(expected) => {
            let authorization = headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok());
            TransportPrincipal::from_process_bearer(authorization, expected)
        }
        None => Ok(TransportPrincipal::anonymous_loopback()),
    }
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, manager: RealtimeManager, principal: TransportPrincipal) {
    let connection_id = Uuid::new_v4().to_string();
    let filter = SubscriptionFilter::default();

    manager.register_client_with_principal(connection_id.clone(), filter.clone(), principal);
    tracing::info!("Client connected: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let mut rx = manager.subscribe();

    // Task to forward events to client
    let conn_id = connection_id.clone();
    let mgr = manager.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Check if event matches client's filter
            if let Some(filter) = mgr.get_client_filter(&conn_id) {
                if filter.matches(&event) {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Task to handle incoming messages from client
    let conn_id = connection_id.clone();
    let mgr = manager.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Try to parse as filter update
                    if let Ok(new_filter) = serde_json::from_str::<SubscriptionFilter>(&text) {
                        mgr.update_client_filter(&conn_id, new_filter);
                        tracing::debug!("Updated filter for client {}", conn_id);
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    manager.unregister_client(&connection_id);
    tracing::info!("Client disconnected: {}", connection_id);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Given: a real loopback websocket listener with process bearer auth.
        let manager = RealtimeManager::new();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = RealtimeServer::router_with_auth(manager.clone(), Some("secret".to_string()));
        let server = tokio::spawn(async move { axum::serve(listener, app).await });
        let mut client = tokio::net::TcpStream::connect(address).await.unwrap();

        // When: a client completes the upgrade with the configured bearer.
        client
            .write_all(
                format!(
                    "GET /ws HTTP/1.1\r\nHost: {address}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nAuthorization: Bearer secret\r\n\r\n"
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

    #[test]
    fn test_subscription_filter() {
        let filter = SubscriptionFilter {
            event_types: Some(vec![super::super::events::EventType::MemoryCreated]),
            memory_ids: None,
            tags: None,
        };

        let event = RealtimeEvent::memory_created(1, "test".to_string());
        assert!(filter.matches(&event));

        let event = RealtimeEvent::memory_deleted(1);
        assert!(!filter.matches(&event));
    }

    // --- Sequential event ID tests ------------------------------------------

    #[test]
    fn test_broadcast_stamps_sequential_ids() {
        let manager = RealtimeManager::new();
        let _rx = manager.subscribe(); // keep channel alive

        manager.broadcast(RealtimeEvent::memory_created(1, "first".to_string()));
        manager.broadcast(RealtimeEvent::memory_created(2, "second".to_string()));
        manager.broadcast(RealtimeEvent::memory_deleted(3));

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
        manager.broadcast(RealtimeEvent::memory_created(1, "hello".to_string()));
        assert_eq!(manager.current_seq(), 2); // next id to be issued
    }

    // --- Ring buffer eviction tests -----------------------------------------

    #[test]
    fn test_ring_buffer_evicts_oldest_when_full() {
        let max = 3;
        let manager = RealtimeManager::with_buffer_size(max);
        let _rx = manager.subscribe();

        for i in 1..=5u64 {
            manager.broadcast(RealtimeEvent::memory_created(i as i64, format!("m{i}")));
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
            manager.broadcast(RealtimeEvent::memory_deleted(i as i64));
        }

        assert_eq!(manager.buffer.read().len(), max);
    }

    // --- Replay / get_events_after tests ------------------------------------

    #[test]
    fn test_get_events_after_returns_correct_subset() {
        let manager = RealtimeManager::new();
        let _rx = manager.subscribe();

        manager.broadcast(RealtimeEvent::memory_created(1, "a".to_string())); // seq 1
        manager.broadcast(RealtimeEvent::memory_created(2, "b".to_string())); // seq 2
        manager.broadcast(RealtimeEvent::memory_deleted(3)); // seq 3

        let replayed = manager.get_events_after(1);
        assert_eq!(replayed.len(), 2);
        let ids: Vec<u64> = replayed.iter().filter_map(|e| e.seq_id).collect();
        assert_eq!(ids, vec![2, 3]);
    }

    #[test]
    fn test_get_events_after_zero_returns_all() {
        let manager = RealtimeManager::new();
        let _rx = manager.subscribe();

        manager.broadcast(RealtimeEvent::memory_created(1, "x".to_string()));
        manager.broadcast(RealtimeEvent::memory_created(2, "y".to_string()));

        let replayed = manager.get_events_after(0);
        assert_eq!(replayed.len(), 2);
    }

    #[test]
    fn test_get_events_after_last_id_returns_empty() {
        let manager = RealtimeManager::new();
        let _rx = manager.subscribe();

        manager.broadcast(RealtimeEvent::memory_created(1, "only".to_string())); // seq 1

        // Requesting events after the last known ID → nothing new
        let replayed = manager.get_events_after(1);
        assert!(replayed.is_empty());
    }

    #[test]
    fn test_get_events_after_large_id_returns_empty() {
        let manager = RealtimeManager::new();
        let _rx = manager.subscribe();

        manager.broadcast(RealtimeEvent::memory_created(1, "ev".to_string()));

        let replayed = manager.get_events_after(9999);
        assert!(replayed.is_empty());
    }

    // --- Clone shares same state --------------------------------------------

    #[test]
    fn test_clone_shares_buffer() {
        let manager = RealtimeManager::new();
        let cloned = manager.clone();
        let _rx = manager.subscribe();

        manager.broadcast(RealtimeEvent::memory_created(1, "shared".to_string()));

        // cloned should see the same buffer
        assert_eq!(cloned.buffer.read().len(), 1);
        let replayed = cloned.get_events_after(0);
        assert_eq!(replayed.len(), 1);
    }
}
