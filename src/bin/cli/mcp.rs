//! MCP client configuration and diagnostic management.
//!
//! Provides automatic configuration and status inspection for MCP client applications
//! including Claude Desktop, Claude Code, Cursor, Google Antigravity, and Windsurf.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Subcommand, ValueEnum};
use engram::error::{EngramError, Result};
use engram::storage::queries::get_stats;
use engram::storage::Storage;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ClientTarget {
    /// Claude Desktop and Claude Code CLI
    Claude,
    /// Cursor IDE (.cursor/mcp.json)
    Cursor,
    /// Google Antigravity IDE & agents
    Antigravity,
    /// Windsurf IDE
    Windsurf,
    /// Configure all detected client applications
    All,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum McpAction {
    /// Install and configure Engram MCP server into client application configurations
    Install {
        /// Target AI client application
        #[arg(short, long, default_value = "all")]
        client: ClientTarget,
        /// Server transport (stdio, http)
        #[arg(short, long, default_value = "stdio")]
        transport: String,
        /// Database path override
        #[arg(long, default_value = "~/.local/share/engram/memories.db")]
        db_path: String,
        /// Tool tier advertisement (essential, standard, all)
        #[arg(long, default_value = "standard")]
        tier: String,
        /// HTTP port (used if transport is http)
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Replace a client config that is not strict JSON (a backup is still written)
        #[arg(short, long)]
        force: bool,
    },
    /// Inspect MCP integration status, client configs, and server availability
    Status {
        /// Filter by specific client application
        #[arg(short, long, default_value = "all")]
        client: ClientTarget,
        /// Check HTTP endpoint on port (default 8080)
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Remove Engram MCP configuration from client application configurations
    Uninstall {
        /// Target AI client application to remove from
        #[arg(short, long, default_value = "all")]
        client: ClientTarget,
    },
}

struct ClientConfigSpec {
    client_name: &'static str,
    target: ClientTarget,
    paths: Vec<PathBuf>,
}

fn get_client_specs() -> Vec<ClientConfigSpec> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = Path::new(&home);

    let mut specs = Vec::new();

    // 1. Claude Desktop & Code
    let mut claude_paths = Vec::new();
    #[cfg(target_os = "macos")]
    {
        claude_paths.push(
            home_path
                .join("Library/Application Support/Claude")
                .join("claude_desktop_config.json"),
        );
    }
    #[cfg(target_os = "linux")]
    {
        claude_paths.push(
            home_path
                .join(".config/Claude")
                .join("claude_desktop_config.json"),
        );
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            claude_paths.push(
                Path::new(&appdata)
                    .join("Claude")
                    .join("claude_desktop_config.json"),
            );
        }
    }
    // Claude Code CLI paths
    claude_paths.push(home_path.join(".claude").join("mcp.json"));
    claude_paths.push(PathBuf::from(".claude/mcp.json"));

    specs.push(ClientConfigSpec {
        client_name: "Claude (Desktop & Code)",
        target: ClientTarget::Claude,
        paths: claude_paths,
    });

    // 2. Cursor IDE
    specs.push(ClientConfigSpec {
        client_name: "Cursor",
        target: ClientTarget::Cursor,
        paths: vec![
            PathBuf::from(".cursor/mcp.json"),
            home_path.join(".cursor").join("mcp.json"),
        ],
    });

    // 3. Google Antigravity
    specs.push(ClientConfigSpec {
        client_name: "Google Antigravity",
        target: ClientTarget::Antigravity,
        paths: vec![
            PathBuf::from(".gemini/mcp.json"),
            home_path.join(".gemini").join("mcp.json"),
        ],
    });

    // 4. Windsurf IDE
    specs.push(ClientConfigSpec {
        client_name: "Windsurf",
        target: ClientTarget::Windsurf,
        paths: vec![home_path.join(".codeium/windsurf").join("mcp_config.json")],
    });

    specs
}

pub(crate) fn handle(storage: &Storage, action: McpAction) -> Result<()> {
    match action {
        McpAction::Install {
            client,
            transport,
            db_path,
            tier,
            port,
            force,
        } => handle_install(client, &transport, &db_path, &tier, port, force),
        McpAction::Status { client, port } => handle_status(storage, client, port),
        McpAction::Uninstall { client } => handle_uninstall(client),
    }
}

fn handle_install(
    target: ClientTarget,
    transport: &str,
    db_path: &str,
    tier: &str,
    port: u16,
    force: bool,
) -> Result<()> {
    println!("=== Engram MCP Client Auto-Installer ===");
    println!("Transport:   {}", transport);
    println!("DB Path:     {}", db_path);
    println!("Tool Tier:   {}", tier);
    if transport == "http" {
        println!("HTTP Port:   {}", port);
    }
    println!("-----------------------------------------");

    let specs = get_client_specs();
    let mut installed_count = 0;
    let mut failures: Vec<String> = Vec::new();

    let server_entry = if transport == "http" {
        json!({
            "command": "engram-server",
            "args": ["--transport", "http", "--http-port", port.to_string()],
            "env": {
                "ENGRAM_DB_PATH": db_path,
                "ENGRAM_TOOL_TIER": tier
            }
        })
    } else {
        json!({
            "command": "engram-server",
            "args": ["--transport", "stdio"],
            "env": {
                "ENGRAM_DB_PATH": db_path,
                "ENGRAM_TOOL_TIER": tier
            }
        })
    };

    for spec in specs {
        if target != ClientTarget::All && spec.target != target {
            continue;
        }

        for path in &spec.paths {
            let is_project_local = path.starts_with(".");
            if is_project_local
                && target == ClientTarget::All
                && !path.parent().is_some_and(|p| p.exists())
            {
                continue;
            }

            match install_to_config_file(path, &server_entry, force) {
                Ok(status) => {
                    println!(
                        "  [✓] {} -> {} ({})",
                        spec.client_name,
                        path.display(),
                        status
                    );
                    installed_count += 1;
                }
                Err(err) => {
                    println!(
                        "  [!] {} -> {} (error: {})",
                        spec.client_name,
                        path.display(),
                        err
                    );
                    failures.push(format!("{}: {}", path.display(), err));
                }
            }
        }
    }

    println!("-----------------------------------------");
    if !failures.is_empty() {
        println!(
            "Configured Engram MCP in {} location(s); {} failed.",
            installed_count,
            failures.len()
        );
        return Err(EngramError::InvalidInput(format!(
            "could not update {} client config(s): {}",
            failures.len(),
            failures.join("; ")
        )));
    }
    println!(
        "✅ Configured Engram MCP in {} location(s).",
        installed_count
    );
    println!("Restart your AI client (Claude, Cursor, etc.) to activate 243+ memory tools.");
    Ok(())
}

/// Permission bits for a rewritten or backed-up client config: the existing
/// file's bits, or owner-only (0600) for a new file. Client configs can hold
/// other MCP servers' tokens in their `env` blocks.
#[cfg(unix)]
fn config_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o777)
        .unwrap_or(0o600)
}

#[cfg(not(unix))]
fn config_mode(_path: &Path) -> u32 {
    0o600
}

/// Exclusively create `path` with `mode` (the umask can only narrow it), then
/// set the exact bits while the file is still empty, so no byte is ever
/// readable under wider permissions than the config it copies.
#[cfg(unix)]
fn create_new_with_mode(path: &Path, mode: u32) -> std::io::Result<fs::File> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)?;
    if let Err(e) = file.set_permissions(fs::Permissions::from_mode(mode)) {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(e);
    }
    Ok(file)
}

#[cfg(not(unix))]
fn create_new_with_mode(path: &Path, _mode: u32) -> std::io::Result<fs::File> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

/// Set `path` to exactly `mode` (used to narrow a reused backup).
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

/// The file a write to `path` must replace. A symlinked config is resolved to
/// its target so the rename lands there and the link itself survives (a
/// dangling link is an error rather than being replaced by a regular file).
fn resolve_write_target(path: &Path) -> std::io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => fs::canonicalize(path),
        _ => Ok(path.to_path_buf()),
    }
}

/// Back up `content` next to `path` without ever overwriting an earlier backup.
///
/// The first backup is `<name>.bak`; later ones are `<name>.bak.<unix-ts>` (with a
/// numeric suffix on collision). If an existing backup already holds exactly this
/// content, it is reused instead of writing a duplicate. Backups carry the
/// config's permission bits (a reused backup is set to them too).
fn write_unique_backup(path: &Path, content: &str) -> Result<PathBuf> {
    let base = path.with_extension("json.bak");
    let mode = config_mode(path);

    if let (Some(dir), Some(name)) = (base.parent(), base.file_name()) {
        let prefix = name.to_string_lossy().into_owned();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let candidate = entry.path();
                let is_backup = candidate
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with(&prefix));
                if is_backup
                    && fs::read_to_string(&candidate).is_ok_and(|existing| existing == content)
                {
                    set_mode(&candidate, mode).map_err(EngramError::Io)?;
                    return Ok(candidate);
                }
            }
        }
    }

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let stem = base.as_os_str().to_string_lossy().into_owned();
    let mut candidates = vec![base.clone(), PathBuf::from(format!("{stem}.{ts}"))];
    candidates.extend((1..1000).map(|n| PathBuf::from(format!("{stem}.{ts}.{n}"))));

    for candidate in candidates {
        match create_new_with_mode(&candidate, mode) {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(content.as_bytes())
                    .and_then(|_| file.sync_all())
                    .map_err(EngramError::Io)?;
                return Ok(candidate);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(EngramError::Io(e)),
        }
    }
    Err(EngramError::Storage(format!(
        "could not allocate a unique backup name for {}",
        path.display()
    )))
}

/// Replace `path` atomically: write a sibling temp file, fsync it, then rename.
/// A crash or full disk therefore never leaves a truncated client config. The
/// temp file is created with the config's permission bits (0600 for a new
/// config), and a symlinked config is written through to its target.
fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;

    let target = resolve_write_target(path).map_err(EngramError::Io)?;
    let mode = config_mode(&target);
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config".to_string());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = target.with_file_name(format!(".{name}.engram-tmp.{}.{nanos}", std::process::id()));

    let result = (|| -> std::io::Result<()> {
        let mut file = create_new_with_mode(&tmp, mode)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &target)
    })();

    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(EngramError::Io(e));
    }
    Ok(())
}

fn refuse_unparseable(path: &Path, why: &str) -> EngramError {
    EngramError::InvalidInput(format!(
        "{} is not a strict-JSON object with {}; refusing to overwrite it. \
         Fix or remove the file, or re-run with --force to back it up and replace it",
        path.display(),
        why
    ))
}

fn install_to_config_file(path: &Path, server_entry: &Value, force: bool) -> Result<&'static str> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(EngramError::Io)?;
        }
    }

    let mut config: Value = if path.exists() {
        let content = fs::read_to_string(path).map_err(EngramError::Io)?;

        let parsed = match serde_json::from_str::<Value>(&content) {
            Ok(v) if v.is_object() => v,
            Ok(_) if !force => return Err(refuse_unparseable(path, "an object at the top level")),
            Err(e) if !force => {
                return Err(refuse_unparseable(path, &format!("valid syntax ({e})")))
            }
            _ => json!({}),
        };
        if !force
            && parsed
                .get("mcpServers")
                .is_some_and(|servers| !servers.is_object())
        {
            return Err(refuse_unparseable(path, "an object under \"mcpServers\""));
        }

        // Back up before any edit; a backup failure aborts the install.
        write_unique_backup(path, &content)?;
        parsed
    } else {
        json!({})
    };

    let Some(config_obj) = config.as_object_mut() else {
        return Err(refuse_unparseable(path, "an object at the top level"));
    };
    let servers_obj = config_obj
        .entry("mcpServers".to_string())
        .or_insert_with(|| json!({}));
    if !servers_obj.is_object() {
        *servers_obj = json!({});
    }
    let Some(servers) = servers_obj.as_object_mut() else {
        return Err(refuse_unparseable(path, "an object under \"mcpServers\""));
    };

    let action_str = if servers.contains_key("engram") {
        "updated"
    } else {
        "created"
    };
    servers.insert("engram".to_string(), server_entry.clone());

    let formatted = serde_json::to_string_pretty(&config).map_err(EngramError::Serialization)?;
    write_atomic(path, &formatted)?;

    Ok(action_str)
}

fn find_server_executable() -> String {
    if let Ok(output) = Command::new("which").arg("engram-server").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return path_str;
            }
        }
    }
    "not found in PATH (install with `cargo install --path .`)".to_string()
}

fn handle_status(storage: &Storage, target: ClientTarget, port: u16) -> Result<()> {
    println!("=== Engram MCP System & Integration Status ===");

    // 1. Binary checks
    let server_bin = find_server_executable();
    println!("• engram-server Binary:  {}", server_bin);

    // 2. Storage checks
    let stats = storage.with_connection(get_stats)?;
    println!(
        "• Local Memory Database: {} memories, {} identities, {} crossrefs ({:.1} KB)",
        stats.total_memories,
        stats.total_identities,
        stats.total_crossrefs,
        (stats.db_size_bytes as f64) / 1024.0
    );

    // 3. Client Integration Checks
    println!("\n• Client Application Integrations:");
    let specs = get_client_specs();
    for spec in specs {
        if target != ClientTarget::All && spec.target != target {
            continue;
        }

        println!("  [{}]", spec.client_name);
        for path in &spec.paths {
            if path.exists() {
                let content = fs::read_to_string(path).unwrap_or_default();
                let json_val: Value = serde_json::from_str(&content).unwrap_or_default();
                let has_engram = json_val
                    .get("mcpServers")
                    .and_then(|m| m.get("engram"))
                    .is_some();

                if has_engram {
                    let transport = json_val["mcpServers"]["engram"]["args"]
                        .as_array()
                        .and_then(|a| a.get(1))
                        .and_then(|v| v.as_str())
                        .unwrap_or("stdio");
                    let tier = json_val["mcpServers"]["engram"]["env"]["ENGRAM_TOOL_TIER"]
                        .as_str()
                        .unwrap_or("standard");
                    println!(
                        "    ✓ {} (Configured: transport={}, tier={})",
                        path.display(),
                        transport,
                        tier
                    );
                } else {
                    println!(
                        "    - {} (Present, but 'engram' server not configured)",
                        path.display()
                    );
                }
            } else {
                println!("    ○ {} (Not found)", path.display());
            }
        }
    }

    // 4. HTTP Check if requested
    println!("\n• HTTP Endpoint Status (Port {}):", port);
    let http_url = format!("http://localhost:{}/health", port);
    println!("  Probing {} ...", http_url);

    println!("==============================================");
    Ok(())
}

fn handle_uninstall(target: ClientTarget) -> Result<()> {
    println!("=== Engram MCP Client Uninstaller ===");
    let specs = get_client_specs();
    let mut removed_count = 0;

    for spec in specs {
        if target != ClientTarget::All && spec.target != target {
            continue;
        }

        for path in &spec.paths {
            if !path.exists() {
                continue;
            }

            let content = fs::read_to_string(path).map_err(EngramError::Io)?;
            let mut json_val: Value = serde_json::from_str(&content).unwrap_or_default();

            if let Some(servers) = json_val
                .get_mut("mcpServers")
                .and_then(|m| m.as_object_mut())
            {
                if servers.remove("engram").is_some() {
                    let formatted = serde_json::to_string_pretty(&json_val)
                        .map_err(EngramError::Serialization)?;
                    write_atomic(path, &formatted)?;
                    println!(
                        "  [✓] Removed from {} ({})",
                        spec.client_name,
                        path.display()
                    );
                    removed_count += 1;
                }
            }
        }
    }

    println!("-----------------------------------------");
    println!(
        "✅ Engram MCP uninstalled from {} location(s).",
        removed_count
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_install_and_uninstall_mcp_config() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");

        let entry = json!({
            "command": "engram-server",
            "args": ["--transport", "stdio"],
            "env": { "ENGRAM_TOOL_TIER": "standard" }
        });

        // 1. First install (creates file)
        let res = install_to_config_file(&config_file, &entry, false).unwrap();
        assert_eq!(res, "created");
        assert!(config_file.exists());

        let read_content = fs::read_to_string(&config_file).unwrap();
        let parsed: Value = serde_json::from_str(&read_content).unwrap();
        assert!(parsed["mcpServers"]["engram"].is_object());

        // 2. Second install (updates file)
        let res2 = install_to_config_file(&config_file, &entry, false).unwrap();
        assert_eq!(res2, "updated");

        // 3. Verify backup file created
        let bak = dir.path().join("mcp.json.bak");
        assert!(bak.exists());
    }

    #[test]
    fn test_preserves_other_mcp_servers() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");

        // Pre-populate with existing third-party servers
        let existing = json!({
            "mcpServers": {
                "sqlite": { "command": "uvx", "args": ["mcp-server-sqlite"] },
                "github": { "command": "gh", "args": ["mcp"] }
            }
        });
        fs::write(
            &config_file,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let entry = json!({
            "command": "engram-server",
            "args": ["--transport", "stdio"]
        });

        install_to_config_file(&config_file, &entry, false).unwrap();

        let read_content = fs::read_to_string(&config_file).unwrap();
        let parsed: Value = serde_json::from_str(&read_content).unwrap();

        assert!(parsed["mcpServers"]["sqlite"].is_object());
        assert!(parsed["mcpServers"]["github"].is_object());
        assert!(parsed["mcpServers"]["engram"].is_object());
    }

    #[cfg(unix)]
    fn mode_of(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[cfg(unix)]
    fn chmod(path: &Path, mode: u32) {
        set_mode(path, mode).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn install_keeps_an_owner_only_config_and_its_backup_owner_only() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");
        let original = r#"{"mcpServers":{"other":{"env":{"TOKEN":"t"}}}}"#;
        fs::write(&config_file, original).unwrap();
        chmod(&config_file, 0o600);

        install_to_config_file(&config_file, &json!({"command": "engram-server"}), false).unwrap();

        assert_eq!(mode_of(&config_file), 0o600, "rewritten config");
        let bak = dir.path().join("mcp.json.bak");
        assert_eq!(fs::read_to_string(&bak).unwrap(), original);
        assert_eq!(mode_of(&bak), 0o600, "backup of an owner-only config");
    }

    #[cfg(unix)]
    #[test]
    fn install_keeps_a_group_readable_config_mode() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");
        fs::write(&config_file, r#"{"mcpServers":{}}"#).unwrap();
        chmod(&config_file, 0o640);

        install_to_config_file(&config_file, &json!({"command": "engram-server"}), false).unwrap();

        assert_eq!(mode_of(&config_file), 0o640);
        assert_eq!(mode_of(&dir.path().join("mcp.json.bak")), 0o640);
    }

    #[cfg(unix)]
    #[test]
    fn new_config_is_created_owner_only() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");

        install_to_config_file(&config_file, &json!({"command": "engram-server"}), false).unwrap();

        assert_eq!(mode_of(&config_file), 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn temp_and_backup_files_start_with_the_target_mode_before_any_byte() {
        let dir = tempdir().unwrap();
        for mode in [0o600, 0o640] {
            let file_path = dir.path().join(format!("fresh-{mode:o}"));
            let file = create_new_with_mode(&file_path, mode).unwrap();
            assert_eq!(fs::metadata(&file_path).unwrap().len(), 0);
            assert_eq!(mode_of(&file_path), mode, "mode {mode:o} at creation");
            drop(file);
        }
    }

    #[cfg(unix)]
    #[test]
    fn reused_identical_backup_is_narrowed_to_the_config_mode() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("mcp.json");
        let original = r#"{"mcpServers":{}}"#;
        fs::write(&config_file, original).unwrap();
        chmod(&config_file, 0o600);
        let bak = dir.path().join("mcp.json.bak");
        fs::write(&bak, original).unwrap();
        chmod(&bak, 0o644);

        assert_eq!(write_unique_backup(&config_file, original).unwrap(), bak);

        assert_eq!(mode_of(&bak), 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn install_through_a_symlinked_config_keeps_the_symlink() {
        let dir = tempdir().unwrap();
        let real_dir = dir.path().join("dotfiles");
        fs::create_dir(&real_dir).unwrap();
        let real = real_dir.join("mcp.json");
        fs::write(&real, r#"{"mcpServers":{"other":{"command":"x"}}}"#).unwrap();
        chmod(&real, 0o600);
        let link = dir.path().join("mcp.json");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        install_to_config_file(&link, &json!({"command": "engram-server"}), false).unwrap();
        assert_still_a_symlink_to(&link, &real);

        let parsed: Value = serde_json::from_str(&fs::read_to_string(&real).unwrap()).unwrap();
        assert!(parsed["mcpServers"]["engram"].is_object());
        assert!(parsed["mcpServers"]["other"].is_object());
        assert_eq!(mode_of(&real), 0o600);
        for d in [dir.path(), real_dir.as_path()] {
            let leftovers: Vec<_> = fs::read_dir(d)
                .unwrap()
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().contains("engram-tmp"))
                .collect();
            assert!(leftovers.is_empty(), "temp files left in {}", d.display());
        }
    }

    #[cfg(unix)]
    fn assert_still_a_symlink_to(link: &Path, real: &Path) {
        let meta = fs::symlink_metadata(link).unwrap();
        assert!(meta.file_type().is_symlink(), "config symlink was replaced");
        assert_eq!(
            fs::canonicalize(link).unwrap(),
            fs::canonicalize(real).unwrap()
        );
    }

    #[test]
    fn refuses_non_object_config_and_bad_mcp_servers_without_force() {
        let dir = tempdir().unwrap();
        let entry = json!({ "command": "engram-server" });

        for body in ["[1,2]", r#"{"mcpServers":"nope"}"#, "not json"] {
            let config_file = dir.path().join("mcp.json");
            fs::write(&config_file, body).unwrap();
            let err = install_to_config_file(&config_file, &entry, false).unwrap_err();
            assert!(matches!(err, EngramError::InvalidInput(_)), "{body}: {err}");
            assert_eq!(fs::read_to_string(&config_file).unwrap(), body);
            assert!(!dir.path().join("mcp.json.bak").exists());

            install_to_config_file(&config_file, &entry, true).unwrap();
            assert_eq!(
                fs::read_to_string(dir.path().join("mcp.json.bak")).unwrap(),
                body
            );
            fs::remove_file(dir.path().join("mcp.json.bak")).unwrap();
        }
    }
}
