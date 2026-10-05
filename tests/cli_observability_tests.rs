//! O1: the CLI's stdout keeps carrying the user's content (it is the command's
//! result), and its stderr stays free of it even with `RUST_LOG=trace`.
//!
//! Honest scope: `engram-cli` installs no tracing subscriber, so its stderr is
//! empty by construction and this test cannot catch a leak in library log
//! sites (those are covered by the in-process capture tests). It is a
//! regression guard: if the CLI later installs a subscriber, any content in
//! its logs fails here, and stdout correctness is asserted either way.

use std::process::{Command, Output};

use tempfile::TempDir;

const PRIVATE_TEXT: &str = "cli-private-text-sentinel-7e8f";
const QUERY_TEXT: &str = "cli-query-sentinel-9a0b";

fn run(home: &TempDir, db: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_engram-cli"))
        .env_clear()
        .env("HOME", home.path())
        .env("ENGRAM_DB_PATH", db)
        .env("RUST_LOG", "trace")
        .current_dir(home.path())
        .args(args)
        .output()
        .expect("run engram-cli")
}

#[test]
fn cli_stdout_has_the_content_and_stderr_has_none_of_it_even_at_trace() {
    let home = tempfile::tempdir().expect("home");
    let db = home.path().join(format!("{PRIVATE_TEXT}.db"));

    let created = run(&home, &db, &["create", PRIVATE_TEXT]);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let searched = run(&home, &db, &["search", QUERY_TEXT]);
    assert!(
        searched.status.success(),
        "{}",
        String::from_utf8_lossy(&searched.stderr)
    );
    let found = run(&home, &db, &["search", PRIVATE_TEXT]);

    // Protocol/stdout stays correct: the command result carries the content.
    assert!(String::from_utf8_lossy(&created.stdout).contains(PRIVATE_TEXT));
    assert!(String::from_utf8_lossy(&found.stdout).contains(PRIVATE_TEXT));

    // The log stream carries neither the content, the query nor the db path.
    for output in [&created, &searched, &found] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        for needle in [PRIVATE_TEXT, QUERY_TEXT, "cli-private-text-sentinel"] {
            assert!(
                !stderr.contains(needle),
                "stderr leaked {needle:?}: {stderr}"
            );
        }
    }
}
