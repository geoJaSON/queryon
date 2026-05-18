//! Connection passwords, stored only in the OS keychain (Windows Credential
//! Manager). Never persisted to SQLite, never returned to the frontend.

use crate::error::AppResult;
use crate::models::ConnId;

const SERVICE: &str = "com.queryon.app";

fn entry(id: ConnId) -> AppResult<keyring::Entry> {
    Ok(keyring::Entry::new(SERVICE, &format!("conn:{id}"))?)
}

pub fn set_password(id: ConnId, password: &str) -> AppResult<()> {
    entry(id)?.set_password(password)?;
    Ok(())
}

pub fn get_password(id: ConnId) -> AppResult<String> {
    match entry(id)?.get_password() {
        Ok(p) => Ok(p),
        // No stored secret → treat as empty password rather than an error.
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_password(id: ConnId) -> AppResult<()> {
    match entry(id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
