// Hand-kept TypeScript mirror of the Rust DTOs in src-tauri/src/models.rs.
// Keep this file in lockstep with that module.

export type SslMode = "disable" | "prefer" | "require";

export interface ConnectionProfile {
  id: number;
  name: string;
  host: string;
  port: number;
  database: string;
  username: string;
  ssl_mode: SslMode;
  color: string | null;
  created_at: string;
  updated_at: string;
}

export interface ConnectionInput {
  name: string;
  host: string;
  port: number;
  database: string;
  username: string;
  /** Only sent when set/changed; never read back from the keychain. */
  password: string | null;
  ssl_mode: SslMode;
  color: string | null;
}

export interface ServerInfo {
  version: string;
  postgis_version: string | null;
}

/** Internally-tagged: { kind: "text" } | { kind: "geometry", srid: number|null } | ... */
export type RenderHint =
  | { kind: "text" }
  | { kind: "json" }
  | { kind: "bytea" }
  | { kind: "geometry"; srid: number | null };

export interface ColumnMeta {
  name: string;
  type_name: string;
  type_oid: number;
  render_hint: RenderHint;
}

/** A single result set. `rows` is row-major; each cell is the text form or null. */
export interface StatementResult {
  columns: ColumnMeta[];
  rows: (string | null)[][];
  row_count: number;
  command_tag: string;
  elapsed_ms: number;
  truncated: boolean;
  notices: string[];
}

export interface SqlExecResult {
  statements: StatementResult[];
}

export interface AppError {
  kind: string;
  message: string;
  detail: string | null;
}

// ---- Introspection (Phase 2) ----

export interface DbInfo {
  name: string;
  is_current: boolean;
}

export interface SchemaObjects {
  tables: string[];
  views: string[];
  mat_views: string[];
  functions: string[];
}

export interface ColumnInfo {
  name: string;
  data_type: string;
  udt_name: string;
  /** Schema of the underlying type ("pg_catalog" for builtins). */
  udt_schema: string;
  nullable: boolean;
  default: string | null;
  is_pk: boolean;
  is_geometry: boolean;
}

export interface IndexInfo {
  name: string;
  definition: string;
}

export interface TableMeta {
  schema: string;
  name: string;
  columns: ColumnInfo[];
  indexes: IndexInfo[];
  approx_rows: number;
}

export interface SortSpec {
  column: string;
  descending: boolean;
}

export interface FilterSpec {
  column: string;
  op: string;
  value: string | null;
}

// ---- Roles & schema ops (Phase 3) ----

export interface RoleInfo {
  name: string;
  superuser: boolean;
  createdb: boolean;
  createrole: boolean;
  can_login: boolean;
  replication: boolean;
  member_of: string[];
}

export interface RoleSpec {
  name: string;
  login: boolean;
  superuser: boolean;
  createdb: boolean;
  createrole: boolean;
  password: string | null;
}

export interface GrantSpec {
  role: string;
  privileges: string[];
  object_type: "schema" | "table" | "all_tables";
  schema: string;
  object: string | null;
}

// ---- History & saved queries (Phase 4) ----

export interface HistoryEntry {
  id: number;
  sql: string;
  success: boolean;
  error: string | null;
  row_count: number | null;
  elapsed_ms: number | null;
  executed_at: string;
}

export interface SavedQuery {
  id: number;
  connection_id: number;
  name: string;
  sql: string;
  created_at: string;
}
