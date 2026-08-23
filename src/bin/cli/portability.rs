//! CLI handlers for Markdown export and import (RFC 0004).

use std::path::PathBuf;
use std::str::FromStr;

use clap::Subcommand;
use engram::error::Result;
use engram::portability::{
    export_markdown, import_markdown, ExportGrouping, ExportOptions, ImportOptions,
};
use engram::storage::Storage;

#[derive(Subcommand)]
pub(crate) enum ExportAction {
    /// Export memories to Markdown files (Obsidian vault compatible)
    #[command(alias = "vault")]
    Markdown {
        /// Output directory path
        #[arg(short, long, default_value = "./memories-export")]
        path: PathBuf,
        /// Grouping strategy (flat, day, workspace, entity, type)
        #[arg(short, long, default_value = "flat")]
        group: String,
        /// Optional workspace filter
        #[arg(short, long)]
        workspace: Option<String>,
        /// Include related memory wikilinks
        #[arg(long, default_value_t = true)]
        include_links: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum ImportAction {
    /// Import memories from Markdown files (Obsidian vault compatible)
    #[command(alias = "vault")]
    Markdown {
        /// Input directory path
        #[arg(short, long, default_value = "./memories-export")]
        path: PathBuf,
        /// Dry run mode (simulate import without mutating database)
        #[arg(long)]
        dry_run: bool,
        /// Target workspace override
        #[arg(short, long)]
        workspace: Option<String>,
        /// Force overwrite even on version conflicts
        #[arg(long)]
        force_version: bool,
    },
}

pub(crate) fn handle_export(storage: &Storage, action: ExportAction) -> Result<()> {
    match action {
        ExportAction::Markdown {
            path,
            group,
            workspace,
            include_links,
        } => {
            let grouping = ExportGrouping::from_str(&group)?;
            let report = export_markdown(
                storage,
                &ExportOptions {
                    output_dir: path,
                    grouping,
                    workspace,
                    include_links,
                },
            )?;
            println!(
                "Exported {} memories to {} (workspace: {})",
                report.files_written, report.output_dir, report.workspace
            );
            Ok(())
        }
    }
}

pub(crate) fn handle_import(storage: &Storage, action: ImportAction) -> Result<()> {
    match action {
        ImportAction::Markdown {
            path,
            dry_run,
            workspace,
            force_version,
        } => {
            let report = import_markdown(
                storage,
                &ImportOptions {
                    input_dir: path,
                    dry_run,
                    target_workspace: workspace,
                    force_version,
                },
            )?;
            println!(
                "Scanned {} files: {} in_sync, {} new, {} pending, {} conflict, {} applied (dry_run: {})",
                report.scanned,
                report.in_sync,
                report.new,
                report.pending,
                report.conflict,
                report.applied,
                report.dry_run
            );
            Ok(())
        }
    }
}
