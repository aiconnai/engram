//! O1: the background worker logs the class of a provider failure, never the
//! provider body (which can echo the memory text); the failure itself is still
//! returned to the caller and recorded on the job.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use rusqlite::Connection;

use super::{EmbeddingQueue, EmbeddingRequest, EmbeddingWorker};
use crate::embedding::Embedder;
use crate::error::{EngramError, Result};
use crate::observability::test_capture::{assert_logs_exclude, captured, install};

const BODY: &str = "worker-provider-body-sentinel-6b7c";
const TEXT: &str = "worker-private-text-sentinel-8d9e";

struct EchoingProvider;

impl Embedder for EchoingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Err(EngramError::Embedding(format!(
            "API error 500: {BODY} echo={text}"
        )))
    }

    fn dimensions(&self) -> usize {
        8
    }

    fn model_name(&self) -> &str {
        "echoing-provider-test"
    }
}

#[tokio::test]
async fn worker_failure_log_has_class_but_not_provider_body_or_memory_text() {
    install();
    let conn = Connection::open_in_memory().expect("db");
    conn.execute(
        "CREATE TABLE embedding_queue (
            memory_id INTEGER PRIMARY KEY, status TEXT NOT NULL, started_at TEXT,
            completed_at TEXT, error TEXT, retry_count INTEGER NOT NULL DEFAULT 0)",
        [],
    )
    .expect("table");
    conn.execute(
        "INSERT INTO embedding_queue (memory_id, status, retry_count) VALUES (1, 'pending', 0)",
        [],
    )
    .expect("row");
    let worker = EmbeddingWorker {
        embedder: Arc::new(EchoingProvider),
        queue: EmbeddingQueue::new(1),
        conn: Arc::new(Mutex::new(conn)),
        batch_size: 1,
        batch_timeout: Duration::from_secs(1),
    };
    let mut batch = vec![EmbeddingRequest {
        memory_id: 1,
        content: TEXT.to_string(),
    }];

    let result = worker.process_batch(&mut batch).await;

    // The caller and the job row keep the full provider message.
    assert!(
        matches!(&result, Err(EngramError::Embedding(m)) if m.contains(BODY)),
        "{result:?}"
    );
    let stored: String = worker
        .conn
        .lock()
        .query_row(
            "SELECT error FROM embedding_queue WHERE memory_id = 1",
            [],
            |r| r.get(0),
        )
        .expect("error column");
    assert!(stored.contains(BODY));
    // Logs carry the failure, classified, and nothing else.
    assert!(captured().contains("Embedding batch failed"));
    assert_logs_exclude(&[BODY, TEXT]);
}
