//! CLI handler for Spatial Memory Palace commands (`engram palace`).

use clap::Subcommand;
use engram::error::{EngramError, Result};
use engram::spatial::{generate_palace_visualizer, PalaceFormat, PalaceGraph};
use engram::storage::Storage;

#[derive(Subcommand, Debug)]
pub(crate) enum PalaceAction {
    /// List wings, rooms, and drawer counts in the memory palace
    Ls {
        /// Target workspace (Palace)
        #[arg(short = 'w', long, default_value = "default")]
        workspace: String,
        /// Optional wing filter
        #[arg(short = 'W', long)]
        wing: Option<String>,
    },
    /// View a specific memory drawer by ID
    Drawer {
        /// Memory drawer ID
        id: i64,
    },
    /// Generate or export a visual representation of the palace
    Visualize {
        /// Target workspace (Palace)
        #[arg(short = 'w', long, default_value = "default")]
        workspace: String,
        /// Optional wing filter
        #[arg(short = 'W', long)]
        wing: Option<String>,
        /// Export format: html, ascii, svg, mermaid, json
        #[arg(short = 'f', long, default_value = "ascii")]
        format: String,
        /// Optional output file path (default: print to stdout)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
}

pub(crate) fn handle(storage: &Storage, action: PalaceAction) -> Result<()> {
    match action {
        PalaceAction::Ls { workspace, wing } => {
            let graph = PalaceGraph::extract(storage, &workspace, wing.as_deref())?;
            println!("{}", graph.render_ascii());
            Ok(())
        }
        PalaceAction::Drawer { id } => {
            storage.with_connection(
                |conn| match engram::storage::queries::get_memory(conn, id) {
                    Ok(mem) => {
                        let scope_display = match mem.scope.scope_id() {
                            Some(id) => format!("{}:{}", mem.scope.scope_type(), id),
                            None => mem.scope.scope_type().to_string(),
                        };
                        println!(
                            "📦 [DRAWER #{}] Type: {} | Scope: {}",
                            mem.id,
                            mem.memory_type.as_str(),
                            scope_display
                        );
                        println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                        println!("{}", mem.content);
                        println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                        println!(
                            "Tags: {} | Importance: {:.2} | Created: {}",
                            mem.tags.join(", "),
                            mem.importance,
                            mem.created_at
                        );
                        Ok(())
                    }
                    Err(EngramError::NotFound(_)) => {
                        eprintln!("Error: Drawer with ID {} not found", id);
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
            )
        }
        PalaceAction::Visualize {
            workspace,
            wing,
            format,
            output,
        } => {
            let fmt = format.parse::<PalaceFormat>()?;
            let res = generate_palace_visualizer(
                storage,
                &workspace,
                wing.as_deref(),
                fmt,
                output.as_deref(),
            )?;

            if let Some(path) = &res.output_path {
                println!("✅ Palace visualization saved to: {}", path);
            } else {
                println!("{}", res.rendered);
            }
            Ok(())
        }
    }
}
