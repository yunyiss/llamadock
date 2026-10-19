<script lang="ts">
  import {
    addEngine,
    applyEngineToAll,
    chooseWorkDir,
    dropEngine,
    useEngine,
    workspace,
  } from "$lib/stores/workspace.svelte";

  let open = $state(false);

  const current = $derived(
    workspace.engines.find((e) => e.name === workspace.currentEngine) ?? null,
  );
  const scriptExe = $derived(workspace.model?.launcher?.exe ?? null);
  const label = $derived(current?.name ?? scriptExe?.split(/[\\/]/).pop() ?? "未识别");
  const currentInvalid = $derived(
    current ? workspace.invalidEngines.includes(current.name) : false,
  );

  function pick(name: string) {
    open = false;
    void useEngine(name, true);
  }

  function batch() {
    open = false;
    if (current && scriptExe) void applyEngineToAll(scriptExe, current.path);
  }
</script>

<header class="topbar">
  <div class="brand"><span class="logo">L</span> llama.cpp 中控台</div>

  <button class="workdir" type="button" onclick={chooseWorkDir} title="选择工作目录">
    <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path d="M1.5 2h5l1.2 1.5h6.8A1 1 0 0 1 16 4.5v8a1 1 0 0 1-1 1H1a1 1 0 0 1-1-1v-9A1 1 0 0 1 1.5 2z" />
    </svg>
    <span class="path">{workspace.workDir ?? "未选择工作目录"}</span>
  </button>

  <div class="spacer"></div>

  <div class="engine-wrap">
    <button
      class="engine"
      class:invalid={currentInvalid}
      type="button"
      onclick={() => (open = !open)}
    >
      引擎 <span class="tag">{label}</span>
      <span class="caret">▾</span>
    </button>

    {#if open}
      <button class="backdrop" type="button" aria-label="关闭菜单" onclick={() => (open = false)}
      ></button>
      <div class="menu">
        {#if workspace.engines.length === 0}
          <div class="empty">还没有保存的引擎版本</div>
        {/if}

        {#each workspace.engines as e (e.name)}
          <div class="row">
            <button class="item" type="button" onclick={() => pick(e.name)}>
              <span class="dot" class:on={workspace.currentEngine === e.name} class:off={workspace.currentEngine !== e.name}></span>
              <span class="info">
                <span class="name">
                  {#if workspace.invalidEngines.includes(e.name)}⚠ {/if}{e.name}
                </span>
                <span class="path" title={e.path}>{e.path}</span>
              </span>
            </button>
            <button class="del" type="button" title="移除该版本" onclick={() => void dropEngine(e.name)}>×</button>
          </div>
        {/each}

        <div class="sep"></div>
        <button class="action" type="button" onclick={() => void addEngine()}>
          ＋ 从文件添加引擎…
        </button>
        {#if current && scriptExe && scriptExe !== current.path}
          <button class="action" type="button" onclick={batch}>
            批量应用：切到「{current.name}」
          </button>
        {/if}
      </div>
    {/if}
  </div>
</header>

<style>
  .topbar {
    height: 52px;
    flex: 0 0 52px;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    letter-spacing: 0.2px;
  }
  .logo {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: linear-gradient(135deg, var(--brand), #2c8a3c);
    display: grid;
    place-items: center;
    font-size: 12px;
    color: #06210e;
    font-weight: 800;
  }
  .workdir {
    display: flex;
    align-items: center;
    gap: 8px;
    max-width: 420px;
    font: inherit;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--muted);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 5px 10px;
    cursor: pointer;
  }
  .workdir:hover {
    border-color: #33405a;
  }
  .workdir svg {
    flex: 0 0 auto;
    opacity: 0.7;
  }
  .workdir .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .spacer {
    flex: 1;
  }
  .engine-wrap {
    position: relative;
  }
  .engine {
    display: flex;
    align-items: center;
    gap: 8px;
    font: inherit;
    font-size: 12px;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 5px 10px;
    cursor: pointer;
  }
  .engine:hover {
    border-color: #33405a;
  }
  .engine.invalid {
    border-color: var(--warn);
  }
  .tag {
    background: var(--brand-soft);
    color: var(--brand);
    border-radius: 999px;
    padding: 1px 8px;
    font-size: 11px;
    font-weight: 600;
  }
  .caret {
    color: var(--faint);
  }
  .backdrop {
    position: fixed;
    inset: 0;
    border: none;
    background: transparent;
    padding: 0;
    cursor: default;
    z-index: 20;
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 21;
    width: 360px;
    padding: 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
  }
  .empty {
    padding: 12px;
    color: var(--faint);
    font-size: 12px;
    text-align: center;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .item {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
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
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    display: block;
    font-size: 13px;
  }
  .path {
    display: block;
    margin-top: 1px;
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .del {
    padding: 2px 6px;
    border: none;
    border-radius: var(--radius);
    background: none;
    color: var(--faint);
    font-size: 14px;
    cursor: pointer;
  }
  .del:hover {
    color: var(--danger);
    background: var(--surface-2);
  }
  .sep {
    height: 1px;
    margin: 6px 4px;
    background: var(--border);
  }
  .action {
    display: block;
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: var(--radius);
    background: none;
    font: inherit;
    font-size: 12px;
    color: var(--muted);
    text-align: left;
    cursor: pointer;
  }
  .action:hover {
    color: var(--text);
    background: var(--surface-2);
  }
</style>
