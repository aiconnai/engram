//! Fuzz target: `engram::intelligence::entity_extraction::extract_entities`.
//!
//! Public API under test (no database connection, so no SQLite/FFI path):
//! `extract_entities(content, &ExtractionConfig, None)`. The module documents
//! "never panics on any input" and bounded output; this target checks both and
//! the structural invariants of the result.
#![no_main]

use engram::intelligence::entity_extraction::{extract_entities, ExtractionConfig};
use libfuzzer_sys::fuzz_target;

// Input layout: byte 0 = feature flags, byte 1 = `max_entities`, rest = UTF-8
// text (invalid sequences are replaced, so every input exercises the parser).
fuzz_target!(|data: &[u8]| {
    let [flags, max_entities, rest @ ..] = data else {
        return;
    };
    let (flags, max_entities) = (*flags, *max_entities);
    let content = String::from_utf8_lossy(rest);
    let content = content.as_ref();
    let config = ExtractionConfig {
        extract_mentions: flags & 0b0000_0001 != 0,
        extract_emails: flags & 0b0000_0010 != 0,
        extract_urls: flags & 0b0000_0100 != 0,
        extract_names: flags & 0b0000_1000 != 0,
        lookup_aliases: flags & 0b0001_0000 != 0,
        min_confidence: if flags & 0b0010_0000 != 0 { 0.0 } else { 0.3 },
        max_entities: usize::from(max_entities),
    };

    let result = extract_entities(content, &config, None);
    let trimmed = content.trim();

    assert!(result.entities.len() <= config.max_entities);
    assert_eq!(
        result.resolved_count, 0,
        "no connection => nothing resolves"
    );
    assert!(result.entities.len() <= result.total_mentions);

    let mut last_position = 0usize;
    let mut seen = std::collections::HashSet::new();
    for entity in &result.entities {
        assert!(entity.count >= 1);
        assert!(entity.confidence >= config.min_confidence);
        assert!(entity.position <= trimmed.len());
        assert!(
            trimmed.is_char_boundary(entity.position),
            "position must be a char boundary of the (trimmed) input"
        );
        assert!(entity.position >= last_position, "sorted by position");
        last_position = entity.position;
        assert!(seen.insert(entity.normalized.clone()), "deduplicated");
        assert!(entity.resolved_id.is_none());
    }
});
