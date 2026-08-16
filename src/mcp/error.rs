//! Normalized MCP tool error contract (RFC 0006).
//!
//! Provides a standardized, typed error structure for all MCP tool handlers
//! and transport layers, eliminating fragmented ad-hoc error formats.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::EngramError;

/// Standardized error codes for MCP tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorCode {
    InvalidParams,
    MissingArgument,
    NotFound,
    ToolNotFound,
    PermissionDenied,
    Conflict,
    VersionMismatch,
    RateLimited,
    InternalError,
}

impl ToolErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidParams => "invalid_params",
            Self::MissingArgument => "missing_argument",
            Self::NotFound => "not_found",
            Self::ToolNotFound => "tool_not_found",
            Self::PermissionDenied => "permission_denied",
            Self::Conflict => "conflict",
            Self::VersionMismatch => "version_mismatch",
            Self::RateLimited => "rate_limited",
            Self::InternalError => "internal_error",
        }
    }
}

/// Standardized error detail payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<String>,
}

/// Canonical MCP tool error wrapper matching `{ "error": { "code": ..., "message": ... } }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolError {
    pub error: ToolErrorDetail,
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.error.code, self.error.message)
    }
}

impl std::error::Error for ToolError {}

impl ToolError {
    /// Create a new tool error with code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ToolErrorDetail {
                code: code.into(),
                message: message.into(),
                tool: None,
                current_mode: None,
                required_mode: None,
                details: None,
                audit_id: None,
            },
        }
    }

    /// Attach structured context details to the error.
    pub fn with_details(mut self, details: Value) -> Self {
        self.error.details = Some(details);
        self
    }

    /// Attach an audit event ID for tracing.
    pub fn with_audit_id(mut self, audit_id: impl Into<String>) -> Self {
        self.error.audit_id = Some(audit_id.into());
        self
    }

    /// Helper for invalid parameters.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(ToolErrorCode::InvalidParams.as_str(), message)
    }

    /// Helper for missing required argument.
    pub fn missing_argument(arg: impl Into<String>) -> Self {
        let name = arg.into();
        Self::new(
            ToolErrorCode::MissingArgument.as_str(),
            format!("Missing required argument: '{name}'"),
        )
        .with_details(json!({ "argument": name }))
    }

    /// Helper for not found entity.
    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        let entity_name = entity.into();
        let id_str = id.into();
        Self::new(
            ToolErrorCode::NotFound.as_str(),
            format!("{entity_name} not found: '{id_str}'"),
        )
        .with_details(json!({ "entity": entity_name, "id": id_str }))
    }

    /// Helper for unknown tool.
    pub fn tool_not_found(tool_name: impl Into<String>) -> Self {
        let name = tool_name.into();
        let mut err = Self::new(
            ToolErrorCode::ToolNotFound.as_str(),
            format!("Unknown tool: {name}"),
        );
        err.error.tool = Some(name);
        err
    }

    /// Helper for permission denial.
    pub fn permission_denied(
        tool_name: impl Into<String>,
        current_mode: impl Into<String>,
        required_mode: impl Into<String>,
    ) -> Self {
        let tool = tool_name.into();
        let current = current_mode.into();
        let required = required_mode.into();
        let mut err = Self::new(
            ToolErrorCode::PermissionDenied.as_str(),
            format!("{tool} requires {required} mode"),
        );
        err.error.tool = Some(tool);
        err.error.current_mode = Some(current);
        err.error.required_mode = Some(required);
        err
    }

    /// Helper for version concurrency mismatch.
    pub fn version_mismatch(expected: i64, actual: i64) -> Self {
        Self::new(
            ToolErrorCode::VersionMismatch.as_str(),
            format!("Version mismatch: expected {expected}, but found {actual}"),
        )
        .with_details(json!({ "expected_version": expected, "actual_version": actual }))
    }

    /// Helper for resource conflicts.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ToolErrorCode::Conflict.as_str(), message)
    }

    /// Helper for internal server errors.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ToolErrorCode::InternalError.as_str(), message)
    }

    /// Convert into standard JSON response payload.
    pub fn into_value(self) -> Value {
        json!(self)
    }

    /// Check if a JSON response represents a tool error.
    pub fn is_error_response(value: &Value) -> bool {
        if let Some(err) = value.get("error") {
            if err.is_object() || err.is_string() {
                return true;
            }
        }
        false
    }

    /// Get reference to error code string.
    pub fn code(&self) -> &str {
        &self.error.code
    }

    /// Get reference to error message string.
    pub fn message(&self) -> &str {
        &self.error.message
    }
}

impl From<EngramError> for ToolError {
    fn from(err: EngramError) -> Self {
        match err {
            EngramError::NotFound(id) => Self::not_found("memory", id.to_string()),
            EngramError::InvalidInput(msg) => Self::invalid_params(msg),
            EngramError::Conflict(msg) => Self::conflict(msg),
            EngramError::Duplicate {
                existing_id,
                message,
            } => Self::conflict(message).with_details(json!({ "existing_id": existing_id })),
            EngramError::Unauthorized(msg) | EngramError::Auth(msg) => {
                Self::new(ToolErrorCode::PermissionDenied.as_str(), msg)
            }
            EngramError::RateLimited(secs) => Self::new(
                ToolErrorCode::RateLimited.as_str(),
                format!("Rate limited: retry after {secs} seconds"),
            )
            .with_details(json!({ "retry_after_seconds": secs })),
            EngramError::Database(e) => Self::internal(format!("Database error: {e}")),
            EngramError::Io(e) => Self::internal(format!("IO error: {e}")),
            other => Self::internal(other.to_string()),
        }
    }
}

/// Result alias for MCP tool execution.
pub type ToolResult<T> = std::result::Result<T, ToolError>;

/// Standard handler result producing a JSON value on success or a ToolError on failure.
pub type HandlerResult = ToolResult<Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_error_serialization() {
        let err = ToolError::missing_argument("workspace");
        let val = err.into_value();

        assert!(val.get("error").is_some());
        assert_eq!(val["error"]["code"], "missing_argument");
        assert!(val["error"]["message"]
            .as_str()
            .unwrap()
            .contains("workspace"));
        assert_eq!(val["error"]["details"]["argument"], "workspace");
    }

    #[test]
    fn test_permission_denied_shape() {
        let err = ToolError::permission_denied("memory_delete", "read_only", "admin");
        let val = err.into_value();

        assert_eq!(val["error"]["code"], "permission_denied");
        assert_eq!(val["error"]["tool"], "memory_delete");
        assert_eq!(val["error"]["current_mode"], "read_only");
        assert_eq!(val["error"]["required_mode"], "admin");
        assert_eq!(val["error"]["message"], "memory_delete requires admin mode");
    }

    #[test]
    fn test_tool_not_found_shape() {
        let err = ToolError::tool_not_found("unknown_tool");
        let val = err.into_value();

        assert_eq!(val["error"]["code"], "tool_not_found");
        assert_eq!(val["error"]["tool"], "unknown_tool");
        assert_eq!(val["error"]["message"], "Unknown tool: unknown_tool");
    }

    #[test]
    fn test_from_engram_error() {
        let engram_err = EngramError::NotFound(42);
        let tool_err: ToolError = engram_err.into();

        assert_eq!(tool_err.code(), "not_found");
        assert!(tool_err.message().contains("42"));
    }

    #[test]
    fn test_is_error_response() {
        assert!(ToolError::is_error_response(&json!({"error": "foo"})));
        assert!(ToolError::is_error_response(
            &json!({"error": {"code": "not_found", "message": "bar"}})
        ));
        assert!(!ToolError::is_error_response(
            &json!({"status": "success", "data": 123})
        ));
    }
}
