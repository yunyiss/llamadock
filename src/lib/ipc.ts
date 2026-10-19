// 前后端通信封装：命令调用与事件订阅的统一入口。
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

export { invoke, listen };
export type { UnlistenFn };

/** 连通性检查（P0 用于验证 IPC 通路）。 */
export function ping(): Promise<string> {
  return invoke<string>("ping");
}
