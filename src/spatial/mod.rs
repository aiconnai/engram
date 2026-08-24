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

/// Generate or export a memory palace visualization for a workspace.
pub fn generate_palace_visualizer(
    storage: &Storage,
    workspace: &str,
    target_wing: Option<&str>,
    format: PalaceFormat,
    output_path: Option<&str>,
) -> Result<PalaceVisualizerOutput> {
    let graph = PalaceGraph::extract(storage, workspace, target_wing)?;
    let rendered = graph.render(format);

    let saved_path = if let Some(path_str) = output_path {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    EngramError::InvalidInput(format!("Failed to create output directory: {}", e))
                })?;
            }
        }
        fs::write(path, &rendered).map_err(|e| {
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
        workspace: workspace.to_string(),
        format,
        wings_count: graph.wings_count,
        rooms_count: graph.rooms_count,
        total_drawers: graph.total_drawers,
        rendered,
        output_path: saved_path,
    })
}
