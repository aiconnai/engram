//! Markdown and Obsidian Portability Engine (RFC 0004).
//!
//! Exposes programmatic export and import routines with standardized `engram_*`
//! YAML frontmatter, grouping modes, and drift detection.

use crate::auth::{
    access::{check_environment_mode, markdown_required_mode, ModeDenial},
    TransportPrincipal,
};
use crate::error::{EngramError, Result};
use crate::storage::Storage;
use std::path::PathBuf;

pub(crate) mod export;
mod frontmatter;
pub(crate) mod import;
mod paths;

/// Only the dependencies used by the workflow; adapters enforce call policy.
/// Stored-workspace checks stay inside the read/mutation snapshot.
pub(crate) struct MarkdownContext<'a> {
    pub storage: &'a Storage,
    pub principal: Option<&'a TransportPrincipal>,
}

/// Grouping mode for Markdown export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportGrouping {
    #[default]
    Flat,
    Day,
    Workspace,
    Type,
    Entity,
}

impl std::str::FromStr for ExportGrouping {
    type Err = EngramError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "flat" => Ok(Self::Flat),
            "day" => Ok(Self::Day),
            "workspace" | "project" => Ok(Self::Workspace),
            "type" => Ok(Self::Type),
            "entity" | "tag" => Ok(Self::Entity),
            _ => Err(EngramError::InvalidInput(format!(
                "Unknown export grouping: '{s}'. Expected flat, day, workspace, type, or entity."
            ))),
        }
    }
}

/// Export configuration options.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub output_dir: PathBuf,
    pub grouping: ExportGrouping,
    pub workspace: Option<String>,
    pub include_links: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./memories-export"),
            grouping: ExportGrouping::Flat,
            workspace: None,
            include_links: true,
        }
    }
}

/// Result of an export run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportReport {
    pub files_written: usize,
    pub output_dir: String,
    pub workspace: String,
}

/// Import configuration options.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub input_dir: PathBuf,
    /// Review/stage only without writing to SQLite (RFC 0004 default: `true`; set `false` to apply writes).
    pub dry_run: bool,
    pub target_workspace: Option<String>,
    pub force_version: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::from("./memories-export"),
            dry_run: true,
            target_workspace: None,
            force_version: false,
        }
    }
}

/// Result of an import run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportReport {
    pub scanned: usize,
    pub in_sync: usize,
    pub new: usize,
    pub pending: usize,
    pub conflict: usize,
    pub applied: usize,
    pub dry_run: bool,
}

/// Export memories to Markdown files.
pub fn export_markdown(storage: &Storage, opts: &ExportOptions) -> Result<ExportReport> {
    library_policy("memory_export_markdown")?;
    let workspace = opts.workspace.as_deref().unwrap_or("default");
    let group = match opts.grouping {
        ExportGrouping::Flat => "flat",
        ExportGrouping::Day => "day",
        ExportGrouping::Workspace => "workspace",
        ExportGrouping::Type => "type",
        ExportGrouping::Entity => "entity",
    };
    let context = MarkdownContext {
        storage,
        principal: None,
    };
    let result = export::export(
        &context,
        export::ExportRequest {
            workspace,
            output_dir: Some(opts.output_dir.to_str().unwrap_or("./memories-export")),
            group,
            include_links: opts.include_links,
        },
    );
    // The Rust contract treats an empty workspace as a successful empty export.
    let files_written = match result {
        Ok(outcome) => outcome.files_written,
        Err(export::ExportFailure::EmptyWorkspace(_)) => 0,
        Err(error) => return Err(EngramError::InvalidInput(error.to_string())),
    };
    Ok(ExportReport {
        files_written,
        output_dir: opts.output_dir.display().to_string(),
        workspace: workspace.into(),
    })
}

/// Import Markdown files back into storage with drift detection.
pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<ImportReport> {
    library_policy("memory_import_markdown")?;
    let context = MarkdownContext {
        storage,
        principal: None,
    };
    let outcome = import::import(
        &context,
        import::ImportRequest {
            input_dir: opts.input_dir.to_str().unwrap_or("./memories-export"),
            workspace: opts.target_workspace.as_deref(),
            force_version: opts.force_version,
            confirm: !opts.dry_run,
        },
    )
    .map_err(|error| {
        let (error, attempted) = error.into_parts();
        let message = if attempted.is_some() {
            "Import completion is uncertain; inspect the canonical database before retrying".into()
        } else {
            compatibility_error_message(error)
        };
        EngramError::InvalidInput(message)
    })?;
    Ok(ImportReport {
        scanned: outcome.scanned,
        in_sync: outcome.in_sync,
        new: outcome.new,
        pending: outcome.pending,
        conflict: outcome.conflict,
        applied: outcome.applied,
        dry_run: opts.dry_run,
    })
}

// Preserve the established Rust facade's error messages, without a JSON/handler round-trip.
fn compatibility_error_message(error: EngramError) -> String {
    match error {
        EngramError::NotFound(id) => format!("memory not found: '{id}'"),
        EngramError::InvalidInput(message)
        | EngramError::Conflict(message)
        | EngramError::Unauthorized(message)
        | EngramError::Auth(message)
        | EngramError::Duplicate { message, .. } => message,
        EngramError::RateLimited(seconds) => format!("Rate limited: retry after {seconds} seconds"),
        EngramError::Database(error) => format!("Database error: {error}"),
        EngramError::Io(error) => format!("IO error: {error}"),
        other => other.to_string(),
    }
}

fn library_policy(operation: &str) -> Result<()> {
    crate::observability::record_authorization_check();
    check_environment_mode(markdown_required_mode()).map_err(|denial| {
        let message = match denial {
            ModeDenial::InvalidConfiguration(_) => denial.to_string(),
            ModeDenial::Insufficient { required, .. } => {
                format!("{operation} requires {} mode", required.as_str())
            }
        };
        EngramError::InvalidInput(message)
    })
}

/// Preview Markdown files import without mutating storage (dry-run mode).
pub fn preview_markdown(
    storage: &Storage,
    input_dir: PathBuf,
    target_workspace: Option<String>,
) -> Result<ImportReport> {
    import_markdown(
        storage,
        &ImportOptions {
            input_dir,
            dry_run: true,
            target_workspace,
            force_version: false,
        },
    )
}

#[cfg(test)]
mod tests;
