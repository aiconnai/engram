//! CLI compression handlers for AAAK ultra-dense token reduction.

use std::fs;
use std::path::Path;

use engram::error::{EngramError, Result};
use engram::intelligence::aaak::{AaakCompressor, AaakMode};
use engram::storage::queries::get_memory;
use engram::storage::Storage;

pub fn handle_compress(
    storage: &Storage,
    input: Option<String>,
    is_file: bool,
    memory_id: Option<i64>,
    mode_str: &str,
) -> Result<()> {
    let mode = mode_str
        .parse::<AaakMode>()
        .map_err(EngramError::InvalidInput)?;

    let text = if let Some(mem_id) = memory_id {
        storage.with_connection(|conn| get_memory(conn, mem_id).map(|m| m.content))?
    } else if let Some(inp) = input {
        if is_file {
            let expanded = shellexpand::tilde(&inp).to_string();
            fs::read_to_string(Path::new(&expanded)).map_err(|e| {
                EngramError::InvalidInput(format!("Failed to read file '{}': {}", expanded, e))
            })?
        } else {
            inp
        }
    } else {
        return Err(EngramError::InvalidInput(
            "Must provide text input, --file <path>, or --memory-id <id>".to_string(),
        ));
    };

    let result = AaakCompressor::compress(&text, mode);

    println!("{}", result.compressed);
    eprintln!("\n────────────────────────────────────────────────────────────────────────");
    eprintln!(
        "📊 Original: {} bytes ({} tok) | Compressed: {} bytes ({} tok)",
        result.original_bytes,
        result.original_tokens,
        result.compressed_bytes,
        result.compressed_tokens
    );
    eprintln!(
        "⚡ Ratio: {:.2}x | Savings: {:.1}% | Mode: {:?}",
        result.compression_ratio, result.token_savings_pct, result.mode
    );
    eprintln!("────────────────────────────────────────────────────────────────────────");

    Ok(())
}

pub fn handle_decompress(input: Option<String>, is_file: bool) -> Result<()> {
    let text = if let Some(inp) = input {
        if is_file {
            let expanded = shellexpand::tilde(&inp).to_string();
            fs::read_to_string(Path::new(&expanded)).map_err(|e| {
                EngramError::InvalidInput(format!("Failed to read file '{}': {}", expanded, e))
            })?
        } else {
            inp
        }
    } else {
        return Err(EngramError::InvalidInput(
            "Must provide text input or --file <path>".to_string(),
        ));
    };

    let expanded = AaakCompressor::decompress(&text);
    println!("{}", expanded);

    Ok(())
}
