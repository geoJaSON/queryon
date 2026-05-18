// Open workspace tabs. Phase 1: SQL editor tabs. Table-view tabs land in Phase 2.

import type { SqlExecResult } from "../types";

export type TabKind = "sql" | "table" | "roles" | "history";

export interface Tab {
  id: number;
  kind: TabKind;
  title: string;
  /** SQL editor buffer (kind === "sql"). */
  sql: string;
  /** Last execution result rendered in this tab. */
  result: SqlExecResult | null;
  error: string | null;
  running: boolean;
  // Reserved for Phase 2 table-view tabs:
  schema?: string;
  table?: string;
}

let seq = 0;

export const tabsState = $state<{ tabs: Tab[]; activeId: number | null }>({
  tabs: [],
  activeId: null,
});

export function activeTab(): Tab | null {
  return tabsState.tabs.find((t) => t.id === tabsState.activeId) ?? null;
}

export function newSqlTab(sql = ""): Tab {
  const id = ++seq;
  const tab: Tab = {
    id,
    kind: "sql",
    title: `query ${id}`,
    sql,
    result: null,
    error: null,
    running: false,
  };
  tabsState.tabs.push(tab);
  tabsState.activeId = id;
  return tab;
}

export function newTableTab(schema: string, table: string): Tab {
  // Re-use an existing tab for the same table if open.
  const existing = tabsState.tabs.find(
    (t) => t.kind === "table" && t.schema === schema && t.table === table,
  );
  if (existing) {
    tabsState.activeId = existing.id;
    return existing;
  }
  const id = ++seq;
  const tab: Tab = {
    id,
    kind: "table",
    title: `${schema}.${table}`,
    sql: "",
    result: null,
    error: null,
    running: false,
    schema,
    table,
  };
  tabsState.tabs.push(tab);
  tabsState.activeId = id;
  return tab;
}

export function newRolesTab(): Tab {
  const existing = tabsState.tabs.find((t) => t.kind === "roles");
  if (existing) {
    tabsState.activeId = existing.id;
    return existing;
  }
  const id = ++seq;
  const tab: Tab = {
    id,
    kind: "roles",
    title: "roles",
    sql: "",
    result: null,
    error: null,
    running: false,
  };
  tabsState.tabs.push(tab);
  tabsState.activeId = id;
  return tab;
}

export function newHistoryTab(): Tab {
  const existing = tabsState.tabs.find((t) => t.kind === "history");
  if (existing) {
    tabsState.activeId = existing.id;
    return existing;
  }
  const id = ++seq;
  const tab: Tab = {
    id,
    kind: "history",
    title: "history",
    sql: "",
    result: null,
    error: null,
    running: false,
  };
  tabsState.tabs.push(tab);
  tabsState.activeId = id;
  return tab;
}

export function closeTab(id: number) {
  const i = tabsState.tabs.findIndex((t) => t.id === id);
  if (i < 0) return;
  tabsState.tabs.splice(i, 1);
  if (tabsState.activeId === id) {
    const next = tabsState.tabs[i] ?? tabsState.tabs[i - 1] ?? null;
    tabsState.activeId = next ? next.id : null;
  }
}

export function selectTab(id: number) {
  tabsState.activeId = id;
}
