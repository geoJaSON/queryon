<script lang="ts">
  import { tabsState, newSqlTab, closeTab, selectTab } from "../stores/tabs.svelte";
</script>

<div class="bar">
  {#each tabsState.tabs as t (t.id)}
    <div class="tab" class:active={tabsState.activeId === t.id}>
      <button class="lbl" onclick={() => selectTab(t.id)}>
        {t.kind === "table" ? "▦" : "›"} {t.title}
      </button>
      <button class="x" onclick={() => closeTab(t.id)} title="Close">✕</button>
    </div>
  {/each}
  <button class="add" onclick={() => newSqlTab()} title="New query">+</button>
</div>

<style>
  .bar {
    display: flex;
    align-items: stretch;
    border-bottom: 1px solid var(--fg-faint);
    background: var(--bg-panel);
    height: 26px;
    overflow-x: auto;
  }
  .tab {
    display: flex;
    align-items: center;
    border-right: 1px solid var(--grid);
    background: var(--bg);
  }
  .tab.active {
    background: var(--bg-raised);
    border-bottom: 2px solid var(--accent);
  }
  .lbl {
    border: none;
    text-transform: none;
    letter-spacing: 0;
    padding: 0 8px;
    color: var(--fg-dim);
  }
  .tab.active .lbl {
    color: var(--fg);
  }
  .lbl:hover {
    background: transparent;
  }
  .x {
    border: none;
    padding: 0 6px;
    color: var(--fg-faint);
    font-size: 10px;
  }
  .x:hover {
    color: var(--danger);
    background: transparent;
  }
  .add {
    border: none;
    padding: 0 12px;
    color: var(--accent);
  }
</style>
