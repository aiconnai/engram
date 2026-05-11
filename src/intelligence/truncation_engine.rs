//! Truncation Engine for intelligent context truncation (RTK-inspired)
//! Provides smart truncation strategies for fitting context within token budgets

use serde::{Deserialize, Serialize};

/// Strategy for truncating content
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TruncationStrategy {
    /// Simple character-based truncation
    Simple,
    /// Smart truncation preserving sentence boundaries
    Smart,
    /// Preserve most recent content (sliding window)
    PreserveRecent,
}

/// Configuration for the truncation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncationConfig {
    /// Maximum tokens for the output
    pub max_tokens: usize,
    /// Number of tokens to preserve for recent content
    pub preserve_recent: usize,
    /// Truncation strategy to use
    pub strategy: TruncationStrategy,
}

impl Default for TruncationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2000,
            preserve_recent: 1000,
            strategy: TruncationStrategy::Smart,
        }
    }
}

/// Engine for intelligently truncating content to fit token budgets
pub struct TruncationEngine {
    config: TruncationConfig,
}

impl TruncationEngine {
    /// Create a new TruncationEngine with the given config
    pub fn with_config(config: TruncationConfig) -> Self {
        Self { config }
    }

    /// Truncate content to fit within a token budget
    pub fn truncate_to_budget(&self, content: &str, budget_tokens: usize) -> String {
        let budget_chars = budget_tokens * 4; // rough estimate: 1 token ≈ 4 chars

        if content.len() <= budget_chars {
            return content.to_string();
        }

        match self.config.strategy {
            TruncationStrategy::Simple => {
                let truncated = content.chars().take(budget_chars).collect::<String>();
                format!("{}...", truncated)
            }
            TruncationStrategy::Smart => {
                // Try to break at sentence boundaries
                let mut result = String::new();
                let mut char_count = 0;

                for sentence in content.split('.') {
                    let sentence_with_period = format!("{}.", sentence);
                    if char_count + sentence_with_period.len() <= budget_chars {
                        result.push_str(&sentence_with_period);
                        char_count += sentence_with_period.len();
                    } else {
                        break;
                    }
                }

                if result.is_empty() {
                    // Fallback to simple truncation
                    let truncated = content.chars().take(budget_chars - 3).collect::<String>();
                    format!("{}...", truncated)
                } else {
                    result
                }
            }
            TruncationStrategy::PreserveRecent => {
                // Keep the most recent content (end of the string)
                let start = content.len().saturating_sub(budget_chars);
                let recent = &content[start..];
                format!("...{}", recent)
            }
        }
    }

    /// Estimate the number of tokens in a string
    pub fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4 // rough estimate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_truncation() {
        let engine = TruncationEngine::with_config(TruncationConfig {
            max_tokens: 10,
            preserve_recent: 5,
            strategy: TruncationStrategy::Simple,
        });

        let content = "This is a long piece of content that needs to be truncated";
        let result = engine.truncate_to_budget(content, 10);
        assert!(result.len() <= 43); // 10 tokens * 4 chars + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_smart_truncation() {
        let engine = TruncationEngine::with_config(TruncationConfig {
            max_tokens: 20,
            preserve_recent: 10,
            strategy: TruncationStrategy::Smart,
        });

        let content = "First sentence. Second sentence. Third sentence. Fourth sentence.";
        let result = engine.truncate_to_budget(content, 20);
        // Should break at sentence boundary
        assert!(result.contains("First sentence"));
    }

    #[test]
    fn test_estimate_tokens() {
        let engine = TruncationEngine::with_config(Default::default());
        let text = "Hello world";
        let tokens = engine.estimate_tokens(text);
        assert_eq!(tokens, 3); // 11 chars / 4 ≈ 2.75 → 3
    }
}
