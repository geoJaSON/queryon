// The ONLY module that talks to the Rust backend. Every Tauri command gets a
// typed wrapper here so the rest of the app never touches `invoke` directly.

import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectionInput,
  ConnectionProfile,
  DbInfo,
  FilterSpec,
  GrantSpec,
  HistoryEntry,
  RoleInfo,
  RoleSpec,
  SavedQuery,
  SchemaObjects,
  ServerInfo,
  SortSpec,
  SqlExecResult,
  TableMeta,
} from "./types";

export const ipc = {
  // --- Connections / secrets ---
  listConnections: () =>
    invoke<ConnectionProfile[]>("list_connections"),

  saveConnection: (profile: ConnectionInput) =>
    invoke<ConnectionProfile>("save_connection", { profile }),

  updateConnection: (id: number, profile: ConnectionInput) =>
    invoke<ConnectionProfile>("update_connection", { id, profile }),

  deleteConnection: (id: number) =>
    invoke<void>("delete_connection", { id }),

  testConnection: (id: number) =>
    invoke<ServerInfo>("test_connection", { id }),

  openConnection: (id: number) =>
    invoke<ServerInfo>("open_connection", { id }),

  closeConnection: (id: number) =>
    invoke<void>("close_connection", { id }),

  // --- SQL execution ---
  runSql: (id: number, sql: string, maxRows?: number) =>
    invoke<SqlExecResult>("run_sql", { id, sql, maxRows: maxRows ?? null }),

  cancelQuery: (id: number) => invoke<void>("cancel_query", { id }),

  // --- Introspection ---
  listDatabases: (id: number) =>
    invoke<DbInfo[]>("list_databases", { id }),

  listSchemas: (id: number) =>
    invoke<string[]>("list_schemas", { id }),

  listObjects: (id: number, schema: string) =>
    invoke<SchemaObjects>("list_objects", { id, schema }),

  describeTable: (id: number, schema: string, table: string) =>
    invoke<TableMeta>("describe_table", { id, schema, table }),

  fetchTablePage: (
    id: number,
    schema: string,
    table: string,
    page: number,
    pageSize: number,
    sort: SortSpec | null,
    filters: FilterSpec[],
    geojson: boolean,
  ) =>
    invoke<SqlExecResult>("fetch_table_page", {
      id,
      schema,
      table,
      page,
      pageSize,
      sort,
      filters,
      geojson,
    }),

  // --- Roles ---
  listRoles: (id: number) => invoke<RoleInfo[]>("list_roles", { id }),

  createRole: (id: number, spec: RoleSpec) =>
    invoke<void>("create_role", { id, spec }),

  alterRole: (id: number, name: string, spec: RoleSpec) =>
    invoke<void>("alter_role", { id, name, spec }),

  dropRole: (id: number, name: string) =>
    invoke<void>("drop_role", { id, name }),

  grantPrivilege: (id: number, spec: GrantSpec) =>
    invoke<void>("grant_privilege", { id, spec }),

  revokePrivilege: (id: number, spec: GrantSpec) =>
    invoke<void>("revoke_privilege", { id, spec }),

  // --- Schema ops ---
  createSchema: (id: number, name: string, owner: string | null) =>
    invoke<void>("create_schema", { id, name, owner }),

  dropSchema: (id: number, name: string, cascade: boolean) =>
    invoke<void>("drop_schema", { id, name, cascade }),

  // --- History & saved queries ---
  listHistory: (
    connectionId: number,
    limit: number,
    offset: number,
    search: string | null,
  ) =>
    invoke<HistoryEntry[]>("list_history", {
      connectionId,
      limit,
      offset,
      search,
    }),

  clearHistory: (connectionId: number) =>
    invoke<void>("clear_history", { connectionId }),

  saveQuery: (connectionId: number, name: string, sql: string) =>
    invoke<SavedQuery>("save_query", { connectionId, name, sql }),

  listSavedQueries: (connectionId: number) =>
    invoke<SavedQuery[]>("list_saved_queries", { connectionId }),

  deleteSavedQuery: (id: number) =>
    invoke<void>("delete_saved_query", { id }),

  // --- Export & geometry ---
  exportResult: (
    data: { columns: string[]; rows: (string | null)[][] },
    format: "csv" | "json",
    path: string,
  ) => invoke<void>("export_result", { data, format, path }),

  decodeGeometry: (id: number, hex: string) =>
    invoke<string>("decode_geometry", { id, hex }),
};
