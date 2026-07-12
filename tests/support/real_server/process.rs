use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use super::child_process::{
    base_command, pick_loopback_port, port_is_bindable, stdio_timeout_error,
    stop_child_and_redact_stderr, wait_until,
};
use super::config::{start_error, CleanupReport, RealServerConfig, StartError};
use super::http_client::{http_get, http_json_rpc};
use super::io::{spawn_stderr_reader, spawn_stdout_reader};

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(5);
const STDIO_RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

pub struct RealServer {
    child: Child,
    temp_dir: Option<TempDir>,
    db_path: PathBuf,
    api_key: String,
    port: Option<u16>,
    stdin: Option<ChildStdin>,
    stdout_rx: Option<mpsc::Receiver<Result<String, String>>>,
    stderr_log: Arc<Mutex<String>>,
    stderr_thread: Option<JoinHandle<()>>,
}

impl RealServer {
    pub fn start_stdio(config: RealServerConfig) -> Result<Self, StartError> {
        Self::start_stdio_in(config, tempfile::tempdir().map_err(start_error)?)
    }

    pub fn start_stdio_in(config: RealServerConfig, temp_dir: TempDir) -> Result<Self, StartError> {
        let db_path = temp_dir.path().join("stdio-memories.db");
        Self::start_stdio_with_state(config, db_path, Some(temp_dir))
    }

    #[allow(dead_code)] // Shared support is compiled independently by tests that do not reuse state.
    pub fn start_stdio_at(config: RealServerConfig, db_path: PathBuf) -> Result<Self, StartError> {
        Self::start_stdio_with_state(config, db_path, None)
    }

    fn start_stdio_with_state(
        config: RealServerConfig,
        db_path: PathBuf,
        temp_dir: Option<TempDir>,
    ) -> Result<Self, StartError> {
        let mut command = base_command(&config, &db_path);
        command.arg("--transport").arg("stdio");
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().map_err(start_error)?;
        let stdin = child.stdin.take().ok_or_else(|| StartError {
            message: "engram-server stdio stdin was not piped".to_string(),
        })?;
        let stdout = child.stdout.take().ok_or_else(|| StartError {
            message: "engram-server stdio stdout was not piped".to_string(),
        })?;
        let stderr = child.stderr.take();
        let stdout_rx = spawn_stdout_reader(stdout);
        let stderr_log = Arc::new(Mutex::new(String::new()));
        let stderr_thread =
            stderr.map(|stderr| spawn_stderr_reader(stderr, Arc::clone(&stderr_log)));

        Ok(Self {
            child,
            temp_dir,
            db_path,
            api_key: config.api_key,
            port: None,
            stdin: Some(stdin),
            stdout_rx: Some(stdout_rx),
            stderr_log,
            stderr_thread,
        })
    }

    pub fn start_http(config: RealServerConfig) -> Result<Self, StartError> {
        let temp_dir = tempfile::tempdir().map_err(start_error)?;
        let db_path = temp_dir.path().join("http-memories.db");
        Self::start_http_with_state(config, db_path, Some(temp_dir))
    }

    #[allow(dead_code)] // Shared support is compiled independently by tests that do not reuse state.
    pub fn start_http_at(config: RealServerConfig, db_path: PathBuf) -> Result<Self, StartError> {
        Self::start_http_with_state(config, db_path, None)
    }

    fn start_http_with_state(
        config: RealServerConfig,
        db_path: PathBuf,
        temp_dir: Option<TempDir>,
    ) -> Result<Self, StartError> {
        let port = pick_loopback_port().map_err(start_error)?;
        let mut command = base_command(&config, &db_path);
        command
            .arg("--transport")
            .arg("http")
            .arg("--http-port")
            .arg(port.to_string())
            .arg("--http-api-key")
            .arg(config.api_key.as_str());
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().map_err(start_error)?;
        let stderr = child.stderr.take();
        let stderr_log = Arc::new(Mutex::new(String::new()));
        let stderr_thread =
            stderr.map(|stderr| spawn_stderr_reader(stderr, Arc::clone(&stderr_log)));
        let executable = config.executable().to_path_buf();
        let mut server = Self {
            child,
            temp_dir,
            db_path,
            api_key: config.api_key,
            port: Some(port),
            stdin: None,
            stdout_rx: None,
            stderr_log,
            stderr_thread,
        };
        server.wait_for_http_ready(&executable)?;
        Ok(server)
    }

    pub fn temp_path(&self) -> &Path {
        self.temp_dir.as_ref().map_or_else(
            || self.db_path.parent().expect("database path has a parent"),
            TempDir::path,
        )
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn stdio_request(&mut self, request: Value) -> Result<Value, StartError> {
        let stdin = self.stdin.as_mut().ok_or_else(|| StartError {
            message: "stdio request attempted against non-stdio server".to_string(),
        })?;
        writeln!(stdin, "{request}").map_err(start_error)?;
        stdin.flush().map_err(start_error)?;
        let line = self.read_stdio_line()?;
        serde_json::from_str(line.trim()).map_err(start_error)
    }

    pub fn http_json_rpc(&self, request: Value) -> Result<Value, StartError> {
        let port = self.port.ok_or_else(|| StartError {
            message: "HTTP request attempted against non-HTTP server".to_string(),
        })?;
        http_json_rpc(port, Some(&self.api_key), request)
    }

    pub fn shutdown_and_verify(mut self) -> CleanupReport {
        let child_id = self.child.id();
        let port = self.port;
        let temp_path = self.temp_path().to_path_buf();
        let owns_temp_dir = self.temp_dir.is_some();
        let redacted_stderr = stop_child_and_redact_stderr(
            &mut self.child,
            &mut self.stderr_thread,
            &self.stderr_log,
            &self.api_key,
        );
        drop(self.temp_dir.take());
        let temp_removed = owns_temp_dir && wait_until(CLEANUP_TIMEOUT, || !temp_path.exists());
        let port_released = port.map(|p| wait_until(CLEANUP_TIMEOUT, || port_is_bindable(p)));
        CleanupReport {
            child_id,
            temp_path,
            temp_removed,
            port,
            port_released,
            redacted_stderr,
        }
    }

    fn read_stdio_line(&mut self) -> Result<String, StartError> {
        let receive_result = self
            .stdout_rx
            .as_ref()
            .ok_or_else(|| StartError {
                message: "stdio response attempted against non-stdio server".to_string(),
            })?
            .recv_timeout(STDIO_RESPONSE_TIMEOUT);
        match receive_result {
            Ok(Ok(line)) if !line.trim().is_empty() => Ok(line),
            Ok(Ok(_)) => Err(StartError {
                message: "engram-server closed stdio before returning JSON-RPC response"
                    .to_string(),
            }),
            Ok(Err(error)) => Err(StartError { message: error }),
            Err(mpsc::RecvTimeoutError::Timeout) => stdio_timeout_error(
                &mut self.child,
                &mut self.stderr_thread,
                &self.stderr_log,
                &self.api_key,
            ),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(StartError {
                message: "engram-server stdio stdout closed".to_string(),
            }),
        }
    }

    fn wait_for_http_ready(&mut self, executable: &Path) -> Result<(), StartError> {
        let port = self.port.expect("HTTP server must have a port");
        let deadline = Instant::now() + READY_TIMEOUT;
        let mut last_error = "HTTP server was not probed".to_string();
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().map_err(start_error)? {
                return Err(StartError {
                    message: format!(
                        "engram-server exited before HTTP readiness with status {status}; stderr: {}",
                        self.take_redacted_stderr()
                    ),
                });
            }
            match http_get(port, "/health") {
                Ok(response) if response.contains("\"status\":\"ok\"") => return Ok(()),
                Ok(response) => last_error = format!("unexpected /health response: {response}"),
                Err(error) => last_error = error.to_string(),
            }
            thread::sleep(POLL_INTERVAL);
        }
        let stderr = self.take_redacted_stderr();
        Err(StartError {
            message: format!(
                "timed out waiting for HTTP readiness from {}: {last_error}; stderr: {stderr}",
                executable.display(),
            ),
        })
    }

    fn take_redacted_stderr(&mut self) -> String {
        stop_child_and_redact_stderr(
            &mut self.child,
            &mut self.stderr_thread,
            &self.stderr_log,
            &self.api_key,
        )
    }
}

impl Drop for RealServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
