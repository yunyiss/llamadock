<script lang="ts">
  // 「?」按钮：只负责开关全局说明气泡，气泡本身由顶层 TipLayer 渲染。
  import { closeTip, nextTipId, openTip, tipState } from "$lib/popover.svelte";

  let { text }: { text: string } = $props();

  const id = nextTipId();
  let btn: HTMLButtonElement;
  const open = $derived(tipState()?.id === id);

  function toggle() {
    if (open) {
      closeTip();
      return;
    }
    openTip({ id, text, anchor: btn });
  }
</script>

<button
  bind:this={btn}
  class="help"
  class:on={open}
  type="button"
  title="参数说明"
  aria-label="参数说明"
  aria-expanded={open}
  onclick={toggle}
>?</button>

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
</style>
