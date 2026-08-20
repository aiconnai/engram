//! Engram CLI
//!
//! Command-line interface for memory management.

mod args;
#[cfg(feature = "attestation")]
mod attest;
mod core;
mod graph;
mod interactive;
mod maintenance;
mod mcp;
mod mine;
#[cfg(feature = "onnx-embed")]
mod model;

mod portability;
mod session;
#[cfg(feature = "snapshot")]
mod snapshot;
mod util;
mod wake_up;

use clap::Parser;
use engram::error::Result;
use engram::storage::Storage;
use engram::types::{StorageConfig, StorageMode};

use args::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    #[cfg(feature = "onnx-embed")]
    if let Commands::Model { action } = &cli.command {
        return model::handle_model_action(action);
    }

    let storage = open_storage(&cli.db_path)?;

    match cli.command {
        Commands::Create {
            content,
            r#type,
            tags,
            importance,
        } => core::create(&storage, content, r#type, tags, importance)?,
        Commands::Get { id } => core::get(&storage, id)?,
        Commands::List {
            limit,
            tags,
            r#type,
        } => core::list(&storage, limit, tags, r#type)?,
        Commands::Search {
            query,
            limit,
            explain,
        } => core::search(&storage, query, limit, explain)?,
        Commands::Delete { id } => core::delete(&storage, id)?,
        Commands::Stats => core::stats(&storage)?,
        Commands::Mine {
            path,
            mode,
            wing,
            room,
            workspace,
        } => mine::handle_mine(&storage, &path, &mode, wing, room, &workspace)?,
        Commands::WakeUp { workspace, format } => {
            wake_up::handle_wake_up(&storage, &workspace, &format)?
        }
        Commands::Session { action } => session::handle(&storage, action)?,
        Commands::Maintenance { action } => maintenance::handle(&storage, action)?,
        Commands::Mcp { action } => mcp::handle(&storage, action)?,
        Commands::Export { action } => portability::handle_export(&storage, action)?,

        Commands::Import { action } => portability::handle_import(&storage, action)?,
        Commands::Graph {
            format,
            output,
            max_nodes,
        } => graph::export(&storage, format, output, max_nodes)?,
        Commands::Link {
            from,
            to,
            edge_type,
        } => core::link(&storage, from, to, edge_type)?,
        Commands::Versions { id } => core::versions(&storage, id)?,
        Commands::Interactive => interactive::run(&storage)?,
        #[cfg(feature = "snapshot")]
        Commands::Snapshot { action } => snapshot::handle(&storage, action)?,
        #[cfg(feature = "attestation")]
        Commands::Attest { action } => attest::handle(&storage, action)?,
        #[cfg(feature = "onnx-embed")]
        Commands::Model { .. } => unreachable!("model commands are handled before storage opens"),
    }

    Ok(())
}

fn open_storage(db_path: &str) -> Result<Storage> {
    let config = StorageConfig {
        db_path: shellexpand::tilde(db_path).to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    };

    Storage::open(config)
}
