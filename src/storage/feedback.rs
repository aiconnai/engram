//! Feedback processing loop for autonomous memory learning.
//!
//! Connects `memory_feedback` signals to `UtilityTracker` adjustments
//! and triggers auto-consolidation when utility drops too low.

use std::sync::Arc;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::search::utility::UtilityTracker;

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
    /// Optional auto-consolidator for low-utility memories.
    /// Wrapped in Arc to allow shared ownership.
    auto_consolidator: Option<Arc<dyn AutoConsolidatorTrait>>,
}

impl FeedbackProcessor {
    /// Create a new FeedbackProcessor without an auto-consolidator.
    pub fn new() -> Self {
        Self {
            auto_consolidator: None,
        }
    }

    /// Create a new FeedbackProcessor with an auto-consolidator.
    pub fn with_consolidator<T: AutoConsolidatorTrait + 'static>(consolidator: Arc<T>) -> Self {
        Self {
            auto_consolidator: Some(consolidator as Arc<dyn AutoConsolidatorTrait>),
        }
    }

    /// Process a feedback signal for a memory and adjust its utility score.
    ///
    /// # Arguments
    /// * `memory_id` - The ID of the memory receiving feedback.
    /// * `signal` - The feedback signal: "helpful", "not_helpful", "outdated", "conflict".
    /// * `conn` - Database connection.
    ///
    /// # Returns
    /// A tuple of (new_utility_score, scheduled_for_consolidation).
    pub fn process_feedback(
        &self,
        memory_id: i64,
        signal: &str,
        conn: &Connection,
    ) -> Result<(f64, bool)> {
        // 1. Determine the delta based on the signal
        let delta = match signal {
            "helpful" => 0.1,
            "not_helpful" => -0.1,
            "outdated" => -0.2,
            "conflict" => -0.3,
            _ => 0.0,
        };

        // 2. Adjust utility score using UtilityTracker
        let tracker = UtilityTracker::new();

        // Get current score first
        let _current_score = match tracker.get_utility(conn, memory_id) {
            Ok(score) => score.score,
            Err(_) => tracker.config.initial_score,
        };

        // Apply the delta (simplified adjustment; in practice, UtilityTracker
        // has its own update rules via record_retrieval)
        // We'll use record_retrieval to stay consistent with existing logic
        let was_useful = delta > 0.0;
        tracker.record_retrieval(conn, memory_id, was_useful, "feedback")?;

        // Get updated score
        let new_score = tracker.get_utility(conn, memory_id)?.score;

        // 3. Check if score is too low and schedule for consolidation
        let mut scheduled = false;
        if new_score < 0.2 {
            if let Some(ref ac) = self.auto_consolidator {
                let ids = vec![memory_id];
                ac.schedule_consolidation(&ids)?;
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
    /// Total number of feedback events processed.
    pub total_feedback: i64,
    /// Number of memories scheduled for consolidation due to low utility.
    pub consolidation_scheduled: i64,
    /// Average utility score before feedback processing.
    pub avg_score_before: f64,
    /// Average utility score after feedback processing.
    pub avg_score_after: f64,
}

/// Get feedback loop statistics.
///
/// This is a placeholder that will be expanded with actual DB queries.
pub fn get_feedback_loop_stats(_conn: &Connection) -> Result<FeedbackLoopStats> {
    // TODO: Implement actual statistics from DB
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
        assert!(!scheduled); // Score shouldn't be below 0.2 for helpful
    }

    #[test]
    fn test_process_not_helpful_feedback() {
        let conn = setup();
        let processor = FeedbackProcessor::new();

        // First make it have some score
        processor.process_feedback(2, "helpful", &conn).unwrap();

        // Then give negative feedback
        let (score, scheduled) = processor.process_feedback(2, "not_helpful", &conn).unwrap();
        assert!(score >= 0.0);
        // Might not be scheduled unless score drops below 0.2
    }
}
