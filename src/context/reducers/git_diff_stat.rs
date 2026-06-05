use super::output::ReducerOutput;
use super::redaction::{redact_text, NoopRedactor, Redactor};
use super::util::strip_ansi;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffStatFile {
    pub path: String,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub binary: bool,
    pub high_risk: bool,
}

pub fn parse_git_diff_stat(input: &str) -> Vec<DiffStatFile> {
    let mut files: BTreeMap<String, DiffStatFile> = BTreeMap::new();

    for raw_line in input.lines() {
        let line = strip_ansi(raw_line);
        let trimmed = line.trim();

        if let Some(path) = parse_diff_git_path(trimmed) {
            merge_file(&mut files, DiffStatFile::new(path, None, None, false));
            continue;
        }

        if let Some(file) = parse_numstat_line(trimmed) {
            merge_file(&mut files, file);
            continue;
        }

        if let Some(file) = parse_stat_line(&line) {
            merge_file(&mut files, file);
        }
    }

    files.into_values().collect()
}

pub fn reduce_git_diff_stat(input: &str) -> ReducerOutput {
    reduce_git_diff_stat_with_redactor(input, &NoopRedactor)
}

pub fn reduce_git_diff_stat_with_redactor(input: &str, redactor: &dyn Redactor) -> ReducerOutput {
    let files = parse_git_diff_stat(input);
    let additions: u64 = files.iter().filter_map(|file| file.additions).sum();
    let deletions: u64 = files.iter().filter_map(|file| file.deletions).sum();
    let high_risk_count = files.iter().filter(|file| file.high_risk).count();
    let saw_raw_hunks = input
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with("@@"));
    let summary = format!(
        "git_diff_stat@v1: changed_files={}; additions={additions}; deletions={deletions}; high_risk_files={high_risk_count}",
        files.len()
    );
    let mut output = ReducerOutput::new(summary);

    output.lossy = true;
    output.raw_required_for_full_debug = saw_raw_hunks;
    output.confidence = if files.is_empty() { 0.65 } else { 0.9 };

    output.add_fact("reducer", "git_diff_stat@v1");
    output.add_fact("changed_file_count", files.len().to_string());
    output.add_fact("total_additions", additions.to_string());
    output.add_fact("total_deletions", deletions.to_string());

    if saw_raw_hunks {
        output
            .add_warning("git_diff_stat@v1 detected raw diff hunks and intentionally omitted them");
    }

    for file in &files {
        let additions = file
            .additions
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let deletions = file
            .deletions
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let value = format!(
            "{} +{} -{} binary={} high_risk={}",
            file.path, additions, deletions, file.binary, file.high_risk
        );
        let mut metadata = BTreeMap::new();
        metadata.insert("path".to_string(), file.path.clone());
        metadata.insert("additions".to_string(), additions);
        metadata.insert("deletions".to_string(), deletions);
        metadata.insert("binary".to_string(), file.binary.to_string());
        metadata.insert("high_risk".to_string(), file.high_risk.to_string());
        let value = redact_text(redactor, &value, &mut output);
        output.add_fact_with_metadata("changed_file", value, metadata);

        if file.high_risk {
            let value = redact_text(redactor, &file.path, &mut output);
            output.add_fact("high_risk_file", value);
        }
    }

    output.add_evidence("changed_files", !files.is_empty());
    output.add_evidence(
        "additions_deletions",
        files.iter().any(|file| file.additions.is_some()),
    );
    output.add_evidence("high_risk_file_classification", true);
    output.add_evidence("raw_diff_hunks", false);

    output
}

pub fn high_risk_file(path: &str) -> bool {
    let normalized = path.trim_start_matches("a/").trim_start_matches("b/");
    let lower = normalized.to_ascii_lowercase();

    matches!(normalized, "Cargo.toml" | "Cargo.lock")
        || lower.ends_with(".sql")
        || lower.contains("schema")
        || lower.contains("migration")
        || lower.starts_with("src/storage/")
        || lower.starts_with("src/mcp/")
        || lower.starts_with("src/auth")
        || lower.starts_with("src/hooks/")
        || lower.starts_with(".github/workflows/")
        || lower.starts_with("docs/harness/bin/")
        || matches!(
            normalized,
            "docs/harness/INVARIANTS.md"
                | "docs/harness/GATES.md"
                | "docs/harness/CODE_REVIEW_POLICY.md"
        )
        || lower.contains("secret")
        || lower.contains("credential")
        || lower.contains("token")
        || lower.contains("private_key")
}

impl DiffStatFile {
    fn new(path: String, additions: Option<u64>, deletions: Option<u64>, binary: bool) -> Self {
        let high_risk = high_risk_file(&path);
        Self {
            path,
            additions,
            deletions,
            binary,
            high_risk,
        }
    }
}

fn parse_numstat_line(line: &str) -> Option<DiffStatFile> {
    let mut parts = line.split('\t');
    let additions = parts.next()?;
    let deletions = parts.next()?;
    let path = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    let binary = additions == "-" && deletions == "-";
    let additions = additions.parse::<u64>().ok();
    let deletions = deletions.parse::<u64>().ok();
    Some(DiffStatFile::new(
        path.to_string(),
        additions,
        deletions,
        binary,
    ))
}

fn parse_stat_line(line: &str) -> Option<DiffStatFile> {
    let (path, stat) = line.split_once('|')?;
    let path = path.trim();
    if path.is_empty() || path.contains(" files changed") {
        return None;
    }

    let additions = stat.chars().filter(|ch| *ch == '+').count() as u64;
    let deletions = stat.chars().filter(|ch| *ch == '-').count() as u64;
    let binary = stat.contains("Bin ");
    Some(DiffStatFile::new(
        path.to_string(),
        Some(additions),
        Some(deletions),
        binary,
    ))
}

fn parse_diff_git_path(line: &str) -> Option<String> {
    let rest = line.strip_prefix("diff --git ")?;
    let mut parts = rest.split_whitespace();
    let _old = parts.next()?;
    let new = parts.next()?;
    Some(new.trim_start_matches("b/").to_string())
}

fn merge_file(files: &mut BTreeMap<String, DiffStatFile>, next: DiffStatFile) {
    files
        .entry(next.path.clone())
        .and_modify(|current| {
            current.additions = merge_count(current.additions, next.additions);
            current.deletions = merge_count(current.deletions, next.deletions);
            current.binary |= next.binary;
            current.high_risk |= next.high_risk;
        })
        .or_insert(next);
}

fn merge_count(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_fact(output: &ReducerOutput, kind: &str, needle: &str) -> bool {
        output
            .observed_facts
            .iter()
            .any(|fact| fact.kind == kind && fact.value.contains(needle))
    }

    #[test]
    fn git_diff_stat_preserves_changed_files_counts_and_high_risk_without_hunks() {
        let stat = "\
12\t3\tsrc/storage/migrations.rs
4\t0\tsrc/lib.rs
diff --git a/src/mcp/protocol.rs b/src/mcp/protocol.rs
@@ -1,2 +1,3 @@
-raw removed hunk
+raw added hunk
";

        let output = reduce_git_diff_stat(stat);

        assert!(has_fact(
            &output,
            "changed_file",
            "src/storage/migrations.rs +12 -3"
        ));
        assert!(has_fact(&output, "changed_file", "src/lib.rs +4 -0"));
        assert!(has_fact(
            &output,
            "high_risk_file",
            "src/storage/migrations.rs"
        ));
        assert!(has_fact(&output, "high_risk_file", "src/mcp/protocol.rs"));
        assert!(!output
            .observed_facts
            .iter()
            .any(|fact| fact.value.contains("raw added hunk")));
        assert!(output.raw_required_for_full_debug);
    }
}
