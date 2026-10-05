//! Q2-B07: `engram-cli mcp install` must never destroy a client config it cannot
//! parse and must never overwrite an existing backup. Drives the real binary
//! against a throwaway HOME/cwd.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const STRICT_OTHER_SERVER: &str =
    r#"{"mcpServers":{"other-server":{"command":"other","args":["x"]}}}"#;
// Trailing comma: valid for editors that accept JSON5/JSONC, invalid for serde_json.
const LENIENT_OTHER_SERVER: &str =
    "{\n  // keep me\n  \"mcpServers\": {\n    \"other-server\": {\"command\": \"other\"},\n  },\n}\n";

struct Sandbox {
    home: TempDir,
    cwd: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            home: tempfile::tempdir().expect("home"),
            cwd: tempfile::tempdir().expect("cwd"),
        }
    }

    fn config(&self) -> PathBuf {
        self.home.path().join(".cursor").join("mcp.json")
    }

    fn write_config(&self, content: &str) {
        let path = self.config();
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, content).expect("write config");
    }

    fn install(&self, extra: &[&str]) -> Output {
        let db = self.home.path().join("memories.db");
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_engram-cli"));
        cmd.env_clear()
            .env("HOME", self.home.path())
            .env("ENGRAM_DB_PATH", &db)
            .current_dir(self.cwd.path())
            .args(["mcp", "install", "--client", "cursor"])
            .args(extra);
        cmd.output().expect("run engram-cli")
    }
}

fn backups(config: &Path) -> Vec<PathBuf> {
    let dir = config.parent().expect("parent");
    let prefix = format!("{}.bak", config.file_name().unwrap().to_string_lossy());
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .expect("read_dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(&prefix))
        })
        .collect();
    found.sort();
    found
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn install_refuses_unparseable_config_and_leaves_it_untouched() {
    let sb = Sandbox::new();
    sb.write_config(LENIENT_OTHER_SERVER);

    let out = sb.install(&[]);

    assert!(
        !out.status.success(),
        "install must fail on unparseable config; stdout={}",
        stdout(&out)
    );
    assert_eq!(
        fs::read_to_string(sb.config()).unwrap(),
        LENIENT_OTHER_SERVER,
        "unparseable config must not be modified"
    );
    let combined = format!("{}{}", stdout(&out), String::from_utf8_lossy(&out.stderr));
    assert!(
        combined.contains("--force"),
        "error must name the --force escape hatch: {combined}"
    );
}

#[test]
fn install_force_backs_up_then_replaces_unparseable_config() {
    let sb = Sandbox::new();
    sb.write_config(LENIENT_OTHER_SERVER);

    let out = sb.install(&["--force"]);

    assert!(out.status.success(), "stdout={}", stdout(&out));
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(sb.config()).unwrap()).unwrap();
    assert!(parsed["mcpServers"]["engram"].is_object());
    let baks = backups(&sb.config());
    assert_eq!(baks.len(), 1);
    assert_eq!(fs::read_to_string(&baks[0]).unwrap(), LENIENT_OTHER_SERVER);
}

#[test]
fn second_install_never_overwrites_the_first_backup() {
    let sb = Sandbox::new();
    sb.write_config(STRICT_OTHER_SERVER);

    assert!(sb.install(&[]).status.success());
    let baks = backups(&sb.config());
    assert_eq!(baks.len(), 1, "first run creates one backup");
    assert_eq!(fs::read_to_string(&baks[0]).unwrap(), STRICT_OTHER_SERVER);

    assert!(sb.install(&[]).status.success());
    let baks = backups(&sb.config());
    assert_eq!(baks.len(), 2, "second run must add a new backup: {baks:?}");
    assert_eq!(
        fs::read_to_string(&baks[0]).unwrap(),
        STRICT_OTHER_SERVER,
        "original backup must stay byte-identical"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(sb.config()).unwrap()).unwrap();
    assert!(parsed["mcpServers"]["other-server"].is_object());
    assert!(parsed["mcpServers"]["engram"].is_object());

    // A third run changes nothing: the installed content is already backed up,
    // so no duplicate backup is written, and no temp file is left behind.
    assert!(sb.install(&[]).status.success());
    assert_eq!(
        backups(&sb.config()).len(),
        2,
        "unchanged content must not create another backup"
    );
    let leftovers: Vec<_> = fs::read_dir(sb.config().parent().unwrap())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("engram-tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "atomic write left temp files: {leftovers:?}"
    );
}
