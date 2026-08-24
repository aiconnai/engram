use std::collections::HashMap;
use std::fs;

use serde_json::{json, Value};

use super::format::{
    build_index_markdown, compute_file_subdir, format_memory_markdown, sanitize_filename,
};
use super::query::{build_related_map, query_workspace_memories};
use crate::mcp::handlers::markdown_export::validate_export_dir;
use crate::mcp::handlers::HandlerContext;

pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = match params.get("workspace").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return json!({"error": "workspace is required"}),
    };

    let default_dir = format!("./engram-export/{}", workspace);
    let output_dir = params
        .get("output_dir")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_dir);

    let output_path = match validate_export_dir(output_dir) {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Invalid output_dir: {}", e)}),
    };

    let include_links = params
        .get("include_links")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let group = params
        .get("group")
        .and_then(|v| v.as_str())
        .unwrap_or("flat")
        .to_string();

    // 1. Query all memories in workspace
    let memories = match query_workspace_memories(ctx, &workspace) {
        Ok(m) => m,
        Err(e) => return json!({"error": format!("Failed to query memories: {}", e)}),
    };

    if memories.is_empty() {
        return json!({
            "error": format!("No memories found in workspace '{}'", workspace),
            "files_written": 0
        });
    }

    // 2. If include_links, query cross-references for all memory IDs
    let related_map: HashMap<i64, Vec<(i64, String)>> = if include_links {
        let memory_ids: Vec<i64> = memories
            .iter()
            .filter_map(|m| m.get("id").and_then(|v| v.as_i64()))
            .collect();
        build_related_map(ctx, &memory_ids)
    } else {
        HashMap::new()
    };

    // 3. Create root output directory
    if let Err(e) = fs::create_dir_all(&output_path) {
        return json!({"error": format!("Failed to create output directory: {}", e)});
    }

    // First pass: compute filenames (stem only, no subdir)
    let id_to_filename: HashMap<i64, String> = memories
        .iter()
        .filter_map(|mem| {
            let id = mem.get("id").and_then(|v| v.as_i64())?;
            let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let title = content.lines().next().unwrap_or("untitled");
            let sanitized = sanitize_filename(title);
            Some((id, format!("{}-{}", id, sanitized)))
        })
        .collect();

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for mem in &memories {
        let mem_type = mem
            .get("memory_type")
            .and_then(|v| v.as_str())
            .unwrap_or("note");
        *type_counts.entry(mem_type.to_string()).or_insert(0) += 1;
    }

    // Second pass: write files
    let mut files_written: usize = 0;
    for mem in &memories {
        let id = mem.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

        // Determine target directory based on grouping mode
        let target_dir = match compute_file_subdir(&group, mem) {
            Some(subdir) => {
                let d = output_path.join(&subdir);
                if let Err(e) = fs::create_dir_all(&d) {
                    return json!({
                        "error": format!("Failed to create directory {}: {}", d.display(), e)
                    });
                }
                d
            }
            None => output_path.clone(),
        };

        let filename = id_to_filename
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("{}", id));
        let file_path = target_dir.join(format!("{}.md", filename));

        let md = format_memory_markdown(mem, include_links, &related_map, &id_to_filename);

        if let Err(e) = fs::write(&file_path, &md) {
            return json!({"error": format!("Failed to write {}: {}", file_path.display(), e)});
        }
        files_written += 1;
    }

    // 4. Write index.md in root
    let index_path = output_path.join("index.md");
    let index = build_index_markdown(&workspace, &memories, &type_counts, &id_to_filename, &group);

    if let Err(e) = fs::write(&index_path, &index) {
        return json!({"error": format!("Failed to write index: {}", e)});
    }

    json!({
        "files_written": files_written + 1,
        "output_dir": output_path.to_string_lossy(),
        "index_path": index_path.to_string_lossy(),
        "memories_exported": memories.len(),
        "type_breakdown": type_counts,
        "group": group
    })
}
