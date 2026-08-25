//! Spatial memory management & Mnemonic Palace visualizer (Method of Loci).

pub mod visualizer;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub use visualizer::{PalaceDrawer, PalaceFormat, PalaceGraph, PalaceRoom, PalaceWing};

use crate::error::{EngramError, Result};
use crate::storage::Storage;

/// Output of a palace visualization generation or export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceVisualizerOutput {
    pub workspace: String,
    pub format: PalaceFormat,
    pub wings_count: usize,
    pub rooms_count: usize,
    pub total_drawers: usize,
    pub rendered: String,
    pub output_path: Option<String>,
}

fn validate_output_file_path(path_str: &str) -> std::result::Result<std::path::PathBuf, String> {
    if path_str.trim().is_empty() {
        return Err("output path must not be empty".to_string());
    }
    if path_str.contains('\0') {
        return Err("output path must not contain null bytes".to_string());
    }

    let p = Path::new(path_str);
    let parent = p.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create output directory: {}", e))?;
    }

    let canon_parent =
        fs::canonicalize(parent).map_err(|e| format!("cannot resolve parent directory: {}", e))?;
    let file_name = p
        .file_name()
        .ok_or_else(|| "path has no file name component".to_string())?;
    let canonical = canon_parent.join(file_name);

    if let Ok(base_str) = std::env::var("ENGRAM_EXPORT_BASE_DIR") {
        if !base_str.is_empty() {
            let base = fs::canonicalize(&base_str)
                .map_err(|e| format!("ENGRAM_EXPORT_BASE_DIR cannot be resolved: {}", e))?;
            if !canonical.starts_with(&base) {
                return Err(format!(
                    "output path '{}' is outside the allowed export base directory",
                    path_str
                ));
            }
        }
    }

    Ok(canonical)
}

/// Generate or export a memory palace visualization for a workspace.
pub fn generate_palace_visualizer(
    storage: &Storage,
    workspace: &str,
    target_wing: Option<&str>,
    format: PalaceFormat,
    output_path: Option<&str>,
) -> Result<PalaceVisualizerOutput> {
    let ws = crate::types::normalize_workspace(workspace).unwrap_or_else(|_| workspace.to_string());
    let graph = PalaceGraph::extract(storage, &ws, target_wing)?;
    let rendered = graph.render(format);

    let saved_path = if let Some(path_str) = output_path {
        let path = validate_output_file_path(path_str).map_err(EngramError::InvalidInput)?;
        fs::write(&path, &rendered).map_err(|e| {
            EngramError::InvalidInput(format!(
                "Failed to write visualizer output to {}: {}",
                path.display(),
                e
            ))
        })?;
        Some(path.to_string_lossy().to_string())
    } else {
        None
    };

    Ok(PalaceVisualizerOutput {
        workspace: ws,
        format,
        wings_count: graph.wings_count,
        rooms_count: graph.rooms_count,
        total_drawers: graph.total_drawers,
        rendered,
        output_path: saved_path,
    })
}
