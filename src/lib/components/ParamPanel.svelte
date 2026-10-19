<script lang="ts">
  // 参数面板：由解析结果驱动，按「模型 / 服务 / 性能 / GPU」分组可编辑。
  import { setParam, workspace } from "$lib/stores/workspace.svelte";
  import { GROUP_LABELS, GROUP_ORDER } from "$lib/params";
  import ParamRow from "./ParamRow.svelte";

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
    <div class="empty">未在该脚本中识别到 llama.cpp 启动行，可在「原始脚本」中直接编辑</div>
  {:else}
    {#key workspace.selected}
      {#each groups as g (g.key)}
        <section class="group">
          <div class="group-title">{g.label}</div>
          <div class="card">
            {#each g.rows as r (r.key)}
              <ParamRow param={r} onChange={(present, value) => setParam(r.key, present, value)} />
            {/each}
          </div>
        </section>
      {/each}

      {#if workspace.model.unknown.length > 0}
        <section class="group">
          <div class="group-title">未识别参数（原样保留）</div>
          <div class="card">
            <div class="unknown">{workspace.model.unknown.join("   ")}</div>
          </div>
        </section>
      {/if}
    {/key}
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
  .unknown {
    padding: 10px 14px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--muted);
    word-break: break-all;
  }
</style>
