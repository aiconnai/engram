use serde::{Deserialize, Deserializer, Serialize};

use super::core::Memory;
use super::memory::{MemoryScope, MemoryTier, MemoryType};

/// Search result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matched memory
    pub memory: Memory,
    /// Overall relevance score
    pub score: f32,
    /// How the result matched
    pub match_info: MatchInfo,
}

/// Information about how a search result matched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchInfo {
    /// Which search strategy was used
    pub strategy: SearchStrategy,
    /// Terms that matched (for keyword search)
    #[serde(default)]
    pub matched_terms: Vec<String>,
    /// Highlighted snippets
    #[serde(default)]
    pub highlights: Vec<String>,
    /// Semantic similarity score (if used)
    pub semantic_score: Option<f32>,
    /// Keyword/BM25 score (if used)
    pub keyword_score: Option<f32>,
}

/// Search strategy used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchStrategy {
    #[serde(alias = "keyword")]
    KeywordOnly,
    #[serde(alias = "semantic")]
    SemanticOnly,
    #[default]
    Hybrid,
}

impl SearchStrategy {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "keyword" | "keyword_only" => Some(SearchStrategy::KeywordOnly),
            "semantic" | "semantic_only" => Some(SearchStrategy::SemanticOnly),
            "hybrid" => Some(SearchStrategy::Hybrid),
            _ => None,
        }
    }
}

fn deserialize_search_strategy_opt<'de, D>(
    deserializer: D,
) -> Result<Option<SearchStrategy>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None => Ok(None),
        Some("auto") => Ok(None),
        Some(other) => SearchStrategy::parse_str(other).map(Some).ok_or_else(|| {
            <D::Error as serde::de::Error>::custom(format!("Invalid search strategy: {}", other))
        }),
    }
}

/// Fields to sort by
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    #[default]
    CreatedAt,
    UpdatedAt,
    LastAccessedAt,
    Importance,
    AccessCount,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Search options wrapper
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    pub limit: Option<i64>,
    pub min_score: Option<f32>,
    pub tags: Option<Vec<String>>,
    #[serde(alias = "type")]
    pub memory_type: Option<MemoryType>,
    /// Force a specific search strategy
    #[serde(default, deserialize_with = "deserialize_search_strategy_opt")]
    pub strategy: Option<SearchStrategy>,
    /// Include match explanations
    #[serde(default)]
    pub explain: bool,
    /// Filter by memory scope
    pub scope: Option<MemoryScope>,
    /// Filter by workspace (single workspace)
    pub workspace: Option<String>,
    /// Filter by multiple workspaces (OR logic)
    pub workspaces: Option<Vec<String>>,
    /// Filter by memory tier
    pub tier: Option<MemoryTier>,
    /// Include transcript chunks in search (default: false)
    /// By default, transcript_chunk memories are excluded from search
    #[serde(default)]
    pub include_transcripts: bool,
    /// Advanced filter expression with AND/OR/comparison operators (RML-932)
    /// Takes precedence over `tags` and `memory_type` if specified
    pub filter: Option<serde_json::Value>,
    // Phase 5 - Lifecycle management (ENG-37)
    /// Include archived memories in search results (default: false)
    #[serde(default)]
    pub include_archived: bool,
    /// Filter by hierarchical scope path (prefix search).
    ///
    /// When set, only memories whose `scope_path` starts with (or equals) this value
    /// are returned. For example, `"global/org:acme"` will match memories at
    /// `"global/org:acme"`, `"global/org:acme/user:alice"`, etc.
    pub scope_path: Option<String>,
    /// Search across all workspaces (default: false).
    ///
    /// When `true`, ignores any `workspace` or `workspaces` filter and returns
    /// results from all workspaces. Each result will include a `workspace` field
    /// in the MCP handler response.
    #[serde(default)]
    pub global: bool,
}
