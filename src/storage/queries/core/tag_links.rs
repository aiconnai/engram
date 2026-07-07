use super::*;

/// Ensure a tag exists and return its ID
pub(super) fn ensure_tag(conn: &Connection, tag: &str) -> Result<i64> {
    conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", params![tag])?;

    let id: i64 = conn.query_row("SELECT id FROM tags WHERE name = ?", params![tag], |row| {
        row.get(0)
    })?;

    Ok(id)
}
