<script lang="ts">
  // 添加参数：搜索式选择器。第一项是「自定义参数」，其余是尚未出现在脚本里的已登记参数。
  import type { ParamView } from "$lib/types";
  import { APPEND_KEY } from "$lib/types";
  import { setParam } from "$lib/stores/workspace.svelte";
  import { GROUP_LABELS, GROUP_ORDER } from "$lib/params";
  import HelpTip from "./HelpTip.svelte";

  let { params, onClose }: { params: ParamView[]; onClose: () => void } = $props();

  let query = $state("");
  let picked = $state<ParamView | null>(null);
  let customRaw = $state("");
  let value = $state("");

  const matched = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const pool = params.filter((p) => !p.present);
    const hit = q
      ? pool.filter(
          (p) =>
            p.desc.toLowerCase().includes(q) ||
            p.key.toLowerCase().includes(q) ||
            p.flags.some((f) => f.toLowerCase().includes(q)),
        )
      : pool;
    return GROUP_ORDER.map((g) => ({
      key: g,
      label: GROUP_LABELS[g],
      rows: hit.filter((p) => p.group === g),
    })).filter((g) => g.rows.length > 0);
  });

  function pick(p: ParamView) {
    picked = p;
    value = p.kind === "bool" ? "on" : "";
  }

  function addPicked() {
    if (!picked) return;
    setParam(picked.key, true, picked.kind === "bool" ? "on" : value.trim());
    onClose();
  }

  function addCustom() {
    const raw = customRaw.trim();
    if (!raw) return;
    setParam(`${APPEND_KEY}:${Date.now()}`, true, raw);
    onClose();
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onClose();
  }}
/>

<div class="overlay">
  <div class="dialog">
    <header>
      <span class="title">添加参数</span>
      {#if !picked}
        <input class="search" type="search" placeholder="搜索名称 / flag…" bind:value={query} />
      {/if}
    </header>

    {#if picked}
      <div class="body">
        <div class="picked">
          <span class="desc">{picked.desc}</span>
          <span class="flags">{picked.flags.join("  ")}</span>
          <HelpTip text={picked.detail} />
        </div>
        {#if picked.kind === "bool"}
          <div class="tip">
            这是开关类参数：添加即表示启用（脚本里写入 <code>{picked.flags[0]}</code>）。
          </div>
        {:else}
          <div class="field">
            <label class="lb" for="pick-value">取值</label>
            <input
              id="pick-value"
              class="input"
              placeholder="例如 8192"
              bind:value={value}
              spellcheck="false"
            />
          </div>
          <div class="tip">{picked.detail}</div>
        {/if}
      </div>
      <footer>
        <button class="ghost" type="button" onclick={() => (picked = null)}>返回</button>
        <button
          class="btn"
          type="button"
          disabled={picked.kind !== "bool" && !value.trim()}
          onclick={addPicked}
        >
          添加
        </button>
      </footer>
    {:else}
      <div class="list">
        <div class="gname">自定义</div>
        <div class="custom">
          <input
            class="input"
            placeholder="flag 与取值，例如 --cache-reuse 256"
            bind:value={customRaw}
            spellcheck="false"
          />
          <button class="btn" type="button" disabled={!customRaw.trim()} onclick={addCustom}>
            添加
          </button>
        </div>

        {#each matched as g (g.key)}
          <div class="gname">{g.label}</div>
          {#each g.rows as p (p.key)}
            <div class="row">
              <button class="pick" type="button" onclick={() => pick(p)}>
                <span class="desc">{p.desc}</span>
                <span class="flags">{p.flags.join("  ")}</span>
              </button>
              <HelpTip text={p.detail} />
            </div>
          {/each}
        {/each}

        {#if matched.length === 0}
          <div class="none">没有匹配的参数（可用上面的自定义输入）</div>
        {/if}
      </div>
      <footer>
        <button class="ghost" type="button" onclick={onClose}>关闭</button>
      </footer>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 45;
    display: grid;
    place-items: center;
    background: rgba(3, 6, 10, 0.6);
  }
  .dialog {
    width: 520px;
    max-height: 74vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.6);
    overflow: hidden;
  }
  header {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .title {
    font-size: 13px;
    font-weight: 600;
  }
  .search {
    flex: 1;
    min-width: 0;
    background: var(--surface-3);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 10px;
    font-family: inherit;
    font-size: 12px;
    color: var(--text);
  }
  .search::placeholder {
    color: var(--faint);
  }
  .search:focus {
    outline: none;
    border-color: var(--accent);
  }
  .body {
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .picked {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .picked .desc {
    font-size: 13px;
  }
  .flags {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .field {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .lb {
    width: 44px;
    flex: 0 0 44px;
    font-size: 12px;
    color: var(--muted);
  }
  .tip {
    font-size: 11px;
    line-height: 16px;
    color: var(--faint);
  }
  .list {
    flex: 1;
    overflow: auto;
    padding: 8px 10px 10px;
    min-height: 0;
  }
  .gname {
    padding: 8px 6px 4px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.4px;
    color: var(--faint);
  }
  .custom {
    display: flex;
    gap: 8px;
    padding: 0 4px 4px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .pick {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border: none;
    border-radius: var(--radius);
    background: none;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .pick:hover {
    background: var(--surface-3);
  }
  .pick .desc {
    flex: 0 0 auto;
    font-size: 12px;
  }
  .none {
    padding: 16px 10px;
    text-align: center;
    color: var(--faint);
    font-size: 12px;
  }
  footer {
    flex: 0 0 auto;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
  }
</style>
