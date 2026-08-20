//! High-throughput CLI mining engine.
//! Ingests Claude Code JSONL transcripts, markdown files, and text documents into verbatim memory.

use std::fs;
use std::path::Path;
use std::time::Instant;

use engram::error::{EngramError, Result};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, MemoryTier, MemoryType};

pub fn handle_mine(
    storage: &Storage,
    path_str: &str,
    mode: &str,
    wing: Option<String>,
    room: Option<String>,
    workspace: &str,
) -> Result<()> {
    let start = Instant::now();
    let expanded = shellexpand::tilde(path_str).to_string();
    let path = Path::new(&expanded);

    if !path.exists() {
        return Err(EngramError::InvalidInput(format!(
            "Path does not exist: {}",
            expanded
        )));
    }

    let default_wing = wing.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("general")
            .to_string()
    });
    let default_room = room.unwrap_or_else(|| "general".to_string());
    let scope_path = format!("wing:{}/room:{}", default_wing, default_room);

    let mut files_to_process = Vec::new();
    if path.is_file() {
        files_to_process.push(path.to_path_buf());
    } else if path.is_dir() {
        collect_files(path, mode, &mut files_to_process)?;
    }

    if files_to_process.is_empty() {
        println!("No matching files found to mine in: {}", expanded);
        return Ok(());
    }

    println!(
        "⛏️  Mining {} file(s) in '{}' mode into [Palace: {}, Wing: {}, Room: {}]...",
        files_to_process.len(),
        mode,
        workspace,
        default_wing,
        default_room
    );

    let mut total_created = 0;
    let mut total_bytes = 0;

    storage.with_transaction(|conn| {
        for file in &files_to_process {
            let content = match fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            total_bytes += content.len();

            let chunks = extract_chunks(&content, mode, file);
            for chunk in chunks {
                if chunk.trim().is_empty() {
                    continue;
                }

                let mut tags = vec![
                    format!("wing:{}", default_wing),
                    format!("room:{}", default_room),
                    format!("source:{}", mode),
                ];
                if let Some(fname) = file.file_name().and_then(|f| f.to_str()) {
                    tags.push(format!("file:{}", fname));
                }

                let input = CreateMemoryInput {
                    content: chunk,
                    memory_type: MemoryType::Verbatim,
                    tags,
                    workspace: Some(workspace.to_string()),
                    tier: MemoryTier::Permanent,
                    ..Default::default()
                };

                let memory = create_memory(conn, &input)?;
                // Update scope_path in SQLite
                conn.execute(
                    "UPDATE memories SET scope_path = ? WHERE id = ?",
                    rusqlite::params![scope_path, memory.id],
                )?;
                total_created += 1;
            }
        }
        Ok(())
    })?;

    let elapsed = start.elapsed();
    println!(
        "✅ Mined {} drawers ({:.2} KB) in {:.2}ms (avg {:.2} µs/record)",
        total_created,
        total_bytes as f64 / 1024.0,
        elapsed.as_secs_f64() * 1000.0,
        if total_created > 0 {
            (elapsed.as_micros() as f64) / (total_created as f64)
        } else {
            0.0
        }
    );

    Ok(())
}

fn collect_files(dir: &Path, mode: &str, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Ignore hidden directories and common ignore folders
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }
                collect_files(&path, mode, files)?;
            } else if path.is_file() {
                let match_file = match mode {
                    "convos" => path
                        .extension()
                        .map(|ext| ext == "jsonl" || ext == "json" || ext == "log" || ext == "txt")
                        .unwrap_or(false),
                    "markdown" => path.extension().map(|ext| ext == "md").unwrap_or(false),
                    _ => true,
                };
                if match_file {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
    match mode {
        "convos" => {
            // Process JSONL or lines
            let mut chunks = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    // Check standard JSONL fields (role + content)
                    if let Some(text) = val.get("content").and_then(|c| c.as_str()) {
                        let role = val
                            .get("role")
                            .and_then(|r| r.as_str())
                            .unwrap_or("speaker");
                        chunks.push(format!("[{}] {}", role, text));
                    } else if let Some(text) = val.get("message").and_then(|m| m.as_str()) {
                        chunks.push(text.to_string());
                    } else if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
                        chunks.push(text.to_string());
                    } else {
                        chunks.push(trimmed.to_string());
                    }
                } else {
                    chunks.push(trimmed.to_string());
                }
            }
            chunks
        }
        "markdown" => {
            // Split by markdown sections (## or paragraphs)
            let mut chunks = Vec::new();
            let mut current_chunk = String::new();
            let fname = file.file_name().and_then(|f| f.to_str()).unwrap_or("");

            for line in content.lines() {
                if (line.starts_with("# ") || line.starts_with("## "))
                    && !current_chunk.trim().is_empty()
                {
                    chunks.push(format!("### File: {}\n{}", fname, current_chunk.trim()));
                    current_chunk.clear();
                }
                current_chunk.push_str(line);
                current_chunk.push('\n');
            }
            if !current_chunk.trim().is_empty() {
                chunks.push(format!("### File: {}\n{}", fname, current_chunk.trim()));
            }
            chunks
        }
        _ => {
            // Default 800-char window chunking with overlap
            let mut chunks = Vec::new();
            let chars: Vec<char> = content.chars().collect();
            let chunk_size = 800;
            let step = 700;

            let mut start = 0;
            while start < chars.len() {
                let end = std::cmp::min(start + chunk_size, chars.len());
                let chunk: String = chars[start..end].iter().collect();
                chunks.push(chunk);
                if end == chars.len() {
                    break;
                }
                start += step;
            }
            chunks
        }
    }
}
