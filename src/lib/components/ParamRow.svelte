<script lang="ts">
  // 单行参数：布尔用开关，其余用输入框；改动通过回调上报。
  import type { ParamView } from "$lib/types";

  let {
    param,
    onChange,
  }: {
    param: ParamView;
    onChange: (present: boolean, value: string | null) => void;
  } = $props();

  const isOn = $derived(param.present && param.value !== "off");

  function toggle() {
    onChange(true, isOn ? "off" : "on");
  }

  function remove() {
    onChange(false, null);
  }
</script>

<div class="row" class:absent={!param.present}>
  <div class="label">
    <div class="t">{param.desc}</div>
    <div class="d"><span class="flag">{param.flags.join(" · ")}</span></div>
  </div>
  <div class="ctrl">
    {#if param.kind === "bool"}
      <button
        class="switch"
        class:off={!isOn}
        type="button"
        aria-label="切换 {param.desc}"
        onclick={toggle}
      ></button>
    {:else}
      <input
        class="input"
        class:num={param.kind === "number"}
        value={param.value ?? ""}
        onchange={(e) => onChange(true, (e.currentTarget as HTMLInputElement).value)}
        spellcheck="false"
      />
    {/if}
    {#if param.present}
      <button class="remove" type="button" title="移除该参数" onclick={remove}>×</button>
    {/if}
  </div>
</div>

<style>
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
  .row.absent {
    opacity: 0.55;
  }
  .label {
    width: 220px;
    flex: 0 0 220px;
  }
  .label .t {
    font-size: 13px;
  }
  .label .d {
    margin-top: 2px;
    font-size: 11px;
    color: var(--faint);
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
  .input.num {
    max-width: 120px;
  }
  .remove {
    padding: 2px 6px;
    border: none;
    border-radius: var(--radius);
    background: none;
    color: var(--faint);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
  }
  .remove:hover {
    color: var(--danger);
    background: var(--surface-2);
  }
  .switch {
    width: 38px;
    height: 22px;
    padding: 0;
    border: none;
    border-radius: 999px;
    background: var(--brand);
    position: relative;
    flex: 0 0 auto;
    cursor: pointer;
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
