//! Storage-scoped drain of pending SQL queue rows.

use chrono::Utc;
use rusqlite::params;

use super::jobs::persist_computed_embedding;
use super::types::{EmbeddingDrainReport, EmbeddingQueueHygieneConfig};
use crate::embedding::Embedder;
use crate::error::{EngramError, Result};
use crate::types::MemoryId;

/// Callback invoked with `(memory_id, embedding)` after an embedding has been
/// durably persisted (for example to mirror it into an in-memory vector index).
pub type PersistedEmbeddingObserver<'a> = &'a (dyn Fn(MemoryId, &[f32]) + Send + Sync);

/// Drain up to `batch_size` pending entries from the SQL `embedding_queue`
/// table, compute their embeddings, and persist them.
///
/// Lock discipline: this function takes `&Storage` rather than `&Connection`
/// so it can scope each `with_connection` acquisition narrowly. The
/// `embedder.embed_batch()` call (which is a blocking HTTP request for cloud
/// backends like OpenAI) MUST run with the connection lock released and outside
/// any transaction, otherwise every other DB operation in the server stalls
/// behind every drain cycle.
///
/// Flow (see the private `super::jobs` module for the per-memory job lifecycle):
///   1. One transaction: SELECT pending rows + mark them 'processing'
///   2. No lock, no transaction: `embed_batch` runs the provider call
///   3. One transaction per memory: embedding row + `has_embedding` flag + job
///      'complete' commit together, or none of them do
///
/// On provider failure (or a short/invalid provider batch) the claimed jobs are
/// marked `failed` with the error and `retry_count` incremented. If persisting
/// one memory fails, its writes roll back and that job is marked `failed`; the
/// other memories of the batch are unaffected. A job whose memory content
/// changed while the provider call was in flight is left to the re-queue made
/// by that update (the stale embedding is not stored).
///
/// Jobs left in `processing` by a crash are NOT touched here; the explicit
/// recovery is [`run_embedding_drain_cycle`] / [`super::requeue_stale_processing_embeddings`].
///
/// Returns the number of claimed memories handled (persisted, or skipped because
/// they were superseded). Returns 0 when the queue is empty. Any failed memory
/// makes the call return `Err` after the successful ones were committed.
///
/// Fixes #10 sintoma A.
pub fn drain_pending_embeddings(
    storage: &crate::storage::Storage,
    embedder: &dyn Embedder,
    batch_size: usize,
) -> Result<usize> {
    drain_pending_embeddings_observed(storage, embedder, batch_size, &|_, _| {})
}

/// Same as [`drain_pending_embeddings`], reporting every persisted embedding to
/// `observer` (after its transaction committed).
pub fn drain_pending_embeddings_observed(
    storage: &crate::storage::Storage,
    embedder: &dyn Embedder,
    batch_size: usize,
    observer: PersistedEmbeddingObserver<'_>,
) -> Result<usize> {
    let claimed = claim_pending_embeddings(storage, batch_size)?;
    if claimed.is_empty() {
        return Ok(0);
    }

    // Provider call: no lock held, no transaction open.
    let contents: Vec<&str> = claimed.iter().map(|(_, c)| c.as_str()).collect();
    crate::observability::record_provider_call(crate::observability::ProviderCall::Embedding);
    let embed_result = embedder.embed_batch(&contents);
    let model = embedder.model_name().to_string();

    match embed_result {
        Ok(embeddings) if embeddings.len() == claimed.len() => {
            persist_claimed_batch(storage, &claimed, &embeddings, &model, observer)
        }
        Ok(embeddings) => {
            let message = format!(
                "embedding provider returned {} embeddings for {} texts",
                embeddings.len(),
                claimed.len()
            );
            let ids: Vec<MemoryId> = claimed.iter().map(|(id, _)| *id).collect();
            fail_claimed_jobs(storage, ids.iter().map(|id| (*id, message.as_str())))?;
            Err(EngramError::Embedding(message))
        }
        Err(e) => {
            crate::observability::record_provider_failure(
                crate::observability::ProviderCall::Embedding,
                &e,
            );
            let message = e.to_string();
            let ids: Vec<MemoryId> = claimed.iter().map(|(id, _)| *id).collect();
            fail_claimed_jobs(storage, ids.iter().map(|id| (*id, message.as_str())))?;
            Err(EngramError::Embedding(message))
        }
    }
}

/// One iteration of the server's drain thread: run the cycle and log its
/// outcome. Failures are logged as an operation event with the error *class*
/// only; the provider's message (which can echo memory text) stays in the
/// per-job `error` column and in the returned error.
pub fn run_embedding_drain_cycle_logged(
    storage: &crate::storage::Storage,
    embedder: &dyn Embedder,
    batch_size: usize,
    config: &EmbeddingQueueHygieneConfig,
    observer: PersistedEmbeddingObserver<'_>,
) {
    let started = std::time::Instant::now();
    match run_embedding_drain_cycle(storage, embedder, batch_size, config, observer) {
        Ok(report) if report.drained > 0 => {
            tracing::info!("Embedding drain processed {} memories", report.drained);
        }
        Ok(_) => {}
        Err(e) => {
            let id = crate::observability::CorrelationId::generate();
            crate::observability::OperationEvent::new(
                "embedding.drain",
                id.as_str(),
                "failed",
                started.elapsed(),
            )
            .failed(true)
            .error_class(crate::observability::redact::error_class(&e))
            .emit();
        }
    }
}

/// Atomically claim up to `limit` pending jobs (pending -> processing).
///
/// SELECT + mark-as-processing share one transaction so a second drainer can't
/// claim the same rows between the two statements.
pub(super) fn claim_pending_embeddings(
    storage: &crate::storage::Storage,
    limit: usize,
) -> Result<Vec<(MemoryId, String)>> {
    let limit = limit as i64;
    storage.with_transaction(|tx| {
        let mut stmt = tx.prepare(
            "SELECT eq.memory_id, m.content
             FROM embedding_queue eq
             JOIN memories m ON eq.memory_id = m.id
             WHERE eq.status = 'pending' AND m.valid_to IS NULL
             ORDER BY eq.queued_at
             LIMIT ?",
        )?;

        let rows: Vec<(MemoryId, String)> = stmt
            .query_map([limit], |row| {
                Ok((row.get::<_, MemoryId>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        drop(stmt);

        if !rows.is_empty() {
            let now = Utc::now().to_rfc3339();
            for &(id, _) in &rows {
                tx.execute(
                    "UPDATE embedding_queue SET status = 'processing', started_at = ?
                     WHERE memory_id = ?",
                    params![now, id],
                )?;
            }
        }

        Ok(rows)
    })
}

/// Persist each embedding in its own transaction; mark failures explicitly.
fn persist_claimed_batch(
    storage: &crate::storage::Storage,
    claimed: &[(MemoryId, String)],
    embeddings: &[Vec<f32>],
    model: &str,
    observer: PersistedEmbeddingObserver<'_>,
) -> Result<usize> {
    let mut handled = 0usize;
    let mut failures: Vec<(MemoryId, String)> = Vec::new();

    for ((id, content), embedding) in claimed.iter().zip(embeddings) {
        let outcome = storage
            .with_transaction(|tx| persist_computed_embedding(tx, *id, content, embedding, model));
        match outcome {
            Ok(persisted) => {
                handled += 1;
                if persisted {
                    observer(*id, embedding);
                }
            }
            Err(e) => failures.push((*id, e.to_string())),
        }
    }

    if failures.is_empty() {
        return Ok(handled);
    }

    let first = failures[0].1.clone();
    let total = failures.len();
    fail_claimed_jobs(storage, failures.iter().map(|(id, m)| (*id, m.as_str())))?;
    Err(EngramError::Embedding(format!(
        "{total} of {} embeddings failed to persist (first: {first})",
        claimed.len()
    )))
}

/// Mark claimed jobs `failed` (retry_count + 1) in one transaction. Only rows
/// still in `processing` are touched, so a job re-queued meanwhile is not clobbered.
fn fail_claimed_jobs<'a>(
    storage: &crate::storage::Storage,
    failures: impl Iterator<Item = (MemoryId, &'a str)>,
) -> Result<()> {
    let failures: Vec<(MemoryId, &str)> = failures.collect();
    storage.with_transaction(|conn| {
        for (id, message) in &failures {
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', error = ?,
                     retry_count = retry_count + 1
                 WHERE memory_id = ? AND status = 'processing'",
                params![message, id],
            )?;
        }
        Ok(())
    })
}

/// One full drain cycle: explicit stale-lease recovery, then drain until empty.
///
/// This is what the server's background worker runs on start and on every
/// interval (owner: the embedding drain thread in `engram-server`; operator
/// runbook: `docs/OPERATIONS.md`, "Embedding queue"). Recovery moves
/// `processing` jobs whose lease (`config.stale_processing_after`, default 15
/// minutes) expired back to `pending` (or to `failed` once `config.max_retries`
/// is exhausted), so a crash/reopen never strands work. Until the lease expires
/// the jobs are visible in health as `processing`, and as `stale_processing`
/// (status degraded) afterwards.
///
/// Retryable `failed` jobs are not requeued automatically: a failed job means
/// the provider or a local write rejected it, and retrying it is an explicit
/// operator action (`maintenance queue-hygiene --requeue-failed --apply`).
pub fn run_embedding_drain_cycle(
    storage: &crate::storage::Storage,
    embedder: &dyn Embedder,
    batch_size: usize,
    config: &EmbeddingQueueHygieneConfig,
    observer: PersistedEmbeddingObserver<'_>,
) -> Result<EmbeddingDrainReport> {
    let recovered = storage.with_transaction(|conn| {
        super::hygiene::requeue_stale_processing_embeddings(
            conn,
            config.stale_processing_after,
            config.max_retries,
        )
    })?;
    if recovered.requeued_stale > 0 || recovered.failed_exhausted > 0 {
        tracing::warn!(
            requeued_stale = recovered.requeued_stale,
            failed_exhausted = recovered.failed_exhausted,
            "Recovered embedding jobs left in processing past their lease"
        );
    }

    let mut drained = 0usize;
    // Catch up on a backlog without waiting a full interval between batches.
    loop {
        let handled = drain_pending_embeddings_observed(storage, embedder, batch_size, observer)?;
        drained += handled;
        if handled == 0 || handled < batch_size {
            break;
        }
    }

    Ok(EmbeddingDrainReport {
        requeued_stale: recovered.requeued_stale,
        failed_exhausted: recovered.failed_exhausted,
        drained,
    })
}
