//! RTK-inspired integration orchestrator for LLM context optimization
//!
//! Coordinates OutputFilter, ContextGrouper, and TruncationEngine to provide
//! a pipeline for reducing tokens sent to LLMs. Consolidation is intentionally
//! NOT part of this pipeline — that responsibility lives in `session_end.rs`
//! via the `pending_injections` queue (Sprint 3).

use crate::error::EngramError;
use crate::intelligence::context_grouper::{ContextGrouper, MemoryGroup};
use crate::intelligence::truncation_engine::{TruncationConfig, TruncationEngine};
use crate::types::Memory;
use serde::Serialize;

/// Result of context preparation pipeline
#[derive(Serialize)]
pub struct PreparedContext {
    pub context: String,
    pub token_count: usize,
    pub groups_count: usize,
}

/// Orchestrates all RTK-inspired components for optimal LLM context preparation
pub struct IntegrationOrchestrator {
    context_grouper: ContextGrouper,
    truncation_engine: TruncationEngine,
}

impl IntegrationOrchestrator {
    /// Create a new IntegrationOrchestrator with default components
    pub fn new() -> Self {
        Self {
            context_grouper: ContextGrouper::new(),
            truncation_engine: TruncationEngine::with_config(TruncationConfig::default()),
        }
    }

    /// Complete pipeline for preparing context for LLM consumption
    pub fn prepare_context_for_llm(
        &self,
        query: &str,
        memories: &[Memory],
        budget: usize,
    ) -> Result<PreparedContext, EngramError> {
        let relevant = self.filter_irrelevant(memories, query);
        let groups: Vec<MemoryGroup> = self.context_grouper.group_for_context(&relevant);
        let truncated_groups = self.truncation_engine.truncate_groups(&groups, budget);

        let mut context = String::new();
        for group in &truncated_groups {
            context.push_str(&format!("## {}\n{}\n", group.topic, group.summary));
        }

        let token_count = self.truncation_engine.estimate_tokens(&context);

        Ok(PreparedContext {
            context,
            token_count,
            groups_count: groups.len(),
        })
    }

    fn filter_irrelevant(&self, memories: &[Memory], query: &str) -> Vec<Memory> {
        memories
            .iter()
            .filter(|m| self.is_relevant(m, query))
            .cloned()
            .collect()
    }

    fn is_relevant(&self, memory: &Memory, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        query_terms.iter().any(|term| memory.content.contains(term))
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
    use std::collections::HashMap;

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
            stability: 1.0,
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
