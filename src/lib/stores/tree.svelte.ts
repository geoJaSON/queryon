// Lazy schema tree state, scoped to the active connection.

import { ipc } from "../ipc";
import type { SchemaObjects } from "../types";
import { reportError } from "./toast.svelte";

interface TreeState {
  connId: number | null;
  schemas: string[] | null;
  loadingSchemas: boolean;
  /** schema name -> loaded objects (undefined = not yet loaded). */
  objects: Record<string, SchemaObjects | undefined>;
  expanded: Set<string>;
  selected: { schema: string; table: string } | null;
}

export const tree = $state<TreeState>({
  connId: null,
  schemas: null,
  loadingSchemas: false,
  objects: {},
  expanded: new Set(),
  selected: null,
});

export function resetTree(connId: number | null) {
  tree.connId = connId;
  tree.schemas = null;
  tree.loadingSchemas = false;
  tree.objects = {};
  tree.expanded = new Set();
  tree.selected = null;
}

export async function loadSchemas() {
  if (tree.connId === null || tree.loadingSchemas) return;
  tree.loadingSchemas = true;
  try {
    tree.schemas = await ipc.listSchemas(tree.connId);
  } catch (e) {
    reportError(e, "Could not list schemas");
  } finally {
    tree.loadingSchemas = false;
  }
}

export async function toggleSchema(schema: string) {
  if (tree.expanded.has(schema)) {
    tree.expanded.delete(schema);
    tree.expanded = new Set(tree.expanded);
    return;
  }
  tree.expanded.add(schema);
  tree.expanded = new Set(tree.expanded);
  if (tree.objects[schema] === undefined && tree.connId !== null) {
    try {
      const objs = await ipc.listObjects(tree.connId, schema);
      // Reassign (don't mutate in place) so the template re-reads the new
      // key — same reactivity pattern used for `tree.expanded` above.
      tree.objects = { ...tree.objects, [schema]: objs };
    } catch (e) {
      reportError(e, `Could not list objects in ${schema}`);
      tree.expanded.delete(schema);
      tree.expanded = new Set(tree.expanded);
    }
  }
}
