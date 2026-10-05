//! G1 regression: Engram must never drop the POSIX locks SQLite holds on a
//! live database.
//!
//! POSIX (fcntl) advisory locks belong to the (process, inode) pair. Closing
//! ANY descriptor of the database or its `-shm` drops every lock the process
//! holds on that file, including SQLite's, behind the back of SQLite's unix
//! VFS. Another process that then opens and closes the database believes it
//! is the last connection: it checkpoints and deletes the live WAL. Commits
//! made afterwards by the still-open process go to the unlinked WAL and are
//! lost on restart.
//!
//! The scenario needs a second process, because SQLite tracks locks per inode
//! inside one process and would not attempt the WAL deletion in-process. The
//! test binary re-executes itself (`child_storage_helper`, `#[ignore]`d so it
//! only runs when invoked explicitly) to act as that second process; all
//! files live in a caller-owned temp dir.

#![cfg(unix)]

use std::path::Path;
use std::process::Command;

use engram::storage::Storage;
use engram::types::{StorageConfig, StorageMode};

/// Set only when this binary re-executes itself as the second process.
const CHILD_DB_ENV: &str = "ENGRAM_G1_CHILD_DB";
/// `touch` (open, read, close) or `count` (print the probe row count).
const CHILD_MODE_ENV: &str = "ENGRAM_G1_CHILD_MODE";
const COUNT_PREFIX: &str = "G1_PROBE_COUNT=";
const CHILD_TEST_NAME: &str = "child_storage_helper";

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

fn probe_count(storage: &Storage) -> i64 {
    storage
        .with_connection(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM g1_probe", [], |r| r.get(0))?)
        })
        .expect("count probe rows")
}

fn insert_probe(storage: &Storage, value: &str) {
    storage
        .with_transaction(|conn| {
            conn.execute("INSERT INTO g1_probe (value) VALUES (?1)", [value])?;
            Ok(())
        })
        .expect("insert probe row");
}

/// Second-process helper, run only via `run_child` (`--ignored --exact`).
#[test]
#[ignore = "second-process helper for the G1 tests; run by run_child"]
fn child_storage_helper() {
    let Ok(db_path) = std::env::var(CHILD_DB_ENV) else {
        return;
    };
    let storage = Storage::open(file_config(Path::new(&db_path))).expect("child open");
    let count = probe_count(&storage);
    if std::env::var(CHILD_MODE_ENV).as_deref() == Ok("count") {
        println!("{COUNT_PREFIX}{count}");
    }
    // Dropping the last handle closes the connection: if this process
    // believes it is the last connection, SQLite checkpoints and deletes
    // the WAL here.
    drop(storage);
}

/// Run `child_storage_helper` in a separate process; returns its stdout.
fn run_child(db_path: &Path, mode: &str) -> String {
    let exe = std::env::current_exe().expect("test binary path");
    let output = Command::new(exe)
        .args([
            CHILD_TEST_NAME,
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_DB_ENV, db_path)
        .env(CHILD_MODE_ENV, mode)
        .output()
        .expect("spawn child process");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "child ({mode}) failed: {}\nstdout:\n{stdout}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("1 passed"),
        "child ({mode}) did not run the helper test:\n{stdout}"
    );
    stdout
}

/// Row count as seen by a fresh process (what a restart would see).
fn count_in_fresh_process(db_path: &Path) -> i64 {
    let stdout = run_child(db_path, "count");
    stdout
        .lines()
        .find_map(|line| line.split_once(COUNT_PREFIX))
        .and_then(|(_, n)| n.trim().parse().ok())
        .unwrap_or_else(|| panic!("no probe count in child output:\n{stdout}"))
}

fn open_with_probe_table(db_path: &Path) -> Storage {
    let storage = Storage::open(file_config(db_path)).expect("open storage");
    storage
        .with_connection(|conn| {
            conn.execute_batch("CREATE TABLE IF NOT EXISTS g1_probe (value TEXT NOT NULL);")?;
            Ok(())
        })
        .expect("create probe table");
    storage
}

/// Commits made after another process opened and closed the database must
/// stay visible to a fresh process while this process is still running.
fn assert_writes_survive_foreign_open_close(storage: &Storage, db_path: &Path) {
    let base = probe_count(storage);
    insert_probe(storage, "before-foreign-open");
    assert_eq!(probe_count(storage), base + 1);

    // Another process (CLI, hook client, test reader) opens and closes.
    run_child(db_path, "touch");

    // The still-live process commits again.
    insert_probe(storage, "after-foreign-open");
    assert_eq!(
        probe_count(storage),
        base + 2,
        "own connection sees both commits"
    );

    // A fresh process (equivalent to a restart without a clean checkpoint)
    // must see every committed row.
    assert_eq!(
        count_in_fresh_process(db_path),
        base + 2,
        "commit made after a foreign open/close was lost (live WAL deleted)"
    );
    assert!(
        Path::new(&format!("{}-wal", db_path.display())).exists(),
        "live WAL was deleted by another process"
    );
}

#[test]
fn committed_writes_survive_another_process_open_and_close() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("memory.db");
    let storage = open_with_probe_table(&db_path);

    assert_writes_survive_foreign_open_close(&storage, &db_path);
}

#[test]
fn second_in_process_open_does_not_drop_the_first_handles_locks() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("memory.db");
    let storage = open_with_probe_table(&db_path);

    // A second handle on the same, already-existing file in this process
    // (StoragePool, a second Storage::open) must not close descriptors that
    // SQLite does not own either.
    let second = Storage::open(file_config(&db_path)).expect("second in-process open");
    drop(second);

    assert_writes_survive_foreign_open_close(&storage, &db_path);
}

// ── MCP tools that take caller-controlled paths ─────────────────────────────

mod mcp_paths {
    use super::*;
    use std::sync::Arc;

    use engram::embedding::{create_embedder, EmbeddingCache};
    use engram::mcp::handlers::{self, HandlerContext};
    use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
    use engram::types::EmbeddingConfig;
    use parking_lot::Mutex;
    use serde_json::{json, Value};

    fn handler_context(storage: &Storage) -> HandlerContext {
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        HandlerContext {
            storage: storage.clone(),
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
                engram::search::HnswConfig::new(
                    embedder.dimensions(),
                    engram::search::VectorMetric::Cosine,
                ),
            ))),
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

    fn assert_refused_as_active(output: &Value, what: &str) {
        let text = output.to_string();
        assert!(
            text.contains("active Engram database"),
            "{what}: expected an active-database refusal, got {output}"
        );
    }

    fn setup() -> (
        tempfile::TempDir,
        std::path::PathBuf,
        Storage,
        HandlerContext,
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("memory.db");
        let storage = open_with_probe_table(&db_path);
        insert_probe(&storage, "seed");
        let ctx = handler_context(&storage);
        (dir, db_path, storage, ctx)
    }

    #[test]
    fn replication_recover_default_source_keeps_locks_and_recovers_latest_state() {
        let (dir, db_path, storage, ctx) = setup();
        let target = dir.path().join("recovered.db");

        let report = handlers::dispatch(
            &ctx,
            "replication_recover",
            json!({"target_db_path": target.to_string_lossy()}),
        );

        // Lock check first, so a pre-fix run shows the data loss itself.
        assert_writes_survive_foreign_open_close(&storage, &db_path);
        assert_eq!(report["success"], json!(true), "recover: {report}");
        let recovered = rusqlite::Connection::open(&target).expect("open recovered");
        let rows: i64 = recovered
            .query_row("SELECT COUNT(*) FROM g1_probe", [], |r| r.get(0))
            .expect("recovered probe rows");
        assert_eq!(rows, 1, "recovered copy holds the committed seed row");
    }

    #[test]
    fn replication_recover_refuses_active_target_and_point_in_time() {
        let (dir, db_path, storage, ctx) = setup();
        let target = db_path.to_string_lossy().to_string();

        let into_active = handlers::dispatch(
            &ctx,
            "replication_recover",
            json!({"target_db_path": target, "source_db_path": "/nonexistent/source.db"}),
        );
        let pitr = handlers::dispatch(
            &ctx,
            "replication_recover",
            json!({
                "target_db_path": dir.path().join("pitr.db").to_string_lossy(),
                "target_frame": 1
            }),
        );

        assert_writes_survive_foreign_open_close(&storage, &db_path);
        assert_refused_as_active(&into_active, "active target");
        assert!(
            pitr["error"]
                .as_str()
                .is_some_and(|e| e.contains("not supported while it is open")),
            "point-in-time from the active database: {pitr}"
        );
    }

    #[test]
    fn document_ingest_refuses_an_alias_of_the_active_database() {
        let (dir, db_path, storage, ctx) = setup();
        let alias = dir.path().join("notes.md");
        std::os::unix::fs::symlink(&db_path, &alias).expect("symlink alias");

        let output = handlers::dispatch(
            &ctx,
            "memory_ingest_document",
            json!({"path": alias.to_string_lossy()}),
        );

        assert_writes_survive_foreign_open_close(&storage, &db_path);
        assert_refused_as_active(&output, "memory_ingest_document");
    }

    /// DuckDB's `sqlite` scanner links its own SQLite copy; attaching the
    /// active file and closing it would drop this process's locks.
    #[cfg(feature = "duckdb-graph")]
    #[test]
    fn duckdb_graph_tools_do_not_attach_the_active_database() {
        let (_dir, db_path, storage, ctx) = setup();

        let mut outputs = Vec::new();
        for (tool, args) in [
            (
                "memory_graph_path",
                json!({"scope": "global", "source_id": 1, "target_id": 2}),
            ),
            (
                "memory_temporal_snapshot",
                json!({"scope": "global", "timestamp": "2024-06-01"}),
            ),
        ] {
            outputs.push((tool, handlers::dispatch(&ctx, tool, args)));
        }

        assert_writes_survive_foreign_open_close(&storage, &db_path);
        for (tool, output) in outputs {
            assert!(
                !output.to_string().contains("failed to open DuckDB graph"),
                "{tool}: {output}"
            );
        }
    }

    /// `/dev/fd/N` names an open descriptor of the database. On macOS its
    /// `stat` reports the devfs device, so a device + inode check alone
    /// misses it; opening and closing it drops this process's locks.
    #[cfg(feature = "multimodal")]
    #[test]
    fn descriptor_alias_paths_are_refused_or_snapshotted() {
        use std::os::unix::io::AsRawFd;

        let (dir, db_path, storage, ctx) = setup();
        // Held open (not closed) until the lock checks are done.
        let held = std::fs::File::open(&db_path).expect("open descriptor");
        let alias = format!("/dev/fd/{}", held.as_raw_fd());

        let media = handlers::dispatch(&ctx, "memory_ingest_media", json!({"media_path": alias}));
        let recover = handlers::dispatch(
            &ctx,
            "replication_recover",
            json!({
                "target_db_path": dir.path().join("from-fd.db").to_string_lossy(),
                "source_db_path": alias
            }),
        );

        assert_writes_survive_foreign_open_close(&storage, &db_path);
        assert!(
            media.to_string().contains("refusing to use"),
            "memory_ingest_media via {alias}: {media}"
        );
        // Either refused, or recognized as the active DB and snapshotted
        // through SQLite; never copied through the descriptor alias.
        assert!(
            recover["success"] == json!(true) || recover.to_string().contains("refusing to use"),
            "replication_recover via {alias}: {recover}"
        );
        drop(held);
    }

    #[cfg(feature = "multimodal")]
    #[test]
    fn media_ingest_refuses_the_active_database() {
        let (_dir, db_path, storage, ctx) = setup();

        let output = handlers::dispatch(
            &ctx,
            "memory_ingest_media",
            json!({"media_path": db_path.to_string_lossy()}),
        );

        assert_writes_survive_foreign_open_close(&storage, &db_path);
        assert_refused_as_active(&output, "memory_ingest_media");
    }
}
