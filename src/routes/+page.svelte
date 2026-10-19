<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import ScriptList from "$lib/components/ScriptList.svelte";
  import ParamPanel from "$lib/components/ParamPanel.svelte";
  import LogTerminal from "$lib/components/LogTerminal.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import { initWorkspace } from "$lib/stores/workspace.svelte";

  onMount(() => {
    // 启动时恢复上次工作目录并扫描脚本
    void initWorkspace();
  });
</script>

<div class="app">
  <TopBar />

  <div class="body">
    <ScriptList />

    <main class="main">
      <nav class="tabs">
        <button class="tab active" type="button">参数面板</button>
        <button class="tab" type="button">原始脚本</button>
        <div class="right">
          <button class="ghost" type="button">保存</button>
        </div>
      </nav>

      <ParamPanel />
      <LogTerminal />
    </main>
  </div>

  <StatusBar />
</div>

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
    gap: 8px;
  }
</style>
