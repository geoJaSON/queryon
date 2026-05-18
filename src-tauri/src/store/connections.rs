use rusqlite::{params, Connection as SqliteConn};

use crate::error::AppResult;
use crate::models::{ConnId, ConnectionInput, ConnectionProfile, SslMode};
use crate::store::now_iso;

fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<ConnectionProfile> {
    let ssl: String = row.get(6)?;
    Ok(ConnectionProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        host: row.get(2)?,
        port: row.get::<_, i64>(3)? as u16,
        database: row.get(4)?,
        username: row.get(5)?,
        ssl_mode: SslMode::parse(&ssl),
        color: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

const COLS: &str =
    "id, name, host, port, database, username, ssl_mode, color, created_at, updated_at";

pub fn list(conn: &SqliteConn) -> AppResult<Vec<ConnectionProfile>> {
    let mut stmt =
        conn.prepare(&format!("SELECT {COLS} FROM connections ORDER BY name COLLATE NOCASE"))?;
    let rows = stmt.query_map([], row_to_profile)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &SqliteConn, id: ConnId) -> AppResult<ConnectionProfile> {
    Ok(conn.query_row(
        &format!("SELECT {COLS} FROM connections WHERE id = ?1"),
        params![id],
        row_to_profile,
    )?)
}

pub fn insert(conn: &SqliteConn, input: &ConnectionInput) -> AppResult<ConnId> {
    let ts = now_iso();
    conn.execute(
        "INSERT INTO connections
            (name, host, port, database, username, ssl_mode, color, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?8)",
        params![
            input.name,
            input.host,
            input.port as i64,
            input.database,
            input.username,
            input.ssl_mode.as_str(),
            input.color,
            ts,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &SqliteConn, id: ConnId, input: &ConnectionInput) -> AppResult<()> {
    conn.execute(
        "UPDATE connections SET
            name=?2, host=?3, port=?4, database=?5, username=?6,
            ssl_mode=?7, color=?8, updated_at=?9
         WHERE id=?1",
        params![
            id,
            input.name,
            input.host,
            input.port as i64,
            input.database,
            input.username,
            input.ssl_mode.as_str(),
            input.color,
            now_iso(),
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &SqliteConn, id: ConnId) -> AppResult<()> {
    conn.execute("DELETE FROM connections WHERE id=?1", params![id])?;
    Ok(())
}
