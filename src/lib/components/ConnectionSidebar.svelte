<script lang="ts">
  import {
    conns,
    refreshConnections,
    openConnection,
    closeConnection,
  } from "../stores/connections.svelte";
  import { ipc } from "../ipc";
  import { reportError, pushToast } from "../stores/toast.svelte";
  import type { ConnectionProfile } from "../types";
  import ConnectionForm from "./ConnectionForm.svelte";

  let editing = $state<ConnectionProfile | null>(null);
  let creating = $state(false);

  async function del(p: ConnectionProfile) {
    if (!confirm(`Delete connection "${p.name}"? This also removes its stored password.`))
      return;
    try {
      await ipc.deleteConnection(p.id);
      pushToast("ok", "Deleted");
      await refreshConnections();
    } catch (e) {
      reportError(e);
    }
  }

  function onSaved() {
    editing = null;
    creating = false;
    refreshConnections();
  }
</script>

<div class="panel-title">Connections</div>
<div class="list">
  {#each conns.profiles as p (p.id)}
    {@const active = conns.activeId === p.id}
    <div class="item" class:active>
      <button
        class="name"
        ondblclick={() => openConnection(p.id)}
        title="Double-click to connect"
      >
        <span class="dot" class:on={active}></span>{p.name}
      </button>
      <div class="meta muted">{p.username}@{p.host}:{p.port}/{p.database}</div>
      <div class="actions">
        {#if active}
          <button onclick={() => closeConnection(p.id)}>disconnect</button>
        {:else}
          <button onclick={() => openConnection(p.id)}>connect</button>
        {/if}
        <button onclick={() => (editing = p)}>edit</button>
        <button class="danger" onclick={() => del(p)}>del</button>
      </div>
    </div>
  {/each}
  {#if conns.profiles.length === 0}
    <p class="faint empty">no saved connections</p>
  {/if}
</div>

<div class="foot">
  <button class="primary" onclick={() => (creating = true)}>+ new connection</button>
</div>

{#if creating}
  <ConnectionForm onclose={() => (creating = false)} onsaved={onSaved} />
{/if}
{#if editing}
  <ConnectionForm profile={editing} onclose={() => (editing = null)} onsaved={onSaved} />
{/if}

<style>
  .list {
    flex: 1;
    overflow: auto;
    padding: 4px;
  }
  .item {
    border: 1px solid transparent;
    border-bottom: 1px solid var(--grid);
    padding: 4px 2px 6px;
  }
  .item.active {
    border-color: var(--fg-faint);
    background: var(--bg-raised);
  }
  .name {
    border: none;
    padding: 2px 0;
    text-transform: none;
    letter-spacing: 0;
    width: 100%;
    text-align: left;
    color: var(--fg);
  }
  .name:hover {
    background: transparent;
    color: var(--accent);
  }
  .dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-right: 7px;
    border: 1px solid var(--fg-faint);
  }
  .dot.on {
    background: var(--fg);
    border-color: var(--fg);
    box-shadow: var(--glow);
  }
  .meta {
    font-size: 10px;
    padding-left: 14px;
    margin-bottom: 4px;
  }
  .actions {
    display: flex;
    gap: 4px;
    padding-left: 14px;
  }
  .actions button {
    padding: 1px 6px;
    font-size: 10px;
  }
  .empty {
    text-align: center;
    margin-top: 2em;
  }
  .foot {
    padding: 6px;
    border-top: 1px solid var(--fg-faint);
  }
  .foot button {
    width: 100%;
  }
</style>
