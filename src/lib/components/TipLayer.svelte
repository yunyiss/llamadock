<script lang="ts">
  // 说明气泡渲染层：挂在页面顶层，逐帧读取锚点位置，
  // 因此滚动、容器裁剪、父级样式都不会影响它。
  import { closeTip, tipState } from "$lib/popover.svelte";

  const WIDTH = 340;

  const tip = $derived(tipState());
  let panel = $state<HTMLDivElement | null>(null);
  let pos = $state({ x: 0, y: 0 });

  function place(anchor: HTMLElement) {
    const rect = anchor.getBoundingClientRect();
    const height = panel?.offsetHeight ?? 120;
    const maxX = Math.max(8, window.innerWidth - WIDTH - 12);
    const x = Math.min(Math.max(8, rect.left), maxX);
    const y =
      rect.bottom + 6 + height > window.innerHeight
        ? Math.max(8, rect.top - height - 6)
        : rect.bottom + 6;
    pos = { x, y };
  }

  $effect(() => {
    const active = tip;
    if (!active) return;

    // 位置跟随：布局在无滚动情况下发生变化（拖动日志面板、窗口缩放）时重新对齐
    let raf = 0;
    const track = () => {
      place(active.anchor);
      raf = requestAnimationFrame(track);
    };
    raf = requestAnimationFrame(track);

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") closeTip();
    };
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      if (active.anchor.contains(target) || panel?.contains(target)) return;
      closeTip();
    };
    // 内部滚动容器（如参数面板）的 scroll 不冒泡，必须用 capture 才能收到。
    // 内容一旦滚动，锚点可能被容器裁掉、但它的矩形仍在视口内，只有关闭气泡
    // 才不会让气泡停在错误位置压住顶栏 / 标签栏 / 面板头部。
    const onScroll = (event: Event) => {
      const target = event.target as Node | null;
      if (!target) return;
      if (panel?.contains(target)) return; // 气泡自身内部滚动时不关闭
      if (!target.contains(active.anchor)) return; // 与锚点无关的滚动（如日志面板）不关闭
      closeTip();
    };

    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("pointerdown", onPointerDown, true);
    document.addEventListener("scroll", onScroll, true);

    return () => {
      cancelAnimationFrame(raf);
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("pointerdown", onPointerDown, true);
      document.removeEventListener("scroll", onScroll, true);
    };
  });
</script>

{#if tip}
  <div
    class="tip"
    bind:this={panel}
    style="left:{pos.x}px; top:{pos.y}px; width:{WIDTH}px"
    role="tooltip"
  >
    {tip.text}
  </div>
{/if}

<style>
  .tip {
    position: fixed;
    z-index: 60;
    max-height: 320px;
    overflow: auto;
    padding: 10px 12px;
    background: var(--surface-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.6);
    font-size: 12px;
    line-height: 18px;
    color: var(--text);
  }
</style>
