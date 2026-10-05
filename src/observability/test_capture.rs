//! Test-only capture of every `tracing` event emitted in this test binary.
//!
//! Redaction tests assert that sentinel values never appear in captured log
//! output. The subscriber is process-global (installed once) because handlers
//! log from `spawn_blocking` and background threads; each test therefore uses a
//! sentinel unique to that test instead of resetting a shared buffer.

use std::io;
use std::sync::{Arc, Mutex, Once, OnceLock};

use tracing_subscriber::fmt::MakeWriter;

static INSTALL: Once = Once::new();
static BUFFER: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

fn buffer() -> &'static Arc<Mutex<Vec<u8>>> {
    BUFFER.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for SharedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Ok(mut buf) = self.0.lock() {
            buf.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Install the capturing subscriber (idempotent). TRACE level, no ANSI colors,
/// so a test sees everything a `RUST_LOG=trace` operator would.
pub(crate) fn install() {
    INSTALL.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .with_writer(SharedWriter(buffer().clone()))
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .finish();
        // Another test module may have installed a subscriber first; capture is
        // best-effort then and the assertions below would fail loudly.
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// Everything logged so far by any test in this binary.
pub(crate) fn captured() -> String {
    buffer()
        .lock()
        .map(|buf| String::from_utf8_lossy(&buf).into_owned())
        .unwrap_or_default()
}

/// Assert none of `needles` appears in the captured logs, and that capture is
/// live (a canary event is visible), so an empty buffer cannot pass silently.
#[track_caller]
pub(crate) fn assert_logs_exclude(needles: &[&str]) {
    tracing::info!(target: "engram::observability::canary", "capture canary");
    let logs = captured();
    assert!(
        logs.contains("capture canary"),
        "log capture is not active; got {} bytes",
        logs.len()
    );
    for needle in needles {
        assert!(
            !logs.contains(needle),
            "log output leaked {needle:?}; matching lines:\n{}",
            logs.lines()
                .filter(|line| line.contains(needle))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
