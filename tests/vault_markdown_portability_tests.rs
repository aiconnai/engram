//! End-to-end integration tests for Markdown & Obsidian Vault Portability Adapter (RFC 0004).

use std::fs;
use std::path::PathBuf;

use engram::portability::{
    export_markdown, import_markdown, preview_markdown, ExportGrouping, ExportOptions,
    ImportOptions,
};
use engram::storage::queries::{create_memory, get_memory};
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, MemoryType};

fn temp_vault_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "engram_vault_test_{}_{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp vault dir");
    dir
}

fn setup_memories(storage: &Storage, workspace: &str) -> (i64, i64) {
    let m1 = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Authentication architecture: OAuth 2.0 with JWT tokens and short-lived access keys.".to_string(),
                    memory_type: MemoryType::Decision,
                    workspace: Some(workspace.to_string()),
                    tags: vec!["security".to_string(), "auth".to_string()],
                    importance: Some(0.9),
                    ..Default::default()
                },
            )
        })
        .expect("create memory 1");

    let m2 = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Database connection retry policy: Exponential backoff with jitter up to 30s.".to_string(),
                    memory_type: MemoryType::Learning,
                    workspace: Some(workspace.to_string()),
                    tags: vec!["database".to_string(), "resilience".to_string()],
                    importance: Some(0.8),
                    ..Default::default()
                },
            )
        })
        .expect("create memory 2");

    (m1.id, m2.id)
}

#[test]
fn test_vault_export_all_grouping_modes() {
    let storage = Storage::open_in_memory().expect("open memory storage");
    let ws = "vault_groupings";
    let (id1, id2) = setup_memories(&storage, ws);

    // 1. Flat grouping
    let flat_dir = temp_vault_dir("flat");
    let rep_flat = export_markdown(
        &storage,
        &ExportOptions {
            output_dir: flat_dir.clone(),
            grouping: ExportGrouping::Flat,
            workspace: Some(ws.to_string()),
            include_links: true,
        },
    )
    .expect("export flat");
    assert_eq!(rep_flat.files_written, 3);

    let files: Vec<_> = fs::read_dir(&flat_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert!(files.iter().any(|f| f.starts_with(&format!("{}-", id1))));
    assert!(files.iter().any(|f| f.starts_with(&format!("{}-", id2))));

    // Verify YAML frontmatter fields in exported file
    let sample_path = flat_dir.join(
        files
            .iter()
            .find(|f| f.starts_with(&format!("{}-", id1)))
            .unwrap(),
    );
    let content = fs::read_to_string(sample_path).unwrap();
    assert!(content.contains("---"));
    assert!(content.contains(&format!("engram_id: {}", id1)));
    assert!(content.contains(&format!("engram_workspace: {}", ws)));
    assert!(content.contains("engram_content_hash: sha256:"));
    assert!(content.contains("engram_tags:"));
    assert!(content.contains("  - security"));

    // 2. Type grouping
    let type_dir = temp_vault_dir("type");
    let rep_type = export_markdown(
        &storage,
        &ExportOptions {
            output_dir: type_dir.clone(),
            grouping: ExportGrouping::Type,
            workspace: Some(ws.to_string()),
            include_links: true,
        },
    )
    .expect("export type");
    assert_eq!(rep_type.files_written, 3);
    assert!(type_dir.join("decision").exists());
    assert!(type_dir.join("learning").exists());

    // 3. Workspace grouping
    let ws_dir = temp_vault_dir("workspace");
    let rep_ws = export_markdown(
        &storage,
        &ExportOptions {
            output_dir: ws_dir.clone(),
            grouping: ExportGrouping::Workspace,
            workspace: Some(ws.to_string()),
            include_links: true,
        },
    )
    .expect("export workspace");
    assert_eq!(rep_ws.files_written, 3);
    assert!(ws_dir.join(ws).exists());

    // Cleanup
    let _ = fs::remove_dir_all(flat_dir);
    let _ = fs::remove_dir_all(type_dir);
    let _ = fs::remove_dir_all(ws_dir);
}

#[test]
fn test_vault_import_drift_detection_and_confirmed_update() {
    let storage = Storage::open_in_memory().expect("open memory storage");
    let ws = "vault_drift";
    let (id1, _) = setup_memories(&storage, ws);

    let vault_dir = temp_vault_dir("drift");
    export_markdown(
        &storage,
        &ExportOptions {
            output_dir: vault_dir.clone(),
            grouping: ExportGrouping::Flat,
            workspace: Some(ws.to_string()),
            include_links: true,
        },
    )
    .expect("export initial");

    // Locate exported file for id1
    let files: Vec<_> = fs::read_dir(&vault_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    let file1_name = files
        .iter()
        .find(|f| f.starts_with(&format!("{}-", id1)))
        .unwrap();
    let file1_path = vault_dir.join(file1_name);

    // 1. Dry run on unmodified export should show 100% in_sync
    let preview1 = preview_markdown(&storage, vault_dir.clone(), Some(ws.to_string()))
        .expect("preview unmodified");
    assert_eq!(preview1.in_sync, 2);
    assert_eq!(preview1.pending, 0);

    // 2. Modify memory content in Markdown file (simulate human editing in Obsidian)
    let raw = fs::read_to_string(&file1_path).unwrap();
    let modified = raw.replace("OAuth 2.0 with JWT", "OAuth 2.1 with strict PKCE and JWT");
    fs::write(&file1_path, modified).unwrap();

    // 3. Dry run should now detect hash drift as pending update
    let preview2 = preview_markdown(&storage, vault_dir.clone(), Some(ws.to_string()))
        .expect("preview modified");
    eprintln!("DEBUG preview2: {:?}", preview2);
    assert_eq!(preview2.in_sync, 1);
    assert_eq!(preview2.pending, 1);
    assert_eq!(preview2.applied, 0);

    // 4. Confirmed import applies the update into SQLite
    let report = import_markdown(
        &storage,
        &ImportOptions {
            input_dir: vault_dir.clone(),
            dry_run: false,
            target_workspace: Some(ws.to_string()),
            force_version: false,
        },
    )
    .expect("confirmed import");

    assert_eq!(report.applied, 1);

    // Verify the canonical database record now has updated content and bumped version
    let updated = storage
        .with_connection(|conn| get_memory(conn, id1))
        .unwrap();
    assert!(updated.content.contains("OAuth 2.1 with strict PKCE"));
    assert_eq!(updated.version, 2);

    let _ = fs::remove_dir_all(vault_dir);
}

#[test]
fn test_vault_import_untracked_obsidian_notes_with_custom_frontmatter() {
    let storage = Storage::open_in_memory().expect("open memory storage");
    let ws = "vault_new_notes";

    let vault_dir = temp_vault_dir("untracked");

    // Create an Obsidian note created directly by human with Obsidian-specific frontmatter
    let note_content = r#"---
aliases:
  - "Distributed Lock Guide"
cssclasses:
  - wide-page
engram_workspace: vault_new_notes
engram_type: learning
engram_importance: 0.85
engram_tags:
  - distributed-systems
  - redis
---
# Redis Redlock Pattern

Use Redlock for distributed mutual exclusion with TTL safety margins.
"#;

    let note_path = vault_dir.join("redis-redlock.md");
    fs::write(note_path, note_content).unwrap();

    // 1. Dry run should classify this note as `new`
    let preview = preview_markdown(&storage, vault_dir.clone(), Some(ws.to_string()))
        .expect("preview new note");
    assert_eq!(preview.new, 1);
    assert_eq!(preview.applied, 0);

    // 2. Confirmed import creates a new canonical memory in database
    let report = import_markdown(
        &storage,
        &ImportOptions {
            input_dir: vault_dir.clone(),
            dry_run: false,
            target_workspace: Some(ws.to_string()),
            force_version: false,
        },
    )
    .expect("import new note");

    assert_eq!(report.applied, 1);

    // Verify memory was inserted into database with tags and content
    let all = storage
        .with_connection(|conn| {
            engram::storage::queries::list_memories(
                conn,
                &engram::types::ListOptions {
                    workspace: Some(ws.to_string()),
                    ..Default::default()
                },
            )
        })
        .unwrap();

    assert_eq!(all.len(), 1);
    assert!(all[0]
        .content
        .contains("Redlock for distributed mutual exclusion"));
    assert_eq!(
        all[0].tags,
        vec!["distributed-systems".to_string(), "redis".to_string()]
    );
    assert_eq!(all[0].memory_type, MemoryType::Learning);

    let _ = fs::remove_dir_all(vault_dir);
}

#[test]
fn test_vault_export_and_wikilinks_resolution() {
    let storage = Storage::open_in_memory().expect("open memory storage");
    let ws = "vault_wikilinks";
    let (id1, id2) = setup_memories(&storage, ws);

    // Create cross-reference link between id1 and id2
    storage
        .with_transaction(|conn| {
            conn.execute(
                "INSERT INTO crossrefs (from_id, to_id, edge_type, score, strength, source)
                 VALUES (?1, ?2, 'relates_to', 1.0, 1.0, 'user')",
                rusqlite::params![id1, id2],
            )?;
            Ok(())
        })
        .unwrap();

    let vault_dir = temp_vault_dir("wikilinks");
    let rep = export_markdown(
        &storage,
        &ExportOptions {
            output_dir: vault_dir.clone(),
            grouping: ExportGrouping::Flat,
            workspace: Some(ws.to_string()),
            include_links: true,
        },
    )
    .expect("export wikilinks");
    assert_eq!(rep.files_written, 3);

    // Verify file1 contains wikilink to file2
    let files: Vec<_> = fs::read_dir(&vault_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    let file1_name = files
        .iter()
        .find(|f| f.starts_with(&format!("{}-", id1)))
        .unwrap();
    let file2_name = files
        .iter()
        .find(|f| f.starts_with(&format!("{}-", id2)))
        .unwrap();
    let file2_stem = file2_name.strip_suffix(".md").unwrap();

    let content1 = fs::read_to_string(vault_dir.join(file1_name)).unwrap();
    assert!(content1.contains("## Related"));
    assert!(content1.contains(&format!("[[{}]]", file2_stem)));

    let _ = fs::remove_dir_all(vault_dir);
}

#[test]
fn test_vault_import_conflict_resolution() {
    let storage = Storage::open_in_memory().expect("open memory storage");
    let ws = "vault_conflicts";
    let (id1, _) = setup_memories(&storage, ws);

    let vault_dir = temp_vault_dir("conflicts");
    export_markdown(
        &storage,
        &ExportOptions {
            output_dir: vault_dir.clone(),
            grouping: ExportGrouping::Flat,
            workspace: Some(ws.to_string()),
            include_links: false,
        },
    )
    .expect("export initial");

    // 1. Modify file content locally in vault
    let files: Vec<_> = fs::read_dir(&vault_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    let file1_name = files
        .iter()
        .find(|f| f.starts_with(&format!("{}-", id1)))
        .unwrap();
    let file1_path = vault_dir.join(file1_name);
    let raw = fs::read_to_string(&file1_path).unwrap();
    fs::write(
        &file1_path,
        raw.replace("OAuth 2.0", "OAuth 2.0 Modified in Vault"),
    )
    .unwrap();

    // 2. Concurrently update database record out-of-band (bumping DB version to 2)
    storage
        .with_transaction(|conn| {
            engram::storage::queries::update_memory(
                conn,
                id1,
                &engram::types::UpdateMemoryInput {
                    content: Some("OAuth 2.0 with JWT tokens and biometric MFA.".to_string()),
                    ..Default::default()
                },
            )
        })
        .unwrap();

    // 3. Dry run detects version conflict (DB version 2 > file version 1)
    let preview = preview_markdown(&storage, vault_dir.clone(), Some(ws.to_string()))
        .expect("preview conflict");
    assert_eq!(preview.conflict, 1);
    assert_eq!(preview.applied, 0);

    // 4. Force import overrides conflict and applies the vault version
    let force_report = import_markdown(
        &storage,
        &ImportOptions {
            input_dir: vault_dir.clone(),
            dry_run: false,
            target_workspace: Some(ws.to_string()),
            force_version: true,
        },
    )
    .expect("force import");

    assert_eq!(force_report.applied, 1);

    let updated = storage
        .with_connection(|conn| get_memory(conn, id1))
        .unwrap();
    assert!(updated.content.contains("OAuth 2.0 Modified in Vault"));
    assert_eq!(updated.version, 3);

    let _ = fs::remove_dir_all(vault_dir);
}
