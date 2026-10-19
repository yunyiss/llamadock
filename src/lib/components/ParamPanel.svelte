<script lang="ts">
  // 参数面板：由解析结果驱动，按「模型 / 服务 / 性能 / GPU」分组只读展示（P3 起可编辑）。
  import { workspace } from "$lib/stores/workspace.svelte";
  import { GROUP_LABELS, GROUP_ORDER } from "$lib/params";

  const groups = $derived(
    GROUP_ORDER.map((key) => ({
      key,
      label: GROUP_LABELS[key],
      rows: (workspace.model?.params ?? []).filter((p) => p.group === key),
    })).filter((g) => g.rows.length > 0),
  );
</script>

<div class="content">
  {#if !workspace.selected}
    <div class="empty">从左侧选择一个脚本以查看参数</div>
  {:else if !workspace.model}
    <div class="empty">解析中…</div>
  {:else if !workspace.model.found}
    <div class="empty">未在该脚本中识别到 llama.cpp 启动行</div>
  {:else}
    {#each groups as g (g.key)}
      <section class="group">
        <div class="group-title">{g.label}</div>
        <div class="card">
          {#each g.rows as r (r.key)}
            <div class="row" class:absent={!r.present}>
              <div class="label">
                <div class="t">{r.desc}</div>
                <div class="d"><span class="flag">{r.flags.join(" · ")}</span></div>
              </div>
              <div class="ctrl">
                {#if r.kind === "bool"}
                  <span class="switch" class:off={!r.present || r.value === "off"}></span>
                {:else}
                  <span class="field" class:num={r.kind === "number"}>{r.value ?? "未设置"}</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/each}

    {#if workspace.model.unknown.length > 0}
      <section class="group">
        <div class="group-title">未识别参数</div>
        <div class="card">
          <div class="unknown">{workspace.model.unknown.join("   ")}</div>
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .content {
    flex: 1;
    overflow: auto;
    padding: 16px;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--faint);
    font-size: 12px;
  }
  .group {
    flex: 0 0 auto;
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
  .unknown {
    padding: 10px 14px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--muted);
    word-break: break-all;
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
