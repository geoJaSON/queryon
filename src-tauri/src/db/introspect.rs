//! Schema introspection + a safe, server-generated table-page query
//! (geometry columns projected as EWKT).

use tokio_postgres::types::{ToSql, Type};

use crate::db::sql_util::qident;
use crate::db::value;
use crate::error::{AppError, AppResult};
use crate::models::{
    ColumnInfo, DbInfo, FilterSpec, IndexInfo, SchemaObjects, SortSpec, SqlExecResult,
    StatementResult, TableMeta,
};
use crate::state::PoolEntry;

pub async fn list_databases(entry: &PoolEntry) -> AppResult<Vec<DbInfo>> {
    let client = entry.pool.get().await?;
    let rows = client
        .query(
            "SELECT datname, datname = current_database()
             FROM pg_database
             WHERE datistemplate = false
             ORDER BY datname",
            &[],
        )
        .await?;
    Ok(rows
        .iter()
        .map(|r| DbInfo {
            name: r.get(0),
            is_current: r.get(1),
        })
        .collect())
}

pub async fn list_schemas(entry: &PoolEntry) -> AppResult<Vec<String>> {
    let client = entry.pool.get().await?;
    let rows = client
        .query(
            "SELECT nspname FROM pg_namespace
             WHERE nspname NOT LIKE 'pg_%' AND nspname <> 'information_schema'
             ORDER BY nspname",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|r| r.get(0)).collect())
}

pub async fn list_objects(entry: &PoolEntry, schema: &str) -> AppResult<SchemaObjects> {
    let client = entry.pool.get().await?;
    let rows = client
        .query(
            "SELECT c.relname, c.relkind
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1 AND c.relkind IN ('r','p','v','m')
             ORDER BY c.relname",
            &[&schema],
        )
        .await?;

    let mut o = SchemaObjects {
        tables: vec![],
        views: vec![],
        mat_views: vec![],
        functions: vec![],
    };
    for r in &rows {
        let name: String = r.get(0);
        let kind: i8 = r.get::<_, i8>(1);
        match kind as u8 as char {
            'r' | 'p' => o.tables.push(name),
            'v' => o.views.push(name),
            'm' => o.mat_views.push(name),
            _ => {}
        }
    }

    let frows = client
        .query(
            "SELECT p.proname FROM pg_proc p
             JOIN pg_namespace n ON n.oid = p.pronamespace
             WHERE n.nspname = $1 ORDER BY p.proname",
            &[&schema],
        )
        .await?;
    o.functions = frows.iter().map(|r| r.get(0)).collect();
    Ok(o)
}

pub async fn describe_table(
    entry: &PoolEntry,
    schema: &str,
    table: &str,
) -> AppResult<TableMeta> {
    let client = entry.pool.get().await?;

    let col_rows = client
        .query(
            "SELECT column_name, data_type, udt_name, udt_schema, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_schema = $1 AND table_name = $2
             ORDER BY ordinal_position",
            &[&schema, &table],
        )
        .await?;

    let pk_rows = client
        .query(
            "SELECT a.attname
             FROM pg_index i
             JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
             WHERE i.indrelid = to_regclass(quote_ident($1)||'.'||quote_ident($2))
               AND i.indisprimary",
            &[&schema, &table],
        )
        .await?;
    let pks: Vec<String> = pk_rows.iter().map(|r| r.get(0)).collect();

    let columns: Vec<ColumnInfo> = col_rows
        .iter()
        .map(|r| {
            let name: String = r.get(0);
            let udt: String = r.get(2);
            let nullable: String = r.get(4);
            ColumnInfo {
                is_pk: pks.contains(&name),
                is_geometry: udt == "geometry" || udt == "geography",
                name,
                data_type: r.get(1),
                udt_name: udt,
                udt_schema: r.get(3),
                nullable: nullable == "YES",
                default: r.get(5),
            }
        })
        .collect();

    let idx_rows = client
        .query(
            "SELECT indexname, indexdef FROM pg_indexes
             WHERE schemaname = $1 AND tablename = $2 ORDER BY indexname",
            &[&schema, &table],
        )
        .await?;
    let indexes = idx_rows
        .iter()
        .map(|r| IndexInfo {
            name: r.get(0),
            definition: r.get(1),
        })
        .collect();

    let approx_rows: i64 = client
        .query_opt(
            "SELECT c.reltuples::bigint
             FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1 AND c.relname = $2",
            &[&schema, &table],
        )
        .await?
        .map(|r| r.get(0))
        .unwrap_or(0);

    Ok(TableMeta {
        schema: schema.to_string(),
        name: table.to_string(),
        columns,
        indexes,
        approx_rows,
    })
}

const ALLOWED_OPS: &[&str] = &[
    "=", "<>", "<", "<=", ">", ">=", "LIKE", "ILIKE", "IS NULL", "IS NOT NULL",
];

#[allow(clippy::too_many_arguments)]
pub async fn fetch_table_page(
    entry: &PoolEntry,
    schema: &str,
    table: &str,
    page: i64,
    page_size: i64,
    sort: Option<SortSpec>,
    filters: Vec<FilterSpec>,
    geojson: bool,
) -> AppResult<SqlExecResult> {
    let meta = describe_table(entry, schema, table).await?;
    if meta.columns.is_empty() {
        return Err(AppError::msg(format!(
            "{schema}.{table} has no columns or is not visible"
        )));
    }
    let by_name: std::collections::HashMap<&str, &ColumnInfo> =
        meta.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    // Projection — geometry/geography wrapped as EWKT (or GeoJSON).
    let proj = meta
        .columns
        .iter()
        .map(|c| {
            if c.is_geometry {
                let f = if geojson { "ST_AsGeoJSON" } else { "ST_AsEWKT" };
                format!("{f}({}::geometry) AS {}", qident(&c.name), qident(&c.name))
            } else {
                qident(&c.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut sql = format!(
        "SELECT {proj} FROM {}.{}",
        qident(schema),
        qident(table)
    );

    // Parameterized WHERE. All parameters are bound as TEXT (see
    // `prepare_typed` below); comparison values are cast server-side to the
    // column's own type so filters work on int/uuid/date/... columns, while
    // LIKE/ILIKE compare the column as text.
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();
    let mut clauses: Vec<String> = Vec::new();
    for f in &filters {
        let Some(col_info) = by_name.get(f.column.as_str()) else {
            return Err(AppError::msg(format!("unknown filter column: {}", f.column)));
        };
        if !ALLOWED_OPS.contains(&f.op.as_str()) {
            return Err(AppError::msg(format!("unsupported operator: {}", f.op)));
        }
        let col = qident(&f.column);
        if f.op == "IS NULL" || f.op == "IS NOT NULL" {
            clauses.push(format!("{col} {}", f.op));
        } else if f.op == "LIKE" || f.op == "ILIKE" {
            params.push(Box::new(f.value.clone().unwrap_or_default()));
            clauses.push(format!("{col}::text {} ${}", f.op, params.len()));
        } else {
            params.push(Box::new(f.value.clone().unwrap_or_default()));
            clauses.push(format!(
                "{col} {} ${}::{}.{}",
                f.op,
                params.len(),
                qident(&col_info.udt_schema),
                qident(&col_info.udt_name)
            ));
        }
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }

    if let Some(s) = &sort {
        if !by_name.contains_key(s.column.as_str()) {
            return Err(AppError::msg(format!("unknown sort column: {}", s.column)));
        }
        sql.push_str(&format!(
            " ORDER BY {} {}",
            qident(&s.column),
            if s.descending { "DESC" } else { "ASC" }
        ));
    }

    let size = page_size.clamp(1, 5000);
    let offset = page.max(0) * size;
    sql.push_str(&format!(" LIMIT {size} OFFSET {offset}"));

    let client = entry.pool.get().await?;
    let started = std::time::Instant::now();
    let param_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|b| b.as_ref() as &(dyn ToSql + Sync))
        .collect();
    // Declare every parameter as TEXT so Rust `String` values always encode;
    // the SQL casts them to the column type server-side.
    let stmt = client
        .prepare_typed(&sql, &vec![Type::TEXT; params.len()])
        .await?;
    let rows = client.query(&stmt, &param_refs).await?;

    let columns: Vec<_> = if let Some(r) = rows.first() {
        r.columns()
            .iter()
            .map(|c| {
                value::column_meta(
                    c.name(),
                    c.type_(),
                    entry.geometry_oid,
                    entry.geography_oid,
                )
            })
            .collect()
    } else {
        // No rows: synthesize column meta from the table description.
        meta.columns
            .iter()
            .map(|c| crate::models::ColumnMeta {
                name: c.name.clone(),
                type_name: c.udt_name.clone(),
                type_oid: 0,
                render_hint: if c.is_geometry {
                    crate::models::RenderHint::Geometry { srid: None }
                } else {
                    crate::models::RenderHint::Text
                },
            })
            .collect()
    };

    let data: Vec<Vec<Option<String>>> = rows
        .iter()
        .map(|r| {
            r.columns()
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    value::cell(r, i, c.type_(), entry.geometry_oid, entry.geography_oid)
                })
                .collect()
        })
        .collect();

    let rc = data.len() as u64;
    Ok(SqlExecResult {
        statements: vec![StatementResult {
            columns,
            rows: data,
            row_count: rc,
            command_tag: format!("SELECT {rc}"),
            elapsed_ms: started.elapsed().as_millis() as u64,
            truncated: false,
            notices: vec![],
        }],
    })
}
