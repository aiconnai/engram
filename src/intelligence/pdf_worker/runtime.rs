use std::io::{Read, Write};

use super::WorkerResponse;

const MAX_INPUT_BYTES: u64 = 10 * 1024 * 1024;
#[cfg(all(unix, not(target_os = "macos")))]
const MEMORY_BYTES: u64 = 512 * 1024 * 1024;
const CPU_SECONDS: u64 = 3;

pub(super) fn run() -> Result<(), String> {
    let limits = parse_args(std::env::args().skip(1))?;
    apply_resource_limits()?;
    let mut input = Vec::new();
    std::io::stdin()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut input)
        .map_err(|error| format!("failed to read PDF input: {error}"))?;
    if input.len() as u64 > MAX_INPUT_BYTES {
        return Err(format!("PDF input exceeds {MAX_INPUT_BYTES} bytes"));
    }
    let response = match crate::intelligence::document_ingest::extract_pdf_sections_in_worker(
        &input,
        limits.max_pages,
        limits.max_text_bytes,
    ) {
        Ok(sections) => WorkerResponse {
            sections,
            error: None,
        },
        Err(error) => WorkerResponse {
            sections: Vec::new(),
            error: Some(error.to_string()),
        },
    };
    let response = serde_json::to_vec(&response)
        .map_err(|error| format!("failed to encode worker response: {error}"))?;
    std::io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write worker response: {error}"))
}

struct WorkerLimits {
    max_pages: usize,
    max_text_bytes: usize,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<WorkerLimits, String> {
    let mut max_pages = None;
    let mut max_text_bytes = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--max-pages" => {
                max_pages = Some(value.parse().map_err(|_| "invalid max pages".to_string())?)
            }
            "--max-text-bytes" => {
                max_text_bytes = Some(
                    value
                        .parse()
                        .map_err(|_| "invalid max text bytes".to_string())?,
                )
            }
            _ => return Err(format!("unknown worker argument: {flag}")),
        }
    }
    Ok(WorkerLimits {
        max_pages: max_pages.ok_or_else(|| "missing --max-pages".to_string())?,
        max_text_bytes: max_text_bytes.ok_or_else(|| "missing --max-text-bytes".to_string())?,
    })
}

#[cfg(unix)]
fn apply_resource_limits() -> Result<(), String> {
    apply_memory_limit()?;
    rlimit::Resource::CPU
        .set(CPU_SECONDS, CPU_SECONDS)
        .map_err(|error| format!("failed to set PDF worker CPU limit: {error}"))?;
    rlimit::Resource::FSIZE
        .set(0, 0)
        .map_err(|error| format!("failed to set PDF worker file limit: {error}"))?;
    rlimit::Resource::NOFILE
        .set(16, 16)
        .map_err(|error| format!("failed to set PDF worker descriptor limit: {error}"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn apply_memory_limit() -> Result<(), String> {
    rlimit::Resource::AS
        .set(MEMORY_BYTES, MEMORY_BYTES)
        .map_err(|error| format!("failed to set PDF worker memory limit: {error}"))
}

#[cfg(target_os = "macos")]
fn apply_memory_limit() -> Result<(), String> {
    Err(
        "PDF extraction is unavailable on macOS because a hard process memory limit cannot be enforced"
            .to_string(),
    )
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    #[test]
    fn macos_pdf_worker_fails_closed_without_hard_memory_limit() {
        let error = super::apply_memory_limit().expect_err("macOS must fail closed");
        assert!(error.contains("hard process memory limit"));
    }
}

#[cfg(not(unix))]
fn apply_resource_limits() -> Result<(), String> {
    Err("PDF extraction is unavailable without worker resource limits on this platform".to_string())
}
