//! Tool-output observation/archival and working-memory tools.
use serde_json::{json, Value};

use super::super::HandlerContext;
use super::safe_truncate;

/// Observe and record a tool invocation as an Episodic memory.
///
/// Params:
/// - `tool_name` (string, required) — name of the tool that was called
/// - `tool_input` (any JSON value, required) — the input passed to the tool
/// - `tool_output` (string, required) — the output returned by the tool
/// - `session_id` (string, optional, default: "unknown") — session identifier for grouping
/// - `compress` (bool, optional, default: true) — compact vs full JSON storage
pub fn memory_observe_tool_use(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::create_memory;
    use crate::types::{CreateMemoryInput, MemoryType};

    let tool_name = match params.get("tool_name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return json!({"error": "tool_name is required"}),
    };

    let tool_input = match params.get("tool_input") {
        Some(v) => v.clone(),
        None => return json!({"error": "tool_input is required"}),
    };

    let tool_output = match params.get("tool_output").and_then(|v| v.as_str()) {
        Some(o) => o.to_string(),
        None => return json!({"error": "tool_output is required"}),
    };

    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let compress = params
        .get("compress")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let content = if compress {
        let input_str = serde_json::to_string(&tool_input).unwrap_or_default();
        let input_preview = if input_str.len() > 200 {
            format!("{}…", safe_truncate(&input_str, 200))
        } else {
            input_str
        };
        let output_preview = if tool_output.len() > 200 {
            format!("{}…", safe_truncate(&tool_output, 200))
        } else {
            tool_output.clone()
        };
        format!(
            "[{}] input→{} output→{}",
            tool_name, input_preview, output_preview
        )
    } else {
        serde_json::to_string(&json!({
            "tool_name": tool_name,
            "input": tool_input,
            "output": tool_output
        }))
        .unwrap_or_else(|_| format!("[{}] observation", tool_name))
    };

    let tags = vec![
        "tool-observation".to_string(),
        format!("session:{}", session_id),
        tool_name.clone(),
    ];

    let input = CreateMemoryInput {
        content,
        memory_type: MemoryType::Episodic,
        tags,
        workspace: Some("default".to_string()),
        ..Default::default()
    };

    let result = ctx
        .storage
        .with_transaction(|conn| create_memory(conn, &input));

    match result {
        Ok(memory) => json!({
            "id": memory.id,
            "compressed": compress
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── Endless Mode: tool output archival ───────────────────────────────

/// Archive a tool's full raw output as an Episodic memory and return a compact
/// summary, solving the O(N²) context window growth problem.
///
/// Params:
/// - `tool_name` (string, required) — name of the tool whose output is being archived
/// - `raw_output` (string, required) — full raw output string
/// - `session_id` (string, optional, default: "unknown") — session identifier
/// - `compress_summary` (bool, optional, default: true) — whether to generate a summary
/// - `summary_tokens` (usize, optional, default: 500) — max tokens for the summary
pub fn memory_archive_tool_output(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::create_memory;
    use crate::types::{CreateMemoryInput, MemoryType};

    let tool_name = match params.get("tool_name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return json!({"error": "tool_name is required"}),
    };

    let raw_output = match params.get("raw_output").and_then(|v| v.as_str()) {
        Some(o) => o.to_string(),
        None => return json!({"error": "raw_output is required"}),
    };

    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let compress_summary = params
        .get("compress_summary")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let summary_tokens = params
        .get("summary_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as usize;

    // Step 1: Store the full raw output as an Episodic memory in workspace "archive".
    let tags = vec![
        "tool-archive".to_string(),
        format!("session:{}", session_id),
        tool_name.clone(),
    ];

    let input = CreateMemoryInput {
        content: raw_output.clone(),
        memory_type: MemoryType::Episodic,
        tags,
        workspace: Some("archive".to_string()),
        ..Default::default()
    };

    let archive_memory = match ctx
        .storage
        .with_transaction(|conn| create_memory(conn, &input))
    {
        Ok(m) => m,
        Err(e) => return json!({"error": e.to_string()}),
    };

    let archive_id = archive_memory.id;

    // Step 2: Build summary.
    let summary = if compress_summary {
        let max_chars = summary_tokens * 4;
        let slice = safe_truncate(&raw_output, max_chars);

        // Find last sentence boundary within the slice.
        let boundary = slice
            .rfind(['.', '!', '?', '\n'])
            .map(|pos| pos + 1)
            .unwrap_or(slice.len());

        let trimmed = slice[..boundary].trim_end();
        format!("[{} summary] {}", tool_name, trimmed)
    } else {
        raw_output.clone()
    };

    // Step 3: Compute token estimates.
    let raw_tokens_estimate = raw_output.len() / 4;
    let summary_tokens_estimate = summary.len() / 4;
    let compression_ratio = summary_tokens_estimate as f64 / (raw_tokens_estimate.max(1)) as f64;

    json!({
        "archive_id": archive_id,
        "summary": summary,
        "raw_tokens_estimate": raw_tokens_estimate,
        "summary_tokens_estimate": summary_tokens_estimate,
        "compression_ratio": compression_ratio
    })
}

/// Retrieve the full raw output for a previously archived tool output.
///
/// Params:
/// - `archive_id` (i64, required) — ID returned by `memory_archive_tool_output`
pub fn memory_get_archived_output(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_memory;

    let archive_id = match params.get("archive_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "archive_id is required"}),
    };

    let memory = match ctx
        .storage
        .with_connection(|conn| match get_memory(conn, archive_id) {
            Ok(m) => Ok(Some(m)),
            Err(crate::error::EngramError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }) {
        Ok(Some(m)) => m,
        Ok(None) => return json!({"error": "Archive not found", "archive_id": archive_id}),
        Err(e) => return json!({"error": e.to_string()}),
    };

    // Extract tool_name from tags: the first tag that isn't "tool-archive" or starts with "session:".
    let tool_name = memory
        .tags
        .iter()
        .find(|t| *t != "tool-archive" && !t.starts_with("session:"))
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    json!({
        "archive_id": archive_id,
        "tool_name": tool_name,
        "content": memory.content,
        "created_at": memory.created_at.to_rfc3339()
    })
}

/// Assemble a structured working-memory markdown block for the current session.
///
/// Combines compact tool-observations with references to archived full outputs,
/// keeping context growth O(1) per tool call instead of O(N).
///
/// Params:
/// - `session_id` (string, required)
/// - `token_budget` (usize, optional, default: 4000)
/// - `include_tool_names` (array of string, optional) — whitelist of tool names to include
/// - `since_minutes` (u64, optional) — only include observations from the last N minutes
pub fn memory_get_working_memory(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::list_memories;
    use crate::types::{ListOptions, SortField, SortOrder};
    use chrono::{Duration, Utc};

    let session_id = match params.get("session_id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "session_id is required"}),
    };

    let token_budget = params
        .get("token_budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(4000) as usize;

    let include_tool_names: Vec<String> = params
        .get("include_tool_names")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    // Optional recency filter: only observations created within the last N minutes.
    let since_cutoff = params
        .get("since_minutes")
        .and_then(|v| v.as_u64())
        .map(|mins| Utc::now() - Duration::minutes(mins as i64));

    let session_tag = format!("session:{}", session_id);

    // Helper: check whether a memory's tags contain the session tag.
    let has_session_tag = |tags: &[String]| tags.contains(&session_tag);

    // Helper: extract the tool name from a memory's tags.
    let extract_tool_name = |tags: &[String], exclude_prefix: &str| -> String {
        tags.iter()
            .find(|t| *t != exclude_prefix && !t.starts_with("session:"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    };

    // Helper: check include_tool_names filter.
    let passes_tool_filter = |tags: &[String]| -> bool {
        if include_tool_names.is_empty() {
            return true;
        }
        tags.iter().any(|t| include_tool_names.contains(t))
    };

    // Fetch tool observations from workspace "default".
    let obs_options = ListOptions {
        workspace: Some("default".to_string()),
        tags: Some(vec!["tool-observation".to_string()]),
        sort_by: Some(SortField::CreatedAt),
        sort_order: Some(SortOrder::Asc),
        limit: Some(1000),
        ..Default::default()
    };

    let all_observations = match ctx
        .storage
        .with_connection(|conn| list_memories(conn, &obs_options))
    {
        Ok(mems) => mems,
        Err(e) => return json!({"error": e.to_string()}),
    };

    // Filter observations by session tag, optional tool name whitelist, and recency.
    let observations: Vec<_> = all_observations
        .into_iter()
        .filter(|m| {
            has_session_tag(&m.tags)
                && passes_tool_filter(&m.tags)
                && since_cutoff.is_none_or(|cutoff| m.created_at >= cutoff)
        })
        .collect();

    // Fetch archive entries from workspace "archive".
    let archive_options = ListOptions {
        workspace: Some("archive".to_string()),
        tags: Some(vec!["tool-archive".to_string()]),
        sort_by: Some(SortField::CreatedAt),
        sort_order: Some(SortOrder::Asc),
        limit: Some(1000),
        ..Default::default()
    };

    let all_archives = match ctx
        .storage
        .with_connection(|conn| list_memories(conn, &archive_options))
    {
        Ok(mems) => mems,
        Err(e) => return json!({"error": e.to_string()}),
    };

    // Filter archive entries by session tag, optional tool name whitelist, and recency.
    let archives: Vec<_> = all_archives
        .into_iter()
        .filter(|m| {
            has_session_tag(&m.tags)
                && passes_tool_filter(&m.tags)
                && since_cutoff.is_none_or(|cutoff| m.created_at >= cutoff)
        })
        .collect();

    // Build archive_refs for the return value.
    let archive_refs: Vec<Value> = archives
        .iter()
        .map(|m| {
            let tool_name = extract_tool_name(&m.tags, "tool-archive");
            json!({"id": m.id, "tool_name": tool_name})
        })
        .collect();

    // Pre-compute the archive-refs section so we can reserve its size before
    // budgeting observation content (fixes P2: archive refs previously appended
    // without any budget check, allowing overflow past token_budget).
    let archive_section: String = archives
        .iter()
        .map(|m| {
            let tn = extract_tool_name(&m.tags, "tool-archive");
            format!(
                "**Archive ref:** [{}] ID={} — call `memory_get_archived_output` with archive_id={} to retrieve full output\n",
                tn, m.id, m.id
            )
        })
        .collect();

    // Reserve 500 tokens for structural markdown + archive section, then split
    // the rest evenly across observations.
    let archive_reserved = archive_section.len() / 4;
    let obs_count = observations.len();
    let content_budget_chars = (token_budget.saturating_sub(500 + archive_reserved)) * 4;
    let chars_per_obs = content_budget_chars
        .checked_div(obs_count)
        .unwrap_or(content_budget_chars);

    // Build markdown.
    let mut md = format!(
        "# Working Memory — Session {}\n\n## Tool Observations ({} total)\n\n",
        session_id, obs_count
    );

    for (i, m) in observations.iter().enumerate() {
        let tool_name = extract_tool_name(&m.tags, "tool-observation");
        let content = if m.content.len() > chars_per_obs && chars_per_obs > 0 {
            format!("{}…", safe_truncate(&m.content, chars_per_obs))
        } else {
            m.content.clone()
        };
        md.push_str(&format!(
            "### {} (observation #{})\n{}\n\n---\n",
            tool_name,
            i + 1,
            content
        ));
    }

    md.push_str(&archive_section);

    let tokens_estimate = md.len() / 4;

    json!({
        "working_memory": md,
        "observation_count": obs_count,
        "archive_count": archives.len(),
        "archive_refs": archive_refs,
        "tokens_estimate": tokens_estimate
    })
}
