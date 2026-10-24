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
    cancelSwitch,
    commitSave,
    discardAndSwitch,
    initProcessEvents,
    initWorkspace,
    isDirty,
    keepAndSwitch,
    prepareSave,
    setRaw,
    workspace,
  } from "$lib/stores/workspace.svelte";
  import { initTheme } from "$lib/theme.svelte";

  let tab = $state<"params" | "raw">("params");
  const dirty = $derived(isDirty());
  const currentName = $derived(workspace.selected?.split(/[\\/]/).pop() ?? "");

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
      {#if !workspace.workDir}
        <div class="welcome">
          <h2>开始使用 LlamaDock</h2>
          <p class="lead">
            LlamaDock 管理本地的 llama.cpp 启动脚本（<code>.cmd</code> / <code>.bat</code> / <code>.sh</code> / <code>.ps1</code>）：可视化编辑参数、切换引擎版本、在工具内启动并查看实时日志。
          </p>
          <ol>
            <li>在左侧点选脚本，右侧会列出它已有的参数</li>
            <li>点底部「启动」，日志会在下方自动展开</li>
          </ol>
          <p class="hint">配置文件在当前路径下（llamadock.config.json）</p>
        </div>
      {:else}
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

{#if workspace.switchPrompt}
  <div class="overlay">
    <div class="dialog-sm" role="dialog" aria-modal="true" aria-label="切换脚本">
      <p class="ask">
        「{currentName}」还有未保存的改动。切换到「{workspace.switchPrompt.name}」前要保留这些改动吗？
      </p>
      <p class="ask-tip">选择「保留改动」会先记在本地，切回该脚本时自动恢复，不会写盘。</p>
      <div class="ask-foot">
        <button class="ghost" type="button" onclick={cancelSwitch}>取消</button>
        <button class="ghost" type="button" onclick={discardAndSwitch}>放弃改动</button>
        <button class="btn" type="button" onclick={keepAndSwitch}>保留改动并切换</button>
      </div>
    </div>
  </div>
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
  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 12px;
    max-width: 520px;
    margin: 0 auto;
    padding: 32px 24px;
  }
  .welcome h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }
  .welcome .lead {
    color: var(--muted);
    font-size: 13px;
    line-height: 20px;
  }
  .welcome ol {
    margin: 4px 0 8px;
    padding-left: 20px;
    color: var(--muted);
    font-size: 13px;
    line-height: 22px;
  }
  .welcome code {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text);
    background: var(--surface-3);
    border-radius: 4px;
    padding: 1px 5px;
  }
  .welcome .hint {
    color: var(--faint);
    font-size: 11px;
  }
  .overlay {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 20;
  }
  .dialog-sm {
    width: min(420px, 88vw);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 18px 20px 14px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.35);
  }
  .ask {
    font-size: 13px;
    line-height: 20px;
    color: var(--text);
  }
  .ask-tip {
    margin-top: 8px;
    font-size: 11px;
    line-height: 16px;
    color: var(--faint);
  }
  .ask-foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
