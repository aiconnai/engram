use clap::{Parser, Subcommand};

#[cfg(feature = "attestation")]
use crate::attest::AttestAction;
use crate::maintenance::MaintenanceAction;
#[cfg(feature = "onnx-embed")]
use crate::model::ModelAction;
use crate::session::SessionAction;
#[cfg(feature = "snapshot")]
use crate::snapshot::SnapshotAction;

#[derive(Parser)]
#[command(name = "engram")]
#[command(about = "AI Memory Infrastructure CLI")]
#[command(version)]
pub(crate) struct Cli {
    /// Database path
    #[arg(
        long,
        env = "ENGRAM_DB_PATH",
        default_value = "~/.local/share/engram/memories.db"
    )]
    pub(crate) db_path: String,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Create a new memory
    Create {
        /// Content to remember
        content: String,
        /// Memory type
        #[arg(short, long, default_value = "note")]
        r#type: String,
        /// Tags (comma-separated)
        #[arg(short = 'T', long)]
        tags: Option<String>,
        /// Importance (0-1)
        #[arg(short, long)]
        importance: Option<f32>,
    },
    /// Get a memory by ID
    Get {
        /// Memory ID
        id: i64,
    },
    /// List memories
    List {
        /// Maximum number to return
        #[arg(short, long, default_value = "20")]
        limit: i64,
        /// Filter by tags (comma-separated)
        #[arg(short = 'T', long)]
        tags: Option<String>,
        /// Filter by type
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// Search memories
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: i64,
        /// Show match explanations
        #[arg(short, long)]
        explain: bool,
    },
    /// Delete a memory
    Delete {
        /// Memory ID
        id: i64,
    },
    /// Show statistics
    Stats,
    /// Session continuation and handoff workflows
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// Read-only maintenance and storage diagnostics
    Maintenance {
        #[command(subcommand)]
        action: MaintenanceAction,
    },
    /// Export knowledge graph
    Graph {
        /// Output format (html, json)
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Output file (- for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
        /// Maximum nodes
        #[arg(short, long, default_value = "500")]
        max_nodes: i64,
    },
    /// Link two memories
    Link {
        /// Source memory ID
        from: i64,
        /// Target memory ID
        to: i64,
        /// Relationship type
        #[arg(short, long, default_value = "related_to")]
        edge_type: String,
    },
    /// Show version history
    Versions {
        /// Memory ID
        id: i64,
    },
    /// Interactive mode
    Interactive,
    /// Export memories to Markdown or other formats
    Export {
        #[command(subcommand)]
        action: crate::portability::ExportAction,
    },
    /// Import memories from Markdown files
    Import {
        #[command(subcommand)]
        action: crate::portability::ImportAction,
    },
    /// Manage local embedding models
    #[cfg(feature = "onnx-embed")]
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
    /// Create, load, or inspect .egm snapshots
    #[cfg(feature = "snapshot")]
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },
    /// Log and verify document attestations
    #[cfg(feature = "attestation")]
    Attest {
        #[command(subcommand)]
        action: AttestAction,
    },
}
