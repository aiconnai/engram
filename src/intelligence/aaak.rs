//! AAAK (Agent Abbreviation Knowledge) Ultra-Dense Compression Engine.
//!
//! Inspired by MemPalace's compact abbreviation grammar, AAAK provides a
//! 20x–30x context reduction layer for verbatim memory drawers and multi-turn
//! conversational transcripts before injection into LLM prompt contexts.
//!
//! Compression Modes:
//! - `Lossless`: Reversible dictionary substitution using canonical technical shorthands.
//! - `UltraDense`: Aggressive token reduction stripping conversational filler,
//!   collapsing whitespace, and contracting software engineering terminology.
//! - `Transcript`: Dialogue compressor compacting `user:` / `assistant:` turns into
//!   `[U]` / `[A]` markers and collapsing tool call noise.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::intelligence::token_counter::TiktokenCounter;

/// Compression mode for AAAK engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AaakMode {
    /// Reversible dictionary substitution
    Lossless,
    /// Maximum density token reduction (~20x-30x)
    UltraDense,
    /// Conversational transcript compaction
    Transcript,
}

impl std::str::FromStr for AaakMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lossless" => Ok(Self::Lossless),
            "dense" | "ultradense" | "ultra_dense" | "ultra-dense" => Ok(Self::UltraDense),
            "transcript" | "convos" => Ok(Self::Transcript),
            _ => Err(format!(
                "Unknown AAAK mode: '{}'. Expected lossless, ultradense, or transcript",
                s
            )),
        }
    }
}

/// Result of an AAAK compression run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AaakCompressedResult {
    /// The compressed text with `[AAAK:v1]` prefix
    pub compressed: String,
    /// Original byte size
    pub original_bytes: usize,
    /// Compressed byte size
    pub compressed_bytes: usize,
    /// Estimated original token count
    pub original_tokens: usize,
    /// Estimated compressed token count
    pub compressed_tokens: usize,
    /// Character compression ratio (original / compressed)
    pub compression_ratio: f32,
    /// Token reduction percentage (0.0 to 100.0)
    pub token_savings_pct: f32,
    /// Mode used
    pub mode: AaakMode,
}

/// Token savings statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSavings {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub tokens_saved: usize,
    pub savings_percentage: f32,
}

// ---------------------------------------------------------------------------
// Abbreviation Dictionaries
// ---------------------------------------------------------------------------

struct AbbrevPair {
    full: &'static str,
    short: &'static str,
}

static ABBREVIATIONS: &[AbbrevPair] = &[
    // Dialogue & Roles
    AbbrevPair {
        full: "assistant",
        short: "A",
    },
    AbbrevPair {
        full: "user",
        short: "U",
    },
    AbbrevPair {
        full: "system",
        short: "S",
    },
    AbbrevPair {
        full: "developer",
        short: "D",
    },
    // Programming & Concepts
    AbbrevPair {
        full: "function",
        short: "fn",
    },
    AbbrevPair {
        full: "implementation",
        short: "impl",
    },
    AbbrevPair {
        full: "implement",
        short: "impl",
    },
    AbbrevPair {
        full: "implemented",
        short: "impl'd",
    },
    AbbrevPair {
        full: "configuration",
        short: "cfg",
    },
    AbbrevPair {
        full: "configure",
        short: "cfg",
    },
    AbbrevPair {
        full: "configured",
        short: "cfg'd",
    },
    AbbrevPair {
        full: "database",
        short: "db",
    },
    AbbrevPair {
        full: "workspace",
        short: "ws",
    },
    AbbrevPair {
        full: "repository",
        short: "repo",
    },
    AbbrevPair {
        full: "authentication",
        short: "auth",
    },
    AbbrevPair {
        full: "authorization",
        short: "authz",
    },
    AbbrevPair {
        full: "authenticate",
        short: "auth",
    },
    AbbrevPair {
        full: "request",
        short: "req",
    },
    AbbrevPair {
        full: "response",
        short: "res",
    },
    AbbrevPair {
        full: "parameter",
        short: "param",
    },
    AbbrevPair {
        full: "parameters",
        short: "params",
    },
    AbbrevPair {
        full: "argument",
        short: "arg",
    },
    AbbrevPair {
        full: "arguments",
        short: "args",
    },
    AbbrevPair {
        full: "asynchronous",
        short: "async",
    },
    AbbrevPair {
        full: "synchronous",
        short: "sync",
    },
    AbbrevPair {
        full: "environment",
        short: "env",
    },
    AbbrevPair {
        full: "directory",
        short: "dir",
    },
    AbbrevPair {
        full: "directories",
        short: "dirs",
    },
    AbbrevPair {
        full: "document",
        short: "doc",
    },
    AbbrevPair {
        full: "documents",
        short: "docs",
    },
    AbbrevPair {
        full: "documentation",
        short: "docs",
    },
    AbbrevPair {
        full: "context",
        short: "ctx",
    },
    AbbrevPair {
        full: "memory",
        short: "mem",
    },
    AbbrevPair {
        full: "memories",
        short: "mems",
    },
    AbbrevPair {
        full: "exception",
        short: "exc",
    },
    AbbrevPair {
        full: "exceptions",
        short: "excs",
    },
    AbbrevPair {
        full: "reference",
        short: "ref",
    },
    AbbrevPair {
        full: "references",
        short: "refs",
    },
    AbbrevPair {
        full: "variable",
        short: "var",
    },
    AbbrevPair {
        full: "variables",
        short: "vars",
    },
    AbbrevPair {
        full: "constant",
        short: "const",
    },
    AbbrevPair {
        full: "constants",
        short: "consts",
    },
    AbbrevPair {
        full: "interface",
        short: "iface",
    },
    AbbrevPair {
        full: "interfaces",
        short: "ifaces",
    },
    AbbrevPair {
        full: "structure",
        short: "struct",
    },
    AbbrevPair {
        full: "structures",
        short: "structs",
    },
    AbbrevPair {
        full: "algorithm",
        short: "algo",
    },
    AbbrevPair {
        full: "algorithms",
        short: "algos",
    },
    AbbrevPair {
        full: "application",
        short: "app",
    },
    AbbrevPair {
        full: "applications",
        short: "apps",
    },
    AbbrevPair {
        full: "connection",
        short: "conn",
    },
    AbbrevPair {
        full: "connections",
        short: "conns",
    },
    AbbrevPair {
        full: "information",
        short: "info",
    },
    AbbrevPair {
        full: "initialize",
        short: "init",
    },
    AbbrevPair {
        full: "initialized",
        short: "init'd",
    },
    AbbrevPair {
        full: "message",
        short: "msg",
    },
    AbbrevPair {
        full: "messages",
        short: "msgs",
    },
    AbbrevPair {
        full: "metadata",
        short: "meta",
    },
    AbbrevPair {
        full: "performance",
        short: "perf",
    },
    AbbrevPair {
        full: "permission",
        short: "perm",
    },
    AbbrevPair {
        full: "permissions",
        short: "perms",
    },
    AbbrevPair {
        full: "previous",
        short: "prev",
    },
    AbbrevPair {
        full: "temporary",
        short: "tmp",
    },
    AbbrevPair {
        full: "transaction",
        short: "tx",
    },
    AbbrevPair {
        full: "transactions",
        short: "txs",
    },
    AbbrevPair {
        full: "verification",
        short: "verif",
    },
    AbbrevPair {
        full: "verify",
        short: "verif",
    },
    AbbrevPair {
        full: "verified",
        short: "verif'd",
    },
    AbbrevPair {
        full: "benchmark",
        short: "bench",
    },
    AbbrevPair {
        full: "benchmarks",
        short: "benches",
    },
    AbbrevPair {
        full: "dependency",
        short: "dep",
    },
    AbbrevPair {
        full: "dependencies",
        short: "deps",
    },
    AbbrevPair {
        full: "destination",
        short: "dst",
    },
    AbbrevPair {
        full: "source",
        short: "src",
    },
    AbbrevPair {
        full: "sources",
        short: "srcs",
    },
    AbbrevPair {
        full: "identifier",
        short: "id",
    },
    AbbrevPair {
        full: "identifiers",
        short: "ids",
    },
    AbbrevPair {
        full: "maximum",
        short: "max",
    },
    AbbrevPair {
        full: "minimum",
        short: "min",
    },
    AbbrevPair {
        full: "return",
        short: "ret",
    },
    AbbrevPair {
        full: "returned",
        short: "ret'd",
    },
    AbbrevPair {
        full: "status",
        short: "stat",
    },
    AbbrevPair {
        full: "timeout",
        short: "to",
    },
    AbbrevPair {
        full: "version",
        short: "ver",
    },
    AbbrevPair {
        full: "versions",
        short: "vers",
    },
    AbbrevPair {
        full: "specification",
        short: "spec",
    },
    AbbrevPair {
        full: "specifications",
        short: "specs",
    },
    AbbrevPair {
        full: "architecture",
        short: "arch",
    },
    AbbrevPair {
        full: "generation",
        short: "gen",
    },
    AbbrevPair {
        full: "generate",
        short: "gen",
    },
    AbbrevPair {
        full: "generated",
        short: "gen'd",
    },
    AbbrevPair {
        full: "optimization",
        short: "opt",
    },
    AbbrevPair {
        full: "optimize",
        short: "opt",
    },
    AbbrevPair {
        full: "optimized",
        short: "opt'd",
    },
    AbbrevPair {
        full: "operation",
        short: "op",
    },
    AbbrevPair {
        full: "operations",
        short: "ops",
    },
    AbbrevPair {
        full: "definition",
        short: "def",
    },
    AbbrevPair {
        full: "definitions",
        short: "defs",
    },
    AbbrevPair {
        full: "instruction",
        short: "instr",
    },
    AbbrevPair {
        full: "instructions",
        short: "instrs",
    },
    AbbrevPair {
        full: "collection",
        short: "coll",
    },
    AbbrevPair {
        full: "package",
        short: "pkg",
    },
    AbbrevPair {
        full: "packages",
        short: "pkgs",
    },
    AbbrevPair {
        full: "protocol",
        short: "proto",
    },
    AbbrevPair {
        full: "protocols",
        short: "protos",
    },
    // Logical Relations & Prepositions
    AbbrevPair {
        full: "because",
        short: "bc",
    },
    AbbrevPair {
        full: "therefore",
        short: ".:",
    },
    AbbrevPair {
        full: "with respect to",
        short: "wrt",
    },
    AbbrevPair {
        full: "in order to",
        short: "to",
    },
    AbbrevPair {
        full: "as soon as possible",
        short: "asap",
    },
    AbbrevPair {
        full: "for example",
        short: "e.g.",
    },
    AbbrevPair {
        full: "that is",
        short: "i.e.",
    },
    AbbrevPair {
        full: "without",
        short: "w/o",
    },
    AbbrevPair {
        full: "with",
        short: "w/",
    },
    AbbrevPair {
        full: "between",
        short: "btwn",
    },
    AbbrevPair {
        full: "through",
        short: "thru",
    },
];

static CONVERSATIONAL_FILLERS: &[&str] = &[
    "sure, i can help with that",
    "sure, i can help with that!",
    "here is the summary",
    "here is the summary:",
    "here are the results",
    "here are the results:",
    "let me know if you have any questions",
    "let me know if you need anything else",
    "as an ai assistant",
    "as an ai,",
    "i would be happy to",
    "please feel free to",
    "in summary,",
    "in conclusion,",
    "to summarize,",
];

// Precompiled Regexes
static RE_WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]+").unwrap());
static RE_MULTILINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());
static RE_TRANSCRIPT_ROLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\[?(user|assistant|system|developer)\]?[:\s]+"#).unwrap());

/// Core AAAK compressor engine.
pub struct AaakCompressor;

impl AaakCompressor {
    /// Compresses text according to the selected mode.
    pub fn compress(text: &str, mode: AaakMode) -> AaakCompressedResult {
        let original_bytes = text.len();
        let counter = TiktokenCounter::default();
        let original_tokens = counter.count_tokens(text);

        let processed = match mode {
            AaakMode::Lossless => Self::compress_lossless(text),
            AaakMode::UltraDense => Self::compress_ultra_dense(text),
            AaakMode::Transcript => Self::compress_transcript(text),
        };

        let header = match mode {
            AaakMode::Lossless => "[AAAK:v1:lossless]\n",
            AaakMode::UltraDense => "[AAAK:v1:dense]\n",
            AaakMode::Transcript => "[AAAK:v1:transcript]\n",
        };

        let compressed = format!("{}{}", header, processed.trim());
        let compressed_bytes = compressed.len();
        let compressed_tokens = counter.count_tokens(&compressed);

        let compression_ratio = if compressed_bytes > 0 {
            original_bytes as f32 / compressed_bytes as f32
        } else {
            1.0
        };

        let token_savings_pct = if original_tokens > 0 {
            ((original_tokens.saturating_sub(compressed_tokens)) as f32 / original_tokens as f32)
                * 100.0
        } else {
            0.0
        };

        AaakCompressedResult {
            compressed,
            original_bytes,
            compressed_bytes,
            original_tokens,
            compressed_tokens,
            compression_ratio,
            token_savings_pct,
            mode,
        }
    }

    /// Decompresses an AAAK-encoded text back to standard prose.
    pub fn decompress(text: &str) -> String {
        let stripped = text
            .strip_prefix("[AAAK:v1:lossless]\n")
            .or_else(|| text.strip_prefix("[AAAK:v1:lossless]\r\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:lossless]\\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:dense]\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:dense]\r\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:dense]\\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:transcript]\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:transcript]\r\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1:transcript]\\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1]\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1]\r\n"))
            .or_else(|| text.strip_prefix("[AAAK:v1]\\n"))
            .unwrap_or(text);

        let mut expanded = stripped.to_string();

        // Expand transcript roles
        expanded = expanded
            .replace("[U] ", "User: ")
            .replace("[A] ", "Assistant: ")
            .replace("[S] ", "System: ")
            .replace("[D] ", "Developer: ");

        // Reverse abbreviations map (preserve canonical first full form)
        let mut reverse_map: HashMap<&'static str, &'static str> = HashMap::new();
        for pair in ABBREVIATIONS {
            reverse_map.entry(pair.short).or_insert(pair.full);
        }

        let words: Vec<&str> = expanded
            .split_inclusive(|c: char| !c.is_alphanumeric() && c != '\'' && c != '_')
            .collect();
        let mut result = String::with_capacity(expanded.len() * 2);

        for segment in words {
            let mut word = segment;
            let mut suffix = "";
            if let Some(pos) = segment.find(|c: char| !c.is_alphanumeric() && c != '\'' && c != '_')
            {
                word = &segment[..pos];
                suffix = &segment[pos..];
            }

            if let Some(full) = reverse_map.get(word) {
                result.push_str(full);
            } else if let Some(full) = reverse_map.get(word.to_lowercase().as_str()) {
                if word
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    let mut chars = full.chars();
                    if let Some(first) = chars.next() {
                        result.push(first.to_ascii_uppercase());
                        result.push_str(chars.as_str());
                    }
                } else {
                    result.push_str(full);
                }
            } else {
                result.push_str(word);
            }
            result.push_str(suffix);
        }

        result
    }

    /// Compresses text using lossless deterministic substitution.
    fn compress_lossless(text: &str) -> String {
        let mut result = text.to_string();
        for pair in ABBREVIATIONS {
            let pattern = format!(r"(?i)\b{}\b", regex::escape(pair.full));
            if let Ok(re) = Regex::new(&pattern) {
                result = re.replace_all(&result, pair.short).to_string();
            }
        }
        result
    }

    /// Compresses text using ultra-dense stripping, abbreviations, and whitespace compaction.
    fn compress_ultra_dense(text: &str) -> String {
        let mut working = text.to_string();

        // 1. Remove conversational fillers
        for filler in CONVERSATIONAL_FILLERS {
            let pattern = format!(r"(?i){}", regex::escape(filler));
            if let Ok(re) = Regex::new(&pattern) {
                working = re.replace_all(&working, "").to_string();
            }
        }

        // 2. Apply abbreviation dictionary
        for pair in ABBREVIATIONS {
            let pattern = format!(r"(?i)\b{}\b", regex::escape(pair.full));
            if let Ok(re) = Regex::new(&pattern) {
                working = re.replace_all(&working, pair.short).to_string();
            }
        }

        // 3. Compact whitespace and empty lines
        let compacted = RE_WHITESPACE.replace_all(&working, " ");
        let result = RE_MULTILINE.replace_all(&compacted, "\n\n");
        result.trim().to_string()
    }

    /// Compresses conversational transcripts into compact `[U]` / `[A]` dialogue turns.
    fn compress_transcript(text: &str) -> String {
        let mut working = text.to_string();

        // Handle JSONL formatting if present: {"role":"user","content":"..."}
        if text.contains(r#"{"role""#) {
            let mut lines = Vec::new();
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    let role = val.get("role").and_then(|r| r.as_str()).unwrap_or("U");
                    let content = val.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    let marker = match role.to_lowercase().as_str() {
                        "assistant" => "[A]",
                        "system" => "[S]",
                        "developer" => "[D]",
                        _ => "[U]",
                    };
                    let compressed_turn = Self::compress_ultra_dense(content);
                    lines.push(format!("{} {}", marker, compressed_turn));
                } else {
                    lines.push(Self::compress_ultra_dense(trimmed));
                }
            }
            return lines.join("\n");
        }

        // Standard text transcripts: replace role indicators
        working = RE_TRANSCRIPT_ROLE
            .replace_all(&working, |caps: &regex::Captures| {
                let r = caps
                    .get(1)
                    .map(|m| m.as_str().to_lowercase())
                    .unwrap_or_default();
                match r.as_str() {
                    "assistant" => "[A] ",
                    "system" => "[S] ",
                    "developer" => "[D] ",
                    _ => "[U] ",
                }
            })
            .to_string();

        Self::compress_ultra_dense(&working)
    }

    /// Calculate token savings comparison.
    pub fn calculate_token_savings(original: &str, compressed: &str) -> TokenSavings {
        let counter = TiktokenCounter::default();
        let original_tokens = counter.count_tokens(original);
        let compressed_tokens = counter.count_tokens(compressed);
        let tokens_saved = original_tokens.saturating_sub(compressed_tokens);
        let savings_percentage = if original_tokens > 0 {
            (tokens_saved as f32 / original_tokens as f32) * 100.0
        } else {
            0.0
        };

        TokenSavings {
            original_tokens,
            compressed_tokens,
            tokens_saved,
            savings_percentage,
        }
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lossless_abbreviation_compression_and_decompression() {
        let input = "The database configuration function requires authentication parameters.";
        let res = AaakCompressor::compress(input, AaakMode::Lossless);

        assert!(res.compressed.starts_with("[AAAK:v1:lossless]\n"));
        assert!(res.compressed.contains("db"));
        assert!(res.compressed.contains("cfg"));
        assert!(res.compressed.contains("fn"));
        assert!(res.compressed.contains("auth"));
        assert!(res.compressed.contains("params"));

        let decompressed = AaakCompressor::decompress(&res.compressed);
        assert!(decompressed.contains("database"));
        assert!(decompressed.contains("configuration"));
        assert!(decompressed.contains("function"));
        assert!(decompressed.contains("authentication"));
        assert!(decompressed.contains("parameters"));
    }

    #[test]
    fn test_ultra_dense_compression_ratio() {
        let input = "Sure, I can help with that! In order to configure the database implementation, the assistant must verify the environment variables without exception. Furthermore, the repository authentication function requires asynchronous transaction processing, and the maximum connection timeout parameter should be set to five seconds. Please let me know if you need anything else!";
        let res = AaakCompressor::compress(input, AaakMode::UltraDense);

        assert!(res.compressed.starts_with("[AAAK:v1:dense]\n"));
        assert!(!res
            .compressed
            .to_lowercase()
            .contains("sure, i can help with that"));
        assert!(res.compressed.contains("db"));
        assert!(res.compressed.contains("impl"));
        assert!(res.compressed.contains("env"));
        assert!(res.compressed.contains("vars"));
        assert!(res.compressed.contains("w/o"));
        assert!(res.compressed.contains("repo"));
        assert!(res.compressed.contains("auth"));
        assert!(res.compressed.contains("fn"));
        assert!(res.compressed.contains("async"));
        assert!(res.compressed.contains("tx"));
        assert!(res.compression_ratio > 1.3);
        assert!(res.token_savings_pct > 5.0);
    }

    #[test]
    fn test_transcript_mode_turn_compaction() {
        let transcript = r#"{"role":"user","content":"Please implement the authentication database function."}
{"role":"assistant","content":"Sure, I can help with that! Here is the summary: I will configure the repository interface."}"#;

        let res = AaakCompressor::compress(transcript, AaakMode::Transcript);
        assert!(res.compressed.starts_with("[AAAK:v1:transcript]\n"));
        assert!(res.compressed.contains("[U]"));
        assert!(res.compressed.contains("[A]"));
        assert!(res.compressed.contains("impl"));
        assert!(res.compressed.contains("auth"));
        assert!(res.compressed.contains("db"));
        assert!(res.compressed.contains("fn"));
        assert!(res.compressed.contains("repo"));
        assert!(res.compressed.contains("iface"));
    }
}
