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

#[test]
fn http_body_limit_accepts_normal_body_and_rejects_limit_plus_one_before_parse() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn_with_env(
        &[
            "--transport",
            "http",
            "--http-port",
            &port.to_string(),
            "--http-api-key",
            "secret-key",
        ],
        &[("ENGRAM_HTTP_MAX_BODY_BYTES", "256")],
    )
    .expect("spawn body-limited server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    let normal = http_post_json(port, "/mcp", initialize_request(), Some("secret-key"))
        .expect("normal bounded request");
    assert_eq!(normal.status, 200, "response: {}", normal.raw);

    let notification = http_post_json(port, "/mcp", notification_request(), Some("secret-key"))
        .expect("bounded notification");
    assert_eq!(notification.status, 202, "response: {}", notification.raw);

    let oversized =
        http_post_raw(port, "/mcp", &[b'x'; 257], Some("secret-key")).expect("oversized request");
    assert_eq!(oversized.status, 413, "response: {}", oversized.raw);
}

#[test]
fn http_auth_rejects_oversized_body_before_collection_or_json_parse() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn_with_env(
        &[
            "--transport",
            "http",
            "--http-port",
            &port.to_string(),
            "--http-api-key",
            "secret-key",
        ],
        &[("ENGRAM_HTTP_MAX_BODY_BYTES", "64")],
    )
    .expect("spawn authenticated body-limited server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    let response = http_post_raw(port, "/v1/mcp", &[b'x'; 65], None)
        .expect("unauthenticated oversized request");
    assert_eq!(response.status, 401, "response: {}", response.raw);
}

#[test]
fn http_request_timeout_bounds_slow_body_setup_with_stable_response() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn_with_env(
        &[
            "--transport",
            "http",
            "--http-port",
            &port.to_string(),
            "--http-api-key",
            "secret-key",
        ],
        &[("ENGRAM_HTTP_REQUEST_TIMEOUT_MS", "100")],
    )
    .expect("spawn request-timeout server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    let response = http_post_slow_partial_body(port, "/mcp", 64, b"{", "secret-key")
        .expect("slow body should receive timeout response");
    assert_eq!(response.status, 408, "response: {}", response.raw);
    assert!(
        response.body.contains("Request Timeout"),
        "stable timeout body missing: {}",
        response.raw
    );

    // Cancellation of the timed-out extraction must leave the listener healthy.
    let healthy = http_post_json(port, "/mcp", initialize_request(), Some("secret-key"))
        .expect("healthy request after timeout");
    assert_eq!(healthy.status, 200, "response: {}", healthy.raw);
}

#[test]
fn http_timeout_bounds_sse_setup_but_not_established_stream() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn_with_env(
        &[
            "--transport",
            "http",
            "--http-port",
            &port.to_string(),
            "--http-api-key",
            "secret-key",
        ],
        &[("ENGRAM_HTTP_REQUEST_TIMEOUT_MS", "100")],
    )
    .expect("spawn SSE timeout server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    let mut stream = open_sse_stream(port, "secret-key").expect("authorized SSE setup");
    thread::sleep(Duration::from_millis(250));
    stream
        .set_nonblocking(true)
        .expect("set SSE stream nonblocking");
    let mut chunk = [0_u8; 256];
    loop {
        match stream.read(&mut chunk) {
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Ok(0) => panic!("established SSE stream was closed by setup timeout"),
            Ok(_) => continue,
            Err(error) => panic!("unexpected SSE read error: {error}"),
        }
    }

    let healthy = http_post_json(port, "/mcp", initialize_request(), Some("secret-key"))
        .expect("healthy MCP request while SSE remains open");
    assert_eq!(healthy.status, 200, "response: {}", healthy.raw);
}

#[test]
fn trusted_proxy_spoofed_forwarding_is_ignored_by_rate_key() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(&[
        "--transport",
        "http",
        "--http-port",
        &port.to_string(),
        "--http-api-key",
        "secret-key",
        "--http-rate-limit-rps",
        "1",
        "--http-rate-limit-burst",
        "1",
    ])
    .expect("spawn server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", "198.51.100.1")]
        )
        .unwrap()
        .status,
        200
    );
    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", "198.51.100.2")]
        )
        .unwrap()
        .status,
        429
    );
}

#[test]
fn trusted_proxy_cidr_honors_and_normalizes_forwarded_chain() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn_with_env(
        &[
            "--transport",
            "http",
            "--http-port",
            &port.to_string(),
            "--http-api-key",
            "secret-key",
            "--http-rate-limit-rps",
            "1",
            "--http-rate-limit-burst",
            "1",
        ],
        &[("ENGRAM_HTTP_TRUSTED_PROXIES", "127.0.0.0/8,10.0.0.0/8")],
    )
    .expect("spawn server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", "198.51.100.1, 10.1.2.3")]
        )
        .unwrap()
        .status,
        200
    );
    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", "198.51.100.2")]
        )
        .unwrap()
        .status,
        200
    );
    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", "198.51.100.1")]
        )
        .unwrap()
        .status,
        429
    );
}

#[test]
fn trusted_proxy_malformed_or_overlong_chain_falls_back_to_peer() {
    let _guard = HTTP_SECURITY_TEST_LOCK.lock().expect("test lock");
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn_with_env(
        &[
            "--transport",
            "http",
            "--http-port",
            &port.to_string(),
            "--http-api-key",
            "secret-key",
            "--http-rate-limit-rps",
            "1",
            "--http-rate-limit-burst",
            "1",
        ],
        &[("ENGRAM_HTTP_TRUSTED_PROXIES", "127.0.0.0/8")],
    )
    .expect("spawn server");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("readiness");

    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", "not-an-ip")]
        )
        .unwrap()
        .status,
        200
    );
    let overlong = std::iter::repeat_n("198.51.100.1", 33)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        http_post_json_with_headers(
            port,
            "/mcp",
            initialize_request(),
            Some("secret-key"),
            &[("X-Forwarded-For", &overlong)]
        )
        .unwrap()
        .status,
        429
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
        Self::spawn_with_env(args, &[])
    }

    fn spawn_with_env(args: &[&str], env: &[(&str, &str)]) -> std::io::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let mut command = base_command_in_tempdir(&temp_dir);
        command.args(args);
        command.envs(env.iter().copied());
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
    http_post_json_with_headers(port, path, body, bearer, &[])
}

fn http_post_json_with_headers(
    port: u16,
    path: &str,
    body: serde_json::Value,
    bearer: Option<&str>,
    headers: &[(&str, &str)],
) -> Result<HttpResponse, String> {
    let body = body.to_string();
    let auth = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let extra_headers = headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect::<String>();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth}{extra_headers}Connection: close\r\n\r\n{body}",
        body.len()
    );
    http_request(port, &request)
}

fn http_post_raw(
    port: u16,
    path: &str,
    body: &[u8],
    bearer: Option<&str>,
) -> Result<HttpResponse, String> {
    let auth = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth}Connection: close\r\n\r\n",
        body.len()
    );
    let mut request = headers.into_bytes();
    request.extend_from_slice(body);
    http_request_bytes(port, &request)
}

fn http_post_slow_partial_body(
    port: u16,
    path: &str,
    content_length: usize,
    partial_body: &[u8],
    bearer: &str,
) -> Result<HttpResponse, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|err| err.to_string())?;
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {content_length}\r\nAuthorization: Bearer {bearer}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(headers.as_bytes())
        .and_then(|_| stream.write_all(partial_body))
        .and_then(|_| stream.flush())
        .map_err(|err| err.to_string())?;
    read_http_response(&mut stream)
}

fn http_get(port: u16, path: &str, bearer: Option<&str>) -> Result<HttpResponse, String> {
    let auth = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}Connection: close\r\n\r\n");
    http_request(port, &request)
}

fn open_sse_stream(port: u16, bearer: &str) -> Result<TcpStream, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|err| err.to_string())?;
    let request = format!(
        "GET /v1/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {bearer}\r\nConnection: keep-alive\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|err| err.to_string())?;

    let mut raw = Vec::new();
    let mut byte = [0_u8; 1];
    while !raw.ends_with(b"\r\n\r\n") {
        stream
            .read_exact(&mut byte)
            .map_err(|err| err.to_string())?;
        raw.push(byte[0]);
        if raw.len() > 16 * 1024 {
            return Err("SSE response headers exceeded 16 KiB".to_string());
        }
    }
    let headers = String::from_utf8_lossy(&raw);
    if !headers.starts_with("HTTP/1.1 200")
        || !headers.to_ascii_lowercase().contains("text/event-stream")
    {
        return Err(format!("unexpected SSE response: {headers}"));
    }
    Ok(stream)
}

fn http_request(port: u16, request: &str) -> Result<HttpResponse, String> {
    http_request_bytes(port, request.as_bytes())
}

fn http_request_bytes(port: u16, request: &[u8]) -> Result<HttpResponse, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|err| err.to_string())?;
    stream.write_all(request).map_err(|err| err.to_string())?;
    stream.flush().map_err(|err| err.to_string())?;

    read_http_response(&mut stream)
}

fn read_http_response(stream: &mut TcpStream) -> Result<HttpResponse, String> {
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

fn notification_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
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
