//! Multimodal processing support
//!
//! Provides vision capabilities for processing image data using
//! multiple AI provider backends (Google Gemini, OpenAI).
//!
//! # Feature Flags
//!
//! - `multimodal`: Enables this module (requires API key for a supported provider)

pub mod audio;
pub mod hashing;
pub(crate) mod process;
pub mod screenshot;
pub mod video;
pub mod vision;

use std::time::Duration;

/// Connect deadline for provider HTTP calls.
pub(crate) const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Whole-request deadline for provider HTTP calls (vision, transcription).
/// `reqwest::Client::new()` has none, so a stalled provider would block the
/// calling MCP handler indefinitely.
pub(crate) const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// HTTP client with explicit connect and total deadlines.
pub(crate) fn http_client() -> reqwest::Client {
    http_client_with(HTTP_CONNECT_TIMEOUT, HTTP_REQUEST_TIMEOUT)
}

pub(crate) fn http_client_with(connect: Duration, total: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(connect)
        .timeout(total)
        .build()
        // Building only fails if the TLS backend cannot initialise; the
        // default client would fail the same way on first use.
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[cfg(test)]
mod http_client_tests {
    use super::*;

    #[tokio::test]
    async fn stalled_provider_is_cut_off_by_the_client_deadline() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        // Accept and never answer.
        let _hold = tokio::spawn(async move {
            let _conn = listener.accept().await;
            std::future::pending::<()>().await;
        });
        let client = http_client_with(Duration::from_secs(2), Duration::from_millis(300));

        let started = std::time::Instant::now();
        let err = client
            .post(format!("http://{addr}/v1"))
            .body("{}")
            .send()
            .await
            .unwrap_err();

        assert!(err.is_timeout(), "{err}");
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn default_deadlines_are_bounded() {
        assert!(HTTP_REQUEST_TIMEOUT <= Duration::from_secs(300));
        assert!(HTTP_CONNECT_TIMEOUT < HTTP_REQUEST_TIMEOUT);
    }
}
