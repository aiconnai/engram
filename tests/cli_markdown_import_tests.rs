//! Exercise the real CLI import path against private synthetic fixtures.
//! These checks compare logical SQLite state, not physical database bytes:
//! opening SQLite may checkpoint files without applying a Markdown import.
#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, StorageConfig, StorageMode};

const WORKSPACE: &str = "cli_markdown_import_fixture";
const ORIGINAL: &str = "Original synthetic note, unchanged until confirmation.";
const EDITED: &str = "# Edited synthetic note\n\nUpdated only with confirmation.";
const NEW: &str = "# New synthetic note\n\nInserted only with confirmation.";

#[derive(Debug, PartialEq)]
struct MemoryRow {
    id: i64,
    content: String,
    version: i64,
    workspace: String,
    importance: Option<f64>,
    updated_at: Option<String>,
}

#[derive(Debug, PartialEq)]
struct DatabaseState {
    memories: Vec<MemoryRow>,
    tags: Vec<(i64, String)>,
}

struct Fixture {
    directory: tempfile::TempDir,
    vault: PathBuf,
    temporary: PathBuf,
    database: PathBuf,
    existing_id: i64,
}

fn private_directory(path: &Path) {
    fs::create_dir(path).expect("create private fixture directory");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("restrict fixture directory permissions");
}

fn open_storage(database: &Path) -> Storage {
    Storage::open(StorageConfig {
        db_path: database.to_str().expect("UTF-8 fixture path").to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .expect("open synthetic fixture through official Storage")
}

fn write_markdown(path: &Path, id: Option<i64>, tag: &str, body: &str) {
    let identity = id.map_or_else(String::new, |id| format!("engram_id: {id}\n"));
    fs::write(
        path,
        format!(
            "---\n{identity}engram_workspace: {WORKSPACE}\nengram_type: note\n\
             engram_version: 1\nengram_importance: 0.75\nengram_tags:\n  - {tag}\n\
             ---\n{body}\n"
        ),
    )
    .expect("write synthetic Markdown input");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .expect("restrict Markdown fixture permissions");
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("private CLI fixture");
        let vault = directory.path().join("vault");
        let temporary = directory.path().join("temporary");
        private_directory(&vault);
        private_directory(&temporary);
        let database = directory.path().join("synthetic.db");
        let existing_id = {
            let storage = open_storage(&database);
            storage
                .with_transaction(|conn| {
                    Ok(create_memory(
                        conn,
                        &CreateMemoryInput {
                            content: ORIGINAL.into(),
                            workspace: Some(WORKSPACE.into()),
                            tags: vec!["original-tag".into()],
                            importance: Some(0.5),
                            ..Default::default()
                        },
                    )?
                    .id)
                })
                .expect("seed one existing memory")
        }; // Close Storage before any child process opens the same database.
        write_markdown(
            &vault.join("00-edit.md"),
            Some(existing_id),
            "edited-tag",
            EDITED,
        );
        write_markdown(&vault.join("01-new.md"), None, "new-tag", NEW);
        Self {
            directory,
            vault,
            temporary,
            database,
            existing_id,
        }
    }

    fn state(&self) -> DatabaseState {
        let storage = open_storage(&self.database);
        storage
            .with_connection(|conn| {
                let mut statement = conn.prepare(
                    "SELECT id,content,version,workspace,importance,updated_at \
                     FROM memories ORDER BY id",
                )?;
                let memories = statement
                    .query_map([], |row| {
                        Ok(MemoryRow {
                            id: row.get(0)?,
                            content: row.get(1)?,
                            version: row.get(2)?,
                            workspace: row.get(3)?,
                            importance: row.get(4)?,
                            updated_at: row.get(5)?,
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
            .expect("snapshot logical fixture state through official Storage")
    }

    fn import(&self, database: &Path, flags: &[&str]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_engram-cli"))
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("TMPDIR", &self.temporary)
            .env("ENGRAM_DB_PATH", database)
            .env("ENGRAM_EXPORT_BASE_DIR", &self.vault)
            .env("ENGRAM_PERMISSION_MODE", "scoped_write")
            .current_dir(self.directory.path())
            .args(["import", "markdown", "--path"])
            .arg(&self.vault)
            .args(["--workspace", WORKSPACE])
            .args(flags)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start the real engram-cli test binary");
        let deadline = Instant::now() + Duration::from_secs(30);
        while child.try_wait().expect("poll CLI child").is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let output = child.wait_with_output().expect("reap timed-out CLI child");
                panic!(
                    "CLI import exceeded 30 seconds with flags {flags:?}:\n{}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        child
            .wait_with_output()
            .expect("collect and reap CLI child")
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "CLI failed: status={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_markdown_import_without_confirm_never_applies_creates_or_updates() {
    let fixture = Fixture::new();
    let before = fixture.state();
    assert_eq!(before.memories.len(), 1);
    assert_eq!(before.memories[0].content, ORIGINAL);
    for flags in [
        &[][..],
        &["--dry-run"][..],
        &["--force-version"][..],
        &["--dry-run", "--force-version"][..],
    ] {
        let output = fixture.import(&fixture.database, flags);
        assert_success(&output); // A denied import is not evidence of safe review.
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2 files"), "{stdout}");
        assert!(stdout.contains("0 applied"), "{stdout}");
        assert!(stdout.contains("dry_run: true"), "{stdout}");
        assert!(
            stdout.contains("--confirm"),
            "review should explain how to apply: {stdout}"
        );
        assert_eq!(
            fixture.state(),
            before,
            "review mutated rows, versions or tags with {flags:?}"
        );
    }
}

#[test]
fn cli_markdown_import_confirm_applies_both_the_update_and_new_note() {
    let fixture = Fixture::new();
    let before = fixture.state();
    let output = fixture.import(&fixture.database, &["--confirm"]);
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 applied"), "{stdout}");
    assert!(stdout.contains("dry_run: false"), "{stdout}");

    let after = fixture.state();
    assert_eq!(before.memories.len(), 1);
    assert_eq!(after.memories.len(), 2);
    assert!(after.memories.iter().all(|row| row.workspace == WORKSPACE));
    let edited = after
        .memories
        .iter()
        .find(|row| row.id == fixture.existing_id)
        .unwrap();
    assert_eq!(edited.content, EDITED);
    assert_eq!(edited.version, before.memories[0].version + 1);
    assert_eq!(edited.importance, Some(0.75));
    assert!(after.tags.contains(&(edited.id, "edited-tag".into())));
    assert!(!after.tags.contains(&(edited.id, "original-tag".into())));
    let created = after
        .memories
        .iter()
        .find(|row| row.id != fixture.existing_id)
        .unwrap();
    assert_eq!(created.content, NEW);
    assert_eq!(created.version, 1);
    assert!(after.tags.contains(&(created.id, "new-tag".into())));
}

#[test]
fn cli_markdown_import_conflicting_flags_fail_before_creating_a_database() {
    let fixture = Fixture::new();
    let before = fixture.state();
    for (index, flags) in [["--confirm", "--dry-run"], ["--dry-run", "--confirm"]]
        .iter()
        .enumerate()
    {
        let absent_database = fixture
            .directory
            .path()
            .join(format!("rejected-{index}.db"));
        assert!(!absent_database.exists());
        let output = fixture.import(&absent_database, flags);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("--confirm"), "{stderr}");
        assert!(stderr.contains("--dry-run"), "{stderr}");
        assert!(
            !absent_database.exists(),
            "argument rejection opened a new database"
        );
        assert_eq!(fixture.state(), before);
    }
}
