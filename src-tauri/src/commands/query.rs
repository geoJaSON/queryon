use tauri::State;

use crate::db::query::{self, DEFAULT_MAX_ROWS};
use crate::db::pool;
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
    // Discard notices left over from unrelated activity on this pool.
    entry.notices.lock().unwrap().clear();
    let qid = state.set_cancel(id, client.cancel_token());

    let result = query::run_sql(
        &client,
        entry.geometry_oid,
        entry.geography_oid,
        &sql,
        cap,
        &entry.notices,
    )
    .await;

    state.remove_cancel(id, qid);

    if result.is_err() {
        // A failed statement may have left this client inside an aborted
        // transaction; rolling back before the client returns to the pool
        // keeps the next checkout from hitting "current transaction is
        // aborted". Outside a transaction this is a harmless no-op warning.
        let _ = client.batch_execute("ROLLBACK").await;
        entry.notices.lock().unwrap().clear();
    }
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

/// Best-effort cancel of every in-flight query on `id`. The cancel request
/// opens a fresh connection negotiated with the same TLS connector as the
/// pool, so it works for all configured ssl modes.
#[tauri::command]
pub async fn cancel_query(state: State<'_, AppState>, id: ConnId) -> AppResult<()> {
    for token in state.take_cancels(id) {
        let _ = token.cancel_query(pool::tls_connector()).await;
    }
    Ok(())
}
