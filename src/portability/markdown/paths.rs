use std::path::PathBuf;

pub(crate) fn validate_export_dir(dir: &str) -> Result<PathBuf, String> {
    if dir.is_empty() {
        return Err("directory path must not be empty".to_string());
    }
    if dir.contains('\0') {
        return Err("directory path must not contain null bytes".to_string());
    }

    let p = std::path::Path::new(dir);

    let canonical = if p.exists() {
        std::fs::canonicalize(p).map_err(|e| format!("cannot resolve directory: {}", e))?
    } else {
        let parent = p.parent().unwrap_or_else(|| std::path::Path::new("."));
        let canon_parent = std::fs::canonicalize(parent)
            .map_err(|e| format!("cannot resolve parent directory: {}", e))?;
        canon_parent.join(p.file_name().ok_or("path has no final component")?)
    };

    // If ENGRAM_EXPORT_BASE_DIR is set, enforce boundary.
    if let Ok(base_str) = std::env::var("ENGRAM_EXPORT_BASE_DIR") {
        if !base_str.is_empty() {
            let base = std::fs::canonicalize(&base_str)
                .map_err(|e| format!("ENGRAM_EXPORT_BASE_DIR cannot be resolved: {}", e))?;
            if !canonical.starts_with(&base) {
                return Err(format!(
                    "directory '{}' is outside the allowed export base directory",
                    dir
                ));
            }
        }
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn export_base_dir_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_export_base_dir<T>(base_dir: Option<&str>, test: impl FnOnce() -> T) -> T {
        let _guard = export_base_dir_lock().lock().unwrap();
        match base_dir {
            Some(base_dir) => std::env::set_var("ENGRAM_EXPORT_BASE_DIR", base_dir),
            None => std::env::remove_var("ENGRAM_EXPORT_BASE_DIR"),
        }
        let result = test();
        std::env::remove_var("ENGRAM_EXPORT_BASE_DIR");
        result
    }

    #[test]
    fn test_validate_export_dir_rejects_traversal_with_base_dir() {
        let tmp = std::env::temp_dir();
        let result = with_export_base_dir(Some(tmp.to_str().unwrap()), || {
            validate_export_dir("../../../etc")
        });
        assert!(result.is_err(), "expected rejection for path traversal");
        let msg = result.unwrap_err();
        assert!(msg.contains("outside"), "unexpected message: {}", msg);
    }

    #[test]
    fn test_validate_export_dir_rejects_empty() {
        let result = validate_export_dir("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_validate_export_dir_rejects_null_bytes() {
        let result = validate_export_dir("some\0dir");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null"));
    }

    #[test]
    fn test_validate_export_dir_accepts_valid_path_no_base_dir() {
        let tmp = std::env::temp_dir();
        let result = with_export_base_dir(None, || validate_export_dir(tmp.to_str().unwrap()));
        assert!(result.is_ok(), "expected ok, got {:?}", result);
    }

    #[test]
    fn test_validate_export_dir_enforces_base_dir_absolute_path() {
        let tmp = std::env::temp_dir();
        let result =
            with_export_base_dir(Some(tmp.to_str().unwrap()), || validate_export_dir("/etc"));
        assert!(result.is_err(), "expected rejection outside base dir");
    }

    #[test]
    fn test_validate_export_dir_allows_within_base_dir() {
        let tmp = std::env::temp_dir();
        let valid = tmp.join("my-export").to_string_lossy().to_string();
        let result =
            with_export_base_dir(Some(tmp.to_str().unwrap()), || validate_export_dir(&valid));
        assert!(
            result.is_ok(),
            "expected ok within base dir, got {:?}",
            result
        );
    }
}
