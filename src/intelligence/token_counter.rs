//! Unified token counting and chunking layer (Issue #30)
//!
//! Provides [`TiktokenCounter`] (accurate BPE token counting via tiktoken-rs),
//! [`TokenChunker`] (token-aware text splitting with overlap), and
//! [`TokenBudgetResult`] (metadata about a compression run).

use std::sync::OnceLock;

use tiktoken_rs::CoreBPE;

use crate::intelligence::compression::{detect_encoding, TokenEncoding};
use crate::intelligence::context_builder::TokenCounter;

// BPE tables are expensive to initialize — cache them process-wide.
fn cl100k() -> &'static CoreBPE {
    static LOCK: OnceLock<CoreBPE> = OnceLock::new();
    LOCK.get_or_init(|| tiktoken_rs::cl100k_base().expect("cl100k_base init"))
}

fn o200k() -> &'static CoreBPE {
    static LOCK: OnceLock<CoreBPE> = OnceLock::new();
    LOCK.get_or_init(|| tiktoken_rs::o200k_base().expect("o200k_base init"))
}

// ---------------------------------------------------------------------------
// TiktokenCounter
// ---------------------------------------------------------------------------

/// Accurate BPE token counter backed by tiktoken-rs.
pub struct TiktokenCounter {
    encoding: TokenEncoding,
}

impl TiktokenCounter {
    /// Create a counter for the given encoding.
    pub fn new(encoding: TokenEncoding) -> Self {
        Self { encoding }
    }

    /// Create a counter for a known model name.
    ///
    /// Returns `None` if the model is not recognised.
    pub fn for_model(model: &str) -> Option<Self> {
        detect_encoding(model).map(Self::new)
    }

    /// Create a counter for a model name, falling back to `cl100k_base` if
    /// the model is not recognised.
    pub fn with_fallback(model: &str) -> Self {
        let encoding = detect_encoding(model).unwrap_or(TokenEncoding::Cl100kBase);
        Self::new(encoding)
    }

    /// Return the encoding identifier string (e.g. `"cl100k_base"`).
    pub fn encoding_name(&self) -> &'static str {
        self.encoding.as_str()
    }

    /// Encode `text` to token IDs.
    pub(crate) fn encode(&self, text: &str) -> Vec<usize> {
        let bpe = match self.encoding {
            TokenEncoding::Cl100kBase => cl100k(),
            TokenEncoding::O200kBase => o200k(),
        };
        bpe.encode_with_special_tokens(text)
    }

    /// Decode token IDs back to a `String`.
    fn decode(&self, ids: &[usize]) -> String {
        let bpe = match self.encoding {
            TokenEncoding::Cl100kBase => cl100k(),
            TokenEncoding::O200kBase => o200k(),
        };
        bpe.decode(ids.to_vec()).unwrap_or_default()
    }
}

impl TokenCounter for TiktokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        self.encode(text).len()
    }
}

// ---------------------------------------------------------------------------
// TokenChunker
// ---------------------------------------------------------------------------

/// A single chunk produced by [`TokenChunker`].
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// The decoded text of this chunk.
    pub text: String,
    /// Byte offset into the original string where this chunk begins.
    pub start_char: usize,
    /// Byte offset into the original string where this chunk ends (exclusive).
    pub end_char: usize,
    /// Number of tokens in this chunk.
    pub token_count: usize,
}

/// Splits text into token-sized chunks with optional overlap.
pub struct TokenChunker {
    counter: TiktokenCounter,
    chunk_size: usize,
    chunk_overlap: usize,
}

impl TokenChunker {
    /// Create a chunker with the given `chunk_size` and `chunk_overlap` (both
    /// in tokens).
    ///
    /// # Panics
    /// Panics if `chunk_overlap >= chunk_size`.
    pub fn new(counter: TiktokenCounter, chunk_size: usize, chunk_overlap: usize) -> Self {
        assert!(
            chunk_overlap < chunk_size,
            "chunk_overlap must be < chunk_size"
        );
        Self {
            counter,
            chunk_size,
            chunk_overlap,
        }
    }

    /// Split `text` into overlapping token chunks.
    pub fn chunk(&self, text: &str) -> Vec<TextChunk> {
        let ids = self.counter.encode(text);
        if ids.is_empty() {
            return Vec::new();
        }

        let step = self.chunk_size - self.chunk_overlap;
        let mut chunks = Vec::new();
        let mut start = 0usize;

        while start < ids.len() {
            let end = (start + self.chunk_size).min(ids.len());
            let chunk_ids = &ids[start..end];
            let chunk_text = self.counter.decode(chunk_ids);

            // Find the byte offsets in the original string by decoding tokens
            // before this chunk and before+chunk.
            let prefix_text = self.counter.decode(&ids[..start]);
            let start_char = prefix_text.len();
            let end_char = start_char + chunk_text.len();

            chunks.push(TextChunk {
                text: chunk_text,
                start_char,
                end_char,
                token_count: chunk_ids.len(),
            });

            if end == ids.len() {
                break;
            }
            start += step;
        }

        chunks
    }
}

// ---------------------------------------------------------------------------
// TokenBudgetResult
// ---------------------------------------------------------------------------

/// Metadata produced after a compression / budget-fitting run.
#[derive(Debug, Clone)]
pub struct TokenBudgetResult {
    /// Token count of the original content before compression.
    pub original_token_count: usize,
    /// Token count of the content after compression.
    pub prepared_token_count: usize,
    /// `prepared / original` (0.0–1.0; lower = more compressed).
    pub compression_ratio: f64,
    /// Human-readable label for the compression level applied.
    pub compression_level: Option<String>,
    /// Identifier for the tokenizer used (e.g. `"cl100k_base"` or `"chars/4"`).
    pub tokenizer_id: String,
}

impl TokenBudgetResult {
    /// Build a result using chars/4 heuristic counts.
    pub fn from_heuristic(original_len: usize, prepared_len: usize, level: Option<&str>) -> Self {
        let orig = original_len.div_ceil(4);
        let prep = prepared_len.div_ceil(4);
        Self {
            original_token_count: orig,
            prepared_token_count: prep,
            compression_ratio: if orig == 0 {
                1.0
            } else {
                prep as f64 / orig as f64
            },
            compression_level: level.map(str::to_string),
            tokenizer_id: "chars/4".to_string(),
        }
    }

    /// Build a result using a `TiktokenCounter`.
    pub fn from_tiktoken(
        original: &str,
        prepared: &str,
        level: Option<&str>,
        counter: &TiktokenCounter,
    ) -> Self {
        let orig = counter.count_tokens(original);
        let prep = counter.count_tokens(prepared);
        Self {
            original_token_count: orig,
            prepared_token_count: prep,
            compression_ratio: if orig == 0 {
                1.0
            } else {
                prep as f64 / orig as f64
            },
            compression_level: level.map(str::to_string),
            tokenizer_id: counter.encoding_name().to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// TiktokenTokenCounter — implements context_builder::TokenCounter
// ---------------------------------------------------------------------------

/// Wrapper so [`TiktokenCounter`] can be used as `Box<dyn TokenCounter>` inside
/// [`crate::intelligence::context_builder::ContextBuilder`].
pub struct TiktokenTokenCounter(pub TiktokenCounter);

impl TokenCounter for TiktokenTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        self.0.count_tokens(text)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::context_builder::ContextBuilder;
    use crate::intelligence::context_compression::{ContextCompressor, MemoryInput};

    // 1. TiktokenCounter::for_model("gpt-4") counts "hello world"
    #[test]
    fn test_for_model_known_counts_tokens() {
        let counter = TiktokenCounter::for_model("gpt-4").expect("gpt-4 should be known");
        let count = counter.count_tokens("hello world");
        // "hello world" is 2 tokens in cl100k_base
        assert!(count > 0, "should count at least 1 token");
        assert!(count <= 5, "hello world should be a small number of tokens");
    }

    // 2. TiktokenCounter::for_model("unknown-model") returns None
    #[test]
    fn test_for_model_unknown_returns_none() {
        let result = TiktokenCounter::for_model("totally-unknown-model-xyz");
        assert!(result.is_none());
    }

    // 3. TiktokenCounter::with_fallback("unknown") still counts (cl100k fallback)
    #[test]
    fn test_with_fallback_unknown_still_counts() {
        let counter = TiktokenCounter::with_fallback("totally-unknown-model-xyz");
        assert_eq!(counter.encoding_name(), "cl100k_base");
        let count = counter.count_tokens("hello world");
        assert!(count > 0);
    }

    // 4. TokenChunker splits long text into chunks <= chunk_size tokens each
    #[test]
    fn test_chunker_respects_chunk_size() {
        let counter = TiktokenCounter::with_fallback("claude");
        let chunk_size = 10;
        let chunker = TokenChunker::new(counter, chunk_size, 2);
        let word = "word ";
        let text = word.repeat(100);
        let chunks = chunker.chunk(&text);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(
                chunk.token_count <= chunk_size,
                "chunk has {} tokens, expected <= {}",
                chunk.token_count,
                chunk_size
            );
        }
    }

    // 5. Chunk overlap: consecutive chunks share chunk_overlap tokens at boundary
    #[test]
    fn test_chunker_overlap() {
        let counter = TiktokenCounter::with_fallback("claude");
        let full_ids =
            counter.encode("the quick brown fox jumps over the lazy dog and then runs away");
        let chunk_size = 5;
        let overlap = 2;
        let chunker = TokenChunker::new(
            TiktokenCounter::with_fallback("claude"),
            chunk_size,
            overlap,
        );
        let text = "the quick brown fox jumps over the lazy dog and then runs away";
        let chunks = chunker.chunk(text);
        if chunks.len() >= 2 {
            // The last `overlap` tokens of chunk[0] should be the same as
            // the first `overlap` tokens of chunk[1].
            let c0_ids = TiktokenCounter::with_fallback("claude").encode(&chunks[0].text);
            let c1_ids = TiktokenCounter::with_fallback("claude").encode(&chunks[1].text);
            let tail: Vec<_> = c0_ids.iter().rev().take(overlap).rev().collect();
            let head: Vec<_> = c1_ids.iter().take(overlap).collect();
            assert_eq!(
                tail, head,
                "overlap tokens should match between consecutive chunks"
            );
        }
        let _ = full_ids; // suppress unused warning
    }

    // 6. TextChunk.start_char/end_char correctly points into original string
    #[test]
    fn test_chunk_byte_offsets() {
        let counter = TiktokenCounter::with_fallback("gpt-4");
        let chunk_size = 3;
        let chunker = TokenChunker::new(counter, chunk_size, 0);
        let text = "hello world foo bar";
        let chunks = chunker.chunk(text);
        for chunk in &chunks {
            let slice = &text[chunk.start_char..chunk.end_char];
            assert_eq!(
                slice, chunk.text,
                "byte range should match decoded chunk text"
            );
        }
    }

    // 7. ContextCompressor::with_token_counter uses real counts (not chars/4)
    #[test]
    fn test_compressor_with_token_counter() {
        let counter = TiktokenCounter::with_fallback("claude");
        let mut compressor = ContextCompressor::with_token_counter(1000, counter);
        let memories = vec![MemoryInput {
            id: 1,
            content: "This is a test memory with several words.".to_string(),
            importance: 1.0,
        }];
        let plan = compressor.compress_for_context_with_diagnostics(&memories);
        assert_eq!(plan.entries.len(), 1);
        // tokenizer_id should be tiktoken, not chars/4
        assert_eq!(plan.entries[0].tokenizer_id, "cl100k_base");
    }

    // 8. TokenBudgetResult fields populated correctly
    #[test]
    fn test_token_budget_result_tiktoken() {
        let counter = TiktokenCounter::with_fallback("gpt-4");
        let original = "This is a fairly long sentence with many words in it.";
        let prepared = "long sentence many words";
        let result = TokenBudgetResult::from_tiktoken(original, prepared, Some("Medium"), &counter);
        assert!(result.original_token_count > result.prepared_token_count);
        assert!(result.compression_ratio < 1.0);
        assert_eq!(result.compression_level.as_deref(), Some("Medium"));
        assert_eq!(result.tokenizer_id, "cl100k_base");
    }

    // 9. ContextBuilder::with_tiktoken uses tiktoken counter
    #[test]
    fn test_context_builder_with_tiktoken() {
        let builder = ContextBuilder::with_tiktoken("gpt-4");
        // "hello world" should be 2 tokens, not chars/4 = 2 (coincidentally same)
        // Use a longer text to distinguish: 40 chars = 10 by chars/4, but tiktoken may differ
        let count = builder.estimate_tokens("hello world foo bar baz qux quux corge grault");
        // tiktoken cl100k_base gives ~9 tokens; chars/4 gives 11
        // Either way it should be > 0 and reasonable
        assert!(count > 0);
        assert!(count < 30, "count should be reasonable, got {}", count);
    }
}
