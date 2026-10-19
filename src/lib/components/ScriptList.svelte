<script lang="ts">
  // 左侧脚本列表。P0 为静态占位，P1 接入真实扫描结果。
  type Item = { name: string; sub: string; badge: string; state: "on" | "off" };

  const scripts: Item[] = [
    { name: "qwen2.5-32b.cmd", sub: "运行中 · PID 12345", badge: "CUDA", state: "on" },
    { name: "llama3-8b.sh", sub: "未启动", badge: "Vulkan", state: "off" },
    { name: "deepseek-r1-q4.cmd", sub: "未启动", badge: "CUDA", state: "off" },
    { name: "qwen2.5-coder-7b.cmd", sub: "已停止 · 退出码 0", badge: "CUDA", state: "off" },
  ];

  let active = 0;
</script>

<aside class="sidebar">
  <div class="head">
    <input class="search" type="search" placeholder="搜索脚本…" />
  </div>

  <div class="list">
    {#each scripts as s, i}
      <div class="item" class:active={i === active}>
        <span class="dot {s.state}"></span>
        <div class="meta">
          <div class="name">{s.name}</div>
          <div class="sub">{s.sub}</div>
        </div>
        <span class="badge">{s.badge}</span>
      </div>
    {/each}
  </div>

  <div class="foot">
    <button class="ghost" type="button">＋ 新增</button>
    <button class="ghost" type="button">刷新</button>
    <button class="ghost" type="button">打开文件夹</button>
  </div>
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
    padding: 8px 10px;
    border-radius: var(--radius);
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
  }
  .name {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    font-size: 11px;
    color: var(--faint);
    margin-top: 1px;
  }
  .foot {
    padding: 10px 12px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
