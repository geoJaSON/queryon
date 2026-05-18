<script lang="ts">
  import { ipc } from "../ipc";
  import { conns } from "../stores/connections.svelte";
  import { reportError, pushToast } from "../stores/toast.svelte";
  import type { GrantSpec } from "../types";

  let { role, onclose }: { role: string; onclose: () => void } = $props();

  const ALL_PRIVS = [
    "SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE",
    "REFERENCES", "TRIGGER", "USAGE", "CREATE", "CONNECT", "EXECUTE",
  ];

  let objectType = $state<"schema" | "all_tables" | "table">("schema");
  let schema = $state("public");
  let object = $state("");
  let chosen = $state<Set<string>>(new Set(["USAGE"]));
  let busy = $state(false);

  function toggle(p: string) {
    chosen.has(p) ? chosen.delete(p) : chosen.add(p);
    chosen = new Set(chosen);
  }

  function spec(): GrantSpec {
    return {
      role,
      privileges: [...chosen],
      object_type: objectType,
      schema,
      object: objectType === "table" ? object : null,
    };
  }

  async function run(revoke: boolean) {
    if (conns.activeId === null) return;
    busy = true;
    try {
      if (revoke) await ipc.revokePrivilege(conns.activeId, spec());
      else await ipc.grantPrivilege(conns.activeId, spec());
      pushToast("ok", revoke ? "Revoked" : "Granted");
      onclose();
    } catch (e) {
      reportError(e, "Privilege change failed");
    } finally {
      busy = false;
    }
  }
</script>

<div
  class="backdrop"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
  role="presentation"
>
  <div class="modal panel" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
    <div class="panel-title">Privileges — {role}</div>
    <div class="form">
      <label for="g-obj">Object</label>
      <select id="g-obj" bind:value={objectType}>
        <option value="schema">schema</option>
        <option value="all_tables">all tables in schema</option>
        <option value="table">single table</option>
      </select>
      <label for="g-sch">Schema</label>
      <input id="g-sch" bind:value={schema} />
      {#if objectType === "table"}
        <label for="g-tbl">Table</label>
        <input id="g-tbl" bind:value={object} />
      {/if}
      <label>Privileges</label>
      <div class="privs">
        {#each ALL_PRIVS as p}
          <button class="pchip" class:on={chosen.has(p)} onclick={() => toggle(p)}>
            {p}
          </button>
        {/each}
      </div>
    </div>
    <div class="foot">
      <button onclick={onclose} disabled={busy}>cancel</button>
      <span class="spacer"></span>
      <button class="danger" onclick={() => run(true)} disabled={busy || !chosen.size}>
        revoke
      </button>
      <button class="primary" onclick={() => run(false)} disabled={busy || !chosen.size}>
        grant
      </button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 600;
  }
  .modal {
    width: 460px;
  }
  .form {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .privs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .pchip {
    font-size: 10px;
    padding: 2px 6px;
  }
  .pchip.on {
    border-color: var(--accent);
    color: var(--accent);
    background: rgba(255, 176, 0, 0.08);
  }
  .foot {
    display: flex;
    gap: 6px;
    padding: 10px 12px;
    border-top: 1px solid var(--fg-faint);
  }
  .spacer {
    flex: 1;
  }
</style>
