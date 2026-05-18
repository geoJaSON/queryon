use rusqlite::Connection as SqliteConn;

use crate::error::AppResult;

/// Idempotent schema setup. Bump `SCHEMA_VERSION` and add steps when the
/// shape changes.
const SCHEMA_VERSION: i64 = 1;

pub fn run(conn: &SqliteConn) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);

        CREATE TABLE IF NOT EXISTS connections (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL,
            host       TEXT NOT NULL,
            port       INTEGER NOT NULL DEFAULT 5432,
            database   TEXT NOT NULL,
            username   TEXT NOT NULL,
            ssl_mode   TEXT NOT NULL DEFAULT 'prefer',
            color      TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS saved_queries (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            connection_id INTEGER REFERENCES connections(id) ON DELETE CASCADE,
            name          TEXT NOT NULL,
            sql           TEXT NOT NULL,
            created_at    TEXT NOT NULL,
            UNIQUE(connection_id, name)
        );

        CREATE TABLE IF NOT EXISTS query_history (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            connection_id INTEGER REFERENCES connections(id) ON DELETE CASCADE,
            sql           TEXT NOT NULL,
            success       INTEGER NOT NULL,
            error         TEXT,
            row_count     INTEGER,
            elapsed_ms    INTEGER,
            executed_at   TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_history_conn_time
            ON query_history(connection_id, executed_at DESC);
        "#,
    )?;

    let cur: Option<i64> = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| r.get(0))
        .ok();
    if cur.is_none() {
        conn.execute("INSERT INTO schema_version(version) VALUES (?1)", [SCHEMA_VERSION])?;
    }
    Ok(())
}
