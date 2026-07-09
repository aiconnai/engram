//! Shared helpers for operational-context record splitting.

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde_json::{json, Map, Value};

use crate::context::policy::{
    redact_field, redact_optional_field, redact_string_list, OperationalContextPolicy,
    RedactedText, RedactionReport,
};
use crate::error::{EngramError, Result};
use crate::storage::{get_context_event, get_context_summary, ContextEvent};

pub(crate) fn reducer_name<'a>(
    source: &'a str,
    external_reducer: Option<&'a str>,
    reducer_name: Option<&'a str>,
) -> &'a str {
    reducer_name
        .and_then(non_empty)
        .or_else(|| external_reducer.and_then(non_empty))
        .unwrap_or(if source.eq_ignore_ascii_case("rtk") {
            "rtk_external_summary"
        } else {
            "context_record"
        })
}

pub(crate) fn reducer_version<'a>(
    source_version: Option<&'a str>,
    reducer_version: Option<&'a str>,
) -> &'a str {
    reducer_version
        .and_then(non_empty)
        .or_else(|| source_version.and_then(non_empty))
        .unwrap_or("1")
}

pub(crate) fn non_empty(value: &str) -> Option<&str> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn load_event(conn: &Connection, event_id: i64) -> Result<ContextEvent> {
    get_context_event(conn, event_id)?
        .ok_or_else(|| EngramError::Internal("context event insert was not readable".to_string()))
}

pub(crate) fn load_summary_id(conn: &Connection, summary_id: i64) -> Result<i64> {
    get_context_summary(conn, summary_id)?
        .map(|summary| summary.id)
        .ok_or_else(|| EngramError::Internal("context summary insert was not readable".to_string()))
}

pub(crate) fn redact_field_result(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: &str,
) -> Result<String> {
    redact_field(policy, report, field, value).map_err(redaction_error)
}

pub(crate) fn redact_optional_result(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: Option<String>,
) -> Result<Option<String>> {
    redact_optional_field(policy, report, field, &value).map_err(redaction_error)
}

pub(crate) fn redact_string_list_result(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    values: &[String],
) -> Result<Vec<String>> {
    redact_string_list(policy, report, field, values).map_err(redaction_error)
}

pub(crate) fn redact_json_value(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: Value,
) -> Result<Value> {
    match value {
        Value::String(value) => {
            redact_field_result(policy, report, field, &value).map(Value::String)
        }
        Value::Array(values) => values
            .into_iter()
            .enumerate()
            .map(|(idx, value)| {
                redact_json_value(policy, report, &format!("{field}[{idx}]"), value)
            })
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Value::Object(values) => {
            let mut output = Map::new();
            for (key, value) in values {
                let nested = format!("{field}.{key}");
                if sensitive_key(&key) {
                    report.record(
                        &nested,
                        &RedactedText {
                            text: String::new(),
                            redacted: true,
                            classes: vec!["metadata_sensitive_key".to_string()],
                        },
                    );
                    output.insert(
                        key,
                        Value::String("[REDACTED:metadata_sensitive_key]".to_string()),
                    );
                } else {
                    output.insert(key, redact_json_value(policy, report, &nested, value)?);
                }
            }
            Ok(Value::Object(output))
        }
        value => Ok(value),
    }
}

pub(crate) fn metadata_map(metadata: Option<Value>) -> Map<String, Value> {
    match metadata {
        Some(Value::Object(map)) => map,
        Some(value) => {
            let mut map = Map::new();
            map.insert("value".to_string(), value);
            map
        }
        None => Map::new(),
    }
}

pub(crate) fn object_map(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("value".to_string(), other);
            map
        }
    }
}

pub(crate) fn insert_opt(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_string(), json!(value));
    }
}

pub(crate) fn normalized_labels(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut labels = Vec::new();
    for value in values {
        let label = value.trim().to_ascii_lowercase();
        if !label.is_empty() && !labels.iter().any(|existing| existing == &label) {
            labels.push(label);
        }
    }
    labels
}

pub(crate) fn push_label(labels: &mut Vec<String>, label: &str) {
    if !labels.iter().any(|existing| existing == label) {
        labels.push(label.to_string());
    }
}

pub(crate) fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

pub(crate) fn require_non_empty(value: String, field: &str) -> Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(EngramError::InvalidInput(format!("{field} is required")))
    } else {
        Ok(value)
    }
}

pub(crate) fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub(crate) fn optional_i32(value: Option<i64>, field: &str) -> Result<Option<i32>> {
    value
        .map(|value| {
            i32::try_from(value).map_err(|_| {
                EngramError::InvalidInput(format!("{field} must fit in a 32-bit integer"))
            })
        })
        .transpose()
}

pub(crate) fn parse_datetime_or_now(value: Option<String>, field: &str) -> Result<DateTime<Utc>> {
    parse_optional_datetime(value, field).map(|value| value.unwrap_or_else(Utc::now))
}

pub(crate) fn parse_optional_datetime(
    value: Option<String>,
    field: &str,
) -> Result<Option<DateTime<Utc>>> {
    let Some(value) = clean_optional(value) else {
        return Ok(None);
    };
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(|err| EngramError::InvalidInput(format!("{field} must be RFC3339: {err}")))
}

pub(crate) fn sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("authorization")
        || lower.contains("cookie")
}

pub(crate) fn redaction_error(err: impl std::fmt::Display) -> EngramError {
    EngramError::InvalidInput(format!("operational context redaction failed: {err}"))
}
