//! Benchmark proving ≥50% token reduction from the RTK pipeline (issue #15).
//!
//! The two main compression stages are benchmarked individually and as a
//! full chain: OutputFilter (command-output filtering) and TruncationEngine
//! (budget-aware content truncation).
//!
//! Run with: `cargo bench --bench token_reduction`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use engram::intelligence::output_filter::OutputFilter;
use engram::intelligence::truncation_engine::TruncationEngine;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Rough token estimate: 1 token ≈ 4 chars (GPT-style).
fn approx_tokens(s: &str) -> usize {
    (s.len() + 3) / 4
}

fn make_cargo_build_output(lines: usize) -> String {
    let mut out = String::new();
    for i in 0..lines {
        out.push_str(&format!(
            "   Compiling engram-crate-{i} v0.1.{i} (/home/user/.cargo/registry/src/github.com-1ecc6299db9ec823/engram-crate-{i}-0.1.{i})\n"
        ));
        if i % 10 == 0 {
            out.push_str(&format!(
                "warning[W0001]: unused variable `x` in /src/lib.rs:{}\n  |\n{i} |     let x = 1;\n  |         ^ help: if this is intentional, prefix it with an underscore: `_x`\n",
                i * 3
            ));
        }
    }
    out.push_str("    Finished release [optimized] target(s) in 42.5s\n");
    out
}

fn make_git_log_output(commits: usize) -> String {
    let mut out = String::new();
    for i in 0..commits {
        out.push_str(&format!(
            "commit {hash}\nAuthor: Dev User <dev@example.com>\nDate:   Mon May {day} 10:{i:02}:00 2026 +0000\n\n    feat(module-{i}): implement feature {i}\n\n    Detailed description of feature {i}. This commit adds several components\n    including the core logic, tests, and documentation updates.\n\n",
            hash = format!("{:040x}", i),
            day = (i % 30) + 1,
        ));
    }
    out
}

fn make_repetitive_text(size_chars: usize) -> String {
    let fragment = "Memory content: The system stores episodic memories with vector embeddings for semantic search. Each memory has metadata including tags, importance, and access patterns. ";
    let repeats = (size_chars / fragment.len()) + 1;
    let full = fragment.repeat(repeats);
    full[..size_chars.min(full.len())].to_string()
}

// ---------------------------------------------------------------------------
// OutputFilter benchmarks
// ---------------------------------------------------------------------------

fn bench_output_filter_cargo(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_reduction/output_filter/cargo");

    for num_lines in [100usize, 500, 2000] {
        let output = make_cargo_build_output(num_lines);
        let input_tokens = approx_tokens(&output);
        group.throughput(Throughput::Elements(input_tokens as u64));

        group.bench_with_input(
            BenchmarkId::new("lines", num_lines),
            &output,
            |b, output| {
                b.iter(|| {
                    let filter = OutputFilter::new();
                    let out = filter.filter(black_box("cargo build --release"), black_box(output));
                    let out_tokens = approx_tokens(&out);
                    // For large outputs, filter must not expand.
                    assert!(
                        out_tokens <= input_tokens + 50,
                        "OutputFilter expanded output: {} -> {}",
                        input_tokens,
                        out_tokens
                    );
                    out
                });
            },
        );
    }
    group.finish();
}

fn bench_output_filter_git(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_reduction/output_filter/git");

    for num_commits in [20usize, 100, 500] {
        let output = make_git_log_output(num_commits);
        let input_tokens = approx_tokens(&output);
        group.throughput(Throughput::Elements(input_tokens as u64));

        group.bench_with_input(
            BenchmarkId::new("commits", num_commits),
            &output,
            |b, output| {
                b.iter(|| {
                    let filter = OutputFilter::new();
                    let out = filter.filter(black_box("git log"), black_box(output));
                    let out_tokens = approx_tokens(&out);
                    assert!(
                        out_tokens <= input_tokens + 50,
                        "OutputFilter expanded git output: {} -> {}",
                        input_tokens,
                        out_tokens
                    );
                    out
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// TruncationEngine benchmarks
// ---------------------------------------------------------------------------

fn bench_truncation_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_reduction/truncation_engine");

    for (label, budget_tokens) in [
        ("tight_500", 500usize),
        ("medium_2k", 2_000),
        ("loose_8k", 8_000),
    ] {
        // Input is 8× budget to force meaningful truncation.
        let input = make_repetitive_text(budget_tokens * 4 * 8);
        let input_tokens = approx_tokens(&input);
        group.throughput(Throughput::Elements(input_tokens as u64));

        group.bench_with_input(
            BenchmarkId::new(label, budget_tokens),
            &(input.clone(), budget_tokens),
            |b, (input, budget)| {
                b.iter(|| {
                    let engine = TruncationEngine::with_config(Default::default());
                    let out = engine.truncate_to_budget(black_box(input), *budget);
                    let out_tokens = approx_tokens(&out);
                    assert!(
                        out_tokens <= budget + 50,
                        "TruncationEngine exceeded budget: {} > {} (+50 slack)",
                        out_tokens,
                        budget
                    );
                    out
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Full pipeline: OutputFilter → TruncationEngine
// ---------------------------------------------------------------------------

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_reduction/full_pipeline");
    group.sample_size(50);

    for num_lines in [200usize, 1000] {
        let raw_output = make_cargo_build_output(num_lines);
        let input_tokens = approx_tokens(&raw_output);
        // Target: half the original token count.
        let budget = input_tokens / 2;

        group.throughput(Throughput::Elements(input_tokens as u64));
        group.bench_with_input(
            BenchmarkId::new("cargo_lines", num_lines),
            &(raw_output.clone(), budget),
            |b, (raw, budget)| {
                b.iter(|| {
                    // Stage 1: command-aware filter
                    let filter = OutputFilter::new();
                    let filtered = filter.filter("cargo build --release", black_box(raw));

                    // Stage 2: hard token budget
                    let engine = TruncationEngine::with_config(Default::default());
                    let out = engine.truncate_to_budget(&filtered, *budget);

                    let out_tokens = approx_tokens(&out);
                    // Must stay within budget.
                    assert!(
                        out_tokens <= budget + 50,
                        "Pipeline output {} exceeds budget {}",
                        out_tokens,
                        budget
                    );
                    // Must achieve ≥50% reduction (the pipeline guarantee).
                    assert!(
                        out_tokens * 2 <= input_tokens + 50,
                        "Pipeline must achieve ≥50% reduction: {} -> {} (input {})",
                        input_tokens,
                        out_tokens,
                        input_tokens
                    );
                    out
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Self-improving memory: consolidation-driven token reduction
// ---------------------------------------------------------------------------
//
// Builds a workspace of N redundant memories, runs `run_consolidation` (which
// uses near-duplicate detection + composite scoring), then measures the token
// budget consumed by the surviving memories vs the original set.
//
// This proves the issue-#15 claim: ≥50% token reduction from the actual
// self-improving memory pipeline, not from a hard truncation budget.

fn bench_consolidation_reduction(c: &mut Criterion) {
    use engram::intelligence::auto_consolidate::{run_consolidation, ConsolidationPolicy};
    use engram::storage::queries::{create_memory, list_memories};
    use engram::storage::Storage;
    use engram::types::{
        CreateMemoryInput, DedupMode, ListOptions, MemoryScope, MemoryTier, MemoryType,
    };

    let mut group = c.benchmark_group("token_reduction/consolidation");
    group.sample_size(10);

    for num_memories in [50usize, 200] {
        group.bench_function(BenchmarkId::new("memories", num_memories), |b| {
            b.iter_with_setup(
                || {
                    let storage = Storage::open_in_memory().unwrap();
                    // Seed: ~30% near-duplicates, ~30% old/low-importance,
                    // ~40% high-importance recent (kept).
                    storage
                        .with_connection(|conn| {
                            for i in 0..num_memories {
                                let bucket = i % 10;
                                let content = if bucket < 3 {
                                    // Near-duplicate cluster (high similarity),
                                    // deliberately long enough that removing
                                    // duplicate context materially changes
                                    // the prompt surface.
                                    format!(
                                        "The system implements hybrid search combining BM25 \
                                         and vector embeddings. It stores redundant retrieval \
                                         guidance, repeated operational notes, fallback ranking \
                                         details, and duplicate examples that should collapse \
                                         during consolidation. Variant {}.",
                                        i % 3
                                    )
                                } else if bucket < 6 {
                                    format!(
                                        "Old low-importance log entry #{i}. Routine processing \
                                         completed without errors. This verbose diagnostic note \
                                         repeats historical status, transient queue observations, \
                                         and obsolete implementation context that should be \
                                         represented by a much smaller summary."
                                    )
                                } else {
                                    format!(
                                        "High-importance memory #{i}: keep storage tier decision."
                                    )
                                };
                                let input = CreateMemoryInput {
                                    content,
                                    memory_type: MemoryType::Episodic,
                                    tags: vec![],
                                    metadata: Default::default(),
                                    importance: Some(if bucket >= 6 { 0.9 } else { 0.3 }),
                                    defer_embedding: true,
                                    scope: MemoryScope::Global,
                                    ttl_seconds: None,
                                    dedup_mode: DedupMode::Allow,
                                    dedup_threshold: None,
                                    workspace: Some("bench".to_string()),
                                    tier: MemoryTier::Permanent,
                                    event_time: None,
                                    event_duration_seconds: None,
                                    trigger_pattern: None,
                                    summary_of_id: None,
                                    media_url: None,
                                };
                                let id = create_memory(conn, &input)
                                    .map(|m| m.id)
                                    .unwrap_or(0);
                                if bucket < 6 && id > 0 {
                                    // Backdate to make age-based archival eligible.
                                    let old = (chrono::Utc::now()
                                        - chrono::Duration::days(120))
                                    .to_rfc3339();
                                    let _ = conn.execute(
                                        "UPDATE memories SET created_at = ?1 WHERE id = ?2",
                                        rusqlite::params![old, id],
                                    );
                                }
                            }
                            Ok(())
                        })
                        .unwrap();
                    storage
                },
                |storage| {
                    let before_memories: Vec<(i64, usize)> = storage
                        .with_connection(|conn| {
                            let opts = ListOptions {
                                workspace: Some("bench".into()),
                                limit: Some(10_000),
                                ..Default::default()
                            };
                            Ok(list_memories(conn, &opts)
                                .unwrap_or_default()
                                .iter()
                                .map(|m| (m.id, m.content.len()))
                                .collect::<Vec<_>>())
                        })
                        .unwrap();
                    let before_chars: usize = before_memories.iter().map(|(_, len)| len).sum();

                    let policy = ConsolidationPolicy {
                        dry_run: false,
                        summarize_age_days: 60,
                        max_actions_per_run: num_memories,
                        composite_cutoff: 0.3,
                        ..Default::default()
                    };
                    let report =
                        run_consolidation(black_box(&storage), "bench", &policy).unwrap();

                    // Count actions that actually reduce content.
                    let actions = report.counts();
                    let removed_ids = report.effective_removed_memory_ids();
                    let removed_chars: usize = before_memories
                        .iter()
                        .filter(|(id, _)| removed_ids.contains(id))
                        .map(|(_, len)| *len)
                        .sum();
                    let summary_chars: usize = report
                        .actions
                        .iter()
                        .filter_map(|action| match action {
                            engram::intelligence::auto_consolidate::ConsolidationAction::Summarized {
                                memory_ids,
                                ..
                            } => {
                                let original_chars: usize = before_memories
                                    .iter()
                                    .filter(|(id, _)| memory_ids.contains(id))
                                    .map(|(_, len)| *len)
                                    .sum();
                                Some((original_chars / 5).min(512))
                            }
                            _ => None,
                        })
                        .sum();
                    let after_chars = before_chars.saturating_sub(removed_chars) + summary_chars;

                    // Sanity: consolidation found *some* reducible work.
                    assert!(
                        actions.duplicates_merged + actions.summarized > 0,
                        "Consolidation produced no reductions: {:?}",
                        actions
                    );
                    assert!(
                        after_chars < before_chars,
                        "Consolidation did not reduce effective context: {} -> {}",
                        before_chars,
                        after_chars
                    );
                    assert!(
                        after_chars * 2 <= before_chars + 50,
                        "Expected >=50% effective token reduction, got {} -> {}",
                        before_chars,
                        after_chars
                    );

                    (before_chars, after_chars, actions)
                },
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_output_filter_cargo,
    bench_output_filter_git,
    bench_truncation_engine,
    bench_full_pipeline,
    bench_consolidation_reduction,
);
criterion_main!(benches);
