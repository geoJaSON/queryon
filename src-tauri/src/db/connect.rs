//! Opening/testing a connection: build a pool, verify it works, and resolve
//! the per-database PostGIS type OIDs.

use deadpool_postgres::Pool;

use crate::db::pool::NoticeSink;
use crate::error::AppResult;
use crate::models::ServerInfo;
use crate::state::PoolEntry;

/// Probe a freshly built pool: server version, PostGIS version, and the
/// geometry/geography type OIDs (which vary per database).
pub async fn probe(pool: Pool, notices: NoticeSink) -> AppResult<(ServerInfo, PoolEntry)> {
    let client = pool.get().await?;

    let version: String = client.query_one("SELECT version()", &[]).await?.get(0);

    let postgis_version: Option<String> = client
        .query_opt(
            "SELECT extversion FROM pg_extension WHERE extname = 'postgis'",
            &[],
        )
        .await?
        .map(|r| r.get(0));

    let geometry_oid: Option<u32> = client
        .query_opt("SELECT oid FROM pg_type WHERE typname = 'geometry'", &[])
        .await?
        .map(|r| r.get::<_, u32>(0));

    let geography_oid: Option<u32> = client
        .query_opt("SELECT oid FROM pg_type WHERE typname = 'geography'", &[])
        .await?
        .map(|r| r.get::<_, u32>(0));

    drop(client);

    let info = ServerInfo {
        version: short_version(&version),
        postgis_version,
    };
    let entry = PoolEntry {
        pool,
        geometry_oid,
        geography_oid,
        notices,
    };
    Ok((info, entry))
}

/// "PostgreSQL 16.2 on x86_64..." -> "PostgreSQL 16.2"
fn short_version(v: &str) -> String {
    v.split(" on ").next().unwrap_or(v).trim().to_string()
}
