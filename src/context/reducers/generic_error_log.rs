use super::output::ReducerOutput;
use super::redaction::{redact_text, NoopRedactor, Redactor};
use super::util::{strip_ansi, truncate};
use std::collections::{BTreeMap, VecDeque};

const DEFAULT_MAX_FINAL_ERRORS: usize = 8;
const DEFAULT_MAX_WARNINGS: usize = 8;
const DEFAULT_MAX_REPEATED_LINES: usize = 8;
const DEFAULT_TAIL_LINES: usize = 20;
const DEFAULT_MAX_LINE_CHARS: usize = 240;

pub fn reduce_generic_error_log(log: &str) -> ReducerOutput {
    reduce_generic_error_log_with_redactor(log, &NoopRedactor)
}

pub fn reduce_generic_error_log_with_redactor(log: &str, redactor: &dyn Redactor) -> ReducerOutput {
    reduce_generic_error_log_with_limits(
        log,
        DEFAULT_MAX_FINAL_ERRORS,
        DEFAULT_MAX_WARNINGS,
        DEFAULT_MAX_REPEATED_LINES,
        DEFAULT_TAIL_LINES,
        DEFAULT_MAX_LINE_CHARS,
        redactor,
    )
}

fn reduce_generic_error_log_with_limits(
    log: &str,
    max_final_errors: usize,
    max_warnings: usize,
    max_repeated_lines: usize,
    tail_lines: usize,
    max_line_chars: usize,
    redactor: &dyn Redactor,
) -> ReducerOutput {
    let mut total_lines = 0usize;
    let mut line_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut error_lines = Vec::new();
    let mut warning_lines = Vec::new();
    let mut tail = VecDeque::new();
    let mut truncated_lines = 0usize;

    for raw_line in log.lines() {
        total_lines += 1;
        let line = strip_ansi(raw_line);
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *line_counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }

        let lower = trimmed.to_ascii_lowercase();
        if is_error_line(&lower) {
            error_lines.push(trimmed.to_string());
        }
        if is_warning_line(&lower) {
            warning_lines.push(trimmed.to_string());
        }

        if tail_lines > 0 {
            if tail.len() == tail_lines {
                tail.pop_front();
            }
            tail.push_back(trimmed.to_string());
        }
    }

    let repeated = repeated_lines(line_counts, max_repeated_lines);
    let final_errors = last_n(&error_lines, max_final_errors);
    let selected_warnings = last_n(&warning_lines, max_warnings);
    let summary = format!(
        "generic_error_log@v1: lines={total_lines}; final_errors={}; warnings={}; repeated_lines={}",
        final_errors.len(),
        selected_warnings.len(),
        repeated.len()
    );
    let mut output = ReducerOutput::new(summary);

    let dropped_or_truncated = total_lines > tail.len()
        || error_lines.len() > final_errors.len()
        || warning_lines.len() > selected_warnings.len()
        || repeated.len() == max_repeated_lines && repeated.len() < repeated_line_count(log);
    output.lossy = true;
    output.raw_required_for_full_debug = total_lines > 0;
    if dropped_or_truncated {
        output.add_warning(
            "generic_error_log omitted raw log content; use provenance/raw artifact for full debug",
        );
    }
    output.confidence = 0.85;

    output.add_fact("reducer", "generic_error_log@v1");
    output.add_fact("line_count", total_lines.to_string());
    output.add_fact("error_line_count", error_lines.len().to_string());
    output.add_fact("warning_line_count", warning_lines.len().to_string());

    for error in final_errors {
        let (line, truncated) = truncate(error, max_line_chars);
        if truncated {
            truncated_lines += 1;
        }
        let value = redact_text(redactor, &line, &mut output);
        output.add_fact("final_error", value);
    }

    for warning in selected_warnings {
        let (line, truncated) = truncate(warning, max_line_chars);
        if truncated {
            truncated_lines += 1;
        }
        let value = redact_text(redactor, &line, &mut output);
        output.add_warning(value.clone());
        output.add_fact("warning", value);
    }

    for (line, count) in repeated {
        let (line, truncated) = truncate(&line, max_line_chars);
        if truncated {
            truncated_lines += 1;
        }
        let value = format!("count={count} {line}");
        let value = redact_text(redactor, &value, &mut output);
        output.add_fact("repeated_line", value);
    }

    for line in tail {
        let (line, truncated) = truncate(&line, max_line_chars);
        if truncated {
            truncated_lines += 1;
        }
        let value = redact_text(redactor, &line, &mut output);
        output.add_fact("tail_context_line", value);
    }

    if truncated_lines > 0 {
        output.add_warning(format!(
            "generic_error_log@v1 truncated {truncated_lines} selected lines"
        ));
        output.lossy = true;
    }

    output.add_evidence("final_errors", !error_lines.is_empty());
    output.add_evidence("warnings", !warning_lines.is_empty());
    output.add_evidence("repeated_line_counts", true);
    output.add_evidence("tail_context", total_lines > 0);
    output.add_evidence("full_raw_log", false);

    output
}

fn is_error_line(lower: &str) -> bool {
    lower.contains("error")
        || lower.contains("failed")
        || lower.contains("failure")
        || lower.contains("panic")
        || lower.contains("fatal")
}

fn is_warning_line(lower: &str) -> bool {
    lower.contains("warning") || lower.starts_with("warn") || lower.contains(" warn:")
}

fn repeated_lines(
    counts: BTreeMap<String, usize>,
    max_repeated_lines: usize,
) -> Vec<(String, usize)> {
    let mut repeated: Vec<(String, usize)> =
        counts.into_iter().filter(|(_, count)| *count > 1).collect();
    repeated.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    repeated.truncate(max_repeated_lines);
    repeated
}

fn repeated_line_count(log: &str) -> usize {
    let mut counts = BTreeMap::new();
    for line in log.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *counts.entry(trimmed).or_insert(0usize) += 1;
        }
    }
    counts.values().filter(|count| **count > 1).count()
}

fn last_n<T>(values: &[T], n: usize) -> Vec<&T> {
    values.iter().skip(values.len().saturating_sub(n)).collect()
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
    fn generic_error_log_preserves_final_errors_warnings_repeats_and_tail() {
        let log = "\
starting worker
warn: retrying transient request
warn: retrying transient request
info: still running
error[E0425]: cannot find value `missing` in this scope
thread 'main' panicked at src/main.rs:9:5
fatal: process failed with exit code 2
";

        let output = reduce_generic_error_log(log);

        assert!(has_fact(
            &output,
            "final_error",
            "error[E0425]: cannot find value"
        ));
        assert!(has_fact(&output, "final_error", "exit code 2"));
        assert!(has_fact(
            &output,
            "warning",
            "warn: retrying transient request"
        ));
        assert!(has_fact(
            &output,
            "repeated_line",
            "count=2 warn: retrying transient request"
        ));
        assert!(has_fact(
            &output,
            "tail_context_line",
            "fatal: process failed"
        ));
        assert!(output.lossy);
    }
}
