//! Turso/libSQL implementation of the StorageBackend trait (Phase 6 - ENG-54)
//!
//! This module provides a Turso/libSQL-based storage backend that implements
//! the `StorageBackend` trait, enabling distributed SQLite with edge replicas.
//!
//! # Features
//!
//! - **Embedded replicas**: Local SQLite with sync to Turso cloud
//! - **Edge-native**: Sub-millisecond reads from local replica
//! - **Sync on demand**: Push/pull changes to cloud
//! - **Compatible schema**: Same migrations as SQLite backend
//!
//! # Usage
//!
//! ```rust,ignore
//! use engram::storage::TursoBackend;
//!
//! // Connect to Turso cloud with embedded replica
//! let backend = TursoBackend::new(
//!     "libsql://your-db.turso.io",
//!     "your-auth-token",
//!     Some("/path/to/local/replica.db"),
//! ).await?;
//!
//! // Or use local-only mode (no cloud sync)
//! let backend = TursoBackend::local_only("/path/to/db.sqlite").await?;
//! ```

mod core;
mod impls;
mod impls_crud;
mod impls_maintenance;
mod impls_query;
mod impls_relations;

pub use core::{TursoBackend, TursoConfig};
