<script lang="ts">
  // 底部日志停靠区：xterm.js 渲染；按脚本分流日志；可拖动调整高度 / 收起；ctrl+点击链接用浏览器打开。
  import { onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import "@xterm/xterm/css/xterm.css";
  import { clearLogs, logs, workspace } from "$lib/stores/workspace.svelte";
  import { openUrl } from "$lib/ipc";

  const COLLAPSED = 40;
  const MIN_HEIGHT = 40;
  const DEFAULT_HEIGHT = 200;

  let host: HTMLDivElement;
  let term: Terminal | undefined;
  const fit = new FitAddon();

  let height = $state(DEFAULT_HEIGHT);
  let collapsed = $state(false);
  let savedHeight = DEFAULT_HEIGHT;
  let atBottom = $state(true);
  let notice = $state("");

  // 已经写给终端的进度（用于按脚本切换 / 增量追加）
  let shownFor: string | null = null;
  let shownLen = 0;
  let follow = true;

  const buffer = $derived(workspace.selected ? (logs.get(workspace.selected) ?? "") : "");

  $effect(() => {
    const path = workspace.selected;
    const text = buffer;
    if (!term) return;

    if (path !== shownFor) {
      shownFor = path;
      shownLen = 0;
      term.reset();
    }
    if (text.length < shownLen) {
      term.reset();
      shownLen = 0;
    }
    if (text.length > shownLen) {
      const chunk = text.slice(shownLen);
      shownLen = text.length;
      const wantFollow = follow;
      term.write(chunk, () => {
        if (wantFollow) term?.scrollToBottom();
      });
    }
  });

  onMount(() => {
    term = new Terminal({
      fontSize: 12,
      fontFamily: '"Cascadia Code", Consolas, "SF Mono", ui-monospace, monospace',
      convertEol: false,
      scrollback: 5000,
      theme: { background: "#12161d", foreground: "#8b95a7", cursor: "#8b95a7" },
    });
    term.loadAddon(fit);
    term.loadAddon(
      new WebLinksAddon((event: MouseEvent, uri: string) => {
        // 与 Windows Terminal 一致：按住 Ctrl 再点才打开
        if (!event.ctrlKey) return;
        void openUrl(uri).catch((e) => {
          notice = String(e);
        });
      }),
    );
    term.open(host);
    fit.fit();

    term.onScroll(() => {
      const buf = term!.buffer.active;
      follow = buf.viewportY >= buf.baseY;
      atBottom = follow;
    });

    const observer = new ResizeObserver(() => {
      try {
        fit.fit();
      } catch {
        /* 尺寸暂不可用时忽略 */
      }
    });
    observer.observe(host);

    return () => observer.disconnect();
  });

  function onGripDown(event: MouseEvent) {
    event.preventDefault();
    const startY = event.clientY;
    const startHeight = height;

    const move = (e: MouseEvent) => {
      const max = Math.max(MIN_HEIGHT, window.innerHeight * 0.85);
      height = Math.min(Math.max(startHeight + (startY - e.clientY), MIN_HEIGHT), max);
      collapsed = false;
    };
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  function toggleCollapse() {
    if (collapsed) {
      collapsed = false;
      height = savedHeight;
    } else {
      savedHeight = height;
      collapsed = true;
      height = COLLAPSED;
    }
  }

  function jumpToBottom() {
    term?.scrollToBottom();
    follow = true;
    atBottom = true;
  }

  function clearLog() {
    if (workspace.selected) clearLogs(workspace.selected);
    term?.reset();
    shownLen = 0;
    follow = true;
    atBottom = true;
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
</script>

<section class="dock" style="flex: 0 0 {height}px">
  <button
    class="grip"
    type="button"
    aria-label="拖动调整日志高度"
    onmousedown={onGripDown}
  ></button>

  <div class="head">
    <button class="chev" type="button" title="收起 / 展开日志" onclick={toggleCollapse}>
      {collapsed ? "▴" : "▾"}
    </button>
    <span class="title">日志</span>
    {#if notice}<span class="notice">{notice}</span>{/if}
    <div class="right">
      <button class="chip" type="button" onclick={clearLog}>清空</button>
      <button class="chip" type="button" onclick={() => void copyLog()}>复制</button>
    </div>
  </div>

  <div class="log-wrap">
    <div class="log" bind:this={host}></div>
    {#if !atBottom && !collapsed}
      <button class="jump" type="button" onclick={jumpToBottom}>↓ 回到最新</button>
    {/if}
  </div>
</section>

<style>
  .dock {
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .grip {
    height: 6px;
    flex: 0 0 6px;
    width: 100%;
    padding: 0;
    border: none;
    background: transparent;
    cursor: ns-resize;
  }
  .grip:hover {
    background: var(--border);
  }
  .head {
    height: 34px;
    flex: 0 0 34px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }
  .chev {
    padding: 0 4px;
    border: none;
    background: none;
    color: var(--muted);
    font-size: 12px;
    cursor: pointer;
  }
  .chev:hover {
    color: var(--text);
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
  .log-wrap {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .log {
    height: 100%;
    padding: 6px 8px;
  }
  .log :global(.xterm) {
    height: 100%;
  }
  .jump {
    position: absolute;
    right: 16px;
    bottom: 10px;
    padding: 4px 12px;
    border: 1px solid var(--accent);
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--accent);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
</style>
