use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StartError {
    pub(crate) message: String,
}

impl std::fmt::Display for StartError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StartError {}

#[derive(Clone, Debug)]
pub struct RealServerConfig {
    pub(crate) executable: PathBuf,
    pub(crate) api_key: String,
}

impl RealServerConfig {
    pub fn from_cargo_bin() -> Self {
        Self {
            executable: cargo_bin_path(),
            api_key: "real-server-harness-secret".to_string(),
        }
    }

    pub fn with_executable(mut self, executable: PathBuf) -> Self {
        self.executable = executable;
        self
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn executable(&self) -> &Path {
        &self.executable
    }
}

pub struct CleanupReport {
    pub child_id: u32,
    pub temp_path: PathBuf,
    pub temp_removed: bool,
    pub port: Option<u16>,
    pub port_released: Option<bool>,
    pub redacted_stderr: String,
}

pub(crate) fn start_error(error: impl std::fmt::Display) -> StartError {
    StartError {
        message: error.to_string(),
    }
}

pub(crate) fn redact(input: &str, api_key: &str) -> String {
    input.replace(api_key, "<redacted-api-key>")
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

pub fn bad_executable_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("engram-real-server-missing-{}", std::process::id()));
    path
}
