// 前后端通信封装：命令调用与事件订阅的统一入口。
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { Config, EditEntry, EnginePath, ScriptEntry, ScriptModel } from "./types";

export { invoke, listen };
export type { UnlistenFn };

/** 连通性检查（P0 用于验证 IPC 通路）。 */
export function ping(): Promise<string> {
  return invoke<string>("ping");
}

/** 读取应用配置。 */
export function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}

/** 选择工作目录；取消时返回 null。 */
export function pickWorkDir(): Promise<string | null> {
  return invoke<string | null>("pick_work_dir");
}

/** 扫描当前工作目录下的脚本。 */
export function scanScripts(): Promise<ScriptEntry[]> {
  return invoke<ScriptEntry[]>("scan_scripts");
}

/** 在系统文件管理器中打开目录。 */
export function openFolder(path: string): Promise<void> {
  return invoke<void>("open_folder", { path });
}

/** 判断路径是否存在（用于引擎/模型失效提示）。 */
export function pathExists(path: string): Promise<boolean> {
  return invoke<boolean>("path_exists", { path });
}

/** 解析脚本文件中的 llama.cpp 启动参数。 */
export function parseScript(path: string): Promise<ScriptModel> {
  return invoke<ScriptModel>("parse_script", { path });
}

/** 解析内存文本（编辑器与参数面板同步用）。 */
export function parseText(path: string, text: string): Promise<ScriptModel> {
  return invoke<ScriptModel>("parse_text", { path, text });
}

/** 按参数改动重建脚本文本（不写盘）。 */
export function rebuildScript(path: string, edits: EditEntry[]): Promise<string> {
  return invoke<string>("rebuild_script", { path, edits });
}

/** 写回脚本（保留原编码、校验未被外部修改、原子写）。 */
export function saveScript(path: string, content: string, base: string): Promise<void> {
  return invoke<void>("save_script", { path, content, base });
}

/** 已保存的引擎版本列表。 */
export function listEngines(): Promise<EnginePath[]> {
  return invoke<EnginePath[]>("list_engines");
}

/** 新增或更新一个引擎版本。 */
export function saveEngine(name: string, path: string): Promise<EnginePath[]> {
  return invoke<EnginePath[]>("save_engine", { name, path });
}

/** 移除一个引擎版本。 */
export function removeEngine(name: string): Promise<EnginePath[]> {
  return invoke<EnginePath[]>("remove_engine", { name });
}

/** 记录当前选中的引擎版本。 */
export function setCurrentEngine(name: string | null): Promise<void> {
  return invoke<void>("set_current_engine", { name });
}

/** 选择引擎可执行文件。 */
export function pickEngineFile(): Promise<string | null> {
  return invoke<string | null>("pick_engine_file");
}

/** 替换当前脚本的引擎路径，返回新文本。 */
export function switchEngineText(path: string, newExe: string): Promise<string> {
  return invoke<string>("switch_engine_text", { path, newExe });
}

/** 批量替换引用同一引擎的脚本，返回改动数量。 */
export function switchEngineAll(oldExe: string, newExe: string): Promise<number> {
  return invoke<number>("switch_engine_all", { oldExe, newExe });
}
