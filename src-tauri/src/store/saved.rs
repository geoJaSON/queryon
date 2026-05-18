#![allow(dead_code)] // wired in Phase 4

use rusqlite::{params, Connection as SqliteConn};
use serde::Serialize;

use crate::error::AppResult;
use crate::models::ConnId;
use crate::store::now_iso;

#[derive(Debug, Serialize)]
pub struct SavedQuery {
    pub id: i64,
    pub connection_id: ConnId,
    pub name: String,
    pub sql: String,
    pub created_at: String,
}

pub fn save(
    conn: &SqliteConn,
    connection_id: ConnId,
    name: &str,
    sql: &str,
) -> AppResult<SavedQuery> {
    conn.execute(
        "INSERT INTO saved_queries (connection_id, name, sql, created_at)
         VALUES (?1,?2,?3,?4)
         ON CONFLICT(connection_id, name) DO UPDATE SET sql=excluded.sql",
        params![connection_id, name, sql, now_iso()],
    )?;
    let id = conn.query_row(
        "SELECT id FROM saved_queries WHERE connection_id=?1 AND name=?2",
        params![connection_id, name],
        |r| r.get(0),
    )?;
    Ok(SavedQuery {
        id,
        connection_id,
        name: name.to_string(),
        sql: sql.to_string(),
        created_at: now_iso(),
    })
}

pub fn list(conn: &SqliteConn, connection_id: ConnId) -> AppResult<Vec<SavedQuery>> {
    let mut stmt = conn.prepare(
        "SELECT id, connection_id, name, sql, created_at
         FROM saved_queries WHERE connection_id=?1 ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![connection_id], |r| {
        Ok(SavedQuery {
            id: r.get(0)?,
            connection_id: r.get(1)?,
            name: r.get(2)?,
            sql: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn delete(conn: &SqliteConn, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM saved_queries WHERE id=?1", params![id])?;
    Ok(())
}
