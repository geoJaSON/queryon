//! Shared application state managed by Tauri.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use deadpool_postgres::Pool;
use rusqlite::Connection as SqliteConn;

use crate::error::{AppError, AppResult};
use crate::models::ConnId;

/// An open pool plus the per-connection PostGIS type OIDs (resolved at open
/// time because they vary per database).
#[derive(Clone)]
pub struct PoolEntry {
    pub pool: Pool,
    pub geometry_oid: Option<u32>,
    pub geography_oid: Option<u32>,
}

pub struct AppState {
    pools: Mutex<HashMap<ConnId, PoolEntry>>,
    /// In-flight query cancel tokens, keyed by connection id.
    cancels: Mutex<HashMap<ConnId, tokio_postgres::CancelToken>>,
    pub store: Mutex<SqliteConn>,
    #[allow(dead_code)]
    pub config_dir: PathBuf,
}

impl AppState {
    pub fn new(store: SqliteConn, config_dir: PathBuf) -> Self {
        AppState {
            pools: Mutex::new(HashMap::new()),
            cancels: Mutex::new(HashMap::new()),
            store: Mutex::new(store),
            config_dir,
        }
    }

    pub fn set_cancel(&self, id: ConnId, token: tokio_postgres::CancelToken) {
        self.cancels.lock().unwrap().insert(id, token);
    }

    pub fn take_cancel(&self, id: ConnId) -> Option<tokio_postgres::CancelToken> {
        self.cancels.lock().unwrap().remove(&id)
    }

    pub fn set_pool(&self, id: ConnId, entry: PoolEntry) {
        self.pools.lock().unwrap().insert(id, entry);
    }

    pub fn remove_pool(&self, id: ConnId) {
        self.pools.lock().unwrap().remove(&id);
    }

    /// Clone the pool entry out under a short lock so callers can `.await`
    /// without holding the std mutex guard.
    pub fn pool(&self, id: ConnId) -> AppResult<PoolEntry> {
        self.pools
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(AppError::NotConnected)
    }
}
