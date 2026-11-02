//! Workflow tests cross the typed seam, with real SQLite and private files.
use super::{
    export::{export, ExportRequest},
    import::{import, ImportRequest},
    *,
};
use crate::auth::{PermissionSet, TokenClaims, UserId};
use crate::storage::queries::create_memory;
use crate::types::CreateMemoryInput;

fn seed(storage: &Storage, workspace: &str, content: &str) -> i64 {
    storage
        .with_transaction(|conn| {
            Ok(create_memory(
                conn,
                &CreateMemoryInput {
                    content: content.into(),
                    workspace: Some(workspace.into()),
                    ..Default::default()
                },
            )?
            .id)
        })
        .unwrap()
}

fn restricted_principal() -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("markdown-core-test"),
        key_id: "fixture".into(),
        permissions: PermissionSet::standard_user(),
        namespace: Some("alpha".into()),
        issued_at: chrono::Utc::now(),
        expires_at: None,
    })
    .unwrap()
}

#[test]
fn typed_workflow_preserves_details_without_search_or_transport_state() {
    let storage = Storage::open_in_memory().unwrap();
    let id = seed(&storage, "alpha", "Original note");
    let ctx = MarkdownContext {
        storage: &storage,
        principal: None,
    };
    let fixture = tempfile::tempdir().unwrap();
    let outcome = export(
        &ctx,
        ExportRequest {
            workspace: "alpha",
            output_dir: fixture.path().to_str(),
            group: "flat",
            include_links: false,
        },
    )
    .unwrap();
    assert_eq!(outcome.memories_exported, 1);
    assert_eq!(outcome.files_written, 2);
    let note = fixture.path().join(format!("{id}-original-note.md"));
    let text = std::fs::read_to_string(&note).unwrap();
    std::fs::write(&note, text.replace("Original note", "Edited note")).unwrap();
    // Import the edited note deliberately; index.md is a separate untracked note.
    std::fs::remove_file(&outcome.index_path).unwrap();
    let request = |confirm| ImportRequest {
        input_dir: fixture.path().to_str().unwrap(),
        workspace: Some("alpha"),
        force_version: false,
        confirm,
    };
    let preview = import(&ctx, request(false)).unwrap();
    assert!(preview.dry_run);
    assert_eq!(
        (preview.scanned, preview.pending, preview.applied),
        (1, 1, 0)
    );
    let detail = preview
        .files
        .iter()
        .find(|file| file.engram_id == Some(id))
        .unwrap();
    assert_eq!(detail.status, "pending_update");
    assert!(detail.applied.is_none());
    let applied = import(&ctx, request(true)).unwrap();
    assert!(!applied.dry_run);
    assert_eq!((applied.pending, applied.applied), (1, 1));
    assert_eq!(
        applied
            .files
            .iter()
            .find(|file| file.engram_id == Some(id))
            .unwrap()
            .applied,
        Some(true)
    );
    let content = storage
        .with_connection(|conn| {
            Ok(
                conn.query_row("SELECT content FROM memories WHERE id=?1", [id], |row| {
                    row.get::<_, String>(0)
                })?,
            )
        })
        .unwrap();
    assert_eq!(content, "Edited note");
}

#[test]
fn typed_import_authorizes_entire_batch_before_writes() {
    let storage = Storage::open_in_memory().unwrap();
    let own = seed(&storage, "alpha", "Original own note");
    let foreign = seed(&storage, "beta", "Foreign note");
    let principal = restricted_principal();
    let ctx = MarkdownContext {
        storage: &storage,
        principal: Some(&principal),
    };
    let fixture = tempfile::tempdir().unwrap();
    for (name, id) in [("a-own.md", own), ("b-foreign.md", foreign)] {
        std::fs::write(fixture.path().join(name), format!("---\nengram_id: {id}\nengram_version: 1\nengram_workspace: alpha\n---\nChanged note\n")).unwrap();
    }
    let error = import(
        &ctx,
        ImportRequest {
            input_dir: fixture.path().to_str().unwrap(),
            workspace: Some("alpha"),
            force_version: true,
            confirm: true,
        },
    )
    .unwrap_err();
    let (error, attempted) = error.into_parts();
    assert!(matches!(error, EngramError::NotFound(id) if id == foreign));
    assert!(attempted.is_none());
    let content = storage
        .with_connection(|conn| {
            Ok(
                conn.query_row("SELECT content FROM memories WHERE id=?1", [own], |row| {
                    row.get::<_, String>(0)
                })?,
            )
        })
        .unwrap();
    assert_eq!(content, "Original own note");
}

#[test]
fn typed_export_refuses_foreign_workspace_before_writing() {
    let storage = Storage::open_in_memory().unwrap();
    seed(&storage, "beta", "Foreign note");
    let principal = restricted_principal();
    let ctx = MarkdownContext {
        storage: &storage,
        principal: Some(&principal),
    };
    let fixture = tempfile::tempdir().unwrap();
    let output = fixture.path().join("output");
    assert!(export(
        &ctx,
        ExportRequest {
            workspace: "beta",
            output_dir: output.to_str(),
            group: "flat",
            include_links: true
        }
    )
    .is_err());
    assert!(!output.exists());
}

#[test]
fn public_facades_keep_environment_policy_in_isolated_processes() {
    const CASE: &str = "ENGRAM_MARKDOWN_CORE_POLICY_CASE";
    if let Ok(case) = std::env::var(CASE) {
        let fixture = tempfile::tempdir().unwrap();
        let root = case.ends_with("-root");
        let mode = case.strip_suffix("-root").unwrap_or(&case);
        std::env::remove_var("ENGRAM_EXPORT_BASE_DIR");
        if root {
            std::env::set_var("ENGRAM_EXPORT_BASE_DIR", fixture.path());
        }
        std::env::set_var("ENGRAM_PERMISSION_MODE", mode);
        let storage = Storage::open_in_memory().unwrap();
        let exported = export_markdown(
            &storage,
            &ExportOptions {
                output_dir: fixture.path().into(),
                ..Default::default()
            },
        );
        let imported = import_markdown(
            &storage,
            &ImportOptions {
                input_dir: fixture.path().into(),
                ..Default::default()
            },
        );
        let allowed = mode == "maintenance" || mode == "admin" || (mode == "scoped_write" && root);
        if allowed {
            assert_eq!(exported.unwrap().files_written, 0);
            let report = imported.unwrap();
            assert!(report.dry_run);
            assert_eq!(report.applied, 0);
        } else {
            for (operation, message) in [
                ("memory_export_markdown", exported.unwrap_err()),
                ("memory_import_markdown", imported.unwrap_err()),
            ] {
                let expected = if mode == "invalid" {
                    "ENGRAM_PERMISSION_MODE must be one of read_only, scoped_write, maintenance, admin".into()
                } else {
                    format!(
                        "{operation} requires {} mode",
                        if root { "scoped_write" } else { "maintenance" }
                    )
                };
                assert!(
                    matches!(message, EngramError::InvalidInput(message) if message == expected)
                );
            }
        }
        return;
    }
    for root in [false, true] {
        for mode in [
            "read_only",
            "scoped_write",
            "maintenance",
            "admin",
            "invalid",
        ] {
            let case = format!("{mode}{}", if root { "-root" } else { "" });
            let name = concat!(
                module_path!(),
                "::public_facades_keep_environment_policy_in_isolated_processes"
            );
            let name = name.split_once("::").unwrap().1;
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", name, "--nocapture"])
                .env(CASE, &case)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while child.try_wait().unwrap().is_none() {
                if std::time::Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("policy case {case} timed out");
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let output = child.wait_with_output().unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("running 1 test"),
                "policy child selected no test: {stdout}"
            );
            assert!(
                output.status.success(),
                "case {case}: {stdout} {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
