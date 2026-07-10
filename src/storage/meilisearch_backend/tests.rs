use super::document::{
    build_memory_from_doc, scope_from_parts, visibility_from_str, visibility_to_str,
    MeilisearchMemory,
};
use super::filters::{
    build_filter_from_list_options, build_filter_from_search_options, build_scope_filter,
    build_tags_filter, build_workspace_filter, escape_filter_value, sort_to_meili,
};
use crate::types::{LifecycleState, MemoryScope, MemoryTier, MemoryType, Visibility};
use crate::{ListOptions, SearchOptions, SortField, SortOrder};
use std::collections::HashMap;

// --- escape_filter_value ---

#[test]
fn test_escape_filter_value_plain() {
    assert_eq!(escape_filter_value("hello"), "hello");
}

#[test]
fn test_escape_filter_value_quotes() {
    assert_eq!(escape_filter_value(r#"say "hi""#), r#"say \"hi\""#);
}

#[test]
fn test_escape_filter_value_backslashes() {
    assert_eq!(escape_filter_value(r"path\to"), r"path\\to");
}

#[test]
fn test_escape_filter_value_mixed() {
    assert_eq!(escape_filter_value(r#"a\"b"#), r#"a\\\"b"#);
}

// --- build_tags_filter ---

#[test]
fn test_build_tags_filter_empty() {
    assert_eq!(build_tags_filter(&[]), None);
}

#[test]
fn test_build_tags_filter_single() {
    let tags = vec!["rust".to_string()];
    assert_eq!(
        build_tags_filter(&tags),
        Some(r#"tags = "rust""#.to_string())
    );
}

#[test]
fn test_build_tags_filter_multiple() {
    let tags = vec!["rust".to_string(), "async".to_string()];
    assert_eq!(
        build_tags_filter(&tags),
        Some(r#"tags = "rust" AND tags = "async""#.to_string())
    );
}

#[test]
fn test_build_tags_filter_special_chars() {
    let tags = vec![r#"say "hi""#.to_string()];
    assert_eq!(
        build_tags_filter(&tags),
        Some(r#"tags = "say \"hi\"""#.to_string())
    );
}

// --- build_workspace_filter ---

#[test]
fn test_build_workspace_filter_empty() {
    assert_eq!(build_workspace_filter(&[]), None);
}

#[test]
fn test_build_workspace_filter_single() {
    let ws = vec!["default".to_string()];
    assert_eq!(
        build_workspace_filter(&ws),
        Some(r#"workspace = "default""#.to_string())
    );
}

#[test]
fn test_build_workspace_filter_multiple() {
    let ws = vec!["proj-a".to_string(), "proj-b".to_string()];
    assert_eq!(
        build_workspace_filter(&ws),
        Some(r#"workspace IN ["proj-a", "proj-b"]"#.to_string())
    );
}

// --- build_scope_filter ---

#[test]
fn test_build_scope_filter_global() {
    let parts = build_scope_filter(&MemoryScope::Global);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], r#"scope = "global""#);
    assert_eq!(parts[1], "scope_id IS NULL");
}

#[test]
fn test_build_scope_filter_user() {
    let parts = build_scope_filter(&MemoryScope::User {
        user_id: "u123".to_string(),
    });
    assert_eq!(parts[0], r#"scope = "user""#);
    assert_eq!(parts[1], r#"scope_id = "u123""#);
}

#[test]
fn test_build_scope_filter_session() {
    let parts = build_scope_filter(&MemoryScope::Session {
        session_id: "s-abc".to_string(),
    });
    assert_eq!(parts[0], r#"scope = "session""#);
    assert_eq!(parts[1], r#"scope_id = "s-abc""#);
}

#[test]
fn test_build_scope_filter_agent() {
    let parts = build_scope_filter(&MemoryScope::Agent {
        agent_id: "agent-1".to_string(),
    });
    assert_eq!(parts[0], r#"scope = "agent""#);
    assert_eq!(parts[1], r#"scope_id = "agent-1""#);
}

// --- build_filter_from_search_options ---

#[test]
fn test_search_filter_defaults() {
    let opts = SearchOptions::default();
    let filter = build_filter_from_search_options(&opts).unwrap();
    // Default excludes transcript_chunk and archived
    let f = filter.unwrap();
    assert!(f.contains(r#"memory_type != "transcript_chunk""#));
    assert!(f.contains(r#"lifecycle_state != "archived""#));
}

#[test]
fn test_search_filter_with_workspace() {
    let opts = SearchOptions {
        workspace: Some("my-proj".to_string()),
        ..Default::default()
    };
    let filter = build_filter_from_search_options(&opts).unwrap().unwrap();
    assert!(filter.contains(r#"workspace = "my-proj""#));
}

#[test]
fn test_search_filter_with_tags_and_type() {
    let opts = SearchOptions {
        tags: Some(vec!["rust".to_string()]),
        memory_type: Some(MemoryType::Note),
        ..Default::default()
    };
    let filter = build_filter_from_search_options(&opts).unwrap().unwrap();
    assert!(filter.contains(r#"memory_type = "note""#));
    assert!(filter.contains(r#"tags = "rust""#));
    // When memory_type is set, transcript_chunk exclusion is NOT added
    assert!(!filter.contains("transcript_chunk"));
}

#[test]
fn test_search_filter_rejects_advanced_filter() {
    let opts = SearchOptions {
        filter: Some(serde_json::json!({"and": []})),
        ..Default::default()
    };
    assert!(build_filter_from_search_options(&opts).is_err());
}

#[test]
fn test_search_filter_include_transcripts() {
    let opts = SearchOptions {
        include_transcripts: true,
        ..Default::default()
    };
    let filter = build_filter_from_search_options(&opts).unwrap();
    let f = filter.unwrap();
    // Should NOT exclude transcript_chunk when include_transcripts is true
    assert!(!f.contains("transcript_chunk"));
}

#[test]
fn test_search_filter_include_archived() {
    let opts = SearchOptions {
        include_archived: true,
        ..Default::default()
    };
    let filter = build_filter_from_search_options(&opts).unwrap();
    let f = filter.unwrap();
    assert!(!f.contains("archived"));
}

// --- build_filter_from_list_options ---

#[test]
fn test_list_filter_defaults() {
    let opts = ListOptions::default();
    let filter = build_filter_from_list_options(&opts).unwrap();
    // Default only excludes archived
    let f = filter.unwrap();
    assert!(f.contains(r#"lifecycle_state != "archived""#));
}

#[test]
fn test_list_filter_with_workspace_and_tier() {
    let opts = ListOptions {
        workspace: Some("eng".to_string()),
        tier: Some(MemoryTier::Permanent),
        ..Default::default()
    };
    let filter = build_filter_from_list_options(&opts).unwrap().unwrap();
    assert!(filter.contains(r#"workspace = "eng""#));
    assert!(filter.contains(r#"tier = "permanent""#));
}

#[test]
fn test_list_filter_rejects_metadata_filter() {
    let opts = ListOptions {
        metadata_filter: Some(HashMap::from([(
            "key".to_string(),
            serde_json::json!("val"),
        )])),
        ..Default::default()
    };
    assert!(build_filter_from_list_options(&opts).is_err());
}

// --- sort_to_meili ---

#[test]
fn test_sort_created_at_desc() {
    assert_eq!(
        sort_to_meili(SortField::CreatedAt, SortOrder::Desc),
        "created_at:desc"
    );
}

#[test]
fn test_sort_importance_asc() {
    assert_eq!(
        sort_to_meili(SortField::Importance, SortOrder::Asc),
        "importance:asc"
    );
}

#[test]
fn test_sort_all_fields() {
    // Verify all sort fields produce valid output
    let fields = [
        SortField::CreatedAt,
        SortField::UpdatedAt,
        SortField::LastAccessedAt,
        SortField::Importance,
        SortField::AccessCount,
    ];
    for field in fields {
        let result = sort_to_meili(field, SortOrder::Desc);
        assert!(result.ends_with(":desc"));
        assert!(!result.starts_with(':'));
    }
}

// --- scope_from_parts ---

#[test]
fn test_scope_from_parts_user() {
    let scope = scope_from_parts("user", Some("u1".to_string()));
    assert!(matches!(scope, MemoryScope::User { user_id } if user_id == "u1"));
}

#[test]
fn test_scope_from_parts_global_fallback() {
    let scope = scope_from_parts("unknown", None);
    assert!(matches!(scope, MemoryScope::Global));
}

#[test]
fn test_scope_from_parts_missing_id_falls_back() {
    // "user" without an ID falls back to Global
    let scope = scope_from_parts("user", None);
    assert!(matches!(scope, MemoryScope::Global));
}

// --- visibility_from_str / visibility_to_str roundtrip ---

#[test]
fn test_visibility_roundtrip() {
    for vis in [Visibility::Private, Visibility::Shared, Visibility::Public] {
        let s = visibility_to_str(vis);
        let back = visibility_from_str(s);
        assert_eq!(back, vis);
    }
}

#[test]
fn test_visibility_unknown_defaults_private() {
    assert_eq!(visibility_from_str("unknown"), Visibility::Private);
}

// --- build_memory_from_doc ---

#[test]
fn test_build_memory_from_doc_roundtrip() {
    let doc = MeilisearchMemory {
        id: 42,
        content: "test memory".to_string(),
        memory_type: "note".to_string(),
        tags: vec!["tag1".to_string()],
        metadata: Some(HashMap::new()),
        created_at: 1700000000,
        updated_at: 1700001000,
        last_accessed_at: Some(1700002000),
        importance: 0.8,
        access_count: 5,
        owner_id: None,
        visibility: "private".to_string(),
        scope: "global".to_string(),
        scope_id: None,
        workspace: "default".to_string(),
        tier: "permanent".to_string(),
        version: 1,
        has_embedding: true,
        expires_at: None,
        content_hash: Some("abc123".to_string()),
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        procedure_success_count: 0,
        procedure_failure_count: 0,
        summary_of_id: None,
        lifecycle_state: "active".to_string(),
    };

    let memory = build_memory_from_doc(doc);

    assert_eq!(memory.id, 42);
    assert_eq!(memory.content, "test memory");
    assert_eq!(memory.memory_type, MemoryType::Note);
    assert_eq!(memory.tags, vec!["tag1".to_string()]);
    assert_eq!(memory.importance, 0.8);
    assert_eq!(memory.access_count, 5);
    assert_eq!(memory.workspace, "default");
    assert_eq!(memory.tier, MemoryTier::Permanent);
    assert!(memory.has_embedding);
    assert_eq!(memory.content_hash, Some("abc123".to_string()));
    assert!(matches!(memory.scope, MemoryScope::Global));
    assert_eq!(memory.visibility, Visibility::Private);
    assert_eq!(memory.lifecycle_state, LifecycleState::Active);
}
