use std::path::Path;

use clap::Subcommand;
use engram::embedding::onnx_registry::{default_model_dir, find_model, REGISTRY};
use engram::error::{EngramError, Result};
use sha2::{Digest, Sha256};

#[derive(Subcommand)]
pub(crate) enum ModelAction {
    /// Download a local embedding model into the cache
    Download {
        /// Model registry name
        #[arg(default_value = "minilm-l6-v2")]
        name: String,
    },
    /// List local embedding models
    List,
    /// Print the local cache path for a model
    Path {
        /// Model registry name
        #[arg(default_value = "minilm-l6-v2")]
        name: String,
    },
}

pub(crate) fn handle_model_action(action: &ModelAction) -> Result<()> {
    match action {
        ModelAction::List => list_models(),
        ModelAction::Path { name } => print_model_path(name),
        ModelAction::Download { name } => download_model(name),
    }
}

fn list_models() -> Result<()> {
    for entry in REGISTRY {
        let dir = default_model_dir();
        let installed = model_files_present(&dir);
        let status = if installed {
            "installed"
        } else {
            "not installed"
        };
        println!(
            "{}\t{}\t{} dims\tmax_seq_len={}",
            entry.name, status, entry.dimensions, entry.max_seq_len
        );
    }
    Ok(())
}

fn print_model_path(name: &str) -> Result<()> {
    let entry = find_model(name).ok_or_else(|| {
        EngramError::InvalidInput(format!("Unknown local embedding model: {name}"))
    })?;
    let dir = default_model_dir_for(entry.name);
    println!("{}", dir.display());
    Ok(())
}

fn download_model(name: &str) -> Result<()> {
    let entry = find_model(name).ok_or_else(|| {
        EngramError::InvalidInput(format!("Unknown local embedding model: {name}"))
    })?;
    let dir = default_model_dir_for(entry.name);
    std::fs::create_dir_all(&dir)?;

    download_file(entry.model_url, entry.model_sha256, &dir.join("model.onnx"))?;
    download_file(
        entry.tokenizer_url,
        entry.tokenizer_sha256,
        &dir.join("tokenizer.json"),
    )?;

    println!("Downloaded {} to {}", entry.name, dir.display());
    Ok(())
}

fn default_model_dir_for(name: &str) -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("engram")
        .join("models")
        .join(name)
}

fn model_files_present(dir: &Path) -> bool {
    dir.join("model.onnx").is_file() && dir.join("tokenizer.json").is_file()
}

fn download_file(url: &str, expected_sha256: &str, target: &Path) -> Result<()> {
    if target.is_file()
        && sha256_file(target)? == expected_sha256_or_current(expected_sha256, target)?
    {
        println!("{} already present", target.display());
        return Ok(());
    }

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| EngramError::Config(format!("Failed to create download runtime: {e}")))?;
    let bytes = runtime.block_on(async {
        let response = reqwest::Client::new().get(url).send().await?;
        let response = response.error_for_status()?;
        response.bytes().await
    })?;

    let actual_hash = sha256_bytes(&bytes);
    if !expected_sha256.is_empty() && actual_hash != expected_sha256 {
        return Err(EngramError::Config(format!(
            "SHA-256 mismatch for {url}: expected {expected_sha256}, got {actual_hash}"
        )));
    }

    let tmp = target.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, target)?;
    println!("Downloaded {}", target.display());
    Ok(())
}

fn expected_sha256_or_current(expected: &str, target: &Path) -> Result<String> {
    if expected.is_empty() {
        sha256_file(target)
    } else {
        Ok(expected.to_string())
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
