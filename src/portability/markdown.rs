//! Markdown and Obsidian Portability Engine (RFC 0004).
//!
//! Exposes programmatic export and import routines with standardized `engram_*`
//! YAML frontmatter, grouping modes, and drift detection.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;

use crate::embedding::TfIdfEmbedder;
use crate::error::{EngramError, Result};
use crate::mcp::handlers::markdown_export::{
    memory_export_markdown as handler_export, memory_import_markdown as handler_import,
};
use crate::mcp::handlers::HandlerContext;
use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use crate::storage::Storage;

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
    pub dry_run: bool,
    pub target_workspace: Option<String>,
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

fn create_ephemeral_context(storage: Storage) -> HandlerContext {
    HandlerContext {
        storage,
        embedder: Arc::new(TfIdfEmbedder::new(128)),
        fuzzy_engine: Arc::new(parking_lot::Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 300,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("langfuse runtime"),
        ),
    }
}

/// Export memories to Markdown files.
pub fn export_markdown(storage: &Storage, opts: &ExportOptions) -> Result<ExportReport> {
    let ctx = create_ephemeral_context(storage.clone());
    let workspace = opts
        .workspace
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let group_str = match opts.grouping {
        ExportGrouping::Flat => "flat",
        ExportGrouping::Day => "day",
        ExportGrouping::Workspace => "workspace",
        ExportGrouping::Type => "type",
        ExportGrouping::Entity => "entity",
    };

    let val = handler_export(
        &ctx,
        json!({
            "workspace": workspace,
            "output_dir": opts.output_dir.to_str().unwrap_or("./memories-export"),
            "group": group_str,
            "include_links": true
        }),
    );

    if let Some(err) = val.get("error").and_then(|v| v.as_str()) {
        if val.get("files_written").and_then(|v| v.as_u64()) == Some(0) {
            return Ok(ExportReport {
                files_written: 0,
                output_dir: opts.output_dir.display().to_string(),
                workspace,
            });
        }
        return Err(EngramError::InvalidInput(err.to_string()));
    }

    let files_written = val
        .get("files_written")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    Ok(ExportReport {
        files_written,
        output_dir: opts.output_dir.display().to_string(),
        workspace,
    })
}

/// Import Markdown files back into storage with drift detection.
pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<ImportReport> {
    let ctx = create_ephemeral_context(storage.clone());

    let val = handler_import(
        &ctx,
        json!({
            "input_dir": opts.input_dir.to_str().unwrap_or("./memories-export"),
            "workspace": opts.target_workspace,
            "confirm": !opts.dry_run,
        }),
    );

    if let Some(err) = val.get("error").and_then(|v| v.as_str()) {
        return Err(EngramError::InvalidInput(err.to_string()));
    }

    let scanned = val.get("scanned").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let in_sync = val.get("in_sync").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let new = val.get("new").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let pending = val.get("pending").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let conflict = val.get("conflict").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let applied = val.get("applied").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    Ok(ImportReport {
        scanned,
        in_sync,
        new,
        pending,
        conflict,
        applied,
        dry_run: opts.dry_run,
    })
}
