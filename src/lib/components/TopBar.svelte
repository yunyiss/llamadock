<script lang="ts">
  import { workspace, chooseWorkDir } from "$lib/stores/workspace.svelte";
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

  <div class="engine">
    引擎 <span class="tag">未识别</span> <span class="caret">▾</span>
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
  .engine {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 5px 10px;
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
</style>
