use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{json, Value};

use super::filesystem::{validate_component, write_batch};
use super::format::{
    build_index_markdown, compute_file_subdir, format_memory_markdown, sanitize_filename,
};
use super::query::query_export_snapshot;
use crate::mcp::handlers::markdown_export::validate_export_dir;
use crate::mcp::handlers::HandlerContext;

pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
    // Direct CLI/library callers receive the same authorization as dispatch.
    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "memory_export_markdown",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }
    let workspace = match params.get("workspace").and_then(Value::as_str) {
        Some(w) => w,
        None => return json!({"error":"workspace is required"}),
    };
    let default_dir = format!("./engram-export/{workspace}");
    let output_dir = params
        .get("output_dir")
        .and_then(Value::as_str)
        .unwrap_or(&default_dir);
    let output_path = match validate_export_dir(output_dir) {
        Ok(p) => p,
        Err(e) => return json!({"error":format!("Invalid output_dir: {e}")}),
    };
    let include_links = params
        .get("include_links")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let group = params
        .get("group")
        .and_then(Value::as_str)
        .unwrap_or("flat");
    if !matches!(group, "flat" | "day" | "workspace" | "type" | "entity") {
        return json!({"error":"Invalid export group"});
    }
    let (memories, related_map) = match query_export_snapshot(ctx, workspace, include_links) {
        Ok(result) => result,
        Err(e) => return json!({"error":format!("Failed to read export snapshot: {e}")}),
    };
    if memories.is_empty() {
        return json!({"error":format!("No memories found in workspace '{workspace}'"),"files_written":0});
    }
    let mut id_to_filename = HashMap::new();
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut paths = Vec::with_capacity(memories.len());
    for mem in &memories {
        let Some(id) = mem["id"].as_i64() else {
            return json!({"error":"Invalid exported memory id"});
        };
        let content = mem["content"].as_str().unwrap_or("");
        let filename = format!(
            "{id}-{}",
            sanitize_filename(content.lines().next().unwrap_or("untitled"))
        );
        let mut relative = PathBuf::new();
        if let Some(subdir) = compute_file_subdir(group, mem) {
            if validate_component(&subdir).is_err() {
                return json!({"error":"Unsafe export grouping component"});
            }
            relative.push(subdir);
        }
        relative.push(format!("{filename}.md"));
        paths.push(relative);
        id_to_filename.insert(id, filename);
        *type_counts
            .entry(mem["memory_type"].as_str().unwrap_or("note").to_string())
            .or_default() += 1;
    }
    // Validate/read/render the complete batch before any directory or file is created.
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
    if let Err(e) = write_batch(&output_path, &batch, &ctx.storage) {
        return json!({"error":format!("Failed to write export (batch may be incomplete): {e}")});
    }
    json!({
        "files_written":batch.len(),"output_dir":output_path.to_string_lossy(),
        "index_path":output_path.join("index.md").to_string_lossy(),
        "memories_exported":memories.len(),"type_breakdown":type_counts,"group":group
    })
}
