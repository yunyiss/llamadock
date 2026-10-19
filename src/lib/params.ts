// 参数分组的展示元数据（与后端 registry 的 group 取值对齐）。
import type { ParamGroup } from "$lib/types";

export const GROUP_ORDER = ["model", "server", "perf", "gpu", "log"] as const satisfies readonly ParamGroup[];

export const GROUP_LABELS: Record<ParamGroup, string> = {
  model: "模型",
  server: "服务",
  perf: "性能",
  gpu: "GPU / 加速",
  log: "日志",
};
