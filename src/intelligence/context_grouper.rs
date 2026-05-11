//! Context Grouper for intelligent memory grouping inspired by RTK
//! Groups memories by topic to optimize context injection for LLMs

use std::collections::HashMap;

/// Memory type - import from parent module
use crate::types::Memory;

/// Represents a group of related memories clustered by topic
#[derive(Debug, Clone)]
pub struct MemoryGroup {
    /// The topic extracted from the group's memories
    pub topic: String,
    /// A summary of the group's contents
    pub summary: String,
    /// Number of memories in the group
    pub count: usize,
    /// IDs of the memories in the group
    pub memory_ids: Vec<i64>,
}

/// Groups memories by topic to optimize context injection for LLMs
/// Inspired by RTK's grouping feature for context management
pub struct ContextGrouper {
    /// Maximum number of memories per group before summarization is triggered
    pub max_group_size: usize,
    /// Minimum similarity threshold for grouping (reserved for future vector-based grouping)
    pub min_similarity: f32,
}

impl ContextGrouper {
    /// Creates a new ContextGrouper with default settings
    pub fn new() -> Self {
        Self {
            max_group_size: 5,
            min_similarity: 0.7,
        }
    }

    /// Groups a list of memories by extracted topic, then summarizes each group
    pub fn group_for_context(&self, memories: &[Memory]) -> Vec<MemoryGroup> {
        let mut groups: HashMap<String, Vec<Memory>> = HashMap::new();

        // 1. Extract topic for each memory and group them
        for memory in memories {
            let topic = self.extract_topic(&memory.content);
            groups.entry(topic).or_default().push(memory.clone());
        }

        // 2. Convert groups to MemoryGroup structs with summaries
        let mut result: Vec<MemoryGroup> = groups
            .into_iter()
            .map(|(topic, mems)| {
                let summary = self.summarize_group(&mems);
                MemoryGroup {
                    topic,
                    summary,
                    count: mems.len(),
                    memory_ids: mems.iter().map(|m| m.id).collect(),
                }
            })
            .collect();

        // 3. Sort by group size (largest first) for relevance
        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    /// Extracts a topic key from memory content.
    ///
    /// Strategy: lowercase, strip punctuation, drop stopwords, then use the
    /// first remaining token as the group key. Coarse on purpose — two
    /// memories sharing their leading content word land in the same group
    /// (e.g. "Rust is..." and "Rust has..." → "rust"). For richer grouping,
    /// callers should layer BM25/vector similarity on top.
    fn extract_topic(&self, content: &str) -> String {
        const STOPWORDS: &[&str] = &[
            "the", "and", "for", "with", "that", "this", "user", "from", "into", "have", "has",
            "was", "were", "are", "but", "not", "you", "your", "our", "their", "his", "her", "its",
            "about",
        ];

        let cleaned: String = content
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();

        cleaned
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .find(|w| w.len() > 3 && !STOPWORDS.contains(&w.as_str()))
            .unwrap_or_else(|| "general".to_string())
    }

    /// Summarizes a group of memories using existing summarization logic
    fn summarize_group(&self, memories: &[Memory]) -> String {
        if memories.len() > self.max_group_size {
            // For large groups, create a compact summary
            format!(
                "Summary of {} memories about related topics",
                memories.len()
            )
        } else {
            // Combine contents for small groups
            let combined = memories
                .iter()
                .map(|m| m.content.as_str())
                .collect::<Vec<&str>>()
                .join(" ");

            if combined.len() > 500 {
                format!("{}... (truncated)", &combined[..500])
            } else {
                combined
            }
        }
    }

    /// Finds all memories in the list that match a given topic
    pub fn find_similar_by_topic(&self, topic: &str, memories: &[Memory]) -> Vec<Memory> {
        memories
            .iter()
            .filter(|m| self.extract_topic(&m.content) == topic)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MemoryType;
    use chrono::Utc;

    // Helper to create test memories
    fn create_test_memory(id: i64, content: &str) -> Memory {
        Memory {
            id,
            content: content.to_string(),
            memory_type: MemoryType::Note,
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
    fn test_group_for_context() {
        let grouper = ContextGrouper::new();
        let memories = vec![
            create_test_memory(1, "User prefers dark mode in UI settings"),
            create_test_memory(2, "User likes coffee with oat milk"),
            create_test_memory(3, "UI color scheme set to dark"),
            create_test_memory(4, "Coffee preference: oat milk only"),
            create_test_memory(5, "User works on Engram project"),
        ];

        let groups = grouper.group_for_context(&memories);
        assert!(!groups.is_empty());
    }

    #[test]
    fn test_extract_topic() {
        let grouper = ContextGrouper::new();
        // "User" is in stopwords, "prefers" is the first content word.
        let topic = grouper.extract_topic("User prefers dark mode in UI settings");
        assert_eq!(topic, "prefers");

        // Same leading content word → same topic key.
        assert_eq!(
            grouper.extract_topic("Rust is a systems programming language"),
            grouper.extract_topic("Rust has memory safety"),
        );

        // No tokens longer than 3 chars after stopwords → fallback.
        assert_eq!(grouper.extract_topic("a b c"), "general");
    }

    #[test]
    fn test_find_similar_by_topic() {
        let grouper = ContextGrouper::new();
        let memories = vec![
            create_test_memory(1, "Rust is a systems programming language"),
            create_test_memory(2, "Python is great for AI"),
            create_test_memory(3, "Rust has memory safety"),
        ];

        let rust_memories = grouper.find_similar_by_topic("rust", &memories);
        assert_eq!(rust_memories.len(), 2);
        assert_eq!(rust_memories[0].id, 1);
        assert_eq!(rust_memories[1].id, 3);
    }

    #[test]
    fn groups_share_keys_for_similar_content() {
        let grouper = ContextGrouper::new();
        let memories = vec![
            create_test_memory(1, "Rust ownership and borrowing"),
            create_test_memory(2, "Rust traits and generics"),
            create_test_memory(3, "Python decorators are cool"),
        ];
        let groups = grouper.group_for_context(&memories);
        // 2 distinct topic keys ("rust" + "python"), not 3.
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].count, 2); // sorted desc by count
        assert_eq!(groups[0].topic, "rust");
    }
}
