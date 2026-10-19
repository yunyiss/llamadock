<script lang="ts">
  // 底部日志停靠区。P0 为静态占位，P5 接入 xterm.js 与实际日志流。
  type Line = { level: "info" | "plain"; text: string };

  const lines: Line[] = [
    { level: "info", text: "llama_model_loader: loaded meta data with 29 key-value pairs" },
    { level: "info", text: "llama_load_model_from_file: using device CUDA0 (NVIDIA RTX 4090, 24564 MiB)" },
    { level: "info", text: "llama_new_context_with_model: n_ctx = 4096, n_batch = 2048" },
    { level: "info", text: "llama_new_context_with_model: flash_attn = 1" },
    { level: "plain", text: "main: server is listening on http://127.0.0.1:8080" },
  ];
</script>

<section class="dock">
  <div class="head">
    <span class="title">日志</span>
    <div class="right">
      <button class="chip on" type="button">自动滚动</button>
      <button class="chip" type="button">暂停</button>
      <button class="chip" type="button">清空</button>
      <button class="chip" type="button">复制</button>
    </div>
  </div>
  <div class="log">
    {#each lines as l}
      <div class="line" class:hl={l.level === "plain"}>
        {#if l.level === "info"}<span class="lv">[info]</span>&nbsp;{/if}{l.text}
      </div>
    {/each}
  </div>
</section>

<style>
  .dock {
    flex: 0 0 158px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .head {
    height: 34px;
    flex: 0 0 34px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }
  .title {
    font-weight: 600;
  }
  .right {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .log {
    flex: 1;
    overflow: auto;
    padding: 10px 14px;
    font-family: var(--mono);
    font-size: 12px;
    line-height: 19px;
  }
  .line {
    white-space: pre;
    color: var(--muted);
  }
  .line.hl {
    color: var(--text);
  }
  .lv {
    color: var(--brand);
  }
</style>
