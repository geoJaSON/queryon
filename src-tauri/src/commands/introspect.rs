use tauri::State;

use crate::db::introspect;
use crate::error::AppResult;
use crate::models::{
    ConnId, DbInfo, FilterSpec, SchemaObjects, SortSpec, SqlExecResult, TableMeta,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_databases(
    state: State<'_, AppState>,
    id: ConnId,
) -> AppResult<Vec<DbInfo>> {
    introspect::list_databases(&state.pool(id)?).await
}

#[tauri::command]
pub async fn list_schemas(
    state: State<'_, AppState>,
    id: ConnId,
) -> AppResult<Vec<String>> {
    introspect::list_schemas(&state.pool(id)?).await
}

#[tauri::command]
pub async fn list_objects(
    state: State<'_, AppState>,
    id: ConnId,
    schema: String,
) -> AppResult<SchemaObjects> {
    introspect::list_objects(&state.pool(id)?, &schema).await
}

#[tauri::command]
pub async fn describe_table(
    state: State<'_, AppState>,
    id: ConnId,
    schema: String,
    table: String,
) -> AppResult<TableMeta> {
    introspect::describe_table(&state.pool(id)?, &schema, &table).await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_table_page(
    state: State<'_, AppState>,
    id: ConnId,
    schema: String,
    table: String,
    page: i64,
    page_size: i64,
    sort: Option<SortSpec>,
    filters: Vec<FilterSpec>,
    geojson: bool,
) -> AppResult<SqlExecResult> {
    introspect::fetch_table_page(
        &state.pool(id)?,
        &schema,
        &table,
        page,
        page_size,
        sort,
        filters,
        geojson,
    )
    .await
}
