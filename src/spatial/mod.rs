//! Spatial memory management & Mnemonic Palace visualizer (Method of Loci).

pub mod visualizer;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path};

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

    let base = match std::env::var("ENGRAM_EXPORT_BASE_DIR") {
        Ok(base_str) if !base_str.trim().is_empty() => Some(
            fs::canonicalize(&base_str)
                .map_err(|e| format!("ENGRAM_EXPORT_BASE_DIR cannot be resolved: {}", e))?,
        ),
        _ => None,
    };
    let outside_base = || {
        format!(
            "output path '{}' is outside the allowed export base directory",
            path_str
        )
    };

    // Decide containment before touching the filesystem: a path outside the
    // export base must not create directories on its way to being refused.
    if let Some(base) = &base {
        if p.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err("output path must not contain '..' components".to_string());
        }
        let existing = parent
            .ancestors()
            .find(|ancestor| ancestor.exists())
            .unwrap_or_else(|| Path::new("."));
        let existing = if existing.as_os_str().is_empty() {
            Path::new(".")
        } else {
            existing
        };
        let canon_existing = fs::canonicalize(existing)
            .map_err(|e| format!("cannot resolve parent directory: {}", e))?;
        if !canon_existing.starts_with(base) {
            return Err(outside_base());
        }
    }

    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create output directory: {}", e))?;
    }

    let canon_parent =
        fs::canonicalize(parent).map_err(|e| format!("cannot resolve parent directory: {}", e))?;
    let file_name = p
        .file_name()
        .ok_or_else(|| "path has no file name component".to_string())?;
    let canonical = canon_parent.join(file_name);

    if let Some(base) = &base {
        if !canonical.starts_with(base) {
            return Err(outside_base());
        }
    }
    // `fs::write` follows a symlink in the final component, which would leave
    // the directory that was just checked.
    if fs::symlink_metadata(&canonical).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err("output path must not be a symbolic link".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs in a child process so `ENGRAM_EXPORT_BASE_DIR` never leaks into
    /// sibling tests that share this process environment.
    #[test]
    fn output_outside_export_base_is_refused_without_creating_directories() {
        const CHILD: &str = "ENGRAM_SPATIAL_OUTPUT_PATH_CHILD";
        if let Ok(root) = std::env::var(CHILD) {
            let root = Path::new(&root);
            let stray = root.join("outside/nested/palace.html");
            let error = validate_output_file_path(stray.to_str().unwrap()).unwrap_err();
            assert!(error.contains("outside the allowed export base"), "{error}");
            assert!(
                !root.join("outside").exists(),
                "refusal must not create directories"
            );

            let escape = root.join("base/../outside/palace.html");
            assert!(validate_output_file_path(escape.to_str().unwrap()).is_err());
            assert!(!root.join("outside").exists());

            let inside = root.join("base/new/palace.html");
            let resolved = validate_output_file_path(inside.to_str().unwrap()).unwrap();
            assert!(resolved.ends_with("base/new/palace.html"));

            #[cfg(unix)]
            {
                let link = root.join("base/link.html");
                std::os::unix::fs::symlink(root.join("target.html"), &link).unwrap();
                let error = validate_output_file_path(link.to_str().unwrap()).unwrap_err();
                assert!(error.contains("symbolic link"), "{error}");
            }
            return;
        }

        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("base")).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "spatial::tests::output_outside_export_base_is_refused_without_creating_directories",
                "--nocapture",
            ])
            .env(CHILD, root.path())
            .env("ENGRAM_EXPORT_BASE_DIR", root.path().join("base"))
            .status()
            .unwrap();
        assert!(status.success());
    }
}
