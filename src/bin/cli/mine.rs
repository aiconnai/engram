//! High-throughput CLI mining engine.
//! Ingests Claude Code JSONL transcripts, markdown files, and text documents into verbatim memory.
//! Supports real-time auto-mining daemon mode (`--watch`) via filesystem event notifications.

use std::collections::HashMap;
use std::fs;
#[cfg(feature = "watcher")]
use std::fs::File;
#[cfg(feature = "watcher")]
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
#[cfg(feature = "watcher")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "watcher")]
use std::sync::Arc;
#[cfg(feature = "watcher")]
use std::time::Duration;
use std::time::Instant;

use engram::error::{EngramError, Result};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, DedupMode, MemoryTier, MemoryType};

/// Configuration options for the mining engine.
#[derive(Debug, Clone)]
pub struct MineOptions<'a> {
    pub path_str: &'a str,
    pub mode: &'a str,
    pub wing: Option<String>,
    pub room: Option<String>,
    pub workspace: &'a str,
    pub watch: bool,
    pub debounce_ms: u64,
}

struct SpatialTarget<'a> {
    default_wing: String,
    default_room: String,
    scope_path: String,
    workspace: &'a str,
    #[allow(dead_code)]
    mode: &'a str,
}

pub fn handle_mine(storage: &Storage, opts: MineOptions) -> Result<()> {
    let start = Instant::now();
    let expanded = shellexpand::tilde(opts.path_str).to_string();
    let path = Path::new(&expanded);

    if !path.exists() {
        return Err(EngramError::InvalidInput(format!(
            "Path does not exist: {}",
            expanded
        )));
    }

    let default_wing = opts.wing.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("general")
            .to_string()
    });
    let default_room = opts.room.unwrap_or_else(|| "general".to_string());
    let scope_path = format!("wing:{}/room:{}", default_wing, default_room);

    let target = SpatialTarget {
        default_wing,
        default_room,
        scope_path,
        workspace: opts.workspace,
        mode: opts.mode,
    };

    let mut files_to_process = Vec::new();
    if path.is_file() {
        files_to_process.push(path.to_path_buf());
    } else if path.is_dir() {
        collect_files(path, opts.mode, &mut files_to_process)?;
    }

    let mut file_offsets: HashMap<PathBuf, u64> = HashMap::new();

    if !files_to_process.is_empty() {
        println!(
            "⛏️  Mining {} file(s) in '{}' mode into [Palace: {}, Wing: {}, Room: {}]...",
            files_to_process.len(),
            opts.mode,
            target.workspace,
            target.default_wing,
            target.default_room
        );

        let mut total_created = 0;
        let mut total_bytes = 0;

        storage.with_transaction(|conn| {
            for file in &files_to_process {
                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let len = content.len() as u64;
                file_offsets.insert(file.clone(), len);
                total_bytes += content.len();

                let chunks = extract_chunks(&content, opts.mode, file);
                for chunk in chunks {
                    if chunk.trim().is_empty() {
                        continue;
                    }

                    let mut tags = vec![
                        format!("wing:{}", target.default_wing),
                        format!("room:{}", target.default_room),
                        format!("source:{}", opts.mode),
                    ];
                    if let Some(fname) = file.file_name().and_then(|f| f.to_str()) {
                        tags.push(format!("file:{}", fname));
                    }

                    let input = CreateMemoryInput {
                        content: chunk,
                        memory_type: MemoryType::Verbatim,
                        tags,
                        workspace: Some(target.workspace.to_string()),
                        tier: MemoryTier::Permanent,
                        dedup_mode: DedupMode::Skip,
                        ..Default::default()
                    };

                    let memory = create_memory(conn, &input)?;
                    conn.execute(
                        "UPDATE memories SET scope_path = ? WHERE id = ?",
                        rusqlite::params![target.scope_path, memory.id],
                    )?;
                    total_created += 1;
                }
            }
            Ok(())
        })?;

        let elapsed = start.elapsed();
        println!(
            "✅ Initial baseline: mined {} drawers ({:.2} KB) in {:.2}ms (avg {:.2} µs/record)",
            total_created,
            total_bytes as f64 / 1024.0,
            elapsed.as_secs_f64() * 1000.0,
            if total_created > 0 {
                (elapsed.as_micros() as f64) / (total_created as f64)
            } else {
                0.0
            }
        );
    } else {
        println!(
            "No existing files found in: {}. Ready for new files.",
            expanded
        );
    }

    if !opts.watch {
        return Ok(());
    }

    run_watcher_loop(storage, path, &target, file_offsets, opts.debounce_ms)
}

#[cfg(feature = "watcher")]
fn run_watcher_loop(
    storage: &Storage,
    path: &Path,
    target: &SpatialTarget,
    mut file_offsets: HashMap<PathBuf, u64>,
    debounce_ms: u64,
) -> Result<()> {
    use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    println!("\n👀 Auto-mining daemon active in real-time mode.");
    println!("📁 Monitoring: {}", path.display());
    println!(
        "🏛️ Palace: {} | Wing: {} | Room: {}",
        target.workspace, target.default_wing, target.default_room
    );
    println!("Press Ctrl+C to stop.\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Catch SIGINT / Ctrl+C
    ctrlc_handler(r);

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .map_err(|e| EngramError::Storage(format!("Failed to initialize file watcher: {}", e)))?;

    let watch_mode = if path.is_dir() {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    watcher
        .watch(path, watch_mode)
        .map_err(|e| EngramError::Storage(format!("Failed to watch path: {}", e)))?;

    let mut last_processed = Instant::now();
    let debounce_duration = Duration::from_millis(debounce_ms);

    while running.load(Ordering::SeqCst) {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(200)) {
            if last_processed.elapsed() < debounce_duration {
                continue;
            }

            for event_path in event.paths {
                if !is_matching_file(&event_path, target.mode) {
                    continue;
                }

                std::thread::sleep(Duration::from_millis(50)); // Small settle time

                let newly_mined =
                    process_file_update(storage, &event_path, target, &mut file_offsets);

                if let Ok(count) = newly_mined {
                    if count > 0 {
                        let now = chrono::Local::now().format("%H:%M:%S");
                        let fname = event_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("file");
                        println!(
                            "[{}] ⚡ Auto-mined {} new drawer(s) from '{}' -> [Wing: {}, Room: {}]",
                            now, count, fname, target.default_wing, target.default_room
                        );
                    }
                }
            }
            last_processed = Instant::now();
        }
    }

    println!("\n🛑 Auto-mining daemon stopped cleanly.");
    Ok(())
}

#[cfg(not(feature = "watcher"))]
fn run_watcher_loop(
    _storage: &Storage,
    _path: &Path,
    _target: &SpatialTarget,
    _file_offsets: HashMap<PathBuf, u64>,
    _debounce_ms: u64,
) -> Result<()> {
    Err(EngramError::InvalidInput(
        "Watch mode requires the 'watcher' feature. Rebuild with: cargo build --features watcher"
            .to_string(),
    ))
}

#[cfg(feature = "watcher")]
fn process_file_update(
    storage: &Storage,
    file_path: &Path,
    target: &SpatialTarget,
    file_offsets: &mut HashMap<PathBuf, u64>,
) -> Result<usize> {
    if !file_path.exists() || !file_path.is_file() {
        return Ok(0);
    }

    let current_size = match fs::metadata(file_path) {
        Ok(m) => m.len(),
        Err(_) => return Ok(0),
    };

    let prev_offset = *file_offsets.get(file_path).unwrap_or(&0);

    // If file was truncated or recreated, reset to 0
    let offset = if current_size < prev_offset {
        0
    } else {
        prev_offset
    };

    if current_size == offset {
        return Ok(0);
    }

    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Ok(0),
    };

    if offset > 0 && file.seek(SeekFrom::Start(offset)).is_err() {
        return Ok(0);
    }

    let mut new_bytes = Vec::new();
    if file.read_to_end(&mut new_bytes).is_err() {
        return Ok(0);
    }

    let new_content = String::from_utf8_lossy(&new_bytes);
    if new_content.trim().is_empty() {
        file_offsets.insert(file_path.to_path_buf(), current_size);
        return Ok(0);
    }

    let chunks = extract_chunks(&new_content, target.mode, file_path);
    if chunks.is_empty() {
        file_offsets.insert(file_path.to_path_buf(), current_size);
        return Ok(0);
    }

    let mut created_count = 0;
    let fname = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("");

    storage.with_transaction(|conn| {
        for chunk in chunks {
            if chunk.trim().is_empty() {
                continue;
            }

            let tags = vec![
                format!("wing:{}", target.default_wing),
                format!("room:{}", target.default_room),
                format!("source:{}", target.mode),
                format!("file:{}", fname),
            ];

            let input = CreateMemoryInput {
                content: chunk,
                memory_type: MemoryType::Verbatim,
                tags,
                workspace: Some(target.workspace.to_string()),
                tier: MemoryTier::Permanent,
                dedup_mode: DedupMode::Skip,
                ..Default::default()
            };

            let memory = create_memory(conn, &input)?;
            conn.execute(
                "UPDATE memories SET scope_path = ? WHERE id = ?",
                rusqlite::params![target.scope_path, memory.id],
            )?;
            created_count += 1;
        }
        Ok(())
    })?;

    file_offsets.insert(file_path.to_path_buf(), current_size);
    Ok(created_count)
}

fn is_matching_file(path: &Path, mode: &str) -> bool {
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if name.starts_with('.') || name.ends_with('~') || name.ends_with(".tmp") {
            return false;
        }
    }

    match mode {
        "convos" => path
            .extension()
            .map(|ext| ext == "jsonl" || ext == "json" || ext == "log" || ext == "txt")
            .unwrap_or(false),
        "markdown" => path.extension().map(|ext| ext == "md").unwrap_or(false),
        _ => true,
    }
}

fn collect_files(dir: &Path, mode: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }
                collect_files(&path, mode, files)?;
            } else if path.is_file() && is_matching_file(&path, mode) {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
    match mode {
        "convos" => {
            let mut chunks = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
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

#[cfg(feature = "watcher")]
fn ctrlc_handler(running: Arc<AtomicBool>) {
    ctrlc_helper(move || {
        running.store(false, Ordering::SeqCst);
    });
}

#[cfg(feature = "watcher")]
fn ctrlc_helper<F: FnOnce() + Send + 'static>(f: F) {
    let cell = std::sync::Mutex::new(Some(f));
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            if let Ok(mut lock) = cell.lock() {
                if let Some(handler) = lock.take() {
                    handler();
                }
            }
        }
    });
}
