<script lang="ts">
  // 底部状态栏：跟随当前脚本的进程状态 / PID / 端口 / 引擎，以及启动与停止。
  import { canRun, currentPort, killScript, runScript, runtimeOf, workspace } from "$lib/stores/workspace.svelte";

  const rt = $derived(runtimeOf(workspace.selected));
  const running = $derived(rt.state === "running");
  const runnable = $derived(canRun());
  const port = $derived(currentPort());
  const engineName = $derived(workspace.model?.launcher?.exe?.split(/[\\/]/).pop() ?? null);

  const tone = $derived(
    running
      ? "run"
      : rt.state === "idle"
        ? "idle"
        : rt.stopRequested || !rt.exitCode
          ? "stop"
          : "err",
  );

  const statusText = $derived(
    running
      ? "运行中"
      : rt.state === "idle"
        ? "未启动"
        : rt.stopRequested || !rt.exitCode
          ? "已停止"
          : `异常退出（退出码 ${rt.exitCode}）`,
  );

  const dotClass = $derived(tone === "run" ? "on" : tone === "err" ? "err" : "off");
</script>

<footer class="statusbar">
  <span class="st {tone}"><span class="dot {dotClass}"></span> {statusText}</span>

  {#if rt.pid !== null}
    <span class="kv">PID <b>{rt.pid}</b></span>
  {/if}
  {#if port}
    <span class="kv">端口 <b>{port}</b></span>
  {/if}
  {#if engineName}
    <span class="kv">引擎 <b>{engineName}</b></span>
  {/if}

  <div class="right">
    {#if running}
      <button class="btn danger" type="button" onclick={killScript}>停止</button>
    {:else}
      <button class="btn" type="button" disabled={!runnable} onclick={runScript}>
        {runnable ? "启动" : "无法运行 .sh"}
      </button>
    {/if}
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
    font-weight: 600;
  }
  .st.run {
    color: var(--brand);
  }
  .st.stop {
    color: var(--muted);
  }
  .st.idle {
    color: var(--faint);
  }
  .st.err {
    color: var(--danger);
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
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
