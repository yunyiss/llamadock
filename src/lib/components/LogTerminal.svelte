<script lang="ts">
  // 底部日志停靠区：xterm.js 渲染；按脚本分流日志；可拖动调整高度 / 收起；ctrl+点击链接用浏览器打开。
  import { onMount, untrack } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import "@xterm/xterm/css/xterm.css";
  import { clearLogs, logPanel, logsTick, readLogs, workspace } from "$lib/stores/workspace.svelte";
  import { newCursor, type LogCursor } from "$lib/log-buffer";
  import { openUrl } from "$lib/ipc";

  const COLLAPSED = 40;
  const MIN_HEIGHT = 40;
  const DEFAULT_HEIGHT = 200;

  let host: HTMLDivElement;
  let term: Terminal | undefined;
  let fit: FitAddon | undefined;

  // 打开应用时默认收起；启动脚本后由 logPanel.expandTick 触发弹出
  let height = $state(COLLAPSED);
  let collapsed = $state(true);
  let savedHeight = DEFAULT_HEIGHT;
  let dragging = $state(false);
  let atBottom = $state(true);
  let notice = $state("");

  // 每个脚本的消费游标：按块序号推进（长度变短不再等于"内容回退"，因此不再清屏重写）
  let cursor: LogCursor = newCursor();
  let shownFor: string | null = null;
  let follow = true;

  // 写入节流：洪泛期不再每个后端批次都写一次，攒到 WRITE_INTERVAL 合并写入
  const WRITE_INTERVAL = 150;
  let writeTimer: ReturnType<typeof setTimeout> | null = null;

  /** 消费新日志并写入终端（节流到点的回调）。 */
  function flushLogs() {
    writeTimer = null;
    const path = workspace.selected;
    if (!term || !path || collapsed) return;

    const { text, cursor: next } = readLogs(path, cursor);
    cursor = next;
    if (!text) return;

    const wantFollow = follow;
    term.write(text, () => {
      if (wantFollow) term?.scrollToBottom();
    });
  }

  function scheduleFlush() {
    if (writeTimer !== null) return;
    writeTimer = setTimeout(flushLogs, WRITE_INTERVAL);
  }

  // 订阅日志信号（追加 / 清空 / 淘汰都会触发）
  const tick = $derived(logsTick());

  $effect(() => {
    const path = workspace.selected;
    const signal = tick;
    const folded = collapsed;

    if (!term) return;
    if (path !== shownFor) {
      // 切换脚本：先清屏，再立刻补齐该脚本已有日志（不等节流）
      shownFor = path;
      cursor = newCursor();
      term.reset();
      flushLogs();
      return;
    }
    // 折叠时先不消费，展开后一次性补齐（在 40px 高度里渲染每条日志毫无意义）
    if (!path || folded || !signal) return;
    scheduleFlush();
  });

  onMount(() => {
    // FitAddon 必须与 Terminal 同生命周期：放在模块级会在重新挂载时残留已销毁的实例引用
    fit = new FitAddon();
    term = new Terminal({
      fontSize: 12,
      fontFamily: '"Cascadia Code", Consolas, "SF Mono", ui-monospace, monospace',
      convertEol: false,
      scrollback: 1500,
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

    const scrollSub = term.onScroll(() => {
      const buf = term!.buffer.active;
      follow = buf.viewportY >= buf.baseY;
      atBottom = follow;
    });

    const observer = new ResizeObserver(() => {
      // 折叠时不重排：在 40px 高度里重排整个 scrollback 毫无意义，展开时再对齐
      if (collapsed) return;
      try {
        fit?.fit();
      } catch {
        /* 尺寸暂不可用时忽略 */
      }
    });
    observer.observe(host);

    // 卸载时全部释放，避免切换脚本 / 重载后残留（定时器、终端、滚动订阅、尺寸观察器）
    return () => {
      if (writeTimer !== null) {
        clearTimeout(writeTimer);
        writeTimer = null;
      }
      scrollSub.dispose();
      observer.disconnect();
      term?.dispose();
      term = undefined;
      shownFor = null;
      cursor = newCursor();
    };
  });

  function onGripDown(event: MouseEvent) {
    event.preventDefault();
    dragging = true;
    const startY = event.clientY;
    const startHeight = height;

    const move = (e: MouseEvent) => {
      const max = Math.max(MIN_HEIGHT, window.innerHeight * 0.85);
      height = Math.min(Math.max(startHeight + (startY - e.clientY), MIN_HEIGHT), max);
      collapsed = false;
      savedHeight = height;
    };
    const up = () => {
      dragging = false;
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      refit();
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  /** 展开后等布局落定再对齐一次（折叠期间跳过了重排）。 */
  function refit() {
    requestAnimationFrame(() => {
      try {
        fit?.fit();
      } catch {
        /* 尺寸暂不可用时忽略 */
      }
    });
  }

  function expand() {
    if (!collapsed) return;
    collapsed = false;
    height = Math.max(savedHeight, DEFAULT_HEIGHT);
    refit();
  }

  function collapse() {
    if (collapsed) return;
    savedHeight = height;
    collapsed = true;
    height = COLLAPSED;
  }

  function toggleCollapse() {
    if (collapsed) expand();
    else collapse();
  }

  // 启动脚本后自动弹出（打开应用时保持收起状态）
  $effect(() => {
    const tick = logPanel.expandTick;
    if (!tick) return;
    untrack(expand);
  });

  function jumpToBottom() {
    term?.scrollToBottom();
    follow = true;
    atBottom = true;
  }

  function clearLog() {
    if (workspace.selected) clearLogs(workspace.selected);
    term?.reset();
    cursor = newCursor();
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

<section class="dock" class:dragging style:flex-basis="{height}px">
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
    flex: 0 0 auto;
    min-height: 0;
    /* 展开 / 收起的高度动画；拖动时关闭以免跟手延迟 */
    transition: flex-basis 220ms cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
  }
  .dock.dragging {
    transition: none;
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
