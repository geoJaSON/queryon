<script lang="ts">
  import type { StatementResult } from "../types";

  let {
    statements = [],
    error = null,
  }: { statements?: StatementResult[]; error?: string | null } = $props();
</script>

<div class="msgs">
  {#if error}
    <div class="line err">✗ {error}</div>
  {/if}
  {#each statements as s, i}
    <div class="line">
      <span class="accent">[{i + 1}]</span>
      {s.command_tag || "OK"} — {s.row_count} row{s.row_count === 1 ? "" : "s"} in
      {s.elapsed_ms} ms{s.truncated ? " (truncated)" : ""}
    </div>
    {#each s.notices as n}
      <!-- Already prefixed with the server severity (NOTICE/WARNING/...). -->
      <div class="line notice">{n}</div>
    {/each}
  {/each}
  {#if !error && statements.length === 0}
    <div class="line faint">no messages</div>
  {/if}
</div>

<style>
  .msgs {
    height: 100%;
    overflow: auto;
    padding: 4px 8px;
    font-size: 12px;
  }
  .line {
    white-space: pre-wrap;
    padding: 1px 0;
  }
  .err {
    color: var(--danger);
  }
  .notice {
    color: var(--warn);
  }
  .accent {
    color: var(--accent);
    margin-right: 6px;
  }
</style>
