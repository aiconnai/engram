//! MCP tool error normalization utilities (RFC 0006 / G-7).
//!
//! Provides normalization of ad-hoc and legacy `{"error": "<string>"}` payloads into
//! the canonical RFC 0006 structured error shape `{"error": {"code": ..., "message": ...}}`.

use serde_json::{json, Value};

use crate::mcp::error::ToolErrorCode;

/// Infer the RFC 0006 error code from a legacy error message string.
pub fn infer_error_code(msg: &str) -> &'static str {
    let lower = msg.to_ascii_lowercase();
    if lower.contains("is required") || lower.contains("missing") {
        ToolErrorCode::MissingArgument.as_str()
    } else if lower.contains("not found") {
        ToolErrorCode::NotFound.as_str()
    } else if lower.contains("permission")
        || lower.contains("unauthorized")
        || lower.contains("requires")
    {
        ToolErrorCode::PermissionDenied.as_str()
    } else if lower.contains("conflict") || lower.contains("duplicate") {
        ToolErrorCode::Conflict.as_str()
    } else {
        ToolErrorCode::InvalidParams.as_str()
    }
}

/// Normalize a tool output JSON payload.
///
/// If `value` contains `{"error": "<string>"}`, it upgrades the error into:
/// `{"error": {"code": inferred_code, "message": msg}}`
/// while preserving any other existing fields.
/// If `value` already contains a structured error object or is a success response, it is returned unchanged.
pub fn normalize_tool_output(mut value: Value) -> Value {
    if let Value::Object(ref mut map) = value {
        if let Some(err_val) = map.get("error") {
            if let Some(msg) = err_val.as_str() {
                let code = infer_error_code(msg);
                map.insert(
                    "error".to_string(),
                    json!({
                        "code": code,
                        "message": msg,
                    }),
                );
            }
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_missing_argument() {
        assert_eq!(infer_error_code("name is required"), "missing_argument");
        assert_eq!(
            infer_error_code("Missing required argument: 'id'"),
            "missing_argument"
        );
    }

    #[test]
    fn test_infer_not_found() {
        assert_eq!(infer_error_code("Memory 42 not found"), "not_found");
        assert_eq!(infer_error_code("block 'intro' not found"), "not_found");
    }

    #[test]
    fn test_infer_invalid_params() {
        assert_eq!(
            infer_error_code("limit must be a non-negative integer"),
            "invalid_params"
        );
        assert_eq!(infer_error_code("id must be an integer"), "invalid_params");
        assert_eq!(infer_error_code("out of range"), "invalid_params");
    }

    #[test]
    fn test_normalize_legacy_string_error() {
        let legacy = json!({"error": "name is required"});
        let normalized = normalize_tool_output(legacy);

        assert_eq!(normalized["error"]["code"], "missing_argument");
        assert_eq!(normalized["error"]["message"], "name is required");
    }

    #[test]
    fn test_normalize_preserves_structured_error() {
        let structured = json!({
            "error": {
                "code": "permission_denied",
                "message": "admin required"
            }
        });
        let normalized = normalize_tool_output(structured.clone());
        assert_eq!(normalized, structured);
    }

    #[test]
    fn test_normalize_preserves_success() {
        let success = json!({"status": "success", "id": 123});
        let normalized = normalize_tool_output(success.clone());
        assert_eq!(normalized, success);
    }
}
