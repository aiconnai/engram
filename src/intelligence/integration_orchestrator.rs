//! RTK-inspired integration orchestrator for LLM context optimization
//!
//! Coordinates OutputFilter, ContextGrouper, TruncationEngine, and AutoConsolidator
//! to provide a complete pipeline for reducing tokens sent to LLMs.

use crate::error::EngramError;
use crate::intelligence::context_grouper::{ContextGrouper, MemoryGroup};
use crate::intelligence::output_filter::OutputFilter;
use crate::intelligence::truncation_engine::{TruncationConfig, TruncationEngine};
use crate::types::Memory;
use serde::Serialize;
use std::collections::HashMap;

/// Minimal auto-consolidator for scheduling memory consolidation
pub struct AutoConsolidator;

impl AutoConsolidator {
    pub fn new() -> Self {
        Self
    }

    /// Schedule consolidation of similar memories
    pub fn schedule_consolidation(&self, _memories: &[Memory]) -> Result<(), EngramError> {
        // Minimal implementation - will be expanded in future iterations
        Ok(())
    }
}

/// Result of context preparation pipeline
#[derive(Serialize)]
pub struct PreparedContext {
    pub context: String,
    pub token_count: usize,
    pub groups_count: usize,
    pub consolidation_scheduled: bool,
}

/// Orchestrates all RTK-inspired components for optimal LLM context preparation
pub struct IntegrationOrchestrator {
    output_filter: OutputFilter,
    context_grouper: ContextGrouper,
    truncation_engine: TruncationEngine,
    auto_consolidator: AutoConsolidator,
}

impl IntegrationOrchestrator {
    /// Create a new IntegrationOrchestrator with default components
    pub fn new() -> Self {
        Self {
            output_filter: OutputFilter::new(),
            context_grouper: ContextGrouper::new(),
            truncation_engine: TruncationEngine::with_config(TruncationConfig::default()),
            auto_consolidator: AutoConsolidator::new(),
        }
    }

    /// Complete pipeline for preparing context for LLM consumption
    pub fn prepare_context_for_llm(
        &self,
        query: &str,
        memories: &[Memory],
        budget: usize,
    ) -> Result<PreparedContext, EngramError> {
        // 1. Filter irrelevant memories
        let relevant = self.filter_irrelevant(memories, query);

        // 2. Group by topic using ContextGrouper
        let groups: Vec<MemoryGroup> = self.context_grouper.group_for_context(&relevant);

        // 3. Truncate groups to fit token budget
        let truncated_groups = self.truncation_engine.truncate_groups(&groups, budget);

        // 4. Build final context string
        let mut context = String::new();
        for group in &truncated_groups {
            context.push_str(&format!("## {}\n{}\n", group.topic, group.summary));
        }

        // 5. Check if consolidation is needed
        let consolidation_scheduled = if self.should_consolidate(&relevant) {
            self.auto_consolidator.schedule_consolidation(&relevant)?;
            true
        } else {
            false
        };

        let token_count = self.truncation_engine.estimate_tokens(&context);

        Ok(PreparedContext {
            context,
            token_count,
            groups_count: groups.len(),
            consolidation_scheduled,
        })
    }

    /// Filter out memories irrelevant to the query
    fn filter_irrelevant(&self, memories: &[Memory], query: &str) -> Vec<Memory> {
        memories
            .iter()
            .filter(|m| self.is_relevant(m, query))
            .cloned()
            .collect()
    }

    /// Simple relevance heuristic: check if query terms appear in memory content
    fn is_relevant(&self, memory: &Memory, query: &str) -> bool {
        if query.is_empty() {
            return true; // Empty query matches all memories
        }
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        query_terms.iter().any(|term| memory.content.contains(term))
    }

    /// Determine if memory consolidation should be scheduled
    fn should_consolidate(&self, memories: &[Memory]) -> bool {
        // Schedule consolidation if we have many similar memories
        memories.len() > 10
    }
}

impl Default for IntegrationOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Memory, MemoryType};
    use chrono::Utc;

    fn create_test_memory(id: i64, content: &str, memory_type: MemoryType) -> Memory {
        Memory {
            id,
            content: content.to_string(),
            memory_type,
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
            importance: 0.5,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: crate::types::Visibility::default(),
            scope: crate::types::MemoryScope::default(),
            workspace: "default".to_string(),
            tier: crate::types::MemoryTier::default(),
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
            lifecycle_state: crate::types::LifecycleState::default(),
            media_url: None,
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = IntegrationOrchestrator::new();
        let _ = orchestrator;
    }

    #[test]
    fn test_prepare_context_for_llm() {
        let orchestrator = IntegrationOrchestrator::new();
        let memories = vec![
            create_test_memory(1, "User prefers dark mode", MemoryType::Preference),
            create_test_memory(2, "User likes coffee", MemoryType::Preference),
        ];

        let result = orchestrator.prepare_context_for_llm("prefers dark mode", &memories, 4000);
        assert!(result.is_ok());

        let prepared = result.unwrap();
        assert!(!prepared.context.is_empty());
        assert!(prepared.token_count > 0);
    }
}
