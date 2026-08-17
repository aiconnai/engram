use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use super::{authenticate_transport_principal, principal_can_read_workspace, AppState};
use crate::mcp::progress::ProgressNotification;
use crate::realtime::{EventType, RealtimeEvent};

// ---------------------------------------------------------------------------
// SSE query parameters
// ---------------------------------------------------------------------------

/// Query parameters for the `GET /v1/events` SSE endpoint.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct EventsQuery {
    /// Comma-separated list of event types to subscribe to.
    /// Accepted values: `memory_created`, `memory_updated`, `memory_deleted`,
    /// `crossref_created`, `crossref_deleted`, `sync_started`, `sync_completed`,
    /// `sync_failed`.
    /// If omitted, all event types are streamed.
    pub(super) event_types: Option<String>,

    /// Filter events to a specific workspace (matched against `data.workspace`).
    /// If omitted, events from all workspaces are streamed.
    pub(super) workspace: Option<String>,
}

impl EventsQuery {
    /// Parse the `event_types` query param into a `Vec<EventType>`.
    /// Unknown tokens are silently ignored.
    pub(super) fn parsed_event_types(&self) -> Option<Vec<EventType>> {
        let raw = self.event_types.as_deref()?;
        let types: Vec<EventType> = raw
            .split(',')
            .filter_map(|s| parse_event_type(s.trim()))
            .collect();
        if types.is_empty() {
            None
        } else {
            Some(types)
        }
    }
}

/// Parse a snake_case string into an `EventType`.
pub(super) fn parse_event_type(s: &str) -> Option<EventType> {
    match s {
        "memory_created" => Some(EventType::MemoryCreated),
        "memory_updated" => Some(EventType::MemoryUpdated),
        "memory_deleted" => Some(EventType::MemoryDeleted),
        "crossref_created" => Some(EventType::CrossrefCreated),
        "crossref_deleted" => Some(EventType::CrossrefDeleted),
        "sync_started" => Some(EventType::SyncStarted),
        "sync_completed" => Some(EventType::SyncCompleted),
        "sync_failed" => Some(EventType::SyncFailed),
        _ => None,
    }
}

/// Serialize an `EventType` to its SSE `event:` field string.
pub(super) fn event_type_to_str(et: EventType) -> &'static str {
    match et {
        EventType::MemoryCreated => "memory_created",
        EventType::MemoryUpdated => "memory_updated",
        EventType::MemoryDeleted => "memory_deleted",
        EventType::CrossrefCreated => "crossref_created",
        EventType::CrossrefDeleted => "crossref_deleted",
        EventType::SyncStarted => "sync_started",
        EventType::SyncCompleted => "sync_completed",
        EventType::SyncFailed => "sync_failed",
    }
}

// ---------------------------------------------------------------------------
// SSE handler
// ---------------------------------------------------------------------------

/// Reconnection backoff hint sent to SSE clients (milliseconds).
pub(super) const SSE_RETRY_MS: u64 = 3000;

/// Convert a `RealtimeEvent` into an SSE `Event`, stamping the `id:` field
/// with `seq_id` when present.
pub(super) fn realtime_event_to_sse(event: &RealtimeEvent) -> Event {
    let event_type_str = event_type_to_str(event.event_type);
    let data = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
    let mut sse = Event::default().event(event_type_str).data(data);
    if let Some(id) = event.seq_id {
        sse = sse.id(format!("{id}"));
    }
    sse
}

/// Convert a `ProgressNotification` into an SSE `Event` with event type `progress`.
///
/// The full JSON-RPC notification is placed in the `data:` field so that SSE
/// clients can parse it identically to the stdio transport wire format.
#[allow(dead_code)]
pub(super) fn progress_to_sse(notification: &ProgressNotification) -> Event {
    let data = serde_json::to_string(notification).unwrap_or_else(|_| "{}".to_string());
    Event::default().event("progress").data(data)
}

/// `GET /v1/events` — resumable Server-Sent Events stream of `RealtimeEvent`s.
///
/// Each event is sent as:
/// ```text
/// id: <seq_id>
/// event: <event_type>
/// data: <JSON of RealtimeEvent>
/// retry: 3000
/// ```
///
/// **Resumable streams:** clients that reconnect after a drop should include
/// the `Last-Event-Id` header set to the last `id` value they received.
/// The server will replay all buffered events with a higher sequence number
/// before continuing with the live stream.
///
/// Query parameters:
/// - `event_types` — comma-separated list of event types to subscribe to
/// - `workspace` — filter events by workspace (matched against `data.workspace`)
///
/// Requires `Authorization: Bearer <token>` when the server was started with an API key.
pub(super) async fn handle_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let principal = match authenticate_transport_principal(&state.api_key, &headers) {
        Ok(principal) => principal,
        Err(_) => {
            state.metrics.on_events_request(true, false);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    if !principal_can_read_workspace(&principal, query.workspace.as_deref()) {
        state.metrics.on_events_request(true, false);
        return Err(StatusCode::FORBIDDEN);
    }

    // If realtime is not enabled, return 503.
    let manager = match state.realtime {
        Some(m) => m,
        None => {
            state.metrics.on_events_request(false, true);
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    state.metrics.on_events_request(false, false);

    // Parse Last-Event-Id header for replay support.
    let last_event_id: Option<u64> = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let event_type_filter = query.parsed_event_types();
    let workspace_filter = query.workspace.clone();

    // Build a filter closure reused for both replay and live events.
    let apply_filters = {
        let et_filter = event_type_filter.clone();
        let ws_filter = workspace_filter.clone();
        move |event: &RealtimeEvent| -> bool {
            if let Some(ref types) = et_filter {
                if !types.contains(&event.event_type) {
                    return false;
                }
            }
            if let Some(ref ws) = ws_filter {
                let event_ws = event
                    .data
                    .as_ref()
                    .and_then(|d: &serde_json::Value| d.get("workspace"))
                    .and_then(|v: &serde_json::Value| v.as_str());
                match event_ws {
                    Some(ews) if ews == ws => {}
                    _ => return false,
                }
            }
            true
        }
    };

    // Subscribe to the live broadcast channel *before* draining the buffer so
    // we don't miss any events that arrive between the two operations.
    let rx = manager.subscribe();
    let broadcast_stream = BroadcastStream::new(rx);

    // Build the replay burst (may be empty if no Last-Event-Id or nothing to replay).
    let replay_events: Vec<Result<Event, Infallible>> = if let Some(last_id) = last_event_id {
        manager
            .get_events_after(last_id)
            .into_iter()
            .filter(|e| apply_filters(e))
            .map(|e| Ok::<Event, Infallible>(realtime_event_to_sse(&e)))
            .collect()
    } else {
        vec![]
    };

    let replay_stream = tokio_stream::iter(replay_events);

    // Live stream from broadcast channel.
    let live_stream = broadcast_stream.filter_map(move |result| {
        match result {
            // Lagged: the receiver fell behind — skip dropped events without crashing.
            Err(_lagged) => None,
            Ok(event) => {
                if !apply_filters(&event) {
                    return None;
                }
                Some(Ok::<Event, Infallible>(realtime_event_to_sse(&event)))
            }
        }
    });

    // Chain: replay burst first, then live events.
    let combined = replay_stream.chain(live_stream);

    // Prepend a `retry:` field so clients know the reconnection backoff.
    // The retry directive is sent as a synthetic SSE comment event emitted once
    // at the start of the stream.
    let retry_event = std::iter::once(Ok::<Event, Infallible>(
        Event::default().retry(Duration::from_millis(SSE_RETRY_MS)),
    ));
    let full_stream = tokio_stream::iter(retry_event).chain(combined);

    Ok(Sse::new(full_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30))))
}
