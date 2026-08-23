use std::fs;
use std::path::PathBuf;

pub(super) fn collect_md_files(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut result = Vec::new();
    collect_md_files_inner(&PathBuf::from(dir), &mut result)?;
    Ok(result)
}

pub(super) fn collect_md_files_inner(
    dir: &PathBuf,
    out: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files_inner(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !file_name.starts_with('.')
                && !file_name.starts_with('_')
                && file_name != "index.md"
                && file_name != "readme.md"
            {
                out.push(path);
            }
        }
    }
    Ok(())
}
