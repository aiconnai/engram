//! Import security regressions use private synthetic fixtures. Environment
//! configuration and potentially blocking inputs run only in bounded children.
#![cfg(unix)]

use super::handler::memory_import_markdown;
use super::test_support::ctx;
use crate::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use crate::mcp::handlers::{dispatch, HandlerContext};
use crate::storage::queries::create_memory;
use crate::types::CreateMemoryInput;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;

fn restricted_context() -> HandlerContext {
    let mut context = ctx();
    context.principal = Some(
        TransportPrincipal::from_token_claims(TokenClaims {
            user_id: UserId::from_string("import-fixture-user"),
            key_id: "import-fixture-key".into(),
            permissions: PermissionSet::standard_user(),
            namespace: Some("alpha".into()),
            issued_at: chrono::Utc::now(),
            expires_at: None,
        })
        .unwrap(),
    );
    context
}

fn memory(context: &HandlerContext, workspace: &str, content: &str) -> i64 {
    context
        .storage
        .with_transaction(|conn| {
            Ok(create_memory(
                conn,
                &CreateMemoryInput {
                    content: content.into(),
                    workspace: Some(workspace.into()),
                    tags: vec!["original".into()],
                    importance: Some(0.5),
                    ..Default::default()
                },
            )?
            .id)
        })
        .unwrap()
}

#[derive(Debug, PartialEq)]
struct MemoryState {
    id: i64,
    content: String,
    version: i64,
    importance: Option<f64>,
    workspace: String,
}

#[derive(Debug, PartialEq)]
struct DatabaseState {
    memories: Vec<MemoryState>,
    tags: Vec<(i64, String)>,
}

fn database_state(context: &HandlerContext) -> DatabaseState {
    context
        .storage
        .with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id,content,version,importance,workspace FROM memories ORDER BY id",
            )?;
            let memories = statement
                .query_map([], |row| {
                    Ok(MemoryState {
                        id: row.get(0)?,
                        content: row.get(1)?,
                        version: row.get(2)?,
                        importance: row.get(3)?,
                        workspace: row.get(4)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let mut statement = conn.prepare(
                "SELECT mt.memory_id,t.name FROM memory_tags mt \
                 JOIN tags t ON t.id=mt.tag_id ORDER BY mt.memory_id,t.name",
            )?;
            let tags = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(DatabaseState { memories, tags })
        })
        .unwrap()
}

fn write_note(path: &Path, id: Option<&str>, workspace: &str, content: &str) {
    let id_line = id.map_or_else(String::new, |id| format!("engram_id: {id}\n"));
    fs::write(
        path,
        format!(
            "---\n{id_line}engram_workspace: {workspace}\nengram_type: note\n\
             engram_version: 1\nengram_importance: 0.9\nengram_tags:\n  - changed\n\
             ---\n{content}\n"
        ),
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

fn invoke(context: &HandlerContext, params: Value, through_dispatch: bool) -> Value {
    if through_dispatch {
        dispatch(context, "memory_import_markdown", params)
    } else {
        memory_import_markdown(context, params)
    }
}

fn arguments(root: &Path, confirm: bool, force_version: bool) -> Value {
    json!({
        "input_dir": root,
        "workspace": "alpha",
        "confirm": confirm,
        "force_version": force_version,
    })
}

fn assert_rejected(result: &Value) {
    assert!(
        result.get("error").is_some(),
        "expected a top-level error: {result}"
    );
    assert_eq!(
        result["applied"], 0,
        "failed import reported applied writes: {result}"
    );
}

fn foreign_and_missing(root: &Path, confirm: bool, force_version: bool) {
    for through_dispatch in [false, true] {
        let foreign = restricted_context();
        let id = memory(&foreign, "beta", "beta-only-secret");
        write_note(
            &root.join("note.md"),
            Some(&id.to_string()),
            "alpha",
            "attempted foreign edit",
        );
        let missing = restricted_context();
        let foreign_before = database_state(&foreign);
        let missing_before = database_state(&missing);
        let params = arguments(root, confirm, force_version);
        let denied_foreign = invoke(&foreign, params.clone(), through_dispatch);
        let denied_missing = invoke(&missing, params, through_dispatch);

        assert_rejected(&denied_foreign);
        assert_rejected(&denied_missing);
        assert_eq!(
            denied_foreign["error"]["code"], "not_found",
            "{denied_foreign}"
        );
        assert_eq!(
            denied_foreign["error"], denied_missing["error"],
            "the same caller-supplied ID must not reveal whether a foreign row exists"
        );
        let rendered = denied_foreign.to_string();
        for secret in ["beta-only-secret", "content_hash", "db_version", "beta"] {
            assert!(
                !rendered.contains(secret),
                "foreign state leaked: {rendered}"
            );
        }
        assert_eq!(database_state(&foreign), foreign_before);
        assert_eq!(database_state(&missing), missing_before);
    }
}

fn rejected_batch_rolls_back(root: &Path) {
    for through_dispatch in [false, true] {
        let context = restricted_context();
        let own = memory(&context, "alpha", "own original");
        let foreign = memory(&context, "beta", "foreign original");
        write_note(
            &root.join("00-own.md"),
            Some(&own.to_string()),
            "alpha",
            "own edit",
        );
        write_note(
            &root.join("99-foreign.md"),
            Some(&foreign.to_string()),
            "alpha",
            "foreign edit",
        );
        let before = database_state(&context);
        let result = invoke(&context, arguments(root, true, true), through_dispatch);
        assert_rejected(&result);
        assert_eq!(result["error"]["code"], "not_found", "{result}");
        assert_eq!(
            database_state(&context),
            before,
            "earlier authorized writes survived a denied batch"
        );
    }
}

fn authorized_update_and_create(root: &Path) {
    for through_dispatch in [false, true] {
        let context = restricted_context();
        let own = memory(&context, "alpha", "own original");
        write_note(
            &root.join("own.md"),
            Some(&own.to_string()),
            "alpha",
            "own edit",
        );
        // The explicit authorized override takes precedence over frontmatter.
        write_note(&root.join("new.md"), None, "beta", "authorized new note");
        let before = database_state(&context);
        let preview = invoke(&context, arguments(root, false, false), through_dispatch);
        assert!(preview.get("error").is_none(), "{preview}");
        assert_eq!(preview["applied"], 0, "{preview}");
        assert_eq!(database_state(&context), before, "preview mutated storage");

        let result = invoke(&context, arguments(root, true, false), through_dispatch);
        assert!(result.get("error").is_none(), "{result}");
        assert_eq!(result["applied"], 2, "{result}");
        let after = database_state(&context);
        assert_eq!(after.memories.len(), 2);
        assert!(after.memories.iter().all(|row| row.workspace == "alpha"));
        let updated = after.memories.iter().find(|row| row.id == own).unwrap();
        assert_eq!(updated.content, "own edit");
        assert_eq!(updated.version, 2);
        assert!((updated.importance.unwrap() - 0.9).abs() < 1e-6);
        assert!(after.tags.contains(&(own, "changed".into())));
    }
}

fn unconfined_is_denied_before_read(root: &Path) {
    let context = restricted_context();
    let absent = root.join("must-not-be-scanned");
    assert!(!absent.exists());
    for through_dispatch in [false, true] {
        let before = database_state(&context);
        let result = invoke(&context, arguments(&absent, true, false), through_dispatch);
        assert_eq!(
            result["error"]["code"], "permission_denied",
            "authorization must precede path validation: {result}"
        );
        assert_eq!(result["error"]["required_mode"], "maintenance", "{result}");
        assert!(!absent.exists());
        assert_eq!(database_state(&context), before);
    }
}

fn unsafe_entry(root: &Path, kind: &str) {
    let context = restricted_context();
    let own = memory(&context, "alpha", "own original");
    write_note(
        &root.join("00-own.md"),
        Some(&own.to_string()),
        "alpha",
        "own edit",
    );
    let outside = root.parent().unwrap().join("outside.md");
    write_note(&outside, None, "alpha", "outside fixture content");
    match kind {
        "leaf-link" => symlink(&outside, root.join("99-leaf.md")).unwrap(),
        "directory-link" => {
            let directory = root.parent().unwrap().join("outside-directory");
            fs::create_dir(&directory).unwrap();
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
            write_note(
                &directory.join("outside.md"),
                None,
                "alpha",
                "outside directory content",
            );
            symlink(&directory, root.join("99-directory")).unwrap();
        }
        "cycle" => symlink(root, root.join("99-cycle")).unwrap(),
        "hardlink" => fs::hard_link(&outside, root.join("99-hardlink.md")).unwrap(),
        "fifo" => {
            use std::os::unix::ffi::OsStrExt;
            let name =
                std::ffi::CString::new(root.join("99-fifo.md").as_os_str().as_bytes()).unwrap();
            // SAFETY: name is NUL-terminated, references only this private test
            // fixture, and remains alive for the duration of mkfifo.
            assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        }
        _ => panic!("unknown unsafe-entry fixture: {kind}"),
    }
    let before = database_state(&context);
    for through_dispatch in [false, true] {
        let result = invoke(&context, arguments(root, true, false), through_dispatch);
        assert_rejected(&result);
        assert_eq!(
            database_state(&context),
            before,
            "filesystem preflight allowed partial application"
        );
    }
    assert!(fs::read_to_string(outside)
        .unwrap()
        .contains("outside fixture content"));
}

fn invalid_explicit_ids(root: &Path) {
    let context = restricted_context();
    let before = database_state(&context);
    for invalid in ["", "0", "-1", "-2", "not-an-id", "9223372036854775808"] {
        write_note(
            &root.join("invalid.md"),
            Some(invalid),
            "alpha",
            "must not become a new note",
        );
        for through_dispatch in [false, true] {
            let result = invoke(&context, arguments(root, true, true), through_dispatch);
            assert_rejected(&result);
            assert_eq!(
                result["error"]["code"], "invalid_params",
                "invalid explicit ID {invalid}: {result}"
            );
            assert_eq!(database_state(&context), before);
        }
    }
}

#[test]
fn p0_import_empty_canonical_id_does_not_become_a_new_note() {
    let context = ctx();
    let fixture = tempfile::tempdir().unwrap();
    fs::write(
        fixture.path().join("empty-id.md"),
        "---\nengram_id:\n---\nmust not create\n",
    )
    .unwrap();
    let before = database_state(&context);
    let result =
        memory_import_markdown(&context, json!({"input_dir":fixture.path(),"confirm":true}));
    assert_rejected(&result);
    assert_eq!(result["error"]["code"], "invalid_params");
    assert_eq!(database_state(&context), before);
}

#[test]
fn p0_import_fixture_child() {
    let Some(case) = std::env::var_os("ENGRAM_IMPORT_SECURITY_CASE") else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os("ENGRAM_IMPORT_SECURITY_ROOT").unwrap());
    let case = case.to_str().unwrap();
    match case {
        "idor-preview" => foreign_and_missing(&root, false, false),
        "idor-preview-force" => foreign_and_missing(&root, false, true),
        "idor-confirm" => foreign_and_missing(&root, true, false),
        "idor-confirm-force" => foreign_and_missing(&root, true, true),
        "batch-rollback" => rejected_batch_rolls_back(&root),
        "authorized" => authorized_update_and_create(&root),
        "unconfined" => unconfined_is_denied_before_read(&root),
        "invalid-id" => invalid_explicit_ids(&root),
        _ => unsafe_entry(&root, case),
    }
}

fn run_fixture(case: &str) {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("base");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "mcp::handlers::markdown_export::import::security_tests::p0_import_fixture_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", fixture.path())
        .env("ENGRAM_IMPORT_SECURITY_CASE", case)
        .env("ENGRAM_IMPORT_SECURITY_ROOT", &root)
        .env("ENGRAM_PERMISSION_MODE", "scoped_write")
        .current_dir(&root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if case != "unconfined" {
        command.env("ENGRAM_EXPORT_BASE_DIR", &root);
    }
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "import fixture {case} exceeded 10 seconds (unsafe reads must not block):\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "import fixture {case} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "the exact child-test filter did not execute the fixture"
    );
}

macro_rules! fixture_test {
    ($name:ident, $case:literal) => {
        #[test]
        fn $name() {
            run_fixture($case);
        }
    };
}

fixture_test!(p0_import_foreign_id_preview_matches_missing, "idor-preview");
fixture_test!(
    p0_import_foreign_id_forced_preview_matches_missing,
    "idor-preview-force"
);
fixture_test!(p0_import_foreign_id_confirm_matches_missing, "idor-confirm");
fixture_test!(
    p0_import_foreign_id_forced_confirm_matches_missing,
    "idor-confirm-force"
);
fixture_test!(
    p0_import_denied_batch_rolls_back_earlier_own_update,
    "batch-rollback"
);
fixture_test!(
    p0_import_authorized_update_and_create_remain_available,
    "authorized"
);
fixture_test!(
    p0_import_without_root_requires_maintenance_before_path_validation,
    "unconfined"
);
fixture_test!(
    p0_import_rejects_leaf_symlink_without_partial_writes,
    "leaf-link"
);
fixture_test!(
    p0_import_rejects_directory_symlink_without_partial_writes,
    "directory-link"
);
fixture_test!(
    p0_import_rejects_directory_cycle_in_bounded_process,
    "cycle"
);
fixture_test!(
    p0_import_rejects_hardlink_without_partial_writes,
    "hardlink"
);
fixture_test!(p0_import_rejects_fifo_without_blocking, "fifo");
fixture_test!(
    p0_import_rejects_explicit_invalid_ids_instead_of_creating,
    "invalid-id"
);
