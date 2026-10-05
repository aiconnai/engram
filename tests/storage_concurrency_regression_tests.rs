//! C2 regression tests: migrations on open, concurrent writers and slow
//! providers must not corrupt state or hold the SQLite connection.
//!
//! Waits are bounded (generous upper bounds, no microsecond timings).

use std::path::Path;
use std::sync::{mpsc, Arc, Barrier};
use std::thread;
use std::time::Duration;

use engram::storage::migrations::SCHEMA_VERSION;
use engram::storage::Storage;
use engram::types::{StorageConfig, StorageMode};
use rusqlite::Connection;

/// Upper bound for any wait that must not block on a held lock.
const BOUNDED_WAIT: Duration = Duration::from_secs(10);

fn file_config(path: &Path) -> StorageConfig {
    StorageConfig {
        db_path: path.to_string_lossy().to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    }
}

fn schema_rows(path: &Path) -> Vec<i64> {
    let conn = Connection::open(path).expect("open raw");
    let mut stmt = conn
        .prepare("SELECT version FROM schema_version ORDER BY version")
        .expect("prepare");
    let rows = stmt
        .query_map([], |r| r.get::<_, i64>(0))
        .expect("query")
        .collect::<Result<Vec<_>, _>>()
        .expect("rows");
    rows
}

#[test]
fn test_concurrent_first_open_of_fresh_db_all_succeed() {
    const OPENERS: usize = 4;
    for round in 0..5 {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(format!("fresh-{round}.db"));
        let barrier = Arc::new(Barrier::new(OPENERS));
        let handles: Vec<_> = (0..OPENERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let config = file_config(&path);
                thread::spawn(move || {
                    barrier.wait();
                    Storage::open(config).map(|_| ()).map_err(|e| e.to_string())
                })
            })
            .collect();
        for handle in handles {
            let outcome = handle.join().expect("opener thread");
            assert!(
                outcome.is_ok(),
                "round {round}: concurrent open failed: {outcome:?}"
            );
        }
        let expected: Vec<i64> = (1..=SCHEMA_VERSION as i64).collect();
        assert_eq!(schema_rows(&path), expected, "each migration recorded once");
    }
}

#[test]
fn test_concurrent_run_migrations_record_each_version_once() {
    const RUNNERS: usize = 4;
    for round in 0..5 {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(format!("raw-{round}.db"));
        // WAL already enabled, so only the migration runner races.
        Connection::open(&path)
            .and_then(|c| c.query_row("PRAGMA journal_mode=WAL", [], |r| r.get::<_, String>(0)))
            .expect("enable wal");
        let barrier = Arc::new(Barrier::new(RUNNERS));
        let handles: Vec<_> = (0..RUNNERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let path = path.clone();
                thread::spawn(move || {
                    let conn = Connection::open(&path).map_err(|e| e.to_string())?;
                    conn.busy_timeout(BOUNDED_WAIT).map_err(|e| e.to_string())?;
                    barrier.wait();
                    engram::storage::migrations::run_migrations(&conn).map_err(|e| e.to_string())
                })
            })
            .collect();
        for handle in handles {
            let outcome = handle.join().expect("runner thread");
            assert!(
                outcome.is_ok(),
                "round {round}: concurrent migration failed: {outcome:?}"
            );
        }
        let expected: Vec<i64> = (1..=SCHEMA_VERSION as i64).collect();
        assert_eq!(schema_rows(&path), expected, "each migration recorded once");
    }
}

#[test]
fn test_concurrent_open_switching_existing_db_to_wal_succeeds() {
    const OPENERS: usize = 4;
    for round in 0..5 {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(format!("delete-mode-{round}.db"));
        {
            // Fully migrated database still in rollback-journal mode.
            let conn = Connection::open(&path).expect("open");
            conn.query_row("PRAGMA journal_mode=DELETE", [], |r| r.get::<_, String>(0))
                .expect("delete mode");
            engram::storage::migrations::run_migrations(&conn).expect("migrate");
        }
        let barrier = Arc::new(Barrier::new(OPENERS));
        let handles: Vec<_> = (0..OPENERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let config = file_config(&path);
                thread::spawn(move || {
                    barrier.wait();
                    Storage::open(config).map(|_| ()).map_err(|e| e.to_string())
                })
            })
            .collect();
        for handle in handles {
            let outcome = handle.join().expect("opener thread");
            assert!(
                outcome.is_ok(),
                "round {round}: WAL switch failed: {outcome:?}"
            );
        }
    }
}

#[test]
fn test_second_open_is_noop_and_preserves_data() {
    use engram::storage::queries::create_memory;
    use engram::types::CreateMemoryInput;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("reopen.db");
    let first = Storage::open(file_config(&path)).expect("first open");
    let id = first
        .with_transaction(|conn| {
            let input = CreateMemoryInput {
                content: "survives reopen".to_string(),
                ..Default::default()
            };
            create_memory(conn, &input).map(|m| m.id)
        })
        .expect("create");
    drop(first);
    let rows_before = schema_rows(&path);

    let second = Storage::open(file_config(&path)).expect("second open");
    let content: String = second
        .with_connection(|conn| {
            Ok(
                conn.query_row("SELECT content FROM memories WHERE id = ?", [id], |r| {
                    r.get(0)
                })?,
            )
        })
        .expect("read back");
    assert_eq!(content, "survives reopen");
    drop(second);
    assert_eq!(
        schema_rows(&path),
        rows_before,
        "second open re-ran nothing"
    );
}

#[test]
fn test_writer_waits_bounded_for_held_write_lock() {
    use engram::storage::queries::create_memory;
    use engram::types::CreateMemoryInput;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("writers.db");
    let storage = Arc::new(Storage::open(file_config(&path)).expect("open"));

    // An external writer holds the write lock, then rolls back its changes.
    let holder = Connection::open(&path).expect("holder");
    holder
        .execute_batch(
            "BEGIN IMMEDIATE;
             CREATE TABLE lock_probe (x INTEGER);
             INSERT INTO lock_probe VALUES (1);",
        )
        .expect("hold write lock");

    let (tx, rx) = mpsc::channel();
    let writer = {
        let storage = Arc::clone(&storage);
        thread::spawn(move || {
            let result = storage.with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: "second writer".to_string(),
                    ..Default::default()
                };
                create_memory(conn, &input).map(|m| m.id)
            });
            let _ = tx.send(result.map_err(|e| e.to_string()));
        })
    };

    // Still blocked while the lock is held (busy wait, not failure).
    if let Ok(early) = rx.recv_timeout(Duration::from_millis(200)) {
        panic!("writer returned while the lock was held (should wait): {early:?}");
    }
    holder.execute_batch("ROLLBACK;").expect("release lock");
    let id = rx
        .recv_timeout(BOUNDED_WAIT)
        .expect("writer finished within bound")
        .expect("writer succeeded after lock release");
    writer.join().expect("writer thread");

    let (count, probe_tables): (i64, i64) = storage
        .with_connection(|conn| {
            Ok(conn.query_row(
                "SELECT (SELECT COUNT(*) FROM memories),
                        (SELECT COUNT(*) FROM sqlite_master WHERE name = 'lock_probe')",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?)
        })
        .expect("count");
    assert_eq!(
        (count, probe_tables),
        (1, 0),
        "rollback left nothing behind"
    );
    assert!(id >= 1);
}

#[test]
fn test_memory_ids_are_not_reused_after_delete_or_rollback() {
    use engram::error::EngramError;
    use engram::storage::queries::create_memory;
    use engram::types::CreateMemoryInput;

    let storage = Storage::open_in_memory().expect("open");
    let create = |content: &str| {
        storage.with_transaction(|conn| {
            let input = CreateMemoryInput {
                content: content.to_string(),
                ..Default::default()
            };
            create_memory(conn, &input).map(|m| m.id)
        })
    };
    let first = create("first").expect("first");
    let second = create("second").expect("second");

    // Failing transaction: nothing persists.
    let failed: Result<i64, EngramError> = storage.with_transaction(|conn| {
        let input = CreateMemoryInput {
            content: "doomed".to_string(),
            ..Default::default()
        };
        create_memory(conn, &input)?;
        Err(EngramError::Internal(
            "injected failure after insert".to_string(),
        ))
    });
    assert!(failed.is_err());

    // Hard-delete the highest committed id; AUTOINCREMENT must not reuse it.
    storage
        .with_connection(|conn| {
            conn.execute("DELETE FROM memories WHERE id = ?", [second])?;
            Ok(())
        })
        .expect("hard delete");
    let third = create("third").expect("third");
    assert!(
        first < second && second < third,
        "ids {first} {second} {third}"
    );
    let doomed: i64 = storage
        .with_connection(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE content = 'doomed'",
                [],
                |r| r.get(0),
            )?)
        })
        .expect("count doomed");
    assert_eq!(doomed, 0);
}

// ── Slow provider must not hold the SQLite connection ────────────────────────

/// Embedder that blocks until released, signalling when it was entered.
struct BlockingEmbedder {
    entered: parking_lot::Mutex<Option<mpsc::Sender<()>>>,
    release: parking_lot::Mutex<mpsc::Receiver<()>>,
}

impl engram::embedding::Embedder for BlockingEmbedder {
    fn embed(&self, _text: &str) -> engram::error::Result<Vec<f32>> {
        if let Some(tx) = self.entered.lock().take() {
            let _ = tx.send(());
        }
        // Bounded: never hangs the suite even if the test forgets to release.
        let _ = self.release.lock().recv_timeout(BOUNDED_WAIT);
        Ok(vec![0.25; 8])
    }

    fn dimensions(&self) -> usize {
        8
    }

    fn model_name(&self) -> &str {
        "blocking-mock"
    }
}

fn handler_context(
    storage: Storage,
    embedder: Arc<dyn engram::embedding::Embedder>,
) -> engram::mcp::handlers::HandlerContext {
    use engram::embedding::EmbeddingCache;
    use engram::search::{
        AdaptiveCacheConfig, FuzzyEngine, HnswConfig, HnswIndex, SearchConfig, SearchResultCache,
        VectorMetric,
    };
    let dims = embedder.dimensions();
    engram::mcp::handlers::HandlerContext {
        storage,
        embedder,
        fuzzy_engine: Arc::new(parking_lot::Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(HnswIndex::new(HnswConfig::new(
            dims,
            VectorMetric::Cosine,
        )))),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        progress_reporter: None,
        principal: None,
    }
}

#[test]
fn test_slow_embedder_does_not_hold_connection() {
    use engram::storage::queries::create_memory;
    use engram::types::CreateMemoryInput;
    use serde_json::json;

    let dir = tempfile::tempdir().expect("tempdir");
    let storage = Storage::open(file_config(&dir.path().join("slow.db"))).expect("open");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let embedder: Arc<dyn engram::embedding::Embedder> = Arc::new(BlockingEmbedder {
        entered: parking_lot::Mutex::new(Some(entered_tx)),
        release: parking_lot::Mutex::new(release_rx),
    });

    let creator = {
        let storage = storage.clone();
        thread::spawn(move || {
            let ctx = handler_context(storage, embedder);
            engram::mcp::handlers::dispatch(
                &ctx,
                "memory_create",
                json!({"content": "slow provider memory", "defer_embedding": false}),
            )
        })
    };
    entered_rx
        .recv_timeout(BOUNDED_WAIT)
        .expect("provider was called");

    // While the provider is blocked, a second writer on the same Storage
    // (same connection mutex) must complete within a bounded wait.
    let (done_tx, done_rx) = mpsc::channel();
    let second = {
        let storage = storage.clone();
        thread::spawn(move || {
            let result = storage.with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: "written during slow provider".to_string(),
                    ..Default::default()
                };
                create_memory(conn, &input).map(|m| m.id)
            });
            let _ = done_tx.send(result.map_err(|e| e.to_string()));
        })
    };
    let second_id = done_rx
        .recv_timeout(BOUNDED_WAIT)
        .expect("second writer not blocked by slow provider")
        .expect("second writer succeeded");
    second.join().expect("second writer thread");

    release_tx.send(()).expect("release provider");
    let created = creator.join().expect("creator thread");
    let first_id = created["id"].as_i64().expect("memory_create returned id");
    assert_ne!(first_id, second_id);

    let (memories, embeddings): (i64, i64) = storage
        .with_connection(|conn| {
            Ok(conn.query_row(
                "SELECT (SELECT COUNT(*) FROM memories),
                        (SELECT COUNT(*) FROM embeddings WHERE memory_id = ?)",
                [first_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?)
        })
        .expect("counts");
    assert_eq!(memories, 2);
    assert_eq!(embeddings, 1, "embedding stored after provider returned");
}
