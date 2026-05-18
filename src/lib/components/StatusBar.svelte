<script lang="ts">
  import { conns, activeProfile } from "../stores/connections.svelte";
  const prof = $derived(activeProfile());
</script>

<footer class="status">
  <span class="seg">
    {#if prof}
      <span class="dot on"></span>{prof.name}
      <span class="muted">— {prof.username}@{prof.host}:{prof.port}/{prof.database}</span>
    {:else}
      <span class="dot off"></span><span class="muted">no connection</span>
    {/if}
  </span>
  <span class="spacer"></span>
  {#if conns.serverInfo}
    <span class="seg muted">{conns.serverInfo.version}</span>
    {#if conns.serverInfo.postgis_version}
      <span class="seg accent">postgis {conns.serverInfo.postgis_version}</span>
    {/if}
  {/if}
  {#if conns.busy}<span class="seg accent blink">working</span>{/if}
</footer>

<style>
  .status {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 2px 10px;
    border-top: 1px solid var(--fg-faint);
    background: var(--bg-panel);
    font-size: 11px;
    height: 22px;
  }
  .spacer {
    flex: 1;
  }
  .accent {
    color: var(--accent);
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    margin-right: 6px;
    border: 1px solid currentColor;
  }
  .dot.on {
    background: var(--fg);
    box-shadow: var(--glow);
  }
  .dot.off {
    background: transparent;
    color: var(--fg-faint);
  }
</style>
