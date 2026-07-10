//! Listener configuration contract tests for `engram-server`.
//!
//! These tests exercise the real binary so bind defaults and clap parsing stay
//! observable at the process boundary.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use tempfile::TempDir;

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
static LISTENER_CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn help_exposes_loopback_bind_address_defaults_and_stdio_transport_default() {
    let _guard = listener_config_test_guard();
    // Given: the real engram-server binary.
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let output = base_command_in_tempdir(&temp_dir)
        .arg("--help")
        .output()
        .expect("run engram-server --help");

    // When: clap renders the CLI contract.
    let stdout = String::from_utf8(output.stdout).expect("help stdout is UTF-8");

    // Then: stdio remains the default transport, every network listener has an
    // explicit bind-address knob, loopback is the documented default, and WS
    // remains disabled by default at port 0.
    assert!(output.status.success(), "--help should succeed: {stdout}");
    assert!(
        stdout.contains("ENGRAM_TRANSPORT"),
        "transport env missing: {stdout}"
    );
    assert!(
        stdout.contains("[default: stdio]"),
        "stdio default missing: {stdout}"
    );
    assert!(
        stdout.contains("--http-bind-address"),
        "HTTP bind flag missing: {stdout}"
    );
    assert!(
        stdout.contains("ENGRAM_HTTP_BIND_ADDRESS"),
        "HTTP bind env missing: {stdout}"
    );
    assert!(
        stdout.contains("--ws-bind-address"),
        "WS bind flag missing: {stdout}"
    );
    assert!(
        stdout.contains("ENGRAM_WS_BIND_ADDRESS"),
        "WS bind env missing: {stdout}"
    );
    assert!(
        stdout.contains("[default: 127.0.0.1]"),
        "loopback default missing: {stdout}"
    );
    assert!(
        stdout.contains("--ws-port"),
        "WS port flag missing: {stdout}"
    );
    assert!(
        stdout.contains("[default: 0]"),
        "WS disabled default missing: {stdout}"
    );

    #[cfg(feature = "grpc")]
    {
        assert!(
            stdout.contains("--grpc-bind-address"),
            "gRPC bind flag missing: {stdout}"
        );
        assert!(
            stdout.contains("ENGRAM_GRPC_BIND_ADDRESS"),
            "gRPC bind env missing: {stdout}"
        );
    }
}

#[test]
fn default_startup_uses_stdio_without_enabling_websocket_listener() {
    let _guard = listener_config_test_guard();
    // Given: a real server process with no transport or websocket flags.
    let mut process = ServerProcess::spawn(&[], &[]).expect("spawn default stdio server");

    // When: an initialize request is sent over stdio.
    let response = process
        .stdio_request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "listener-config-test", "version": "0.0.0"}
            }
        }))
        .expect("stdio initialize response");

    // Then: the process is really using stdio by default, and the port-0 WS
    // default did not open or announce a websocket listener.
    assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
    thread::sleep(Duration::from_millis(100));
    let stderr = process.stderr_snapshot();
    assert!(
        !stderr.contains("WebSocket server listening"),
        "WS listener should stay disabled at default port 0: {stderr}"
    );
}

#[test]
fn http_listener_defaults_to_loopback_and_accepts_explicit_public_bind_address() {
    let _guard = listener_config_test_guard();
    // Given: an HTTP transport with no bind-address override.
    let default_port = pick_loopback_port();
    let mut default_process = ServerProcess::spawn(
        &[
            "--transport",
            "http",
            "--http-port",
            &default_port.to_string(),
        ],
        &[],
    )
    .expect("spawn default HTTP server");

    // When: startup completes.
    default_process
        .wait_for_log(&format!(
            "HTTP transport listening on 127.0.0.1:{default_port}"
        ))
        .expect("default HTTP loopback log");

    // Then: the default listener is loopback and reachable through loopback.
    assert_http_health(default_port);
    drop(default_process);

    // Given: a caller explicitly requests a non-loopback bind address.
    let explicit_port = pick_loopback_port();
    let mut explicit_process = ServerProcess::spawn(
        &[
            "--transport",
            "http",
            "--http-bind-address",
            "0.0.0.0",
            "--http-port",
            &explicit_port.to_string(),
        ],
        &[],
    )
    .expect("spawn explicit HTTP server");

    // When: startup completes.
    explicit_process
        .wait_for_log(&format!(
            "HTTP transport listening on 0.0.0.0:{explicit_port}"
        ))
        .expect("explicit HTTP bind log");

    // Then: the explicit address was passed unchanged to the listener path.
    assert_http_health(explicit_port);
}

#[test]
fn http_bind_address_env_is_parsed_before_socket_startup() {
    let _guard = listener_config_test_guard();
    // Given: an HTTP transport configured via environment instead of CLI.
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(
        &["--transport", "http", "--http-port", &port.to_string()],
        &[("ENGRAM_HTTP_BIND_ADDRESS", "0.0.0.0")],
    )
    .expect("spawn HTTP server from env bind address");

    // When: startup completes.
    process
        .wait_for_log(&format!("HTTP transport listening on 0.0.0.0:{port}"))
        .expect("HTTP env bind log");

    // Then: the env-provided address reaches the listener unchanged.
    assert_http_health(port);
}

#[test]
fn websocket_listener_uses_configured_loopback_address_when_enabled() {
    let _guard = listener_config_test_guard();
    // Given: a websocket listener enabled on a free port with an explicit env bind address.
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(
        &["--ws-port", &port.to_string()],
        &[("ENGRAM_WS_BIND_ADDRESS", "127.0.0.1")],
    )
    .expect("spawn stdio server with websocket listener");

    // When: the websocket server starts.
    process
        .wait_for_log(&format!("WebSocket server listening on 127.0.0.1:{port}"))
        .expect("websocket loopback log");

    // Then: the listener accepts local TCP connections on that exact port.
    TcpStream::connect(("127.0.0.1", port)).expect("connect to websocket listener");
}

#[test]
fn invalid_bind_address_is_rejected_before_any_socket_opens() {
    let _guard = listener_config_test_guard();
    // Given: a free port and an invalid HTTP bind-address value.
    let port = pick_loopback_port();

    // When: clap parses the invalid value.
    let output = base_command_in_tempdir(&tempfile::tempdir().expect("tempdir"))
        .arg("--transport")
        .arg("http")
        .arg("--http-port")
        .arg(port.to_string())
        .arg("--http-bind-address")
        .arg("not-an-ip-address")
        .output()
        .expect("run invalid bind-address command");

    // Then: startup fails before the listener can bind the requested port.
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(!output.status.success(), "invalid bind address should fail");
    assert!(
        stderr.contains("invalid value"),
        "clap parse error missing: {stderr}"
    );
    assert!(
        stderr.contains("--http-bind-address"),
        "flag name missing: {stderr}"
    );
    assert!(port_is_bindable(port), "port {port} should remain unopened");
}

#[cfg(feature = "grpc")]
#[test]
fn grpc_listener_defaults_to_loopback_address() {
    let _guard = listener_config_test_guard();
    // Given: a gRPC transport with no bind-address override.
    let port = pick_loopback_port();
    let mut process = ServerProcess::spawn(
        &["--transport", "grpc", "--grpc-port", &port.to_string()],
        &[],
    )
    .expect("spawn gRPC server");

    // When: startup announces the configured address and the socket accepts TCP.
    process
        .wait_for_log(&format!("gRPC transport listening on 127.0.0.1:{port}"))
        .expect("gRPC loopback log");
    wait_for_tcp_connect(port, &process).expect("gRPC socket readiness");

    // Then: the gRPC listener is bound on loopback.
    wait_for_tcp_connect(port, &process).expect("connect to grpc listener");
}

struct ServerProcess {
    child: Child,
    _temp_dir: TempDir,
    stderr: Arc<Mutex<String>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
    stdout_rx: Option<mpsc::Receiver<String>>,
}

impl ServerProcess {
    fn spawn(args: &[&str], envs: &[(&str, &str)]) -> std::io::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let mut command = base_command_in_tempdir(&temp_dir);
        command.args(args);
        for (key, value) in envs {
            command.env(key, value);
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn()?;
        let stdout_rx = child.stdout.take().map(spawn_stdout_reader);
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
            stdout_rx,
        })
    }

    fn stdio_request(&mut self, request: serde_json::Value) -> Result<serde_json::Value, String> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| "server stdin is not piped".to_string())?;
        writeln!(stdin, "{request}").map_err(|err| err.to_string())?;
        stdin.flush().map_err(|err| err.to_string())?;
        let line = self
            .stdout_rx
            .as_ref()
            .ok_or_else(|| "server stdout is not piped".to_string())?
            .recv_timeout(IO_TIMEOUT)
            .map_err(|err| format!("{err}; stderr: {}", self.stderr_snapshot()))?;
        serde_json::from_str(&line).map_err(|err| err.to_string())
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

fn base_command_in_tempdir(temp_dir: &TempDir) -> Command {
    let db_path = temp_dir.path().join("listener-config.db");
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

fn listener_config_test_guard() -> std::sync::MutexGuard<'static, ()> {
    LISTENER_CONFIG_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn spawn_stdout_reader(stdout: impl Read + Send + 'static) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let _ = tx.send(line);
                }
                Err(_) => break,
            }
        }
    });
    rx
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

#[cfg(feature = "grpc")]
fn wait_for_tcp_connect(port: u16, process: &ServerProcess) -> Result<(), String> {
    let deadline = Instant::now() + READY_TIMEOUT;
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(_) => return Ok(()),
            Err(error) => last_error = error.to_string(),
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(format!(
        "port {port} never accepted TCP connections: {last_error}; stderr: {}",
        process.stderr_snapshot()
    ))
}

fn assert_http_health(port: u16) {
    let deadline = Instant::now() + READY_TIMEOUT;
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match http_get(port, "/health") {
            Ok(response) if response.contains("\"status\":\"ok\"") => return,
            Ok(response) => last_error = response,
            Err(error) => last_error = error,
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("HTTP health never became ready on port {port}: {last_error}");
}

fn http_get(port: u16, path: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| err.to_string())?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|err| err.to_string())?;
    stream.flush().map_err(|err| err.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| err.to_string())?;
    Ok(response)
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
