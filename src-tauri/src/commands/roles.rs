use tauri::State;

use crate::db::roles;
use crate::error::AppResult;
use crate::models::{ConnId, GrantSpec, RoleInfo, RoleSpec};
use crate::state::AppState;

#[tauri::command]
pub async fn list_roles(state: State<'_, AppState>, id: ConnId) -> AppResult<Vec<RoleInfo>> {
    roles::list_roles(&state.pool(id)?).await
}

#[tauri::command]
pub async fn create_role(
    state: State<'_, AppState>,
    id: ConnId,
    spec: RoleSpec,
) -> AppResult<()> {
    roles::create_role(&state.pool(id)?, &spec).await
}

#[tauri::command]
pub async fn alter_role(
    state: State<'_, AppState>,
    id: ConnId,
    name: String,
    spec: RoleSpec,
) -> AppResult<()> {
    roles::alter_role(&state.pool(id)?, &name, &spec).await
}

#[tauri::command]
pub async fn drop_role(
    state: State<'_, AppState>,
    id: ConnId,
    name: String,
) -> AppResult<()> {
    roles::drop_role(&state.pool(id)?, &name).await
}

#[tauri::command]
pub async fn grant_privilege(
    state: State<'_, AppState>,
    id: ConnId,
    spec: GrantSpec,
) -> AppResult<()> {
    roles::grant_privilege(&state.pool(id)?, &spec).await
}

#[tauri::command]
pub async fn revoke_privilege(
    state: State<'_, AppState>,
    id: ConnId,
    spec: GrantSpec,
) -> AppResult<()> {
    roles::revoke_privilege(&state.pool(id)?, &spec).await
}
