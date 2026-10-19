<script lang="ts">
  // 底部日志停靠区：xterm.js 渲染实时 stdout/stderr，支持自动滚动 / 暂停 / 清空 / 复制。
  import { onDestroy, onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import { onLog, onProcessState } from "$lib/events";

  let host: HTMLDivElement;
  let term: Terminal | undefined;
  const fit = new FitAddon();

  let autoScroll = $state(true);
  let paused = $state(false);
  let notice = $state("");

  let buffered: string[] = [];
  const unlisteners: Array<() => void> = [];

  function append(text: string) {
    if (!term) return;
    if (paused) {
      if (buffered.length < 5000) buffered.push(text);
      return;
    }
    term.write(text, () => {
      if (autoScroll) term?.scrollToBottom();
    });
  }

  function togglePause() {
    paused = !paused;
    if (!paused && term) {
      const text = buffered.join("");
      buffered = [];
      if (text) {
        term.write(text, () => {
          if (autoScroll) term?.scrollToBottom();
        });
      }
    }
  }

  function clearLog() {
    buffered = [];
    term?.clear();
  }

  async function copyLog() {
    if (!term) return;
    term.selectAll();
    const text = term.getSelection();
    term.clearSelection();
    try {
      await navigator.clipboard.writeText(text);
      notice = "已复制日志";
    } catch {
      notice = "复制失败，请手动选择复制";
    }
  }

  onMount(() => {
    term = new Terminal({
      fontSize: 12,
      fontFamily: '"Cascadia Code", Consolas, "SF Mono", ui-monospace, monospace',
      convertEol: false,
      scrollback: 5000,
      theme: { background: "#12161d", foreground: "#8b95a7", cursor: "#8b95a7" },
    });
    term.loadAddon(fit);
    term.open(host);
    fit.fit();

    const observer = new ResizeObserver(() => {
      try {
        fit.fit();
      } catch {
        /* 容器尺寸暂不可用时忽略 */
      }
    });
    observer.observe(host);

    void (async () => {
      unlisteners.push(
        await onLog((payload) => append(payload.data)),
        await onProcessState((payload) => {
          if (payload.state === "running") {
            buffered = [];
            notice = "";
            term?.reset();
          } else {
            notice = payload.code === null ? "进程已结束" : `进程已结束（退出码 ${payload.code}）`;
          }
        }),
      );
    })();

    return () => observer.disconnect();
  });

  onDestroy(() => {
    unlisteners.forEach((off) => off());
    term?.dispose();
  });
</script>

<section class="dock">
  <div class="head">
    <span class="title">日志</span>
    {#if notice}<span class="notice">{notice}</span>{/if}
    <div class="right">
      <button
        class="chip"
        class:on={autoScroll}
        type="button"
        onclick={() => (autoScroll = !autoScroll)}
      >
        自动滚动
      </button>
      <button class="chip" class:on={paused} type="button" onclick={togglePause}>
        {paused ? "继续" : "暂停"}
      </button>
      <button class="chip" type="button" onclick={clearLog}>清空</button>
      <button class="chip" type="button" onclick={() => void copyLog()}>复制</button>
    </div>
  </div>
  <div class="log" bind:this={host}></div>
</section>

<style>
  .dock {
    flex: 0 0 190px;
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
  .notice {
    color: var(--faint);
    font-size: 11px;
  }
  .right {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .log {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    padding: 6px 8px;
  }
  .log :global(.xterm) {
    height: 100%;
  }
</style>
