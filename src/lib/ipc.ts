// 前后端通信封装：命令调用与事件订阅的统一入口。
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { Config, ScriptEntry } from "./types";

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
