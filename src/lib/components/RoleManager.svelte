<script lang="ts">
  import { ipc } from "../ipc";
  import { conns } from "../stores/connections.svelte";
  import { reportError, pushToast } from "../stores/toast.svelte";
  import { confirmAction } from "../stores/confirm.svelte";
  import type { RoleInfo, RoleSpec } from "../types";
  import GrantEditor from "./GrantEditor.svelte";

  let roles = $state<RoleInfo[]>([]);
  let loading = $state(false);
  let grantFor = $state<string | null>(null);

  // Create/edit form
  let editing = $state<string | null>(null); // role name being altered, or null
  let f = $state<RoleSpec>(blank());
  function blank(): RoleSpec {
    return {
      name: "",
      login: true,
      superuser: false,
      createdb: false,
      createrole: false,
      password: null,
    };
  }

  async function load() {
    if (conns.activeId === null) return;
    loading = true;
    try {
      roles = await ipc.listRoles(conns.activeId);
    } catch (e) {
      reportError(e, "Could not list roles");
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void conns.activeId;
    load();
  });

  function startEdit(r: RoleInfo) {
    editing = r.name;
    f = {
      name: r.name,
      login: r.can_login,
      superuser: r.superuser,
      createdb: r.createdb,
      createrole: r.createrole,
      password: null,
    };
  }
  function startCreate() {
    editing = null;
    f = blank();
  }

  async function submit() {
    if (conns.activeId === null || !f.name.trim()) return;
    try {
      if (editing) await ipc.alterRole(conns.activeId, editing, f);
      else await ipc.createRole(conns.activeId, f);
      pushToast("ok", editing ? "Role altered" : "Role created");
      f = blank();
      editing = null;
      load();
    } catch (e) {
      reportError(e, "Role operation failed");
    }
  }

  async function drop(name: string) {
    const ok = await confirmAction({
      title: "Drop role",
      body: `Permanently drop role "${name}". This fails if the role owns objects.`,
      sql: `DROP ROLE "${name.replace(/"/g, '""')}";`,
      danger: true,
    });
    if (!ok || conns.activeId === null) return;
    try {
      await ipc.dropRole(conns.activeId, name);
      pushToast("ok", "Role dropped");
      load();
    } catch (e) {
      reportError(e, "Drop failed");
    }
  }
</script>

<div class="rm">
  <div class="toolbar">
    <h3>Roles</h3>
    <button onclick={load} disabled={loading}>↻ refresh</button>
    <button class="primary" onclick={startCreate}>+ new role</button>
    <span class="spacer"></span>
    {#if conns.activeId === null}<span class="faint">not connected</span>{/if}
  </div>

  <div class="split">
    <div class="list">
      <table>
        <thead>
          <tr><th>name</th><th>login</th><th>super</th><th>createdb</th><th>createrole</th><th>member of</th><th></th></tr>
        </thead>
        <tbody>
          {#each roles as r (r.name)}
            <tr class:sel={editing === r.name}>
              <td>{r.name}</td>
              <td class="b">{r.can_login ? "●" : ""}</td>
              <td class="b">{r.superuser ? "●" : ""}</td>
              <td class="b">{r.createdb ? "●" : ""}</td>
              <td class="b">{r.createrole ? "●" : ""}</td>
              <td class="faint">{r.member_of.join(", ")}</td>
              <td class="acts">
                <button onclick={() => startEdit(r)}>edit</button>
                <button onclick={() => (grantFor = r.name)}>grants</button>
                <button class="danger" onclick={() => drop(r.name)}>drop</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="editor panel">
      <div class="panel-title">{editing ? `Alter ${editing}` : "Create role"}</div>
      <div class="form">
        <label for="r-name">Name</label>
        <input id="r-name" bind:value={f.name} disabled={!!editing} />
        <label class="ck"><input type="checkbox" bind:checked={f.login} /> LOGIN</label>
        <label class="ck"><input type="checkbox" bind:checked={f.superuser} /> SUPERUSER</label>
        <label class="ck"><input type="checkbox" bind:checked={f.createdb} /> CREATEDB</label>
        <label class="ck"><input type="checkbox" bind:checked={f.createrole} /> CREATEROLE</label>
        <label for="r-pw">Password {#if editing}<span class="faint">(blank = keep)</span>{/if}</label>
        <input
          id="r-pw"
          type="password"
          autocomplete="off"
          value={f.password ?? ""}
          oninput={(e) => (f.password = e.currentTarget.value || null)}
        />
        <div class="frow">
          {#if editing}<button onclick={startCreate}>cancel</button>{/if}
          <span class="spacer"></span>
          <button class="primary" onclick={submit} disabled={!f.name.trim()}>
            {editing ? "alter" : "create"}
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

{#if grantFor}
  <GrantEditor role={grantFor} onclose={() => (grantFor = null)} />
{/if}

<style>
  .rm {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--fg-faint);
  }
  .spacer {
    flex: 1;
  }
  .split {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .list {
    flex: 1;
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  th {
    position: sticky;
    top: 0;
    background: var(--bg-raised);
    color: var(--fg-dim);
    text-align: left;
    padding: 4px 8px;
    font-weight: normal;
    border-bottom: 1px solid var(--fg-faint);
  }
  td {
    padding: 3px 8px;
    border-bottom: 1px solid var(--grid);
    white-space: nowrap;
  }
  tr.sel td {
    background: var(--bg-raised);
  }
  td.b {
    color: var(--accent);
    text-align: center;
  }
  .acts {
    display: flex;
    gap: 4px;
  }
  .acts button {
    padding: 1px 6px;
    font-size: 10px;
  }
  .editor {
    width: 280px;
    flex-shrink: 0;
    border-top: none;
    border-bottom: none;
    border-right: none;
  }
  .form {
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ck {
    text-transform: none;
    font-size: 12px;
    color: var(--fg);
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .frow {
    display: flex;
    margin-top: 6px;
  }
</style>
