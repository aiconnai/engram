//! Markdown export/import handlers — human-readable memory files.
//!
//! Exports memories as Markdown files with RFC 0004 canonical YAML frontmatter
//! and wiki-style `[[links]]` for browsing and version control. Import supports
//! review mode (dry-run) and confirm mode.

mod export;
mod frontmatter;
mod import;
mod paths;

pub use export::memory_export_markdown;
pub use import::{memory_import_markdown, ImportStatus};
pub(crate) use paths::validate_export_dir;
