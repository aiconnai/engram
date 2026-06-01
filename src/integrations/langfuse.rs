//! Langfuse integration for observability-to-memory pipeline (Phase 3 - ENG-35)
//!
//! This module provides integration with Langfuse for:
//! - Fetching traces and generations
//! - Extracting patterns from successful/failed prompts
//! - Converting traces to memories automatically
//!
//! All code is feature-gated behind `#[cfg(feature = "langfuse")]`

use crate::error::{EngramError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Langfuse API error
#[derive(Debug, thiserror::Error)]
pub enum LangfuseError {
    #[error("Langfuse API error: {0}")]
    ApiError(String),
    #[error("Langfuse connection error: {0}")]
    ConnectionError(String),
    #[error("Langfuse authentication failed")]
    AuthenticationFailed,
    #[error("Rate limited by Langfuse API")]
    RateLimited,
}

impl From<LangfuseError> for EngramError {
    fn from(e: LangfuseError) -> Self {
        EngramError::Internal(e.to_string())
    }
}

/// Validate a Langfuse base URL to prevent SSRF attacks.
///
/// Rules:
/// - Must use `https://` (unless `ENGRAM_ALLOW_HTTP_INTEGRATIONS=1` is set)
/// - Must not target loopback addresses (`127.0.0.1`, `::1`, `localhost`)
/// - Must not target link-local addresses (`169.254.x.x`)
pub fn validate_base_url(url: &str) -> Result<()> {
    let allow_http = std::env::var("ENGRAM_ALLOW_HTTP_INTEGRATIONS")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !allow_http && !url.starts_with("https://") {
        return Err(EngramError::InvalidInput(
            "LANGFUSE_BASE_URL must use https:// scheme".to_string(),
        ));
    }

    // Extract the host portion for IP/hostname checks.
    // Strip scheme prefix (http:// or https://).
    let after_scheme = url
        .find("://")
        .map(|i| &url[i + 3..])
        .unwrap_or(url);

    // Take only up to the first '/' or end of string (removes path).
    let host_port = after_scheme
        .split('/')
        .next()
        .unwrap_or(after_scheme);

    // Strip port if present (handle [::1]:8080 too).
    let host = if host_port.starts_with('[') {
        // IPv6 bracketed: [::1] or [::1]:port
        host_port
            .trim_start_matches('[')
            .split(']')
            .next()
            .unwrap_or(host_port)
    } else {
        // IPv4 or hostname: 127.0.0.1:8080 or localhost:3000
        host_port.split(':').next().unwrap_or(host_port)
    };

    // Reject loopback.
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return Err(EngramError::InvalidInput(format!(
            "LANGFUSE_BASE_URL must not target loopback address: {host}"
        )));
    }

    // Reject link-local (169.254.0.0/16).
    if host.starts_with("169.254.") {
        return Err(EngramError::InvalidInput(
            "LANGFUSE_BASE_URL must not target link-local address (169.254.x.x)".to_string(),
        ));
    }

    Ok(())
}

/// Langfuse configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangfuseConfig {
    pub public_key: String,
    pub secret_key: String,
    pub base_url: String,
}

impl LangfuseConfig {
    /// Create config from environment variables.
    ///
    /// Returns `None` if required keys are missing.
    /// Panics if `LANGFUSE_BASE_URL` is set to an unsafe value (loopback, link-local, or non-https).
    /// Use `ENGRAM_ALLOW_HTTP_INTEGRATIONS=1` to permit `http://` in development only.
    pub fn from_env() -> Option<Self> {
        let public_key = std::env::var("LANGFUSE_PUBLIC_KEY").ok()?;
        let secret_key = std::env::var("LANGFUSE_SECRET_KEY").ok()?;
        let base_url = std::env::var("LANGFUSE_BASE_URL")
            .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string());

        validate_base_url(&base_url)
            .expect("LANGFUSE_BASE_URL failed security validation");

        Some(Self {
            public_key,
            secret_key,
            base_url,
        })
    }
}

/// A trace from Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub id: String,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub level: Option<String>,
    pub status_message: Option<String>,
    pub scores: Vec<TraceScore>,
}

/// A score attached to a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceScore {
    pub name: String,
    pub value: f64,
    pub comment: Option<String>,
}

/// A generation (LLM call) within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceGeneration {
    pub id: String,
    pub trace_id: String,
    pub name: Option<String>,
    pub model: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub usage: Option<GenerationUsage>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub status_message: Option<String>,
    pub level: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub completion_start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Token usage for a generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationUsage {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
}

/// Sync task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTask {
    pub task_id: String,
    pub task_type: String,
    pub status: SyncStatus,
    pub progress_percent: i32,
    pub traces_processed: i64,
    pub memories_created: i64,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Sync status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Running,
    Completed,
    Failed,
}

/// Progress update for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub traces_processed: i64,
    pub memories_created: i64,
    pub errors: Vec<String>,
    pub last_trace_id: Option<String>,
}

/// Pattern extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExtraction {
    pub pattern_type: PatternType,
    pub description: String,
    pub confidence: f64,
    pub source_trace_ids: Vec<String>,
    pub suggested_memory_type: String,
    pub suggested_content: String,
    pub suggested_tags: Vec<String>,
}

/// Types of patterns that can be extracted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    SuccessfulPrompt,
    ErrorPattern,
    UserPreference,
    ToolUsage,
    WorkflowPattern,
}

/// Langfuse API client
pub struct LangfuseClient {
    client: reqwest::Client,
    config: LangfuseConfig,
}

impl LangfuseClient {
    /// Create a new Langfuse client
    pub fn new(config: LangfuseConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// Create client from environment variables
    pub fn from_env() -> Option<Self> {
        LangfuseConfig::from_env().map(Self::new)
    }

    /// Test connection to Langfuse
    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/api/public/health", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LangfuseError::ConnectionError(e.to_string()))?;

        Ok(response.status().is_success())
    }

    /// Fetch traces since a given timestamp
    pub async fn fetch_traces(&self, since: DateTime<Utc>, limit: usize) -> Result<Vec<Trace>> {
        let url = format!("{}/api/public/traces", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.public_key, Some(&self.config.secret_key))
            .query(&[
                ("fromTimestamp", since.to_rfc3339()),
                ("limit", limit.to_string()),
            ])
            .send()
            .await
            .map_err(|e| LangfuseError::ConnectionError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(LangfuseError::AuthenticationFailed.into());
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LangfuseError::RateLimited.into());
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LangfuseError::ApiError(format!("Status {}: {}", status, body)).into());
        }

        #[derive(Deserialize)]
        struct TraceResponse {
            data: Vec<Trace>,
        }

        let trace_response: TraceResponse = response
            .json()
            .await
            .map_err(|e| LangfuseError::ApiError(format!("Failed to parse response: {}", e)))?;

        Ok(trace_response.data)
    }

    /// Fetch generations for a specific trace
    pub async fn fetch_generations(&self, trace_id: &str) -> Result<Vec<TraceGeneration>> {
        let url = format!("{}/api/public/observations", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.public_key, Some(&self.config.secret_key))
            .query(&[("traceId", trace_id), ("type", "GENERATION")])
            .send()
            .await
            .map_err(|e| LangfuseError::ConnectionError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LangfuseError::ApiError(format!("Status {}: {}", status, body)).into());
        }

        #[derive(Deserialize)]
        struct GenerationResponse {
            data: Vec<TraceGeneration>,
        }

        let gen_response: GenerationResponse = response
            .json()
            .await
            .map_err(|e| LangfuseError::ApiError(format!("Failed to parse response: {}", e)))?;

        Ok(gen_response.data)
    }

    /// Fetch a single trace by ID
    pub async fn fetch_trace(&self, trace_id: &str) -> Result<Option<Trace>> {
        let url = format!("{}/api/public/traces/{}", self.config.base_url, trace_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.public_key, Some(&self.config.secret_key))
            .send()
            .await
            .map_err(|e| LangfuseError::ConnectionError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LangfuseError::ApiError(format!("Status {}: {}", status, body)).into());
        }

        let trace: Trace = response
            .json()
            .await
            .map_err(|e| LangfuseError::ApiError(format!("Failed to parse response: {}", e)))?;

        Ok(Some(trace))
    }
}

/// Extract patterns from traces
pub fn extract_patterns(traces: &[Trace]) -> Vec<PatternExtraction> {
    let mut patterns = Vec::new();

    // Group traces by success/failure
    let successful: Vec<_> = traces
        .iter()
        .filter(|t| {
            t.scores
                .iter()
                .any(|s| s.name.to_lowercase().contains("success") && s.value > 0.7)
        })
        .collect();

    let failed: Vec<_> = traces
        .iter()
        .filter(|t| t.level.as_deref() == Some("ERROR") || t.status_message.is_some())
        .collect();

    // Extract successful prompt patterns
    if successful.len() >= 3 {
        patterns.push(PatternExtraction {
            pattern_type: PatternType::SuccessfulPrompt,
            description: format!(
                "Found {} successful traces with high scores",
                successful.len()
            ),
            confidence: 0.8,
            source_trace_ids: successful.iter().take(5).map(|t| t.id.clone()).collect(),
            suggested_memory_type: "procedural".to_string(),
            suggested_content: format!("Successful pattern from {} traces", successful.len()),
            suggested_tags: vec![
                "langfuse".to_string(),
                "pattern".to_string(),
                "success".to_string(),
            ],
        });
    }

    // Extract error patterns
    if failed.len() >= 3 {
        let error_messages: Vec<_> = failed
            .iter()
            .filter_map(|t| t.status_message.as_ref())
            .collect();

        patterns.push(PatternExtraction {
            pattern_type: PatternType::ErrorPattern,
            description: format!("Found {} traces with errors", failed.len()),
            confidence: 0.7,
            source_trace_ids: failed.iter().take(5).map(|t| t.id.clone()).collect(),
            suggested_memory_type: "issue".to_string(),
            suggested_content: format!(
                "Error pattern: {} occurrences. Sample errors: {:?}",
                failed.len(),
                error_messages.into_iter().take(3).collect::<Vec<_>>()
            ),
            suggested_tags: vec![
                "langfuse".to_string(),
                "pattern".to_string(),
                "error".to_string(),
            ],
        });
    }

    // Extract user preference patterns (repeated choices)
    let mut user_actions: HashMap<String, i32> = HashMap::new();
    for trace in traces {
        if let Some(user_id) = &trace.user_id {
            if let Some(name) = &trace.name {
                let key = format!("{}:{}", user_id, name);
                *user_actions.entry(key).or_insert(0) += 1;
            }
        }
    }

    for (key, count) in user_actions {
        if count >= 5 {
            patterns.push(PatternExtraction {
                pattern_type: PatternType::UserPreference,
                description: format!("User preference: {} (seen {} times)", key, count),
                confidence: (count as f64 / 10.0).min(0.95),
                source_trace_ids: vec![],
                suggested_memory_type: "preference".to_string(),
                suggested_content: format!("User frequently uses: {}", key),
                suggested_tags: vec!["langfuse".to_string(), "preference".to_string()],
            });
        }
    }

    patterns
}

/// Convert a trace to memory content
pub fn trace_to_memory_content(trace: &Trace, generations: &[TraceGeneration]) -> String {
    let mut content = String::new();

    // Add trace info
    if let Some(name) = &trace.name {
        content.push_str(&format!("# {}\n\n", name));
    }

    content.push_str(&format!("**Trace ID:** {}\n", trace.id));
    content.push_str(&format!("**Timestamp:** {}\n", trace.timestamp));

    if let Some(user_id) = &trace.user_id {
        content.push_str(&format!("**User:** {}\n", user_id));
    }

    if let Some(session_id) = &trace.session_id {
        content.push_str(&format!("**Session:** {}\n", session_id));
    }

    // Add input/output
    if let Some(input) = &trace.input {
        content.push_str(&format!(
            "\n## Input\n```json\n{}\n```\n",
            serde_json::to_string_pretty(input).unwrap_or_default()
        ));
    }

    if let Some(output) = &trace.output {
        content.push_str(&format!(
            "\n## Output\n```json\n{}\n```\n",
            serde_json::to_string_pretty(output).unwrap_or_default()
        ));
    }

    // Add scores
    if !trace.scores.is_empty() {
        content.push_str("\n## Scores\n");
        for score in &trace.scores {
            content.push_str(&format!("- **{}:** {:.2}", score.name, score.value));
            if let Some(comment) = &score.comment {
                content.push_str(&format!(" ({})", comment));
            }
            content.push('\n');
        }
    }

    // Add generations
    if !generations.is_empty() {
        content.push_str("\n## Generations\n");
        for gen in generations {
            if let Some(model) = &gen.model {
                content.push_str(&format!("\n### {} ({})\n", gen.id, model));
            } else {
                content.push_str(&format!("\n### {}\n", gen.id));
            }

            if let Some(usage) = &gen.usage {
                if let Some(total) = usage.total_tokens {
                    content.push_str(&format!("Tokens: {}\n", total));
                }
            }
        }
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_base_url tests ───────────────────────────────────────────────

    #[test]
    fn test_validate_base_url_rejects_http() {
        let result = validate_base_url("http://cloud.langfuse.com");
        assert!(result.is_err(), "http:// must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("https"),
            "error should mention https requirement, got: {msg}"
        );
    }

    #[test]
    fn test_validate_base_url_rejects_link_local() {
        let result = validate_base_url("https://169.254.169.254/latest/meta-data");
        assert!(result.is_err(), "link-local address must be rejected");
    }

    #[test]
    fn test_validate_base_url_rejects_loopback_localhost() {
        let result = validate_base_url("https://localhost/api");
        assert!(result.is_err(), "localhost must be rejected");
    }

    #[test]
    fn test_validate_base_url_rejects_loopback_ipv4() {
        let result = validate_base_url("https://127.0.0.1/api");
        assert!(result.is_err(), "127.0.0.1 must be rejected");
    }

    #[test]
    fn test_validate_base_url_rejects_loopback_ipv6() {
        let result = validate_base_url("https://[::1]/api");
        assert!(result.is_err(), "::1 must be rejected");
    }

    #[test]
    fn test_validate_base_url_accepts_https_public() {
        let result = validate_base_url("https://cloud.langfuse.com");
        assert!(result.is_ok(), "public https URL must be accepted, got: {result:?}");
    }

    #[test]
    fn test_validate_base_url_accepts_custom_https() {
        let result = validate_base_url("https://api.langfuse.com");
        assert!(result.is_ok(), "custom https URL must be accepted");
    }

    // ── existing tests ────────────────────────────────────────────────────────

    #[test]
    fn test_config_from_env() {
        // This test just verifies the function doesn't panic
        // Actual env vars won't be set in tests
        let _ = LangfuseConfig::from_env();
    }

    #[test]
    fn test_extract_patterns_empty() {
        let patterns = extract_patterns(&[]);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_trace_to_memory_content() {
        let trace = Trace {
            id: "test-123".to_string(),
            name: Some("Test Trace".to_string()),
            user_id: Some("user-1".to_string()),
            session_id: None,
            input: Some(serde_json::json!({"query": "hello"})),
            output: Some(serde_json::json!({"response": "world"})),
            metadata: None,
            tags: vec!["test".to_string()],
            timestamp: Utc::now(),
            level: None,
            status_message: None,
            scores: vec![TraceScore {
                name: "quality".to_string(),
                value: 0.9,
                comment: None,
            }],
        };

        let content = trace_to_memory_content(&trace, &[]);
        assert!(content.contains("Test Trace"));
        assert!(content.contains("test-123"));
        assert!(content.contains("quality"));
    }
}
