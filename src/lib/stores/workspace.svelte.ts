// 工作目录、脚本列表与编辑草稿状态（Svelte 5 rune 模块）。
import {
  getConfig,
  parseScript,
  parseText,
  pickWorkDir,
  rebuildScript,
  saveScript,
  scanScripts,
} from "$lib/ipc";
import type { EditEntry, ScriptEntry, ScriptModel } from "$lib/types";

export const workspace = $state({
  workDir: null as string | null,
  scripts: [] as ScriptEntry[],
  selected: null as string | null,
  /** 当前文本的解析视图（可能来自草稿） */
  model: null as ScriptModel | null,
  /** 磁盘上的原始文本，保存时作为比对基准 */
  baseText: null as string | null,
  /** 编辑后的草稿文本；null 表示未改动 */
  draft: null as string | null,
  /** 待应用的参数改动 */
  edits: [] as EditEntry[],
  /** 待确认的差异 */
  pending: null as { oldText: string; newText: string } | null,
  saving: false,
  loading: false,
  error: null as string | null,
});

let rebuildTimer: ReturnType<typeof setTimeout> | null = null;

/** 是否有未保存的改动。 */
export function isDirty(): boolean {
  return workspace.draft !== null && workspace.draft !== workspace.baseText;
}

/** 启动时恢复上次的工作目录并扫描。 */
export async function initWorkspace(): Promise<void> {
  const cfg = await getConfig();
  if (cfg.workDir) {
    workspace.workDir = cfg.workDir;
    await refreshScripts();
  }
}

/** 重新扫描脚本列表。 */
export async function refreshScripts(): Promise<void> {
  if (!workspace.workDir) return;
  workspace.loading = true;
  try {
    workspace.scripts = await scanScripts();
    workspace.error = null;
  } catch (e) {
    workspace.error = String(e);
  } finally {
    workspace.loading = false;
  }
}

/** 选择新的工作目录并扫描。 */
export async function chooseWorkDir(): Promise<void> {
  const dir = await pickWorkDir();
  if (!dir) return;
  workspace.workDir = dir;
  resetSelection();
  await refreshScripts();
}

/** 选中脚本：解析并记录磁盘基准文本。 */
export async function selectScript(path: string): Promise<void> {
  workspace.selected = path;
  resetSelection();
  workspace.selected = path;
  try {
    const model = await parseScript(path);
    workspace.model = model;
    workspace.baseText = model.raw;
  } catch (e) {
    workspace.error = String(e);
  }
}

/** 记录一条参数改动，并去抖重建草稿。 */
export function setParam(key: string, present: boolean, value: string | null): void {
  const entry: EditEntry = { key, present, value };
  const index = workspace.edits.findIndex((e) => e.key === key);
  if (index >= 0) workspace.edits[index] = entry;
  else workspace.edits.push(entry);
  scheduleRebuild();
}

/** 直接编辑原始文本（丢弃参数改动）。 */
export async function setRaw(text: string): Promise<void> {
  workspace.edits = [];
  workspace.draft = text;
  await refreshModelFromDraft();
}

function resetSelection(): void {
  workspace.model = null;
  workspace.baseText = null;
  workspace.draft = null;
  workspace.edits = [];
  workspace.pending = null;
  workspace.error = null;
}

function scheduleRebuild(): void {
  if (rebuildTimer) clearTimeout(rebuildTimer);
  rebuildTimer = setTimeout(() => void applyEdits(), 250);
}

/** 把参数改动应用到草稿文本，并同步刷新参数视图。 */
export async function applyEdits(): Promise<void> {
  if (!workspace.selected || workspace.edits.length === 0) return;
  try {
    workspace.draft = await rebuildScript(workspace.selected, workspace.edits);
    await refreshModelFromDraft();
  } catch (e) {
    workspace.error = String(e);
  }
}

async function refreshModelFromDraft(): Promise<void> {
  if (!workspace.selected) return;
  const text = workspace.draft ?? workspace.baseText;
  if (text === null) return;
  try {
    workspace.model = await parseText(workspace.selected, text);
    workspace.error = null;
  } catch (e) {
    workspace.error = String(e);
  }
}

/** 生成待确认的差异；无改动时返回 false。 */
export async function prepareSave(): Promise<boolean> {
  if (rebuildTimer) {
    clearTimeout(rebuildTimer);
    rebuildTimer = null;
  }
  await applyEdits();
  if (!workspace.baseText || workspace.draft === null || workspace.draft === workspace.baseText) {
    return false;
  }
  workspace.pending = { oldText: workspace.baseText, newText: workspace.draft };
  return true;
}

/** 取消保存。 */
export function cancelSave(): void {
  workspace.pending = null;
}

/** 确认保存：写回后重新解析。 */
export async function commitSave(): Promise<boolean> {
  const pending = workspace.pending;
  const path = workspace.selected;
  if (!pending || !path || !workspace.baseText) return false;

  workspace.saving = true;
  try {
    await saveScript(path, pending.newText, workspace.baseText);
    workspace.pending = null;
    const keep = path;
    resetSelection();
    await selectScript(keep);
    return true;
  } catch (e) {
    workspace.error = String(e);
    workspace.pending = null;
    return false;
  } finally {
    workspace.saving = false;
  }
}
