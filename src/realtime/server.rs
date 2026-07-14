//! WebSocket server for real-time updates

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use parking_lot::RwLock;
use tokio::sync::{broadcast, Semaphore};

use crate::auth::TransportPrincipal;
use crate::types::normalize_workspace;

use super::auth::{principal_can_subscribe, websocket_principal};
use super::config::RealtimeConfig;
use super::events::{RealtimeEvent, SubscriptionFilter};
use super::metrics::RealtimeMetrics;
use super::origin::{origin_is_allowed, parse_origin_allowlist, WS_ALLOWED_ORIGINS_ENV};
use super::socket::handle_socket;

/// Connection ID
pub type ConnectionId = String;

struct RealtimeClient {
    filter: SubscriptionFilter,
    principal: TransportPrincipal,
    workspace: String,
}

/// Default maximum number of events retained in the replay ring buffer.
const DEFAULT_MAX_BUFFERED_EVENTS: usize = 500;

/// Drops the client registration when the upgraded connection ends.
pub(super) struct RegisteredClientGuard {
    pub(super) manager: RealtimeManager,
    pub(super) connection_id: ConnectionId,
}

impl Drop for RegisteredClientGuard {
    fn drop(&mut self) {
        self.manager.unregister_client(&self.connection_id);
    }
}

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

    pub(super) fn try_register_client_with_principal(
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

    pub(super) fn client_matches_event(&self, id: &str, event: &RealtimeEvent) -> bool {
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
    config: Result<RealtimeConfig, String>,
}

#[derive(Clone)]
struct RealtimeServerState {
    manager: RealtimeManager,
    auth_key: Option<String>,
    allowed_origins: Arc<HashSet<String>>,
    config: RealtimeConfig,
    connection_slots: Arc<Semaphore>,
    metrics: Arc<RealtimeMetrics>,
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
            config: RealtimeConfig::from_env(),
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
        Self::router_with_resources(
            manager,
            auth_key,
            allowed_origins,
            RealtimeConfig::default(),
            Arc::new(RealtimeMetrics::default()),
        )
    }

    fn router_with_resources(
        manager: RealtimeManager,
        auth_key: Option<String>,
        allowed_origins: HashSet<String>,
        config: RealtimeConfig,
        metrics: Arc<RealtimeMetrics>,
    ) -> Router {
        let state = RealtimeServerState {
            manager,
            auth_key,
            allowed_origins: Arc::new(allowed_origins),
            connection_slots: Arc::new(Semaphore::new(config.max_connections)),
            config,
            metrics,
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
        let config = self
            .config
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
        let app = Self::router_with_resources(
            self.manager,
            self.auth_key,
            allowed_origins,
            config,
            Arc::new(RealtimeMetrics::default()),
        );

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
        "resources": state.metrics.snapshot(),
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

    let connection_permit = match state.connection_slots.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            state
                .metrics
                .connection_cap_rejections
                .fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                target = "engram::realtime",
                "websocket connection cap reached"
            );
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };

    // Axum/tungstenite rejects declared frames above max+1 before allocating
    // their payload. The one-byte margin lets the application return the
    // documented 1009 close for the exact boundary regression.
    let transport_limit = state.config.max_message_bytes.saturating_add(1);
    let config = state.config.clone();
    let metrics = state.metrics.clone();

    ws.max_frame_size(transport_limit)
        .max_message_size(transport_limit)
        .on_upgrade(move |socket| {
            handle_socket(
                socket,
                state.manager,
                principal,
                requested_workspace,
                config,
                metrics,
                connection_permit,
            )
        })
        .into_response()
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
