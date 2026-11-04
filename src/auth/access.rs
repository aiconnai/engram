//! Shared permission modes and persisted-memory authorization.
//!
//! Transport catalog classification and wire denials remain in their adapters.
use crate::auth::{Permission, PermissionSet, ResourceType, TransportPrincipal};
use crate::error::{EngramError, Result};
use crate::observability::{record_permission_denied, PermissionDeniedReason};
use rusqlite::{Connection, OptionalExtension};

pub(crate) const MODE_ENV: &str = "ENGRAM_PERMISSION_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionMode {
    ReadOnly,
    ScopedWrite,
    Maintenance,
    Admin,
}

impl PermissionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ScopedWrite => "scoped_write",
            Self::Maintenance => "maintenance",
            Self::Admin => "admin",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "read_only" => Some(Self::ReadOnly),
            "scoped_write" => Some(Self::ScopedWrite),
            "maintenance" => Some(Self::Maintenance),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    pub(crate) fn allows(self, required: Self) -> bool {
        self >= required
    }
}

pub(crate) fn allows_all_workspaces(principal: &TransportPrincipal) -> bool {
    !matches!(principal, TransportPrincipal::AnonymousLoopback(_))
        && principal.allows_workspace(None)
}

pub(crate) fn principal_allows_mode(
    principal: &TransportPrincipal,
    required: PermissionMode,
) -> bool {
    let permissions = &principal.auth_context().permissions;
    match required {
        PermissionMode::ReadOnly => {
            permissions.has_permission(Permission::Read, ResourceType::Memory)
        }
        PermissionMode::ScopedWrite => {
            permissions.has_permission(Permission::Write, ResourceType::Memory)
        }
        PermissionMode::Maintenance | PermissionMode::Admin => {
            permissions.has_permission(Permission::Admin, ResourceType::System)
        }
    }
}

pub(crate) fn principal_permission_mode(permissions: &PermissionSet) -> PermissionMode {
    if permissions.has_permission(Permission::Admin, ResourceType::System) {
        PermissionMode::Admin
    } else if permissions.has_permission(Permission::Write, ResourceType::Memory) {
        PermissionMode::ScopedWrite
    } else {
        PermissionMode::ReadOnly
    }
}

/// Mode required for Markdown filesystem operations, including import previews.
pub(crate) fn markdown_required_mode() -> PermissionMode {
    if std::env::var("ENGRAM_EXPORT_BASE_DIR").is_ok_and(|root| !root.trim().is_empty()) {
        PermissionMode::ScopedWrite
    } else {
        PermissionMode::Maintenance
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ModeDenial {
    #[error("{MODE_ENV} must be one of read_only, scoped_write, maintenance, admin")]
    InvalidConfiguration(String),
    #[error("operation requires {required:?} mode")]
    Insufficient {
        current: PermissionMode,
        required: PermissionMode,
    },
}

/// Evaluate process policy without depending on a tool name or JSON envelope.
pub(crate) fn check_environment_mode(
    required: PermissionMode,
) -> std::result::Result<(), ModeDenial> {
    let raw = match std::env::var(MODE_ENV) {
        Ok(value) if !value.trim().is_empty() => value,
        Ok(_) | Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(std::env::VarError::NotUnicode(_)) => {
            record_permission_denied(PermissionDeniedReason::InvalidModeConfig);
            return Err(ModeDenial::InvalidConfiguration("<non-unicode>".into()));
        }
    };
    let Some(current) = PermissionMode::parse(&raw) else {
        record_permission_denied(PermissionDeniedReason::InvalidModeConfig);
        return Err(ModeDenial::InvalidConfiguration(raw));
    };
    if current.allows(required) {
        return Ok(());
    }
    record_permission_denied(PermissionDeniedReason::ModeInsufficient);
    Err(ModeDenial::Insufficient { current, required })
}

/// Authorize on the same connection/snapshot as the caller's read or mutation.
/// Restricted principals cannot distinguish missing rows from foreign rows.
pub fn ensure_memory_access(
    conn: &Connection,
    principal: Option<&TransportPrincipal>,
    id: i64,
) -> Result<()> {
    let Some(principal) = principal.filter(|p| !allows_all_workspaces(p)) else {
        return Ok(());
    };
    let workspace: Option<String> = conn
        .query_row(
            "SELECT workspace FROM memories WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()?;
    match workspace {
        Some(workspace) if principal.allows_workspace(Some(&workspace)) => Ok(()),
        Some(_) | None => Err(EngramError::NotFound(id)),
    }
}
