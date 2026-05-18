<script lang="ts">
  import type { Tab } from "../stores/tabs.svelte";
  import { conns } from "../stores/connections.svelte";
  import { ipc } from "../ipc";
  import { reportError } from "../stores/toast.svelte";
  import type { FilterSpec, SortSpec, TableMeta } from "../types";
  import ResultsGrid from "./ResultsGrid.svelte";

  let { tab }: { tab: Tab } = $props();

  const OPS = ["=", "<>", "<", "<=", ">", ">=", "LIKE", "ILIKE", "IS NULL", "IS NOT NULL"];

  let meta = $state<TableMeta | null>(null);
  let page = $state(0);
  let pageSize = $state(100);
  let sortCol = $state("");
  let sortDesc = $state(false);
  let geojson = $state(false);
  let fCol = $state("");
  let fOp = $state("=");
  let fVal = $state("");
  let filters = $state<FilterSpec[]>([]);

  async function loadMeta() {
    if (conns.activeId === null || !tab.schema || !tab.table) return;
    try {
      meta = await ipc.describeTable(conns.activeId, tab.schema, tab.table);
    } catch (e) {
      reportError(e, "describe failed");
    }
  }

  async function load() {
    if (conns.activeId === null || !tab.schema || !tab.table) return;
    tab.running = true;
    tab.error = null;
    const sort: SortSpec | null = sortCol
      ? { column: sortCol, descending: sortDesc }
      : null;
    try {
      tab.result = await ipc.fetchTablePage(
        conns.activeId,
        tab.schema,
        tab.table,
        page,
        pageSize,
        sort,
        filters,
        geojson,
      );
    } catch (e: any) {
      tab.result = null;
      tab.error = e?.message ?? String(e);
      reportError(e, "fetch failed");
    } finally {
      tab.running = false;
    }
  }

  function addFilter() {
    if (!fCol) return;
    const needsVal = fOp !== "IS NULL" && fOp !== "IS NOT NULL";
    filters = [
      ...filters,
      { column: fCol, op: fOp, value: needsVal ? fVal : null },
    ];
    fVal = "";
    page = 0;
    load();
  }
  function dropFilter(i: number) {
    filters = filters.filter((_, x) => x !== i);
    page = 0;
    load();
  }
  function prev() {
    if (page > 0) {
      page--;
      load();
    }
  }
  function next() {
    page++;
    load();
  }

  $effect(() => {
    // (Re)load when the tab's target changes.
    void tab.table;
    loadMeta();
    load();
  });

  const cur = $derived(tab.result?.statements[0] ?? null);
</script>

<div class="tv">
  <div class="toolbar">
    <button onclick={prev} disabled={page === 0 || tab.running}>◂ prev</button>
    <span class="pg">page {page + 1}</span>
    <button onclick={next} disabled={tab.running}>next ▸</button>
    <label class="inl">rows
      <select bind:value={pageSize} onchange={() => { page = 0; load(); }}>
        <option value={50}>50</option>
        <option value={100}>100</option>
        <option value={500}>500</option>
        <option value={1000}>1000</option>
      </select>
    </label>
    <label class="inl">sort
      <select bind:value={sortCol} onchange={() => { page = 0; load(); }}>
        <option value="">—</option>
        {#each meta?.columns ?? [] as c}
          <option value={c.name}>{c.name}</option>
        {/each}
      </select>
    </label>
    <button
      onclick={() => { sortDesc = !sortDesc; load(); }}
      disabled={!sortCol}
      title="Toggle sort direction"
    >
      {sortDesc ? "desc" : "asc"}
    </button>
    <label class="inl">
      <input type="checkbox" bind:checked={geojson} onchange={load} /> geojson
    </label>
    <span class="spacer"></span>
    {#if meta}<span class="faint">~{meta.approx_rows} rows</span>{/if}
  </div>

  <div class="filterbar">
    <select bind:value={fCol}>
      <option value="">column…</option>
      {#each meta?.columns ?? [] as c}
        <option value={c.name}>{c.name}</option>
      {/each}
    </select>
    <select bind:value={fOp}>
      {#each OPS as o}<option value={o}>{o}</option>{/each}
    </select>
    <input
      bind:value={fVal}
      placeholder="value"
      disabled={fOp === "IS NULL" || fOp === "IS NOT NULL"}
      onkeydown={(e) => e.key === "Enter" && addFilter()}
    />
    <button onclick={addFilter} disabled={!fCol}>+ filter</button>
    {#each filters as f, i}
      <button class="chip" onclick={() => dropFilter(i)} title="Remove">
        {f.column} {f.op} {f.value ?? ""} ✕
      </button>
    {/each}
  </div>

  <div class="body">
    {#if cur && cur.columns.length}
      <ResultsGrid result={cur} />
    {:else if tab.error}
      <p class="danger-text pad">{tab.error}</p>
    {:else}
      <p class="faint pad blink">loading</p>
    {/if}
  </div>
</div>

<style>
  .tv {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
  .toolbar,
  .filterbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--grid);
    flex-wrap: wrap;
  }
  .inl {
    display: flex;
    align-items: center;
    gap: 4px;
    text-transform: none;
    font-size: 11px;
  }
  .pg {
    color: var(--fg-dim);
    font-size: 11px;
  }
  .spacer {
    flex: 1;
  }
  .chip {
    padding: 1px 6px;
    font-size: 10px;
    border-color: var(--accent);
    color: var(--accent);
    text-transform: none;
  }
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .body > :global(*) {
    flex: 1;
    min-width: 0;
  }
  .pad {
    padding: 10px;
  }
</style>
