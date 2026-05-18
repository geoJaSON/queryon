<script lang="ts">
  import { toasts, dismissToast } from "../stores/toast.svelte";
</script>

<div class="stack">
  {#each toasts.items as t (t.id)}
    <div class="toast {t.kind}" onclick={() => dismissToast(t.id)} role="button" tabindex="-1">
      <div class="line">
        <span class="tag">
          {t.kind === "error" ? "ERR" : t.kind === "ok" ? "OK " : "INF"}
        </span>
        <span>{t.text}</span>
      </div>
      {#if t.detail}<pre class="detail">{t.detail}</pre>{/if}
    </div>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    right: 12px;
    bottom: 34px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-width: 460px;
    z-index: 1000;
  }
  .toast {
    background: var(--bg-raised);
    border: 1px solid var(--fg-faint);
    padding: 6px 10px;
    cursor: pointer;
  }
  .toast.error {
    border-color: var(--danger);
    color: var(--danger);
  }
  .toast.ok {
    border-color: var(--fg);
  }
  .tag {
    color: var(--accent);
    margin-right: 8px;
  }
  .toast.error .tag {
    color: var(--danger);
  }
  .detail {
    margin-top: 4px;
    font-size: 11px;
    white-space: pre-wrap;
    color: var(--fg-dim);
    max-height: 160px;
    overflow: auto;
  }
</style>
