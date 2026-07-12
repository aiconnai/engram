#![cfg(feature = "pdf")]

#[cfg(not(target_os = "macos"))]
use std::io::Write;
use std::process::{Command, Stdio};
#[cfg(not(target_os = "macos"))]
use std::time::{Duration, Instant};

fn worker() -> Command {
    let path = std::env::var_os("CARGO_BIN_EXE_engram-pdf-worker")
        .expect("Cargo must expose the packaged PDF worker to integration tests");
    Command::new(path)
}

/// On macOS the worker fails closed at startup because a hard process memory
/// limit (RLIMIT_AS) cannot be enforced; extraction-path tests only run where
/// the sandbox is enforceable.
#[cfg(target_os = "macos")]
#[test]
fn worker_fails_closed_on_macos_without_hard_memory_limit() {
    let output = worker()
        .args(["--max-pages", "200", "--max-text-bytes", "2097152"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("worker output");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("hard process memory limit"));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn worker_extracts_valid_pdf_with_bounded_protocol() {
    let pdf = include_bytes!("fixtures/pdf/valid.pdf");
    let mut child = worker()
        .args(["--max-pages", "200", "--max-text-bytes", "2097152"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("worker should spawn");

    child
        .stdin
        .take()
        .expect("worker stdin")
        .write_all(pdf)
        .expect("write fixture");
    let output = child.wait_with_output().expect("worker output");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(payload["sections"][0]["page"], 1);
    assert!(payload["sections"][0]["content"]
        .as_str()
        .is_some_and(|text| text.contains("Engram PDF fixture")));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn worker_rejects_compression_bomb_within_a_wall_clock_budget() {
    let pdf = include_bytes!("fixtures/pdf/compression-bomb.pdf");
    let started = Instant::now();
    let mut child = worker()
        .args(["--max-pages", "200", "--max-text-bytes", "2097152"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("worker should spawn");

    child
        .stdin
        .take()
        .expect("worker stdin")
        .write_all(pdf)
        .expect("write fixture");
    let output = child.wait_with_output().expect("worker output");

    assert!(!output.status.success());
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(output.stdout.len() <= 4096);
}
