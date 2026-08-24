//! Integration tests for Cloudflare R2 / S3 SQLite WAL Delta Replication & PITR.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use engram::sync::wal_replication::{
    compute_wal_checksum, RecoveryOptions, WalDeltaPack, WalDeltaReader, WalFrame, WalHeader,
    WalRecoveryEngine, WalReplicationError, WalReplicationStreamer, WAL_FRAME_HEADER_SIZE,
    WAL_HEADER_SIZE, WAL_MAGIC_BE, WAL_MAGIC_LE,
};
use rusqlite::Connection;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_db_path(prefix: &str) -> PathBuf {
    let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join("engram_wal_tests");
    let _ = fs::create_dir_all(&dir);
    dir.join(format!(
        "{}_{}_{}_{}.db",
        prefix,
        std::process::id(),
        now,
        count
    ))
}

fn cleanup_db(path: &Path) {
    let wal = PathBuf::from(format!("{}-wal", path.display()));
    let shm = PathBuf::from(format!("{}-shm", path.display()));
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(wal);
    let _ = fs::remove_file(shm);
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. WAL Header Parser & Checksums
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_wal_header_be_and_le_round_trip() {
    // Big-endian WAL header
    let header = WalHeader::new(4096, 42, 0x11223344, 0x55667788);
    assert_eq!(header.magic, WAL_MAGIC_BE);
    assert_eq!(header.page_size, 4096);
    assert_eq!(header.checkpoint_seq, 42);
    assert_eq!(header.salt1, 0x11223344);
    assert_eq!(header.salt2, 0x55667788);
    assert!(!header.is_little_endian_checksum);

    let serialized = header.serialize();
    assert_eq!(serialized.len(), WAL_HEADER_SIZE);
    assert!(header.verify_checksum(&serialized));

    let parsed = WalHeader::parse(&serialized).expect("parse serialized header");
    assert_eq!(parsed.magic, header.magic);
    assert_eq!(parsed.page_size, header.page_size);
    assert_eq!(parsed.checkpoint_seq, header.checkpoint_seq);
    assert_eq!(parsed.salt1, header.salt1);
    assert_eq!(parsed.salt2, header.salt2);
    assert_eq!(parsed.checksum1, header.checksum1);
    assert_eq!(parsed.checksum2, header.checksum2);
    assert!(!parsed.is_little_endian_checksum);

    // Little-endian magic (0x377f0683)
    let mut le_serialized = serialized;
    le_serialized[0..4].copy_from_slice(&WAL_MAGIC_LE.to_be_bytes());
    // In little-endian magic, checksums are parsed as LE
    let le_parsed = WalHeader::parse(&le_serialized).expect("parse LE header");
    assert_eq!(le_parsed.magic, WAL_MAGIC_LE);
    assert!(le_parsed.is_little_endian_checksum);
}

#[test]
fn test_wal_header_validation_failures() {
    // Too short
    let short_bytes = [0u8; 16];
    assert!(matches!(
        WalHeader::parse(&short_bytes),
        Err(WalReplicationError::InvalidHeader(_))
    ));

    // Invalid magic
    let mut bad_magic = WalHeader::new(4096, 1, 10, 20).serialize();
    bad_magic[0..4].copy_from_slice(&0xdeadbeef_u32.to_be_bytes());
    assert!(matches!(
        WalHeader::parse(&bad_magic),
        Err(WalReplicationError::InvalidHeader(_))
    ));

    // Invalid page size (not power of two)
    let mut bad_page_size = WalHeader::new(4096, 1, 10, 20).serialize();
    bad_page_size[8..12].copy_from_slice(&3000_u32.to_be_bytes());
    assert!(matches!(
        WalHeader::parse(&bad_page_size),
        Err(WalReplicationError::InvalidHeader(_))
    ));
}

#[test]
fn test_wal_checksum_computation() {
    let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let (c1_be, c2_be) = compute_wal_checksum(&data, false, (0, 0));
    assert_ne!(c1_be, 0);
    assert_ne!(c2_be, 0);

    let (c1_le, c2_le) = compute_wal_checksum(&data, true, (0, 0));
    assert_ne!(c1_le, 0);
    assert_ne!(c2_le, 0);
    assert_ne!(c1_be, c1_le);
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. WAL Frame Serialization & Parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_wal_frame_round_trip() {
    let page_size = 512;
    let mut dummy_data = vec![0u8; page_size as usize];
    for (i, byte) in dummy_data.iter_mut().enumerate() {
        *byte = (i % 255) as u8;
    }

    let frame = WalFrame {
        frame_index: 5,
        page_number: 2,
        db_size_pages: 10, // Commit frame
        salt1: 0xaabbccdd,
        salt2: 0xeeff0011,
        checksum1: 0x12345678,
        checksum2: 0x87654321,
        data: dummy_data.clone(),
    };

    assert!(frame.is_commit());

    let bytes = frame.serialize(false);
    assert_eq!(bytes.len(), WAL_FRAME_HEADER_SIZE + page_size as usize);

    let parsed = WalFrame::parse(&bytes, 5, page_size, false).expect("parse frame");
    assert_eq!(parsed.frame_index, 5);
    assert_eq!(parsed.page_number, 2);
    assert_eq!(parsed.db_size_pages, 10);
    assert_eq!(parsed.salt1, 0xaabbccdd);
    assert_eq!(parsed.salt2, 0xeeff0011);
    assert_eq!(parsed.checksum1, 0x12345678);
    assert_eq!(parsed.checksum2, 0x87654321);
    assert_eq!(parsed.data, dummy_data);

    // Non-commit frame (db_size_pages = 0)
    let non_commit = WalFrame {
        db_size_pages: 0,
        ..frame
    };
    assert!(!non_commit.is_commit());
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Delta Pack Serializer, Gzip Compression, and Checksums
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_delta_pack_compression_and_integrity() {
    let page_size = 4096;
    let header = WalHeader::new(page_size, 1, 100, 200);

    let mut frames = Vec::new();
    for i in 1..=4 {
        frames.push(WalFrame {
            frame_index: i,
            page_number: i,
            db_size_pages: if i == 4 { 4 } else { 0 },
            salt1: 100,
            salt2: 200,
            checksum1: i * 10,
            checksum2: i * 20,
            data: vec![(i & 0xff) as u8; page_size as usize],
        });
    }

    let delta = engram::sync::wal_replication::WalDelta {
        header: header.clone(),
        start_frame: 1,
        end_frame: 4,
        total_wal_frames: 4,
        checkpoint_seq: 1,
        frames: frames.clone(),
    };

    // Pack with gzip compression
    let pack = WalDeltaPack::pack(&delta, true, Some("workspace-test".to_string()))
        .expect("pack compressed delta");
    assert!(pack.compressed);
    assert_eq!(pack.frame_count, 4);
    assert_eq!(pack.start_frame, 1);
    assert_eq!(pack.end_frame, 4);
    assert_eq!(pack.db_identifier.as_deref(), Some("workspace-test"));

    // Payload should be compressed (smaller than raw frames json)
    let raw_len = serde_json::to_vec(&frames).unwrap().len();
    assert!(pack.payload.len() < raw_len);

    // Verify and unpack
    assert!(pack.verify_checksum().is_ok());
    let unpacked = pack.unpack_frames().expect("unpack frames");
    assert_eq!(unpacked.len(), 4);
    assert_eq!(unpacked, frames);

    // Test serialization of the whole pack
    let pack_bytes = pack.to_bytes().expect("serialize pack");
    let deserialized_pack = WalDeltaPack::from_bytes(&pack_bytes).expect("deserialize pack");
    assert_eq!(deserialized_pack.pack_id, pack.pack_id);
    assert_eq!(deserialized_pack.checksum_sha256, pack.checksum_sha256);

    // Tamper test: modifying a byte in payload must fail checksum verification
    let mut tampered = pack;
    if let Some(first) = tampered.payload.first_mut() {
        *first ^= 0xff;
    }
    assert!(matches!(
        tampered.verify_checksum(),
        Err(WalReplicationError::ChecksumMismatch { .. })
    ));
}

#[test]
fn test_wal_delta_reader_direct_extraction() {
    let db_path = temp_db_path("delta_reader_test");
    let _guard = defer_cleanup(&db_path);

    let conn = Connection::open(&db_path).expect("open connection");
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT);
        INSERT INTO notes (body) VALUES ('First entry');
        "#,
    )
    .expect("setup wal");

    let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
    let header = WalDeltaReader::read_header(&wal_path).expect("read header");
    assert_eq!(header.magic, WAL_MAGIC_BE);

    let delta = WalDeltaReader::extract_delta_frames(&wal_path, 0, None).expect("extract delta");
    assert!(!delta.frames.is_empty());
    assert_eq!(delta.start_frame, 1);
    assert_eq!(delta.end_frame, delta.total_wal_frames);
    drop(conn);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. WalReplicationStreamer with Live SQLite DB in WAL Mode
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_wal_streamer_lag_computation_and_flushing() {
    let db_path = temp_db_path("streamer_test");
    defer_cleanup(&db_path);

    // 1. Open SQLite database in WAL mode and populate data
    let conn = Connection::open(&db_path).expect("open connection");
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
        INSERT INTO users (name) VALUES ('Alice');
        INSERT INTO users (name) VALUES ('Bob');
        "#,
    )
    .expect("setup wal db");

    let mut streamer = WalReplicationStreamer::new(&db_path)
        .with_compression(true)
        .with_identifier("test-db");

    // 2. Compute lag before flush
    let lag = streamer.compute_lag().expect("compute lag");
    assert!(lag.wal_exists);
    assert!(lag.total_wal_frames > 0);
    assert_eq!(lag.last_replicated_frame, 0);
    assert_eq!(lag.unreplicated_frames, lag.total_wal_frames);
    assert!(!lag.is_synced);

    let status = streamer.status().expect("get status");
    assert_eq!(status.status, "lagging");

    // 3. Flush delta
    let delta_pack = streamer
        .flush_delta()
        .expect("flush delta")
        .expect("pack produced");
    assert!(delta_pack.frame_count > 0);
    assert_eq!(delta_pack.start_frame, 1);
    assert_eq!(delta_pack.end_frame, lag.total_wal_frames);

    // 4. Verify lag is now 0 and synced
    let post_lag = streamer.compute_lag().expect("post lag");
    assert!(post_lag.is_synced);
    assert_eq!(post_lag.unreplicated_frames, 0);
    assert_eq!(post_lag.last_replicated_frame, delta_pack.end_frame);

    let post_status = streamer.status().expect("post status");
    assert_eq!(post_status.status, "synced");

    // 5. Subsequent flush with no new transactions returns None
    let no_pack = streamer.flush_delta().expect("second flush");
    assert!(no_pack.is_none());

    // 6. Add more data and flush again
    conn.execute("INSERT INTO users (name) VALUES ('Charlie');", [])
        .expect("insert Charlie");

    let lag3 = streamer.compute_lag().expect("lag after insert");
    assert!(!lag3.is_synced);
    assert!(lag3.unreplicated_frames > 0);

    let pack2 = streamer
        .flush_delta()
        .expect("flush second delta")
        .expect("second pack");
    assert_eq!(pack2.start_frame, delta_pack.end_frame + 1);
    assert_eq!(pack2.end_frame, lag3.total_wal_frames);

    drop(conn);
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. WalRecoveryEngine: Point-In-Time Recovery (PITR) & Replay
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_point_in_time_recovery_end_to_end() {
    let source_db = temp_db_path("pitr_source");
    let base_snapshot = temp_db_path("pitr_base_snap");
    let target_t1 = temp_db_path("pitr_target_t1");
    let target_t2 = temp_db_path("pitr_target_t2");
    let target_t3 = temp_db_path("pitr_target_t3");

    defer_cleanup(&source_db);
    defer_cleanup(&base_snapshot);
    defer_cleanup(&target_t1);
    defer_cleanup(&target_t2);
    defer_cleanup(&target_t3);

    // Initialize database in WAL mode
    let conn = Connection::open(&source_db).expect("open source");
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        CREATE TABLE ledger (id INTEGER PRIMARY KEY, note TEXT, amount REAL);
        "#,
    )
    .expect("init schema");

    // Take initial base snapshot (checkpointed or clean)
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .expect("checkpoint");
    fs::copy(&source_db, &base_snapshot).expect("copy base snapshot");

    let mut streamer = WalReplicationStreamer::new(&source_db).with_compression(true);

    // Transaction 1: Add Payment 1 ($100)
    conn.execute(
        "INSERT INTO ledger (note, amount) VALUES ('Payment 1', 100.0);",
        [],
    )
    .expect("T1 insert");
    let pack_t1 = streamer.flush_delta().expect("flush T1").expect("pack T1");

    // Transaction 2: Add Payment 2 ($250)
    conn.execute(
        "INSERT INTO ledger (note, amount) VALUES ('Payment 2', 250.0);",
        [],
    )
    .expect("T2 insert");
    let pack_t2 = streamer.flush_delta().expect("flush T2").expect("pack T2");

    // Transaction 3: Add Payment 3 ($500)
    conn.execute(
        "INSERT INTO ledger (note, amount) VALUES ('Payment 3', 500.0);",
        [],
    )
    .expect("T3 insert");
    let pack_t3 = streamer.flush_delta().expect("flush T3").expect("pack T3");

    drop(conn);

    // ── Test Recovery at T1 (PITR up to pack_t1.end_frame) ──
    let report_t1 = WalRecoveryEngine::recover_from_delta_packs(
        Some(&base_snapshot),
        &target_t1,
        &[pack_t1.clone(), pack_t2.clone(), pack_t3.clone()],
        &RecoveryOptions {
            target_frame: Some(pack_t1.end_frame),
            commit_boundary_only: true,
            verify_integrity: true,
            ..Default::default()
        },
    )
    .expect("recover T1");

    assert!(report_t1.success);
    assert_eq!(report_t1.integrity_check, "ok");

    let conn_t1 = Connection::open(&target_t1).expect("open target T1");
    let count_t1: i64 = conn_t1
        .query_row("SELECT COUNT(*) FROM ledger", [], |r| r.get(0))
        .expect("count T1");
    let sum_t1: f64 = conn_t1
        .query_row("SELECT SUM(amount) FROM ledger", [], |r| r.get(0))
        .expect("sum T1");
    assert_eq!(count_t1, 1);
    assert_eq!(sum_t1, 100.0);
    drop(conn_t1);

    // ── Test Recovery at T2 (PITR up to pack_t2.end_frame) ──
    let report_t2 = WalRecoveryEngine::recover_from_delta_packs(
        Some(&base_snapshot),
        &target_t2,
        &[pack_t1.clone(), pack_t2.clone(), pack_t3.clone()],
        &RecoveryOptions {
            target_frame: Some(pack_t2.end_frame),
            commit_boundary_only: true,
            verify_integrity: true,
            ..Default::default()
        },
    )
    .expect("recover T2");

    assert!(report_t2.success);
    assert_eq!(report_t2.integrity_check, "ok");

    let conn_t2 = Connection::open(&target_t2).expect("open target T2");
    let count_t2: i64 = conn_t2
        .query_row("SELECT COUNT(*) FROM ledger", [], |r| r.get(0))
        .expect("count T2");
    let sum_t2: f64 = conn_t2
        .query_row("SELECT SUM(amount) FROM ledger", [], |r| r.get(0))
        .expect("sum T2");
    assert_eq!(count_t2, 2);
    assert_eq!(sum_t2, 350.0);
    drop(conn_t2);

    // ── Test Full Recovery to T3 ──
    let report_t3 = WalRecoveryEngine::recover_from_delta_packs(
        Some(&base_snapshot),
        &target_t3,
        &[pack_t1, pack_t2, pack_t3],
        &RecoveryOptions::default(),
    )
    .expect("recover T3");

    assert!(report_t3.success);
    assert_eq!(report_t3.integrity_check, "ok");

    let conn_t3 = Connection::open(&target_t3).expect("open target T3");
    let count_t3: i64 = conn_t3
        .query_row("SELECT COUNT(*) FROM ledger", [], |r| r.get(0))
        .expect("count T3");
    let sum_t3: f64 = conn_t3
        .query_row("SELECT SUM(amount) FROM ledger", [], |r| r.get(0))
        .expect("sum T3");
    assert_eq!(count_t3, 3);
    assert_eq!(sum_t3, 850.0);
    drop(conn_t3);
}

#[test]
fn test_direct_wal_point_in_time_recovery() {
    let source_db = temp_db_path("direct_pitr_source");
    let target_db = temp_db_path("direct_pitr_target");
    defer_cleanup(&source_db);
    defer_cleanup(&target_db);

    let conn = Connection::open(&source_db).expect("open direct source");
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        CREATE TABLE docs (id INTEGER PRIMARY KEY, title TEXT);
        INSERT INTO docs (title) VALUES ('Document 1');
        INSERT INTO docs (title) VALUES ('Document 2');
        "#,
    )
    .expect("init direct db");

    let source_wal = PathBuf::from(format!("{}-wal", source_db.display()));
    let report = WalRecoveryEngine::point_in_time_recovery(
        &source_db,
        &source_wal,
        &target_db,
        &RecoveryOptions::default(),
    )
    .expect("direct PITR");

    assert!(report.success);
    assert_eq!(report.integrity_check, "ok");

    let target_conn = Connection::open(&target_db).expect("open recovered target");
    let count: i64 = target_conn
        .query_row("SELECT COUNT(*) FROM docs", [], |r| r.get(0))
        .expect("query recovered count");
    assert_eq!(count, 2);

    drop(conn);
    drop(target_conn);
}

// ── 6. MCP Replication Tool Handlers Dispatch ──────────────────────────────

#[test]
fn test_mcp_replication_tool_handlers_dispatch() {
    use engram::embedding::{create_embedder, EmbeddingCache};
    use engram::mcp::handlers::{self, dispatch};
    use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
    use engram::storage::Storage;
    use engram::types::EmbeddingConfig;
    use parking_lot::Mutex;
    use serde_json::json;
    use std::sync::Arc;

    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("embedder");
    let ctx = handlers::HandlerContext {
        storage,
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
    };

    // 1. replication_status for in-memory database
    let status_mem = dispatch(&ctx, "replication_status", json!({}));
    assert_eq!(status_mem["status"], "in_memory");

    // 2. replication_status with a physical WAL database
    let db_path = temp_db_path("mcp_rep_src");
    let target_db = temp_db_path("mcp_rep_target");
    let _g1 = defer_cleanup(&db_path);
    let _g2 = defer_cleanup(&target_db);

    let conn = Connection::open(&db_path).expect("open connection");
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        CREATE TABLE settings (k TEXT PRIMARY KEY, v TEXT);
        INSERT INTO settings VALUES ('theme', 'dark');
        "#,
    )
    .expect("setup table");

    let db_str = db_path.to_string_lossy().to_string();
    let status_disk = dispatch(&ctx, "replication_status", json!({ "db_path": db_str }));
    assert_eq!(status_disk["status"], "lagging");

    // 3. replication_sync_now
    let sync_res = dispatch(
        &ctx,
        "replication_sync_now",
        json!({ "db_path": db_str, "compress": true }),
    );
    assert_eq!(sync_res["synced"], true);
    assert!(sync_res["pack_id"].is_string());

    // 4. replication_recover
    let recover_res = dispatch(
        &ctx,
        "replication_recover",
        json!({
            "source_db_path": db_str,
            "target_db_path": target_db.to_string_lossy().to_string(),
        }),
    );
    assert_eq!(recover_res["success"], true);
    assert_eq!(recover_res["integrity_check"], "ok");

    let target_conn = Connection::open(&target_db).expect("open target");
    let val: String = target_conn
        .query_row("SELECT v FROM settings WHERE k = 'theme'", [], |r| r.get(0))
        .expect("query setting");
    assert_eq!(val, "dark");

    drop(conn);
    drop(target_conn);
}

// ── Test Clean-up RAII Guard ─────────────────────────────────────────────────

struct CleanupGuard(PathBuf);
impl Drop for CleanupGuard {
    fn drop(&mut self) {
        cleanup_db(&self.0);
    }
}

fn defer_cleanup(path: &Path) -> CleanupGuard {
    CleanupGuard(path.to_path_buf())
}
