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
    /// Import memories from Markdown files (Obsidian vault compatible; defaults to review/dry-run mode)
    #[command(alias = "vault")]
    Markdown {
        /// Input directory path
        #[arg(short, long, default_value = "./memories-export")]
        path: PathBuf,
        /// Apply staged creates and updates to SQLite (required to mutate database per RFC 0004)
        #[arg(long)]
        confirm: bool,
        /// Explicit dry-run review mode (default when --confirm is omitted)
        #[arg(long, conflicts_with = "confirm")]
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

pub(crate) fn resolve_effective_dry_run(confirm: bool, dry_run: bool) -> bool {
    dry_run || !confirm
}

pub(crate) fn handle_import(storage: &Storage, action: ImportAction) -> Result<()> {
    match action {
        ImportAction::Markdown {
            path,
            confirm,
            dry_run,
            workspace,
            force_version,
        } => {
            let effective_dry_run = resolve_effective_dry_run(confirm, dry_run);
            let report = import_markdown(
                storage,
                &ImportOptions {
                    input_dir: path,
                    dry_run: effective_dry_run,
                    target_workspace: workspace,
                    force_version,
                },
            )?;
            let suffix = if report.dry_run {
                " [dry-run — pass --confirm to apply]"
            } else {
                ""
            };
            println!(
                "Scanned {} files: {} in_sync, {} new, {} pending, {} conflict, {} applied (dry_run: {}){}",
                report.scanned,
                report.in_sync,
                report.new,
                report.pending,
                report.conflict,
                report.applied,
                report.dry_run,
                suffix
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[derive(Parser)]
    struct TestImportCli {
        #[command(subcommand)]
        action: ImportAction,
    }

    #[test]
    fn cli_import_markdown_defaults_to_safe_dry_run_until_confirmed() {
        let parsed = TestImportCli::try_parse_from(["test", "markdown", "--path", "./vault"])
            .expect("parse default markdown import");
        match parsed.action {
            ImportAction::Markdown {
                confirm, dry_run, ..
            } => {
                assert!(!confirm);
                assert!(!dry_run);
                assert!(resolve_effective_dry_run(confirm, dry_run));
            }
        }

        let explicit_dry =
            TestImportCli::try_parse_from(["test", "markdown", "--path", "./vault", "--dry-run"])
                .expect("parse explicit --dry-run");
        match explicit_dry.action {
            ImportAction::Markdown {
                confirm, dry_run, ..
            } => {
                assert!(!confirm);
                assert!(dry_run);
                assert!(resolve_effective_dry_run(confirm, dry_run));
            }
        }

        let confirmed =
            TestImportCli::try_parse_from(["test", "markdown", "--path", "./vault", "--confirm"])
                .expect("parse --confirm");
        match confirmed.action {
            ImportAction::Markdown {
                confirm, dry_run, ..
            } => {
                assert!(confirm);
                assert!(!dry_run);
                assert!(!resolve_effective_dry_run(confirm, dry_run));
            }
        }

        let conflict = TestImportCli::try_parse_from([
            "test",
            "markdown",
            "--path",
            "./vault",
            "--confirm",
            "--dry-run",
        ]);
        assert!(
            conflict.is_err(),
            "--confirm and --dry-run must conflict in CLI parsing"
        );
    }
}
