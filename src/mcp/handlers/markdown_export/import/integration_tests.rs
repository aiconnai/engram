use super::handler::memory_import_markdown;
use super::test_support::*;
use serde_json::json;

#[test]
fn test_import_in_sync_when_body_matches() {
    let c = ctx();
    let id = make_memory(&c, "hello world", &["alpha"]);
    let dir = tempfile::tempdir().unwrap();
    write_md(
        dir.path(),
        "m.md",
        Some(id),
        1,
        &["alpha"],
        0.5,
        "hello world",
        &[],
    );

    let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
    assert_eq!(status_of(&r, id), Some("in_sync"), "result={}", r);
    assert_eq!(r["applied"].as_i64(), Some(0));
}

#[test]
fn test_import_in_sync_when_body_normalized_matches() {
    let c = ctx();
    let id = make_memory(&c, "hello world", &["alpha"]);
    let dir = tempfile::tempdir().unwrap();
    write_md(
        dir.path(),
        "m.md",
        Some(id),
        1,
        &["alpha"],
        0.5,
        "  HELLO   world  ",
        &[],
    );

    let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
    assert_eq!(status_of(&r, id), Some("in_sync"), "result={}", r);
    assert_eq!(r["applied"].as_i64(), Some(0));
}

#[test]
fn test_import_confirm_applies_update() {
    let c = ctx();
    let id = make_memory(&c, "original content", &["alpha"]);
    let dir = tempfile::tempdir().unwrap();
    // same version (1) as DB, changed body → pending_update
    write_md(
        dir.path(),
        "m.md",
        Some(id),
        1,
        &["alpha"],
        0.5,
        "edited content",
        &[],
    );

    // review mode: staged, no write
    let review = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
    assert_eq!(status_of(&review, id), Some("pending_update"));
    assert_eq!(review["applied"].as_i64(), Some(0));
    assert_eq!(
        db_row(&c, id).0,
        "original content",
        "review must not write"
    );

    // confirm: applies
    let applied = memory_import_markdown(
        &c,
        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
    );
    assert_eq!(applied["applied"].as_i64(), Some(1));
    let (content, version, _) = db_row(&c, id);
    assert_eq!(content, "edited content");
    assert_eq!(version, 2, "version must increment");
}

#[test]
fn test_import_confirm_resyncs_tags_and_importance() {
    let c = ctx();
    let id = make_memory(&c, "base", &["old"]);
    let dir = tempfile::tempdir().unwrap();
    write_md(
        dir.path(),
        "m.md",
        Some(id),
        1,
        &["new1", "new2"],
        0.9,
        "base edited",
        &[],
    );

    let r = memory_import_markdown(
        &c,
        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
    );
    assert_eq!(r["applied"].as_i64(), Some(1));
    let tags = db_tags(&c, id);
    assert_eq!(
        tags,
        vec!["new1".to_string(), "new2".to_string()],
        "tags resynced"
    );
    let (_, _, importance) = db_row(&c, id);
    assert!(
        (importance - 0.9).abs() < 1e-6,
        "importance resynced: {}",
        importance
    );
}

#[test]
fn test_import_conflict_blocks_then_force_applies() {
    let c = ctx();
    let id = make_memory(&c, "c", &[]);
    // bump DB version to 3 so the file (version 1) is stale
    c.storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET version = 3 WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(crate::error::EngramError::Database)
        })
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    write_md(dir.path(), "m.md", Some(id), 1, &[], 0.5, "stale edit", &[]);

    // no force → conflict, no write
    let blocked = memory_import_markdown(
        &c,
        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
    );
    assert_eq!(status_of(&blocked, id), Some("conflict"));
    assert_eq!(blocked["applied"].as_i64(), Some(0));
    assert_eq!(db_row(&c, id).0, "c", "conflict must not write");

    // force → applies
    let forced = memory_import_markdown(
        &c,
        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true, "force_version": true}),
    );
    assert_eq!(forced["applied"].as_i64(), Some(1));
    assert_eq!(db_row(&c, id).0, "stale edit");
}

#[test]
fn test_import_new_inserts_memory() {
    let c = ctx();
    let dir = tempfile::tempdir().unwrap();
    // engram_id not present in DB
    write_md(
        dir.path(),
        "new.md",
        Some(999_999),
        1,
        &["x"],
        0.5,
        "brand new memory",
        &[],
    );

    let review = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
    assert_eq!(status_of(&review, 999_999), Some("new"));
    assert_eq!(review["applied"].as_i64(), Some(0));

    let applied = memory_import_markdown(
        &c,
        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
    );
    assert_eq!(applied["applied"].as_i64(), Some(1));
    // a memory with that content now exists (under a fresh autoincrement id)
    let count: i64 = c
        .storage
        .with_connection(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE content = ?1",
                rusqlite::params!["brand new memory"],
                |r| r.get(0),
            )
            .map_err(crate::error::EngramError::Database)
        })
        .unwrap();
    assert_eq!(count, 1, "new memory inserted");
}

#[test]
fn test_import_ignores_obsidian_keys() {
    let c = ctx();
    let id = make_memory(&c, "hello world", &["alpha"]);
    let dir = tempfile::tempdir().unwrap();
    // extra non-engram_ frontmatter keys must be ignored, not error
    write_md(
        dir.path(),
        "m.md",
        Some(id),
        1,
        &["alpha"],
        0.5,
        "hello world",
        &[("aliases", "[foo, bar]"), ("cssclasses", "note-card")],
    );

    let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
    assert_eq!(
        status_of(&r, id),
        Some("in_sync"),
        "obsidian keys ignored; result={}",
        r
    );
}

#[test]
fn test_import_skips_file_without_engram_id() {
    let c = ctx();
    let dir = tempfile::tempdir().unwrap();
    write_md(
        dir.path(),
        "plain.md",
        None,
        1,
        &[],
        0.5,
        "just an obsidian note",
        &[],
    );

    let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
    let skipped = r["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["status"] == "skipped");
    assert!(
        skipped,
        "file without engram_id must be skipped; result={}",
        r
    );
    assert_eq!(r["applied"].as_i64(), Some(0));
}
