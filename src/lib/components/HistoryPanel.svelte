<script lang="ts">
  import { ipc } from "../ipc";
  import { conns } from "../stores/connections.svelte";
  import { reportError, pushToast } from "../stores/toast.svelte";
  import { newSqlTab } from "../stores/tabs.svelte";
  import type { HistoryEntry, SavedQuery } from "../types";

  let view = $state<"history" | "saved">("history");
  let history = $state<HistoryEntry[]>([]);
  let saved = $state<SavedQuery[]>([]);
  let search = $state("");

  async function load() {
    if (conns.activeId === null) return;
    try {
      if (view === "history") {
        history = await ipc.listHistory(
          conns.activeId,
          300,
          0,
          search.trim() || null,
        );
      } else {
        saved = await ipc.listSavedQueries(conns.activeId);
      }
    } catch (e) {
      reportError(e, "Could not load");
    }
  }

  $effect(() => {
    void conns.activeId;
    void view;
    load();
  });

  function open(sql: string) {
    newSqlTab(sql);
  }

  async function clearHist() {
    if (conns.activeId === null) return;
    if (!confirm("Clear all history for this connection?")) return;
    try {
      await ipc.clearHistory(conns.activeId);
      pushToast("ok", "History cleared");
      load();
    } catch (e) {
      reportError(e);
    }
  }

  async function delSaved(id: number) {
    try {
      await ipc.deleteSavedQuery(id);
      load();
    } catch (e) {
      reportError(e);
    }
  }
</script>

<div class="hp">
  <div class="toolbar">
    <button class:on={view === "history"} onclick={() => (view = "history")}>history</button>
    <button class:on={view === "saved"} onclick={() => (view = "saved")}>saved</button>
    {#if view === "history"}
      <input
        placeholder="search…"
        bind:value={search}
        onkeydown={(e) => e.key === "Enter" && load()}
      />
      <button onclick={load}>find</button>
      <button class="danger" onclick={clearHist}>clear</button>
    {/if}
    <span class="spacer"></span>
    {#if conns.activeId === null}<span class="faint">not connected</span>{/if}
  </div>

  <div class="list">
    {#if view === "history"}
      {#each history as h (h.id)}
        <button class="entry" onclick={() => open(h.sql)} ondblclick={() => open(h.sql)}>
          <div class="meta">
            <span class={h.success ? "ok" : "err"}>{h.success ? "✓" : "✗"}</span>
            <span class="faint">{h.executed_at.replace("T", " ").slice(0, 19)}</span>
            {#if h.row_count !== null}<span class="faint">· {h.row_count} rows</span>{/if}
            {#if h.elapsed_ms !== null}<span class="faint">· {h.elapsed_ms} ms</span>{/if}
          </div>
          <pre>{h.sql}</pre>
          {#if h.error}<div class="err small">{h.error}</div>{/if}
        </button>
      {:else}
        <p class="faint pad">no history</p>
      {/each}
    {:else}
      {#each saved as s (s.id)}
        <div class="entry srow">
          <button class="sopen" onclick={() => open(s.sql)}>
            <div class="meta"><span class="accent">★ {s.name}</span></div>
            <pre>{s.sql}</pre>
          </button>
          <button class="danger del" onclick={() => delSaved(s.id)}>del</button>
        </div>
      {:else}
        <p class="faint pad">no saved queries</p>
      {/each}
    {/if}
  </div>
</div>

<style>
  .hp {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--fg-faint);
  }
  .toolbar > button.on {
    border-color: var(--accent);
    color: var(--accent);
  }
  .spacer {
    flex: 1;
  }
  .list {
    flex: 1;
    overflow: auto;
  }
  .entry {
    display: block;
    width: 100%;
    text-align: left;
    border: none;
    border-bottom: 1px solid var(--grid);
    padding: 6px 10px;
    color: var(--fg);
    text-transform: none;
    letter-spacing: 0;
  }
  .entry:hover {
    background: var(--bg-raised);
  }
  .meta {
    font-size: 11px;
    margin-bottom: 3px;
  }
  pre {
    white-space: pre-wrap;
    word-break: break-all;
    font-size: 12px;
    color: var(--fg-dim);
    max-height: 80px;
    overflow: hidden;
  }
  .ok {
    color: var(--fg);
  }
  .err {
    color: var(--danger);
  }
  .small {
    font-size: 11px;
  }
  .pad {
    padding: 12px;
  }
  .accent {
    color: var(--accent);
  }
  .srow {
    display: flex;
    align-items: stretch;
    padding: 0;
  }
  .sopen {
    flex: 1;
    text-align: left;
    border: none;
    background: transparent;
    color: var(--fg);
    padding: 6px 10px;
    text-transform: none;
  }
  .sopen:hover {
    background: var(--bg-raised);
  }
  .del {
    border: none;
    padding: 0 10px;
    font-size: 10px;
  }
</style>
