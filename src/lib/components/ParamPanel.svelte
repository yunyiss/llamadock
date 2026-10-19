<script lang="ts">
  // 参数面板内容区。P0 为静态占位，P2 起由解析结果驱动渲染。
  type Row = {
    label: string;
    flags: string;
    hint?: string;
    kind: "path" | "number" | "string" | "bool";
    value?: string;
    on?: boolean;
  };
  type Group = { title: string; rows: Row[] };

  const groups: Group[] = [
    {
      title: "模型",
      rows: [
        {
          label: "模型路径",
          flags: "-m --model",
          hint: "模型文件路径",
          kind: "path",
          value: "D:\\models\\Qwen2.5-32B-Instruct-Q4_K_M.gguf",
        },
        { label: "上下文长度", flags: "-c --ctx-size", hint: "越大越吃显存", kind: "number", value: "4096" },
      ],
    },
    {
      title: "服务",
      rows: [
        { label: "监听地址", flags: "--host", kind: "string", value: "127.0.0.1" },
        { label: "端口", flags: "--port", kind: "number", value: "8080" },
        { label: "并发数", flags: "--parallel", kind: "number", value: "4" },
      ],
    },
    {
      title: "性能 / GPU",
      rows: [
        { label: "CPU 线程数", flags: "-t --threads", kind: "number", value: "8" },
        { label: "GPU 卸载层数", flags: "-ngl --n-gpu-layers", kind: "number", value: "99" },
        { label: "Flash Attention", flags: "--flash-attn", kind: "bool", on: true },
      ],
    },
  ];
</script>

<div class="content">
  {#each groups as g}
    <section class="group">
      <div class="group-title">{g.title}</div>
      <div class="card">
        {#each g.rows as r}
          <div class="row">
            <div class="label">
              <div class="t">{r.label}</div>
              <div class="d">
                <span class="flag">{r.flags}</span>{#if r.hint}<span class="hint"> · {r.hint}</span>{/if}
              </div>
            </div>
            <div class="ctrl">
              {#if r.kind === "bool"}
                <span class="switch" class:off={!r.on}></span>
              {:else}
                <span class="field" class:num={r.kind === "number"}>{r.value}</span>
                {#if r.kind === "path"}
                  <button class="btn" type="button">选择文件</button>
                {/if}
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .content {
    flex: 1;
    overflow: auto;
    padding: 16px;
    min-height: 0;
  }
  .group {
    margin-bottom: 16px;
  }
  .group-title {
    margin-bottom: 8px;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child {
    border-bottom: none;
  }
  .label {
    width: 220px;
    flex: 0 0 220px;
  }
  .label .t {
    font-size: 13px;
  }
  .label .d {
    font-size: 11px;
    color: var(--faint);
    margin-top: 2px;
  }
  .flag {
    font-family: var(--mono);
    color: var(--accent);
    opacity: 0.85;
  }
  .ctrl {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .field {
    flex: 1;
    min-width: 0;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 10px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .field.num {
    flex: 0 0 auto;
    max-width: 120px;
  }
  .switch {
    width: 38px;
    height: 22px;
    border-radius: 999px;
    background: var(--brand);
    position: relative;
    flex: 0 0 auto;
  }
  .switch::after {
    content: "";
    position: absolute;
    top: 3px;
    right: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #06210e;
  }
  .switch.off {
    background: var(--surface-3);
  }
  .switch.off::after {
    right: auto;
    left: 3px;
    background: var(--muted);
  }
</style>
