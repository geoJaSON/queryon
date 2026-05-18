use tauri::State;

use crate::db::query::{self, DEFAULT_MAX_ROWS};
use crate::error::AppResult;
use crate::models::{ConnId, SqlExecResult};
use crate::state::AppState;
use crate::store::history;

#[tauri::command]
pub async fn run_sql(
    state: State<'_, AppState>,
    id: ConnId,
    sql: String,
    max_rows: Option<usize>,
) -> AppResult<SqlExecResult> {
    let entry = state.pool(id)?;
    let cap = max_rows.unwrap_or(DEFAULT_MAX_ROWS);

    // Hold a dedicated client so its cancel token stays valid for the run.
    let client = entry.pool.get().await?;
    state.set_cancel(id, client.cancel_token());

    let result = query::run_sql(
        &client,
        entry.geometry_oid,
        entry.geography_oid,
        &sql,
        cap,
    )
    .await;

    state.take_cancel(id);
    drop(client);

    // Auto-record into history (best-effort; never fails the query).
    {
        let conn = state.store.lock().unwrap();
        match &result {
            Ok(r) => {
                let rows: u64 = r.statements.iter().map(|s| s.row_count).sum();
                let ms: i64 = r.statements.iter().map(|s| s.elapsed_ms as i64).sum();
                let _ = history::record(
                    &conn,
                    id,
                    &sql,
                    true,
                    None,
                    Some(rows as i64),
                    Some(ms),
                );
            }
            Err(e) => {
                let _ = history::record(
                    &conn,
                    id,
                    &sql,
                    false,
                    Some(&e.to_string()),
                    None,
                    None,
                );
            }
        }
    }

    result
}

/// Best-effort cancel of the in-flight query on `id`. The cancel request opens
/// a fresh non-TLS connection to the server (libpq semantics); it succeeds for
/// `sslmode=disable` servers, which is the common local case.
#[tauri::command]
pub async fn cancel_query(state: State<'_, AppState>, id: ConnId) -> AppResult<()> {
    if let Some(token) = state.take_cancel(id) {
        let _ = token.cancel_query(tokio_postgres::NoTls).await;
    }
    Ok(())
}
