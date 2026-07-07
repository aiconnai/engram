use super::format::*;
use serde_json::json;

#[test]
fn test_sanitize_filename_basic() {
    assert_eq!(sanitize_filename("Hello World!"), "hello-world");
}

#[test]
fn test_sanitize_filename_preserves_alphanumeric() {
    assert_eq!(sanitize_filename("my-note_123"), "my-note_123");
}

#[test]
fn test_sanitize_filename_empty_input() {
    assert_eq!(sanitize_filename(""), "untitled");
}

#[test]
fn test_sanitize_filename_all_special_chars() {
    assert_eq!(sanitize_filename("!!!@@@"), "untitled");
}

#[test]
fn test_sanitize_filename_truncates_long_input() {
    let long_input = "a".repeat(100);
    let result = sanitize_filename(&long_input);
    assert!(result.len() <= 40);
}

#[test]
fn test_sanitize_filename_trims_dashes() {
    assert_eq!(sanitize_filename("  hello  "), "hello");
}

// ── pluralize_type ─────────────────────────────────────────────────

#[test]
fn test_pluralize_type_note() {
    assert_eq!(pluralize_type("note"), "notes");
}

#[test]
fn test_pluralize_type_todo() {
    assert_eq!(pluralize_type("todo"), "todos");
}

#[test]
fn test_pluralize_type_summary() {
    assert_eq!(pluralize_type("summary"), "summaries");
}

#[test]
fn test_pluralize_type_already_plural() {
    assert_eq!(pluralize_type("issues"), "issues");
}

// ── parse_tags ─────────────────────────────────────────────────────

#[test]
fn test_parse_tags_empty() {
    assert!(parse_tags("").is_empty());
}

#[test]
fn test_parse_tags_comma_separated() {
    assert_eq!(
        parse_tags("rust, memory, test"),
        vec!["rust", "memory", "test"]
    );
}

#[test]
fn test_parse_tags_json_array() {
    assert_eq!(parse_tags(r#"["alpha","beta"]"#), vec!["alpha", "beta"]);
}

// ── format_memory_markdown ─────────────────────────────────────────

#[test]
fn test_grouping_flat() {
    let mem = json!({
        "memory_type": "note",
        "created_at": "2026-05-31T02:07:00Z",
        "workspace": "default",
        "tags": "rust,arch"
    });
    let subdir = compute_file_subdir("flat", &mem);
    assert!(subdir.is_none(), "flat mode: no subdir");
}

#[test]
fn test_grouping_by_type() {
    let mem = json!({
        "memory_type": "decision",
        "created_at": "2026-05-31T02:07:00Z",
        "workspace": "default",
        "tags": ""
    });
    let subdir = compute_file_subdir("type", &mem);
    assert_eq!(subdir.as_deref(), Some("decision"));
}

#[test]
fn test_grouping_by_day() {
    let mem = json!({
        "memory_type": "note",
        "created_at": "2026-05-31T02:07:00Z",
        "workspace": "default",
        "tags": ""
    });
    let subdir = compute_file_subdir("day", &mem);
    assert_eq!(subdir.as_deref(), Some("2026-05-31"));
}

#[test]
fn test_grouping_by_workspace() {
    let mem = json!({
        "memory_type": "note",
        "created_at": "2026-05-31T02:07:00Z",
        "workspace": "myproject",
        "tags": ""
    });
    let subdir = compute_file_subdir("workspace", &mem);
    assert_eq!(subdir.as_deref(), Some("myproject"));
}

#[test]
fn test_grouping_by_entity_first_tag() {
    let mem = json!({
        "memory_type": "note",
        "created_at": "2026-05-31T02:07:00Z",
        "workspace": "default",
        "tags": "rust,arch"
    });
    let subdir = compute_file_subdir("entity", &mem);
    // sorted alphabetically: "arch" < "rust"
    assert_eq!(subdir.as_deref(), Some("arch"));
}

#[test]
fn test_grouping_by_entity_no_tags() {
    let mem = json!({
        "memory_type": "note",
        "created_at": "2026-05-31T02:07:00Z",
        "workspace": "default",
        "tags": ""
    });
    let subdir = compute_file_subdir("entity", &mem);
    assert_eq!(subdir.as_deref(), Some("untagged"));
}
