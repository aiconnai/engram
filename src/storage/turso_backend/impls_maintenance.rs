//! Stats/health/maintenance method bodies for `impl StorageBackend for
//! TursoBackend`, plus the (small, whole) `TransactionalBackend` and
//! `CloudSyncBackend` trait impls.
//!
//! Split out of `impls.rs` (ENG storage split) to keep files under the
//! repository's line-count limit. The `StorageBackend` methods here are
//! `pub(super)` free functions called directly from the trait method stubs
//! in `impls.rs`; behavior is unchanged from the original single-file
//! implementation.

use std::collections::HashMap;
use std::time::Instant;

use super::core::TursoBackend;
use crate::error::{EngramError, Result};
use crate::storage::backend::{
    validate_savepoint_name, CloudSyncBackend, DerivedIndexHealth, DerivedIndexStatus,
    HealthStatus, StorageBackend, StorageStats, SyncDelta, SyncResult, SyncState,
    TransactionalBackend,
};

pub(super) fn get_stats(backend: &TursoBackend) -> Result<StorageStats> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.read().await;

            let memory_count: i64 = conn
                .query("SELECT COUNT(*) FROM memories WHERE valid_to IS NULL", ())
                .await
                .ok()
                .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                .map(|r| r.get(0).unwrap_or(0))
                .unwrap_or(0);

            let crossref_count: i64 = conn
                .query("SELECT COUNT(*) FROM crossrefs WHERE valid_to IS NULL", ())
                .await
                .ok()
                .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                .map(|r| r.get(0).unwrap_or(0))
                .unwrap_or(0);

            let tag_count: i64 = conn
                .query("SELECT COUNT(DISTINCT tag_id) FROM memory_tags", ())
                .await
                .ok()
                .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                .map(|r| r.get(0).unwrap_or(0))
                .unwrap_or(0);

            let schema_version: i32 = conn
                .query("SELECT COALESCE(MAX(version), 0) FROM schema_version", ())
                .await
                .ok()
                .and_then(|mut r| futures::executor::block_on(r.next()).ok().flatten())
                .map(|r| r.get(0).unwrap_or(0))
                .unwrap_or(0);

            Ok(StorageStats {
                total_memories: memory_count,
                total_tags: tag_count,
                total_crossrefs: crossref_count,
                total_versions: 0,
                total_identities: 0,
                total_entities: 0,
                db_size_bytes: 0,
                memories_with_embeddings: 0,
                memories_pending_embedding: 0,
                last_sync: None,
                sync_pending: false,
                storage_mode: "turso".to_string(),
                schema_version,
                workspaces: HashMap::new(),
                type_counts: HashMap::new(),
                tier_counts: HashMap::new(),
            })
        })
    })
}

pub(super) fn health_check(backend: &TursoBackend) -> Result<HealthStatus> {
    let start = Instant::now();

    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    let result = tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.read().await;
            conn.query("SELECT 1", ()).await
        })
    });

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(_) => Ok(HealthStatus {
            healthy: true,
            latency_ms,
            error: None,
            details: HashMap::from([
                ("backend".to_string(), "turso".to_string()),
                ("url".to_string(), backend.config.url.clone()),
            ]),
            derived_indexes: backend
                .get_stats()
                .ok()
                .map(|stats| {
                    vec![DerivedIndexHealth::external(
                        "memories",
                        DerivedIndexStatus::Healthy,
                        stats.total_memories,
                        stats.total_memories,
                        HashMap::from([("index".to_string(), "memories".to_string())]),
                    )]
                })
                .unwrap_or_else(|| {
                    vec![DerivedIndexHealth::external(
                        "memories",
                        DerivedIndexStatus::Unavailable,
                        0,
                        0,
                        HashMap::from([
                            ("index".to_string(), "memories".to_string()),
                            (
                                "error".to_string(),
                                "failed to read index stats".to_string(),
                            ),
                        ]),
                    )]
                }),
        }),
        Err(e) => Ok(HealthStatus {
            healthy: false,
            latency_ms,
            error: Some(e.to_string()),
            details: HashMap::from([
                ("backend".to_string(), "turso".to_string()),
                ("url".to_string(), backend.config.url.clone()),
            ]),
            derived_indexes: vec![DerivedIndexHealth::external(
                "memories",
                DerivedIndexStatus::Unavailable,
                0,
                0,
                HashMap::from([
                    ("index".to_string(), "memories".to_string()),
                    ("error".to_string(), e.to_string()),
                ]),
            )],
        }),
    }
}

pub(super) fn optimize(backend: &TursoBackend) -> Result<()> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.write().await;
            conn.execute("VACUUM", ())
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?;
            Ok(())
        })
    })
}

pub(super) fn schema_version(backend: &TursoBackend) -> Result<i32> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let conn = backend.conn.read().await;
            let version: i32 = conn
                .query("SELECT COALESCE(MAX(version), 0) FROM schema_version", ())
                .await
                .map_err(|e| EngramError::Storage(e.to_string()))?
                .next()
                .await
                .ok()
                .flatten()
                .map(|r| r.get(0).unwrap_or(0))
                .unwrap_or(0);
            Ok(version)
        })
    })
}

impl TransactionalBackend for TursoBackend {
    fn with_transaction<F, T>(&self, _f: F) -> Result<T>
    where
        F: FnOnce(&dyn StorageBackend) -> Result<T>,
    {
        Err(EngramError::Storage(
            "TursoBackend::with_transaction is unsupported: callers should use real \
             Turso/libSQL transaction APIs until a transaction-scoped StorageBackend exists"
                .to_string(),
        ))
    }

    fn savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute(&format!("SAVEPOINT {}", name), ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }

    fn release_savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute(&format!("RELEASE SAVEPOINT {}", name), ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }

    fn rollback_to_savepoint(&self, name: &str) -> Result<()> {
        let name = validate_savepoint_name(name)?;
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let conn = self.conn.write().await;
                conn.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), ())
                    .await
                    .map_err(|e| EngramError::Storage(e.to_string()))?;
                Ok(())
            })
        })
    }
}

impl CloudSyncBackend for TursoBackend {
    fn push(&self) -> Result<SyncResult> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;
        tokio::task::block_in_place(|| rt.block_on(self.sync()))
    }

    fn pull(&self) -> Result<SyncResult> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;
        tokio::task::block_in_place(|| rt.block_on(self.sync()))
    }

    fn sync_delta(&self, _since_version: u64) -> Result<SyncDelta> {
        Err(EngramError::Sync(
            "Turso backend does not expose CloudSyncBackend::sync_delta; embedded replica \
             synchronization is handled by sync()"
                .to_string(),
        ))
    }

    fn sync_state(&self) -> Result<SyncState> {
        Err(EngramError::Sync(
            "Turso backend does not expose CloudSyncBackend::sync_state; embedded replica \
             synchronization is handled by sync()"
                .to_string(),
        ))
    }

    fn force_sync(&self) -> Result<SyncResult> {
        self.push()
    }
}
