<script lang="ts">
  import { chooseWorkDir, workspace } from "$lib/stores/workspace.svelte";
  import { cycleTheme, theme, themeLabel } from "$lib/theme.svelte";
</script>

<header class="topbar">
  <div class="brand"><img class="logo" src="/app-icon.png" alt="" /> LlamaDock</div>

  <button class="workdir" type="button" onclick={chooseWorkDir} title="选择工作目录">
    <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path d="M1.5 2h5l1.2 1.5h6.8A1 1 0 0 1 16 4.5v8a1 1 0 0 1-1 1H1a1 1 0 0 1-1-1v-9A1 1 0 0 1 1.5 2z" />
    </svg>
    <span class="path">{workspace.workDir ?? "未选择工作目录"}</span>
  </button>

  <div class="spacer"></div>

  {#if workspace.selected}
    <div class="current" title={workspace.selected}>{workspace.selected}</div>
  {/if}

  <button class="theme" type="button" title={themeLabel()} aria-label={themeLabel()} onclick={cycleTheme}>
    {#if theme.mode === "auto"}
      <!-- 跟随系统 -->
      <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
        <path
          d="M1.75 2h12.5c.966 0 1.75.784 1.75 1.75v7.5A1.75 1.75 0 0 1 14.25 13H1.75A1.75 1.75 0 0 1 0 11.25v-7.5C0 2.784.784 2 1.75 2ZM1.5 3.75v7.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25H1.75a.25.25 0 0 0-.25.25ZM4 14.5a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5a.75.75 0 0 1-.75-.75Z"
        />
      </svg>
    {:else if theme.mode === "light"}
      <!-- 浅色 -->
      <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
        <path
          d="M8 12a4 4 0 1 1 0-8 4 4 0 0 1 0 8Zm0-1.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Zm0-8.25a.75.75 0 0 1 .75.75v1a.75.75 0 0 1-1.5 0v-1A.75.75 0 0 1 8 2.25Zm0 11.5a.75.75 0 0 1 .75.75v1a.75.75 0 0 1-1.5 0v-1a.75.75 0 0 1 .75-.75ZM3.5 8a.75.75 0 0 1-.75.75h-1a.75.75 0 0 1 0-1.5h1A.75.75 0 0 1 3.5 8Zm11.5.75a.75.75 0 0 1 0-1.5h1a.75.75 0 0 1 0 1.5h-1ZM5.404 3.343a.75.75 0 0 1 1.06 0l.708.708a.75.75 0 1 1-1.06 1.06l-.708-.707a.75.75 0 0 1 0-1.06Zm7.778 7.778a.75.75 0 0 1 1.06 0l.708.707a.75.75 0 1 1-1.06 1.06l-.708-.707a.75.75 0 0 1 0-1.06ZM3.343 10.596a.75.75 0 0 1 0 1.06l-.708.708a.75.75 0 0 1-1.06-1.06l.707-.708a.75.75 0 0 1 1.061 0Zm7.778-7.778a.75.75 0 0 1 0 1.06l-.707.708a.75.75 0 0 1-1.06-1.06l.707-.708a.75.75 0 0 1 1.06 0Z"
        />
      </svg>
    {:else}
      <!-- 深色 -->
      <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
        <path
          d="M9.598 1.591a.749.749 0 0 1 .785-.175 7.001 7.001 0 1 1-8.967 8.967.75.75 0 0 1 .961-.96 5.5 5.5 0 0 0 7.046-7.046.75.75 0 0 1 .175-.786Zm1.616 1.945a7 7 0 0 1-7.678 7.678 5.499 5.499 0 1 0 7.678-7.678Z"
        />
      </svg>
    {/if}
  </button>
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
    object-fit: cover;
    display: block;
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
    border-color: var(--border-strong);
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
  .current {
    max-width: 420px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
  }
  .theme {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    flex: 0 0 auto;
    color: var(--muted);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
  }
  .theme:hover {
    color: var(--text);
    border-color: var(--border-strong);
  }
</style>
