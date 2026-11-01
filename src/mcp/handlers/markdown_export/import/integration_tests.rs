use super::handler::memory_import_markdown;
use super::test_support::*;
use serde_json::json;

#[cfg(unix)]
#[test]
fn import_post_commit_hardening_error_does_not_report_zero_applied() {
    let fixture = tempfile::tempdir().unwrap();
    let physical = fixture.path().join("physical");
    let replacement = fixture.path().join("replacement");
    std::fs::create_dir(&physical).unwrap();
    std::fs::create_dir(&replacement).unwrap();
    let alias = fixture.path().join("configured-parent");
    std::os::unix::fs::symlink(&physical, &alias).unwrap();
    let db_path = alias.join("memory.db");
    let mut c = ctx();
    c.storage = crate::storage::Storage::open(crate::types::StorageConfig {
        db_path: db_path.to_string_lossy().into_owned(),
        storage_mode: crate::types::StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .unwrap();
    let id = make_memory(&c, "original before post-commit error", &[]);
    let input = fixture.path().join("input");
    std::fs::create_dir(&input).unwrap();
    write_md(
        &input,
        "edit.md",
        Some(id),
        1,
        &[],
        0.5,
        "committed despite hardening error",
        &[],
    );
    let sentinel = fixture.path().join("sentinel");
    std::fs::write(&sentinel, "private fixture").unwrap();
    // SQLite opened the canonical physical parent. Fault-inject a replacement
    // of Storage's configured parent alias: its post-commit hardening fails,
    // while the live DB/WAL/SHM themselves are never opened, moved or replaced.
    std::os::unix::fs::symlink(&sentinel, replacement.join("memory.db")).unwrap();
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&replacement, &alias).unwrap();
    let result = memory_import_markdown(&c, json!({"input_dir":input,"confirm":true}));
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&physical, &alias).unwrap();
    assert!(
        result.get("error").is_some(),
        "hardening failure must remain an error"
    );
    assert_eq!(db_row(&c, id).0, "committed despite hardening error");
    assert!(c
        .storage
        .with_connection(|conn| Ok(conn.is_autocommit()))
        .unwrap());
    assert!(
        result["applied"].is_null(),
        "post-commit error cannot claim applied=0: {result}"
    );
    assert_eq!(result["outcome"], "unknown");
    assert_eq!(result["attempted"], 1);
    assert_eq!(
        std::fs::read_to_string(sentinel).unwrap(),
        "private fixture"
    );
}

#[test]
fn import_tab_separated_invalid_identity_and_version_are_not_missing() {
    for confirm in [true, false] {
        for key in [
            "engram_id",
            "engram_version",
            "'engram_id'",
            "\"engram_version\"",
        ] {
            let c = ctx();
            let fixture = tempfile::tempdir().unwrap();
            std::fs::write(
                fixture.path().join("invalid.md"),
                format!("---\n{key}:\t0\n---\nmust not become a note\n"),
            )
            .unwrap();
            let result =
                memory_import_markdown(&c, json!({"input_dir":fixture.path(),"confirm":confirm}));
            assert!(
                result.get("error").is_some(),
                "{key}, confirm={confirm}: {result}"
            );
            assert_eq!(result["applied"], 0);
            assert_eq!(
                c.storage
                    .with_connection(|conn| Ok(conn.query_row(
                        "SELECT COUNT(*) FROM memories",
                        [],
                        |row| row.get::<_, i64>(0)
                    )?))
                    .unwrap(),
                0
            );
        }
    }
}

#[test]
fn import_database_error_rolls_back_earlier_updates() {
    let c = ctx();
    let first = make_memory(&c, "first original", &["first"]);
    let second = make_memory(&c, "second original", &["second"]);
    let before = (db_row(&c, first), db_row(&c, second));
    let dir = tempfile::tempdir().unwrap();
    write_md(
        dir.path(),
        "00-first.md",
        Some(first),
        1,
        &["changed"],
        0.9,
        "first edit",
        &[],
    );
    write_md(
        dir.path(),
        "99-second.md",
        Some(second),
        1,
        &["changed"],
        0.9,
        "second edit",
        &[],
    );
    c.storage.with_connection(|conn| {
        conn.execute_batch(&format!("CREATE TRIGGER reject_second_import BEFORE UPDATE OF content ON memories WHEN NEW.id = {second} BEGIN SELECT RAISE(ABORT, 'injected import failure'); END;"))?;
        Ok(())
    }).unwrap();
    let result = memory_import_markdown(&c, json!({"input_dir":dir.path(),"confirm":true}));
    assert!(result.get("error").is_some(), "{result}");
    assert_eq!(result["applied"], 0);
    assert_eq!((db_row(&c, first), db_row(&c, second)), before);
    assert_eq!(db_tags(&c, first), vec!["first"]);
    assert_eq!(db_tags(&c, second), vec!["second"]);
}

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
fn test_import_stages_file_without_engram_id_as_new() {
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
    let is_new = r["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["status"] == "new");
    assert!(
        is_new,
        "file without engram_id must be staged as new; result={}",
        r
    );
    assert_eq!(r["applied"].as_i64(), Some(0));

    // When confirm is true, it should be applied to storage
    let r_confirmed = memory_import_markdown(
        &c,
        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
    );
    assert_eq!(r_confirmed["applied"].as_i64(), Some(1));
}
