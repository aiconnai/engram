#![allow(deprecated)]

//! Turso/libSQL implementation of the StorageBackend trait (Phase 6 - ENG-54)
//!
//! # Deprecation Notice
//!
//! The Turso backend is **deprecated** as of v0.22.0 and scheduled for sunset
//! in v0.24.0 (see `docs/adr/2026-08-16-turso-backend-deprecation-and-sunset.md`).
//!
//! Engram's canonical architecture uses local SQLite with WAL mode and S3/R2
//! snapshot attestation for cloud persistence.
//!
//! # Features (Historical / Experimental)
//!
//! - **Embedded replicas**: Local SQLite with sync to Turso cloud
//! - **Edge-native**: Sub-millisecond reads from local replica
//! - **Sync on demand**: Push/pull changes to cloud
//! - **Compatible schema**: Same migrations as SQLite backend

mod core;
mod impls;
mod impls_crud;
mod impls_maintenance;
mod impls_query;
mod impls_relations;

pub use core::{TursoBackend, TursoConfig};
