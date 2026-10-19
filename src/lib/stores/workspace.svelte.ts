// 工作目录、脚本列表、引擎版本与编辑草稿状态（Svelte 5 rune 模块）。
import {
  getConfig,
  listEngines,
  parseScript,
  parseText,
  pathExists,
  pickEngineFile,
  pickWorkDir,
  rebuildScript,
  removeEngine,
  saveEngine,
  saveScript,
  scanScripts,
  setCurrentEngine,
  startProcess,
  stopProcess,
  switchEngineAll,
  switchEngineText,
  watchDir,
} from "$lib/ipc";
import { onLog, onProcessState, onScriptsChanged } from "$lib/events";
import { SvelteMap } from "svelte/reactivity";
import type { EditEntry, EnginePath, ProcessState, ScriptEntry, ScriptModel } from "$lib/types";

/** 当前是否为 Windows。 */
export const IS_WINDOWS =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");

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
  /** 已保存的引擎版本 */
  engines: [] as EnginePath[],
  currentEngine: null as string | null,
  /** 路径已失效的引擎名 */
  invalidEngines: [] as string[],
  saving: false,
  loading: false,
  error: null as string | null,
  /** 工作目录有变化，提示刷新 */
  scriptsStale: false,
});

/** 单个脚本的运行态。 */
export interface Runtime {
  state: ProcessState;
  pid: number | null;
  exitCode: number | null;
  /** 本次退出是否由用户主动停止触发 */
  stopRequested: boolean;
}

const IDLE: Runtime = { state: "idle", pid: null, exitCode: null, stopRequested: false };
const LOG_LIMIT = 400_000;

/** 每个脚本独立的运行态与日志（支持同时运行多个脚本）。 */
export const runtime = new SvelteMap<string, Runtime>();
export const logs = new SvelteMap<string, string>();

/** 取某个脚本的运行态。 */
export function runtimeOf(path: string | null): Runtime {
  if (!path) return IDLE;
  return runtime.get(path) ?? IDLE;
}

/** 清空某个脚本的日志。 */
export function clearLogs(path: string): void {
  logs.set(path, "");
}

function appendLog(script: string, data: string): void {
  let next = (logs.get(script) ?? "") + data;
  if (next.length > LOG_LIMIT) next = next.slice(next.length - LOG_LIMIT);
  logs.set(script, next);
}

let rebuildTimer: ReturnType<typeof setTimeout> | null = null;

/** 是否有未保存的改动。 */
export function isDirty(): boolean {
  return workspace.draft !== null && workspace.draft !== workspace.baseText;
}

/** 启动时恢复配置、扫描脚本、加载引擎列表。 */
export async function initWorkspace(): Promise<void> {
  const cfg = await getConfig();
  workspace.engines = cfg.enginePaths ?? [];
  workspace.currentEngine = cfg.currentEngine ?? null;
  await refreshEngineValidity();
  if (cfg.workDir) {
    workspace.workDir = cfg.workDir;
    await watchDir(cfg.workDir).catch(() => undefined);
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
    workspace.scriptsStale = false;
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
  await watchDir(dir).catch(() => undefined);
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

/** 刷新引擎路径有效性（只校验绝对路径；`llama-server` 这类走 PATH 的名字不判无效）。 */
async function refreshEngineValidity(): Promise<void> {
  const invalid: string[] = [];
  for (const engine of workspace.engines) {
    if (!looksAbsolute(engine.path)) continue;
    try {
      if (!(await pathExists(engine.path))) invalid.push(engine.name);
    } catch {
      invalid.push(engine.name);
    }
  }
  workspace.invalidEngines = invalid;
}

function looksAbsolute(path: string): boolean {
  return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith("/") || path.includes("\\") || path.includes("/");
}

/** 把某个路径存为引擎版本。 */
export async function saveEngineVersion(path: string): Promise<void> {
  const name = path.split(/[\\/]/).pop() ?? path;
  workspace.engines = await saveEngine(name, path);
  workspace.currentEngine = name;
  await refreshEngineValidity();
}

/** 重新加载引擎列表。 */
export async function reloadEngines(): Promise<void> {
  workspace.engines = await listEngines();
  await refreshEngineValidity();
}

/** 从文件添加引擎版本。 */
export async function addEngine(): Promise<void> {
  const path = await pickEngineFile();
  if (!path) return;
  const name = path.split(/[\\/]/).pop() ?? path;
  workspace.engines = await saveEngine(name, path);
  workspace.currentEngine = name;
  await refreshEngineValidity();
}

/** 移除一个引擎版本。 */
export async function dropEngine(name: string): Promise<void> {
  workspace.engines = await removeEngine(name);
  if (workspace.currentEngine === name) workspace.currentEngine = null;
  await refreshEngineValidity();
}

/** 选用某引擎：记录选择，并可应用到当前脚本。 */
export async function useEngine(name: string, apply: boolean): Promise<void> {
  const engine = workspace.engines.find((e) => e.name === name);
  if (!engine) return;
  workspace.currentEngine = name;
  await setCurrentEngine(name);
  if (!apply || !workspace.selected) return;
  try {
    workspace.edits = [];
    workspace.draft = await switchEngineText(workspace.selected, engine.path);
    await refreshModelFromDraft();
  } catch (e) {
    workspace.error = String(e);
  }
}

/** 批量把引用 oldExe 的脚本切到 newExe，返回改动数量。 */
export async function applyEngineToAll(oldExe: string, newExe: string): Promise<number> {
  const count = await switchEngineAll(oldExe, newExe);
  await refreshScripts();
  if (workspace.selected) await selectScript(workspace.selected);
  return count;
}

/** 订阅进程状态 / 日志 / 目录变更事件（应用启动时调用一次）。 */
export async function initProcessEvents(): Promise<void> {
  await onScriptsChanged(() => {
    workspace.scriptsStale = true;
  });
  await onProcessState((payload) => {
    const prev = runtime.get(payload.script) ?? IDLE;
    runtime.set(payload.script, {
      state: payload.state === "running" ? "running" : "exited",
      pid: payload.pid,
      exitCode: payload.code,
      stopRequested: payload.state === "running" ? false : prev.stopRequested,
    });
  });
  await onLog((payload) => appendLog(payload.script, payload.data));
}

/** 当前脚本能否在本平台直接运行（Windows 无法原生执行 .sh）。 */
export function canRun(): boolean {
  if (!workspace.selected) return false;
  const ext = workspace.selected.split(".").pop()?.toLowerCase();
  return !(IS_WINDOWS && ext === "sh");
}

/** 当前脚本的端口（取自 --port 参数）。 */
export function currentPort(): string | null {
  return workspace.model?.params.find((p) => p.key === "--port")?.value ?? null;
}

/** 启动当前脚本（每个脚本独立进程，可同时运行多个）。 */
export async function runScript(): Promise<void> {
  const path = workspace.selected;
  if (!path || !canRun()) return;
  if (isDirty()) {
    workspace.error = "有未保存的改动，请先保存再启动";
    return;
  }
  workspace.error = null;
  logs.set(path, "");
  try {
    await startProcess(path);
  } catch (e) {
    workspace.error = String(e);
  }
}

/** 停止当前脚本的运行进程。 */
export async function killScript(): Promise<void> {
  const path = workspace.selected;
  if (!path) return;
  const prev = runtime.get(path) ?? IDLE;
  runtime.set(path, { ...prev, stopRequested: true });
  try {
    await stopProcess(path);
  } catch (e) {
    workspace.error = String(e);
  }
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
    resetSelection();
    await selectScript(path);
    return true;
  } catch (e) {
    workspace.error = String(e);
    workspace.pending = null;
    return false;
  } finally {
    workspace.saving = false;
  }
}
