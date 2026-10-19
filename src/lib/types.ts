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
export type ParamGroup = "model" | "server" | "perf" | "gpu" | "log";

/** 单个参数（解析结果视图）。 */
export interface ParamView {
  key: string;
  flags: string[];
  kind: ParamKind;
  group: ParamGroup;
  desc: string;
  value: string | null;
  present: boolean;
}

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
