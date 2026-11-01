use super::handler_tests::{ctx, p0_export, p0_memory};
use crate::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use crate::mcp::handlers::dispatch;
use crate::storage::queries::create_memory;
use crate::types::{CreateMemoryInput, MemoryType};
use serde_json::json;
use std::path::{Path, PathBuf};

fn principal(permissions: PermissionSet) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("export-test-user"),
        key_id: "export-test-key".into(),
        permissions,
        namespace: Some("markdown_export_schema".into()),
        issued_at: chrono::Utc::now(),
        expires_at: None,
    })
    .unwrap()
}

#[test]
fn p0_expiration_cutoff_excludes_equality_but_preserves_null_and_future() {
    let c = ctx();
    let null = p0_memory(&c, "No expiration", vec![]);
    let past = p0_memory(&c, "Past", vec![]);
    let equal = p0_memory(&c, "At cutoff", vec![]);
    let future = p0_memory(&c, "Future", vec![]);
    c.storage
        .with_connection(|conn| {
            for (id, date) in [
                (past, "2029-01-01T00:00:00+00:00"),
                (equal, "2030-01-01T00:00:00+00:00"),
                (future, "2031-01-01T00:00:00+00:00"),
            ] {
                conn.execute(
                    "UPDATE memories SET tier='daily',expires_at=?1 WHERE id=?2",
                    rusqlite::params![date, id],
                )?;
            }
            let rows = super::query::query_workspace_memories_at(
                conn,
                "markdown_export_schema",
                "2030-01-01T00:00:00+00:00",
            )?;
            let ids: std::collections::BTreeSet<_> =
                rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
            assert_eq!(ids, std::collections::BTreeSet::from([null, future]));
            Ok(())
        })
        .unwrap();
}

#[test]
fn p0_review_expiration_compares_instants_without_losing_nanoseconds() {
    let c = ctx();
    let unexpired = p0_memory(&c, "No expiry", vec![]);
    let mut expected = std::collections::BTreeSet::from([unexpired]);
    for (date, keep) in [
        ("2030-01-01T00:00:00Z", false),
        ("2030-01-01T00:00:00.000000000Z", false),
        ("2030-01-01T01:00:00+01:00", false),
        ("2029-12-31T23:00:00-01:00", false),
        ("2029-12-31T23:59:59.999999999Z", false),
        ("2030-01-01T00:00:00.000000001Z", true),
    ] {
        let id = p0_memory(&c, date, vec![]);
        c.storage
            .with_connection(|conn| {
                conn.execute(
                    "UPDATE memories SET tier='daily',expires_at=?1 WHERE id=?2",
                    rusqlite::params![date, id],
                )?;
                Ok(())
            })
            .unwrap();
        if keep {
            expected.insert(id);
        }
    }
    c.storage
        .with_connection(|conn| {
            let rows = super::query::query_workspace_memories_at(
                conn,
                "markdown_export_schema",
                "2030-01-01T00:00:00+00:00",
            )?;
            let ids: std::collections::BTreeSet<_> =
                rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
            assert_eq!(ids, expected);
            Ok(())
        })
        .unwrap();
}

#[test]
fn p0_review_invalid_expiration_is_an_explicit_error() {
    let c = ctx();
    let id = p0_memory(&c, "Bad expiration", vec![]);
    c.storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET tier='daily',expires_at='not-a-timestamp' WHERE id=?1",
                [id],
            )?;
            assert!(super::query::query_workspace_memories_at(
                conn,
                "markdown_export_schema",
                "2030-01-01T00:00:00Z"
            )
            .is_err());
            Ok(())
        })
        .unwrap();
}

#[test]
fn p0_invalid_metadata_is_an_error_not_silently_null() {
    let c = ctx();
    let id = p0_memory(&c, "Invalid metadata", vec![]);
    c.storage
        .with_connection(|conn| {
            conn.execute("UPDATE memories SET metadata='not-json' WHERE id=?1", [id])?;
            Ok(())
        })
        .unwrap();
    let fixture = tempfile::tempdir().unwrap();
    let out = fixture.path().join("export");
    let r = p0_export(&c, &out, "flat");
    assert!(r.get("error").is_some(), "{r}");
    assert!(!out.exists());
}

#[test]
fn p0_cross_workspace_and_inactive_relations_are_not_exported() {
    let c = ctx();
    let a = p0_memory(&c, "Visible A", vec![]);
    let b = p0_memory(&c, "Visible B", vec![]);
    let secret = c
        .storage
        .with_transaction(|conn| {
            Ok(create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Other workspace secret".into(),
                    workspace: Some("foreign".into()),
                    ..Default::default()
                },
            )?
            .id)
        })
        .unwrap();
    c.storage.with_connection(|conn| {
        conn.execute("INSERT INTO crossrefs(from_id,to_id,edge_type,score) VALUES(?1,?2,'supersedes',1.0)",[a,secret])?;
        conn.execute("INSERT INTO crossrefs(from_id,to_id,edge_type,score,valid_to) VALUES(?1,?2,'contradicts',1.0,'2000-01-01')",[a,b])?;
        Ok(())
    }).unwrap();
    let fixture = tempfile::tempdir().unwrap();
    let r = p0_export(&c, fixture.path(), "flat");
    assert!(r.get("error").is_none(), "{r}");
    let a = std::fs::read_to_string(fixture.path().join(format!("{a}-visible-a.md"))).unwrap();
    assert!(!a.contains("## Related"), "{a}");
}

#[cfg(unix)]
#[test]
fn p0_active_database_and_hardlink_are_refused_without_opening_them() {
    for alias in [false, true] {
        let fixture = tempfile::tempdir().unwrap();
        let db = fixture
            .path()
            .join(if alias { "state.db" } else { "index.md" });
        let mut c = ctx();
        c.storage = crate::storage::Storage::open(crate::types::StorageConfig {
            db_path: db.to_string_lossy().into_owned(),
            storage_mode: crate::types::StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        })
        .unwrap();
        let id = p0_memory(&c, "Private note", vec![]);
        if alias {
            std::fs::hard_link(&db, fixture.path().join(format!("{id}-private-note.md"))).unwrap();
        }
        let r = p0_export(&c, fixture.path(), "flat");
        assert!(r.get("error").is_some(), "{r}");
        if !alias {
            assert!(!fixture
                .path()
                .join(format!("{id}-private-note.md"))
                .exists());
        }
        let count = c
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT count(*) FROM memories", [], |r| r.get::<_, i64>(0))?)
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}

// Re-exec only this test in a bounded child with a clean environment. This
// isolates server-root/permission configuration from all concurrent unit tests.
#[cfg(unix)]
#[test]
fn p0_export_fixture_child() {
    let Some(root) = std::env::var_os("ENGRAM_P0_FIXTURE") else {
        return;
    };
    let root = PathBuf::from(root);
    let mut c = ctx();
    for (kind, title) in [
        (MemoryType::Note, "Alpha"),
        (MemoryType::Decision, "Beta"),
        (MemoryType::Summary, "Gamma"),
    ] {
        c.storage
            .with_transaction(|conn| {
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: title.into(),
                        memory_type: kind,
                        workspace: Some("markdown_export_schema".into()),
                        tags: vec!["zeta".into(), "alpha".into()],
                        ..Default::default()
                    },
                )?;
                Ok(())
            })
            .unwrap();
    }
    c.storage.with_connection(|conn| {conn.execute("UPDATE memories SET created_at='2026-01-01T00:00:00+00:00',updated_at='2026-01-01T00:00:00+00:00'",[])?;Ok(())}).unwrap();
    c.principal = Some(principal(PermissionSet::standard_user()));
    let out = root.join("export");
    let r = dispatch(
        &c,
        "memory_export_markdown",
        json!({"workspace":"markdown_export_schema","output_dir":out,"group":"type","_permission_mode":"scoped_write"}),
    );
    assert!(r.get("error").is_none(), "{r}");
    assert_eq!(r["memories_exported"], 3);

    let rejected = root.join("foreign-workspace");
    let r = dispatch(
        &c,
        "memory_export_markdown",
        json!({"workspace":"foreign","output_dir":rejected}),
    );
    assert_eq!(r["error"]["code"], "permission_denied", "{r}");
    assert!(!rejected.exists());
    let outside = root.parent().unwrap().join("outside-base");
    let r = p0_export(&c, &outside, "flat");
    assert!(r.get("error").is_some(), "{r}");
    assert!(!outside.exists());

    c.principal = Some(principal(PermissionSet::read_only()));
    let r = p0_export(&c, &root.join("read-only"), "flat");
    assert_eq!(r["error"]["code"], "permission_denied", "{r}");
    assert!(!root.join("read-only").exists());

    std::env::remove_var("ENGRAM_EXPORT_BASE_DIR"); // Child is isolated, only one test.
    c.principal = Some(principal(PermissionSet::standard_user()));
    let r = p0_export(&c, &root.join("unconfined"), "flat");
    assert_eq!(r["error"]["required_mode"], "maintenance", "{r}");
    assert!(!root.join("unconfined").exists());
    c.principal = Some(principal(PermissionSet::admin()));
    let r = p0_export(&c, &root.join("maintenance"), "flat");
    assert!(r.get("error").is_none(), "{r}");
}

#[cfg(unix)]
#[test]
fn p0_separate_processes_produce_byte_identical_trees_and_enforce_roots() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let fixture = tempfile::tempdir().unwrap();
    let mut expected = None;
    for n in 0..4 {
        let root = fixture.path().join(format!("run{n}"));
        std::fs::create_dir(&root).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "mcp::handlers::markdown_export::export::security_tests::p0_export_fixture_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("TMPDIR", fixture.path())
            .env("HOME", &root)
            .env("ENGRAM_P0_FIXTURE", &root)
            .env("ENGRAM_EXPORT_BASE_DIR", &root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() > deadline {
                child.kill().unwrap();
                let _ = child.wait();
                panic!("export fixture subprocess timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fn tree(
            root: &Path,
            base: &Path,
            result: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
        ) {
            for entry in std::fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    tree(&path, base, result);
                } else {
                    result.insert(
                        path.strip_prefix(base).unwrap().to_owned(),
                        std::fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut actual = std::collections::BTreeMap::new();
        tree(&root.join("export"), &root.join("export"), &mut actual);
        assert_eq!(actual.len(), 4);
        if let Some(expected) = &expected {
            assert_eq!(&actual, expected);
        } else {
            expected = Some(actual);
        }
    }
}
