// 工作目录与脚本列表状态（Svelte 5 rune 模块）。
import { getConfig, parseScript, pickWorkDir, scanScripts } from "$lib/ipc";
import type { ScriptEntry, ScriptModel } from "$lib/types";

export const workspace = $state({
  workDir: null as string | null,
  scripts: [] as ScriptEntry[],
  selected: null as string | null,
  model: null as ScriptModel | null,
  loading: false,
  error: null as string | null,
});

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
  workspace.selected = null;
  workspace.model = null;
  await refreshScripts();
}

/** 选中脚本并解析其参数。 */
export async function selectScript(path: string): Promise<void> {
  workspace.selected = path;
  workspace.model = null;
  workspace.error = null;
  try {
    workspace.model = await parseScript(path);
  } catch (e) {
    workspace.error = String(e);
  }
}
