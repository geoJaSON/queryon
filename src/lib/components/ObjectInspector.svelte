<script lang="ts">
  import { ipc } from "../ipc";
  import { conns } from "../stores/connections.svelte";
  import { tree } from "../stores/tree.svelte";
  import { reportError } from "../stores/toast.svelte";
  import type { TableMeta } from "../types";

  let meta = $state<TableMeta | null>(null);
  let open = $state(true);
  let key = $derived(
    tree.selected ? `${tree.selected.schema}.${tree.selected.table}` : "",
  );

  $effect(() => {
    const sel = tree.selected;
    const id = conns.activeId;
    meta = null;
    if (sel && id !== null) {
      ipc
        .describeTable(id, sel.schema, sel.table)
        .then((m) => (meta = m))
        .catch((e) => reportError(e, "describe failed"));
    }
  });
</script>

<div class="panel-title insp" onclick={() => (open = !open)} role="button" tabindex="-1">
  {open ? "▾" : "▸"} Inspector {#if key}<span class="faint">— {key}</span>{/if}
</div>
{#if open}
  <div class="body">
    {#if !meta}
      <p class="faint pad">select a table</p>
    {:else}
      <div class="meta">{meta.columns.length} cols · ~{meta.approx_rows} rows</div>
      <table>
        <thead>
          <tr><th>column</th><th>type</th><th>n</th></tr>
        </thead>
        <tbody>
          {#each meta.columns as c}
            <tr>
              <td class:pk={c.is_pk}>
                {c.is_pk ? "🔑" : ""}{c.is_geometry ? "◈" : ""} {c.name}
              </td>
              <td class="ty">{c.udt_name}</td>
              <td class="faint">{c.nullable ? "" : "•"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if meta.indexes.length}
        <div class="grp">INDEXES</div>
        {#each meta.indexes as ix}
          <div class="ix" title={ix.definition}>{ix.name}</div>
        {/each}
      {/if}
    {/if}
  </div>
{/if}

<style>
  .insp {
    cursor: pointer;
  }
  .body {
    max-height: 38vh;
    overflow: auto;
    border-top: 1px solid var(--grid);
  }
  .pad {
    padding: 10px;
  }
  .meta {
    padding: 4px 8px;
    color: var(--fg-dim);
    font-size: 11px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  th {
    text-align: left;
    color: var(--fg-faint);
    font-weight: normal;
    padding: 2px 8px;
  }
  td {
    padding: 1px 8px;
    border-top: 1px solid var(--grid);
    white-space: nowrap;
  }
  td.pk {
    color: var(--accent);
  }
  .ty {
    color: var(--fg-dim);
  }
  .grp {
    color: var(--fg-faint);
    font-size: 10px;
    padding: 6px 8px 2px;
    letter-spacing: 0.1em;
  }
  .ix {
    padding: 1px 8px;
    color: var(--fg-dim);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
