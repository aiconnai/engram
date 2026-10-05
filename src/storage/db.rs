//! Desmonomorphized query helpers for SQLite operations.
//!
//! Provides single-compilation execution trampolines for common queries
//! (scalars, single rows, existence checks), preventing duplicate LLVM IR
//! expansion across hundreds of rusqlite callsites.

use rusqlite::types::{FromSql, ToSql};
use rusqlite::{Connection, Result, Row};

/// Execute a query expecting a single row, delegating statement preparation
/// and stepping to a non-generic execution core to minimize LLVM IR generation.
pub fn query_row_dyn<T>(
    conn: &Connection,
    sql: &str,
    params: &[&dyn ToSql],
    f: &mut dyn FnMut(&Row<'_>) -> Result<T>,
) -> Result<T> {
    let mut stmt = conn.prepare_cached(sql)?;
    let mut rows = stmt.query(params)?;
    match rows.next()? {
        Some(row) => f(row),
        None => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

/// Execute a query with a closure using dynamic dispatch for the inner row mapping.
pub fn query_row_with<T, F>(conn: &Connection, sql: &str, params: &[&dyn ToSql], f: F) -> Result<T>
where
    F: FnOnce(&Row<'_>) -> Result<T>,
{
    let mut f = Some(f);
    let mut res = None;
    query_row_dyn(conn, sql, params, &mut |row| {
        let func = f.take().expect("closure runs once");
        res = Some(func(row)?);
        Ok(())
    })?;
    Ok(res.expect("row must exist"))
}

/// Query a single scalar column from the first matching row.
pub fn query_scalar<T: FromSql>(conn: &Connection, sql: &str, params: &[&dyn ToSql]) -> Result<T> {
    let mut val = None;
    query_row_dyn(conn, sql, params, &mut |row| {
        val = Some(row.get(0)?);
        Ok(())
    })?;
    Ok(val.expect("row must exist"))
}

/// Query a single scalar column with no parameters.
pub fn query_scalar_0<T: FromSql>(conn: &Connection, sql: &str) -> Result<T> {
    query_scalar(conn, sql, &[])
}

/// Query an optional scalar column from the first matching row.
pub fn query_opt_scalar<T: FromSql>(
    conn: &Connection,
    sql: &str,
    params: &[&dyn ToSql],
) -> Result<Option<T>> {
    let mut stmt = conn.prepare_cached(sql)?;
    let mut rows = stmt.query(params)?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

/// Check if a query returns at least one row.
pub fn query_exists(conn: &Connection, sql: &str, params: &[&dyn ToSql]) -> Result<bool> {
    let mut stmt = conn.prepare_cached(sql)?;
    let mut rows = stmt.query(params)?;
    Ok(rows.next()?.is_some())
}

/// Extension trait for [`Connection`] adding desmonomorphized query helpers.
pub trait DbConnectionExt {
    /// Query a single scalar column from the first matching row.
    fn query_scalar<T: FromSql>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<T>;

    /// Query a single scalar column with no parameters.
    fn query_scalar_0<T: FromSql>(&self, sql: &str) -> Result<T>;

    /// Query an optional scalar column from the first matching row.
    fn query_opt_scalar<T: FromSql>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Option<T>>;

    /// Check if a query returns at least one row.
    fn query_exists(&self, sql: &str, params: &[&dyn ToSql]) -> Result<bool>;
}

impl DbConnectionExt for Connection {
    #[inline]
    fn query_scalar<T: FromSql>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<T> {
        query_scalar(self, sql, params)
    }

    #[inline]
    fn query_scalar_0<T: FromSql>(&self, sql: &str) -> Result<T> {
        query_scalar_0(self, sql)
    }

    #[inline]
    fn query_opt_scalar<T: FromSql>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Option<T>> {
        query_opt_scalar(self, sql, params)
    }

    #[inline]
    fn query_exists(&self, sql: &str, params: &[&dyn ToSql]) -> Result<bool> {
        query_exists(self, sql, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_helpers() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE test (id INTEGER, name TEXT)", [])
            .unwrap();
        conn.execute("INSERT INTO test VALUES (42, 'hello')", [])
            .unwrap();

        let count: i64 = conn.query_scalar_0("SELECT count(*) FROM test").unwrap();
        assert_eq!(count, 1);

        let id: i64 = conn
            .query_scalar("SELECT id FROM test WHERE name = ?", &[&"hello"])
            .unwrap();
        assert_eq!(id, 42);

        let missing: Option<i64> = conn
            .query_opt_scalar("SELECT id FROM test WHERE name = ?", &[&"nonexistent"])
            .unwrap();
        assert_eq!(missing, None);

        let exists = conn
            .query_exists("SELECT 1 FROM test WHERE id = ?", &[&42i64])
            .unwrap();
        assert!(exists);

        let not_exists = conn
            .query_exists("SELECT 1 FROM test WHERE id = ?", &[&99i64])
            .unwrap();
        assert!(!not_exists);
    }
}
