use super::*;

#[test]
fn supervisor_times_out_when_worker_never_reads_stdin() {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args([
            "--exact",
            "intelligence::pdf_worker::tests::worker_that_never_reads_stdin",
            "--nocapture",
        ])
        .env("ENGRAM_PDF_STALL_HELPER", "1");
    let input = vec![0_u8; 1024 * 1024];
    let started = Instant::now();

    let error = extract_with_command(
        command,
        &input,
        200,
        2 * 1024 * 1024,
        Duration::from_millis(200),
        IoFault::NONE,
    )
    .expect_err("stalled worker must time out");

    assert!(error.to_string().contains("timed out"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn worker_that_never_reads_stdin() {
    if std::env::var_os("ENGRAM_PDF_STALL_HELPER").is_some()
        || std::env::var("ENGRAM_PDF_HELPER").as_deref() == Ok("stall")
    {
        std::thread::sleep(Duration::from_secs(30));
    }
}

#[test]
fn supervisor_joins_reader_when_writer_panics() {
    let command = helper_command("exit");
    let started = Instant::now();

    let error = extract_with_command(
        command,
        &[1, 2, 3],
        200,
        2 * 1024 * 1024,
        Duration::from_secs(1),
        IoFault {
            writer_panics: true,
            reader_spawn_fails: false,
        },
    )
    .expect_err("writer panic must be contained");

    assert!(error.to_string().contains("input writer failed"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn supervisor_reaps_worker_when_reader_spawn_fails() {
    let command = helper_command("stall");
    let started = Instant::now();

    let error = extract_with_command(
        command,
        &vec![0_u8; 1024 * 1024],
        200,
        2 * 1024 * 1024,
        Duration::from_secs(1),
        IoFault {
            writer_panics: false,
            reader_spawn_fails: true,
        },
    )
    .expect_err("reader spawn failure must be contained");

    assert!(error.to_string().contains("reader spawn failed"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

fn helper_command(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args([
            "--exact",
            "intelligence::pdf_worker::tests::worker_that_never_reads_stdin",
            "--nocapture",
        ])
        .env("ENGRAM_PDF_HELPER", mode);
    command
}
