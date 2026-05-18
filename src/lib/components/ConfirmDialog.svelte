<script lang="ts">
  import { confirmState, answerConfirm } from "../stores/confirm.svelte";
  const req = $derived(confirmState.req);
</script>

{#if req}
  <div
    class="backdrop"
    onclick={() => answerConfirm(false)}
    onkeydown={(e) => e.key === "Escape" && answerConfirm(false)}
    role="presentation"
  >
    <div class="modal panel" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <div class="panel-title" class:dng={req.danger}>{req.title}</div>
      <div class="content">
        <p>{req.body}</p>
        {#if req.sql}<pre>{req.sql}</pre>{/if}
      </div>
      <div class="foot">
        <button onclick={() => answerConfirm(false)}>cancel</button>
        <span class="spacer"></span>
        <button
          class={req.danger ? "danger" : "primary"}
          onclick={() => answerConfirm(true)}
        >
          {req.danger ? "execute" : "ok"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 700;
  }
  .modal {
    width: 480px;
    max-width: 90vw;
  }
  .dng {
    color: var(--danger);
    border-color: var(--danger);
  }
  .content {
    padding: 12px;
  }
  .content pre {
    margin-top: 8px;
    padding: 8px;
    background: var(--bg-input);
    border: 1px solid var(--fg-faint);
    white-space: pre-wrap;
    color: var(--accent);
    font-size: 12px;
  }
  .foot {
    display: flex;
    padding: 10px 12px;
    border-top: 1px solid var(--fg-faint);
  }
  .spacer {
    flex: 1;
  }
</style>
