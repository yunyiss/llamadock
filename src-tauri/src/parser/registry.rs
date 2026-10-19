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
        key: "-t",
        flags: &["-t", "--threads"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "CPU 线程数",
    },
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
        desc: "启用 Flash Attention",
    },
];

/// 按任意别名查找参数定义，返回其在注册表中的下标。
pub fn index_of(flag: &str) -> Option<usize> {
    REGISTRY.iter().position(|def| def.flags.contains(&flag))
}
