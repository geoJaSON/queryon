use tauri::State;

use crate::db::{connect, pool};
use crate::error::AppResult;
use crate::models::{ConnId, ConnectionInput, ConnectionProfile, ServerInfo};
use crate::state::AppState;
use crate::secrets;
use crate::store::connections as cstore;

#[tauri::command]
pub async fn list_connections(
    state: State<'_, AppState>,
) -> AppResult<Vec<ConnectionProfile>> {
    let conn = state.store.lock().unwrap();
    cstore::list(&conn)
}

#[tauri::command]
pub async fn save_connection(
    state: State<'_, AppState>,
    profile: ConnectionInput,
) -> AppResult<ConnectionProfile> {
    let saved = {
        let conn = state.store.lock().unwrap();
        let id = cstore::insert(&conn, &profile)?;
        cstore::get(&conn, id)?
    };
    if let Some(pw) = &profile.password {
        secrets::set_password(saved.id, pw)?;
    }
    Ok(saved)
}

#[tauri::command]
pub async fn update_connection(
    state: State<'_, AppState>,
    id: ConnId,
    profile: ConnectionInput,
) -> AppResult<ConnectionProfile> {
    let updated = {
        let conn = state.store.lock().unwrap();
        cstore::update(&conn, id, &profile)?;
        cstore::get(&conn, id)?
    };
    // Empty/None password on edit means "keep existing".
    if let Some(pw) = &profile.password {
        if !pw.is_empty() {
            secrets::set_password(id, pw)?;
        }
    }
    // Force a fresh pool on next open so edits take effect.
    state.remove_pool(id);
    Ok(updated)
}

#[tauri::command]
pub async fn delete_connection(state: State<'_, AppState>, id: ConnId) -> AppResult<()> {
    {
        let conn = state.store.lock().unwrap();
        cstore::delete(&conn, id)?;
    }
    state.remove_pool(id);
    secrets::delete_password(id)?;
    Ok(())
}

/// Build a throwaway pool from raw form input and verify it works — nothing
/// is persisted. `id` is the profile being edited (if any) so a blank
/// password can fall back to the stored keychain secret.
#[tauri::command]
pub async fn test_connection(
    profile: ConnectionInput,
    id: Option<ConnId>,
) -> AppResult<ServerInfo> {
    let password = match &profile.password {
        Some(pw) if !pw.is_empty() => pw.clone(),
        _ => match id {
            Some(i) => secrets::get_password(i)?,
            None => String::new(),
        },
    };
    let temp = ConnectionProfile {
        id: id.unwrap_or(0),
        name: profile.name,
        host: profile.host,
        port: profile.port,
        database: profile.database,
        username: profile.username,
        ssl_mode: profile.ssl_mode,
        color: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let (p, sink) = pool::build_pool(&temp, &password)?;
    let (info, _entry) = connect::probe(p, sink).await?;
    Ok(info)
}

#[tauri::command]
pub async fn open_connection(
    state: State<'_, AppState>,
    id: ConnId,
) -> AppResult<ServerInfo> {
    let profile = {
        let conn = state.store.lock().unwrap();
        cstore::get(&conn, id)?
    };
    let password = secrets::get_password(id)?;
    let (p, sink) = pool::build_pool(&profile, &password)?;
    let (info, entry) = connect::probe(p, sink).await?;
    state.set_pool(id, entry);
    Ok(info)
}

#[tauri::command]
pub async fn close_connection(state: State<'_, AppState>, id: ConnId) -> AppResult<()> {
    state.remove_pool(id);
    Ok(())
}
