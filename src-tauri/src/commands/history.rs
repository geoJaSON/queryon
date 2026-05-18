use tauri::State;

use crate::error::AppResult;
use crate::models::ConnId;
use crate::state::AppState;
use crate::store::history::{self, HistoryEntry};
use crate::store::saved::{self, SavedQuery};

#[tauri::command]
pub async fn list_history(
    state: State<'_, AppState>,
    connection_id: ConnId,
    limit: i64,
    offset: i64,
    search: Option<String>,
) -> AppResult<Vec<HistoryEntry>> {
    let conn = state.store.lock().unwrap();
    history::list(&conn, connection_id, limit, offset, search.as_deref())
}

#[tauri::command]
pub async fn clear_history(
    state: State<'_, AppState>,
    connection_id: ConnId,
) -> AppResult<()> {
    let conn = state.store.lock().unwrap();
    history::clear(&conn, connection_id)
}

#[tauri::command]
pub async fn save_query(
    state: State<'_, AppState>,
    connection_id: ConnId,
    name: String,
    sql: String,
) -> AppResult<SavedQuery> {
    let conn = state.store.lock().unwrap();
    saved::save(&conn, connection_id, &name, &sql)
}

#[tauri::command]
pub async fn list_saved_queries(
    state: State<'_, AppState>,
    connection_id: ConnId,
) -> AppResult<Vec<SavedQuery>> {
    let conn = state.store.lock().unwrap();
    saved::list(&conn, connection_id)
}

#[tauri::command]
pub async fn delete_saved_query(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.store.lock().unwrap();
    saved::delete(&conn, id)
}
