<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { keymap } from "@codemirror/view";
  import { EditorState, Prec } from "@codemirror/state";
  import { sql, PostgreSQL } from "@codemirror/lang-sql";
  import type { Tab } from "../stores/tabs.svelte";
  import { conns } from "../stores/connections.svelte";
  import { ipc } from "../ipc";
  import { reportError, pushToast } from "../stores/toast.svelte";
  import ResultsGrid from "./ResultsGrid.svelte";
  import MessagesPane from "./MessagesPane.svelte";

  let { tab }: { tab: Tab } = $props();

  let host = $state<HTMLDivElement | null>(null);
  let view: EditorView | null = null;
  let activeStmt = $state(0);
  let bottomTab = $state<"data" | "messages">("data");

  const phosphorTheme = EditorView.theme(
    {
      "&": { color: "var(--fg)", backgroundColor: "var(--bg-input)", height: "100%" },
      ".cm-content": { fontFamily: "var(--font-mono)", caretColor: "var(--accent)" },
      ".cm-cursor": { borderLeftColor: "var(--accent)" },
      ".cm-gutters": {
        backgroundColor: "var(--bg-panel)",
        color: "var(--fg-faint)",
        border: "none",
      },
      ".cm-activeLine": { backgroundColor: "rgba(51,255,102,0.05)" },
      ".cm-activeLineGutter": { backgroundColor: "var(--bg-raised)" },
      ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
        backgroundColor: "rgba(31,155,65,0.4)",
      },
      ".cm-keyword": { color: "var(--accent)" },
      ".cm-string": { color: "#7fffa8" },
      ".cm-number": { color: "#7fffa8" },
      ".cm-comment": { color: "var(--fg-faint)", fontStyle: "italic" },
    },
    { dark: true },
  );

  function selectedOrAll(): string {
    if (!view) return tab.sql;
    const { from, to } = view.state.selection.main;
    return from === to ? view.state.doc.toString() : view.state.sliceDoc(from, to);
  }

  async function run(sqlText: string) {
    if (conns.activeId === null) {
      reportError({ message: "No active connection" });
      return;
    }
    if (!sqlText.trim()) return;
    tab.running = true;
    tab.error = null;
    try {
      const res = await ipc.runSql(conns.activeId, sqlText);
      tab.result = res;
      activeStmt = 0;
      bottomTab = res.statements.some((s) => s.columns.length > 0)
        ? "data"
        : "messages";
    } catch (e: any) {
      tab.result = null;
      tab.error = e?.message ?? String(e);
      bottomTab = "messages";
      reportError(e, "Query failed");
    } finally {
      tab.running = false;
    }
  }

  async function cancel() {
    if (conns.activeId === null) return;
    try {
      await ipc.cancelQuery(conns.activeId);
    } catch (e) {
      reportError(e, "Cancel failed");
    }
  }

  async function saveCurrent() {
    if (conns.activeId === null) {
      reportError({ message: "No active connection" });
      return;
    }
    const sql = view ? view.state.doc.toString() : tab.sql;
    if (!sql.trim()) return;
    const name = prompt("Save query as:");
    if (!name) return;
    try {
      await ipc.saveQuery(conns.activeId, name, sql);
      pushToast("ok", `Saved “${name}”`);
    } catch (e) {
      reportError(e, "Save failed");
    }
  }

  onMount(() => {
    view = new EditorView({
      parent: host!,
      state: EditorState.create({
        doc: tab.sql,
        extensions: [
          basicSetup,
          sql({ dialect: PostgreSQL }),
          phosphorTheme,
          Prec.highest(
            keymap.of([
              { key: "Mod-Enter", run: () => (run(selectedOrAll()), true) },
              { key: "Mod-s", run: () => (saveCurrent(), true) },
            ]),
          ),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) tab.sql = u.state.doc.toString();
          }),
        ],
      }),
    });
  });

  onDestroy(() => view?.destroy());

  const stmts = $derived(tab.result?.statements ?? []);
  const cur = $derived(stmts[activeStmt] ?? null);
</script>

<div class="editor-tab">
  <div class="toolbar">
    <button
      class="primary"
      onclick={() => run(selectedOrAll())}
      disabled={tab.running || conns.activeId === null}
    >
      {tab.running ? "running…" : "▶ run"}
    </button>
    {#if tab.running}
      <button class="danger" onclick={cancel}>■ cancel</button>
    {/if}
    <button
      onclick={() => view && run(view.state.sliceDoc(
        view.state.selection.main.from, view.state.selection.main.to))}
      disabled={tab.running || conns.activeId === null}
      title="Run selected text only"
    >
      run selection
    </button>
    <button onclick={saveCurrent} disabled={conns.activeId === null} title="Save query">
      ★ save
    </button>
    <span class="hint faint">ctrl+enter</span>
  </div>

  <div class="cm" bind:this={host}></div>

  <div class="results panel">
    <div class="rtabs">
      <button class:on={bottomTab === "data"} onclick={() => (bottomTab = "data")}>
        data
      </button>
      <button class:on={bottomTab === "messages"} onclick={() => (bottomTab = "messages")}>
        messages
      </button>
      <span class="spacer"></span>
      {#if stmts.length > 1}
        <span class="faint">result</span>
        {#each stmts as _, i}
          <button class="rsw" class:on={activeStmt === i} onclick={() => (activeStmt = i)}>
            {i + 1}
          </button>
        {/each}
      {/if}
    </div>
    <div class="rbody">
      {#if bottomTab === "data"}
        {#if cur && cur.columns.length > 0}
          <ResultsGrid result={cur} />
        {:else}
          <div class="faint pad">no tabular result — see messages</div>
        {/if}
      {:else}
        <MessagesPane statements={stmts} error={tab.error} />
      {/if}
    </div>
  </div>
</div>

<style>
  .editor-tab {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--grid);
  }
  .hint {
    font-size: 10px;
  }
  .cm {
    height: 38%;
    min-height: 120px;
    overflow: hidden;
    border-bottom: 1px solid var(--fg-faint);
  }
  .cm :global(.cm-editor) {
    height: 100%;
  }
  .results {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    border: none;
  }
  .rtabs {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-raised);
    border-bottom: 1px solid var(--fg-faint);
    padding: 2px 6px;
  }
  .rtabs button {
    border: none;
    padding: 2px 10px;
    color: var(--fg-dim);
    font-size: 11px;
  }
  .rtabs button.on {
    color: var(--accent);
    border-bottom: 2px solid var(--accent);
  }
  .rsw {
    min-width: 22px;
  }
  .spacer {
    flex: 1;
  }
  .rbody {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .rbody > :global(*) {
    flex: 1;
    min-width: 0;
  }
  .pad {
    padding: 10px;
  }
</style>
