<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import ScriptList from "$lib/components/ScriptList.svelte";
  import ParamPanel from "$lib/components/ParamPanel.svelte";
  import RawEditor from "$lib/components/RawEditor.svelte";
  import LogTerminal from "$lib/components/LogTerminal.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import DiffDialog from "$lib/components/DiffDialog.svelte";
  import {
    applyEdits,
    cancelSave,
    commitSave,
    initProcessEvents,
    initWorkspace,
    isDirty,
    prepareSave,
    setRaw,
    workspace,
  } from "$lib/stores/workspace.svelte";

  let tab = $state<"params" | "raw">("params");
  const dirty = $derived(isDirty());

  onMount(() => {
    // 启动时恢复上次工作目录、订阅进程事件
    void initWorkspace();
    void initProcessEvents();
  });

  async function openTab(next: "params" | "raw") {
    // 切到原始脚本前，先把参数改动落成草稿
    if (next === "raw") await applyEdits();
    tab = next;
  }
</script>

<div class="app">
  <TopBar />

  <div class="body">
    <ScriptList />

    <main class="main">
      <nav class="tabs">
        <button
          class="tab"
          class:active={tab === "params"}
          type="button"
          onclick={() => openTab("params")}
        >
          参数面板
        </button>
        <button class="tab" class:active={tab === "raw"} type="button" onclick={() => openTab("raw")}>
          原始脚本
        </button>
        <div class="right">
          {#if workspace.error}
            <span class="err" title={workspace.error}>{workspace.error}</span>
          {/if}
          <button class="ghost" type="button" disabled={!dirty} onclick={prepareSave}>保存</button>
        </div>
      </nav>

      {#if tab === "params"}
        <ParamPanel />
      {:else if workspace.selected}
        {#key workspace.selected}
          <RawEditor
            initial={workspace.draft ?? workspace.baseText ?? ""}
            onChange={(text) => void setRaw(text)}
          />
        {/key}
      {:else}
        <div class="placeholder">从左侧选择一个脚本</div>
      {/if}

      <LogTerminal />
    </main>
  </div>

  <StatusBar />
</div>

{#if workspace.pending}
  <DiffDialog
    oldText={workspace.pending.oldText}
    newText={workspace.pending.newText}
    onConfirm={() => void commitSave()}
    onCancel={cancelSave}
  />
{/if}

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .tabs {
    height: 44px;
    flex: 0 0 44px;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .tab {
    font: inherit;
    font-size: 13px;
    color: var(--muted);
    background: none;
    border: none;
    padding: 7px 12px;
    border-radius: var(--radius);
    cursor: pointer;
  }
  .tab.active {
    color: var(--text);
    background: var(--surface-3);
  }
  .tabs .right {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .err {
    max-width: 380px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--danger);
  }
  .placeholder {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--faint);
    font-size: 12px;
  }
</style>
