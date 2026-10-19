// 后端推送事件的订阅封装。
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

export interface LogPayload {
  stream: "stdout" | "stderr";
  data: string;
}

export interface ProcessStatePayload {
  state: "running" | "exited";
  pid: number | null;
  code: number | null;
}

/** 订阅进程日志。 */
export function onLog(handler: (payload: LogPayload) => void): Promise<UnlistenFn> {
  return listen<LogPayload>("process://log", (event) => handler(event.payload));
}

/** 订阅进程状态变化。 */
export function onProcessState(
  handler: (payload: ProcessStatePayload) => void,
): Promise<UnlistenFn> {
  return listen<ProcessStatePayload>("process://state", (event) => handler(event.payload));
}
