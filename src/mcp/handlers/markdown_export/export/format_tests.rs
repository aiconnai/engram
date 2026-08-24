use super::format::*;
use crate::storage::queries::compute_content_hash;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_format_memory_markdown_basic() {
    let mem = json!({
        "id": 1,
        "content": "Hello world",
        "memory_type": "note",
        "scope": "user",
        "workspace": "default",
        "tags": "rust,test",
        "importance": 0.8,
        "tier": "permanent",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-02T00:00:00Z",
        "version": 1,
        "metadata": null
    });

    let related_map = HashMap::new();
    let id_to_filename = HashMap::new();
    let md = format_memory_markdown(&mem, false, &related_map, &id_to_filename);

    assert!(md.starts_with("---\n"));
    assert!(md.contains("engram_id: 1"));
    assert!(md.contains("engram_type: note"));
    assert!(md.contains("  - rust"));
    assert!(md.contains("  - test"));
    assert!(md.contains("engram_importance: 0.8"));
    assert!(md.contains("engram_tier: permanent"));
    assert!(md.contains("Hello world"));
}

#[test]
fn test_format_memory_markdown_with_links() {
    let mem = json!({
        "id": 1,
        "content": "Memory one",
        "memory_type": "note",
        "scope": "user",
        "workspace": "default",
        "tags": "",
        "importance": null,
        "tier": "permanent",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": null,
        "version": 1,
        "metadata": null
    });

    let mut related_map = HashMap::new();
    related_map.insert(1i64, vec![(2i64, "related".to_string())]);

    let mut id_to_filename = HashMap::new();
    id_to_filename.insert(2i64, "2-memory-two".to_string());

    let md = format_memory_markdown(&mem, true, &related_map, &id_to_filename);

    assert!(md.contains("## Related"));
    assert!(md.contains("- related [[2-memory-two]]"));
}

#[test]
fn test_format_memory_markdown_no_links_section_when_empty() {
    let mem = json!({
        "id": 1,
        "content": "Solo memory",
        "memory_type": "note",
        "scope": "user",
        "workspace": "default",
        "tags": "",
        "importance": null,
        "tier": "permanent",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": null,
        "version": 1,
        "metadata": null
    });

    let related_map = HashMap::new();
    let id_to_filename = HashMap::new();
    let md = format_memory_markdown(&mem, true, &related_map, &id_to_filename);

    assert!(!md.contains("## Related"));
}

// ── build_index_markdown ───────────────────────────────────────────

#[test]
fn test_build_index_markdown_header() {
    let memories = vec![json!({
        "id": 1,
        "content": "Test note",
        "memory_type": "note",
        "tags": "",
    })];
    let mut type_counts = HashMap::new();
    type_counts.insert("note".to_string(), 1);
    let mut id_to_filename = HashMap::new();
    id_to_filename.insert(1i64, "1-test-note".to_string());

    let index = build_index_markdown("mywork", &memories, &type_counts, &id_to_filename, "flat");

    assert!(index.contains("# mywork -- Engram Export"));
    assert!(index.contains("**Total memories:** 1"));
    assert!(index.contains("**notes/**"));
    assert!(index.contains("| 1 | note | [Test note](1-test-note.md) |  |"));
}

#[test]
fn test_build_index_markdown_type_grouping_links() {
    let memories = vec![json!({
        "id": 1,
        "content": "Architecture decision",
        "memory_type": "decision",
        "tags": "arch",
    })];
    let mut type_counts = HashMap::new();
    type_counts.insert("decision".to_string(), 1);
    let mut id_to_filename = HashMap::new();
    id_to_filename.insert(1i64, "1-architecture-decision".to_string());

    let index = build_index_markdown("mywork", &memories, &type_counts, &id_to_filename, "type");
    assert!(index.contains(
        "| 1 | decision | [Architecture decision](decision/1-architecture-decision.md) | arch |"
    ));
}

// ── memory_export_markdown (requires HandlerContext — skip here) ──
// Integration tests are in tests/markdown_export.rs

// ── RFC 0004 canonical frontmatter ────────────────────────────────

#[test]
fn test_format_memory_markdown_canonical_frontmatter() {
    let mem = json!({
        "id": 42,
        "content": "Authentication is required for every request",
        "memory_type": "note",
        "scope": "user",
        "workspace": "default",
        "tags": "rust,architecture",
        "importance": 0.8,
        "tier": "permanent",
        "created_at": "2026-05-31T02:07:00Z",
        "updated_at": "2026-05-31T04:00:00Z",
        "version": 3,
        "metadata": {"source_session": "sess_abc"}
    });
    let related_map = HashMap::new();
    let id_to_filename = HashMap::new();
    let md = format_memory_markdown(&mem, false, &related_map, &id_to_filename);

    // All 12 engram_ keys must be present
    assert!(md.contains("engram_id: 42"), "missing engram_id");
    assert!(
        md.contains("engram_workspace: default"),
        "missing engram_workspace"
    );
    assert!(md.contains("engram_scope: user"), "missing engram_scope");
    assert!(md.contains("engram_type: note"), "missing engram_type");
    assert!(
        md.contains("engram_created_at:"),
        "missing engram_created_at"
    );
    assert!(
        md.contains("engram_updated_at:"),
        "missing engram_updated_at"
    );
    assert!(
        md.contains("engram_content_hash: sha256:"),
        "missing engram_content_hash"
    );
    assert!(md.contains("engram_version: 3"), "missing engram_version");
    assert!(
        md.contains("engram_importance: 0.8"),
        "missing engram_importance"
    );
    assert!(md.contains("engram_tier: permanent"), "missing engram_tier");
    assert!(
        md.contains("engram_source_session: sess_abc"),
        "missing engram_source_session"
    );
    // Tags must be YAML sequence format
    assert!(md.contains("engram_tags:"), "missing engram_tags key");
    assert!(md.contains("  - rust"), "missing tag sequence item rust");
    assert!(
        md.contains("  - architecture"),
        "missing tag sequence item architecture"
    );
    // Old-style keys must be absent
    assert!(!md.contains("\nid: "), "old id: key must be removed");
    assert!(!md.contains("\ntype: "), "old type: key must be removed");
}

#[test]
fn test_format_memory_markdown_no_source_session() {
    let mem = json!({
        "id": 10,
        "content": "No session here",
        "memory_type": "note",
        "scope": "global",
        "workspace": "work",
        "tags": "",
        "importance": 0.5,
        "tier": "daily",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": null,
        "version": 1,
        "metadata": null
    });
    let related_map = HashMap::new();
    let id_to_filename = HashMap::new();
    let md = format_memory_markdown(&mem, false, &related_map, &id_to_filename);

    assert!(
        !md.contains("engram_source_session"),
        "source_session must be absent when not in metadata"
    );
    assert!(md.contains("engram_id: 10"), "engram_id must be present");
}

// ── content_hash helper ────────────────────────────────────────────

#[test]
fn test_content_hash_prefix() {
    let h = compute_content_hash("hello");
    assert!(h.starts_with("sha256:"), "hash must have sha256: prefix");
    assert_eq!(h.len(), 7 + 64, "sha256 hex is 64 chars");
}

#[test]
fn test_content_hash_deterministic() {
    assert_eq!(compute_content_hash("abc"), compute_content_hash("abc"));
    assert_ne!(compute_content_hash("abc"), compute_content_hash("xyz"));
}
