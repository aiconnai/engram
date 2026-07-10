use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::config::{redact, start_error, RealServerConfig, StartError};

pub(crate) fn base_command(config: &RealServerConfig, db_path: &Path) -> Command {
    let mut command = Command::new(&config.executable);
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
        .arg("--ws-port")
        .arg("0")
        .env("RUST_LOG", "engram=info")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ENGRAM_CLOUD_STORAGE_URI")
        .env_remove("AWS_ACCESS_KEY_ID")
        .env_remove("AWS_SECRET_ACCESS_KEY")
        .env_remove("AWS_SESSION_TOKEN")
        .env("ENGRAM_EMBEDDING_MODEL", "tfidf")
        .env("ENGRAM_TOOL_TIER", "all")
        .env("ENGRAM_CLEANUP_INTERVAL", "0")
        .env("ENGRAM_EMBEDDING_DRAIN_INTERVAL", "0")
        .env("ENGRAM_COMPRESSION_INTERVAL", "0")
        .env("ENGRAM_WS_PORT", "0");
    command
}

pub(crate) fn pick_loopback_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

pub(crate) fn port_is_bindable(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub(crate) fn wait_until(timeout: Duration, mut predicate: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return true;
        }
        thread::sleep(Duration::from_millis(25));
    }
    predicate()
}

pub(crate) fn stop_child_and_collect_stderr(
    child: &mut Child,
    stderr_thread: &mut Option<JoinHandle<()>>,
    stderr_log: &Arc<Mutex<String>>,
) -> String {
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    let _ = child.wait();
    if let Some(stderr_thread) = stderr_thread.take() {
        let _ = stderr_thread.join();
    }
    stderr_log.lock().map(|log| log.clone()).unwrap_or_default()
}

pub(crate) fn stop_child_and_redact_stderr(
    child: &mut Child,
    stderr_thread: &mut Option<JoinHandle<()>>,
    stderr_log: &Arc<Mutex<String>>,
    api_key: &str,
) -> String {
    let stderr = stop_child_and_collect_stderr(child, stderr_thread, stderr_log);
    redact(&stderr, api_key)
}

pub(crate) fn stdio_timeout_error(
    child: &mut Child,
    stderr_thread: &mut Option<JoinHandle<()>>,
    stderr_log: &Arc<Mutex<String>>,
    api_key: &str,
) -> Result<String, StartError> {
    if let Some(status) = child.try_wait().map_err(start_error)? {
        return Err(StartError {
            message: format!(
                "engram-server exited before stdio response with status {status}; stderr: {}",
                stop_child_and_redact_stderr(child, stderr_thread, stderr_log, api_key)
            ),
        });
    }
    let stderr = stop_child_and_redact_stderr(child, stderr_thread, stderr_log, api_key);
    Err(StartError {
        message: format!("timed out waiting for stdio JSON-RPC response; stderr: {stderr}"),
    })
}
