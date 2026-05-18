//! Local SQLite store: connection profiles, query history, saved queries.

pub mod connections;
pub mod history;
pub mod migrations;
pub mod saved;

use std::path::Path;

use rusqlite::Connection as SqliteConn;

use crate::error::AppResult;

pub fn open(dir: &Path) -> AppResult<SqliteConn> {
    std::fs::create_dir_all(dir).ok();
    let conn = SqliteConn::open(dir.join("queryon.db"))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrations::run(&conn)?;
    Ok(conn)
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
