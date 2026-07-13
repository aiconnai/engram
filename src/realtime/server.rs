//! WebSocket server for real-time updates

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header::AUTHORIZATION, header::ORIGIN, HeaderMap, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::{TransportPrincipal, TransportPrincipalError};
use crate::types::normalize_workspace;

use super::events::{RealtimeEvent, SubscriptionFilter};

/// Connection ID
pub type ConnectionId = String;

struct RealtimeClient {
    filter: SubscriptionFilter,
    principal: TransportPrincipal,
    workspace: String,
}

/// Default maximum number of events retained in the replay ring buffer.
const DEFAULT_MAX_BUFFERED_EVENTS: usize = 500;
const WS_ALLOWED_ORIGINS_ENV: &str = "ENGRAM_WS_ALLOWED_ORIGINS";

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
        self.try_register_client_with_principal(id, filter, principal, "default".to_string());
    }

    fn try_register_client_with_principal(
        &self,
        id: ConnectionId,
        filter: SubscriptionFilter,
        principal: TransportPrincipal,
        workspace: String,
    ) -> bool {
        if !principal_can_subscribe(&principal, &workspace) {
            return false;
        }
        self.clients.write().insert(
            id,
            RealtimeClient {
                filter,
                principal,
                workspace,
            },
        );
        true
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

    fn client_matches_event(&self, id: &str, event: &RealtimeEvent) -> bool {
        self.clients.read().get(id).is_some_and(|client| {
            event.workspace() == Some(client.workspace.as_str()) && client.filter.matches(event)
        })
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
    allowed_origins: Result<HashSet<String>, String>,
}

#[derive(Clone)]
struct RealtimeServerState {
    manager: RealtimeManager,
    auth_key: Option<String>,
    allowed_origins: Arc<HashSet<String>>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct WebSocketQuery {
    workspace: Option<String>,
}

impl RealtimeServer {
    /// Create a new WebSocket server
    pub fn new(manager: RealtimeManager, addr: SocketAddr) -> Self {
        Self {
            manager,
            addr,
            auth_key: None,
            allowed_origins: parse_origin_allowlist(
                std::env::var(WS_ALLOWED_ORIGINS_ENV).ok().as_deref(),
            ),
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
        Self::router_with_security(manager, auth_key, HashSet::new())
    }

    fn router_with_security(
        manager: RealtimeManager,
        auth_key: Option<String>,
        allowed_origins: HashSet<String>,
    ) -> Router {
        let state = RealtimeServerState {
            manager,
            auth_key,
            allowed_origins: Arc::new(allowed_origins),
        };
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
        let allowed_origins = self
            .allowed_origins
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
        let app = Self::router_with_security(self.manager, self.auth_key, allowed_origins);

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
    Query(query): Query<WebSocketQuery>,
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

    if !origin_is_allowed(&headers, &state.allowed_origins) {
        tracing::warn!(target = "engram::realtime", "websocket origin rejected");
        return StatusCode::FORBIDDEN.into_response();
    }

    let requested_workspace = match query.workspace.as_deref() {
        Some(raw) => match normalize_workspace(raw) {
            Ok(workspace) => workspace,
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => "default".to_string(),
    };
    if !principal_can_subscribe(&principal, &requested_workspace) {
        tracing::warn!(target = "engram::realtime", "websocket workspace rejected");
        return StatusCode::FORBIDDEN.into_response();
    }

    ws.on_upgrade(move |socket| {
        handle_socket(socket, state.manager, principal, requested_workspace)
    })
    .into_response()
}

fn principal_can_subscribe(principal: &TransportPrincipal, workspace: &str) -> bool {
    principal.allows_workspace(Some(workspace))
        && principal.has_permission(
            crate::auth::Permission::Read,
            crate::auth::ResourceType::Memory,
        )
}

fn origin_is_allowed(headers: &HeaderMap, allowed_origins: &HashSet<String>) -> bool {
    let mut origins = headers.get_all(ORIGIN).iter();
    let Some(origin) = origins.next() else {
        return true;
    };
    if origins.next().is_some() {
        return false;
    }
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    allowed_origins.contains(origin)
}

fn parse_origin_allowlist(raw: Option<&str>) -> Result<HashSet<String>, String> {
    let Some(raw) = raw else {
        return Ok(HashSet::new());
    };

    let mut origins = HashSet::new();
    for entry in raw.split(',') {
        let origin = entry.trim();
        if origin.is_empty() || origin == "*" {
            return Err(format!(
                "{WS_ALLOWED_ORIGINS_ENV} must contain explicit origins without wildcards"
            ));
        }
        let uri = origin
            .parse::<Uri>()
            .map_err(|_| format!("{WS_ALLOWED_ORIGINS_ENV} contains an invalid origin"))?;
        if !matches!(uri.scheme_str(), Some("http" | "https"))
            || uri
                .authority()
                .is_none_or(|authority| authority.as_str().contains('@'))
            || uri.query().is_some()
            || !matches!(uri.path(), "" | "/")
        {
            return Err(format!(
                "{WS_ALLOWED_ORIGINS_ENV} origins must be scheme-and-authority values"
            ));
        }
        origins.insert(origin.trim_end_matches('/').to_string());
    }
    Ok(origins)
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
async fn handle_socket(
    socket: WebSocket,
    manager: RealtimeManager,
    principal: TransportPrincipal,
    workspace: String,
) {
    let connection_id = Uuid::new_v4().to_string();
    let filter = SubscriptionFilter::default();

    if !manager.try_register_client_with_principal(
        connection_id.clone(),
        filter.clone(),
        principal,
        workspace,
    ) {
        tracing::warn!(
            target = "engram::realtime",
            "websocket subscription rejected"
        );
        return;
    }
    tracing::info!("Client connected: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let mut rx = manager.subscribe();

    // Task to forward events to client
    let conn_id = connection_id.clone();
    let mgr = manager.clone();
    let send_task = AbortOnDropTask::new(tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Check if event matches client's filter
            if mgr.client_matches_event(&conn_id, &event) {
                let json = serde_json::to_string(&event).unwrap_or_default();
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    }));

    // Task to handle incoming messages from client
    let conn_id = connection_id.clone();
    let mgr = manager.clone();
    let recv_task = AbortOnDropTask::new(tokio::spawn(async move {
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
    }));

    await_peer_tasks(send_task, recv_task).await;

    manager.unregister_client(&connection_id);
    tracing::info!("Client disconnected: {}", connection_id);
}

struct AbortOnDropTask {
    handle: tokio::task::JoinHandle<()>,
}

impl AbortOnDropTask {
    fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self { handle }
    }

    async fn abort_and_wait(&mut self) {
        self.handle.abort();
        let _ = (&mut self.handle).await;
    }
}

impl Drop for AbortOnDropTask {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn await_peer_tasks(mut send_task: AbortOnDropTask, mut recv_task: AbortOnDropTask) {
    tokio::select! {
        _ = &mut send_task.handle => {
            recv_task.abort_and_wait().await;
        }
        _ = &mut recv_task.handle => {
            send_task.abort_and_wait().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use rand::RngCore;
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

    struct TaskDropSignal(Arc<std::sync::atomic::AtomicBool>);

    impl Drop for TaskDropSignal {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn disconnect_cancels_blocked_send_peer() {
        // Given: a send peer blocked waiting for an event and a completed receive peer.
        let send_dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let send_signal = TaskDropSignal(send_dropped.clone());
        let send_task = AbortOnDropTask::new(tokio::spawn(async move {
            let _signal = send_signal;
            std::future::pending::<()>().await;
        }));
        let recv_task = AbortOnDropTask::new(tokio::spawn(async {}));

        // When: the receive peer observes the client disconnect.
        await_peer_tasks(send_task, recv_task).await;

        // Then: the blocked send peer is cancelled and awaited before cleanup returns.
        assert!(send_dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn send_failure_cancels_blocked_receive_peer() {
        // Given: a completed send peer and a receive peer blocked waiting for client input.
        let recv_dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let recv_signal = TaskDropSignal(recv_dropped.clone());
        let send_task = AbortOnDropTask::new(tokio::spawn(async {}));
        let recv_task = AbortOnDropTask::new(tokio::spawn(async move {
            let _signal = recv_signal;
            std::future::pending::<()>().await;
        }));

        // When: the send peer stops after its websocket write fails.
        await_peer_tasks(send_task, recv_task).await;

        // Then: the blocked receive peer is cancelled and awaited before cleanup returns.
        assert!(recv_dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn coordinator_cancellation_aborts_both_peers() {
        // Given: two live peer tasks owned by a connection coordinator.
        let send_dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let recv_dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let send_signal = TaskDropSignal(send_dropped.clone());
        let recv_signal = TaskDropSignal(recv_dropped.clone());
        let send_task = AbortOnDropTask::new(tokio::spawn(async move {
            let _signal = send_signal;
            std::future::pending::<()>().await;
        }));
        let recv_task = AbortOnDropTask::new(tokio::spawn(async move {
            let _signal = recv_signal;
            std::future::pending::<()>().await;
        }));
        let coordinator = tokio::spawn(await_peer_tasks(send_task, recv_task));
        tokio::task::yield_now().await;

        // When: the connection coordinator is cancelled before either peer finishes.
        coordinator.abort();
        let _ = coordinator.await;
        for _ in 0..100 {
            if send_dropped.load(Ordering::SeqCst) && recv_dropped.load(Ordering::SeqCst) {
                break;
            }
            tokio::task::yield_now().await;
        }

        // Then: ownership cancellation aborts both peer tasks instead of detaching them.
        assert!(send_dropped.load(Ordering::SeqCst));
        assert!(recv_dropped.load(Ordering::SeqCst));
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

        manager.broadcast(
            RealtimeEvent::memory_created(1, "private sentinel".to_string())
                .with_workspace("private"),
        );
        manager.broadcast(
            RealtimeEvent::memory_created(2, "visible sentinel".to_string())
                .with_workspace("default"),
        );

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
