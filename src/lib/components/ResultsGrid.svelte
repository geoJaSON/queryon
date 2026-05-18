<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import type { StatementResult } from "../types";
  import { ipc } from "../ipc";
  import { conns } from "../stores/connections.svelte";
  import { reportError, pushToast } from "../stores/toast.svelte";

  let { result }: { result: StatementResult } = $props();

  const ROW_H = 22;
  const OVERSCAN = 8;

  let scroller = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let viewport = $state(0);

  // Manual fixed-height row virtualization — only the visible window mounts.
  const total = $derived(result.rows.length);
  const start = $derived(Math.max(0, Math.floor(scrollTop / ROW_H) - OVERSCAN));
  const end = $derived(
    Math.min(total, Math.ceil((scrollTop + viewport) / ROW_H) + OVERSCAN),
  );
  const slice = $derived(result.rows.slice(start, end));

  let expanded = $state<{ col: string; val: string; geom: boolean } | null>(null);
  let decoding = $state(false);

  async function exportAs(format: "csv" | "json") {
    try {
      const path = await save({
        defaultPath: `query-result.${format}`,
        filters: [{ name: format.toUpperCase(), extensions: [format] }],
      });
      if (!path) return;
      await ipc.exportResult(
        { columns: result.columns.map((c) => c.name), rows: result.rows },
        format,
        path,
      );
      pushToast("ok", `Exported ${result.rows.length} rows`);
    } catch (e) {
      reportError(e, "Export failed");
    }
  }

  async function decode() {
    if (!expanded || conns.activeId === null) return;
    decoding = true;
    try {
      const ewkt = await ipc.decodeGeometry(conns.activeId, expanded.val);
      expanded = { ...expanded, val: ewkt, geom: false };
    } catch (e) {
      reportError(e, "Geometry decode failed");
    } finally {
      decoding = false;
    }
  }

  function onScroll() {
    if (scroller) scrollTop = scroller.scrollTop;
  }
  $effect(() => {
    if (scroller) viewport = scroller.clientHeight;
  });

  function isGeom(i: number) {
    return result.columns[i]?.render_hint.kind === "geometry";
  }
</script>

<div class="grid">
  <div class="gbar">
    <span class="faint">{result.rows.length} rows{result.truncated ? " (truncated)" : ""}</span>
    <span class="spacer"></span>
    <button class="exp" onclick={() => exportAs("csv")} disabled={!result.rows.length}>
      ⤓ csv
    </button>
    <button class="exp" onclick={() => exportAs("json")} disabled={!result.rows.length}>
      ⤓ json
    </button>
  </div>
  <div class="head" style="--cols:{result.columns.length}">
    <div class="cell idx">#</div>
    {#each result.columns as c}
      <div class="cell" title={`${c.name} :: ${c.type_name}`}>
        {c.name}<span class="ty"> {c.type_name}</span>
      </div>
    {/each}
  </div>

  <div class="scroller" bind:this={scroller} onscroll={onScroll}>
    <div class="spacer" style="height:{total * ROW_H}px">
      <div class="rows" style="transform:translateY({start * ROW_H}px)">
        {#each slice as row, ri (start + ri)}
          <div class="row" style="height:{ROW_H}px">
            <div class="cell idx">{start + ri + 1}</div>
            {#each row as v, ci}
              {#if v === null}
                <div class="cell null">␀</div>
              {:else if isGeom(ci)}
                <button
                  class="cell geom"
                  onclick={() =>
                    (expanded = { col: result.columns[ci].name, val: v, geom: true })}
                  title="geometry — click to expand"
                >
                  ◈ {v}
                </button>
              {:else}
                <button
                  class="cell val"
                  ondblclick={() =>
                    (expanded = { col: result.columns[ci].name, val: v, geom: false })}
                >
                  {v}
                </button>
              {/if}
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

{#if expanded}
  <div
    class="ovl"
    onclick={() => (expanded = null)}
    onkeydown={(e) => e.key === "Escape" && (expanded = null)}
    role="presentation"
  >
    <div class="ovl-box panel" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <div class="panel-title">{expanded.col}</div>
      <pre>{expanded.val}</pre>
      <div class="ovl-foot">
        {#if expanded.geom}
          <button onclick={decode} disabled={decoding || conns.activeId === null}>
            {decoding ? "decoding…" : "◈ decode ewkt"}
          </button>
        {/if}
        <button onclick={() => navigator.clipboard.writeText(expanded!.val)}>copy</button>
        <span class="spacer"></span>
        <button class="primary" onclick={() => (expanded = null)}>close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .grid {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .gbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border-bottom: 1px solid var(--grid);
    font-size: 11px;
  }
  .gbar .spacer {
    flex: 1;
  }
  .exp {
    padding: 1px 8px;
    font-size: 10px;
  }
  .ovl-foot .spacer {
    flex: 1;
  }
  .head,
  .row {
    display: grid;
    grid-template-columns: 48px repeat(var(--cols, 1), minmax(120px, 1fr));
  }
  .head {
    background: var(--bg-raised);
    border-bottom: 1px solid var(--fg-faint);
    position: sticky;
    top: 0;
  }
  .scroller {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .spacer {
    position: relative;
  }
  .rows {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
  }
  .cell {
    padding: 2px 6px;
    border-right: 1px solid var(--grid);
    border-bottom: 1px solid var(--grid);
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 12px;
    text-align: left;
  }
  .head .cell {
    color: var(--fg-dim);
    text-transform: none;
    letter-spacing: 0;
    font-weight: bold;
  }
  .ty {
    color: var(--fg-faint);
    font-weight: normal;
    font-size: 10px;
  }
  .idx {
    color: var(--fg-faint);
    text-align: right;
    background: var(--bg-panel);
  }
  .row:hover .cell:not(.idx) {
    background: var(--bg-raised);
  }
  button.cell {
    border-left: none;
    border-top: none;
    background: transparent;
    color: var(--fg);
  }
  button.cell:hover {
    color: var(--accent);
  }
  .null {
    color: var(--fg-faint);
  }
  .geom {
    color: var(--accent);
  }
  .ovl {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 600;
  }
  .ovl-box {
    width: 70%;
    max-width: 760px;
    max-height: 70%;
    display: flex;
    flex-direction: column;
  }
  .ovl-box pre {
    flex: 1;
    overflow: auto;
    padding: 10px;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--fg);
  }
  .ovl-foot {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    padding: 8px;
    border-top: 1px solid var(--fg-faint);
  }
</style>
