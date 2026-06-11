<script lang="ts">
  import { onMount } from "svelte";
  import ConnectionSidebar from "./lib/components/ConnectionSidebar.svelte";
  import SchemaTree from "./lib/components/SchemaTree.svelte";
  import ObjectInspector from "./lib/components/ObjectInspector.svelte";
  import TabBar from "./lib/components/TabBar.svelte";
  import SqlEditorTab from "./lib/components/SqlEditorTab.svelte";
  import TableViewTab from "./lib/components/TableViewTab.svelte";
  import RoleManager from "./lib/components/RoleManager.svelte";
  import HistoryPanel from "./lib/components/HistoryPanel.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import ConfirmDialog from "./lib/components/ConfirmDialog.svelte";
  import { refreshConnections } from "./lib/stores/connections.svelte";
  import { tabsState, activeTab, newSqlTab } from "./lib/stores/tabs.svelte";

  onMount(refreshConnections);

  const current = $derived(activeTab());
</script>

<div class="shell">
  <header class="titlebar">
    <span class="brand">QUERYON</span>
    <span class="muted">// postgis terminal</span>
  </header>

  <div class="body">
    <aside class="sidebar panel">
      <div class="conns"><ConnectionSidebar /></div>
      <div class="treebox"><SchemaTree /></div>
      <div class="inspbox"><ObjectInspector /></div>
    </aside>

    <main class="workspace">
      <TabBar />
      <div class="tabview">
        {#if current && current.kind === "sql"}
          <!-- Keyed so each tab gets its own CodeMirror instance; without
               this, switching SQL tabs reuses one editor and its buffer. -->
          {#key current.id}
            <SqlEditorTab tab={current} />
          {/key}
        {:else if current && current.kind === "table"}
          {#key current.id}
            <TableViewTab tab={current} />
          {/key}
        {:else if current && current.kind === "roles"}
          <RoleManager />
        {:else if current && current.kind === "history"}
          <HistoryPanel />
        {:else}
          <div class="empty">
            <p class="muted">no query open</p>
            <button class="primary" onclick={() => newSqlTab()}>+ new query</button>
            <p class="faint blink" style="margin-top:1em">&nbsp;</p>
          </div>
        {/if}
      </div>
    </main>
  </div>

  <StatusBar />
  <Toasts />
  <ConfirmDialog />
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .titlebar {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--fg-faint);
    background: var(--bg-panel);
  }
  .brand {
    color: var(--accent);
    text-shadow: var(--glow-amber);
    letter-spacing: 0.3em;
    font-weight: bold;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .sidebar {
    width: 264px;
    flex-shrink: 0;
    border-top: none;
    border-bottom: none;
    border-left: none;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .conns {
    display: flex;
    flex-direction: column;
    max-height: 42%;
    border-bottom: 1px solid var(--fg-faint);
    overflow: hidden;
  }
  .treebox {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }
  .inspbox {
    border-top: 1px solid var(--fg-faint);
    flex-shrink: 0;
  }
  .workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .tabview {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .empty {
    margin: auto;
    text-align: center;
  }
  .empty button {
    margin-top: 1em;
  }
</style>
