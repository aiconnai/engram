//! Keeps the in-memory HNSW index consistent with SQLite.
//!
//! Memories leave the searchable set through many paths: soft delete, TTL
//! expiry, supersede / merge / consolidation (`valid_to`), garbage collection
//! (hard `DELETE`) and content replacement (`has_embedding` reset until the
//! re-embedding job completes). Wiring a removal call into each of them is
//! fragile, so the database is the single source of truth and this module
//! reconciles the index against it: any indexed id that is no longer a live,
//! embedded, unexpired memory is tombstoned, then dead nodes are compacted away
//! once they exceed a bounded share of the graph.
//!
//! Search itself already re-validates every candidate against SQLite (see the
//! hydration step in `hybrid.rs`), so a not-yet-reconciled entry can only waste
//! a candidate slot, never surface a deleted memory.

use chrono::Utc;
use rusqlite::{params_from_iter, Connection};
use std::collections::HashSet;

use crate::error::Result;
use crate::search::hnsw::HnswIndex;

/// Do not rebuild the graph for fewer dead nodes than this.
pub const HNSW_COMPACT_MIN_TOMBSTONES: usize = 64;
/// Rebuild the graph once dead nodes reach this share of all nodes.
pub const HNSW_COMPACT_MAX_TOMBSTONE_RATIO: f32 = 0.25;

/// Ids checked per SQL statement (well below SQLite's variable limit).
const RECONCILE_CHUNK: usize = 500;

/// Outcome of one [`sync_hnsw_with_db`] pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HnswSyncReport {
    /// Stale entries tombstoned because the memory is gone, expired,
    /// superseded or awaiting a fresh embedding.
    pub removed: usize,
    /// Whether the graph was rebuilt to drop tombstoned nodes.
    pub compacted: bool,
}

/// Tombstone every indexed id that is not a live, embedded, unexpired memory.
/// Returns how many entries were removed.
pub fn reconcile_hnsw_with_db(conn: &Connection, hnsw: &mut HnswIndex<i64>) -> Result<usize> {
    let now = Utc::now().to_rfc3339();
    let indexed = hnsw.live_ids();
    let mut removed = 0;

    for chunk in indexed.chunks(RECONCILE_CHUNK) {
        let placeholders = vec!["?"; chunk.len()].join(", ");
        let sql = format!(
            "SELECT id FROM memories
             WHERE valid_to IS NULL AND has_embedding = 1
               AND (expires_at IS NULL OR expires_at > ?)
               AND id IN ({placeholders})"
        );
        let mut args: Vec<rusqlite::types::Value> = Vec::with_capacity(chunk.len() + 1);
        args.push(now.clone().into());
        args.extend(chunk.iter().map(|id| (*id).into()));

        let mut stmt = conn.prepare(&sql)?;
        let live: HashSet<i64> = stmt
            .query_map(params_from_iter(args), |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;

        for id in chunk {
            if !live.contains(id) && hnsw.remove(id) {
                removed += 1;
            }
        }
    }
    Ok(removed)
}

/// Reconcile the index with the database, then compact if tombstones piled up.
///
/// Lock order for callers: storage connection first, then the index write lock.
pub fn sync_hnsw_with_db(conn: &Connection, hnsw: &mut HnswIndex<i64>) -> Result<HnswSyncReport> {
    let removed = reconcile_hnsw_with_db(conn, hnsw)?;
    let compacted = hnsw.compact_if_needed(
        HNSW_COMPACT_MIN_TOMBSTONES,
        HNSW_COMPACT_MAX_TOMBSTONE_RATIO,
    );
    Ok(HnswSyncReport { removed, compacted })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::{enqueue_embedding_job, persist_computed_embedding};
    use crate::search::hnsw::HnswConfig;
    use crate::search::VectorMetric;
    use crate::storage::queries::{cleanup_expired_memories, create_memory};
    use crate::storage::Storage;
    use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};

    const DIM: usize = 4;

    fn new_index() -> HnswIndex<i64> {
        HnswIndex::new(HnswConfig::new(DIM, VectorMetric::Cosine))
    }

    fn vector(seed: usize) -> Vec<f32> {
        let mut v = vec![0.1; DIM];
        v[seed % DIM] = 1.0;
        v
    }

    /// Create an embedded memory (row + `has_embedding`) and index its vector.
    fn embedded_memory(
        conn: &Connection,
        hnsw: &mut HnswIndex<i64>,
        content: &str,
        ttl_seconds: Option<i64>,
    ) -> i64 {
        let input = CreateMemoryInput {
            content: content.to_string(),
            memory_type: MemoryType::Todo,
            tags: vec![],
            metadata: Default::default(),
            importance: None,
            scope: Default::default(),
            workspace: None,
            tier: if ttl_seconds.is_some() {
                MemoryTier::Daily
            } else {
                MemoryTier::Permanent
            },
            defer_embedding: true,
            ttl_seconds,
            dedup_mode: Default::default(),
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        };
        let id = create_memory(conn, &input).expect("create").id;
        enqueue_embedding_job(conn, id).expect("enqueue");
        let embedding = vector(id as usize);
        assert!(
            persist_computed_embedding(conn, id, content, &embedding, "test").expect("persist")
        );
        hnsw.insert(id, &embedding);
        id
    }

    #[test]
    fn reconcile_keeps_live_memories() {
        let storage = Storage::open_in_memory().unwrap();
        let mut hnsw = new_index();
        let id = storage
            .with_connection(|conn| Ok(embedded_memory(conn, &mut hnsw, "keep me", None)))
            .unwrap();
        let report = storage
            .with_connection(|conn| sync_hnsw_with_db(conn, &mut hnsw))
            .unwrap();
        assert_eq!(report, HnswSyncReport::default());
        assert!(hnsw.contains(&id));
    }

    #[test]
    fn expiry_cleanup_is_reflected_in_the_index() {
        let storage = Storage::open_in_memory().unwrap();
        let mut hnsw = new_index();
        let (keep, expire) = storage
            .with_connection(|conn| {
                let keep = embedded_memory(conn, &mut hnsw, "long lived", None);
                let expire = embedded_memory(conn, &mut hnsw, "short lived", Some(3600));
                conn.execute(
                    "UPDATE memories SET expires_at = '2000-01-01T00:00:00+00:00' WHERE id = ?",
                    [expire],
                )?;
                Ok((keep, expire))
            })
            .unwrap();
        storage.with_transaction(cleanup_expired_memories).unwrap();

        let report = storage
            .with_connection(|conn| sync_hnsw_with_db(conn, &mut hnsw))
            .unwrap();
        assert_eq!(report.removed, 1);
        assert!(hnsw.contains(&keep));
        assert!(!hnsw.contains(&expire));
    }

    #[test]
    fn superseded_hard_deleted_and_content_replaced_memories_leave_the_index() {
        let storage = Storage::open_in_memory().unwrap();
        let mut hnsw = new_index();
        let (keep, superseded, purged, replaced) = storage
            .with_connection(|conn| {
                let keep = embedded_memory(conn, &mut hnsw, "stays", None);
                let superseded = embedded_memory(conn, &mut hnsw, "old version", None);
                let purged = embedded_memory(conn, &mut hnsw, "garbage collected", None);
                let replaced = embedded_memory(conn, &mut hnsw, "rewritten by compression", None);
                // Supersede / merge / consolidation close the row (valid_to).
                conn.execute(
                    "UPDATE memories SET valid_to = '2020-01-01T00:00:00+00:00' WHERE id = ?",
                    [superseded],
                )?;
                // Gardening hard-deletes.
                conn.execute("DELETE FROM memories WHERE id = ?", [purged])?;
                // Content replacement clears has_embedding until re-embedded.
                conn.execute(
                    "UPDATE memories SET has_embedding = 0 WHERE id = ?",
                    [replaced],
                )?;
                Ok((keep, superseded, purged, replaced))
            })
            .unwrap();

        let report = storage
            .with_connection(|conn| sync_hnsw_with_db(conn, &mut hnsw))
            .unwrap();
        assert_eq!(report.removed, 3);
        assert!(hnsw.contains(&keep));
        for gone in [superseded, purged, replaced] {
            assert!(
                !hnsw.contains(&gone),
                "memory {gone} should leave the index"
            );
        }
    }

    #[test]
    fn compaction_runs_only_past_the_tombstone_threshold() {
        let mut hnsw = new_index();
        for id in 0..100_i64 {
            hnsw.insert(id, &vector(id as usize));
        }
        for id in 0..10_i64 {
            hnsw.remove(&id);
        }
        // 10 dead of 100 nodes: below the ratio, nothing to do.
        assert!(!hnsw.compact_if_needed(5, 0.25));
        assert_eq!(hnsw.tombstone_count(), 10);

        for id in 10..30_i64 {
            hnsw.remove(&id);
        }
        // 30 dead of 100: over the ratio but under the absolute floor.
        assert!(!hnsw.compact_if_needed(64, 0.25));
        // Over both: rebuilt without the dead nodes, live set preserved.
        assert!(hnsw.compact_if_needed(5, 0.25));
        assert_eq!(hnsw.tombstone_count(), 0);
        assert_eq!(hnsw.len(), 70);
        assert!(hnsw.contains(&99) && !hnsw.contains(&0));
    }
}
