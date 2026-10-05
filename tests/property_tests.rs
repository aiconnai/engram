//! Property-based tests for engram
//!
//! These tests verify invariants that must hold for all inputs:
//! - Normalization is idempotent
//! - Parsers never panic
//! - Bounded operations stay bounded
//!
//! Run with: cargo test --test property_tests

use proptest::prelude::*;

// ============================================================================
// WORKSPACE NORMALIZATION TESTS
// ============================================================================

mod workspace_tests {
    use super::*;
    use engram::types::{normalize_workspace, WorkspaceError, MAX_WORKSPACE_LENGTH};

    proptest! {
        /// Invariant: normalize_workspace never panics on any string input
        #[test]
        fn never_panics(s in ".*") {
            let _ = normalize_workspace(&s);
        }

        /// Invariant: If normalization succeeds, applying it again yields the same result
        #[test]
        fn idempotent_when_valid(s in "[a-z0-9_-]{1,64}") {
            if let Ok(normalized) = normalize_workspace(&s) {
                let twice = normalize_workspace(&normalized);
                prop_assert_eq!(Ok(normalized.clone()), twice);
            }
        }

        /// Invariant: Normalized result only contains allowed characters
        #[test]
        fn output_charset(s in "\\PC{1,100}") {
            if let Ok(normalized) = normalize_workspace(&s) {
                prop_assert!(normalized.chars().all(|c|
                    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
                ));
            }
        }

        /// Invariant: Normalized result respects max length
        #[test]
        fn respects_max_length(s in "\\PC{1,200}") {
            if let Ok(normalized) = normalize_workspace(&s) {
                prop_assert!(normalized.len() <= MAX_WORKSPACE_LENGTH);
            }
        }

        /// Invariant: Empty input always fails
        #[test]
        fn empty_fails(s in "\\s*") {
            let result = normalize_workspace(&s);
            if s.trim().is_empty() {
                prop_assert_eq!(result, Err(WorkspaceError::Empty));
            }
        }

        /// Invariant: Reserved names are rejected
        #[test]
        fn reserved_rejected(prefix in "_{1,5}", suffix in "[a-z0-9]{0,10}") {
            let input = format!("{}{}", prefix, suffix);
            let result = normalize_workspace(&input);
            prop_assert!(result.is_err());
        }
    }
}

// ============================================================================
// ALIAS NORMALIZATION TESTS
// ============================================================================

mod alias_tests {
    use super::*;
    use engram::storage::identity_links::normalize_alias;

    proptest! {
        /// Invariant: normalize_alias never panics on any input
        #[test]
        fn never_panics(s in ".*") {
            let _ = normalize_alias(&s);
        }

        /// Invariant: Normalization is idempotent
        #[test]
        fn idempotent(s in "\\PC{0,100}") {
            let once = normalize_alias(&s);
            let twice = normalize_alias(&once);
            prop_assert_eq!(once, twice);
        }

        /// Invariant: Output is lowercase
        #[test]
        fn lowercase_output(s in "\\PC{1,50}") {
            let normalized = normalize_alias(&s);
            prop_assert!(normalized.chars().all(|c| !c.is_ascii_uppercase()));
        }

        /// Invariant: No leading/trailing whitespace
        #[test]
        fn no_boundary_whitespace(s in "\\PC{1,50}") {
            let normalized = normalize_alias(&s);
            let trimmed = normalized.trim().to_string();
            prop_assert_eq!(normalized, trimmed);
        }

        /// Invariant: No multiple consecutive spaces
        #[test]
        fn no_multiple_spaces(s in "\\PC{1,50}") {
            let normalized = normalize_alias(&s);
            prop_assert!(!normalized.contains("  "));
        }
    }
}

// ============================================================================
// ENTITY EXTRACTION TESTS
// ============================================================================

mod extraction_tests {
    use super::*;
    use engram::intelligence::entities::{EntityExtractionConfig, EntityExtractor};

    proptest! {
        /// Invariant: Extraction never panics on any input
        #[test]
        fn never_panics(s in "\\PC{0,1000}") {
            let extractor = EntityExtractor::new(EntityExtractionConfig::default());
            let _ = extractor.extract(&s);
        }

        /// Invariant: Extraction result length is bounded by input length.
        #[test]
        fn bounded_results(s in "\\PC{0,500}") {
            let extractor = EntityExtractor::new(EntityExtractionConfig::default());
            let result = extractor.extract(&s);
            prop_assert!(result.entities.len() <= s.chars().count());
        }

        /// Invariant: Empty input yields empty results
        #[test]
        fn empty_input_empty_result(s in "\\s*") {
            let extractor = EntityExtractor::new(EntityExtractionConfig::default());
            let result = extractor.extract(&s);
            prop_assert!(result.entities.is_empty());
        }

        /// Invariant: Each entity has non-empty mention text
        #[test]
        fn entities_have_text(s in "@[a-z]{1,10}( @[a-z]{1,10})*") {
            let extractor = EntityExtractor::new(EntityExtractionConfig::default());
            let result = extractor.extract(&s);
            for entity in &result.entities {
                prop_assert!(!entity.text.is_empty());
            }
        }
    }
}

// ============================================================================
// MEMORY TIER TESTS
// ============================================================================

mod tier_tests {
    use super::*;
    use engram::types::MemoryTier;

    proptest! {
        /// Invariant: MemoryTier round-trips through string
        #[test]
        fn tier_roundtrip(tier in prop_oneof![Just(MemoryTier::Permanent), Just(MemoryTier::Daily)]) {
            let s = tier.as_str();
            let parsed: MemoryTier = s.parse().unwrap();
            prop_assert_eq!(tier, parsed);
        }

        /// Invariant: Unknown tier strings fail parsing
        #[test]
        fn unknown_tier_fails(s in "[a-z]{5,20}") {
            if s != "permanent" && s != "daily" {
                let result: Result<MemoryTier, _> = s.parse();
                prop_assert!(result.is_err());
            }
        }
    }
}

// ============================================================================
// SESSION CHUNKING TESTS
// ============================================================================

mod chunking_tests {
    use super::*;
    use chrono::Utc;
    use engram::intelligence::session_indexing::{chunk_conversation, ChunkingConfig, Message};

    fn make_messages(count: usize, content_len: usize) -> Vec<Message> {
        (0..count)
            .map(|i| Message {
                role: if i % 2 == 0 {
                    "user".to_string()
                } else {
                    "assistant".to_string()
                },
                content: "x".repeat(content_len),
                timestamp: Utc::now(),
                id: None,
            })
            .collect()
    }

    proptest! {
        /// Invariant: Chunking never panics
        #[test]
        fn never_panics(msg_count in 0usize..100, content_len in 1usize..500) {
            let messages = make_messages(msg_count, content_len);
            let config = ChunkingConfig::default();
            let _ = chunk_conversation(&messages, &config);
        }

        /// Invariant: Each chunk has at most max_messages
        #[test]
        fn respects_max_messages(msg_count in 1usize..50, max_msgs in 1usize..20) {
            let messages = make_messages(msg_count, 100);
            let config = ChunkingConfig {
                max_messages: max_msgs,
                ..Default::default()
            };
            let chunks = chunk_conversation(&messages, &config);
            for chunk in &chunks {
                prop_assert!(chunk.messages.len() <= max_msgs);
            }
        }

        /// Invariant: Empty input yields empty chunks
        #[test]
        fn empty_input_empty_chunks(_unused: u8) {
            let messages: Vec<Message> = vec![];
            let config = ChunkingConfig::default();
            let chunks = chunk_conversation(&messages, &config);
            prop_assert!(chunks.is_empty());
        }
    }
}

// ============================================================================
// MEMORY TYPE TESTS
// ============================================================================

mod memory_type_tests {
    use super::*;
    use engram::types::MemoryType;

    proptest! {
        /// Invariant: All memory types round-trip
        #[test]
        fn roundtrip(memory_type in prop_oneof![
            Just(MemoryType::Note),
            Just(MemoryType::Todo),
            Just(MemoryType::Issue),
            Just(MemoryType::Decision),
            Just(MemoryType::Preference),
            Just(MemoryType::Learning),
            Just(MemoryType::Context),
            Just(MemoryType::Credential),
            Just(MemoryType::Custom),
            Just(MemoryType::TranscriptChunk),
        ]) {
            let s = memory_type.as_str();
            let parsed: MemoryType = s.parse().unwrap();
            prop_assert_eq!(memory_type, parsed);
        }
    }
}

// ============================================================================
// EDGE TYPE TESTS
// ============================================================================

mod edge_type_tests {
    use super::*;
    use engram::types::EdgeType;

    proptest! {
        /// Invariant: All edge types round-trip
        #[test]
        fn roundtrip(edge_type in prop_oneof![
            Just(EdgeType::RelatedTo),
            Just(EdgeType::DependsOn),
            Just(EdgeType::References),
            Just(EdgeType::Blocks),
            Just(EdgeType::FollowsUp),
            Just(EdgeType::Supersedes),
            Just(EdgeType::Contradicts),
            Just(EdgeType::Implements),
            Just(EdgeType::Extends),
            Just(EdgeType::DerivedFrom),
        ]) {
            let s = edge_type.as_str();
            let parsed: EdgeType = s.parse().unwrap();
            prop_assert_eq!(edge_type, parsed);
        }
    }
}

// ============================================================================
// UNICODE / ADVERSE TEXT TESTS (task C3)
// ============================================================================
//
// Strategies are built from chars (not just `.*`/`\PC`): they deliberately
// include Unicode category C (controls, format chars, bidi overrides/isolates,
// private use), combining marks, ZWJ emoji sequences and the classic
// case-folding troublemakers (ß/ẞ/İ/ς/ﬃ/Kelvin sign) whose lowercase form has a
// DIFFERENT UTF-8 length than the original.
//
// The runs are reproducible: fixed RNG seed + persisted regressions
// (`tests/property_tests.proptest-regressions`, replayed on every run).
// No network, no models, no clock.

mod unicode_adverse_tests {
    use super::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};
    use serde_json::json;
    use std::sync::Arc;

    use engram::text_util::{
        ceil_char_boundary, floor_char_boundary, suffix_bytes, truncate_bytes,
    };

    const SEED: u64 = 0xC3C3_2026_1005;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            rng_seed: RngSeed::Fixed(SEED),
            failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
                "tests/property_tests.proptest-regressions",
            ))),
            ..ProptestConfig::default()
        }
    }

    /// Single chars covering the adverse classes.
    fn adverse_char() -> impl Strategy<Value = char> {
        prop_oneof![
            6 => prop::char::range('a', 'z'),
            2 => Just(' '),
            1 => Just('\n'),
            // Category C: controls, format, bidi, private use, noncharacters.
            3 => prop::sample::select(vec![
                '\u{0}', '\u{7}', '\u{1b}', '\u{7f}', '\u{85}', '\u{200b}', '\u{200c}',
                '\u{200d}', '\u{200e}', '\u{202a}', '\u{202e}', '\u{2066}', '\u{2069}',
                '\u{feff}', '\u{e000}', '\u{fffe}', '\u{10ffff}',
            ]),
            // Combining marks.
            2 => prop::sample::select(vec!['\u{301}', '\u{308}', '\u{327}', '\u{20d7}', '\u{345}']),
            // Case folding changes the UTF-8 length of these.
            4 => prop::sample::select(vec![
                'ß', 'ẞ', 'İ', 'ı', 'ς', 'Σ', 'ǅ', 'ŉ', 'ﬃ', '\u{212a}', 'Å', 'ΐ',
            ]),
            // Emoji building blocks (ZWJ, variation selector, skin tone).
            2 => prop::sample::select(vec!['😀', '👨', '👩', '🏳', '\u{fe0f}', '\u{1f3fb}', '🇧', '🇷']),
            2 => prop::sample::select(vec!['é', '中', 'ñ', 'ø']),
            1 => any::<char>(),
        ]
    }

    /// Whole sequences that must never be cut in half by a *safe* truncation
    /// of at least their own length, and are ZWJ-joined.
    fn adverse_piece() -> impl Strategy<Value = String> {
        prop_oneof![
            10 => adverse_char().prop_map(|c| c.to_string()),
            1 => Just("👨\u{200d}👩\u{200d}👧\u{200d}👦".to_string()),
            1 => Just("🏳\u{fe0f}\u{200d}🌈".to_string()),
            1 => Just("e\u{301}\u{308}\u{327}".to_string()),
            1 => Just("\u{202e}evil\u{202c}".to_string()),
        ]
    }

    fn adverse_string(max_pieces: usize) -> impl Strategy<Value = String> {
        prop::collection::vec(adverse_piece(), 0..max_pieces).prop_map(|v| v.concat())
    }

    fn test_ctx() -> engram::mcp::handlers::HandlerContext {
        use engram::embedding::{create_embedder, EmbeddingCache};
        use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
        let storage = engram::storage::Storage::open_in_memory().expect("in-memory storage");
        let embedder =
            create_embedder(&engram::types::EmbeddingConfig::default()).expect("tfidf embedder");
        engram::mcp::handlers::HandlerContext {
            storage,
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(parking_lot::Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
                engram::search::HnswConfig::new(
                    embedder.dimensions(),
                    engram::search::VectorMetric::Cosine,
                ),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
            principal: None,
        }
    }

    fn add_memory(ctx: &engram::mcp::handlers::HandlerContext, content: &str) -> i64 {
        ctx.storage
            .with_transaction(|conn| {
                let input = engram::types::CreateMemoryInput {
                    content: content.to_string(),
                    importance: Some(0.9),
                    ..Default::default()
                };
                engram::storage::queries::create_memory(conn, &input).map(|m| m.id)
            })
            .expect("create memory")
    }

    // ── shared truncation helpers ────────────────────────────────────────────

    proptest! {
        #![proptest_config(config(512))]

        /// truncate_bytes: valid prefix, within budget, maximal.
        #[test]
        fn truncate_bytes_contract(s in adverse_string(40), n in 0usize..200) {
            let t = truncate_bytes(&s, n);
            prop_assert!(s.starts_with(t));
            prop_assert!(t.len() <= n);
            if t.len() < s.len() {
                let next = s[t.len()..].chars().next().expect("non-empty rest");
                prop_assert!(t.len() + next.len_utf8() > n, "not maximal");
            }
        }

        /// suffix_bytes: valid suffix, within budget, maximal.
        #[test]
        fn suffix_bytes_contract(s in adverse_string(40), n in 0usize..200) {
            let t = suffix_bytes(&s, n);
            prop_assert!(s.ends_with(t));
            prop_assert!(t.len() <= n);
            if t.len() < s.len() {
                let prev = s[..s.len() - t.len()].chars().next_back().expect("non-empty rest");
                prop_assert!(t.len() + prev.len_utf8() > n, "not maximal");
            }
        }

        /// floor/ceil: boundaries bracketing the index, never panicking.
        #[test]
        fn floor_ceil_bracket_index(s in adverse_string(40), i in 0usize..400) {
            let f = floor_char_boundary(&s, i);
            let c = ceil_char_boundary(&s, i);
            prop_assert!(s.is_char_boundary(f) && s.is_char_boundary(c));
            prop_assert!(f <= i.min(s.len()) && i.min(s.len()) <= c);
            prop_assert!(c - f <= 4);
        }
    }

    // ── session chunking: truncate_with_marker contract ──────────────────────

    proptest! {
        #![proptest_config(config(256))]

        /// A message longer than max_chars is cut to at most max_chars BYTES;
        /// head is a prefix and tail a suffix of the original; a shorter message
        /// is kept verbatim. Never panics, including max_chars below the marker size.
        #[test]
        fn chunk_truncation_preserves_contract(
            content in adverse_string(120),
            max_chars in 0usize..160,
        ) {
            use chrono::Utc;
            use engram::intelligence::session_indexing::{chunk_conversation, ChunkingConfig, Message};
            const MARKER: &str = "\n[...truncated...]\n";

            let messages = vec![Message {
                role: "user".to_string(),
                content: content.clone(),
                timestamp: Utc::now(),
                id: Some("m0".to_string()),
            }];
            let cfg = ChunkingConfig { max_chars, ..Default::default() };
            let chunks = chunk_conversation(&messages, &cfg);

            prop_assert_eq!(chunks.len(), 1);
            let kept = &chunks[0].messages[0].content;
            if content.len() <= max_chars {
                prop_assert_eq!(kept, &content);
            } else {
                prop_assert!(kept.len() <= max_chars, "{} > {}", kept.len(), max_chars);
                match kept.split_once(MARKER) {
                    Some((head, tail)) => {
                        prop_assert!(content.starts_with(head));
                        prop_assert!(content.ends_with(tail));
                    }
                    // Budget too small for the marker: plain prefix cut.
                    None => prop_assert!(content.starts_with(kept.as_str())),
                }
            }
        }
    }

    // ── context_budget_check preview ─────────────────────────────────────────

    proptest! {
        #![proptest_config(config(24))]

        #[test]
        fn budget_preview_is_valid_bounded_prefix(content in adverse_string(60)) {
            use engram::intelligence::compression::check_context_budget;
            let result = check_context_budget(&[(1, content.clone())], "gpt-4", None, 1_000_000)
                .expect("budget check");
            let preview = &result.memory_tokens[0].content_preview;
            if content.len() <= 50 {
                prop_assert_eq!(preview, &content);
            } else {
                let body = preview.strip_suffix("...").expect("ellipsis");
                prop_assert!(content.starts_with(body));
                prop_assert!(body.len() <= 50);
                let next = content[body.len()..].chars().next().expect("rest");
                prop_assert!(body.len() + next.len_utf8() > 50, "not maximal");
            }
        }
    }

    // ── BM25 explain highlights (original-text offsets) ──────────────────────

    proptest! {
        #![proptest_config(config(48))]

        /// The highlight is cut from the ORIGINAL content, contains the matched
        /// term case-insensitively, and never panics, whatever case folding does
        /// to byte lengths before the match.
        #[test]
        fn bm25_highlight_is_a_slice_of_the_original(
            before in adverse_string(60),
            after in adverse_string(60),
        ) {
            use engram::search::bm25_search;
            let content = format!("{} target {}", before, after);
            let ctx = test_ctx();
            add_memory(&ctx, &content);

            let results = ctx
                .storage
                .with_connection(|conn| bm25_search(conn, "target", 5, true));
            // An FTS-level error on exotic input is not a panic; only Ok results carry a contract.
            if let Ok(results) = results {
                for r in results {
                    for h in &r.highlights {
                        let body = h.strip_prefix("...").unwrap_or(h);
                        let body = body.strip_suffix("...").unwrap_or(body);
                        prop_assert!(
                            content.contains(body),
                            "highlight {:?} is not a slice of {:?}", h, content
                        );
                        prop_assert!(body.to_lowercase().contains("target") || body.is_empty());
                    }
                }
            }
        }
    }

    // ── MCP tools reached through dispatch ───────────────────────────────────

    proptest! {
        #![proptest_config(config(32))]

        /// memory_search_compact: title is a bounded valid prefix of the first line.
        #[test]
        fn compact_search_title_contract(before in adverse_string(40), after in adverse_string(60)) {
            let ctx = test_ctx();
            let content = format!("{} needle {}", before, after);
            add_memory(&ctx, &content);

            let result = engram::mcp::handlers::dispatch(
                &ctx, "memory_search_compact", json!({"query": "needle"}),
            );
            prop_assert!(result.get("error").is_none(), "{}", result);
            for item in result["results"].as_array().into_iter().flatten() {
                let title = item["title"].as_str().expect("title");
                let first_line = content.lines().next().unwrap_or("");
                let body = title.strip_suffix("...").unwrap_or(title);
                prop_assert!(first_line.starts_with(body), "{:?} vs {:?}", title, first_line);
                prop_assert!(body.len() <= 80);
                if first_line.len() <= 80 && !content.contains('\n') {
                    prop_assert_eq!(title, first_line);
                }
            }
        }

        /// Hex-key tools return a typed error for every non-hex adverse string.
        #[test]
        fn attestation_keys_never_panic(key in adverse_string(40)) {
            let ctx = test_ctx();
            let result = engram::mcp::handlers::dispatch(
                &ctx, "attestation_chain_verify", json!({"verifying_key": key}),
            );
            if !key.is_empty() {
                prop_assert!(result.get("error").is_some(), "key {:?} -> {}", key, result);
            }
        }

        /// session_index never panics for any max_chars / content combination.
        #[test]
        fn session_index_never_panics(content in adverse_string(60), max_chars in 0i64..120) {
            let ctx = test_ctx();
            let result = engram::mcp::handlers::dispatch(
                &ctx,
                "session_index",
                json!({
                    "session_id": "prop",
                    "max_chars": max_chars,
                    "messages": [{"role": "user", "content": content}],
                }),
            );
            prop_assert!(result.get("error").is_none(), "{}", result);
        }
    }

    // ── pure intelligence functions ──────────────────────────────────────────

    proptest! {
        #![proptest_config(config(128))]

        /// Auto-capture only ever returns text taken from the input.
        #[test]
        fn auto_capture_content_comes_from_input(
            pre in adverse_string(20),
            marker in prop::sample::select(vec!["decided to ", "TODO: ", "Note: ", "prefer ", "bug: "]),
            post in adverse_string(120),
        ) {
            use engram::intelligence::auto_capture::AutoCaptureEngine;
            let text = format!("{}{}{}", pre, marker, post);
            let engine = AutoCaptureEngine::with_default_config();
            for c in engine.analyze(&text, "prop") {
                let body = c.content.strip_suffix("...").unwrap_or(&c.content);
                prop_assert!(text.contains(body), "{:?} not in {:?}", c.content, text);
            }
        }

        /// AAAK compression never panics and reports the real byte size.
        #[test]
        fn aaak_compress_never_panics(s in adverse_string(80)) {
            use engram::intelligence::aaak::{AaakCompressor, AaakMode};
            for mode in [AaakMode::Lossless, AaakMode::UltraDense] {
                let res = AaakCompressor::compress(&s, mode);
                prop_assert_eq!(res.original_bytes, s.len());
                let _ = AaakCompressor::decompress(&res.compressed);
            }
        }
    }
}
