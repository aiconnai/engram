use super::output::ReducerOutput;
use super::redaction::{redact_text, NoopRedactor, Redactor};
use super::util::{is_numeric, strip_ansi, truncate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const DEFAULT_MAX_REPRESENTATIVE_LINES: usize = 3;
const DEFAULT_MAX_LINE_CHARS: usize = 240;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgLineMatch {
    pub file: String,
    pub line_number: u64,
    pub column: Option<u64>,
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgFileMatches {
    pub file: String,
    pub match_count: usize,
    pub representative_lines: Vec<RepresentativeLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepresentativeLine {
    pub line_number: u64,
    pub column: Option<u64>,
    pub line: String,
    pub truncated: bool,
}

pub fn parse_rg_match_line(line: &str) -> Option<RgLineMatch> {
    let clean = strip_ansi(line);
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() < 3 {
        return None;
    }

    for index in 1..parts.len() {
        if !is_numeric(parts[index]) {
            continue;
        }

        let file = parts[..index].join(":");
        if file.is_empty() {
            continue;
        }

        let line_number = parts[index].parse::<u64>().ok()?;
        let mut content_index = index + 1;
        let mut column = None;
        if content_index < parts.len() && is_numeric(parts[content_index]) {
            column = parts[content_index].parse::<u64>().ok();
            content_index += 1;
        }

        if content_index > parts.len() {
            return None;
        }

        return Some(RgLineMatch {
            file,
            line_number,
            column,
            line: parts[content_index..].join(":"),
        });
    }

    None
}

pub fn reduce_rg_matches(output: &str) -> ReducerOutput {
    reduce_rg_matches_with_limits(
        output,
        DEFAULT_MAX_REPRESENTATIVE_LINES,
        DEFAULT_MAX_LINE_CHARS,
    )
}

pub fn reduce_rg_matches_with_limits(
    output: &str,
    max_representative_lines: usize,
    max_line_chars: usize,
) -> ReducerOutput {
    reduce_rg_matches_with_redactor(
        output,
        max_representative_lines,
        max_line_chars,
        &NoopRedactor,
    )
}

pub fn reduce_rg_matches_with_redactor(
    rg_output: &str,
    max_representative_lines: usize,
    max_line_chars: usize,
    redactor: &dyn Redactor,
) -> ReducerOutput {
    let mut files: BTreeMap<String, RgFileMatches> = BTreeMap::new();
    let mut unparsed_lines = 0usize;
    let mut truncated_lines = 0usize;

    for raw_line in rg_output.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        let Some(parsed) = parse_rg_match_line(raw_line) else {
            unparsed_lines += 1;
            continue;
        };

        let entry = files
            .entry(parsed.file.clone())
            .or_insert_with(|| RgFileMatches {
                file: parsed.file.clone(),
                match_count: 0,
                representative_lines: Vec::new(),
            });
        entry.match_count += 1;

        if entry.representative_lines.len() < max_representative_lines {
            let (line, truncated) = truncate(&parsed.line, max_line_chars);
            if truncated {
                truncated_lines += 1;
            }
            entry.representative_lines.push(RepresentativeLine {
                line_number: parsed.line_number,
                column: parsed.column,
                line,
                truncated,
            });
        }
    }

    let total_matches: usize = files.values().map(|file| file.match_count).sum();
    let representative_count: usize = files
        .values()
        .map(|file| file.representative_lines.len())
        .sum();
    let summary = format!(
        "rg_matches@v1: files={}; matches={total_matches}; representative_lines={representative_count}; unparsed_lines={unparsed_lines}",
        files.len()
    );
    let mut output = ReducerOutput::new(summary);

    output.lossy =
        unparsed_lines > 0 || truncated_lines > 0 || representative_count < total_matches;
    output.raw_required_for_full_debug = output.lossy && total_matches > 0;
    output.confidence = if unparsed_lines == 0 { 0.9 } else { 0.75 };

    output.add_fact("reducer", "rg_matches@v1");
    output.add_fact("file_count", files.len().to_string());
    output.add_fact("match_count", total_matches.to_string());

    if unparsed_lines > 0 {
        output.add_warning(format!(
            "rg_matches@v1 skipped {unparsed_lines} unparsed non-empty lines"
        ));
    }

    for file in files.values() {
        let value = format!("{} matches={}", file.file, file.match_count);
        let mut metadata = BTreeMap::new();
        metadata.insert("file".to_string(), file.file.clone());
        metadata.insert("match_count".to_string(), file.match_count.to_string());
        let value = redact_text(redactor, &value, &mut output);
        output.add_fact_with_metadata("match_group", value, metadata);

        for representative in &file.representative_lines {
            let location = match representative.column {
                Some(column) => format!("{}:{}:{column}", file.file, representative.line_number),
                None => format!("{}:{}", file.file, representative.line_number),
            };
            let value = format!("{location}:{}", representative.line);
            let mut metadata = BTreeMap::new();
            metadata.insert("file".to_string(), file.file.clone());
            metadata.insert(
                "line_number".to_string(),
                representative.line_number.to_string(),
            );
            if let Some(column) = representative.column {
                metadata.insert("column".to_string(), column.to_string());
            }
            metadata.insert(
                "truncated".to_string(),
                representative.truncated.to_string(),
            );
            let value = redact_text(redactor, &value, &mut output);
            output.add_fact_with_metadata("representative_match", value, metadata);
        }
    }

    output.add_evidence("files_grouped", !files.is_empty());
    output.add_evidence("match_counts", true);
    output.add_evidence("representative_lines", representative_count > 0);
    output.add_evidence("line_truncation_flag", true);
    output.add_evidence("all_raw_matches", representative_count == total_matches);

    output
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
    fn rg_matches_groups_by_file_counts_matches_and_truncates_representative_lines() {
        let rg = "\
src/lib.rs:10:pub mod context;
src/lib.rs:20:pub fn very_long_line() { let token = \"abcdefghijklmnopqrstuvwxyz\"; }
src/context/reducers/mod.rs:3:pub mod cargo_test;
";

        let output = reduce_rg_matches_with_limits(rg, 1, 32);

        assert!(has_fact(&output, "match_group", "src/lib.rs matches=2"));
        assert!(has_fact(
            &output,
            "match_group",
            "src/context/reducers/mod.rs matches=1"
        ));
        assert!(has_fact(
            &output,
            "representative_match",
            "src/lib.rs:10:pub mod context;"
        ));
        assert!(!has_fact(&output, "representative_match", "very_long_line"));
        assert!(output.lossy);
        assert!(output.raw_required_for_full_debug);
    }
}
