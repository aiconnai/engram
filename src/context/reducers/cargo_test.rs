use super::output::ReducerOutput;
use super::redaction::{redact_text, NoopRedactor, Redactor};
use super::util::{parse_path_line_col, strip_ansi};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoTestReduction {
    pub exit_code: i32,
    pub final_verdict: Option<String>,
    pub failed_tests: Vec<String>,
    pub panics: Vec<CargoTestPanic>,
    pub cargo_errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoTestPanic {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u64>,
    pub column: Option<u64>,
}

pub fn parse_cargo_test_output(log: &str, exit_code: i32) -> CargoTestReduction {
    let mut final_verdict = None;
    let mut failed_tests = BTreeSet::new();
    let mut panics: Vec<CargoTestPanic> = Vec::new();
    let mut cargo_errors = Vec::new();
    let mut in_failures_section = false;
    let mut pending_panic_index: Option<usize> = None;

    for raw_line in log.lines() {
        let line = strip_ansi(raw_line);
        let trimmed = line.trim();

        if let Some(index) = pending_panic_index {
            if !trimmed.is_empty()
                && !trimmed.starts_with("note:")
                && !trimmed.starts_with("stack backtrace")
                && !trimmed.starts_with("---- ")
            {
                let panic = &mut panics[index];
                panic.message.push_str(" | ");
                panic.message.push_str(trimmed);
                pending_panic_index = None;
            }
        }

        if let Some(name) = trimmed
            .strip_prefix("test ")
            .and_then(|value| value.strip_suffix(" ... FAILED"))
        {
            failed_tests.insert(name.to_string());
        }

        if trimmed.starts_with("---- ") && trimmed.ends_with(" stdout ----") {
            let name = trimmed
                .trim_start_matches("---- ")
                .trim_end_matches(" stdout ----")
                .trim();
            if !name.is_empty() {
                failed_tests.insert(name.to_string());
            }
        }

        if trimmed == "failures:" {
            in_failures_section = true;
            continue;
        }

        if in_failures_section {
            if trimmed.starts_with("test result:") || trimmed.starts_with("error:") {
                in_failures_section = false;
            } else if raw_line.starts_with("    ")
                && !trimmed.is_empty()
                && !trimmed.starts_with("---- ")
                && !trimmed.contains(' ')
            {
                failed_tests.insert(trimmed.to_string());
            }
        }

        if trimmed.contains("panicked at ") {
            let (file, line_no, column) = parse_panic_location(trimmed)
                .map(|(file, line_no, column)| (Some(file), Some(line_no), column))
                .unwrap_or((None, None, None));
            panics.push(CargoTestPanic {
                message: trimmed.to_string(),
                file,
                line: line_no,
                column,
            });
            pending_panic_index = Some(panics.len() - 1);
        }

        if trimmed.starts_with("test result:") {
            final_verdict = Some(trimmed.to_string());
        }

        if trimmed.starts_with("error:") {
            cargo_errors.push(trimmed.to_string());
        }
    }

    CargoTestReduction {
        exit_code,
        final_verdict,
        failed_tests: failed_tests.into_iter().collect(),
        panics,
        cargo_errors,
    }
}

pub fn reduce_cargo_test(log: &str, exit_code: i32) -> ReducerOutput {
    reduce_cargo_test_with_redactor(log, exit_code, &NoopRedactor)
}

pub fn reduce_cargo_test_with_redactor(
    log: &str,
    exit_code: i32,
    redactor: &dyn Redactor,
) -> ReducerOutput {
    let parsed = parse_cargo_test_output(log, exit_code);
    let verdict = parsed.final_verdict.as_deref().unwrap_or("unknown verdict");
    let summary = format!(
        "cargo_test@v1: {verdict}; exit_code={exit_code}; failed_tests={}; panics={}",
        parsed.failed_tests.len(),
        parsed.panics.len()
    );
    let mut output = ReducerOutput::new(summary);

    output.lossy = true;
    output.raw_required_for_full_debug =
        exit_code != 0 || !parsed.failed_tests.is_empty() || !parsed.panics.is_empty();
    output.confidence = if parsed.final_verdict.is_some() {
        0.95
    } else if exit_code == 0 {
        0.8
    } else {
        0.65
    };

    output.add_fact("reducer", "cargo_test@v1");
    output.add_fact("exit_code", exit_code.to_string());
    if let Some(verdict) = &parsed.final_verdict {
        let value = redact_text(redactor, verdict, &mut output);
        output.add_fact("final_test_verdict", value);
    } else {
        output.add_warning("cargo_test@v1 could not find a final `test result:` verdict");
    }

    for failed_test in &parsed.failed_tests {
        let value = redact_text(redactor, failed_test, &mut output);
        output.add_fact("failed_test", value);
    }

    for panic in &parsed.panics {
        let message = redact_text(redactor, &panic.message, &mut output);
        output.add_fact("panic_message", message);
        if let (Some(file), Some(line)) = (&panic.file, panic.line) {
            let location = match panic.column {
                Some(column) => format!("{file}:{line}:{column}"),
                None => format!("{file}:{line}"),
            };
            let value = redact_text(redactor, &location, &mut output);
            output.add_fact("panic_location", value);
        }
    }

    for error in &parsed.cargo_errors {
        let value = redact_text(redactor, error, &mut output);
        output.add_fact("cargo_error", value);
    }

    output.add_evidence("exit_code", true);
    output.add_evidence("final_test_verdict", parsed.final_verdict.is_some());
    output.add_evidence("failed_test_names", !parsed.failed_tests.is_empty());
    output.add_evidence("panic_messages", !parsed.panics.is_empty());
    output.add_evidence(
        "panic_file_line_locations",
        parsed
            .panics
            .iter()
            .any(|panic| panic.file.is_some() && panic.line.is_some()),
    );
    output.add_evidence("raw_stdout_stderr_body", false);

    output
}

fn parse_panic_location(line: &str) -> Option<(String, u64, Option<u64>)> {
    let tail = line.split_once("panicked at ")?.1;
    for token in tail.split_whitespace() {
        let candidate = token
            .trim_matches('`')
            .trim_matches('\'')
            .trim_matches('"')
            .trim_end_matches(':')
            .trim_end_matches(',');
        if let Some(location) = parse_path_line_col(candidate) {
            return Some(location);
        }
    }

    None
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
    fn cargo_test_preserves_failed_test_panic_location_verdict_and_exit_code() {
        let log = r#"
running 2 tests
test tests::passes ... ok
test tests::fails_with_panic ... FAILED

failures:

---- tests::fails_with_panic stdout ----
thread 'tests::fails_with_panic' panicked at tests/reducer_fixture.rs:17:9:
assertion `left == right` failed
failures:
    tests::fails_with_panic

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
error: test failed, to rerun pass `--lib`
"#;

        let output = reduce_cargo_test(log, 101);

        assert!(has_fact(&output, "exit_code", "101"));
        assert!(has_fact(&output, "failed_test", "tests::fails_with_panic"));
        assert!(has_fact(
            &output,
            "panic_message",
            "assertion `left == right` failed"
        ));
        assert!(has_fact(
            &output,
            "panic_location",
            "tests/reducer_fixture.rs:17:9"
        ));
        assert!(has_fact(&output, "final_test_verdict", "FAILED. 1 passed"));
        assert!(has_fact(&output, "cargo_error", "error: test failed"));
        assert!(output.raw_required_for_full_debug);
    }
}
