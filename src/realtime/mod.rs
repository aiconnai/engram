//! Real-time updates via WebSocket (RML-881)
//!
//! Provides push notifications for memory changes to connected clients.

mod auth;
mod config;
pub(crate) mod events;
mod metrics;
mod origin;
mod server;
mod socket;

pub use events::{EventType, RealtimeEvent, SubscriptionFilter};
pub use server::{RealtimeManager, RealtimeServer};
