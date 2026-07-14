//! Per-connection WebSocket lifecycle.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket};
use tokio::sync::OwnedSemaphorePermit;
use uuid::Uuid;

use crate::auth::TransportPrincipal;

use super::config::RealtimeConfig;
use super::events::SubscriptionFilter;
use super::metrics::{ActiveConnectionGuard, RealtimeMetrics};
use super::server::{RealtimeManager, RegisteredClientGuard};

pub(super) const CLOSE_REASON_MESSAGE_TOO_LARGE: &str = "message exceeds configured byte limit";
pub(super) const CLOSE_REASON_READ_IDLE: &str = "read idle timeout";
const WS_WRITE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const WS_CLOSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// Handle an individual WebSocket connection after a successful upgrade.
pub(super) async fn handle_socket(
    mut socket: WebSocket,
    manager: RealtimeManager,
    principal: TransportPrincipal,
    workspace: String,
    config: RealtimeConfig,
    metrics: Arc<RealtimeMetrics>,
    _connection_permit: OwnedSemaphorePermit,
) {
    let connection_id = Uuid::new_v4().to_string();
    let filter = SubscriptionFilter::default();

    if !manager.try_register_client_with_principal(
        connection_id.clone(),
        filter,
        principal,
        workspace,
    ) {
        tracing::warn!(
            target = "engram::realtime",
            "websocket subscription rejected"
        );
        return;
    }
    let _active_connection = ActiveConnectionGuard::new(metrics.clone());
    let _registered_client = RegisteredClientGuard {
        manager: manager.clone(),
        connection_id: connection_id.clone(),
    };
    tracing::info!("Client connected: {}", connection_id);

    let mut rx = manager.subscribe();
    let idle = tokio::time::sleep(config.read_idle_timeout);
    tokio::pin!(idle);

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(incoming) = incoming else {
                    break;
                };
                let msg = match incoming {
                    Ok(msg) => msg,
                    Err(error) => {
                        tracing::debug!(target = "engram::realtime", %error, "websocket read failed");
                        break;
                    }
                };
                idle.as_mut().reset(tokio::time::Instant::now() + config.read_idle_timeout);
                match msg {
                    Message::Text(text) => {
                        if text.len() > config.max_message_bytes {
                            metrics.oversized_messages.fetch_add(1, Ordering::Relaxed);
                            close_socket(&mut socket, close_code::SIZE, CLOSE_REASON_MESSAGE_TOO_LARGE).await;
                            break;
                        }
                        if let Ok(new_filter) = serde_json::from_str::<SubscriptionFilter>(&text) {
                            manager.update_client_filter(&connection_id, new_filter);
                            tracing::debug!("Updated filter for client {}", connection_id);
                        }
                    }
                    Message::Binary(payload) => {
                        if payload.len() > config.max_message_bytes {
                            metrics.oversized_messages.fetch_add(1, Ordering::Relaxed);
                            close_socket(&mut socket, close_code::SIZE, CLOSE_REASON_MESSAGE_TOO_LARGE).await;
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    Message::Ping(payload) => {
                        if !send_socket(&mut socket, Message::Pong(payload)).await {
                            break;
                        }
                    }
                    Message::Pong(_) => {}
                }
            }
            event = rx.recv() => {
                let Ok(event) = event else {
                    break;
                };
                if manager.client_matches_event(&connection_id, &event) {
                    let Ok(json) = serde_json::to_string(&event) else {
                        continue;
                    };
                    if !send_socket(&mut socket, Message::Text(json)).await {
                        break;
                    }
                }
            }
            () = &mut idle => {
                metrics.idle_disconnects.fetch_add(1, Ordering::Relaxed);
                close_socket(&mut socket, close_code::POLICY, CLOSE_REASON_READ_IDLE).await;
                break;
            }
        }
    }

    tracing::info!("Client disconnected: {}", connection_id);
}

async fn close_socket(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let close = socket.send(Message::Close(Some(CloseFrame {
        code,
        reason: reason.into(),
    })));
    let _ = tokio::time::timeout(WS_CLOSE_TIMEOUT, close).await;
}

async fn send_socket(socket: &mut WebSocket, message: Message) -> bool {
    matches!(
        tokio::time::timeout(WS_WRITE_TIMEOUT, socket.send(message)).await,
        Ok(Ok(()))
    )
}
