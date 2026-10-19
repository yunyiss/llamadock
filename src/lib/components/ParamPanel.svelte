<script lang="ts">
  // 参数面板：引擎路径（可编辑参数）+ 按分组展示的参数 + 未识别参数 + 添加自定义参数。
  import {
    addEngine,
    dropEngine,
    saveEngineVersion,
    setParam,
    workspace,
  } from "$lib/stores/workspace.svelte";
  import { pickEngineFile } from "$lib/ipc";
  import { GROUP_LABELS, GROUP_ORDER } from "$lib/params";
  import { APPEND_KEY, EXE_KEY } from "$lib/types";
  import ParamRow from "./ParamRow.svelte";
  import HelpTip from "./HelpTip.svelte";

  let engineOpen = $state(false);
  let custom = $state("");

  const engineValue = $derived(workspace.model?.launcher?.exe ?? "");
  const groups = $derived(
    GROUP_ORDER.map((key) => ({
      key,
      label: GROUP_LABELS[key],
      rows: (workspace.model?.params ?? []).filter((p) => p.group === key),
    })).filter((g) => g.rows.length > 0),
  );

  async function browseExe() {
    const path = await pickEngineFile();
    if (path) setParam(EXE_KEY, true, path);
  }

  function useSaved(path: string) {
    setParam(EXE_KEY, true, path);
    engineOpen = false;
  }

  function addCustom() {
    const raw = custom.trim();
    if (!raw) return;
    setParam(`${APPEND_KEY}:${Date.now()}`, true, raw);
    custom = "";
  }
</script>

<div class="content">
  {#if !workspace.selected}
    <div class="empty">从左侧选择一个脚本以查看参数</div>
  {:else if !workspace.model}
    <div class="empty">解析中…</div>
  {:else}
    {#key workspace.selected}
      <!-- 引擎：就是启动行的可执行文件，本身当作可编辑参数 -->
      <section class="group">
        <div class="group-title">引擎</div>
        <div class="card">
          <div class="row">
            <div class="label">
              <div class="t">
                可执行文件
                <HelpTip
                  text="脚本启动行里调用的 llama.cpp 可执行文件（如 llama-server、llama-cli，Windows 下为 .exe）。既可以直接写命令名（由系统 PATH 解析），也可以写完整的绝对路径。修改这里等同于修改脚本中的这一项。"
                />
              </div>
              <div class="d">启动行首个 token</div>
            </div>
            <div class="ctrl">
              <input
                class="input"
                value={engineValue}
                onchange={(e) => setParam(EXE_KEY, true, (e.currentTarget as HTMLInputElement).value)}
                spellcheck="false"
              />
              <button class="btn" type="button" onclick={browseExe}>浏览…</button>
              <button class="btn" type="button" onclick={() => (engineOpen = !engineOpen)}>
                已存版本 ▾
              </button>
            </div>
          </div>

          {#if engineOpen}
            <div class="engine-list">
              {#each workspace.engines as e (e.name)}
                <div class="engine-item">
                  <button class="engine-pick" type="button" onclick={() => useSaved(e.path)}>
                    <span class="n">
                      {#if workspace.invalidEngines.includes(e.name)}⚠ {/if}{e.name}
                    </span>
                    <span class="p" title={e.path}>{e.path}</span>
                  </button>
                  <button class="del" type="button" title="移除该版本" onclick={() => void dropEngine(e.name)}>×</button>
                </div>
              {/each}
              {#if workspace.engines.length === 0}
                <div class="none">还没有保存的版本</div>
              {/if}
              <div class="engine-actions">
                <button class="btn" type="button" onclick={() => void saveEngineVersion(engineValue)}>
                  把当前值存为版本
                </button>
                <button class="btn" type="button" onclick={() => void addEngine()}>从文件添加…</button>
              </div>
            </div>
          {/if}
        </div>
      </section>

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
            <div class="unknown-tip">
              未登记的参数不会被面板结构化，可用下方「添加参数」或「原始脚本」编辑。
            </div>
          </div>
        </section>
      {/if}

      <section class="group">
        <div class="group-title">添加参数</div>
        <div class="card">
          <div class="row">
            <div class="label">
              <div class="t">自定义参数</div>
              <div class="d">未登记的 flag 也能加</div>
            </div>
            <div class="ctrl">
              <input
                class="input"
                placeholder="flag 与取值，例如 --cache-reuse 256"
                bind:value={custom}
                spellcheck="false"
              />
              <button class="btn" type="button" onclick={addCustom}>添加</button>
            </div>
          </div>
        </div>
      </section>
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
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 14px;
  }
  .label {
    width: 220px;
    flex: 0 0 220px;
  }
  .label .t {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }
  .label .d {
    margin-top: 2px;
    font-size: 11px;
    color: var(--faint);
  }
  .ctrl {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .engine-list {
    border-top: 1px solid var(--border);
    padding: 6px;
  }
  .engine-item {
    display: flex;
    align-items: center;
  }
  .engine-pick {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 7px 10px;
    border: none;
    border-radius: var(--radius);
    background: none;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .engine-pick:hover {
    background: var(--surface-2);
  }
  .engine-pick .n {
    font-size: 13px;
  }
  .engine-pick .p {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .del {
    padding: 2px 8px;
    border: none;
    border-radius: var(--radius);
    background: none;
    color: var(--faint);
    font-size: 14px;
    cursor: pointer;
  }
  .del:hover {
    color: var(--danger);
    background: var(--surface-2);
  }
  .none {
    padding: 10px;
    text-align: center;
    color: var(--faint);
    font-size: 12px;
  }
  .engine-actions {
    display: flex;
    gap: 8px;
    padding: 6px 4px 2px;
  }
  .unknown {
    padding: 10px 14px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--muted);
    word-break: break-all;
  }
  .unknown-tip {
    padding: 0 14px 10px;
    font-size: 11px;
    color: var(--faint);
  }
</style>
