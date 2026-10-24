<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import ScriptList from "$lib/components/ScriptList.svelte";
  import ParamPanel from "$lib/components/ParamPanel.svelte";
  import { loadRawEditor } from "$lib/lazy";
  import LogTerminal from "$lib/components/LogTerminal.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import DiffDialog from "$lib/components/DiffDialog.svelte";
  import TipLayer from "$lib/components/TipLayer.svelte";
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
  import { initTheme } from "$lib/theme.svelte";

  let tab = $state<"params" | "raw">("params");
  const dirty = $derived(isDirty());

  let disposeEvents: (() => void) | null = null;

  onMount(() => {
    // 主题：首帧已由 app.html 内联脚本写好，这里对齐状态并监听系统主题
    void initTheme();
    // 启动时恢复上次工作目录、订阅进程事件
    void initWorkspace();

    let destroyed = false;
    void initProcessEvents().then((off) => {
      if (destroyed) off();
      else disposeEvents = off;
    });

    // 销毁时取消订阅，避免残留订阅（热更新 / 重载时尤其重要）
    return () => {
      destroyed = true;
      disposeEvents?.();
      disposeEvents = null;
    };
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
      {:else if !workspace.selected}
        <div class="placeholder">从左侧选择一个脚本</div>
      {:else if !workspace.baseText}
        <div class="placeholder">加载中…</div>
      {:else}
        {#await loadRawEditor()}
          <div class="placeholder">加载编辑器…</div>
        {:then RawEditor}
          {#key workspace.selected}
            <RawEditor
              initial={workspace.draft ?? workspace.baseText}
              onChange={(text) => void setRaw(text)}
            />
          {/key}
        {/await}
      {/if}

      <LogTerminal />
    </main>
  </div>

  <StatusBar />
  <TipLayer />
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
