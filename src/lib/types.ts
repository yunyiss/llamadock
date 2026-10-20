// 前后端共享的数据结构（按阶段逐步补齐）。

/** 脚本列表项（由后端扫描返回）。 */
export interface ScriptEntry {
  path: string;
  name: string;
  ext: string;
}

/** 已保存的 llama.cpp 引擎版本。 */
export interface EnginePath {
  name: string;
  path: string;
}

/** 应用配置。 */
export interface Config {
  workDir: string | null;
  recentDirs: string[];
  enginePaths: EnginePath[];
  currentEngine: string | null;
}

/** 参数类型。 */
export type ParamKind = "number" | "bool" | "path" | "string" | "enum";

/** 参数分组。 */
export type ParamGroup = "model" | "server" | "perf" | "gpu" | "spec" | "sampling" | "log";

/** 单个参数（解析结果视图）。 */
export interface ParamView {
  key: string;
  flags: string[];
  kind: ParamKind;
  group: ParamGroup;
  desc: string;
  /** 点「?」时展示的详细说明 */
  detail: string;
  value: string | null;
  present: boolean;
}

/** 保留编辑键：修改启动行的可执行文件路径（与后端 rebuild 常量一致）。 */
export const EXE_KEY = "__exe";

/** 保留编辑键前缀：把原始文本追加到命令末尾（添加任意参数）。 */
export const APPEND_KEY = "__append";

/** 脚本解析模型。 */
export interface ScriptModel {
  path: string;
  found: boolean;
  launcher: { exe: string; lineIndex: number } | null;
  params: ParamView[];
  unknown: string[];
  raw: string;
}

/** 提交给后端的单条参数改动。 */
export interface EditEntry {
  key: string;
  present: boolean;
  value: string | null;
}

/** 进程状态（P5 起由后端推送）。 */
export type ProcessState = "idle" | "running" | "stopped" | "exited";
