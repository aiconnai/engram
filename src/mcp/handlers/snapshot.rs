//! Snapshot MCP tool handlers for agent-portability.
//!
//! Provides three tools for creating, loading, and inspecting .egm snapshot
//! archives — portable knowledge packages for distributing AI memory between
//! agents and deployments.

use std::path::Path;

use serde_json::{json, Value};

use super::HandlerContext;

// ── Hex key parsing helper ────────────────────────────────────────────────────

/// Parse a hex-encoded 32-byte key string into a fixed-size array.
fn parse_hex_key(hex_str: &str) -> std::result::Result<[u8; 32], String> {
    if hex_str.len() != 64 {
        return Err(format!(
            "Key must be 64 hex characters (32 bytes), got {}",
            hex_str.len()
        ));
    }
    let bytes: Vec<u8> = (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16))
        .collect::<std::result::Result<Vec<u8>, _>>()
        .map_err(|e| format!("Invalid hex: {}", e))?;
    if bytes.len() != 32 {
        return Err(format!("Key must be 32 bytes, got {}", bytes.len()));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

// ── Path validation ───────────────────────────────────────────────────────────

/// Validate a snapshot path against traversal and boundary constraints.
///
/// For new files (output paths), the parent directory is canonicalized.
/// For existing files (input paths), the path itself is canonicalized.
/// If `ENGRAM_SNAPSHOTS_DIR` is set, the resolved path must be inside it.
pub(crate) fn validate_snapshot_path(path: &str) -> Result<std::path::PathBuf, String> {
    if path.is_empty() {
        return Err("path must not be empty".to_string());
    }
    if path.contains('\0') {
        return Err("path must not contain null bytes".to_string());
    }

    let p = std::path::Path::new(path);

    // Canonicalize: use the path itself if it exists, otherwise the parent.
    let canonical = if p.exists() {
        std::fs::canonicalize(p).map_err(|e| format!("cannot resolve path: {}", e))?
    } else {
        let parent = p.parent().unwrap_or_else(|| std::path::Path::new("."));
        let canon_parent = std::fs::canonicalize(parent)
            .map_err(|e| format!("cannot resolve parent directory: {}", e))?;
        canon_parent.join(p.file_name().ok_or("path has no file name")?)
    };

    // If ENGRAM_SNAPSHOTS_DIR is set, enforce boundary.
    if let Ok(base_str) = std::env::var("ENGRAM_SNAPSHOTS_DIR") {
        if !base_str.is_empty() {
            let base = std::fs::canonicalize(&base_str)
                .map_err(|e| format!("ENGRAM_SNAPSHOTS_DIR cannot be resolved: {}", e))?;
            if !canonical.starts_with(&base) {
                return Err(format!(
                    "path '{}' is outside the allowed snapshots directory",
                    path
                ));
            }
        }
    }

    Ok(canonical)
}

// ── snapshot_create ───────────────────────────────────────────────────────────

/// Create a .egm snapshot archive from the current storage.
///
/// Accepts workspace/tag/importance/type filters and optional encryption or
/// signing. Returns the snapshot manifest as JSON.
pub fn snapshot_create(ctx: &HandlerContext, params: Value) -> Value {
    use crate::snapshot::SnapshotBuilder;

    let output_path = match params.get("output_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "output_path is required"}),
    };

    let validated_output_path = match validate_snapshot_path(&output_path) {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Invalid output_path: {}", e)}),
    };

    // Build the snapshot builder with optional filters
    let mut builder = SnapshotBuilder::new(ctx.storage.clone());

    if let Some(ws) = params.get("workspace").and_then(|v| v.as_str()) {
        builder = builder.workspace(ws);
    }

    if let Some(tags_arr) = params.get("tags").and_then(|v| v.as_array()) {
        let tags: Vec<String> = tags_arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if !tags.is_empty() {
            builder = builder.tags(tags);
        }
    }

    if let Some(min_imp) = params.get("importance_min").and_then(|v| v.as_f64()) {
        builder = builder.importance_min(min_imp as f32);
    }

    if let Some(types_arr) = params.get("memory_types").and_then(|v| v.as_array()) {
        let types: Vec<String> = types_arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if !types.is_empty() {
            builder = builder.memory_types(types);
        }
    }

    if let Some(desc) = params.get("description").and_then(|v| v.as_str()) {
        builder = builder.description(desc);
    }

    if let Some(creator) = params.get("creator").and_then(|v| v.as_str()) {
        builder = builder.creator(creator);
    }

    let path = validated_output_path.as_path();

    // Parse optional keys
    let encrypt_key_str = params.get("encrypt_key").and_then(|v| v.as_str());
    let sign_key_str = params.get("sign_key").and_then(|v| v.as_str());

    let manifest_result = if let Some(hex) = encrypt_key_str {
        match parse_hex_key(hex) {
            Ok(key) => builder.build_encrypted(path, &key),
            Err(e) => return json!({"error": format!("Invalid encrypt_key: {}", e)}),
        }
    } else if let Some(hex) = sign_key_str {
        match parse_hex_key(hex) {
            Ok(key) => builder.build_signed(path, &key),
            Err(e) => return json!({"error": format!("Invalid sign_key: {}", e)}),
        }
    } else {
        builder.build(path)
    };

    match manifest_result {
        Ok(manifest) => json!({
            "output_path": output_path,
            "format_version": manifest.format_version,
            "engram_version": manifest.engram_version,
            "schema_version": manifest.schema_version,
            "memory_count": manifest.memory_count,
            "entity_count": manifest.entity_count,
            "edge_count": manifest.edge_count,
            "encrypted": manifest.encrypted,
            "signed": manifest.signed,
            "created_at": manifest.created_at.to_rfc3339(),
            "content_hash": manifest.content_hash,
            "creator": manifest.creator,
            "description": manifest.description,
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── snapshot_load ─────────────────────────────────────────────────────────────

/// Load a .egm snapshot archive into storage.
///
/// Accepts a load strategy ("merge", "replace", "isolate", or "dry_run"),
/// an optional target workspace override, and an optional decryption key.
/// Returns a LoadResult describing what was inserted.
pub fn snapshot_load(ctx: &HandlerContext, params: Value) -> Value {
    use crate::snapshot::{LoadStrategy, SnapshotLoader};
    use std::str::FromStr;

    let path_str = match params.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "path is required"}),
    };

    let validated_path = match validate_snapshot_path(&path_str) {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Invalid path: {}", e)}),
    };

    let strategy_str = match params.get("strategy").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "strategy is required"}),
    };

    let strategy = match LoadStrategy::from_str(&strategy_str) {
        Ok(s) => s,
        Err(e) => return json!({"error": format!("Invalid strategy: {}", e)}),
    };

    let target_workspace = params
        .get("target_workspace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let decrypt_key_bytes: Option<[u8; 32]> =
        match params.get("decrypt_key").and_then(|v| v.as_str()) {
            Some(hex) => match parse_hex_key(hex) {
                Ok(key) => Some(key),
                Err(e) => return json!({"error": format!("Invalid decrypt_key: {}", e)}),
            },
            None => None,
        };

    let path = validated_path.as_path();
    let result = SnapshotLoader::load(
        &ctx.storage,
        path,
        strategy,
        target_workspace.as_deref(),
        decrypt_key_bytes.as_ref(),
    );

    match result {
        Ok(load_result) => {
            // Phase L: log attestation for the loaded snapshot manifest (best-effort).
            // Use the raw snapshot archive bytes as document content so the hash
            // matches what an external verifier would compute over the .egm file.
            {
                use crate::attestation::AttestationChain;
                let chain = AttestationChain::new(ctx.storage.clone());
                let snapshot_name = Path::new(&path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_str.clone());
                // Read the archive bytes for attestation (best-effort).
                if let Ok(archive_bytes) = std::fs::read(path) {
                    if let Err(e) =
                        chain.log_document(&archive_bytes, &snapshot_name, None, &[], None)
                    {
                        tracing::warn!(
                            "Attestation hook (snapshot_load): failed to log '{}': {}",
                            snapshot_name,
                            e
                        );
                    }
                }
            }
            json!({
                "strategy": load_result.strategy.to_string(),
                "memories_loaded": load_result.memories_loaded,
                "memories_skipped": load_result.memories_skipped,
                "entities_loaded": load_result.entities_loaded,
                "edges_loaded": load_result.edges_loaded,
                "target_workspace": load_result.target_workspace,
                "snapshot_origin": load_result.snapshot_origin,
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

// ── snapshot_inspect ──────────────────────────────────────────────────────────

/// Inspect a .egm snapshot archive and return metadata without loading.
///
/// Returns manifest information, file list, and file size.
pub fn snapshot_inspect(_ctx: &HandlerContext, params: Value) -> Value {
    use crate::snapshot::SnapshotLoader;

    let path_str = match params.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "path is required"}),
    };

    let validated_path = match validate_snapshot_path(&path_str) {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Invalid path: {}", e)}),
    };

    match SnapshotLoader::inspect(validated_path.as_path()) {
        Ok(info) => {
            let manifest = &info.manifest;
            json!({
                "file_size_bytes": info.file_size_bytes,
                "files": info.files,
                "manifest": {
                    "format_version": manifest.format_version,
                    "engram_version": manifest.engram_version,
                    "min_engram_version": manifest.min_engram_version,
                    "schema_version": manifest.schema_version,
                    "creator": manifest.creator,
                    "description": manifest.description,
                    "created_at": manifest.created_at.to_rfc3339(),
                    "content_hash": manifest.content_hash,
                    "memory_count": manifest.memory_count,
                    "entity_count": manifest.entity_count,
                    "edge_count": manifest.edge_count,
                    "embedding_model": manifest.embedding_model,
                    "embedding_dimensions": manifest.embedding_dimensions,
                    "encrypted": manifest.encrypted,
                    "signed": manifest.signed,
                }
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_snapshot_path;

    #[test]
    fn test_validate_snapshot_path_rejects_traversal_with_base_dir() {
        let tmp = std::env::temp_dir();
        std::env::set_var("ENGRAM_SNAPSHOTS_DIR", tmp.to_str().unwrap());
        let result = validate_snapshot_path("../../../etc/passwd");
        std::env::remove_var("ENGRAM_SNAPSHOTS_DIR");
        assert!(result.is_err(), "expected rejection for path traversal outside base dir");
        let msg = result.unwrap_err();
        assert!(msg.contains("outside"), "unexpected error message: {}", msg);
    }

    #[test]
    fn test_validate_snapshot_path_rejects_empty() {
        let result = validate_snapshot_path("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_validate_snapshot_path_rejects_null_bytes() {
        let result = validate_snapshot_path("foo\0bar.egm");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null"));
    }

    #[test]
    fn test_validate_snapshot_path_accepts_valid_relative() {
        std::env::remove_var("ENGRAM_SNAPSHOTS_DIR");
        let result = validate_snapshot_path("my_snapshot.egm");
        assert!(result.is_ok(), "expected ok, got {:?}", result);
    }

    #[test]
    fn test_validate_snapshot_path_enforces_base_dir_absolute() {
        let tmp = std::env::temp_dir();
        std::env::set_var("ENGRAM_SNAPSHOTS_DIR", tmp.to_str().unwrap());
        let result = validate_snapshot_path("/etc/passwd");
        std::env::remove_var("ENGRAM_SNAPSHOTS_DIR");
        assert!(result.is_err(), "expected rejection outside base dir");
    }

    #[test]
    fn test_validate_snapshot_path_allows_within_base_dir() {
        let tmp = std::env::temp_dir();
        std::env::set_var("ENGRAM_SNAPSHOTS_DIR", tmp.to_str().unwrap());
        let valid = tmp.join("test.egm").to_string_lossy().to_string();
        let result = validate_snapshot_path(&valid);
        std::env::remove_var("ENGRAM_SNAPSHOTS_DIR");
        assert!(result.is_ok(), "expected ok for path within base dir, got {:?}", result);
    }
}
