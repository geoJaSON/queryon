//! Create / drop schemas.

use crate::db::sql_util::{ensure_simple_ident, qident};
use crate::error::AppResult;
use crate::state::PoolEntry;

pub async fn create_schema(
    entry: &PoolEntry,
    name: &str,
    owner: Option<&str>,
) -> AppResult<()> {
    ensure_simple_ident(name)?;
    let mut sql = format!("CREATE SCHEMA {}", qident(name));
    if let Some(o) = owner {
        if !o.is_empty() {
            ensure_simple_ident(o)?;
            sql.push_str(&format!(" AUTHORIZATION {}", qident(o)));
        }
    }
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}

pub async fn drop_schema(entry: &PoolEntry, name: &str, cascade: bool) -> AppResult<()> {
    ensure_simple_ident(name)?;
    let sql = format!(
        "DROP SCHEMA {}{}",
        qident(name),
        if cascade { " CASCADE" } else { "" }
    );
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}
