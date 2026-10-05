//! Redaction of values before they reach logs.
//!
//! Policy (task O1): a log line may say *what kind* of failure happened, never
//! *what the payload was*. Error `Display` output routinely embeds proprietary
//! content (provider response bodies that echo the prompt, request URLs with
//! API keys in the query string, filesystem paths, SQL fragments), so logs use
//! [`redacted`] (error class only) and [`PathLabel`] (no path) instead of `%err`
//! / `path.display()`.
//!
//! This only affects logs. MCP responses, CLI output and persisted job errors
//! keep their full text: callers need it to act, and they are not a log stream.
//!
//! Operators can opt in to full detail for a debugging session with
//! `ENGRAM_LOG_ERROR_DETAIL=1` (never on by default, never meant for shared logs).

use std::fmt;
use std::path::Path;

use crate::error::EngramError;

/// Environment switch for verbose error detail in logs (default off).
pub const DETAIL_ENV: &str = "ENGRAM_LOG_ERROR_DETAIL";

/// Whether the operator opted in to full error detail in logs.
pub fn detail_enabled() -> bool {
    parse_detail_flag(std::env::var(DETAIL_ENV).ok().as_deref())
}

pub(crate) fn parse_detail_flag(raw: Option<&str>) -> bool {
    matches!(
        raw.map(|v| v.trim().to_ascii_lowercase()).as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

/// `true` when `err` is a SQLite busy/locked condition.
pub fn is_sqlite_busy(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(e, _)
            if matches!(
                e.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            )
    )
}

fn text_mentions_timeout(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("timed out") || lower.contains("timeout")
}

/// Best-effort: did a provider/network call fail because it timed out?
///
/// Typed where possible (`reqwest::Error::is_timeout`, `io::ErrorKind::TimedOut`),
/// otherwise a text heuristic on provider-wrapped messages. The text is only
/// inspected, never logged.
pub fn is_timeout(err: &EngramError) -> bool {
    match err {
        EngramError::Io(e) => e.kind() == std::io::ErrorKind::TimedOut,
        #[cfg(any(feature = "openai", feature = "multimodal", feature = "onnx-embed"))]
        EngramError::Http(e) => e.is_timeout(),
        #[cfg(not(any(feature = "openai", feature = "multimodal", feature = "onnx-embed")))]
        EngramError::Http(message) => text_mentions_timeout(message),
        EngramError::Embedding(message)
        | EngramError::Internal(message)
        | EngramError::CloudStorage(message)
        | EngramError::Sync(message) => text_mentions_timeout(message),
        _ => false,
    }
}

/// Stable, low-cardinality class of an error. Never contains payload text.
pub fn error_class(err: &EngramError) -> &'static str {
    if is_timeout(err) {
        return "timeout";
    }
    match err {
        EngramError::Database(e) if is_sqlite_busy(e) => "database_busy",
        EngramError::Database(_) => "database",
        EngramError::Storage(_) => "storage",
        EngramError::NotFound(_) => "not_found",
        EngramError::InvalidInput(_) => "invalid_input",
        EngramError::Embedding(_) => "embedding",
        EngramError::Search(_) => "search",
        EngramError::Sync(_) => "sync",
        EngramError::CloudStorage(_) => "cloud_storage",
        EngramError::Encryption(_) => "encryption",
        EngramError::Serialization(_) => "serialization",
        EngramError::Io(_) => "io",
        EngramError::Http(_) => "http",
        EngramError::Config(_) => "config",
        EngramError::Conflict(_) | EngramError::Duplicate { .. } => "conflict",
        EngramError::Auth(_) | EngramError::Unauthorized(_) => "unauthorized",
        EngramError::RateLimited(_) => "rate_limited",
        EngramError::Internal(_) => "internal",
    }
}

/// `Display` adapter: the error class, or the full error when detail is on.
pub struct Redacted<'a> {
    err: &'a EngramError,
    detail: bool,
}

impl fmt::Display for Redacted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail {
            write!(f, "{}: {}", error_class(self.err), self.err)
        } else {
            f.write_str(error_class(self.err))
        }
    }
}

/// Render `err` for a log field: `error = %redacted(&err)`.
pub fn redacted(err: &EngramError) -> Redacted<'_> {
    redacted_with(err, detail_enabled())
}

/// Same as [`redacted`] with an explicit detail switch (used by tests).
pub fn redacted_with(err: &EngramError, detail: bool) -> Redacted<'_> {
    Redacted { err, detail }
}

/// `Display` adapter for a filesystem path: `<path>` unless detail is on.
pub struct PathLabel<'a> {
    path: &'a Path,
    detail: bool,
}

impl fmt::Display for PathLabel<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail {
            write!(f, "{}", self.path.display())
        } else {
            f.write_str("<path>")
        }
    }
}

/// Render a path for a log field: `path = %path_label(&p)`.
pub fn path_label(path: &Path) -> PathLabel<'_> {
    path_label_with(path, detail_enabled())
}

pub fn path_label_with(path: &Path, detail: bool) -> PathLabel<'_> {
    PathLabel { path, detail }
}

/// `Display` adapter for any value whose text may hold proprietary content
/// (a free-form name, a message): `<redacted>` unless detail is on.
pub struct Opaque<'a, T: fmt::Display + ?Sized> {
    value: &'a T,
    detail: bool,
}

impl<T: fmt::Display + ?Sized> fmt::Display for Opaque<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail {
            self.value.fmt(f)
        } else {
            f.write_str("<redacted>")
        }
    }
}

/// Render an arbitrary displayable value: `name = %opaque(&name)`.
pub fn opaque<T: fmt::Display + ?Sized>(value: &T) -> Opaque<'_, T> {
    Opaque {
        value,
        detail: detail_enabled(),
    }
}

/// Class of an `io::Error` for logs: the kind, never the message (which can
/// embed paths).
pub fn io_class(err: &std::io::Error) -> String {
    format!("{:?}", err.kind())
}

/// Class of a tokio task failure for logs, without the panic payload.
pub fn join_error_class(err: &tokio::task::JoinError) -> &'static str {
    if err.is_panic() {
        "task_panicked"
    } else if err.is_cancelled() {
        "task_cancelled"
    } else {
        "task_failed"
    }
}

/// Log a panic without its payload (the payload can hold request content).
/// Installed by `engram-server` so the default hook does not print it to
/// stderr, which is the server's log stream.
pub fn install_redacting_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if detail_enabled() {
            previous(info);
            return;
        }
        let (file, line) = info
            .location()
            .map(|l| (l.file(), l.line()))
            .unwrap_or(("<unknown>", 0));
        tracing::error!(
            target: "engram::panic",
            file,
            line,
            "panic (payload redacted; set {DETAIL_ENV}=1 for detail)"
        );
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_text_never_appears_in_default_rendering() {
        let err = EngramError::Embedding("API error 500: secret-body".to_string());
        assert_eq!(redacted_with(&err, false).to_string(), "embedding");
        let detailed = redacted_with(&err, true).to_string();
        assert!(
            detailed.contains("secret-body"),
            "opt-in detail: {detailed}"
        );
    }

    #[test]
    fn timeouts_are_classified_without_logging_the_message() {
        let err = EngramError::Embedding("request timed out after 30s: body".to_string());
        assert!(is_timeout(&err));
        assert_eq!(redacted_with(&err, false).to_string(), "timeout");
        let io = EngramError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "x"));
        assert!(is_timeout(&io));
        assert!(!is_timeout(&EngramError::Config("bad".to_string())));
    }

    #[test]
    fn busy_database_is_its_own_class() {
        let busy = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            None,
        );
        assert!(is_sqlite_busy(&busy));
        assert_eq!(error_class(&EngramError::Database(busy)), "database_busy");
        assert!(!is_sqlite_busy(&rusqlite::Error::QueryReturnedNoRows));
    }

    #[test]
    fn paths_and_opaque_values_are_hidden_by_default() {
        let p = Path::new("/home/someone/secret.db");
        assert_eq!(path_label_with(p, false).to_string(), "<path>");
        assert_eq!(
            path_label_with(p, true).to_string(),
            "/home/someone/secret.db"
        );
        assert_eq!(
            Opaque {
                value: "name",
                detail: false
            }
            .to_string(),
            "<redacted>"
        );
    }

    #[test]
    fn detail_flag_parsing_defaults_to_off() {
        assert!(!parse_detail_flag(None));
        assert!(!parse_detail_flag(Some("")));
        assert!(!parse_detail_flag(Some("0")));
        assert!(parse_detail_flag(Some(" TRUE ")));
        assert!(parse_detail_flag(Some("1")));
    }
}
