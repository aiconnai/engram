//! Miscellaneous tool handlers, split by tool family: tags, export/import,
//! maintenance, images, auto-tagging, Langfuse integration (feature-gated),
//! Meilisearch tools (feature-gated), and tool discovery.
//!
//! Stats/cache/compact handlers moved to `stats.rs`.
//! Project context handlers moved to `project_context.rs`.
//! Document ingestion handler moved to `document_ingest.rs`.
//! Summarization/archival handlers moved to `summarize.rs`.

mod auto_tag;
mod discovery;
mod images;
mod import_export;
#[cfg(feature = "langfuse")]
mod langfuse;
mod maintenance;
#[cfg(feature = "meilisearch")]
mod meilisearch;
mod tags;

pub use auto_tag::{memory_auto_tag, memory_suggest_tags};
pub use discovery::discover_tools;
pub use images::{memory_migrate_images, memory_upload_image};
pub use import_export::{memory_export, memory_import};
#[cfg(feature = "langfuse")]
pub use langfuse::{
    langfuse_connect, langfuse_extract_patterns, langfuse_sync, langfuse_sync_status,
    memory_from_trace,
};
pub use maintenance::{memory_rebuild_crossrefs, memory_rebuild_embeddings};
#[cfg(feature = "meilisearch")]
pub use meilisearch::{
    meilisearch_config, meilisearch_reindex, meilisearch_search, meilisearch_status,
};
pub use tags::{memory_tag_hierarchy, memory_tags, memory_validate_tags};
