#![allow(dead_code)] // list/clear wired in Phase 4

use rusqlite::{params, Connection as SqliteConn};
use serde::Serialize;

use crate::error::AppResult;
use crate::models::ConnId;
use crate::store::now_iso;

/// Keep at most this many history rows per connection.
const MAX_PER_CONN: i64 = 5000;

#[derive(Debug, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub sql: String,
    pub success: bool,
    pub error: Option<String>,
    pub row_count: Option<i64>,
    pub elapsed_ms: Option<i64>,
    pub executed_at: String,
}

pub fn record(
    conn: &SqliteConn,
    connection_id: ConnId,
    sql: &str,
    success: bool,
    error: Option<&str>,
    row_count: Option<i64>,
    elapsed_ms: Option<i64>,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO query_history
            (connection_id, sql, success, error, row_count, elapsed_ms, executed_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            connection_id,
            sql,
            success as i64,
            error,
            row_count,
            elapsed_ms,
            now_iso()
        ],
    )?;
    // Prune oldest beyond the cap.
    conn.execute(
        "DELETE FROM query_history WHERE id IN (
            SELECT id FROM query_history WHERE connection_id=?1
            ORDER BY executed_at DESC LIMIT -1 OFFSET ?2)",
        params![connection_id, MAX_PER_CONN],
    )?;
    Ok(())
}

pub fn list(
    conn: &SqliteConn,
    connection_id: ConnId,
    limit: i64,
    offset: i64,
    search: Option<&str>,
) -> AppResult<Vec<HistoryEntry>> {
    let like = search.map(|s| format!("%{s}%"));
    let mut stmt = conn.prepare(
        "SELECT id, sql, success, error, row_count, elapsed_ms, executed_at
         FROM query_history
         WHERE connection_id=?1 AND (?2 IS NULL OR sql LIKE ?2)
         ORDER BY executed_at DESC LIMIT ?3 OFFSET ?4",
    )?;
    let rows = stmt.query_map(params![connection_id, like, limit, offset], |r| {
        Ok(HistoryEntry {
            id: r.get(0)?,
            sql: r.get(1)?,
            success: r.get::<_, i64>(2)? != 0,
            error: r.get(3)?,
            row_count: r.get(4)?,
            elapsed_ms: r.get(5)?,
            executed_at: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn clear(conn: &SqliteConn, connection_id: ConnId) -> AppResult<()> {
    conn.execute(
        "DELETE FROM query_history WHERE connection_id=?1",
        params![connection_id],
    )?;
    Ok(())
}
