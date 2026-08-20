//! Document ingestion for Engram (RML-928)
//!
//! Provides document parsing, chunking, and ingestion into the memory store.
//! Supported formats:
//! - Markdown (.md): Uses pulldown-cmark for parsing, extracts sections
//! - PDF (.pdf): Uses pdf-extract for text extraction by page
//!
//! # Usage
//!
//! ```ignore
//! use engram::intelligence::document_ingest::{DocumentIngestor, IngestConfig};
//! use engram::Storage;
//!
//! let storage = Storage::open_in_memory()?;
//! let ingestor = DocumentIngestor::new(&storage);
//!
//! let result = ingestor.ingest_file("docs/handbook.pdf", IngestConfig::default())?;
//! println!("Ingested {} chunks", result.chunks_created);
//! ```

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use sha2::{Digest, Sha256};

use crate::error::{EngramError, Result};
use crate::storage::queries::{create_memory, list_memories};
use crate::storage::Storage;
use crate::types::{CreateMemoryInput, ListOptions, MemoryType};

/// Maximum file size in bytes (10 MB default)
pub const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of pages accepted by optional PDF ingestion.
const DEFAULT_MAX_PDF_PAGES: usize = 200;

/// Maximum UTF-8 text bytes emitted by optional PDF ingestion.
const DEFAULT_MAX_PDF_TEXT_BYTES: usize = 2 * 1024 * 1024;

/// Default chunk size in characters
pub const DEFAULT_CHUNK_SIZE: usize = 1200;

/// Default overlap between chunks in characters
pub const DEFAULT_OVERLAP: usize = 200;

/// Document format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Markdown,
    Pdf,
}

impl DocumentFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "md" | "markdown" => Some(DocumentFormat::Markdown),
            "pdf" => Some(DocumentFormat::Pdf),
            _ => None,
        }
    }

    /// Parse format from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "md" | "markdown" => Some(DocumentFormat::Markdown),
            "pdf" => Some(DocumentFormat::Pdf),
            "auto" => None, // Will be detected from path
            _ => None,
        }
    }
}

/// Configuration for document ingestion
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// Force specific format (None = auto-detect)
    pub format: Option<DocumentFormat>,
    /// Maximum characters per chunk
    pub chunk_size: usize,
    /// Overlap between chunks in characters
    pub overlap: usize,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Additional tags to add to all chunks
    pub extra_tags: Vec<String>,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            format: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            overlap: DEFAULT_OVERLAP,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            extra_tags: vec![],
        }
    }
}

/// Result of document ingestion
#[derive(Debug, Clone, serde::Serialize)]
pub struct IngestResult {
    /// Document ID (SHA-256 hash of file content)
    pub document_id: String,
    /// Number of chunks created
    pub chunks_created: usize,
    /// Number of chunks skipped (already existed)
    pub chunks_skipped: usize,
    /// Total number of chunks processed
    pub chunks_total: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Warnings encountered during ingestion
    pub warnings: Vec<String>,
}

/// A section extracted from a document
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentSection {
    /// Section path (e.g., "Security > Key Rotation")
    pub section_path: String,
    /// Section content
    pub content: String,
    /// Page number (for PDFs)
    pub page: Option<usize>,
    /// Heading level (1-6 for Markdown)
    pub level: Option<usize>,
}

/// A chunk ready for ingestion
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    /// Chunk content
    pub content: String,
    /// Source file path
    pub source_path: String,
    /// Document ID
    pub doc_id: String,
    /// Chunk index within document
    pub chunk_index: usize,
    /// Section path
    pub section_path: String,
    /// Page number (for PDFs)
    pub page: Option<usize>,
    /// SHA-256 hash of chunk content
    pub chunk_hash: String,
}

#[derive(Debug, Clone, Copy)]
struct PdfExtractionLimits {
    max_input_bytes: usize,
    max_pages: usize,
    max_text_bytes: usize,
}

impl PdfExtractionLimits {
    fn for_config(config: &IngestConfig) -> Self {
        Self {
            max_input_bytes: usize::try_from(config.max_file_size)
                .unwrap_or(usize::MAX)
                .min(DEFAULT_MAX_FILE_SIZE as usize),
            max_pages: DEFAULT_MAX_PDF_PAGES,
            max_text_bytes: DEFAULT_MAX_PDF_TEXT_BYTES,
        }
    }
}

#[cfg(feature = "pdf")]
struct BoundedPdfText {
    value: String,
    max_bytes: usize,
    exceeded: bool,
}

#[cfg(feature = "pdf")]
impl BoundedPdfText {
    fn new(max_bytes: usize) -> Self {
        Self {
            value: String::new(),
            max_bytes,
            exceeded: false,
        }
    }
}

#[cfg(feature = "pdf")]
impl std::io::Write for BoundedPdfText {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next_len = self.value.len().checked_add(bytes.len());
        if next_len.is_none_or(|length| length > self.max_bytes) {
            self.exceeded = true;
            return Err(std::io::Error::other("PDF extracted text limit exceeded"));
        }

        let text = std::str::from_utf8(bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        self.value.push_str(text);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Document ingestor
pub struct DocumentIngestor<'a> {
    storage: &'a Storage,
}

impl<'a> DocumentIngestor<'a> {
    /// Create a new document ingestor
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Ingest a document file
    pub fn ingest_file(
        &self,
        path: impl AsRef<Path>,
        config: IngestConfig,
    ) -> Result<IngestResult> {
        let path = path.as_ref();
        let start = Instant::now();
        let mut warnings = Vec::new();

        if config.chunk_size == 0 {
            return Err(EngramError::InvalidInput(
                "chunk_size must be greater than 0".to_string(),
            ));
        }

        if config.overlap >= config.chunk_size {
            return Err(EngramError::InvalidInput(
                "overlap must be less than chunk_size".to_string(),
            ));
        }

        // Check file exists
        if !path.exists() {
            return Err(EngramError::InvalidInput(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Check file size
        let metadata = fs::metadata(path)
            .map_err(|e| EngramError::Storage(format!("Failed to read file metadata: {}", e)))?;

        // Determine format
        let format = config
            .format
            .or_else(|| DocumentFormat::from_path(path))
            .ok_or_else(|| {
                EngramError::InvalidInput(format!("Unknown file format for: {}", path.display()))
            })?;

        let max_input_bytes = match format {
            DocumentFormat::Markdown => config.max_file_size,
            DocumentFormat::Pdf => config.max_file_size.min(DEFAULT_MAX_FILE_SIZE),
        };
        if metadata.len() > max_input_bytes {
            return Err(EngramError::InvalidInput(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                max_input_bytes
            )));
        }

        // Read at most the configured limit plus one byte so growth after metadata stays bounded.
        let file = fs::File::open(path)
            .map_err(|e| EngramError::Storage(format!("Failed to open file: {}", e)))?;
        let mut content = Vec::new();
        file.take(max_input_bytes.saturating_add(1))
            .read_to_end(&mut content)
            .map_err(|e| EngramError::Storage(format!("Failed to read file: {}", e)))?;
        if u64::try_from(content.len()).map_or(true, |bytes| bytes > max_input_bytes) {
            return Err(EngramError::InvalidInput(format!(
                "File too large: {} bytes (max: {} bytes)",
                content.len(),
                max_input_bytes
            )));
        }

        // Compute document ID
        let doc_id = compute_hash(&content);

        // Extract sections based on format
        let sections = match format {
            DocumentFormat::Markdown => {
                let text = String::from_utf8_lossy(&content);
                extract_markdown_sections(&text)
            }
            DocumentFormat::Pdf => {
                extract_pdf_sections(&content, PdfExtractionLimits::for_config(&config))?
            }
        };

        if sections.is_empty() {
            if matches!(format, DocumentFormat::Pdf) {
                return Err(EngramError::InvalidInput(
                    "No text extracted from PDF".to_string(),
                ));
            }
            warnings.push("No content extracted from document".to_string());
        }

        // Create chunks
        let source_path = path.to_string_lossy().to_string();
        let chunks = create_chunks(sections, &source_path, &doc_id, &config);

        // Ingest chunks
        let existing_hashes = self.existing_chunk_hashes(&doc_id)?;
        let mut chunks_created = 0;
        let mut chunks_skipped = 0;

        for chunk in &chunks {
            if existing_hashes.contains(&chunk.chunk_hash) {
                chunks_skipped += 1;
                continue;
            }

            // Create memory for chunk
            self.create_chunk_memory(chunk, &config.extra_tags)?;
            chunks_created += 1;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(IngestResult {
            document_id: doc_id,
            chunks_created,
            chunks_skipped,
            chunks_total: chunks.len(),
            duration_ms,
            warnings,
        })
    }

    /// Fetch existing chunk hashes for a document in a single pass
    fn existing_chunk_hashes(&self, doc_id: &str) -> Result<HashSet<String>> {
        const PAGE_SIZE: i64 = 500;
        self.storage.with_connection(|conn| {
            let mut hashes = HashSet::new();
            let mut offset = 0;

            loop {
                let mut filter = HashMap::new();
                filter.insert(
                    "doc_id".to_string(),
                    serde_json::Value::String(doc_id.to_string()),
                );

                let options = ListOptions {
                    limit: Some(PAGE_SIZE),
                    offset: Some(offset),
                    tags: Some(vec!["document-chunk".to_string()]),
                    memory_type: None,
                    sort_by: None,
                    sort_order: None,
                    scope: None,
                    workspace: None,
                    workspaces: None,
                    tier: None,
                    metadata_filter: Some(filter),
                    filter: None,
                    include_archived: false,
                };

                let results = list_memories(conn, &options)?;
                for memory in &results {
                    if let Some(hash) = memory.metadata.get("chunk_hash").and_then(|v| v.as_str()) {
                        hashes.insert(hash.to_string());
                    }
                }

                if results.len() < PAGE_SIZE as usize {
                    break;
                }

                offset += PAGE_SIZE;
            }

            Ok(hashes)
        })
    }

    /// Create a memory entry for a chunk
    fn create_chunk_memory(&self, chunk: &DocumentChunk, extra_tags: &[String]) -> Result<()> {
        let mut tags = vec!["document-chunk".to_string()];
        tags.extend(extra_tags.iter().cloned());

        let mut metadata = HashMap::new();
        metadata.insert(
            "source_file".to_string(),
            serde_json::Value::String(
                Path::new(&chunk.source_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
            ),
        );
        metadata.insert(
            "source_path".to_string(),
            serde_json::Value::String(chunk.source_path.clone()),
        );
        metadata.insert(
            "doc_id".to_string(),
            serde_json::Value::String(chunk.doc_id.clone()),
        );
        metadata.insert(
            "chunk_index".to_string(),
            serde_json::Value::Number(chunk.chunk_index.into()),
        );
        metadata.insert(
            "section_path".to_string(),
            serde_json::Value::String(chunk.section_path.clone()),
        );
        metadata.insert(
            "chunk_hash".to_string(),
            serde_json::Value::String(chunk.chunk_hash.clone()),
        );

        if let Some(page) = chunk.page {
            metadata.insert("page".to_string(), serde_json::Value::Number(page.into()));
        }

        let input = CreateMemoryInput {
            content: chunk.content.clone(),
            memory_type: MemoryType::Context,
            tags,
            metadata,
            importance: Some(0.5),
            scope: crate::types::MemoryScope::Global,
            workspace: None,
            tier: crate::types::MemoryTier::Permanent,
            defer_embedding: false,
            ttl_seconds: None,
            dedup_mode: Default::default(),
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        };

        self.storage.with_connection(|conn| {
            create_memory(conn, &input)?;
            Ok(())
        })
    }
}

/// Compute SHA-256 hash of content
fn compute_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Extract sections from Markdown content
fn extract_markdown_sections(content: &str) -> Vec<DocumentSection> {
    let parser = Parser::new(content);
    let mut sections = Vec::new();
    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut current_content = String::new();
    let mut current_section_path = String::new();
    let mut in_heading = false;
    let mut current_heading_text = String::new();
    let mut current_heading_level = 0usize;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Save previous section if it has content
                if !current_content.trim().is_empty() {
                    sections.push(DocumentSection {
                        section_path: if current_section_path.is_empty() {
                            "Preamble".to_string()
                        } else {
                            current_section_path.clone()
                        },
                        content: current_content.trim().to_string(),
                        page: None,
                        level: if heading_stack.is_empty() {
                            None
                        } else {
                            Some(heading_stack.last().map(|(l, _)| *l).unwrap_or(1))
                        },
                    });
                    current_content.clear();
                }

                in_heading = true;
                current_heading_text.clear();
                current_heading_level = heading_level_to_usize(level);
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;

                // Update heading stack
                while !heading_stack.is_empty()
                    && heading_stack.last().map(|(l, _)| *l).unwrap_or(0) >= current_heading_level
                {
                    heading_stack.pop();
                }
                heading_stack.push((current_heading_level, current_heading_text.clone()));

                // Build section path
                current_section_path = heading_stack
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ");
            }
            Event::Text(text) => {
                if in_heading {
                    current_heading_text.push_str(&text);
                } else {
                    current_content.push_str(&text);
                }
            }
            Event::Code(code) => {
                if in_heading {
                    current_heading_text.push_str(&code);
                } else {
                    current_content.push('`');
                    current_content.push_str(&code);
                    current_content.push('`');
                }
            }
            Event::SoftBreak | Event::HardBreak if !in_heading => {
                current_content.push('\n');
            }
            _ => {}
        }
    }

    // Save final section
    if !current_content.trim().is_empty() {
        sections.push(DocumentSection {
            section_path: if current_section_path.is_empty() {
                "Preamble".to_string()
            } else {
                current_section_path
            },
            content: current_content.trim().to_string(),
            page: None,
            level: heading_stack.last().map(|(l, _)| *l),
        });
    }

    sections
}

/// Convert pulldown_cmark HeadingLevel to usize
fn heading_level_to_usize(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Extract sections from PDF content
///
/// Requires the `pdf` feature to be enabled.
#[cfg(feature = "pdf")]
fn extract_pdf_sections(
    content: &[u8],
    limits: PdfExtractionLimits,
) -> Result<Vec<DocumentSection>> {
    validate_pdf_input_bounds(content, limits)?;
    super::pdf_worker::extract(content, limits.max_pages, limits.max_text_bytes)
}

#[cfg(feature = "pdf")]
fn contain_pdf_parser_unwind<T>(operation: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).map_err(|_| {
        EngramError::InvalidInput(
            "PDF extraction failed: parser rejected malformed data".to_string(),
        )
    })?
}

#[cfg(feature = "pdf")]
fn extract_pdf_sections_inner(
    content: &[u8],
    limits: PdfExtractionLimits,
) -> Result<Vec<DocumentSection>> {
    validate_pdf_input_bounds(content, limits)?;
    let mut document = lopdf::Document::load_mem(content)
        .map_err(|error| EngramError::InvalidInput(format!("PDF parsing failed: {error}")))?;
    if document.is_encrypted() {
        document.decrypt("").map_err(|error| {
            EngramError::InvalidInput(format!("PDF decryption failed: {error}"))
        })?;
    }

    let bounded_page_count = document.page_iter().take(limits.max_pages + 1).count();
    if bounded_page_count > limits.max_pages {
        return Err(EngramError::InvalidInput(format!(
            "PDF page count exceeds limit: {} pages (max: {} pages)",
            bounded_page_count, limits.max_pages
        )));
    }

    let pages = document.get_pages();

    let mut extracted_text_bytes = 0usize;
    let mut sections = Vec::with_capacity(pages.len());
    for (page_index, page_number) in pages.keys().enumerate() {
        let mut page_text = BoundedPdfText::new(limits.max_text_bytes - extracted_text_bytes);
        let extraction_result = {
            let writer: &mut dyn std::io::Write = &mut page_text;
            let mut output = pdf_extract::PlainTextOutput::new(writer);
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pdf_extract::output_doc_page(&document, &mut output, *page_number)
            }))
        }
        .map_err(|_| {
            EngramError::InvalidInput(
                "PDF extraction failed: parser rejected malformed page data".to_string(),
            )
        })?;
        if page_text.exceeded {
            return Err(EngramError::InvalidInput(format!(
                "PDF extracted text exceeds limit: more than {} bytes",
                limits.max_text_bytes
            )));
        }
        extraction_result.map_err(|error| {
            EngramError::InvalidInput(format!("PDF extraction failed: {error}"))
        })?;

        extracted_text_bytes = extracted_text_bytes
            .checked_add(page_text.value.len())
            .ok_or_else(|| {
                EngramError::InvalidInput("PDF extracted text byte count overflow".to_string())
            })?;
        if extracted_text_bytes > limits.max_text_bytes {
            return Err(EngramError::InvalidInput(format!(
                "PDF extracted text exceeds limit: {} bytes (max: {} bytes)",
                extracted_text_bytes, limits.max_text_bytes
            )));
        }

        if !page_text.value.trim().is_empty() {
            sections.push(DocumentSection {
                section_path: format!("Page {}", page_index + 1),
                content: page_text.value.trim().to_string(),
                page: Some(page_index + 1),
                level: None,
            });
        }
    }

    Ok(sections)
}

#[cfg(feature = "pdf")]
pub(crate) fn extract_pdf_sections_in_worker(
    content: &[u8],
    max_pages: usize,
    max_text_bytes: usize,
) -> Result<Vec<DocumentSection>> {
    let limits = PdfExtractionLimits {
        max_input_bytes: DEFAULT_MAX_FILE_SIZE as usize,
        max_pages,
        max_text_bytes,
    };
    contain_pdf_parser_unwind(|| extract_pdf_sections_inner(content, limits))
}

#[cfg(feature = "pdf")]
fn validate_pdf_input_bounds(content: &[u8], limits: PdfExtractionLimits) -> Result<()> {
    if content.len() > limits.max_input_bytes {
        return Err(EngramError::InvalidInput(format!(
            "PDF too large: {} bytes (max: {} bytes)",
            content.len(),
            limits.max_input_bytes
        )));
    }

    Ok(())
}

/// Stub for PDF extraction when the `pdf` feature is disabled
#[cfg(not(feature = "pdf"))]
fn extract_pdf_sections(
    _content: &[u8],
    limits: PdfExtractionLimits,
) -> Result<Vec<DocumentSection>> {
    let PdfExtractionLimits {
        max_input_bytes,
        max_pages,
        max_text_bytes,
    } = limits;
    let _ = (max_input_bytes, max_pages, max_text_bytes);
    Err(EngramError::InvalidInput(
        "PDF extraction requires the 'pdf' feature to be enabled".to_string(),
    ))
}

/// Create chunks from sections with overlap
fn create_chunks(
    sections: Vec<DocumentSection>,
    source_path: &str,
    doc_id: &str,
    config: &IngestConfig,
) -> Vec<DocumentChunk> {
    let mut chunks = Vec::new();
    let mut chunk_index = 0;

    for section in sections {
        let section_chunks = chunk_text(&section.content, config.chunk_size, config.overlap);

        for chunk_content in section_chunks {
            let chunk_hash = compute_hash(chunk_content.as_bytes());

            chunks.push(DocumentChunk {
                content: chunk_content,
                source_path: source_path.to_string(),
                doc_id: doc_id.to_string(),
                chunk_index,
                section_path: section.section_path.clone(),
                page: section.page,
                chunk_hash,
            });

            chunk_index += 1;
        }
    }

    chunks
}

/// Chunk text with overlap
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    // If text is smaller than chunk size, return as single chunk
    if text.chars().count() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();

        // Try to break at word boundary
        let chunk = if end < chars.len() {
            if let Some(last_space) = chunk.rfind(|c: char| c.is_whitespace()) {
                if last_space > chunk_size / 2 {
                    // Only break at word boundary if it's in the second half
                    chunk[..last_space].to_string()
                } else {
                    chunk
                }
            } else {
                chunk
            }
        } else {
            chunk
        };

        let chunk_char_count = chunk.chars().count();
        chunks.push(chunk);

        // Move start with overlap
        if start + chunk_char_count >= chars.len() {
            break;
        }

        let step = chunk_char_count.saturating_sub(overlap);
        start += if step == 0 { chunk_char_count } else { step };
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_chunk_text_small() {
        let text = "Hello world";
        let chunks = chunk_text(text, 1200, 200);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    fn test_chunk_text_with_overlap() {
        let text = "A".repeat(2500);
        let chunks = chunk_text(&text, 1200, 200);
        assert!(chunks.len() >= 2);
        // First chunk should be 1200 chars
        assert!(chunks[0].len() <= 1200);
    }

    #[test]
    fn test_markdown_sections() {
        let md = r#"# Title

Introduction text.

## Section 1

Content for section 1.

### Subsection 1.1

Nested content.

## Section 2

Content for section 2.
"#;
        let sections = extract_markdown_sections(md);
        assert!(sections.len() >= 3);

        // Check preamble/title section
        let title_section = sections.iter().find(|s| s.section_path == "Title");
        assert!(title_section.is_some());

        // Check nested section
        let nested = sections
            .iter()
            .find(|s| s.section_path.contains("Subsection"));
        assert!(nested.is_some());
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash(b"test content");
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), 7 + 64); // "sha256:" + 64 hex chars
    }

    #[test]
    fn test_document_format_detection() {
        assert_eq!(
            DocumentFormat::from_path(Path::new("doc.md")),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(
            DocumentFormat::from_path(Path::new("doc.pdf")),
            Some(DocumentFormat::Pdf)
        );
        assert_eq!(DocumentFormat::from_path(Path::new("doc.txt")), None);
    }

    #[test]
    fn test_ingest_config_default() {
        let config = IngestConfig::default();
        assert_eq!(config.chunk_size, DEFAULT_CHUNK_SIZE);
        assert_eq!(config.overlap, DEFAULT_OVERLAP);
        assert_eq!(config.max_file_size, DEFAULT_MAX_FILE_SIZE);
    }

    #[test]
    fn test_ingest_idempotent() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        fs::write(&file_path, "# Title\n\nHello world.\n").unwrap();

        let storage = Storage::open_in_memory().unwrap();
        let ingestor = DocumentIngestor::new(&storage);

        let first = ingestor
            .ingest_file(&file_path, IngestConfig::default())
            .unwrap();
        assert!(first.chunks_created > 0);
        assert_eq!(first.chunks_skipped, 0);

        let second = ingestor
            .ingest_file(&file_path, IngestConfig::default())
            .unwrap();
        assert_eq!(second.chunks_created, 0);
        assert_eq!(second.chunks_skipped, first.chunks_total);
    }

    #[test]
    fn test_invalid_chunk_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        fs::write(&file_path, "Hello").unwrap();

        let storage = Storage::open_in_memory().unwrap();
        let ingestor = DocumentIngestor::new(&storage);

        let config = IngestConfig {
            chunk_size: 0,
            ..Default::default()
        };

        let err = ingestor.ingest_file(&file_path, config).unwrap_err();
        assert!(err.to_string().contains("chunk_size"));
    }

    #[test]
    fn test_invalid_overlap() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        fs::write(&file_path, "Hello").unwrap();

        let storage = Storage::open_in_memory().unwrap();
        let ingestor = DocumentIngestor::new(&storage);

        let config = IngestConfig {
            chunk_size: 200,
            overlap: 200,
            ..Default::default()
        };

        let err = ingestor.ingest_file(&file_path, config).unwrap_err();
        assert!(err.to_string().contains("overlap"));
    }

    #[test]
    fn test_pdf_empty_is_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty.pdf");
        fs::write(&file_path, b"").unwrap();

        let storage = Storage::open_in_memory().unwrap();
        let ingestor = DocumentIngestor::new(&storage);

        let config = IngestConfig {
            format: Some(DocumentFormat::Pdf),
            ..Default::default()
        };

        let err = ingestor.ingest_file(&file_path, config).unwrap_err();
        assert!(err.to_string().contains("PDF"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_parser_boundary_converts_pre_extraction_panic_to_typed_error() {
        // Given: a third-party parser operation that panics before text extraction.
        let parser_operation = || -> Result<Vec<DocumentSection>> { panic!("parser panic") };

        // When: the operation runs through the parser containment boundary.
        let err = contain_pdf_parser_unwind(parser_operation).unwrap_err();

        // Then: unwinding is converted to a typed invalid-input error.
        assert!(matches!(err, EngramError::InvalidInput(_)));
        assert!(err.to_string().contains("malformed data"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_valid_fixture_extracts_bounded_page_when_pdf_feature_enabled() {
        // Given: a deterministic one-page PDF fixture and the default PDF limits.
        let pdf = include_bytes!("../../tests/fixtures/pdf/valid.pdf");

        // When: the in-worker parser path extracts sections through the bounded path.
        let sections =
            extract_pdf_sections_in_worker(pdf, DEFAULT_MAX_PDF_PAGES, DEFAULT_MAX_PDF_TEXT_BYTES)
                .unwrap();

        // Then: exactly one page is exposed with fixture text and page metadata.
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].page, Some(1));
        assert!(sections[0].content.contains("Engram PDF fixture"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_two_page_fixture_preserves_page_boundaries() {
        // Given: a deterministic two-page PDF fixture and the default PDF limits.
        let pdf = include_bytes!("../../tests/fixtures/pdf/valid-two-pages.pdf");

        // When: the in-worker parser path extracts each real PDF page.
        let sections =
            extract_pdf_sections_in_worker(pdf, DEFAULT_MAX_PDF_PAGES, DEFAULT_MAX_PDF_TEXT_BYTES)
                .unwrap();

        // Then: both page texts and their page metadata remain separate.
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].page, Some(1));
        assert_eq!(sections[1].page, Some(2));
        assert!(sections[0].content.contains("page one"));
        assert!(sections[1].content.contains("page two"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_malformed_fixture_returns_parser_error_when_pdf_feature_enabled() {
        // Given: a deterministic malformed PDF fixture under all budget limits.
        let pdf = include_bytes!("../../tests/fixtures/pdf/malformed.pdf");

        // When: the bounded in-worker parser path attempts extraction.
        let err =
            extract_pdf_sections_in_worker(pdf, DEFAULT_MAX_PDF_PAGES, DEFAULT_MAX_PDF_TEXT_BYTES)
                .unwrap_err();

        // Then: the parser failure is reported without a panic.
        assert!(matches!(err, EngramError::InvalidInput(_)));
        assert!(err.to_string().contains("PDF parsing failed"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_malformed_page_data_returns_error_without_unwinding() {
        // Given: a parseable PDF fixture whose page omits required media-box data.
        let pdf = include_bytes!("../../tests/fixtures/pdf/missing-mediabox.pdf");

        // When: the in-worker extractor reaches malformed page data.
        let result = std::panic::catch_unwind(|| {
            extract_pdf_sections_in_worker(pdf, DEFAULT_MAX_PDF_PAGES, DEFAULT_MAX_PDF_TEXT_BYTES)
        });

        // Then: the library boundary contains the parser panic as a typed input error.
        let err = result.expect("PDF extraction must not unwind").unwrap_err();
        assert!(matches!(err, EngramError::InvalidInput(_)));
        assert!(err.to_string().contains("malformed page data"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_over_budget_fixture_rejects_bytes_before_parser() {
        // Given: a deterministic PDF fixture whose bytes exceed the configured input budget.
        let pdf = include_bytes!("../../tests/fixtures/pdf/over-budget.pdf");
        assert_eq!(pdf.len(), DEFAULT_MAX_FILE_SIZE as usize + 1);

        // When: extraction starts with the documented default byte budget.
        let err = extract_pdf_sections(
            pdf,
            PdfExtractionLimits {
                max_input_bytes: DEFAULT_MAX_FILE_SIZE as usize,
                max_pages: DEFAULT_MAX_PDF_PAGES,
                max_text_bytes: DEFAULT_MAX_PDF_TEXT_BYTES,
            },
        )
        .unwrap_err();

        // Then: the pre-parser byte guard rejects it before pdf-extract runs.
        assert!(matches!(err, EngramError::InvalidInput(_)));
        assert!(err.to_string().contains("PDF too large"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_ingest_rejects_above_absolute_ceiling_with_relaxed_config() {
        // Given: a PDF file one byte above the absolute ceiling and a larger caller limit.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("over-budget.pdf");
        fs::write(
            &file_path,
            include_bytes!("../../tests/fixtures/pdf/over-budget.pdf"),
        )
        .unwrap();
        let storage = Storage::open_in_memory().unwrap();
        let ingestor = DocumentIngestor::new(&storage);
        let config = IngestConfig {
            format: Some(DocumentFormat::Pdf),
            max_file_size: DEFAULT_MAX_FILE_SIZE * 2,
            ..IngestConfig::default()
        };

        // When: ingestion enters through the public file boundary.
        let err = ingestor.ingest_file(&file_path, config).unwrap_err();

        // Then: the fixed ceiling rejects the file before parser work begins.
        assert!(err.to_string().contains("File too large"));
        assert!(err.to_string().contains("10485760"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_absolute_input_ceiling_cannot_be_relaxed_by_config() {
        // Given: a caller-configured limit above the fixed PDF parser ceiling.
        let config = IngestConfig {
            max_file_size: DEFAULT_MAX_FILE_SIZE * 2,
            ..IngestConfig::default()
        };

        // When: parser limits are derived from the caller configuration.
        let limits = PdfExtractionLimits::for_config(&config);

        // Then: the fixed parser ceiling remains authoritative.
        assert_eq!(limits.max_input_bytes, DEFAULT_MAX_FILE_SIZE as usize);
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_rejects_real_page_budget_before_text_extraction() {
        // Given: a valid PDF fixture containing exactly one page over the default budget.
        let pdf = include_bytes!("../../tests/fixtures/pdf/page-limit-plus-one.pdf");
        let document = lopdf::Document::load_mem(pdf).unwrap();
        assert_eq!(document.get_pages().len(), DEFAULT_MAX_PDF_PAGES + 1);

        // When: extraction starts with the default page budget.
        let err =
            extract_pdf_sections_in_worker(pdf, DEFAULT_MAX_PDF_PAGES, DEFAULT_MAX_PDF_TEXT_BYTES)
                .unwrap_err();

        // Then: the parsed page guard rejects it before text extraction.
        assert!(matches!(err, EngramError::InvalidInput(_)));
        assert!(err.to_string().contains("PDF page count"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_rejects_extracted_text_budget_during_parser_output() {
        // Given: a valid PDF fixture whose extracted text is one byte over the default budget.
        let pdf = include_bytes!("../../tests/fixtures/pdf/text-limit-plus-one.pdf");

        // When: page extraction writes through the bounded output buffer.
        let err =
            extract_pdf_sections_in_worker(pdf, DEFAULT_MAX_PDF_PAGES, DEFAULT_MAX_PDF_TEXT_BYTES)
                .unwrap_err();

        // Then: extraction stops with a typed text-limit error.
        assert!(matches!(err, EngramError::InvalidInput(_)));
        assert!(err.to_string().contains("PDF extracted text"));
    }
}
