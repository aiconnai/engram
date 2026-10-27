//! Neural cross-encoder reranking module
//!
//! Provides neural reranking using ONNX-based cross-encoder models
//! (e.g., ms-marco-MiniLM-L-6-v2) for high-quality query-document
//! relevance scoring. This module complements the heuristic-based
//! reranker in `rerank.rs` with learned scoring.
//!
//! Feature-gated behind `neural-rerank` to keep the default binary lean.
//!
//! # Status: scoring is not implemented
//!
//! The ONNX forward pass (tokenisation, tensor stacking, `session.run`) is not
//! wired yet; see [`NEURAL_SCORING_IMPLEMENTED`]. Until it is,
//! [`CrossEncoderReranker`] returns an error from `rerank` instead of
//! producing scores, so placeholder output can never reorder results or
//! replace real retrieval scores. Search keeps using the rule-based reranker
//! in `rerank.rs`.

#![cfg(feature = "neural-rerank")]

use std::path::PathBuf;

use crate::error::{EngramError, Result};
use crate::types::Memory;

/// Whether [`CrossEncoderReranker`] actually runs the cross-encoder model.
///
/// `false` until the ONNX inference path exists. Model-routing status
/// (`routing::model`) reads this so a compiled-in but non-functional reranker
/// is never reported as operational. Flip it only together with a real
/// `score_batch` implementation and tests that exercise it.
pub const NEURAL_SCORING_IMPLEMENTED: bool = false;

/// Error text returned while [`NEURAL_SCORING_IMPLEMENTED`] is `false`.
const SCORING_NOT_IMPLEMENTED: &str = "cross-encoder scoring is not implemented: \
     the ONNX forward pass is not wired, so no relevance scores are produced";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A candidate memory for reranking, carrying its original retrieval score.
#[derive(Debug, Clone)]
pub struct RerankCandidate {
    /// The memory to be reranked.
    pub memory: Memory,
    /// Score produced by the upstream retrieval stage (e.g., BM25 + vector).
    pub original_score: f32,
    /// Score assigned by the cross-encoder (None until reranking is applied).
    pub rerank_score: Option<f32>,
}

impl RerankCandidate {
    /// Create a new candidate with no rerank score yet.
    pub fn new(memory: Memory, original_score: f32) -> Self {
        Self {
            memory,
            original_score,
            rerank_score: None,
        }
    }

    /// Return the effective score: rerank score if available, else original.
    pub fn effective_score(&self) -> f32 {
        self.rerank_score.unwrap_or(self.original_score)
    }
}

// ---------------------------------------------------------------------------
// Reranker trait
// ---------------------------------------------------------------------------

/// Trait implemented by all reranking backends.
///
/// Implementors receive a list of `RerankCandidate`s, assign `rerank_score`
/// to each one (consuming the vector), and return them sorted by descending
/// `rerank_score`.
pub trait Reranker: Send + Sync {
    /// Rerank `candidates` with respect to `query`.
    ///
    /// Returns candidates sorted by descending `rerank_score`.  The
    /// `rerank_score` field on each returned candidate is guaranteed to be
    /// `Some(…)`.
    fn rerank(&self, query: &str, candidates: Vec<RerankCandidate>)
        -> Result<Vec<RerankCandidate>>;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the ONNX cross-encoder reranker.
#[derive(Debug, Clone)]
pub struct CrossEncoderConfig {
    /// Path to the ONNX model file (e.g., `model.onnx`).
    pub model_path: PathBuf,
    /// Maximum total token length for `[CLS] query [SEP] document [SEP]`.
    /// Defaults to 512.
    pub max_length: usize,
    /// Number of query-document pairs processed in a single ONNX forward pass.
    /// Defaults to 32.
    pub batch_size: usize,
    /// Candidates with a normalised score below this value are removed from
    /// the output.  Range `[0.0, 1.0]`.  Defaults to `0.0` (keep all).
    pub threshold: f32,
}

impl Default for CrossEncoderConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("model.onnx"),
            max_length: 512,
            batch_size: 32,
            threshold: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-encoder reranker (ONNX runtime)
// ---------------------------------------------------------------------------

/// ONNX-based cross-encoder reranker (scoring not implemented yet).
///
/// Designed to use an ms-marco-MiniLM-L-6-v2 compatible model to score each
/// `(query, document)` pair, min-max normalise the raw logits to `[0, 1]`
/// across the batch, then filter by `config.threshold`.
///
/// **The model is loaded but never run:** [`NEURAL_SCORING_IMPLEMENTED`] is
/// `false`, and [`Reranker::rerank`] returns an error for any non-empty
/// candidate list rather than fabricating scores.
pub struct CrossEncoderReranker {
    config: CrossEncoderConfig,
    /// ONNX inference session.  `None` when running under `cfg(test)` or
    /// when the model file was not found.
    session: Option<ort::session::Session>,
}

impl CrossEncoderReranker {
    /// Load the ONNX cross-encoder from `config.model_path`.
    ///
    /// Returns an error if the model file cannot be opened or the session
    /// cannot be initialised.
    pub fn new(config: CrossEncoderConfig) -> Result<Self> {
        let session = Self::load_session(&config)?;
        Ok(Self {
            config,
            session: Some(session),
        })
    }

    /// Try to load the ONNX session; propagate errors with context.
    fn load_session(config: &CrossEncoderConfig) -> Result<ort::session::Session> {
        ort::session::Session::builder()
            .map_err(|e| {
                EngramError::Config(format!("Failed to create ONNX session builder: {e}"))
            })?
            .commit_from_file(&config.model_path)
            .map_err(|e| {
                EngramError::Config(format!(
                    "Failed to load ONNX model from {:?}: {e}",
                    config.model_path
                ))
            })
    }

    /// Build the cross-encoder input string for a single pair.
    ///
    /// Format: `[CLS] {query} [SEP] {content} [SEP]`
    fn build_input(query: &str, content: &str, max_length: usize) -> String {
        let raw = format!("[CLS] {query} [SEP] {content} [SEP]");
        // Truncate at character boundary (not perfect but avoids panics)
        if raw.chars().count() <= max_length {
            raw
        } else {
            raw.chars().take(max_length).collect()
        }
    }

    /// Score a single batch of inputs via the ONNX session.
    ///
    /// Returns a vector of raw logit scores (one per input).
    ///
    /// Always fails while [`NEURAL_SCORING_IMPLEMENTED`] is `false`: returning
    /// constant placeholder scores would be normalised to `1.0` by
    /// [`Self::normalize`] and presented as real relevance.
    fn score_batch(&self, inputs: &[String]) -> Result<Vec<f32>> {
        if !NEURAL_SCORING_IMPLEMENTED {
            return Err(EngramError::Search(SCORING_NOT_IMPLEMENTED.to_string()));
        }

        let _session = self
            .session
            .as_ref()
            .ok_or_else(|| EngramError::Config("ONNX session not initialised".to_string()))?;

        // Real implementation will:
        //   1. Tokenise each `input` string with a BERT tokeniser
        //   2. Pad/truncate to `config.max_length`
        //   3. Stack into a 2-D tensor (batch × seq_len)
        //   4. Run `session.run([input_ids, attention_mask])` → logits
        //   5. Extract the relevance logit from the model output
        Err(EngramError::Search(format!(
            "{SCORING_NOT_IMPLEMENTED} ({} inputs)",
            inputs.len()
        )))
    }

    /// Min-max normalise a slice of scores to `[0, 1]`.
    fn normalize(scores: &mut [f32]) {
        if scores.is_empty() {
            return;
        }
        let min = scores.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        if (max - min).abs() < f32::EPSILON {
            // All scores identical — set to 1.0 (all equally relevant)
            scores.iter_mut().for_each(|s| *s = 1.0);
        } else {
            scores
                .iter_mut()
                .for_each(|s| *s = (*s - min) / (max - min));
        }
    }
}

impl Reranker for CrossEncoderReranker {
    /// Run the three-stage reranking pipeline:
    ///
    /// 1. **Score** — run ONNX cross-encoder on all `(query, doc)` pairs in batches
    /// 2. **Normalize** — min-max normalise raw logits to `[0, 1]`
    /// 3. **Filter** — discard candidates below `config.threshold`
    ///
    /// Returns candidates sorted by descending `rerank_score`.
    ///
    /// # Errors
    ///
    /// Until [`NEURAL_SCORING_IMPLEMENTED`] is `true`, any non-empty
    /// `candidates` list yields an error (stage 1 cannot produce scores).
    fn rerank(
        &self,
        query: &str,
        candidates: Vec<RerankCandidate>,
    ) -> Result<Vec<RerankCandidate>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // ── Stage 1: Score ──────────────────────────────────────────────────
        let inputs: Vec<String> = candidates
            .iter()
            .map(|c| Self::build_input(query, &c.memory.content, self.config.max_length))
            .collect();

        // Process in batches to respect memory constraints
        let mut raw_scores: Vec<f32> = Vec::with_capacity(inputs.len());
        for chunk in inputs.chunks(self.config.batch_size) {
            let batch_scores = self.score_batch(chunk)?;
            raw_scores.extend(batch_scores);
        }

        // ── Stage 2: Normalize ──────────────────────────────────────────────
        Self::normalize(&mut raw_scores);

        // ── Stage 3: Filter & sort ──────────────────────────────────────────
        let threshold = self.config.threshold;
        let mut scored: Vec<RerankCandidate> = candidates
            .into_iter()
            .zip(raw_scores)
            .filter_map(|(mut candidate, score)| {
                if score < threshold {
                    None
                } else {
                    candidate.rerank_score = Some(score);
                    Some(candidate)
                }
            })
            .collect();

        // Sort descending by rerank score
        scored.sort_by(|a, b| {
            b.rerank_score
                .unwrap_or(0.0)
                .partial_cmp(&a.rerank_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scored)
    }
}

// ---------------------------------------------------------------------------
// RerankerPipeline — convenience wrapper
// ---------------------------------------------------------------------------

/// A composable pipeline that applies a `Reranker` and exposes the three
/// stages (score, normalize, filter) as a single `run` call.
///
/// This is the recommended entry-point when integrating neural reranking
/// into the search pipeline.
pub struct RerankerPipeline {
    inner: Box<dyn Reranker>,
}

impl RerankerPipeline {
    /// Create a pipeline backed by any `Reranker` implementation.
    pub fn new(reranker: impl Reranker + 'static) -> Self {
        Self {
            inner: Box::new(reranker),
        }
    }

    /// Run the full pipeline: score → normalize → filter → sort.
    pub fn run(
        &self,
        query: &str,
        candidates: Vec<RerankCandidate>,
    ) -> Result<Vec<RerankCandidate>> {
        self.inner.rerank(query, candidates)
    }
}

// ---------------------------------------------------------------------------
// Tests (using MockReranker — no ONNX dependency required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LifecycleState, MemoryScope, MemoryTier, MemoryType, Visibility};
    use chrono::Utc;
    use std::collections::HashMap;

    // ── MockReranker ─────────────────────────────────────────────────────────

    /// A test-only reranker that assigns scores based on keyword overlap
    /// between the query and each candidate's content.
    struct MockReranker {
        threshold: f32,
    }

    impl MockReranker {
        fn new(threshold: f32) -> Self {
            Self { threshold }
        }

        /// Count how many query words appear in `text` (case-insensitive).
        fn keyword_overlap(query: &str, text: &str) -> usize {
            let text_lower = text.to_lowercase();
            query
                .split_whitespace()
                .filter(|w| text_lower.contains(&w.to_lowercase()))
                .count()
        }
    }

    impl Reranker for MockReranker {
        fn rerank(
            &self,
            query: &str,
            candidates: Vec<RerankCandidate>,
        ) -> Result<Vec<RerankCandidate>> {
            if candidates.is_empty() {
                return Ok(Vec::new());
            }

            // Stage 1: compute raw scores
            let mut raw: Vec<f32> = candidates
                .iter()
                .map(|c| Self::keyword_overlap(query, &c.memory.content) as f32)
                .collect();

            // Stage 2: normalize
            CrossEncoderReranker::normalize(&mut raw);

            // Stage 3: filter + sort
            let threshold = self.threshold;
            let mut scored: Vec<RerankCandidate> = candidates
                .into_iter()
                .zip(raw)
                .filter_map(|(mut c, s)| {
                    if s < threshold {
                        None
                    } else {
                        c.rerank_score = Some(s);
                        Some(c)
                    }
                })
                .collect();

            scored.sort_by(|a, b| {
                b.rerank_score
                    .unwrap_or(0.0)
                    .partial_cmp(&a.rerank_score.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            Ok(scored)
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_memory(id: i64, content: &str) -> Memory {
        Memory {
            id,
            content: content.to_string(),
            memory_type: MemoryType::Note,
            tags: Vec::new(),
            metadata: HashMap::new(),
            importance: 0.5,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Visibility::Private,
            scope: MemoryScope::Global,
            workspace: "default".to_string(),
            tier: MemoryTier::Permanent,
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: LifecycleState::Active,
            stability: 1.0,
            media_url: None,
        }
    }

    fn make_candidate(id: i64, content: &str, original_score: f32) -> RerankCandidate {
        RerankCandidate::new(make_memory(id, content), original_score)
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_rerank_candidate_fields() {
        let memory = make_memory(42, "some content");
        let candidate = RerankCandidate::new(memory.clone(), 0.75);

        assert_eq!(candidate.memory.id, 42);
        assert_eq!(candidate.memory.content, "some content");
        assert!((candidate.original_score - 0.75).abs() < f32::EPSILON);
        assert!(candidate.rerank_score.is_none());
    }

    #[test]
    fn test_effective_score_falls_back_to_original() {
        let c = make_candidate(1, "hello", 0.6);
        assert!((c.effective_score() - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_score_prefers_rerank_score() {
        let mut c = make_candidate(1, "hello", 0.6);
        c.rerank_score = Some(0.9);
        assert!((c.effective_score() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_basic_reranking_changes_order() {
        // Original order: id=1 (score 0.9), id=2 (score 0.8), id=3 (score 0.7)
        // Query "rust memory" → id=3 has most overlap, so it should rank first.
        let candidates = vec![
            make_candidate(1, "python web framework", 0.9),
            make_candidate(2, "go concurrency patterns", 0.8),
            make_candidate(3, "rust memory management tips", 0.7),
        ];

        let reranker = MockReranker::new(0.0);
        let result = reranker.rerank("rust memory", candidates).unwrap();

        assert_eq!(result.len(), 3);
        // Candidate 3 should be first because it overlaps with both query words.
        assert_eq!(result[0].memory.id, 3);
        // All rerank scores must be Some
        for c in &result {
            assert!(c.rerank_score.is_some());
        }
    }

    #[test]
    fn test_empty_candidates_returns_empty() {
        let reranker = MockReranker::new(0.0);
        let result = reranker.rerank("query", Vec::new()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_threshold_filtering_removes_low_scores() {
        // With threshold > 0, zero-overlap candidates should be removed.
        let candidates = vec![
            make_candidate(1, "rust memory management", 0.9), // overlaps
            make_candidate(2, "completely unrelated topic xyz", 0.8), // no overlap
            make_candidate(3, "memory allocator in rust", 0.7), // overlaps
        ];

        // Use a threshold that eliminates the zero-overlap candidate.
        // After normalisation the zero-overlap candidate scores 0.0.
        let reranker = MockReranker::new(0.01);
        let result = reranker.rerank("rust memory", candidates).unwrap();

        // Only the two overlapping candidates should survive.
        assert_eq!(result.len(), 2);
        for c in &result {
            assert!(c.rerank_score.unwrap() >= 0.01);
        }
    }

    #[test]
    fn test_score_normalization_to_0_1_range() {
        let candidates = vec![
            make_candidate(1, "rust memory management best practices", 0.5), // 3 overlaps
            make_candidate(2, "rust systems programming", 0.5),              // 1 overlap
            make_candidate(3, "python scripting guide", 0.5),                // 0 overlaps
        ];

        let reranker = MockReranker::new(0.0);
        let result = reranker.rerank("rust memory", candidates).unwrap();

        for c in &result {
            let score = c.rerank_score.unwrap();
            assert!(
                (0.0..=1.0).contains(&score),
                "Score {score} is outside [0, 1]"
            );
        }

        // The best candidate should score 1.0 after normalisation.
        let best = result.first().unwrap().rerank_score.unwrap();
        assert!(
            (best - 1.0).abs() < f32::EPSILON,
            "Best score should be 1.0 after normalisation"
        );
    }

    #[test]
    fn test_batch_processing_handles_many_candidates() {
        // Create more candidates than the default batch_size (32) to ensure
        // batch chunking works correctly end-to-end.
        let candidates: Vec<RerankCandidate> = (0..50)
            .map(|i| {
                let content = if i % 3 == 0 {
                    format!("document about rust memory topic {i}")
                } else {
                    format!("unrelated document number {i}")
                };
                make_candidate(i, &content, 1.0 - i as f32 * 0.01)
            })
            .collect();

        let reranker = MockReranker::new(0.0);
        let result = reranker.rerank("rust memory", candidates).unwrap();

        assert_eq!(result.len(), 50);
        // Verify descending sort order
        let mut prev_score = f32::INFINITY;
        for c in &result {
            let score = c.rerank_score.unwrap();
            assert!(score <= prev_score, "Results not sorted descending");
            prev_score = score;
        }
    }

    #[test]
    fn test_single_candidate_normalizes_to_1() {
        let candidates = vec![make_candidate(1, "some content", 0.5)];
        let reranker = MockReranker::new(0.0);
        let result = reranker.rerank("query", candidates).unwrap();

        assert_eq!(result.len(), 1);
        // Single candidate: all scores are identical → normalised to 1.0
        let score = result[0].rerank_score.unwrap();
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_normalize_helper_all_equal_scores() {
        let mut scores = vec![0.5_f32, 0.5, 0.5];
        CrossEncoderReranker::normalize(&mut scores);
        for s in &scores {
            assert!((*s - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_normalize_helper_empty_slice() {
        let mut scores: Vec<f32> = Vec::new();
        CrossEncoderReranker::normalize(&mut scores); // should not panic
    }

    #[test]
    fn test_normalize_helper_distinct_scores() {
        let mut scores = vec![0.0_f32, 5.0, 10.0];
        CrossEncoderReranker::normalize(&mut scores);
        assert!((scores[0] - 0.0).abs() < f32::EPSILON);
        assert!((scores[1] - 0.5).abs() < f32::EPSILON);
        assert!((scores[2] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_build_input_format() {
        let input = CrossEncoderReranker::build_input("my query", "document text", 512);
        assert!(input.starts_with("[CLS]"));
        assert!(input.contains("my query"));
        assert!(input.contains("[SEP]"));
        assert!(input.contains("document text"));
    }

    #[test]
    fn test_build_input_truncation() {
        // max_length of 10 chars — result must not exceed that count
        let input = CrossEncoderReranker::build_input("query", "very long document content", 10);
        assert!(input.chars().count() <= 10);
    }

    #[test]
    fn test_pipeline_delegates_to_reranker() {
        let candidates = vec![
            make_candidate(1, "python web development", 0.9),
            make_candidate(2, "rust systems programming", 0.8),
        ];

        let pipeline = RerankerPipeline::new(MockReranker::new(0.0));
        let result = pipeline.run("rust", candidates).unwrap();

        assert_eq!(result.len(), 2);
        // Candidate 2 mentions "rust" so it should rank first
        assert_eq!(result[0].memory.id, 2);
    }

    /// The ONNX forward pass is not wired yet. The reranker must refuse to
    /// score instead of fabricating constant scores that `normalize` would turn
    /// into `1.0` ("everything is perfectly relevant") and that
    /// `effective_score` would then use in place of the original retrieval score.
    #[test]
    fn test_cross_encoder_refuses_to_fabricate_scores() {
        let reranker = CrossEncoderReranker {
            config: CrossEncoderConfig::default(),
            session: None,
        };
        let candidates = vec![
            make_candidate(1, "alpha", 0.9),
            make_candidate(2, "beta", 0.4),
        ];

        let err = match reranker.rerank("alpha", candidates) {
            Ok(_) => panic!("placeholder scorer must not return reranked candidates"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("not implemented"),
            "error must explain the scorer is not implemented, got: {err}"
        );
    }

    #[test]
    fn test_cross_encoder_rerank_of_nothing_is_still_ok() {
        let reranker = CrossEncoderReranker {
            config: CrossEncoderConfig::default(),
            session: None,
        };
        assert!(reranker.rerank("anything", Vec::new()).unwrap().is_empty());
    }

    #[test]
    fn test_cross_encoder_config_defaults() {
        let config = CrossEncoderConfig::default();
        assert_eq!(config.max_length, 512);
        assert_eq!(config.batch_size, 32);
        assert!((config.threshold - 0.0).abs() < f32::EPSILON);
    }
}
