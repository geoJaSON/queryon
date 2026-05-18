<script lang="ts">
  import { ipc } from "../ipc";
  import type { ConnectionProfile, ConnectionInput, SslMode } from "../types";
  import { reportError, pushToast } from "../stores/toast.svelte";

  let {
    profile = null,
    onclose,
    onsaved,
  }: {
    profile?: ConnectionProfile | null;
    onclose: () => void;
    onsaved: () => void;
  } = $props();

  const isEdit = !!profile;

  let name = $state(profile?.name ?? "");
  let host = $state(profile?.host ?? "localhost");
  let port = $state(profile?.port ?? 5432);
  let database = $state(profile?.database ?? "postgres");
  let username = $state(profile?.username ?? "postgres");
  let password = $state("");
  let ssl_mode = $state<SslMode>(profile?.ssl_mode ?? "prefer");
  let busy = $state(false);

  function build(): ConnectionInput {
    return {
      name: name.trim(),
      host: host.trim(),
      port: Number(port),
      database: database.trim(),
      username: username.trim(),
      // Empty password on edit = "keep existing"; backend treats null that way.
      password: password === "" ? (isEdit ? null : "") : password,
      ssl_mode,
      color: profile?.color ?? null,
    };
  }

  async function save() {
    if (!name.trim()) {
      reportError({ message: "Name is required" });
      return;
    }
    busy = true;
    try {
      if (isEdit && profile) {
        await ipc.updateConnection(profile.id, build());
      } else {
        await ipc.saveConnection(build());
      }
      pushToast("ok", "Saved");
      onsaved();
    } catch (e) {
      reportError(e, "Could not save connection");
    } finally {
      busy = false;
    }
  }

  async function test() {
    busy = true;
    try {
      // Persist first so the backend can build a pool and resolve the password.
      let id: number;
      if (isEdit && profile) {
        await ipc.updateConnection(profile.id, build());
        id = profile.id;
      } else {
        id = (await ipc.saveConnection(build())).id;
      }
      const info = await ipc.testConnection(id);
      pushToast(
        "ok",
        `OK — ${info.version}${info.postgis_version ? ` / PostGIS ${info.postgis_version}` : ""}`,
      );
      onsaved();
    } catch (e) {
      reportError(e, "Connection test failed");
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
    <div class="panel-title">{isEdit ? "Edit connection" : "New connection"}</div>
    <div class="form">
      <label for="f-name">Name</label>
      <input id="f-name" bind:value={name} placeholder="local postgis" />

      <div class="row">
        <div class="col grow">
          <label for="f-host">Host</label>
          <input id="f-host" bind:value={host} />
        </div>
        <div class="col">
          <label for="f-port">Port</label>
          <input id="f-port" type="number" bind:value={port} style="width:80px" />
        </div>
      </div>

      <label for="f-db">Database</label>
      <input id="f-db" bind:value={database} />

      <label for="f-user">Username</label>
      <input id="f-user" bind:value={username} />

      <label for="f-pass">Password {#if isEdit}<span class="faint">(blank = keep)</span>{/if}</label>
      <input id="f-pass" type="password" bind:value={password} autocomplete="off" />

      <label for="f-ssl">SSL mode</label>
      <select id="f-ssl" bind:value={ssl_mode}>
        <option value="disable">disable</option>
        <option value="prefer">prefer</option>
        <option value="require">require</option>
      </select>
    </div>
    <div class="foot">
      <button onclick={onclose} disabled={busy}>cancel</button>
      <span class="spacer"></span>
      <button onclick={test} disabled={busy}>test</button>
      <button class="primary" onclick={save} disabled={busy}>save</button>
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
    z-index: 500;
  }
  .modal {
    width: 420px;
  }
  .form {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .form input,
  .form select {
    margin-bottom: 6px;
  }
  .row {
    display: flex;
    gap: 10px;
  }
  .col {
    display: flex;
    flex-direction: column;
  }
  .col.grow {
    flex: 1;
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
