//! Demo of TruncationEngine inspired by RTK
//!
//! Run with: cargo run --bin truncation_demo

use engram::intelligence::truncation_engine::{TruncationConfig, TruncationEngine};

fn main() {
    let engine = TruncationEngine::with_config(TruncationConfig::default());

    // Example 1: Long log (10,000 chars)
    let long_log = "2024-01-01 10:00:00 INFO Starting...\n".repeat(100);
    let truncated = engine.truncate_to_budget(&long_log, 1000); // 1000 tokens
    println!("Log original: {} chars", long_log.len());
    println!("Log truncated: {} chars", truncated.len());

    // Example 2: File with error
    let file_with_error = "line 1\nline 2\nERROR: something failed\nline 1000";
    let truncated = engine.truncate_to_budget(file_with_error, 500);
    println!("\nFile original: {} chars", file_with_error.len());
    println!("File truncated: {} chars", truncated.len());
    println!("Contains ERROR: {}", truncated.contains("ERROR"));

    // Example 3: Multiple memories (mock)
    let memories = [
        "Memory 1: User prefers dark mode".repeat(100),
        "Memory 2: User likes coffee".repeat(100),
        "Memory 3: User works on Engram".repeat(100),
    ];

    // For demo, we'll just truncate each individually
    let budget_per = 2000 / memories.len();
    println!("\nMemories truncation:");
    for (i, mem) in memories.iter().enumerate() {
        let truncated = engine.truncate_to_budget(mem, budget_per);
        println!(
            "Memory {}: {} -> {} chars",
            i + 1,
            mem.len(),
            truncated.len()
        );
    }
}
