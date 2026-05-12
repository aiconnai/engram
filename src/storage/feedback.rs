//! Feedback processing loop for autonomous memory learning.
//!
//! Connects `memory_feedback` signals to `UtilityTracker` adjustments
//! and triggers auto-consolidation when utility drops too low.

use std::sync::Arc;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::search::utility::{UtilityConfig, UtilityTracker};

// ---------------------------------------------------------------------------
// AutoConsolidator trait
// ---------------------------------------------------------------------------

/// Trait for auto-consolidation engines.
pub trait AutoConsolidatorTrait: Send + Sync {
    /// Schedule a list of memory IDs for consolidation.
    fn schedule_consolidation(&self, memory_ids: &[i64]) -> Result<()>;
}

// ---------------------------------------------------------------------------
// FeedbackProcessor
// ---------------------------------------------------------------------------

/// Processes feedback signals and adjusts memory utility scores,
/// optionally triggering auto-consolidation when scores drop too low.
pub struct FeedbackProcessor {
    tracker: UtilityTracker,
    /// Score below which a memory is forwarded to the auto-consolidator.
    consolidation_threshold: f64,
    auto_consolidator: Option<Arc<dyn AutoConsolidatorTrait>>,
}

impl FeedbackProcessor {
    /// Create a new FeedbackProcessor with default configuration.
    pub fn new() -> Self {
        Self {
            tracker: UtilityTracker::new(),
            consolidation_threshold: 0.2,
            auto_consolidator: None,
        }
    }

    /// Create a FeedbackProcessor with custom utility-tracker config.
    pub fn with_config(config: UtilityConfig) -> Self {
        Self {
            tracker: UtilityTracker::with_config(config),
            consolidation_threshold: 0.2,
            auto_consolidator: None,
        }
    }

    /// Override the score threshold below which memories are scheduled for consolidation.
    pub fn with_consolidation_threshold(mut self, threshold: f64) -> Self {
        self.consolidation_threshold = threshold;
        self
    }

    /// Attach an auto-consolidator that is invoked when a score drops below the threshold.
    pub fn with_consolidator<T: AutoConsolidatorTrait + 'static>(
        mut self,
        consolidator: Arc<T>,
    ) -> Self {
        self.auto_consolidator = Some(consolidator as Arc<dyn AutoConsolidatorTrait>);
        self
    }

    /// Process a feedback signal for a memory and adjust its utility score.
    ///
    /// Returns `(new_utility_score, scheduled_for_consolidation)`.
    pub fn process_feedback(
        &self,
        memory_id: i64,
        signal: &str,
        conn: &Connection,
    ) -> Result<(f64, bool)> {
        // Record with signal-tagged query so explain_utility can show which
        // negative signal drove a decay (helpful/not_helpful/outdated/conflict).
        let (was_useful, signal_tag) = match signal {
            "helpful" => (true, "feedback:helpful"),
            "not_helpful" => (false, "feedback:not_helpful"),
            "outdated" => (false, "feedback:outdated"),
            "conflict" => (false, "feedback:conflict"),
            other => (false, other),
        };
        self.tracker
            .record_retrieval(conn, memory_id, was_useful, signal_tag)?;

        let new_score = self.tracker.get_utility(conn, memory_id)?.score;

        let mut scheduled = false;
        if new_score < self.consolidation_threshold {
            if let Some(ref ac) = self.auto_consolidator {
                ac.schedule_consolidation(&[memory_id])?;
                scheduled = true;
            }
        }

        Ok((new_score, scheduled))
    }
}

impl Default for FeedbackProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Feedback statistics
// ---------------------------------------------------------------------------

/// Statistics about feedback and consolidation.
#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackLoopStats {
    pub total_feedback: i64,
    pub consolidation_scheduled: i64,
    pub avg_score_before: f64,
    pub avg_score_after: f64,
}

pub fn get_feedback_loop_stats(_conn: &Connection) -> Result<FeedbackLoopStats> {
    Ok(FeedbackLoopStats {
        total_feedback: 0,
        consolidation_scheduled: 0,
        avg_score_before: 0.5,
        avg_score_after: 0.5,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::utility::CREATE_UTILITY_FEEDBACK_TABLE;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(CREATE_UTILITY_FEEDBACK_TABLE, []).unwrap();
        conn
    }

    #[test]
    fn test_process_helpful_feedback() {
        let conn = setup();
        let processor = FeedbackProcessor::new();
        let (score, scheduled) = processor.process_feedback(1, "helpful", &conn).unwrap();
        assert!(score >= 0.0);
        assert!(!scheduled);
    }

    #[test]
    fn test_process_not_helpful_feedback() {
        let conn = setup();
        let processor = FeedbackProcessor::new();
        processor.process_feedback(2, "helpful", &conn).unwrap();
        let (score, _) = processor.process_feedback(2, "not_helpful", &conn).unwrap();
        assert!(score >= 0.0);
    }

    #[test]
    fn test_custom_config() {
        let conn = setup();
        let config = UtilityConfig {
            learning_rate: 0.3,
            decay_factor: 0.9,
            initial_score: 0.6,
        };
        let processor = FeedbackProcessor::with_config(config);
        let (score, _) = processor.process_feedback(3, "helpful", &conn).unwrap();
        assert!(score > 0.5);
    }

    #[test]
    fn test_consolidation_threshold() {
        let conn = setup();
        // Very high threshold so that even a fresh score triggers it
        let processor = FeedbackProcessor::new().with_consolidation_threshold(1.0);
        let (_, scheduled) = processor.process_feedback(4, "not_helpful", &conn).unwrap();
        // No consolidator attached, so scheduled must be false even above threshold
        assert!(!scheduled);
    }
}
