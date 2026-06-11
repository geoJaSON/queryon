//! DTOs shared across the IPC boundary. Mirrored by hand in src/lib/types.ts.

use serde::{Deserialize, Serialize};

pub type ConnId = i64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl SslMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SslMode::Disable => "disable",
            SslMode::Prefer => "prefer",
            SslMode::Require => "require",
        }
    }
    pub fn parse(s: &str) -> SslMode {
        match s {
            "disable" => SslMode::Disable,
            "require" => SslMode::Require,
            _ => SslMode::Prefer,
        }
    }
}

/// A persisted connection profile (password lives in the OS keychain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: ConnId,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub ssl_mode: SslMode,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Inbound form data. `password: None` on update means "keep existing".
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionInput {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub ssl_mode: SslMode,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub postgis_version: Option<String>,
}

/// Frontend rendering hint per column. Internally tagged: `{ "kind": "text" }`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderHint {
    Text,
    Json,
    Bytea,
    Geometry { srid: Option<i32> },
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnMeta {
    pub name: String,
    pub type_name: String,
    pub type_oid: u32,
    pub render_hint: RenderHint,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementResult {
    pub columns: Vec<ColumnMeta>,
    /// Row-major; each cell is the text form, or `None` for SQL NULL.
    pub rows: Vec<Vec<Option<String>>>,
    pub row_count: u64,
    pub command_tag: String,
    pub elapsed_ms: u64,
    pub truncated: bool,
    pub notices: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SqlExecResult {
    pub statements: Vec<StatementResult>,
}

// ---- Introspection (Phase 2) ----

#[derive(Debug, Clone, Serialize)]
pub struct DbInfo {
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaObjects {
    pub tables: Vec<String>,
    pub views: Vec<String>,
    pub mat_views: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub udt_name: String,
    /// Schema of the underlying type (`pg_catalog` for builtins) — used to
    /// schema-qualify casts in the filter builder.
    pub udt_schema: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_pk: bool,
    pub is_geometry: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableMeta {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub approx_rows: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortSpec {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterSpec {
    pub column: String,
    /// One of: = <> < <= > >= LIKE ILIKE "IS NULL" "IS NOT NULL"
    pub op: String,
    pub value: Option<String>,
}

// ---- Roles & schema ops (Phase 3) ----

#[derive(Debug, Clone, Serialize)]
pub struct RoleInfo {
    pub name: String,
    pub superuser: bool,
    pub createdb: bool,
    pub createrole: bool,
    pub can_login: bool,
    pub replication: bool,
    pub member_of: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleSpec {
    pub name: String,
    pub login: bool,
    pub superuser: bool,
    pub createdb: bool,
    pub createrole: bool,
    /// Set/replace the role password. `None` = leave unchanged.
    pub password: Option<String>,
}

// ---- Export (Phase 5) ----

/// Lightweight, already-stringified payload the frontend hands back for export
/// (avoids deriving Deserialize on the display models).
#[derive(Debug, Clone, Deserialize)]
pub struct ExportData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrantSpec {
    pub role: String,
    /// Validated against an allowlist before use.
    pub privileges: Vec<String>,
    /// "schema" | "table" | "all_tables"
    pub object_type: String,
    pub schema: String,
    /// Required for object_type == "table".
    pub object: Option<String>,
}
