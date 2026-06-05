//! Compact Operational Context bundles for session resumption.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::context::search::{
    search_context, ArtifactPointer, ContextProvenance, ContextSearchItem, ContextSearchRequest,
    StalenessWarning,
};
use crate::context::{estimate_tokens, ContextBundleMetrics};
use crate::error::Result;

const DEFAULT_BUNDLE_LIMIT: usize = 80;
const DEFAULT_SECTION_LIMIT: usize = 12;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ContextBundleRequest {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub workspace_path_hash: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub max_results: Option<usize>,
    #[serde(default)]
    pub section_limit: Option<usize>,
    #[serde(default)]
    pub include_artifact_pointers: bool,
    #[serde(default)]
    pub current_git_branch: Option<String>,
    #[serde(default)]
    pub current_commit_hash: Option<String>,
    #[serde(default)]
    pub stale_after_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextBundle {
    pub bundle_type: String,
    pub query: Option<String>,
    pub scope: BundleScope,
    pub summary_policy: String,
    pub artifact_policy: String,
    pub failures: Vec<BundleEntry>,
    pub unresolved_blockers: Vec<BundleEntry>,
    pub recent_decisions: Vec<BundleEntry>,
    pub commands_already_run: Vec<CommandEntry>,
    pub files_inspected_or_touched: Vec<FileEntry>,
    pub stale_warnings: Vec<BundleStaleWarning>,
    pub artifact_pointers: Vec<ArtifactPointer>,
    pub relevant_context: Vec<BundleEntry>,
    pub metrics: ContextBundleMetrics,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleScope {
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleEntry {
    pub kind: String,
    pub title: String,
    pub summary: Option<BundleSummary>,
    pub event_type: String,
    pub source: String,
    pub started_at: String,
    pub provenance: ContextProvenance,
    pub staleness: Vec<StalenessWarning>,
    pub artifact_pointers: Vec<ArtifactPointer>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleSummary {
    pub text: String,
    pub derived: bool,
    pub lossy: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandEntry {
    pub command_name: String,
    pub exit_code: Option<i64>,
    pub cwd: Option<String>,
    pub started_at: String,
    pub provenance: ContextProvenance,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub signal: String,
    pub provenance: ContextProvenance,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleStaleWarning {
    pub warning: StalenessWarning,
    pub provenance: ContextProvenance,
}

pub fn build_context_bundle(
    conn: &Connection,
    request: &ContextBundleRequest,
) -> Result<ContextBundle> {
    let section_limit = request
        .section_limit
        .unwrap_or(DEFAULT_SECTION_LIMIT)
        .clamp(1, 50);
    let search_request = ContextSearchRequest {
        query: request.query.clone(),
        repo_id: request.repo_id.clone(),
        workspace_path_hash: request.workspace_path_hash.clone(),
        workspace: request.workspace.clone(),
        session_id: request.session_id.clone(),
        task_id: request.task_id.clone(),
        max_results: Some(request.max_results.unwrap_or(DEFAULT_BUNDLE_LIMIT)),
        include_artifact_pointers: request.include_artifact_pointers,
        current_git_branch: request.current_git_branch.clone(),
        current_commit_hash: request.current_commit_hash.clone(),
        stale_after_days: request.stale_after_days,
        ..Default::default()
    };
    let search = search_context(conn, &search_request)?;

    let mut failures = Vec::new();
    let mut blockers = Vec::new();
    let mut decisions = Vec::new();
    let mut commands = Vec::new();
    let mut files = Vec::new();
    let mut stale_warnings = Vec::new();
    let mut artifact_pointers = Vec::new();
    let mut relevant = Vec::new();

    for item in &search.results {
        if is_failure(item) && failures.len() < section_limit {
            failures.push(bundle_entry("failure", item));
        }
        if is_blocker(item) && blockers.len() < section_limit {
            blockers.push(bundle_entry("unresolved_blocker", item));
        }
        if is_decision(item) && decisions.len() < section_limit {
            decisions.push(bundle_entry("decision", item));
        }
        if let Some(command) = command_entry(item) {
            if commands.len() < section_limit {
                commands.push(command);
            }
        }
        for file in &item.extracted_files {
            if files.len() >= section_limit {
                break;
            }
            if !files.iter().any(|entry: &FileEntry| entry.path == *file) {
                files.push(FileEntry {
                    path: file.clone(),
                    signal: "metadata_path".to_string(),
                    provenance: item.provenance.clone(),
                });
            }
        }
        for warning in &item.staleness {
            stale_warnings.push(BundleStaleWarning {
                warning: warning.clone(),
                provenance: item.provenance.clone(),
            });
        }
        for pointer in &item.artifact_pointers {
            artifact_pointers.push(pointer.clone());
        }
        if relevant.len() < section_limit {
            relevant.push(bundle_entry("relevant_context", item));
        }
    }

    artifact_pointers.sort_by(|a, b| {
        a.artifact_id
            .cmp(&b.artifact_id)
            .then(a.pointer_type.cmp(&b.pointer_type))
    });
    artifact_pointers.dedup_by(|a, b| {
        a.artifact_id == b.artifact_id
            && a.pointer_type == b.pointer_type
            && a.event_id == b.event_id
            && a.summary_id == b.summary_id
    });

    let retrieved_item_count = search.results.len();
    let retrieved_context_tokens_est = search.results.iter().map(item_tokens_est).sum();
    let included_section_entry_count = failures.len()
        + blockers.len()
        + decisions.len()
        + commands.len()
        + files.len()
        + relevant.len();
    let excluded_item_count = retrieved_item_count.saturating_sub(relevant.len());
    let summarized_artifact_ref_count = artifact_pointers
        .iter()
        .filter(|pointer| !pointer.pointer_type.contains("raw"))
        .count();
    let raw_artifact_ref_count = artifact_pointers
        .iter()
        .filter(|pointer| pointer.pointer_type.contains("raw"))
        .count();

    let mut bundle = ContextBundle {
        bundle_type: "operational_context".to_string(),
        query: search.query,
        scope: BundleScope {
            repo_id: search.scope.repo_id,
            workspace_path_hash: search.scope.workspace_path_hash,
            session_id: search.scope.session_id,
            task_id: search.scope.task_id,
        },
        summary_policy: "Summaries are derived/lossy when marked so by reducers.".to_string(),
        artifact_policy: if request.include_artifact_pointers {
            "Artifact pointers included; raw artifact content is never included.".to_string()
        } else {
            "Artifact pointers omitted; raw artifact content is never included.".to_string()
        },
        failures,
        unresolved_blockers: blockers,
        recent_decisions: decisions,
        commands_already_run: commands,
        files_inspected_or_touched: files,
        stale_warnings,
        artifact_pointers,
        relevant_context: relevant,
        metrics: ContextBundleMetrics::default(),
        markdown: String::new(),
    };
    bundle.markdown = render_markdown(&bundle);
    bundle.metrics = ContextBundleMetrics {
        metric_type: "bundle_reuse_audit".to_string(),
        estimated: true,
        method: "chars_div_4_estimate".to_string(),
        retrieved_item_count,
        included_section_entry_count,
        excluded_item_count,
        artifact_pointer_count: bundle.artifact_pointers.len(),
        summarized_artifact_ref_count,
        raw_artifact_ref_count,
        raw_artifact_return_count: 0,
        retrieved_context_tokens_est,
        bundle_tokens_est: estimate_tokens(&bundle.markdown),
        excluded_tokens_est: None,
        notes: vec![
            "Raw artifact content is never included in bundles.".to_string(),
            "Token counts are estimates for audit, not guaranteed savings.".to_string(),
        ],
    };
    Ok(bundle)
}

fn bundle_entry(kind: &str, item: &ContextSearchItem) -> BundleEntry {
    BundleEntry {
        kind: kind.to_string(),
        title: title_for(item),
        summary: item.summary.as_ref().map(|summary| BundleSummary {
            text: truncate(&summary.summary, 700),
            derived: summary.derived,
            lossy: summary.lossy,
            confidence: summary.confidence,
        }),
        event_type: item.event.event_type.clone(),
        source: item.event.source.clone(),
        started_at: item.event.started_at.clone(),
        provenance: item.provenance.clone(),
        staleness: item.staleness.clone(),
        artifact_pointers: item.artifact_pointers.clone(),
    }
}

fn command_entry(item: &ContextSearchItem) -> Option<CommandEntry> {
    item.event
        .command_name
        .as_ref()
        .map(|command_name| CommandEntry {
            command_name: command_name.clone(),
            exit_code: item.event.exit_code,
            cwd: item.event.cwd.clone(),
            started_at: item.event.started_at.clone(),
            provenance: item.provenance.clone(),
        })
}

fn is_failure(item: &ContextSearchItem) -> bool {
    item.event.exit_code.map(|code| code != 0).unwrap_or(false)
        || contains_any(&item.event.event_type, &["fail", "error"])
        || item
            .summary
            .as_ref()
            .map(|summary| contains_any(&summary.summary, &["fail", "error"]))
            .unwrap_or(false)
}

fn is_blocker(item: &ContextSearchItem) -> bool {
    contains_any(&item.event.event_type, &["block", "blocked", "blocker"])
        || contains_any(
            &item.event.metadata.to_string(),
            &["blocker", "blocked", "unresolved"],
        )
        || item
            .summary
            .as_ref()
            .map(|summary| contains_any(&summary.summary, &["blocker", "blocked", "unresolved"]))
            .unwrap_or(false)
}

fn is_decision(item: &ContextSearchItem) -> bool {
    contains_any(&item.event.event_type, &["decision"])
        || item
            .summary
            .as_ref()
            .map(|summary| contains_any(&summary.summary, &["decision", "decided"]))
            .unwrap_or(false)
        || contains_any(&item.event.metadata.to_string(), &["decision", "decided"])
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    let value = value.to_lowercase();
    needles.iter().any(|needle| value.contains(needle))
}

fn item_tokens_est(item: &ContextSearchItem) -> i64 {
    let summary_tokens = item
        .summary
        .as_ref()
        .and_then(|summary| summary.tokens_compact_est)
        .or_else(|| {
            item.summary
                .as_ref()
                .map(|summary| estimate_tokens(&summary.summary))
        })
        .unwrap_or(0);
    let metadata_tokens = estimate_tokens(&item.event.metadata.to_string());
    summary_tokens + metadata_tokens
}

fn title_for(item: &ContextSearchItem) -> String {
    if let Some(summary) = &item.summary {
        return truncate(&summary.summary, 120);
    }
    if let Some(command) = &item.event.command_name {
        return format!("command: {command}");
    }
    if let Some(tool) = &item.event.tool_name {
        return format!("tool: {tool}");
    }
    item.event.event_type.clone()
}

fn render_markdown(bundle: &ContextBundle) -> String {
    let mut out = String::new();
    out.push_str("# Operational Context Bundle\n\n");
    if let Some(query) = &bundle.query {
        out.push_str(&format!("Query: `{}`\n\n", escape_inline(query)));
    }
    out.push_str(
        "Policy: raw artifact content is excluded. Summaries marked lossy are derived/lossy.\n\n",
    );
    render_entries(&mut out, "Recent relevant failures", &bundle.failures);
    render_entries(
        &mut out,
        "Unresolved blockers (inferred)",
        &bundle.unresolved_blockers,
    );
    render_entries(&mut out, "Recent decisions", &bundle.recent_decisions);
    render_commands(&mut out, &bundle.commands_already_run);
    render_files(&mut out, &bundle.files_inspected_or_touched);
    render_stale_warnings(&mut out, &bundle.stale_warnings);
    render_artifacts(&mut out, &bundle.artifact_pointers);
    render_entries(&mut out, "Relevant context", &bundle.relevant_context);
    out
}

fn render_entries(out: &mut String, title: &str, entries: &[BundleEntry]) {
    out.push_str(&format!("## {title}\n"));
    if entries.is_empty() {
        out.push_str("- None found.\n\n");
        return;
    }
    for entry in entries {
        out.push_str(&format!(
            "- {} [{}]\n",
            entry.title,
            provenance_label(&entry.provenance)
        ));
        if let Some(summary) = &entry.summary {
            out.push_str(&format!(
                "  Summary: {} (derived={}, lossy={}, confidence={:.2})\n",
                summary.text, summary.derived, summary.lossy, summary.confidence
            ));
        }
    }
    out.push('\n');
}

fn render_commands(out: &mut String, commands: &[CommandEntry]) {
    out.push_str("## Commands already run\n");
    if commands.is_empty() {
        out.push_str("- None found.\n\n");
        return;
    }
    for command in commands {
        out.push_str(&format!(
            "- `{}` exit={:?} [{}]\n",
            escape_inline(&command.command_name),
            command.exit_code,
            provenance_label(&command.provenance)
        ));
    }
    out.push('\n');
}

fn render_files(out: &mut String, files: &[FileEntry]) {
    out.push_str("## Files inspected or touched\n");
    if files.is_empty() {
        out.push_str("- None found.\n\n");
        return;
    }
    for file in files {
        out.push_str(&format!(
            "- `{}` signal={} [{}]\n",
            escape_inline(&file.path),
            file.signal,
            provenance_label(&file.provenance)
        ));
    }
    out.push('\n');
}

fn render_stale_warnings(out: &mut String, warnings: &[BundleStaleWarning]) {
    out.push_str("## Staleness warnings\n");
    if warnings.is_empty() {
        out.push_str("- None found.\n\n");
        return;
    }
    for stale in warnings {
        out.push_str(&format!(
            "- {}: {} [{}]\n",
            stale.warning.kind,
            stale.warning.message,
            provenance_label(&stale.provenance)
        ));
    }
    out.push('\n');
}

fn render_artifacts(out: &mut String, pointers: &[ArtifactPointer]) {
    out.push_str("## Artifact pointers\n");
    if pointers.is_empty() {
        out.push_str("- None included.\n\n");
        return;
    }
    for pointer in pointers {
        out.push_str(&format!(
            "- {} `{}` [{}]\n",
            pointer.pointer_type,
            escape_inline(&pointer.artifact_id),
            provenance_label(&pointer.provenance)
        ));
    }
    out.push('\n');
}

fn provenance_label(provenance: &ContextProvenance) -> String {
    format!(
        "event_id={} summary_id={:?} session_id={} task_id={:?} source={} started_at={}",
        provenance.event_id,
        provenance.summary_id,
        provenance.session_id,
        provenance.task_id,
        provenance.source,
        provenance.started_at
    )
}

fn truncate(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}...", &value[..boundary])
}

fn escape_inline(value: &str) -> String {
    value.replace('`', "'")
}
