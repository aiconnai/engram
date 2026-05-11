//! Integration Orchestrator for RTK-inspired output filtering
//! Coordinates multiple intelligence components for optimal context injection

use crate::intelligence::context_grouper::ContextGrouper;
use crate::intelligence::truncation_engine::TruncationEngine;
use crate::types::Memory;

/// Orchestrates the integration of multiple intelligence components
pub struct IntegrationOrchestrator {
    context_grouper: ContextGrouper,
    truncation_engine: TruncationEngine,
}

impl IntegrationOrchestrator {
    /// Create a new IntegrationOrchestrator with default components
    pub fn new() -> Self {
        Self {
            context_grouper: ContextGrouper::new(),
            truncation_engine: TruncationEngine::with_config(Default::default()),
        }
    }

    /// Process memories for optimal context injection
    pub fn process_for_injection(&self, memories: &[Memory], token_budget: usize) -> String {
        // Step 1: Group memories by topic
        let groups = self.context_grouper.group_for_context(memories);

        // Step 2: Build injection prompt from groups
        let mut injection = String::from("# Relevant Context\n\n");
        
        for group in groups {
            injection.push_str(&format!("## {}\n{}\n\n", group.topic, group.summary));
        }

        // Step 3: Truncate if necessary
        let truncated = self.truncation_engine.truncate_to_budget(&injection, token_budget);
        
        if truncated.len() < injection.len() {
            format!("{}...(truncated)", truncated)
        } else {
            injection
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_memory(id: i64, content: &str) -> Memory {
        Memory {
            id,
            content: content.to_string(),
            memory_type: crate::types::MemoryType::Note,
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
            importance: 0.5,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: Default::default(),
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
            lifecycle_state: Default::default(),
            media_url: None,
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = IntegrationOrchestrator::new();
        // Just test it creates successfully
        let _ = orchestrator;
    }

    #[test]
    fn test_process_for_injection() {
        let orchestrator = IntegrationOrchestrator::new();
        
        let memories = vec![
            create_test_memory(1, "User prefers dark mode"),
        ];

        let result = orchestrator.process_for_injection(&memories, 1000);
        assert!(!result.is_empty());
        assert!(result.contains("Context"));
    }
}
