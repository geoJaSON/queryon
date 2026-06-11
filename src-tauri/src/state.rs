//! Shared application state managed by Tauri.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use deadpool_postgres::Pool;
use rusqlite::Connection as SqliteConn;

use crate::db::pool::NoticeSink;
use crate::error::{AppError, AppResult};
use crate::models::ConnId;

/// An open pool plus the per-connection PostGIS type OIDs (resolved at open
/// time because they vary per database).
#[derive(Clone)]
pub struct PoolEntry {
    pub pool: Pool,
    pub geometry_oid: Option<u32>,
    pub geography_oid: Option<u32>,
    /// Server NOTICE/WARNING messages collected by the connection tasks.
    pub notices: NoticeSink,
}

pub struct AppState {
    pools: Mutex<HashMap<ConnId, PoolEntry>>,
    /// In-flight query cancel tokens, keyed by connection id, then by a
    /// per-execution id so concurrent queries on one connection don't
    /// clobber each other's token.
    cancels: Mutex<HashMap<ConnId, HashMap<u64, tokio_postgres::CancelToken>>>,
    cancel_seq: AtomicU64,
    pub store: Mutex<SqliteConn>,
    #[allow(dead_code)]
    pub config_dir: PathBuf,
}

impl AppState {
    pub fn new(store: SqliteConn, config_dir: PathBuf) -> Self {
        AppState {
            pools: Mutex::new(HashMap::new()),
            cancels: Mutex::new(HashMap::new()),
            cancel_seq: AtomicU64::new(0),
            store: Mutex::new(store),
            config_dir,
        }
    }

    /// Register an in-flight query; returns the execution id to pass back to
    /// `remove_cancel` when the query finishes.
    pub fn set_cancel(&self, id: ConnId, token: tokio_postgres::CancelToken) -> u64 {
        let qid = self.cancel_seq.fetch_add(1, Ordering::Relaxed);
        self.cancels
            .lock()
            .unwrap()
            .entry(id)
            .or_default()
            .insert(qid, token);
        qid
    }

    pub fn remove_cancel(&self, id: ConnId, qid: u64) {
        let mut map = self.cancels.lock().unwrap();
        if let Some(m) = map.get_mut(&id) {
            m.remove(&qid);
            if m.is_empty() {
                map.remove(&id);
            }
        }
    }

    /// Take every in-flight cancel token for a connection.
    pub fn take_cancels(&self, id: ConnId) -> Vec<tokio_postgres::CancelToken> {
        self.cancels
            .lock()
            .unwrap()
            .remove(&id)
            .map(|m| m.into_values().collect())
            .unwrap_or_default()
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
