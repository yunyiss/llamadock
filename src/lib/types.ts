// 前后端共享的数据结构（按阶段逐步补齐）。

/** 脚本列表项（P1 起由后端扫描返回）。 */
export interface ScriptEntry {
  path: string;
  name: string;
}

/** 进程状态（P5 起由后端推送）。 */
export type ProcessState = "idle" | "running" | "stopped" | "exited";
