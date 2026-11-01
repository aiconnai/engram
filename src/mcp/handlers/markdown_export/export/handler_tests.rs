use super::handler::memory_export_markdown;
use crate::storage::queries::create_memory;
use crate::storage::Storage;
use crate::types::{CreateMemoryInput, MemoryType};
use serde_json::json;
use std::sync::Arc;

pub(super) fn ctx() -> crate::mcp::handlers::HandlerContext {
    crate::mcp::handlers::HandlerContext {
        storage: Storage::open_in_memory().expect("in-memory storage"),
        embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(128)),
        fuzzy_engine: Arc::new(parking_lot::Mutex::new(crate::search::FuzzyEngine::new())),
        search_config: crate::search::SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
        search_cache: Arc::new(crate::search::SearchResultCache::new(
            crate::search::AdaptiveCacheConfig::default(),
        )),
        hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
            crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
        ))),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 300,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        ),
        progress_reporter: None,
        principal: None,
    }
}

fn make_memory(c: &crate::mcp::handlers::HandlerContext) {
    let input = CreateMemoryInput {
        content: "Markdown export schema smoke".to_string(),
        memory_type: MemoryType::Note,
        workspace: Some("markdown_export_schema".to_string()),
        ..Default::default()
    };
    c.storage
        .with_transaction(|conn| create_memory(conn, &input))
        .expect("create memory");
}

#[test]
fn export_markdown_uses_current_scope_schema() {
    let c = ctx();
    make_memory(&c);
    let dir = tempfile::tempdir().expect("tempdir");

    let result = memory_export_markdown(
        &c,
        json!({
            "workspace": "markdown_export_schema",
            "output_dir": dir.path().to_str().unwrap(),
            "include_links": false
        }),
    );

    assert!(result.get("error").is_none(), "result={result}");
    assert_eq!(result["memories_exported"].as_u64(), Some(1));
    assert!(dir.path().join("index.md").exists());
}

pub(super) fn p0_memory(
    c: &crate::mcp::handlers::HandlerContext,
    content: &str,
    tags: Vec<String>,
) -> i64 {
    c.storage
        .with_transaction(|conn| {
            Ok(create_memory(
                conn,
                &CreateMemoryInput {
                    content: content.into(),
                    memory_type: MemoryType::Note,
                    workspace: Some("markdown_export_schema".into()),
                    tags,
                    ..Default::default()
                },
            )?
            .id)
        })
        .unwrap()
}

pub(super) fn p0_export(
    c: &crate::mcp::handlers::HandlerContext,
    path: &std::path::Path,
    group: &str,
) -> serde_json::Value {
    memory_export_markdown(
        c,
        json!({
            "workspace":"markdown_export_schema", "output_dir":path, "group":group
        }),
    )
}

#[test]
fn p0_traversal_tag_is_rejected_before_writing() {
    let c = ctx();
    p0_memory(&c, "Private note", vec!["../../escaped".into()]);
    let fixture = tempfile::tempdir().unwrap();
    let base = fixture.path().join("base");
    std::fs::create_dir(&base).unwrap();
    let out = base.join("export");
    let r = p0_export(&c, &out, "entity");
    assert!(r.get("error").is_some(), "{r}");
    assert!(!out.exists(), "invalid batch must not create output");
    assert!(!fixture.path().join("escaped").exists());
}

#[cfg(unix)]
#[test]
fn p0_symlink_directory_is_rejected() {
    let c = ctx();
    p0_memory(&c, "Private note", vec!["topic".into()]);
    let f = tempfile::tempdir().unwrap();
    let out = f.path().join("export");
    let outside = f.path().join("outside");
    std::fs::create_dir(&out).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, out.join("topic")).unwrap();
    let r = p0_export(&c, &out, "entity");
    assert!(r.get("error").is_some(), "{r}");
    assert_eq!(std::fs::read_dir(out).unwrap().count(), 1);
    assert_eq!(std::fs::read_dir(outside).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn p0_symlink_file_is_not_followed() {
    let c = ctx();
    let id = p0_memory(&c, "Private note", vec![]);
    let f = tempfile::tempdir().unwrap();
    let out = f.path().join("export");
    std::fs::create_dir(&out).unwrap();
    let sentinel = f.path().join("sentinel");
    std::fs::write(&sentinel, b"keep").unwrap();
    std::os::unix::fs::symlink(&sentinel, out.join(format!("{id}-private-note.md"))).unwrap();
    let r = p0_export(&c, &out, "flat");
    assert!(r.get("error").is_some(), "{r}");
    assert_eq!(std::fs::read(sentinel).unwrap(), b"keep");
    assert!(!out.join("index.md").exists());
}

#[test]
fn p0_dispatch_read_only_cannot_create_files() {
    let c = ctx();
    make_memory(&c);
    let f = tempfile::tempdir().unwrap();
    let out = f.path().join("export");
    let r = crate::mcp::handlers::dispatch(
        &c,
        "memory_export_markdown",
        json!({
            "workspace":"markdown_export_schema", "output_dir":out,"_permission_mode":"read_only"
        }),
    );
    assert_eq!(r["error"]["code"], "permission_denied", "{r}");
    assert!(
        matches!(
            r["error"]["required_mode"].as_str(),
            Some("scoped_write" | "maintenance")
        ),
        "{r}"
    );
    assert!(!out.exists());
}

#[test]
fn p0_invalid_row_fails_before_writing_valid_rows() {
    let c = ctx();
    make_memory(&c);
    let id = p0_memory(&c, "Bad row", vec![]);
    c.storage
        .with_connection(|conn| {
            conn.execute("UPDATE memories SET content=x'01' WHERE id=?1", [id])?;
            Ok(())
        })
        .unwrap();
    let f = tempfile::tempdir().unwrap();
    let out = f.path().join("export");
    let r = p0_export(&c, &out, "flat");
    assert!(r.get("error").is_some(), "{r}");
    assert!(!out.exists());
}

#[test]
fn p0_relation_read_error_is_not_a_successful_export() {
    let c = ctx();
    make_memory(&c);
    c.storage
        .with_connection(|conn| {
            conn.execute_batch("DROP TABLE crossrefs")?;
            Ok(())
        })
        .unwrap();
    let f = tempfile::tempdir().unwrap();
    let out = f.path().join("export");
    let r = p0_export(&c, &out, "flat");
    assert!(r.get("error").is_some(), "{r}");
    assert!(!out.exists());
}

#[test]
fn p0_expired_memory_is_absent_from_files_index_and_links() {
    let c = ctx();
    let live = p0_memory(&c, "Live note", vec![]);
    let expired = p0_memory(&c, "Expired private note", vec![]);
    let future = p0_memory(&c, "Future Daily note", vec![]);
    c.storage.with_connection(|conn| {
        conn.execute("UPDATE memories SET tier='daily',expires_at='2000-01-01T00:00:00+00:00' WHERE id=?1",[expired])?;
        conn.execute("UPDATE memories SET tier='daily',expires_at='2999-01-01T00:00:00+00:00' WHERE id=?1",[future])?;
        conn.execute("INSERT INTO crossrefs(from_id,to_id,edge_type,score) VALUES(?1,?2,'supersedes',1.0)",[live,expired])?;
        Ok(())
    }).unwrap();
    let f = tempfile::tempdir().unwrap();
    let r = p0_export(&c, f.path(), "flat");
    assert!(r.get("error").is_none(), "{r}");
    assert_eq!(r["memories_exported"], 2, "{r}");
    for entry in std::fs::read_dir(f.path()).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!text.contains("Expired private note"), "{text}");
        assert!(!text.contains(&format!("[[{expired}-")), "{text}");
        assert!(!text.contains(&format!("[[memory-{expired}")), "{text}");
    }
}

#[test]
fn p0_incoming_edge_is_not_rendered_as_outgoing() {
    let c = ctx();
    let a = p0_memory(&c, "New decision", vec![]);
    let b = p0_memory(&c, "Old decision", vec![]);
    c.storage
        .with_connection(|conn| {
            conn.execute(
                "INSERT INTO crossrefs(from_id,to_id,edge_type,score) VALUES(?1,?2,'supersedes',1.0)",
                [a, b],
            )?;
            Ok(())
        })
        .unwrap();
    let f = tempfile::tempdir().unwrap();
    let r = p0_export(&c, f.path(), "flat");
    assert!(r.get("error").is_none(), "{r}");
    let incoming = std::fs::read_to_string(f.path().join(format!("{b}-old-decision.md"))).unwrap();
    assert!(
        !incoming.contains(&format!("- supersedes [[{a}-new-decision]]")),
        "{incoming}"
    );
    assert!(
        incoming.contains(&format!("- incoming supersedes [[{a}-new-decision]]")),
        "{incoming}"
    );
}
