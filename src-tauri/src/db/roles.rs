//! Role/user management and object privileges. All statements are built by
//! hand (DDL cannot be parameterized); identifiers go through `sql_util`.

use crate::db::sql_util::{ensure_simple_ident, qident, qliteral};
use crate::error::{AppError, AppResult};
use crate::models::{GrantSpec, RoleInfo, RoleSpec};
use crate::state::PoolEntry;

const PRIVILEGES: &[&str] = &[
    "SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE", "REFERENCES", "TRIGGER",
    "USAGE", "CREATE", "CONNECT", "TEMPORARY", "EXECUTE", "ALL",
];

pub async fn list_roles(entry: &PoolEntry) -> AppResult<Vec<RoleInfo>> {
    let client = entry.pool.get().await?;
    let rows = client
        .query(
            "SELECT rolname, rolsuper, rolcreatedb, rolcreaterole,
                    rolcanlogin, rolreplication
             FROM pg_roles ORDER BY rolname",
            &[],
        )
        .await?;

    // Membership in one pass.
    let mem = client
        .query(
            "SELECT u.rolname AS member, g.rolname AS grp
             FROM pg_auth_members m
             JOIN pg_roles g ON g.oid = m.roleid
             JOIN pg_roles u ON u.oid = m.member",
            &[],
        )
        .await?;

    let mut roles: Vec<RoleInfo> = rows
        .iter()
        .map(|r| RoleInfo {
            name: r.get(0),
            superuser: r.get(1),
            createdb: r.get(2),
            createrole: r.get(3),
            can_login: r.get(4),
            replication: r.get(5),
            member_of: vec![],
        })
        .collect();

    for row in &mem {
        let member: String = row.get(0);
        let grp: String = row.get(1);
        if let Some(ri) = roles.iter_mut().find(|x| x.name == member) {
            ri.member_of.push(grp);
        }
    }
    Ok(roles)
}

fn options_clause(spec: &RoleSpec) -> String {
    let mut parts = vec![
        if spec.login { "LOGIN" } else { "NOLOGIN" }.to_string(),
        if spec.superuser { "SUPERUSER" } else { "NOSUPERUSER" }.to_string(),
        if spec.createdb { "CREATEDB" } else { "NOCREATEDB" }.to_string(),
        if spec.createrole {
            "CREATEROLE"
        } else {
            "NOCREATEROLE"
        }
        .to_string(),
    ];
    if let Some(pw) = &spec.password {
        if !pw.is_empty() {
            parts.push(format!("PASSWORD {}", qliteral(pw)));
        }
    }
    parts.join(" ")
}

pub async fn create_role(entry: &PoolEntry, spec: &RoleSpec) -> AppResult<()> {
    ensure_simple_ident(&spec.name)?;
    let sql = format!(
        "CREATE ROLE {} WITH {}",
        qident(&spec.name),
        options_clause(spec)
    );
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}

pub async fn alter_role(entry: &PoolEntry, name: &str, spec: &RoleSpec) -> AppResult<()> {
    ensure_simple_ident(name)?;
    let sql = format!("ALTER ROLE {} WITH {}", qident(name), options_clause(spec));
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}

pub async fn drop_role(entry: &PoolEntry, name: &str) -> AppResult<()> {
    ensure_simple_ident(name)?;
    let sql = format!("DROP ROLE {}", qident(name));
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}

fn build_grant(g: &GrantSpec, revoke: bool) -> AppResult<String> {
    if g.privileges.is_empty() {
        return Err(AppError::msg("no privileges specified"));
    }
    for p in &g.privileges {
        if !PRIVILEGES.contains(&p.to_uppercase().as_str()) {
            return Err(AppError::msg(format!("unsupported privilege: {p}")));
        }
    }
    ensure_simple_ident(&g.role)?;
    ensure_simple_ident(&g.schema)?;
    let privs = g
        .privileges
        .iter()
        .map(|p| p.to_uppercase())
        .collect::<Vec<_>>()
        .join(", ");

    let target = match g.object_type.as_str() {
        "schema" => format!("SCHEMA {}", qident(&g.schema)),
        "all_tables" => format!("ALL TABLES IN SCHEMA {}", qident(&g.schema)),
        "table" => {
            let obj = g
                .object
                .as_deref()
                .ok_or_else(|| AppError::msg("table name required"))?;
            ensure_simple_ident(obj)?;
            format!("TABLE {}.{}", qident(&g.schema), qident(obj))
        }
        other => return Err(AppError::msg(format!("bad object type: {other}"))),
    };

    Ok(if revoke {
        format!("REVOKE {privs} ON {target} FROM {}", qident(&g.role))
    } else {
        format!("GRANT {privs} ON {target} TO {}", qident(&g.role))
    })
}

pub async fn grant_privilege(entry: &PoolEntry, g: &GrantSpec) -> AppResult<()> {
    let sql = build_grant(g, false)?;
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}

pub async fn revoke_privilege(entry: &PoolEntry, g: &GrantSpec) -> AppResult<()> {
    let sql = build_grant(g, true)?;
    entry.pool.get().await?.batch_execute(&sql).await?;
    Ok(())
}
