use super::output::ReducerOutput;
use super::redaction::{redact_text, NoopRedactor, Redactor};
use super::util::{extract_clippy_rule, parse_path_line_col, strip_ansi};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoClippyDiagnostic {
    pub severity: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u64>,
    pub column: Option<u64>,
    pub rule: Option<String>,
    pub help: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoClippyDiagnosticGroup {
    pub file: String,
    pub rule: String,
    pub severity: String,
    pub diagnostics: usize,
    pub spans: Vec<ActionableSpan>,
    pub messages: Vec<String>,
    pub help: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActionableSpan {
    pub file: String,
    pub line: u64,
    pub column: Option<u64>,
}

pub fn parse_cargo_clippy_diagnostics(log: &str) -> Vec<CargoClippyDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut current: Option<CargoClippyDiagnostic> = None;

    for raw_line in log.lines() {
        let line = strip_ansi(raw_line);
        let trimmed = line.trim();

        if let Some((severity, message)) = parse_severity_line(trimmed) {
            if let Some(diagnostic) = current.take() {
                diagnostics.push(diagnostic);
            }
            current = Some(CargoClippyDiagnostic {
                severity,
                rule: extract_clippy_rule(trimmed),
                message,
                file: None,
                line: None,
                column: None,
                help: Vec::new(),
            });
            continue;
        }

        let Some(diagnostic) = current.as_mut() else {
            continue;
        };

        if diagnostic.rule.is_none() {
            diagnostic.rule = extract_clippy_rule(trimmed);
        }

        if let Some((file, line_no, column)) = parse_location_marker(trimmed) {
            diagnostic.file = Some(file);
            diagnostic.line = Some(line_no);
            diagnostic.column = column;
        }

        if let Some(help) = parse_help_line(trimmed) {
            diagnostic.help.push(help.to_string());
        }
    }

    if let Some(diagnostic) = current {
        diagnostics.push(diagnostic);
    }

    diagnostics
}

pub fn reduce_cargo_clippy(log: &str, exit_code: i32) -> ReducerOutput {
    reduce_cargo_clippy_with_redactor(log, exit_code, &NoopRedactor)
}

pub fn reduce_cargo_clippy_with_redactor(
    log: &str,
    exit_code: i32,
    redactor: &dyn Redactor,
) -> ReducerOutput {
    let diagnostics = parse_cargo_clippy_diagnostics(log);
    let groups = group_diagnostics(&diagnostics);
    let summary = format!(
        "cargo_clippy@v1: diagnostic_groups={}; diagnostics={}; exit_code={exit_code}",
        groups.len(),
        diagnostics.len()
    );
    let mut output = ReducerOutput::new(summary);

    output.lossy = true;
    output.raw_required_for_full_debug = !diagnostics.is_empty();
    output.confidence = if !diagnostics.is_empty() || exit_code == 0 {
        0.9
    } else {
        0.6
    };

    output.add_fact("reducer", "cargo_clippy@v1");
    output.add_fact("exit_code", exit_code.to_string());

    if diagnostics.is_empty() && exit_code != 0 {
        output.add_warning("cargo_clippy@v1 saw a non-zero exit code but parsed no diagnostics");
    }

    for group in groups.values() {
        let mut metadata = BTreeMap::new();
        metadata.insert("file".to_string(), group.file.clone());
        metadata.insert("rule".to_string(), group.rule.clone());
        metadata.insert("severity".to_string(), group.severity.clone());
        metadata.insert("diagnostics".to_string(), group.diagnostics.to_string());
        let value = format!(
            "{} {} {} count={}",
            group.file, group.rule, group.severity, group.diagnostics
        );
        let value = redact_text(redactor, &value, &mut output);
        output.add_fact_with_metadata("diagnostic_group", value, metadata);

        for span in &group.spans {
            let location = match span.column {
                Some(column) => format!("{}:{line}:{column}", span.file, line = span.line),
                None => format!("{}:{}", span.file, span.line),
            };
            let mut metadata = BTreeMap::new();
            metadata.insert("file".to_string(), span.file.clone());
            metadata.insert("line".to_string(), span.line.to_string());
            metadata.insert("rule".to_string(), group.rule.clone());
            metadata.insert("severity".to_string(), group.severity.clone());
            let value = redact_text(redactor, &location, &mut output);
            output.add_fact_with_metadata("actionable_span", value, metadata);
        }

        for message in &group.messages {
            let value = redact_text(redactor, message, &mut output);
            output.add_fact("diagnostic_message", value);
        }

        for help in &group.help {
            let value = redact_text(redactor, help, &mut output);
            output.add_fact("diagnostic_help", value);
        }
    }

    output.add_evidence("exit_code", true);
    output.add_evidence(
        "diagnostic_groups_by_file_rule_severity",
        !groups.is_empty(),
    );
    output.add_evidence(
        "actionable_file_line_column_spans",
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.file.is_some() && diagnostic.line.is_some()),
    );
    output.add_evidence(
        "clippy_rules",
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule.is_some()),
    );
    output.add_evidence("raw_code_snippets", false);

    output
}

fn parse_severity_line(line: &str) -> Option<(String, String)> {
    let colon = line.find(':')?;
    let head = &line[..colon];
    let severity = if head == "warning" || head.starts_with("warning[") {
        "warning"
    } else if head == "error" || head.starts_with("error[") {
        "error"
    } else {
        return None;
    };
    let message = line[colon + 1..].trim().to_string();
    Some((severity.to_string(), message))
}

fn parse_location_marker(line: &str) -> Option<(String, u64, Option<u64>)> {
    let rest = line.strip_prefix("--> ")?;
    let location = rest.split_whitespace().next()?;
    parse_path_line_col(location)
}

fn parse_help_line(line: &str) -> Option<&str> {
    line.strip_prefix("help:")
        .or_else(|| line.strip_prefix("= help:"))
        .map(str::trim)
}

fn group_diagnostics(
    diagnostics: &[CargoClippyDiagnostic],
) -> BTreeMap<(String, String, String), CargoClippyDiagnosticGroup> {
    let mut groups = BTreeMap::new();

    for diagnostic in diagnostics {
        let file = diagnostic
            .file
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        let rule = diagnostic
            .rule
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        let key = (file.clone(), rule.clone(), diagnostic.severity.clone());
        let group = groups
            .entry(key)
            .or_insert_with(|| CargoClippyDiagnosticGroup {
                file,
                rule,
                severity: diagnostic.severity.clone(),
                diagnostics: 0,
                spans: Vec::new(),
                messages: Vec::new(),
                help: Vec::new(),
            });

        group.diagnostics += 1;
        group.messages.push(diagnostic.message.clone());
        group.help.extend(diagnostic.help.clone());

        if let (Some(file), Some(line)) = (&diagnostic.file, diagnostic.line) {
            let span = ActionableSpan {
                file: file.clone(),
                line,
                column: diagnostic.column,
            };
            if !group.spans.contains(&span) {
                group.spans.push(span);
            }
        }
    }

    for group in groups.values_mut() {
        let mut seen_messages = BTreeSet::new();
        group
            .messages
            .retain(|message| seen_messages.insert(message.clone()));

        let mut seen_help = BTreeSet::new();
        group.help.retain(|help| seen_help.insert(help.clone()));
    }

    groups
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
    fn cargo_clippy_groups_by_file_rule_severity_and_preserves_span() {
        let log = r#"
warning: used `unwrap()` on a `Result` value
  --> src/mcp/handler.rs:42:18
   |
42 |     value.unwrap()
   |                  ^ help: consider using `expect`
   = note: `-D clippy::unwrap_used` implied by `-D warnings`
help: consider handling the error explicitly
error: this expression creates a reference which is immediately dereferenced
  --> src/mcp/handler.rs:50:9
   = note: `-D clippy::needless_borrow` implied by `-D warnings`
"#;

        let output = reduce_cargo_clippy(log, 1);

        assert!(has_fact(
            &output,
            "diagnostic_group",
            "src/mcp/handler.rs clippy::unwrap_used warning count=1"
        ));
        assert!(has_fact(
            &output,
            "diagnostic_group",
            "src/mcp/handler.rs clippy::needless_borrow error count=1"
        ));
        assert!(has_fact(
            &output,
            "actionable_span",
            "src/mcp/handler.rs:42:18"
        ));
        assert!(has_fact(
            &output,
            "actionable_span",
            "src/mcp/handler.rs:50:9"
        ));
        assert!(has_fact(
            &output,
            "diagnostic_help",
            "consider handling the error explicitly"
        ));
        assert!(output.raw_required_for_full_debug);
    }
}
