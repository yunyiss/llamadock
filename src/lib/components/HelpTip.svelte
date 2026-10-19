<script lang="ts">
  // 「?」气泡说明：点击弹出，点其他任意地方收起。
  let { text }: { text: string } = $props();

  let btn: HTMLButtonElement;
  let pop = $state<{ x: number; y: number } | null>(null);

  const WIDTH = 340;

  function toggle() {
    if (pop) {
      pop = null;
      return;
    }
    const rect = btn.getBoundingClientRect();
    pop = {
      x: Math.min(rect.left, Math.max(8, window.innerWidth - WIDTH - 12)),
      y: Math.min(rect.bottom + 6, Math.max(8, window.innerHeight - 150)),
    };
  }
</script>

<button
  bind:this={btn}
  class="help"
  class:on={!!pop}
  type="button"
  title="参数说明"
  aria-label="参数说明"
  onclick={toggle}
>?</button>

{#if pop}
  <button class="pop-backdrop" type="button" aria-label="关闭说明" onclick={() => (pop = null)}></button>
  <div class="pop" style="left:{pop.x}px; top:{pop.y}px">{text}</div>
{/if}

<style>
  .help {
    width: 15px;
    height: 15px;
    flex: 0 0 auto;
    padding: 0;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--muted);
    font-size: 10px;
    line-height: 1;
    cursor: pointer;
  }
  .help:hover,
  .help.on {
    color: var(--accent);
    border-color: var(--accent);
  }
  .pop-backdrop {
    position: fixed;
    inset: 0;
    z-index: 39;
    padding: 0;
    border: none;
    background: transparent;
  }
  .pop {
    position: fixed;
    z-index: 40;
    width: 340px;
    padding: 10px 12px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    font-size: 12px;
    line-height: 18px;
    color: var(--text);
  }
</style>
