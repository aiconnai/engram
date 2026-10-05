//! Bounded child-process execution for the multimodal pipeline.
//!
//! Every external binary the multimodal code runs (`ffprobe`, `ffmpeg`) goes
//! through [`run_bounded`], which guarantees that on every exit path (success,
//! timeout, spawn failure, wait failure, panic or cancellation of the caller)
//! the child (on unix also anything it forked in its process group) has been killed and
//! reaped. Captured output is capped so a chatty or runaway child cannot grow
//! the parent's memory.
//!
//! Platform note: process-group kill is unix-only. On Windows only the direct
//! child is killed (grandchildren it spawned can survive a timeout).
//!
//! Distinguishing business failure from best-effort teardown: a non-zero exit
//! status is returned to the caller (`BoundedOutput::status`) for it to turn
//! into a domain error; `kill`/`wait` failures during teardown are best-effort
//! and only logged. Output that could not be captured completely (a pipe read
//! error, a reader that never reported) is [`RunError::Output`], never an empty
//! or silently truncated buffer; only the documented byte cap truncates.

use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How often the supervisor polls the child for exit.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// After the child is gone, how long to wait for the output readers to drain.
/// Only matters if a descendant escaped the process group and still holds the
/// pipe open; the reader thread is then detached instead of blocking forever.
const READER_GRACE: Duration = Duration::from_secs(2);

/// Captured result of a child that exited on its own.
#[derive(Debug)]
pub(crate) struct BoundedOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Why [`run_bounded`] did not produce an exit status.
#[derive(Debug)]
pub(crate) enum RunError {
    /// The binary could not be started (missing, not executable, ...).
    Spawn(std::io::Error),
    /// The child outlived the deadline and was killed and reaped.
    Timeout(Duration),
    /// Polling the child failed; the child was killed and reaped.
    Wait(std::io::Error),
    /// The child exited but one of its output streams could not be captured.
    Output {
        stream: &'static str,
        reason: String,
    },
}

/// Run `command` to completion or until `timeout`, capturing at most
/// `stdout_cap` / `stderr_cap` bytes of each stream (the rest is drained and
/// discarded so the child never blocks on a full pipe).
pub(crate) fn run_bounded(
    command: &mut Command,
    timeout: Duration,
    stdout_cap: usize,
    stderr_cap: usize,
) -> Result<BoundedOutput, RunError> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Own process group, so a timeout kills wrappers' children too.
        command.process_group(0);
    }

    let mut child = ChildGuard::new(command.spawn().map_err(RunError::Spawn)?);
    let stdout_rx = drain(child.child.stdout.take(), stdout_cap, "stdout");
    let stderr_rx = drain(child.child.stderr.take(), stderr_cap, "stderr");

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.poll_exit() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(error) => {
                child.terminate();
                return Err(RunError::Wait(error));
            }
        }
        if Instant::now() >= deadline {
            child.terminate();
            return Err(RunError::Timeout(timeout));
        }
        std::thread::sleep(POLL_INTERVAL);
    };

    // `poll_exit` already killed stragglers in the group *before* reaping the
    // leader, so the readers see EOF.
    Ok(BoundedOutput {
        status,
        stdout: collect_output(&stdout_rx, "stdout", READER_GRACE)?,
        stderr: collect_output(&stderr_rx, "stderr", READER_GRACE)?,
    })
}

/// Wait up to `grace` for a reader's result. A read error, a reader that never
/// reports, or one that vanished (spawn failure) is an error: an empty buffer
/// would be indistinguishable from a child that printed nothing.
fn collect_output(
    rx: &mpsc::Receiver<std::io::Result<Vec<u8>>>,
    stream: &'static str,
    grace: Duration,
) -> Result<Vec<u8>, RunError> {
    let reason = match rx.recv_timeout(grace) {
        Ok(Ok(bytes)) => return Ok(bytes),
        Ok(Err(error)) => format!("read failed: {error}"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            format!("reader did not finish within {grace:?} (a descendant may still hold the pipe)")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => "output reader unavailable".to_string(),
    };
    Err(RunError::Output { stream, reason })
}

fn drain<R: Read + Send + 'static>(
    stream: Option<R>,
    cap: usize,
    name: &'static str,
) -> mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::channel();
    let Some(mut stream) = stream else {
        let _ = tx.send(Ok(Vec::new()));
        return rx;
    };
    let spawned = std::thread::Builder::new()
        .name(format!("engram-child-{name}"))
        .spawn(move || {
            let mut kept = Vec::new();
            let mut chunk = [0_u8; 8192];
            let outcome = loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break Ok(kept),
                    Ok(n) => {
                        let room = cap.saturating_sub(kept.len());
                        kept.extend_from_slice(&chunk[..n.min(room)]);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(error) => break Err(error),
                }
            };
            // A closed receiver only means the supervisor stopped waiting.
            let _ = tx.send(outcome);
        });
    if let Err(error) = spawned {
        // The sender was dropped with the closure, so `collect_output` reports it.
        tracing::warn!(target = "engram::multimodal::process", %error, stream = name, "output reader spawn failed");
    }
    rx
}

/// Kills and reaps the child (and its process group) unless it was already
/// reaped. Runs on drop, so unwinding or an early return cannot orphan it.
struct ChildGuard {
    child: Child,
    reaped: bool,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self {
            child,
            reaped: false,
        }
    }

    /// Non-blocking exit check that, on unix, kills the rest of the process
    /// group *before* the leader is reaped. A reaped leader's pid (= the group
    /// id) may be recycled; signalling `-pid` after reaping could hit an
    /// unrelated group. While the zombie leader exists the group id is ours.
    fn poll_exit(&mut self) -> std::io::Result<Option<ExitStatus>> {
        #[cfg(unix)]
        {
            if !self.leader_exited()? {
                return Ok(None);
            }
            self.kill_group();
        }
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status)
    }

    /// True once the leader has exited, without reaping it (`WNOWAIT`).
    #[cfg(unix)]
    fn leader_exited(&self) -> std::io::Result<bool> {
        // SAFETY: `siginfo_t` is plain data for which all-zero is valid, and
        // `waitid` only writes into it. `WNOHANG | WNOWAIT` neither blocks nor
        // reaps the child.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let rc = unsafe {
            libc::waitid(
                libc::P_PID,
                self.child.id() as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        // SAFETY: after a successful waitid, `si_pid` is valid to read.
        let pid = unsafe { info.si_pid() };
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let pid = info.si_pid;
        Ok(pid != 0)
    }

    fn kill_group(&self) {
        #[cfg(unix)]
        {
            // SAFETY: `kill(2)` with a negative pid targets the process group
            // we created with `process_group(0)`; it has no memory-safety
            // preconditions and a failure (ESRCH: group already gone) is fine.
            let _ = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
        }
    }

    fn terminate(&mut self) {
        if self.reaped {
            return;
        }
        self.kill_group();
        if let Err(error) = self.child.kill() {
            // InvalidInput means it already exited; anything else is logged.
            if error.kind() != std::io::ErrorKind::InvalidInput {
                tracing::warn!(target = "engram::multimodal::process", %error, "child kill failed");
            }
        }
        if let Err(error) = self.child.wait() {
            tracing::warn!(target = "engram::multimodal::process", %error, "child wait failed");
        }
        self.reaped = true;
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.terminate();
    }
}

#[cfg(all(test, unix))]
pub(crate) mod test_support {
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::time::Duration;

    /// Write an executable shell script. The executable itself is written by a
    /// separate `cp` process: if this test process opened it for writing, a
    /// `fork` by a concurrent test could inherit that descriptor until its
    /// `exec`, and running the script would fail with ETXTBSY ("Text file
    /// busy"), as seen on Linux.
    pub(crate) fn write_script(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let staged = dir.join(format!(".{name}.staged"));
        std::fs::write(&staged, format!("#!/bin/sh\n{body}\n")).unwrap();
        let copied = std::process::Command::new("cp")
            .arg(&staged)
            .arg(&path)
            .status()
            .unwrap();
        assert!(copied.success(), "cp {} failed", staged.display());
        std::fs::remove_file(&staged).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// True if `pid` is still running after a short grace period. `kill(2)`
    /// returns before the target has exited, and a killed orphan stays a
    /// signalable zombie until its new parent reaps it, so a single
    /// `kill(pid, 0)` right after SIGKILL reports a dead process as alive
    /// (seen on Linux CI runners). Zombies count as gone.
    pub(crate) fn still_running_after_grace(pid: i32) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while running(pid) {
            if std::time::Instant::now() >= deadline {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }

    fn running(pid: i32) -> bool {
        // SAFETY: signal 0 only checks existence/permission.
        let exists = unsafe { libc::kill(pid, 0) == 0 };
        exists && !is_zombie(pid)
    }

    #[cfg(target_os = "linux")]
    fn is_zombie(pid: i32) -> bool {
        // /proc/<pid>/stat is "pid (comm) state ..."; comm may contain ')'.
        std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .and_then(|stat| {
                let rest = &stat[stat.rfind(')')? + 1..];
                rest.trim_start().chars().next()
            })
            == Some('Z')
    }

    #[cfg(not(target_os = "linux"))]
    fn is_zombie(pid: i32) -> bool {
        std::process::Command::new("ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .output()
            .map(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .trim_start()
                    .starts_with('Z')
            })
            .unwrap_or(false)
    }

    pub(crate) fn read_pid(path: &Path) -> i32 {
        for _ in 0..200 {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(pid) = text.trim().parse() {
                    return pid;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("pid file {} never appeared", path.display());
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::test_support::{read_pid, still_running_after_grace, write_script};
    use super::*;

    #[test]
    fn success_captures_bounded_output() {
        let dir = tempfile::tempdir().unwrap();
        let script = write_script(
            dir.path(),
            "ok.sh",
            "echo hello; i=0; while [ $i -lt 2000 ]; do echo err-line-$i >&2; i=$((i+1)); done",
        );
        let out = run_bounded(&mut Command::new(script), Duration::from_secs(10), 1024, 64)
            .expect("runs");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello");
        assert_eq!(out.stderr.len(), 64, "stderr must be capped, not blocked");
    }

    #[test]
    fn timeout_kills_child_and_descendants() {
        let dir = tempfile::tempdir().unwrap();
        let child_pid = dir.path().join("child.pid");
        let grand_pid = dir.path().join("grand.pid");
        let script = write_script(
            dir.path(),
            "hang.sh",
            &format!(
                "echo $$ > {c}\nsleep 300 &\necho $! > {g}\nwait",
                c = child_pid.display(),
                g = grand_pid.display()
            ),
        );
        let started = Instant::now();
        let err = run_bounded(
            &mut Command::new(script),
            Duration::from_millis(400),
            1024,
            1024,
        )
        .expect_err("must time out");
        assert!(matches!(err, RunError::Timeout(_)));
        assert!(started.elapsed() < Duration::from_secs(5));
        let (c, g) = (read_pid(&child_pid), read_pid(&grand_pid));
        assert!(!still_running_after_grace(c), "child {c} left running");
        assert!(!still_running_after_grace(g), "grandchild {g} left running");
    }

    #[test]
    fn stragglers_left_by_an_exited_leader_are_killed_before_return() {
        let dir = tempfile::tempdir().unwrap();
        let held = dir.path().join("held.pid");
        let quiet = dir.path().join("quiet.pid");
        // The leader exits at once, leaving one background child that still
        // holds the output pipes and one that closed them.
        let script = write_script(
            dir.path(),
            "leader.sh",
            &format!(
                "sleep 300 &\necho $! > {h}\nsleep 300 >/dev/null 2>&1 &\necho $! > {q}\nexit 0",
                h = held.display(),
                q = quiet.display()
            ),
        );
        let started = Instant::now();
        let out = run_bounded(&mut Command::new(script), Duration::from_secs(10), 64, 64)
            .expect("leader exits cleanly");
        assert!(out.status.success());
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "waited for the straggler"
        );
        for (file, what) in [(&held, "pipe-holding"), (&quiet, "detached-quiet")] {
            let pid = read_pid(file);
            assert!(
                !still_running_after_grace(pid),
                "{what} straggler {pid} survived"
            );
        }
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("pipe broke"))
        }
    }

    struct InterruptedOnce(bool, &'static [u8]);

    impl Read for InterruptedOnce {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if !self.0 {
                self.0 = true;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            let n = self.1.len().min(buf.len());
            buf[..n].copy_from_slice(&self.1[..n]);
            self.1 = &self.1[n..];
            Ok(n)
        }
    }

    #[test]
    fn a_pipe_read_error_is_an_output_error_not_eof() {
        let rx = drain(Some(FailingReader), 64, "stdout");
        let err = collect_output(&rx, "stdout", READER_GRACE).expect_err("read error");
        assert!(
            matches!(&err, RunError::Output { stream: "stdout", reason } if reason.contains("pipe broke")),
            "{err:?}"
        );
    }

    #[test]
    fn an_interrupted_read_is_retried() {
        let rx = drain(Some(InterruptedOnce(false, b"data")), 64, "stdout");
        assert_eq!(
            collect_output(&rx, "stdout", READER_GRACE).unwrap(),
            b"data"
        );
    }

    #[test]
    fn a_reader_that_never_reports_is_an_output_error_not_empty_output() {
        let (_tx, rx) = mpsc::channel::<std::io::Result<Vec<u8>>>();
        let err = collect_output(&rx, "stderr", Duration::from_millis(20)).expect_err("grace");
        assert!(
            matches!(
                err,
                RunError::Output {
                    stream: "stderr",
                    ..
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn a_vanished_reader_is_an_output_error_not_empty_output() {
        let (tx, rx) = mpsc::channel::<std::io::Result<Vec<u8>>>();
        drop(tx);
        let err = collect_output(&rx, "stdout", READER_GRACE).expect_err("disconnected");
        assert!(
            matches!(
                err,
                RunError::Output {
                    stream: "stdout",
                    ..
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn spawn_error_is_reported() {
        let err = run_bounded(
            &mut Command::new("/nonexistent/engram-bin"),
            Duration::from_secs(1),
            16,
            16,
        )
        .expect_err("spawn must fail");
        assert!(matches!(err, RunError::Spawn(_)));
    }
}
