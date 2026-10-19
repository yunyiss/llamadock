<script lang="ts">
  // 顶栏：品牌 + 工作目录 + 引擎版本切换。P0 为静态占位，P1/P4 接入真实数据。
  let workDir = "D:\\llama\\scripts";
  let engine = "b4000-cuda";
</script>

<header class="topbar">
  <div class="brand"><span class="logo">L</span> llama.cpp 中控台</div>

  <div class="workdir">
    <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path d="M1.5 2h5l1.2 1.5h6.8A1 1 0 0 1 16 4.5v8a1 1 0 0 1-1 1H1a1 1 0 0 1-1-1v-9A1 1 0 0 1 1.5 2z" />
    </svg>
    <span class="path">{workDir}</span>
  </div>

  <div class="spacer"></div>

  <div class="engine">
    引擎 <span class="tag">{engine}</span> <span class="caret">▾</span>
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
    font-size: 12px;
    color: var(--muted);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 5px 10px;
    font-family: var(--mono);
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
