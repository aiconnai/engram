use super::output::{RedactionRecord, ReducerOutput};

/// Minimal redaction interface for reducers.
///
/// This intentionally stays policy-agnostic so the reducer layer can be wired to
/// a future repository-wide redaction policy without depending on it today.
pub trait Redactor: Send + Sync {
    fn redact(&self, input: &str) -> RedactionResult;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionResult {
    pub text: String,
    pub redactions: Vec<RedactionRecord>,
}

impl RedactionResult {
    pub fn unchanged(input: &str) -> Self {
        Self {
            text: input.to_string(),
            redactions: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRedactor;

impl Redactor for NoopRedactor {
    fn redact(&self, input: &str) -> RedactionResult {
        RedactionResult::unchanged(input)
    }
}

pub(crate) fn redact_text(
    redactor: &dyn Redactor,
    input: &str,
    output: &mut ReducerOutput,
) -> String {
    let redacted = redactor.redact(input);
    output.redactions.extend(redacted.redactions);
    redacted.text
}
