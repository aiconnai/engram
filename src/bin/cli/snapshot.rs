use clap::Subcommand;
use engram::error::Result;
use engram::snapshot::{LoadStrategy, SnapshotBuilder, SnapshotLoader};
use engram::storage::Storage;
use std::str::FromStr as _;

#[derive(Subcommand)]
pub(crate) enum SnapshotAction {
    /// Create a snapshot
    Create {
        /// Output path for the .egm file
        #[arg(short, long)]
        output: String,
        /// Workspace to snapshot
        #[arg(short, long)]
        workspace: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Load a snapshot
    Load {
        /// Path to .egm file
        path: String,
        /// Load strategy: merge, replace, isolate, dry_run
        #[arg(short, long, default_value = "merge")]
        strategy: String,
        /// Target workspace
        #[arg(short = 'w', long)]
        target_workspace: Option<String>,
    },
    /// Inspect a snapshot
    Inspect {
        /// Path to .egm file
        path: String,
    },
}

pub(crate) fn handle(storage: &Storage, action: SnapshotAction) -> Result<()> {
    match action {
        SnapshotAction::Create {
            output,
            workspace,
            description,
        } => create(storage, output, workspace, description),
        SnapshotAction::Load {
            path,
            strategy,
            target_workspace,
        } => load(storage, path, strategy, target_workspace),
        SnapshotAction::Inspect { path } => inspect(path),
    }
}

fn create(
    storage: &Storage,
    output: String,
    workspace: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut builder = SnapshotBuilder::new(storage.clone());
    if let Some(ws) = workspace {
        builder = builder.workspace(&ws);
    }
    if let Some(desc) = description {
        builder = builder.description(&desc);
    }
    let path = std::path::Path::new(&output);
    match builder.build(path) {
        Ok(manifest) => {
            println!(
                "Snapshot created: {} ({} memories)",
                output, manifest.memory_count
            );
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        Err(e) => {
            eprintln!("Error creating snapshot: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn load(
    storage: &Storage,
    path: String,
    strategy: String,
    target_workspace: Option<String>,
) -> Result<()> {
    let load_strategy = match LoadStrategy::from_str(&strategy) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Invalid strategy '{}': {}", strategy, e);
            std::process::exit(1);
        }
    };
    let p = std::path::Path::new(&path);
    match SnapshotLoader::load(
        storage,
        p,
        load_strategy,
        target_workspace.as_deref(),
        None,
        None,
    ) {
        Ok(result) => {
            println!(
                "Loaded {} memories, {} skipped",
                result.memories_loaded, result.memories_skipped
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            eprintln!("Error loading snapshot: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn inspect(path: String) -> Result<()> {
    let p = std::path::Path::new(&path);
    match SnapshotLoader::inspect(p) {
        Ok(info) => {
            println!("Snapshot: {}", path);
            println!("  File size: {} bytes", info.file_size_bytes);
            println!("  Memories:  {}", info.manifest.memory_count);
            println!("  Entities:  {}", info.manifest.entity_count);
            println!("  Edges:     {}", info.manifest.edge_count);
            println!("  Created:   {}", info.manifest.created_at.to_rfc3339());
            if let Some(desc) = &info.manifest.description {
                println!("  Desc:      {}", desc);
            }
            println!("  Encrypted: {}", info.manifest.encrypted);
            println!("  Signed:    {}", info.manifest.signed);
        }
        Err(e) => {
            eprintln!("Error inspecting snapshot: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
