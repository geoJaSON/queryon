use tauri::State;

use crate::db::schema_ops;
use crate::error::AppResult;
use crate::models::ConnId;
use crate::state::AppState;

#[tauri::command]
pub async fn create_schema(
    state: State<'_, AppState>,
    id: ConnId,
    name: String,
    owner: Option<String>,
) -> AppResult<()> {
    schema_ops::create_schema(&state.pool(id)?, &name, owner.as_deref()).await
}

#[tauri::command]
pub async fn drop_schema(
    state: State<'_, AppState>,
    id: ConnId,
    name: String,
    cascade: bool,
) -> AppResult<()> {
    schema_ops::drop_schema(&state.pool(id)?, &name, cascade).await
}
