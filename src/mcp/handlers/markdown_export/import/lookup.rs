use crate::error::EngramError;
use crate::mcp::handlers::HandlerContext;

pub(super) fn db_state_for_memory(
    ctx: &HandlerContext,
    engram_id: i64,
) -> Result<Option<(String, i64)>, EngramError> {
    ctx.storage.with_connection(|conn| {
        let result = conn.query_row(
            "SELECT content_hash, version FROM memories WHERE id = ?1",
            rusqlite::params![engram_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngramError::Database(e)),
        }
    })
}
