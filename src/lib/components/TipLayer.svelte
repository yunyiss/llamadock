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

    let raf = 0;
    const track = () => {
      const rect = active.anchor.getBoundingClientRect();
      // 锚点滚出可视区域：直接关闭，避免气泡停在顶部
      if (rect.bottom < 0 || rect.top > window.innerHeight) {
        closeTip();
        return;
      }
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

    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("pointerdown", onPointerDown, true);

    return () => {
      cancelAnimationFrame(raf);
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("pointerdown", onPointerDown, true);
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
    border: 1px solid #33405a;
    border-radius: var(--radius);
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.6);
    font-size: 12px;
    line-height: 18px;
    color: var(--text);
  }
</style>
