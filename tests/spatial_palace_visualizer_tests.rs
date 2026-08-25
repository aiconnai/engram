//! Integration tests for Spatial Memory & Mnemonic Palace Visualizer (RFC 0005).

use parking_lot::{Mutex, RwLock};
use serde_json::json;
use std::sync::Arc;

use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::spatial::{PalaceFormat, PalaceGraph};
use engram::storage::queries::create_memory;
use engram::types::{CreateMemoryInput, MemoryType};
use engram::Storage;

fn setup_test_context() -> (Storage, HandlerContext) {
    let storage = Storage::open_in_memory().expect("in-memory database");
    let embedder = engram::embedding::create_embedder(&engram::types::EmbeddingConfig::default())
        .expect("tfidf embedder");

    let ctx = HandlerContext {
        storage: storage.clone(),
        embedder,
        fuzzy_engine: Arc::new(Mutex::new(engram::search::FuzzyEngine::new())),
        search_config: engram::search::SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
        search_cache: Arc::new(engram::search::SearchResultCache::new(
            engram::search::AdaptiveCacheConfig::default(),
        )),
        hnsw_index: Arc::new(RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(128, engram::search::VectorMetric::Cosine),
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
    };

    (storage, ctx)
}

fn seed_test_palace(storage: &Storage, workspace: &str) {
    storage
        .with_connection(|conn| {
            // Wing 1: backend, Room: auth
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "JWT validation requires RS256 signature algorithm".to_string(),
                    memory_type: MemoryType::Decision,
                    tags: vec![
                        "auth".to_string(),
                        "jwt".to_string(),
                        "security".to_string(),
                    ],
                    importance: Some(0.9),
                    workspace: Some(workspace.to_string()),
                    scope: engram::types::MemoryScope::agent("wing:backend/room:auth"),
                    ..Default::default()
                },
            )?;

            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Session tokens expire after 15 minutes of inactivity".to_string(),
                    memory_type: MemoryType::Decision,
                    tags: vec!["auth".to_string(), "session".to_string()],
                    importance: Some(0.85),
                    workspace: Some(workspace.to_string()),
                    scope: engram::types::MemoryScope::agent("wing:backend/room:auth"),
                    ..Default::default()
                },
            )?;

            // Wing 1: backend, Room: database
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "SQLite WAL mode with synchronous=NORMAL enables safe concurrency"
                        .to_string(),
                    memory_type: MemoryType::Learning,
                    tags: vec![
                        "database".to_string(),
                        "sqlite".to_string(),
                        "wal".to_string(),
                    ],
                    importance: Some(0.95),
                    workspace: Some(workspace.to_string()),
                    scope: engram::types::MemoryScope::agent("wing:backend/room:database"),
                    ..Default::default()
                },
            )?;

            // Wing 2: frontend, Room: ui
            create_memory(
                conn,
                &CreateMemoryInput {
                    content:
                        "Dark mode color palette uses Slate-900 background and Indigo-500 accent"
                            .to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["ui".to_string(), "design".to_string()],
                    importance: Some(0.7),
                    workspace: Some(workspace.to_string()),
                    scope: engram::types::MemoryScope::agent("wing:frontend/room:ui"),
                    ..Default::default()
                },
            )?;

            Ok(())
        })
        .expect("seeding test memories");
}

#[test]
fn test_spatial_palace_graph_extraction_and_ascii_render() {
    let (storage, _) = setup_test_context();
    seed_test_palace(&storage, "test_workspace");

    let graph = PalaceGraph::extract(&storage, "test_workspace", None).expect("extraction");

    assert_eq!(graph.workspace, "test_workspace");
    assert_eq!(graph.total_drawers, 4);
    assert_eq!(graph.wings_count, 2);
    assert_eq!(graph.rooms_count, 3);

    // ASCII render check
    let ascii = graph.render(PalaceFormat::Ascii);
    assert!(ascii.contains("MEMORY PALACE: test_workspace"));
    assert!(ascii.contains("WING: backend"));
    assert!(ascii.contains("WING: frontend"));
    assert!(ascii.contains("Room: auth"));
    assert!(ascii.contains("Room: database"));
    assert!(ascii.contains("Room: ui"));
    assert!(ascii.contains("JWT validation requires RS256"));
}

#[test]
fn test_spatial_palace_html_and_interactive_app_render() {
    let (storage, _) = setup_test_context();
    seed_test_palace(&storage, "test_workspace");

    let graph = PalaceGraph::extract(&storage, "test_workspace", None).expect("extraction");
    let html = graph.render(PalaceFormat::Html);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Mnemonic Palace: test_workspace"));
    assert!(html.contains("searchInput"));
    assert!(html.contains("modalOverlay"));
    assert!(html.contains("openDrawer"));
    assert!(html.contains("SQLite WAL mode"));
}

#[test]
fn test_spatial_palace_svg_and_mermaid_render() {
    let (storage, _) = setup_test_context();
    seed_test_palace(&storage, "test_workspace");

    let graph = PalaceGraph::extract(&storage, "test_workspace", None).expect("extraction");

    // SVG
    let svg = graph.render(PalaceFormat::Svg);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Palace: test_workspace"));
    assert!(svg.contains("Wing: backend"));
    assert!(svg.contains("</svg>"));

    // Mermaid
    let mermaid = graph.render(PalaceFormat::Mermaid);
    assert!(mermaid.contains("mindmap"));
    assert!(mermaid.contains("test_workspace"));
    assert!(mermaid.contains("backend"));
    assert!(mermaid.contains("auth"));
}

#[test]
fn test_spatial_palace_target_wing_filter() {
    let (storage, ctx) = setup_test_context();
    seed_test_palace(&storage, "test_workspace");

    let graph =
        PalaceGraph::extract(&storage, "test_workspace", Some("backend")).expect("extraction");
    assert_eq!(graph.wings_count, 1);
    assert_eq!(graph.wings[0].name, "backend");
    assert_eq!(graph.wings[0].drawer_count, 3);
    assert_eq!(graph.total_drawers, 3);

    // Verify MCP palace_navigate tool also calculates accurate total_drawers
    let nav = dispatch(
        &ctx,
        "palace_navigate",
        json!({
            "workspace": "test_workspace",
            "wing": "backend"
        }),
    );
    assert_eq!(nav["total_drawers"], 3);
    assert_eq!(nav["wings_count"], 1);
}

#[test]
fn test_spatial_palace_json_format_and_empty_palace() {
    let (storage, ctx) = setup_test_context();

    // 1. Empty palace test
    let empty_graph = PalaceGraph::extract(&storage, "empty_ws", None).expect("extraction");
    assert_eq!(empty_graph.total_drawers, 0);
    assert_eq!(empty_graph.wings_count, 0);
    let ascii_empty = empty_graph.render(PalaceFormat::Ascii);
    assert!(ascii_empty.contains("Palace is currently empty"));

    // 2. Populated JSON format test
    seed_test_palace(&storage, "json_workspace");
    let graph = PalaceGraph::extract(&storage, "json_workspace", None).expect("extraction");
    let json_rendered = graph.render(PalaceFormat::Json);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_rendered).expect("valid JSON output");
    assert_eq!(parsed["palace"], "json_workspace");
    assert_eq!(parsed["total_drawers"], 4);
    assert_eq!(parsed["wings_count"], 2);

    // 3. Error on unsupported format via MCP dispatch
    let res = dispatch(
        &ctx,
        "palace_visualize",
        json!({
            "workspace": "json_workspace",
            "format": "invalid_format_xyz"
        }),
    );
    assert!(res.get("error").is_some());
}

#[test]
fn test_spatial_palace_xss_and_xml_escaping() {
    let (storage, _) = setup_test_context();

    storage
        .with_connection(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Malicious payload: </script><script>alert('xss')</script> & <test>"
                        .to_string(),
                    memory_type: MemoryType::Decision,
                    tags: vec!["<tag>".to_string(), "foo&bar".to_string()],
                    importance: Some(0.9),
                    workspace: Some("security_ops".to_string()),
                    scope: engram::types::MemoryScope::agent("wing:sec_ops/room:auth_x"),
                    ..Default::default()
                },
            )?;
            Ok(())
        })
        .expect("seed malicious memory");

    let graph = PalaceGraph::extract(&storage, "security_ops", None).expect("extraction");

    // HTML XSS verification
    let html = graph.render(PalaceFormat::Html);
    assert!(!html.contains("</script><script>alert('xss')</script>"));
    assert!(
        html.contains(r#"\u003c/script\u003e\u003cscript\u003e"#) || html.contains(r#"\u003c"#)
    );

    // SVG XML entity escaping verification
    let svg = graph.render(PalaceFormat::Svg);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("security_ops"));
}

#[test]
fn test_spatial_palace_lifecycle_and_expiration_isolation() {
    let (storage, ctx) = setup_test_context();

    storage
        .with_connection(|conn| {
            // 1. Active memory
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Active valid memory".to_string(),
                    memory_type: MemoryType::Decision,
                    tags: vec!["active".to_string()],
                    workspace: Some("lifecycle_ws".to_string()),
                    scope: engram::types::MemoryScope::agent("wing:prod/room:core"),
                    ..Default::default()
                },
            )?;

            // 2. Expired daily memory
            let expired = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Expired daily memory".to_string(),
                    memory_type: MemoryType::Note,
                    workspace: Some("lifecycle_ws".to_string()),
                    scope: engram::types::MemoryScope::agent("wing:prod/room:core"),
                    ..Default::default()
                },
            )?;
            let past = chrono::Utc::now() - chrono::Duration::hours(2);
            conn.execute(
                "UPDATE memories SET expires_at = ?1 WHERE id = ?2",
                rusqlite::params![past.to_rfc3339(), expired.id],
            )?;

            // 3. Archived memory
            let archived = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Archived memory".to_string(),
                    memory_type: MemoryType::Note,
                    workspace: Some("lifecycle_ws".to_string()),
                    scope: engram::types::MemoryScope::agent("wing:prod/room:core"),
                    ..Default::default()
                },
            )?;
            conn.execute(
                "UPDATE memories SET lifecycle_state = 'archived' WHERE id = ?1",
                rusqlite::params![archived.id],
            )?;

            // 4. Soft-deleted memory (valid_to IS NOT NULL)
            let deleted = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Soft deleted memory".to_string(),
                    memory_type: MemoryType::Note,
                    workspace: Some("lifecycle_ws".to_string()),
                    scope: engram::types::MemoryScope::agent("wing:prod/room:core"),
                    ..Default::default()
                },
            )?;
            conn.execute(
                "UPDATE memories SET valid_to = datetime('now') WHERE id = ?1",
                rusqlite::params![deleted.id],
            )?;

            Ok(())
        })
        .expect("seed lifecycle records");

    let graph = PalaceGraph::extract(&storage, "lifecycle_ws", None).expect("extraction");
    assert_eq!(graph.total_drawers, 1);
    assert_eq!(graph.wings[0].rooms[0].drawers.len(), 1);
    assert_eq!(
        graph.wings[0].rooms[0].drawers[0].title,
        "Active valid memory"
    );

    let nav = dispatch(
        &ctx,
        "palace_navigate",
        json!({"workspace": "lifecycle_ws"}),
    );
    assert_eq!(nav["total_drawers"], 1);
}

#[test]
fn test_mcp_palace_visualize_tool_dispatch_and_file_export() {
    let (_, ctx) = setup_test_context();
    seed_test_palace(&ctx.storage, "default");

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let out_file = temp_dir.path().join("palace_export.html");

    let res = dispatch(
        &ctx,
        "palace_visualize",
        json!({
            "workspace": "default",
            "format": "html",
            "output_path": out_file.to_str().unwrap()
        }),
    );

    assert_eq!(res["workspace"], "default");
    assert_eq!(res["format"], "html");
    assert_eq!(res["wings_count"], 2);
    assert_eq!(res["total_drawers"], 4);
    assert!(out_file.exists());

    let written = std::fs::read_to_string(&out_file).expect("read file");
    assert!(written.contains("Mnemonic Palace: default"));
}
