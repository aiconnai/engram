//! HTTP transport security contract tests.
//!
//! These tests use the real `engram-server` binary so listener fail-closed
//! behavior is observed at the process boundary, not inferred from unit tests.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use tempfile::TempDir;

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
static HTTP_SECURITY_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn public_http_without_key_exits_before_listening() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(&[
        "--transport",
        "http",
        "--http-bind-address",
        "0.0.0.0",
        "--http-port",
        &port.to_string(),
    ])
    .expect("spawn public HTTP server without key");

    let status = process.wait_for_exit().expect("server should exit");
    let stderr = process.stderr_snapshot();

    assert!(!status.success(), "public no-key startup should fail");
    assert!(
        stderr.contains("HTTP transport requires ENGRAM_HTTP_API_KEY")
            || stderr.contains("public HTTP listener"),
        "expected fail-closed auth error, got: {stderr}"
    );
    assert!(
        port_is_bindable(port),
        "port {port} should not be listening"
    );
}

#[test]
fn loopback_no_key_preserves_local_anonymous_mcp() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(&[
        "--transport",
        "http",
        "--http-bind-address",
        "127.0.0.1",
        "--http-port",
        &port.to_string(),
    ])
    .expect("spawn loopback HTTP server without key");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("loopback HTTP readiness");

    let response = http_post_json(port, "/v1/mcp", initialize_request(), None)
        .expect("loopback anonymous initialize");

    assert_eq!(response.status, 200, "response: {}", response.raw);
    let body: serde_json::Value = serde_json::from_str(&response.body).expect("json body");
    assert_eq!(body["result"]["protocolVersion"], "2025-11-25");
}

#[test]
fn loopback_anonymous_memory_search_rejects_cross_workspace_shapes() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(&[
        "--transport",
        "http",
        "--http-bind-address",
        "127.0.0.1",
        "--http-port",
        &port.to_string(),
    ])
    .expect("spawn loopback HTTP server without key");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("loopback HTTP readiness");

    for arguments in [
        json!({"query": "secret"}),
        json!({"query": "secret", "global": true}),
        json!({"query": "secret", "workspaces": ["default", "private"]}),
        json!({"query": "secret", "filters": {"workspace": "private"}}),
        json!({"query": "secret", "filters": [{"global": true}, {"workspace": "private"}]}),
    ] {
        let response = http_post_json(port, "/v1/mcp", tool_call_request(arguments), None)
            .expect("anonymous memory_search request");
        assert_eq!(response.status, 403, "response: {}", response.raw);
    }
}

#[test]
fn keyed_memory_search_preserves_cross_workspace_shapes() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(&[
        "--transport",
        "http",
        "--http-bind-address",
        "127.0.0.1",
        "--http-port",
        &port.to_string(),
        "--http-api-key",
        "secret-key",
    ])
    .expect("spawn loopback HTTP server with key");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("loopback HTTP readiness");

    for arguments in [
        json!({"query": "secret"}),
        json!({"query": "secret", "global": true}),
        json!({"query": "secret", "workspaces": ["default", "private"]}),
        json!({"query": "secret", "filters": {"workspace": "private"}}),
        json!({"query": "secret", "filters": [{"global": true}, {"workspace": "private"}]}),
    ] {
        let response = http_post_json(
            port,
            "/v1/mcp",
            tool_call_request(arguments),
            Some("secret-key"),
        )
        .expect("keyed memory_search request");
        assert_eq!(response.status, 200, "response: {}", response.raw);
    }
}

#[test]
fn public_http_with_key_authenticates_mcp_and_sse() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(&[
        "--transport",
        "http",
        "--http-bind-address",
        "0.0.0.0",
        "--http-port",
        &port.to_string(),
        "--http-api-key",
        "secret-key",
    ])
    .expect("spawn public HTTP server with key");
    process
        .wait_for_log(&format!("HTTP transport listening on 0.0.0.0:{port}"))
        .expect("public keyed HTTP readiness");

    let missing = http_post_json(port, "/v1/mcp", initialize_request(), None)
        .expect("missing bearer request");
    assert_eq!(missing.status, 401, "response: {}", missing.raw);

    let wrong = http_post_json(port, "/v1/mcp", initialize_request(), Some("wrong"))
        .expect("wrong bearer request");
    assert_eq!(wrong.status, 401, "response: {}", wrong.raw);

    let authorized = http_post_json(port, "/v1/mcp", initialize_request(), Some("secret-key"))
        .expect("authorized initialize");
    assert_eq!(authorized.status, 200, "response: {}", authorized.raw);
    let body: serde_json::Value = serde_json::from_str(&authorized.body).expect("json body");
    assert_eq!(body["result"]["protocolVersion"], "2025-11-25");

    let sse_missing = http_get(port, "/v1/events", None).expect("missing SSE bearer");
    assert_eq!(sse_missing.status, 401, "response: {}", sse_missing.raw);

    let sse_authorized = http_get(port, "/v1/events", Some("secret-key")).expect("authorized SSE");
    assert_eq!(
        sse_authorized.status, 200,
        "response: {}",
        sse_authorized.raw
    );
    assert!(
        sse_authorized
            .raw
            .to_ascii_lowercase()
            .contains("content-type: text/event-stream"),
        "SSE content-type missing: {}",
        sse_authorized.raw
    );
}

struct ServerProcess {
    child: Child,
    _temp_dir: TempDir,
    stderr: Arc<Mutex<String>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
}

impl ServerProcess {
    fn spawn(args: &[&str]) -> std::io::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let mut command = base_command_in_tempdir(&temp_dir);
        command.args(args);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = command.spawn()?;
        let stderr = Arc::new(Mutex::new(String::new()));
        let stderr_thread = child
            .stderr
            .take()
            .map(|reader| spawn_stderr_reader(reader, Arc::clone(&stderr)));

        Ok(Self {
            child,
            _temp_dir: temp_dir,
            stderr,
            stderr_thread,
        })
    }

    fn wait_for_log(&mut self, needle: &str) -> Result<(), String> {
        let deadline = Instant::now() + READY_TIMEOUT;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().map_err(|err| err.to_string())? {
                return Err(format!(
                    "server exited with {status}; stderr: {}",
                    self.stderr_snapshot()
                ));
            }
            if self.stderr_snapshot().contains(needle) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(25));
        }
        Err(format!(
            "timed out waiting for log `{needle}`; stderr: {}",
            self.stderr_snapshot()
        ))
    }

    fn wait_for_exit(&mut self) -> Result<std::process::ExitStatus, String> {
        let deadline = Instant::now() + READY_TIMEOUT;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().map_err(|err| err.to_string())? {
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(25));
        }
        Err(format!(
            "timed out waiting for process exit; stderr: {}",
            self.stderr_snapshot()
        ))
    }

    fn stderr_snapshot(&self) -> String {
        self.stderr
            .lock()
            .map(|log| log.clone())
            .unwrap_or_default()
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(thread) = self.stderr_thread.take() {
            let _ = thread.join();
        }
    }
}

struct HttpResponse {
    status: u16,
    body: String,
    raw: String,
}

fn http_post_json(
    port: u16,
    path: &str,
    body: serde_json::Value,
    bearer: Option<&str>,
) -> Result<HttpResponse, String> {
    let body = body.to_string();
    let auth = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth}Connection: close\r\n\r\n{body}",
        body.len()
    );
    http_request(port, &request)
}

fn http_get(port: u16, path: &str, bearer: Option<&str>) -> Result<HttpResponse, String> {
    let auth = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}Connection: close\r\n\r\n");
    http_request(port, &request)
}

fn http_request(port: u16, request: &str) -> Result<HttpResponse, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|err| err.to_string())?;
    stream
        .write_all(request.as_bytes())
        .map_err(|err| err.to_string())?;
    stream.flush().map_err(|err| err.to_string())?;

    let mut raw = String::new();
    let mut buf = [0_u8; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                raw.push_str(&String::from_utf8_lossy(&buf[..n]));
                if raw.contains("\r\n\r\n")
                    && raw.to_ascii_lowercase().contains("text/event-stream")
                {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                if raw.is_empty() {
                    return Err(err.to_string());
                }
                break;
            }
            Err(err) => return Err(err.to_string()),
        }
    }

    let status = raw
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or_else(|| format!("missing HTTP status in response: {raw}"))?;
    let body = raw
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();
    Ok(HttpResponse { status, body, raw })
}

fn initialize_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "http-security-test", "version": "0.0.0"}
        }
    })
}

fn tool_call_request(arguments: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {"name": "memory_search", "arguments": arguments}
    })
}

fn base_command_in_tempdir(temp_dir: &TempDir) -> Command {
    let db_path = temp_dir.path().join("http-security.db");
    let mut command = Command::new(cargo_bin_path());
    command.env_clear();
    preserve_process_env(&mut command, "PATH");
    preserve_process_env(&mut command, "TMPDIR");
    command
        .arg("--db-path")
        .arg(db_path)
        .arg("--embedding-model")
        .arg("tfidf")
        .arg("--cleanup-interval-seconds")
        .arg("0")
        .arg("--embedding-drain-interval-seconds")
        .arg("0")
        .arg("--compression-interval-seconds")
        .arg("0")
        .env("RUST_LOG", "engram=info")
        .env("ENGRAM_TOOL_TIER", "all")
        .env("ENGRAM_EMBEDDING_MODEL", "tfidf")
        .env("ENGRAM_CLEANUP_INTERVAL", "0")
        .env("ENGRAM_EMBEDDING_DRAIN_INTERVAL", "0")
        .env("ENGRAM_COMPRESSION_INTERVAL", "0");
    command
}

fn preserve_process_env(command: &mut Command, key: &str) {
    if let Some(value) = std::env::var_os(key) {
        command.env(key, value);
    }
}

fn spawn_stderr_reader(
    stderr: impl Read + Send + 'static,
    log: Arc<Mutex<String>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(mut guard) = log.lock() {
                        guard.push_str(&line);
                    }
                }
                Err(_) => break,
            }
        }
    })
}

fn pick_loopback_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback port 0");
    listener.local_addr().expect("read local addr").port()
}

fn port_is_bindable(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn cargo_bin_path() -> PathBuf {
    option_env!("CARGO_BIN_EXE_engram-server")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("target");
            path.push("debug");
            path.push(if cfg!(windows) {
                "engram-server.exe"
            } else {
                "engram-server"
            });
            path
        })
}
