//! Structured, serializable error returned to the frontend as
//! `{ kind, message, detail }`.

use serde::ser::SerializeStruct;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Msg(String),

    #[error("not connected")]
    NotConnected,

    #[error("database error: {0}")]
    Db(#[from] tokio_postgres::Error),

    #[error("connection pool error: {0}")]
    Pool(String),

    #[error("local store error: {0}")]
    Store(#[from] rusqlite::Error),

    #[error("keychain error: {0}")]
    Keyring(#[from] keyring::Error),
}

impl AppError {
    pub fn msg(s: impl Into<String>) -> Self {
        AppError::Msg(s.into())
    }

    fn kind(&self) -> &'static str {
        match self {
            AppError::Msg(_) => "error",
            AppError::NotConnected => "not_connected",
            AppError::Db(_) => "db",
            AppError::Pool(_) => "pool",
            AppError::Store(_) => "store",
            AppError::Keyring(_) => "keychain",
        }
    }

    /// Extra context — for DB errors, the Postgres SQLSTATE + detail/hint.
    fn detail(&self) -> Option<String> {
        match self {
            AppError::Db(e) => e.as_db_error().map(|d| {
                let mut s = format!("SQLSTATE {}", d.code().code());
                if let Some(p) = d.detail() {
                    s.push_str(&format!("\n{p}"));
                }
                if let Some(h) = d.hint() {
                    s.push_str(&format!("\nHINT: {h}"));
                }
                s
            }),
            _ => None,
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut st = s.serialize_struct("AppError", 3)?;
        st.serialize_field("kind", self.kind())?;
        st.serialize_field("message", &self.to_string())?;
        st.serialize_field("detail", &self.detail())?;
        st.end()
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        AppError::Pool(e.to_string())
    }
}

impl From<deadpool_postgres::CreatePoolError> for AppError {
    fn from(e: deadpool_postgres::CreatePoolError) -> Self {
        AppError::Pool(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
