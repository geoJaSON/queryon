<script lang="ts">
  import { conns } from "../stores/connections.svelte";
  import {
    tree,
    resetTree,
    loadSchemas,
    toggleSchema,
  } from "../stores/tree.svelte";
  import { newTableTab, newRolesTab, newHistoryTab } from "../stores/tabs.svelte";
  import { ipc } from "../ipc";
  import { reportError, pushToast } from "../stores/toast.svelte";
  import { confirmAction } from "../stores/confirm.svelte";

  async function createSchema() {
    if (conns.activeId === null) return;
    const name = prompt("New schema name:");
    if (!name) return;
    try {
      await ipc.createSchema(conns.activeId, name, null);
      pushToast("ok", `Schema ${name} created`);
      resetTree(conns.activeId);
      loadSchemas();
    } catch (e) {
      reportError(e, "Create schema failed");
    }
  }

  async function dropSchema(name: string) {
    if (conns.activeId === null) return;
    const cascade = confirm(
      `Drop schema "${name}" with CASCADE (drop all contained objects)?\n\nOK = CASCADE, Cancel = restrict`,
    );
    const ok = await confirmAction({
      title: "Drop schema",
      body: `Drop schema "${name}"${cascade ? " and everything in it" : ""}.`,
      sql: `DROP SCHEMA "${name.replace(/"/g, '""')}"${cascade ? " CASCADE" : ""};`,
      danger: true,
    });
    if (!ok) return;
    try {
      await ipc.dropSchema(conns.activeId, name, cascade);
      pushToast("ok", `Schema ${name} dropped`);
      resetTree(conns.activeId);
      loadSchemas();
    } catch (e) {
      reportError(e, "Drop schema failed");
    }
  }

  // Reload the tree whenever the active connection changes.
  $effect(() => {
    const id = conns.activeId;
    if (id !== tree.connId) {
      resetTree(id);
      if (id !== null) loadSchemas();
    }
  });

  function openTable(schema: string, name: string) {
    tree.selected = { schema, table: name };
    newTableTab(schema, name);
  }

  function selectOnly(schema: string, name: string) {
    tree.selected = { schema, table: name };
  }
</script>

<div class="panel-title hdr">
  <span>Schema</span>
  <span class="spacer"></span>
  {#if conns.activeId !== null}
    <button class="mini" title="New schema" onclick={createSchema}>＋</button>
    <button class="mini" title="History & saved" onclick={() => newHistoryTab()}>≣</button>
    <button class="mini" title="Role manager" onclick={() => newRolesTab()}>⚷</button>
  {/if}
</div>
<div class="tree">
  {#if conns.activeId === null}
    <p class="faint pad">connect to browse</p>
  {:else if tree.loadingSchemas && !tree.schemas}
    <p class="faint pad blink">loading</p>
  {:else if tree.schemas}
    {#each tree.schemas as s (s)}
      {@const open = tree.expanded.has(s)}
      {@const objs = tree.objects[s]}
      <div class="node">
        <div class="schrow">
          <button class="row sch" onclick={() => toggleSchema(s)}>
            <span class="tw">{open ? "▾" : "▸"}</span>▣ {s}
          </button>
          <button class="drop" title="Drop schema" onclick={() => dropSchema(s)}>✕</button>
        </div>
        {#if open}
          {#if !objs}
            <div class="row faint" style="padding-left:24px">…</div>
          {:else}
            {#each [["TABLES", objs.tables], ["VIEWS", objs.views], ["MATVIEWS", objs.mat_views], ["FUNCTIONS", objs.functions]] as [label, items]}
              {#if (items as string[]).length}
                <div class="grp" style="padding-left:24px">{label}</div>
                {#each items as it (label + it)}
                  {@const isTbl = label === "TABLES" || label === "VIEWS" || label === "MATVIEWS"}
                  <button
                    class="row leaf"
                    class:sel={tree.selected?.schema === s && tree.selected?.table === it}
                    style="padding-left:34px"
                    onclick={() => isTbl ? selectOnly(s, it) : null}
                    ondblclick={() => isTbl && openTable(s, it)}
                    title={isTbl ? "Double-click to open" : ""}
                  >
                    {label === "FUNCTIONS" ? "ƒ" : "▦"} {it}
                  </button>
                {/each}
              {/if}
            {/each}
            {#if !objs.tables.length && !objs.views.length && !objs.mat_views.length && !objs.functions.length}
              <div class="row faint" style="padding-left:24px">(empty)</div>
            {/if}
          {/if}
        {/if}
      </div>
    {/each}
    {#if tree.schemas.length === 0}
      <p class="faint pad">no schemas</p>
    {/if}
  {/if}
</div>

<style>
  .tree {
    flex: 1;
    overflow: auto;
    padding: 2px 0;
    font-size: 12px;
  }
  .pad {
    padding: 10px;
  }
  .row {
    display: block;
    width: 100%;
    text-align: left;
    border: none;
    text-transform: none;
    letter-spacing: 0;
    padding: 2px 6px;
    color: var(--fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row:hover {
    background: var(--bg-raised);
    color: var(--accent);
  }
  .schrow {
    display: flex;
    align-items: center;
  }
  .schrow .sch {
    flex: 1;
  }
  .drop {
    border: none;
    color: var(--fg-faint);
    padding: 0 6px;
    font-size: 10px;
    opacity: 0;
  }
  .schrow:hover .drop {
    opacity: 1;
  }
  .drop:hover {
    color: var(--danger);
    background: transparent;
  }
  .hdr {
    display: flex;
    align-items: center;
  }
  .hdr .spacer {
    flex: 1;
  }
  .mini {
    border: none;
    padding: 0 6px;
    color: var(--accent);
    font-size: 12px;
  }
  .sch {
    color: var(--fg);
  }
  .tw {
    display: inline-block;
    width: 12px;
    color: var(--fg-dim);
  }
  .grp {
    color: var(--fg-faint);
    font-size: 10px;
    letter-spacing: 0.1em;
    padding: 3px 0 1px;
  }
  .leaf {
    color: var(--fg-dim);
  }
  .leaf.sel {
    background: var(--bg-raised);
    color: var(--accent);
  }
</style>
