//! Spatial memory management & Mnemonic Palace visualizer (Method of Loci).

pub mod visualizer;

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use visualizer::{PalaceDrawer, PalaceFormat, PalaceGraph, PalaceRoom, PalaceWing};

use crate::error::{EngramError, Result};
use crate::storage::Storage;

static PALACE_TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

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

fn ensure_safe_destination(path: &Path) -> std::result::Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!("cannot inspect output path: {}", err)),
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err("output path must not be a symbolic link".to_string());
            }
            if !meta.file_type().is_file() {
                return Err("output path must refer to a regular file".to_string());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if meta.nlink() > 1 {
                    return Err("output path must not have multiple hard links".to_string());
                }
            }
            Ok(())
        }
    }
}

fn validate_output_file_path(
    storage: &Storage,
    path_str: &str,
) -> std::result::Result<PathBuf, String> {
    if path_str.trim().is_empty() {
        return Err("output path must not be empty".to_string());
    }
    if path_str.contains('\0') {
        return Err("output path must not contain null bytes".to_string());
    }

    let p = Path::new(path_str);
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("output path must not contain '..' components".to_string());
    }
    storage
        .refuse_active_sqlite_artifact(p)
        .map_err(|e| e.to_string())?;

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
    storage
        .refuse_active_sqlite_artifact(&canonical)
        .map_err(|e| e.to_string())?;
    ensure_safe_destination(&canonical)?;

    Ok(canonical)
}

fn write_visualizer_atomically(storage: &Storage, path: &Path, rendered: &str) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        EngramError::InvalidInput("output path has no parent directory".to_string())
    })?;
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut created = None;
    for attempt in 0..16_u32 {
        let seq = PALACE_TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".palace-tmp-{pid}-{nanos}-{seq}-{attempt}"));
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&candidate) {
            Ok(file) => {
                created = Some((candidate, file));
                break;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(EngramError::InvalidInput(format!(
                    "Failed to create temporary visualizer file in {}: {}",
                    parent.display(),
                    err
                )));
            }
        }
    }

    let (temp_path, mut temp_file) = created.ok_or_else(|| {
        EngramError::InvalidInput(format!(
            "Failed to allocate unique temporary visualizer file in {}",
            parent.display()
        ))
    })?;

    let write_outcome: Result<()> = (|| {
        temp_file.write_all(rendered.as_bytes()).map_err(|e| {
            EngramError::InvalidInput(format!(
                "Failed to write visualizer output to {}: {}",
                path.display(),
                e
            ))
        })?;
        temp_file.sync_all().map_err(|e| {
            EngramError::InvalidInput(format!(
                "Failed to sync visualizer output for {}: {}",
                path.display(),
                e
            ))
        })?;
        drop(temp_file);

        storage.refuse_active_sqlite_artifact(path)?;
        ensure_safe_destination(path).map_err(EngramError::InvalidInput)?;

        fs::rename(&temp_path, path).map_err(|e| {
            EngramError::InvalidInput(format!(
                "Failed to finalize visualizer output at {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(())
    })();

    if write_outcome.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    write_outcome
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
        let path =
            validate_output_file_path(storage, path_str).map_err(EngramError::InvalidInput)?;
        write_visualizer_atomically(storage, &path, &rendered)?;
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
            let storage = Storage::open_in_memory().unwrap();
            let root = Path::new(&root);
            let stray = root.join("outside/nested/palace.html");
            let error = validate_output_file_path(&storage, stray.to_str().unwrap()).unwrap_err();
            assert!(error.contains("outside the allowed export base"), "{error}");
            assert!(
                !root.join("outside").exists(),
                "refusal must not create directories"
            );

            let escape = root.join("base/../outside/palace.html");
            assert!(validate_output_file_path(&storage, escape.to_str().unwrap()).is_err());
            assert!(!root.join("outside").exists());

            let inside = root.join("base/new/palace.html");
            let resolved = validate_output_file_path(&storage, inside.to_str().unwrap()).unwrap();
            assert!(resolved.ends_with("base/new/palace.html"));

            #[cfg(unix)]
            {
                let link = root.join("base/link.html");
                std::os::unix::fs::symlink(root.join("target.html"), &link).unwrap();
                let error =
                    validate_output_file_path(&storage, link.to_str().unwrap()).unwrap_err();
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

    #[test]
    fn unconfined_dotdot_active_db_and_hardlinks_are_refused_without_side_effects() {
        const CHILD: &str = "ENGRAM_SPATIAL_UNCONFINED_GUARD_CHILD";
        if let Ok(root) = std::env::var(CHILD) {
            let root = Path::new(&root);
            let db_dir = root.join("db");
            fs::create_dir_all(&db_dir).unwrap();
            let db_file = db_dir.join("engram.db");
            let storage = Storage::open(crate::types::StorageConfig {
                db_path: db_file.to_string_lossy().into_owned(),
                storage_mode: crate::types::StorageMode::Local,
                cloud_uri: None,
                encrypt_cloud: false,
                confidence_half_life_days: 30.0,
                auto_sync: false,
                sync_debounce_ms: 5000,
            })
            .unwrap();

            // 1. `..` traversal through a missing directory must be refused without creating `nonexistent`
            //    or touching the live SQLite database when ENGRAM_EXPORT_BASE_DIR is unset.
            let dotdot_db = db_dir.join("nonexistent/../engram.db");
            let err = generate_palace_visualizer(
                &storage,
                "default",
                None,
                PalaceFormat::Html,
                Some(dotdot_db.to_str().unwrap()),
            )
            .unwrap_err();
            assert!(err.to_string().contains("'..'"), "{err}");
            assert!(
                !db_dir.join("nonexistent").exists(),
                "must not create intermediate directory before refusing '..'"
            );

            // 2. Post-canonicalize active SQLite artifact check catches symlinked parent dirs.
            #[cfg(unix)]
            {
                let alias_dir = root.join("alias_db_dir");
                std::os::unix::fs::symlink(&db_dir, &alias_dir).unwrap();
                let alias_db = alias_dir.join("engram.db");
                let err = generate_palace_visualizer(
                    &storage,
                    "default",
                    None,
                    PalaceFormat::Html,
                    Some(alias_db.to_str().unwrap()),
                )
                .unwrap_err();
                assert!(
                    err.to_string().to_lowercase().contains("sqlite")
                        || err.to_string().to_lowercase().contains("database"),
                    "expected active sqlite refusal, got: {err}"
                );

                // 3. Hard-linked regular files must be refused so sibling files cannot be clobbered.
                let victim = root.join("victim.txt");
                fs::write(&victim, b"preserve-me").unwrap();
                let hardlink_out = root.join("hardlink.html");
                fs::hard_link(&victim, &hardlink_out).unwrap();
                let err = generate_palace_visualizer(
                    &storage,
                    "default",
                    None,
                    PalaceFormat::Html,
                    Some(hardlink_out.to_str().unwrap()),
                )
                .unwrap_err();
                assert!(err.to_string().contains("hard links"), "{err}");
                assert_eq!(fs::read_to_string(&victim).unwrap(), "preserve-me");
            }

            // 4. Valid atomic export succeeds and leaves no `.palace-tmp-*` files behind.
            let clean_out = root.join("exports/palace.json");
            let out = generate_palace_visualizer(
                &storage,
                "default",
                None,
                PalaceFormat::Json,
                Some(clean_out.to_str().unwrap()),
            )
            .unwrap();
            assert!(out.output_path.is_some());
            assert!(clean_out.exists());
            let entries: Vec<_> = fs::read_dir(root.join("exports"))
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
                .collect();
            assert_eq!(entries, vec!["palace.json".to_string()]);
            return;
        }

        let root = tempfile::tempdir().unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "spatial::tests::unconfined_dotdot_active_db_and_hardlinks_are_refused_without_side_effects",
                "--nocapture",
            ])
            .env(CHILD, root.path())
            .env_remove("ENGRAM_EXPORT_BASE_DIR")
            .status()
            .unwrap();
        assert!(status.success());
    }
}
