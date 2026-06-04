//! Append-only enrichment event log (ENG-1240).

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::Result;

pub struct EnrichmentEvent<'a> {
    pub operation_id: &'a str,
    pub event_type: &'a str,
    pub memory_id: Option<i64>,
    pub version_id: Option<i64>,
    pub triggered_by: &'a str,
    pub agent_id: Option<&'a str>,
    pub workspace: Option<&'a str>,
    pub params: serde_json::Value,
    pub outcome: serde_json::Value,
    pub status: &'a str,
    pub dry_run: bool,
}

pub fn emit(conn: &Connection, event: &EnrichmentEvent<'_>) -> Result<i64> {
    if event.operation_id.is_empty() {
        return Err(crate::error::EngramError::Internal(
            "enrichment_events: operation_id must not be empty".into(),
        ));
    }
    let params_str = serde_json::to_string(&event.params).unwrap_or_else(|_| "{}".to_string());
    let outcome_str = serde_json::to_string(&event.outcome).unwrap_or_else(|_| "{}".to_string());
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO enrichment_events
             (operation_id, event_type, memory_id, version_id, triggered_by,
              agent_id, workspace, params, outcome, status, dry_run, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            event.operation_id,
            event.event_type,
            event.memory_id,
            event.version_id,
            event.triggered_by,
            event.agent_id,
            event.workspace,
            params_str,
            outcome_str,
            event.status,
            event.dry_run as i32,
            created_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn emit_best_effort(conn: &Connection, event: &EnrichmentEvent<'_>) -> Option<i64> {
    emit(conn, event)
        .map_err(|e| tracing::warn!("enrichment_events emit failed: {e}"))
        .ok()
}

pub fn latest_version_id(conn: &Connection, memory_id: i64) -> Result<Option<i64>> {
    use rusqlite::OptionalExtension;
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM memory_versions WHERE memory_id = ?1 ORDER BY version DESC LIMIT 1",
            params![memory_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::run_migrations;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_emit_persists_all_fields() {
        let conn = test_conn();
        let event = EnrichmentEvent {
            operation_id: "op-abc-123",
            event_type: "consolidation",
            memory_id: Some(42),
            version_id: None,
            triggered_by: "memory_consolidate_batch",
            agent_id: Some("agent-x"),
            workspace: Some("default"),
            params: serde_json::json!({"threshold": 0.8}),
            outcome: serde_json::json!({"merged": 3}),
            status: "completed",
            dry_run: false,
        };
        let id = emit(&conn, &event).expect("emit should succeed");
        assert!(id > 0);

        let (op_id, ev_type, mem_id, trig_by, ag_id, ws, status_val, dry_val): (
            String,
            String,
            Option<i64>,
            String,
            Option<String>,
            Option<String>,
            String,
            i32,
        ) = conn
            .query_row(
                "SELECT operation_id, event_type, memory_id, triggered_by, agent_id,
                         workspace, status, dry_run
                 FROM enrichment_events WHERE id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get(7)?,
                    ))
                },
            )
            .unwrap();

        assert_eq!(op_id, "op-abc-123");
        assert_eq!(ev_type, "consolidation");
        assert_eq!(mem_id, Some(42));
        assert_eq!(trig_by, "memory_consolidate_batch");
        assert_eq!(ag_id.as_deref(), Some("agent-x"));
        assert_eq!(ws.as_deref(), Some("default"));
        assert_eq!(status_val, "completed");
        assert_eq!(dry_val, 0);
    }

    #[test]
    fn test_emit_rejects_empty_operation_id() {
        let conn = test_conn();
        let event = EnrichmentEvent {
            operation_id: "",
            event_type: "test",
            memory_id: None,
            version_id: None,
            triggered_by: "test",
            agent_id: None,
            workspace: None,
            params: serde_json::json!({}),
            outcome: serde_json::json!({}),
            status: "completed",
            dry_run: false,
        };
        assert!(
            emit(&conn, &event).is_err(),
            "empty operation_id must be rejected"
        );
    }

    #[test]
    fn test_emit_best_effort_returns_none_on_missing_table() {
        let conn = Connection::open_in_memory().unwrap();
        // No migrations — table doesn't exist
        let event = EnrichmentEvent {
            operation_id: "op-1",
            event_type: "test",
            memory_id: None,
            version_id: None,
            triggered_by: "test",
            agent_id: None,
            workspace: None,
            params: serde_json::json!({}),
            outcome: serde_json::json!({}),
            status: "completed",
            dry_run: false,
        };
        let result = emit_best_effort(&conn, &event);
        assert!(result.is_none(), "should return None on DB error");
    }

    #[test]
    fn test_latest_version_id_returns_none_when_no_versions() {
        let conn = test_conn();
        let result = latest_version_id(&conn, 99999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_latest_version_id_returns_id_for_existing_version() {
        let conn = test_conn();
        // Insert a memory first (required for FK in memory_versions)
        conn.execute(
            "INSERT INTO memories (content, memory_type, importance, visibility, metadata, valid_from)
             VALUES ('test', 'note', 0.5, 'private', '{}', CURRENT_TIMESTAMP)",
            [],
        ).unwrap();
        let memory_id = conn.last_insert_rowid();

        // Insert a memory_versions row
        conn.execute(
            "INSERT INTO memory_versions (memory_id, version, content, tags, created_at)
             VALUES (?1, 1, 'v1 content', '[]', '2026-01-01T00:00:00Z')",
            rusqlite::params![memory_id],
        )
        .unwrap();
        let version_row_id = conn.last_insert_rowid();

        let result = latest_version_id(&conn, memory_id).unwrap();
        assert_eq!(
            result,
            Some(version_row_id),
            "should return the PK of the latest memory_versions row"
        );
    }
}
