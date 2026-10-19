<script lang="ts">
  import { workspace, refreshScripts, chooseWorkDir, selectScript } from "$lib/stores/workspace.svelte";
  import { openFolder } from "$lib/ipc";

  let query = $state("");

  const filtered = $derived(
    workspace.scripts.filter((s) => s.name.toLowerCase().includes(query.trim().toLowerCase())),
  );

  function openCurrent() {
    if (workspace.workDir) void openFolder(workspace.workDir);
  }
</script>

<aside class="sidebar">
  <div class="head">
    <input class="search" type="search" placeholder="搜索脚本…" bind:value={query} />
  </div>

  {#if !workspace.workDir}
    <div class="empty">
      <p>尚未选择工作目录</p>
      <button class="btn" type="button" onclick={chooseWorkDir}>选择工作目录</button>
    </div>
  {:else}
    {#if workspace.scriptsStale}
      <button class="hint" type="button" onclick={refreshScripts}>目录有变化，点击刷新</button>
    {/if}

    {#if workspace.error}
      <div class="banner">{workspace.error}</div>
    {/if}

    <div class="list">
      {#each filtered as s (s.path)}
        <button
          class="item"
          class:active={workspace.selected === s.path}
          type="button"
          onclick={() => selectScript(s.path)}
        >
          <span class="dot off"></span>
          <span class="meta">
            <span class="name">{s.name}</span>
            <span class="sub" title={s.path}>{s.path}</span>
          </span>
          <span class="badge">{s.ext.toUpperCase()}</span>
        </button>
      {/each}

      {#if filtered.length === 0}
        <div class="none">未找到脚本</div>
      {/if}
    </div>

    <div class="foot">
      <button class="ghost" type="button" onclick={refreshScripts}>刷新</button>
      <button class="ghost" type="button" onclick={openCurrent}>打开文件夹</button>
    </div>
  {/if}
</aside>

<style>
  .sidebar {
    width: 252px;
    flex: 0 0 252px;
    border-right: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .head {
    padding: 12px;
  }
  .search {
    width: 100%;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 10px;
    font-family: inherit;
    font-size: 12px;
    color: var(--text);
  }
  .search::placeholder {
    color: var(--faint);
  }
  .search:focus {
    outline: none;
    border-color: var(--accent);
  }
  .list {
    flex: 1;
    overflow: auto;
    padding: 4px 8px 8px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: var(--radius);
    background: none;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .item:hover {
    background: var(--surface-2);
  }
  .item.active {
    background: var(--surface-3);
    outline: 1px solid var(--border);
  }
  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    display: block;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    display: block;
    margin-top: 1px;
    font-size: 11px;
    color: var(--faint);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    color: var(--faint);
    font-size: 12px;
  }
  .none {
    padding: 16px 10px;
    text-align: center;
    color: var(--faint);
    font-size: 12px;
  }
  .banner {
    margin: 0 12px 8px;
    padding: 8px 10px;
    border: 1px solid var(--danger);
    border-radius: var(--radius);
    color: var(--danger);
    font-size: 12px;
  }
  .hint {
    display: block;
    width: calc(100% - 24px);
    margin: 0 12px 8px;
    padding: 7px 10px;
    background: var(--brand-soft);
    border: 1px solid var(--brand);
    border-radius: var(--radius);
    color: var(--brand);
    font: inherit;
    font-size: 12px;
    text-align: center;
    cursor: pointer;
  }
  .hint:hover {
    background: #17492a;
  }
  .foot {
    padding: 10px 12px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
