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

#[cfg(unix)]
mod crash_and_cleanup {
    use super::*;

    fn sh(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script]);
        command
    }

    fn run(command: Command, input: &[u8], timeout: Duration) -> crate::error::Result<usize> {
        extract_with_command(command, input, 200, 2 * 1024 * 1024, timeout, IoFault::NONE)
            .map(|sections| sections.len())
    }

    #[test]
    fn worker_killed_by_a_signal_is_a_typed_error_not_partial_success() {
        // Emits a valid-looking prefix, then dies like an OOM-killed worker.
        let error = run(
            sh("printf '{\"sections\":['; kill -9 $$"),
            &[1, 2, 3],
            Duration::from_secs(5),
        )
        .expect_err("a crashed worker must fail");
        assert!(
            error.to_string().contains("terminated by a resource limit"),
            "{error}"
        );
    }

    #[test]
    fn worker_that_exits_zero_with_garbage_is_rejected() {
        let error = run(sh("echo not-json"), &[1], Duration::from_secs(5))
            .expect_err("garbage output must fail");
        assert!(error.to_string().contains("invalid response"), "{error}");
    }

    #[test]
    fn worker_exiting_before_reading_large_input_reports_its_exit_not_a_hang() {
        let started = Instant::now();
        let error = run(
            sh("exit 7"),
            &vec![0_u8; 4 * 1024 * 1024],
            Duration::from_secs(5),
        )
        .expect_err("early exit must fail");
        assert!(error.to_string().contains("worker"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn missing_worker_binary_is_a_start_error() {
        let error = run(
            Command::new("/nonexistent/engram-pdf-worker"),
            &[1],
            Duration::from_secs(1),
        )
        .expect_err("spawn must fail");
        assert!(error.to_string().contains("failed to start"), "{error}");
    }

    #[test]
    fn timed_out_worker_process_is_gone_when_extract_returns() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("worker.pid");
        let script = format!("echo $$ > {}; exec sleep 300", pid_file.display());

        let error = run(sh(&script), &[1], Duration::from_millis(500))
            .expect_err("a hung worker must time out");

        assert!(error.to_string().contains("timed out"), "{error}");
        let pid: i32 = std::fs::read_to_string(&pid_file)
            .expect("worker wrote its pid")
            .trim()
            .parse()
            .unwrap();
        // SAFETY: signal 0 only probes for existence.
        let alive = unsafe { libc::kill(pid, 0) } == 0;
        assert!(!alive, "worker {pid} survived the supervisor");
    }
}
