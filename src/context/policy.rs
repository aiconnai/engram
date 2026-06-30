//! Redaction and sensitive-command policy for Operational Context persistence.
//!
//! Operational Context records are intentionally stricter than ordinary memory
//! writes: every summary, event, and raw artifact must be redacted before
//! durable storage, and sensitive commands default to ephemeral/no-raw behavior
//! unless a caller explicitly opts out through policy parameters.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::env;
use std::fmt;

static PRIVATE_KEY_RE: Lazy<std::result::Result<Regex, regex::Error>> = Lazy::new(|| {
    Regex::new(r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----")
});
static JWT_RE: Lazy<std::result::Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"));
static BEARER_RE: Lazy<std::result::Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{16,}"));
static OAUTH_RE: Lazy<std::result::Result<Regex, regex::Error>> = Lazy::new(|| {
    Regex::new(
        r"\b(?:ya29\.[A-Za-z0-9_-]+|gh[opsu]_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{10,})\b",
    )
});
static CLOUD_CREDENTIAL_RE: Lazy<std::result::Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b|\bAIza[0-9A-Za-z_-]{35}\b"));
static DATABASE_URL_RE: Lazy<std::result::Result<Regex, regex::Error>> = Lazy::new(|| {
    Regex::new(r#"(?i)\b(?:postgres(?:ql)?|mysql|mongodb(?:\+srv)?|redis)://[^\s'"]+"#)
});
static COOKIE_RE: Lazy<std::result::Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:cookie|set-cookie)\s*[:=]\s*[^\n]+"));
static ENV_SECRET_RE: Lazy<std::result::Result<Regex, regex::Error>> = Lazy::new(|| {
    Regex::new(
        r#"(?mi)^(\s*[A-Z0-9_]*(?:KEY|TOKEN|SECRET|PASSWORD|PASS|PWD|CREDENTIAL|AUTH)[A-Z0-9_]*\s*=\s*)['"]?[^\s'"]+"#,
    )
});
static API_KEY_RE: Lazy<std::result::Result<Regex, regex::Error>> = Lazy::new(|| {
    Regex::new(
        r#"(?i)\b([A-Z0-9_]*(?:api[_-]?key|apikey|token|secret|password|passwd|pwd|client_secret|access_token|refresh_token)[A-Z0-9_]*\b\s*[:=]\s*)['"]?[^\s'",;]{8,}"#,
    )
});
static EMAIL_RE: Lazy<std::result::Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b"));
static PHONE_RE: Lazy<std::result::Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\b\+?\d[\d .()/-]{7,}\d\b"));

/// Runtime policy for Operational Context redaction.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperationalContextPolicy {
    /// Redact email addresses only when explicitly configured.
    pub redact_emails: bool,
    /// Redact phone numbers only when explicitly configured.
    pub redact_phone_numbers: bool,
    /// Permit raw artifact persistence for sensitive commands.
    pub allow_sensitive_raw: bool,
    /// Permit permanent persistence for records associated with sensitive commands.
    pub allow_sensitive_command_persistence: bool,
    /// Extra repo/session patterns that must be redacted.
    pub custom_secret_patterns: Vec<String>,
}

impl OperationalContextPolicy {
    /// Build policy from environment variables.
    ///
    /// Supported booleans:
    /// - `ENGRAM_OC_REDACT_EMAILS`
    /// - `ENGRAM_OC_REDACT_PHONE_NUMBERS`
    /// - `ENGRAM_OC_ALLOW_SENSITIVE_RAW`
    /// - `ENGRAM_OC_ALLOW_SENSITIVE_COMMAND_PERSISTENCE`
    pub fn from_env() -> Self {
        let mut policy = Self::default();
        if let Some(value) = env_bool("ENGRAM_OC_REDACT_EMAILS") {
            policy.redact_emails = value;
        }
        if let Some(value) = env_bool("ENGRAM_OC_REDACT_PHONE_NUMBERS") {
            policy.redact_phone_numbers = value;
        }
        if let Some(value) = env_bool("ENGRAM_OC_ALLOW_SENSITIVE_RAW") {
            policy.allow_sensitive_raw = value;
        }
        if let Some(value) = env_bool("ENGRAM_OC_ALLOW_SENSITIVE_COMMAND_PERSISTENCE") {
            policy.allow_sensitive_command_persistence = value;
        }
        policy
    }

    /// Build policy from env plus per-call/session parameters.
    ///
    /// Callers may pass overrides either top-level or in one of:
    /// `operational_context_policy`, `context_policy`, or `redaction_policy`.
    pub fn from_params(params: &Value) -> Self {
        let mut policy = Self::from_env();
        policy.apply_value(params);
        for key in [
            "operational_context_policy",
            "context_policy",
            "redaction_policy",
        ] {
            if let Some(value) = params.get(key) {
                policy.apply_value(value);
            }
        }
        policy
    }

    /// Redact a single text field. Errors must be treated as fail-closed by callers.
    pub fn redact_text(&self, input: &str) -> std::result::Result<RedactedText, PolicyError> {
        let mut text = input.to_string();
        let mut classes = BTreeSet::new();

        apply_lazy_regex(
            &mut text,
            &PRIVATE_KEY_RE,
            "[REDACTED:private_key]",
            "private_key",
            &mut classes,
        )?;
        apply_lazy_regex(&mut text, &JWT_RE, "[REDACTED:jwt]", "jwt", &mut classes)?;
        apply_lazy_regex(
            &mut text,
            &BEARER_RE,
            "Bearer [REDACTED:bearer_token]",
            "bearer_token",
            &mut classes,
        )?;
        apply_lazy_regex(
            &mut text,
            &OAUTH_RE,
            "[REDACTED:oauth_token]",
            "oauth_token",
            &mut classes,
        )?;
        apply_lazy_regex(
            &mut text,
            &CLOUD_CREDENTIAL_RE,
            "[REDACTED:cloud_credential]",
            "cloud_credential",
            &mut classes,
        )?;
        apply_lazy_regex(
            &mut text,
            &DATABASE_URL_RE,
            "[REDACTED:database_url]",
            "database_url",
            &mut classes,
        )?;
        apply_lazy_regex(
            &mut text,
            &COOKIE_RE,
            "[REDACTED:cookie]",
            "cookie",
            &mut classes,
        )?;
        apply_lazy_regex(
            &mut text,
            &ENV_SECRET_RE,
            "$1[REDACTED:env_secret]",
            "env_secret",
            &mut classes,
        )?;
        apply_lazy_regex(
            &mut text,
            &API_KEY_RE,
            "$1[REDACTED:api_key]",
            "api_key",
            &mut classes,
        )?;

        if self.redact_emails {
            apply_lazy_regex(
                &mut text,
                &EMAIL_RE,
                "[REDACTED:email]",
                "email",
                &mut classes,
            )?;
        }
        if self.redact_phone_numbers {
            apply_lazy_regex(
                &mut text,
                &PHONE_RE,
                "[REDACTED:phone]",
                "phone",
                &mut classes,
            )?;
        }

        for pattern in &self.custom_secret_patterns {
            let re = Regex::new(pattern).map_err(|err| {
                PolicyError::RedactionFailed(format!("invalid custom redaction pattern: {err}"))
            })?;
            if re.is_match(&text) {
                text = re
                    .replace_all(&text, "[REDACTED:custom_secret]")
                    .into_owned();
                classes.insert("custom_secret".to_string());
            }
        }

        let redacted = !classes.is_empty();
        Ok(RedactedText {
            text,
            redacted,
            classes: classes.into_iter().collect(),
        })
    }

    /// Analyze a command string for sensitive Operational Context handling.
    pub fn analyze_command(&self, command: Option<&str>) -> SensitiveCommandAnalysis {
        let Some(command) = command.map(str::trim).filter(|s| !s.is_empty()) else {
            return SensitiveCommandAnalysis::default();
        };

        let lower = command.to_ascii_lowercase();
        let normalized = lower.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut analysis = SensitiveCommandAnalysis::default();

        if touches_env_file(&normalized) {
            analysis.add_reason("env_file_access");
        }
        if normalized == "printenv"
            || normalized.starts_with("printenv ")
            || normalized == "env"
            || normalized.starts_with("env |")
            || normalized.starts_with("env >")
        {
            analysis.add_reason("environment_dump");
        }
        if normalized.contains("aws sts") {
            analysis.add_reason("aws_sts");
        }
        if normalized.contains("aws secretsmanager")
            || normalized.contains("aws ssm get-parameter")
            || normalized.contains("aws configure")
        {
            analysis.add_reason("cloud_secret_command");
        }
        if normalized.contains("gh auth token")
            || normalized.contains("gh auth status --show-token")
        {
            analysis.add_reason("github_auth_token");
        }
        if normalized.contains("kubectl get secrets")
            || normalized.contains("kubectl get secret")
            || normalized.contains("kubectl describe secret")
        {
            analysis.add_reason("kubernetes_secret_command");
        }
        if (normalized.contains("prod") || normalized.contains("production"))
            && (normalized.contains(" log")
                || normalized.contains(" logs")
                || normalized.contains("dump")
                || normalized.contains("journalctl")
                || normalized.contains("kubectl logs"))
        {
            analysis.add_reason("production_log_dump");
        }
        if (normalized.contains("ci") || normalized.contains("github actions"))
            && (normalized.contains(" log")
                || normalized.contains("logs")
                || normalized.contains("artifact"))
        {
            analysis.add_reason("ci_log_artifact");
        }

        analysis
    }

    /// Sensitive commands must be persisted only as ephemeral records unless overridden.
    pub fn force_ephemeral(&self, analysis: &SensitiveCommandAnalysis) -> bool {
        analysis.is_sensitive && !self.allow_sensitive_command_persistence
    }

    /// Sensitive commands must not persist raw artifacts unless overridden.
    pub fn allow_raw_for(&self, analysis: &SensitiveCommandAnalysis) -> bool {
        !analysis.is_sensitive || self.allow_sensitive_raw
    }

    fn apply_value(&mut self, value: &Value) {
        if let Some(v) = value.get("redact_emails").and_then(parse_bool_value) {
            self.redact_emails = v;
        }
        if let Some(v) = value
            .get("redact_phone_numbers")
            .or_else(|| value.get("redact_phones"))
            .and_then(parse_bool_value)
        {
            self.redact_phone_numbers = v;
        }
        if let Some(v) = value.get("allow_sensitive_raw").and_then(parse_bool_value) {
            self.allow_sensitive_raw = v;
        }
        if let Some(v) = value
            .get("allow_sensitive_command_persistence")
            .or_else(|| value.get("allow_sensitive_persistence"))
            .and_then(parse_bool_value)
        {
            self.allow_sensitive_command_persistence = v;
        }
        if let Some(patterns) = value
            .get("custom_secret_patterns")
            .and_then(Value::as_array)
        {
            self.custom_secret_patterns = patterns
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
    }
}

/// Redacted output for one text field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedText {
    pub text: String,
    pub redacted: bool,
    pub classes: Vec<String>,
}

/// Accumulates redaction decisions for one event/summary/artifact.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RedactionReport {
    classes: BTreeSet<String>,
    fields: BTreeSet<String>,
}

impl RedactionReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, field: &str, redacted: &RedactedText) {
        if redacted.redacted {
            self.fields.insert(field.to_string());
            for class in &redacted.classes {
                self.classes.insert(class.clone());
            }
        }
    }

    pub fn has_redactions(&self) -> bool {
        !self.classes.is_empty()
    }

    pub fn to_value(
        &self,
        policy: &OperationalContextPolicy,
        sensitive: &SensitiveCommandAnalysis,
        raw_persistence: &str,
    ) -> Value {
        json!({
            "status": if self.has_redactions() { "redacted" } else { "clean" },
            "classes": self.classes.iter().cloned().collect::<Vec<_>>(),
            "fields": self.fields.iter().cloned().collect::<Vec<_>>(),
            "sensitive_command": sensitive.is_sensitive,
            "sensitive_reasons": sensitive.reasons.clone(),
            "forced_ephemeral": policy.force_ephemeral(sensitive),
            "raw_persistence": raw_persistence,
            "overrides": {
                "allow_sensitive_raw": policy.allow_sensitive_raw,
                "allow_sensitive_command_persistence": policy.allow_sensitive_command_persistence,
            },
            "policy": {
                "redact_emails": policy.redact_emails,
                "redact_phone_numbers": policy.redact_phone_numbers,
                "custom_secret_patterns_count": policy.custom_secret_patterns.len(),
            }
        })
    }
}

/// Sensitive command detection result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SensitiveCommandAnalysis {
    pub is_sensitive: bool,
    pub reasons: Vec<String>,
}

impl SensitiveCommandAnalysis {
    pub fn add_reason(&mut self, reason: &str) {
        self.is_sensitive = true;
        if !self.reasons.iter().any(|r| r == reason) {
            self.reasons.push(reason.to_string());
        }
    }

    pub fn merge(&mut self, other: SensitiveCommandAnalysis) {
        if other.is_sensitive {
            self.is_sensitive = true;
        }
        for reason in other.reasons {
            if !self.reasons.iter().any(|r| r == &reason) {
                self.reasons.push(reason);
            }
        }
    }
}

/// Policy errors. Callers must fail closed and avoid raw fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    RedactionFailed(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyError::RedactionFailed(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PolicyError {}

pub fn redact_field(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: &str,
) -> std::result::Result<String, PolicyError> {
    let redacted = policy.redact_text(value)?;
    report.record(field, &redacted);
    Ok(redacted.text)
}

pub fn redact_optional_field(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: &Option<String>,
) -> std::result::Result<Option<String>, PolicyError> {
    value
        .as_deref()
        .map(|s| redact_field(policy, report, field, s))
        .transpose()
}

pub fn redact_string_list(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    values: &[String],
) -> std::result::Result<Vec<String>, PolicyError> {
    values
        .iter()
        .map(|value| redact_field(policy, report, field, value))
        .collect()
}

pub fn failed_closed_metadata(reason: impl Into<String>) -> Value {
    json!({
        "status": "failed_closed",
        "classes": [],
        "fields": [],
        "sensitive_command": false,
        "sensitive_reasons": [],
        "forced_ephemeral": true,
        "raw_persistence": "blocked",
        "error": reason.into(),
    })
}

pub fn unknown_redaction_metadata() -> Value {
    json!({
        "status": "unknown",
        "classes": [],
        "fields": [],
        "sensitive_command": false,
        "sensitive_reasons": [],
        "forced_ephemeral": false,
        "raw_persistence": "unknown",
    })
}

pub fn command_hint_from_params(params: &Value) -> Option<String> {
    ["command", "cmd", "command_line"]
        .iter()
        .find_map(|key| params.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

pub fn command_hint_from_tool_use(
    params: &Value,
    tool_name: &str,
    tool_input: &Value,
) -> Option<String> {
    if let Some(command) = command_hint_from_params(params) {
        return Some(command);
    }

    if let Some(command) = tool_input
        .get("command")
        .or_else(|| tool_input.get("cmd"))
        .and_then(Value::as_str)
    {
        let args = tool_input
            .get("args")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|s| !s.is_empty());
        return Some(match args {
            Some(args) => format!("{command} {args}"),
            None => command.to_string(),
        });
    }

    if command_like_tool(tool_name) {
        if let Some(input) = tool_input.as_str() {
            return Some(input.to_string());
        }
        return Some(tool_input.to_string());
    }

    None
}

pub fn command_hint_from_archive(params: &Value, tool_name: &str) -> Option<String> {
    command_hint_from_params(params).or_else(|| {
        if command_like_tool(tool_name) {
            Some(tool_name.to_string())
        } else {
            None
        }
    })
}

fn apply_lazy_regex(
    text: &mut String,
    regex: &Lazy<std::result::Result<Regex, regex::Error>>,
    replacement: &str,
    class: &str,
    classes: &mut BTreeSet<String>,
) -> std::result::Result<(), PolicyError> {
    let re = match &**regex {
        Ok(re) => re,
        Err(err) => {
            return Err(PolicyError::RedactionFailed(format!(
                "built-in redaction pattern failed to compile: {err}"
            )));
        }
    };
    if re.is_match(text) {
        *text = re.replace_all(text.as_str(), replacement).into_owned();
        classes.insert(class.to_string());
    }
    Ok(())
}

fn env_bool(key: &str) -> Option<bool> {
    env::var(key).ok().as_deref().and_then(parse_bool_str)
}

fn parse_bool_value(value: &Value) -> Option<bool> {
    value
        .as_bool()
        .or_else(|| value.as_str().and_then(parse_bool_str))
}

fn parse_bool_str(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn touches_env_file(command: &str) -> bool {
    let reads_file = [
        "cat ", "less ", "more ", "tail ", "head ", "grep ", "rg ", "sed ", "awk ",
    ]
    .iter()
    .any(|prefix| command.starts_with(prefix) || command.contains(&format!("| {prefix}")));
    reads_file
        && (command.contains(".env")
            || command.contains("dotenv")
            || command.contains("secrets.env"))
}

fn command_like_tool(tool_name: &str) -> bool {
    let lower = tool_name.to_ascii_lowercase();
    [
        "shell",
        "bash",
        "zsh",
        "terminal",
        "exec_command",
        "command",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_core_secret_classes() {
        let policy = OperationalContextPolicy::default();
        let input = concat!(
            "OPENAI_API_KEY=sk-testkeyvalue1234567890\n",
            "Authorization: Bearer abcdefghijklmnopqrstuvwxyz\n",
            "jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.sflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c\n", // gitleaks:allow
            "postgres://user:secret@db.example/app\n",
            "Cookie: sid=secret-cookie-value\n",
            "AKIAIOSFODNN7EXAMPLE\n",
            "-----BEGIN PRIVATE KEY-----\nabc123\n-----END PRIVATE KEY-----"
        );

        let redacted = policy.redact_text(input).expect("redact");

        assert!(redacted.redacted);
        assert!(!redacted.text.contains("sk-testkeyvalue"));
        assert!(!redacted.text.contains("abcdefghijklmnopqrstuvwxyz"));
        assert!(!redacted.text.contains("postgres://user:secret"));
        assert!(!redacted.text.contains("secret-cookie-value"));
        assert!(!redacted.text.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(!redacted.text.contains("BEGIN PRIVATE KEY"));
        assert!(redacted.classes.iter().any(|c| c == "env_secret"));
        assert!(redacted.classes.iter().any(|c| c == "bearer_token"));
        assert!(redacted.classes.iter().any(|c| c == "jwt"));
        assert!(redacted.classes.iter().any(|c| c == "database_url"));
        assert!(redacted.classes.iter().any(|c| c == "cookie"));
        assert!(redacted.classes.iter().any(|c| c == "cloud_credential"));
        assert!(redacted.classes.iter().any(|c| c == "private_key"));
    }

    #[test]
    fn redacts_email_and_phone_only_when_configured() {
        let input = "Contact alice@example.com or +1 (415) 555-2671";

        let default_redaction = OperationalContextPolicy::default()
            .redact_text(input)
            .expect("default redact");
        assert_eq!(default_redaction.text, input);

        let policy = OperationalContextPolicy {
            redact_emails: true,
            redact_phone_numbers: true,
            ..Default::default()
        };
        let redacted = policy.redact_text(input).expect("configured redact");
        assert!(!redacted.text.contains("alice@example.com"));
        assert!(!redacted.text.contains("415"));
        assert!(redacted.classes.iter().any(|c| c == "email"));
        assert!(redacted.classes.iter().any(|c| c == "phone"));
    }

    #[test]
    fn detects_sensitive_command_examples() {
        let policy = OperationalContextPolicy::default();
        for command in [
            "cat .env",
            "printenv",
            "aws sts get-caller-identity",
            "gh auth token",
            "kubectl get secrets -n prod",
            "kubectl logs deployment/api -n production > prod.log",
            "cat ci-logs.txt",
        ] {
            let analysis = policy.analyze_command(Some(command));
            assert!(analysis.is_sensitive, "expected sensitive: {command}");
        }
    }

    #[test]
    fn sensitive_commands_force_ephemeral_and_no_raw_by_default() {
        let policy = OperationalContextPolicy::default();
        let analysis = policy.analyze_command(Some("gh auth token"));
        assert!(policy.force_ephemeral(&analysis));
        assert!(!policy.allow_raw_for(&analysis));

        let override_policy = OperationalContextPolicy {
            allow_sensitive_raw: true,
            allow_sensitive_command_persistence: true,
            ..Default::default()
        };
        assert!(!override_policy.force_ephemeral(&analysis));
        assert!(override_policy.allow_raw_for(&analysis));
    }

    #[test]
    fn invalid_custom_pattern_fails_closed() {
        let policy = OperationalContextPolicy {
            custom_secret_patterns: vec!["(".to_string()],
            ..Default::default()
        };

        let err = policy.redact_text("safe input").expect_err("must fail");
        assert!(err.to_string().contains("invalid custom redaction pattern"));
        let metadata = failed_closed_metadata(err.to_string());
        assert_eq!(metadata["status"], "failed_closed");
        assert_eq!(metadata["raw_persistence"], "blocked");
    }

    #[test]
    fn params_override_policy() {
        let policy = OperationalContextPolicy::from_params(&json!({
            "operational_context_policy": {
                "redact_emails": true,
                "redact_phone_numbers": true,
                "allow_sensitive_raw": true,
                "allow_sensitive_command_persistence": true,
                "custom_secret_patterns": ["BEGIN-CUSTOM-[A-Z]+"]
            }
        }));

        assert!(policy.redact_emails);
        assert!(policy.redact_phone_numbers);
        assert!(policy.allow_sensitive_raw);
        assert!(policy.allow_sensitive_command_persistence);
        assert_eq!(policy.custom_secret_patterns.len(), 1);
    }
}
