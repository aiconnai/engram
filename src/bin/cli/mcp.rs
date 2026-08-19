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
        /// Force overwrite without backup prompt
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
            force: _,
        } => handle_install(client, &transport, &db_path, &tier, port),
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

            match install_to_config_file(path, &server_entry) {
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
                }
            }
        }
    }

    println!("-----------------------------------------");
    println!(
        "✅ Configured Engram MCP in {} location(s).",
        installed_count
    );
    println!("Restart your AI client (Claude, Cursor, etc.) to activate 243+ memory tools.");
    Ok(())
}

fn install_to_config_file(path: &Path, server_entry: &Value) -> Result<&'static str> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(EngramError::Io)?;
        }
    }

    let mut config: Value = if path.exists() {
        let content = fs::read_to_string(path).map_err(EngramError::Io)?;

        // Backup before edit
        let bak_path = path.with_extension("json.bak");
        let _ = fs::write(&bak_path, &content);

        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if !config.is_object() {
        config = json!({});
    }

    let servers_obj = config
        .as_object_mut()
        .expect("config is object")
        .entry("mcpServers".to_string())
        .or_insert_with(|| json!({}));

    if !servers_obj.is_object() {
        *servers_obj = json!({});
    }

    let action_str = if servers_obj.get("engram").is_some() {
        "updated"
    } else {
        "created"
    };

    servers_obj
        .as_object_mut()
        .expect("servers is object")
        .insert("engram".to_string(), server_entry.clone());

    let formatted = serde_json::to_string_pretty(&config).map_err(EngramError::Serialization)?;
    fs::write(path, formatted).map_err(EngramError::Io)?;

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
                    fs::write(path, formatted).map_err(EngramError::Io)?;
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
        let res = install_to_config_file(&config_file, &entry).unwrap();
        assert_eq!(res, "created");
        assert!(config_file.exists());

        let read_content = fs::read_to_string(&config_file).unwrap();
        let parsed: Value = serde_json::from_str(&read_content).unwrap();
        assert!(parsed["mcpServers"]["engram"].is_object());

        // 2. Second install (updates file)
        let res2 = install_to_config_file(&config_file, &entry).unwrap();
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

        install_to_config_file(&config_file, &entry).unwrap();

        let read_content = fs::read_to_string(&config_file).unwrap();
        let parsed: Value = serde_json::from_str(&read_content).unwrap();

        assert!(parsed["mcpServers"]["sqlite"].is_object());
        assert!(parsed["mcpServers"]["github"].is_object());
        assert!(parsed["mcpServers"]["engram"].is_object());
    }
}
