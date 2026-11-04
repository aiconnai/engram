//! Export workflow; transport decoding and wire formatting live in adapters.
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

use super::filesystem::{validate_component, write_batch};
use super::format::{
    build_index_markdown, compute_file_subdir, format_memory_markdown, sanitize_filename,
};
use super::query::query_export_snapshot;
use crate::portability::markdown::{paths::validate_export_dir, MarkdownContext};

pub(crate) struct ExportRequest<'a> {
    pub workspace: &'a str,
    pub output_dir: Option<&'a str>,
    pub group: &'a str,
    pub include_links: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportOutcome {
    pub files_written: usize,
    pub output_dir: String,
    pub index_path: String,
    pub memories_exported: usize,
    pub type_breakdown: HashMap<String, usize>,
    pub group: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ExportFailure {
    #[error("No memories found in workspace '{0}'")]
    EmptyWorkspace(String),
    #[error("{0}")]
    Failed(String),
}

pub(crate) fn export(
    ctx: &MarkdownContext<'_>,
    request: ExportRequest<'_>,
) -> Result<ExportOutcome, ExportFailure> {
    let ExportRequest {
        workspace,
        output_dir,
        group,
        include_links,
    } = request;
    let default_dir = format!("./engram-export/{workspace}");
    let output_path = validate_export_dir(output_dir.unwrap_or(&default_dir))
        .map_err(|e| ExportFailure::Failed(format!("Invalid output_dir: {e}")))?;
    if !matches!(group, "flat" | "day" | "workspace" | "type" | "entity") {
        return Err(ExportFailure::Failed("Invalid export group".into()));
    }
    let (memories, related_map) = query_export_snapshot(ctx, workspace, include_links)
        .map_err(|e| ExportFailure::Failed(format!("Failed to read export snapshot: {e}")))?;
    if memories.is_empty() {
        return Err(ExportFailure::EmptyWorkspace(workspace.to_string()));
    }
    let mut id_to_filename = HashMap::new();
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut paths = Vec::with_capacity(memories.len());
    for mem in &memories {
        let Some(id) = mem["id"].as_i64() else {
            return Err(ExportFailure::Failed("Invalid exported memory id".into()));
        };
        let content = mem["content"].as_str().unwrap_or("");
        let filename = format!(
            "{id}-{}",
            sanitize_filename(content.lines().next().unwrap_or("untitled"))
        );
        let mut relative = PathBuf::new();
        if let Some(subdir) = compute_file_subdir(group, mem) {
            validate_component(&subdir)
                .map_err(|_| ExportFailure::Failed("Unsafe export grouping component".into()))?;
            relative.push(subdir);
        }
        relative.push(format!("{filename}.md"));
        paths.push(relative);
        id_to_filename.insert(id, filename);
        *type_counts
            .entry(mem["memory_type"].as_str().unwrap_or("note").to_string())
            .or_default() += 1;
    }
    // Read and render the complete batch before touching the filesystem.
    let mut batch: Vec<_> = memories
        .iter()
        .zip(paths)
        .map(|(mem, path)| {
            (
                path,
                format_memory_markdown(mem, include_links, &related_map, &id_to_filename),
            )
        })
        .collect();
    batch.push((
        PathBuf::from("index.md"),
        build_index_markdown(workspace, &memories, &type_counts, &id_to_filename, group),
    ));
    write_batch(&output_path, &batch, ctx.storage).map_err(|e| {
        ExportFailure::Failed(format!(
            "Failed to write export (batch may be incomplete): {e}"
        ))
    })?;
    Ok(ExportOutcome {
        files_written: batch.len(),
        output_dir: output_path.to_string_lossy().into_owned(),
        index_path: output_path.join("index.md").to_string_lossy().into_owned(),
        memories_exported: memories.len(),
        type_breakdown: type_counts,
        group: group.into(),
    })
}
