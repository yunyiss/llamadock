<script lang="ts">
  // 底部状态栏。P0 为静态占位，P5 接入真实进程状态 / PID / 端口。
</script>

<footer class="statusbar">
  <span class="st"><span class="dot on"></span> 运行中</span>
  <span class="kv">PID <b>12345</b></span>
  <span class="kv">端口 <b>8080</b></span>
  <span class="kv">引擎 <b>b4000-cuda</b></span>
  <div class="right">
    <button class="btn" type="button">重启</button>
    <button class="btn danger" type="button">停止</button>
  </div>
</footer>

<style>
  .statusbar {
    height: 40px;
    flex: 0 0 40px;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 14px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: 12px;
  }
  .st {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--brand);
    font-weight: 600;
  }
  .kv {
    color: var(--muted);
  }
  .kv b {
    color: var(--text);
    font-weight: 500;
    font-family: var(--mono);
  }
  .right {
    margin-left: auto;
    display: flex;
    gap: 8px;
  }
</style>
