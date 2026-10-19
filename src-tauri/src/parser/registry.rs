//! 参数注册表：flag、别名、类型、分组、说明。新增参数只改这一个文件（见技术方案 §4.2）。

use serde::Serialize;

/// 参数类型；部分变体为后续参数预留（前端 `types.ts` 已对齐）。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamKind {
    Number,
    Bool,
    Path,
    String,
    Enum,
}

/// 参数分组；部分变体为后续参数预留（前端 `types.ts` 已对齐）。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamGroup {
    Model,
    Server,
    Perf,
    Gpu,
    Sampling,
    Log,
}

pub struct ParamDef {
    /// 规范名（首个别名）
    pub key: &'static str,
    /// 全部别名
    pub flags: &'static [&'static str],
    pub kind: ParamKind,
    pub group: ParamGroup,
    /// 简短说明
    pub desc: &'static str,
}

pub const REGISTRY: &[ParamDef] = &[
    // 模型
    ParamDef {
        key: "-m",
        flags: &["-m", "--model"],
        kind: ParamKind::Path,
        group: ParamGroup::Model,
        desc: "模型文件路径",
    },
    ParamDef {
        key: "-c",
        flags: &["-c", "--ctx-size"],
        kind: ParamKind::Number,
        group: ParamGroup::Model,
        desc: "上下文长度",
    },
    ParamDef {
        key: "-a",
        flags: &["-a", "--alias"],
        kind: ParamKind::String,
        group: ParamGroup::Model,
        desc: "模型别名",
    },
    // 服务
    ParamDef {
        key: "--host",
        flags: &["--host"],
        kind: ParamKind::String,
        group: ParamGroup::Server,
        desc: "监听地址",
    },
    ParamDef {
        key: "--port",
        flags: &["--port"],
        kind: ParamKind::Number,
        group: ParamGroup::Server,
        desc: "服务端口",
    },
    ParamDef {
        key: "--parallel",
        flags: &["-np", "--parallel"],
        kind: ParamKind::Number,
        group: ParamGroup::Server,
        desc: "并发数",
    },
    ParamDef {
        key: "--jinja",
        flags: &["--jinja"],
        kind: ParamKind::Bool,
        group: ParamGroup::Server,
        desc: "启用 Jinja 模板",
    },
    // 性能
    ParamDef {
        key: "-t",
        flags: &["-t", "--threads"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "CPU 线程数",
    },
    ParamDef {
        key: "-b",
        flags: &["-b", "--batch-size"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "批处理大小",
    },
    ParamDef {
        key: "-ub",
        flags: &["-ub", "--ubatch-size"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "微批处理大小",
    },
    // GPU / 加速
    ParamDef {
        key: "-ngl",
        flags: &["-ngl", "--n-gpu-layers"],
        kind: ParamKind::Number,
        group: ParamGroup::Gpu,
        desc: "GPU 卸载层数",
    },
    ParamDef {
        key: "--flash-attn",
        flags: &["-fa", "--flash-attn", "--no-flash-attn"],
        kind: ParamKind::Bool,
        group: ParamGroup::Gpu,
        desc: "Flash Attention",
    },
    ParamDef {
        key: "-ctk",
        flags: &["-ctk", "--cache-type-k"],
        kind: ParamKind::String,
        group: ParamGroup::Gpu,
        desc: "K 缓存量化类型",
    },
    ParamDef {
        key: "-ctv",
        flags: &["-ctv", "--cache-type-v"],
        kind: ParamKind::String,
        group: ParamGroup::Gpu,
        desc: "V 缓存量化类型",
    },
    // 采样
    ParamDef {
        key: "--temp",
        flags: &["--temp"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "温度",
    },
    ParamDef {
        key: "--top-p",
        flags: &["--top-p"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "Top-P",
    },
    ParamDef {
        key: "--top-k",
        flags: &["--top-k"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "Top-K",
    },
    ParamDef {
        key: "--min-p",
        flags: &["--min-p"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "Min-P",
    },
    ParamDef {
        key: "-n",
        flags: &["-n", "--predict"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "最大生成 token 数",
    },
];

/// 按任意别名查找参数定义，返回其在注册表中的下标。
pub fn index_of(flag: &str) -> Option<usize> {
    REGISTRY.iter().position(|def| def.flags.contains(&flag))
}
