use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::document_ingest::DocumentSection;

const WALL_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerResponse {
    pub sections: Vec<DocumentSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub(crate) fn extract(
    input: &[u8],
    max_pages: usize,
    max_text_bytes: usize,
) -> crate::error::Result<Vec<DocumentSection>> {
    let worker = worker_path().ok_or_else(|| {
        crate::error::EngramError::InvalidInput(
            "PDF extraction worker is unavailable; install the packaged engram-pdf-worker"
                .to_string(),
        )
    })?;
    let mut command = Command::new(worker);
    command.args([
        "--max-pages",
        &max_pages.to_string(),
        "--max-text-bytes",
        &max_text_bytes.to_string(),
    ]);
    extract_with_command(
        command,
        input,
        max_pages,
        max_text_bytes,
        WALL_TIMEOUT,
        IoFault::NONE,
    )
}

#[derive(Clone, Copy)]
struct IoFault {
    writer_panics: bool,
    reader_spawn_fails: bool,
}

impl IoFault {
    const NONE: Self = Self {
        writer_panics: false,
        reader_spawn_fails: false,
    };
}

fn extract_with_command(
    mut command: Command,
    input: &[u8],
    max_pages: usize,
    max_text_bytes: usize,
    timeout: Duration,
    fault: IoFault,
) -> crate::error::Result<Vec<DocumentSection>> {
    let started = Instant::now();
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| worker_error(format!("failed to start: {error}")))?;
    let mut child = ManagedChild::new(child);
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| worker_error("stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| worker_error("stdout unavailable"))?;
    let response_limit = max_text_bytes
        .saturating_add(max_pages.saturating_mul(256))
        .saturating_add(4096);
    let input = input.to_vec();
    let writer = std::thread::Builder::new()
        .name("engram-pdf-stdin".to_string())
        .spawn(move || {
            if fault.writer_panics {
                panic!("injected PDF writer failure");
            }
            stdin.write_all(&input)
        })
        .map_err(|error| worker_error(format!("input writer spawn failed: {error}")))?;
    let reader = if fault.reader_spawn_fails {
        Err(std::io::Error::other("injected PDF reader spawn failure"))
    } else {
        std::thread::Builder::new()
            .name("engram-pdf-stdout".to_string())
            .spawn(move || {
                let mut bytes = Vec::new();
                stdout
                    .take(response_limit.saturating_add(1) as u64)
                    .read_to_end(&mut bytes)
                    .map(|_| bytes)
            })
    };
    let reader = match reader {
        Ok(reader) => reader,
        Err(error) => {
            child.terminate();
            let _ = writer.join();
            return Err(worker_error(format!(
                "response reader spawn failed: {error}"
            )));
        }
    };
    let status = loop {
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                child.terminate();
                let _ = join_io(writer, reader);
                return Err(worker_error(format!("wait failed: {error}")));
            }
        };
        if let Some(status) = status {
            child.reaped = true;
            break status;
        }
        if started.elapsed() >= timeout {
            child.terminate();
            let _ = join_io(writer, reader);
            return Err(worker_error(format!(
                "timed out after {} milliseconds",
                timeout.as_millis()
            )));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let (write_result, response) = join_io(writer, reader)?;
    if response.len() > response_limit {
        return Err(worker_error("response exceeded configured output limit"));
    }
    if !status.success() {
        return Err(worker_error(
            "terminated by a resource limit or parser failure",
        ));
    }
    write_result.map_err(|error| worker_error(format!("failed to send input: {error}")))?;
    let payload = serde_json::from_slice::<WorkerResponse>(&response)
        .map_err(|error| worker_error(format!("returned invalid response: {error}")))?;
    match payload.error {
        Some(error) => Err(crate::error::EngramError::InvalidInput(error)),
        None => Ok(payload.sections),
    }
}

type WriterHandle = std::thread::JoinHandle<std::io::Result<()>>;
type ReaderHandle = std::thread::JoinHandle<std::io::Result<Vec<u8>>>;

fn join_io(
    writer: WriterHandle,
    reader: ReaderHandle,
) -> crate::error::Result<(std::io::Result<()>, Vec<u8>)> {
    let write_result = writer.join();
    let read_result = reader.join();
    let write_result = write_result.map_err(|_| worker_error("input writer failed"))?;
    let response = read_result
        .map_err(|_| worker_error("response reader failed"))?
        .map_err(|error| worker_error(format!("response read failed: {error}")))?;
    Ok((write_result, response))
}

struct ManagedChild {
    child: Child,
    reaped: bool,
}

impl ManagedChild {
    fn new(child: Child) -> Self {
        Self {
            child,
            reaped: false,
        }
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    fn terminate(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
            self.reaped = true;
        }
    }
}

impl std::ops::Deref for ManagedChild {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl std::ops::DerefMut for ManagedChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn worker_error(message: impl std::fmt::Display) -> crate::error::EngramError {
    crate::error::EngramError::InvalidInput(format!("PDF extraction worker {message}"))
}

fn worker_path() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let name = if cfg!(windows) {
        "engram-pdf-worker.exe"
    } else {
        "engram-pdf-worker"
    };
    let directory = executable.parent()?;
    let sibling = directory.join(name);
    if sibling.is_file() {
        return Some(sibling);
    }
    directory
        .parent()
        .map(|parent| parent.join(name))
        .filter(|path| path.is_file())
}

mod runtime;
#[cfg(test)]
mod tests;

pub fn run() -> Result<(), String> {
    runtime::run()
}
