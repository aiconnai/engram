//! Engram - AI Memory Infrastructure
//!
//! Persistent memory for AI agents with semantic search, cloud sync,
//! and knowledge graph visualization.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::large_enum_variant)]

pub mod app_state;
#[cfg(feature = "agent-portability")]
pub mod attestation;
pub mod auth;
pub mod bench;
pub mod embedding;
pub mod error;
pub mod graph;
#[cfg(feature = "hooks")]
pub mod hooks;
#[cfg(feature = "langfuse")]
pub mod integrations;
pub mod intelligence;
pub mod mcp;
#[cfg(feature = "multimodal")]
pub mod multimodal;
pub mod realtime;
pub mod search;
#[cfg(feature = "agent-portability")]
pub mod snapshot;
pub mod storage;
pub mod sync;
pub mod types;
#[cfg(feature = "watcher")]
pub mod watcher;

pub use error::{EngramError, Result};
pub use storage::Storage;
pub use types::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
