// Saved connection profiles + which one is currently open/active.

import { ipc } from "../ipc";
import type { ConnectionProfile, ServerInfo } from "../types";
import { reportError, pushToast } from "./toast.svelte";

interface ConnState {
  profiles: ConnectionProfile[];
  /** id of the connection whose pool is open and active, or null. */
  activeId: number | null;
  serverInfo: ServerInfo | null;
  busy: boolean;
}

export const conns = $state<ConnState>({
  profiles: [],
  activeId: null,
  serverInfo: null,
  busy: false,
});

export function activeProfile(): ConnectionProfile | null {
  return conns.profiles.find((p) => p.id === conns.activeId) ?? null;
}

export async function refreshConnections() {
  try {
    conns.profiles = await ipc.listConnections();
  } catch (e) {
    reportError(e, "Could not load connections");
  }
}

export async function openConnection(id: number) {
  conns.busy = true;
  try {
    const info = await ipc.openConnection(id);
    conns.activeId = id;
    conns.serverInfo = info;
    pushToast("ok", "Connected");
  } catch (e) {
    reportError(e, "Connection failed");
  } finally {
    conns.busy = false;
  }
}

export async function closeConnection(id: number) {
  try {
    await ipc.closeConnection(id);
  } catch (e) {
    reportError(e);
  }
  if (conns.activeId === id) {
    conns.activeId = null;
    conns.serverInfo = null;
  }
}
