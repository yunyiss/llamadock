//! 参数注册表：flag、别名、类型、分组、简述与详细说明。新增/修改参数只改这一个文件（见技术方案 §4.2）。
//!
//! `detail` 文案为 llama.cpp 官方参数说明的浓缩版（以 `llama-server --help` / 官方 README 为准）。

use serde::Serialize;

/// 参数类型；`Enum` 等变体为后续参数预留（前端 `types.ts` 已对齐）。
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

/// 参数分组（前端 `params.ts` 已对齐）。
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
    /// 面板上显示的简短名称
    pub desc: &'static str,
    /// 点「?」时展示的详细说明
    pub detail: &'static str,
}

pub const REGISTRY: &[ParamDef] = &[
    // ── 模型 ─────────────────────────────────────────────
    ParamDef {
        key: "-m",
        flags: &["-m", "--model"],
        kind: ParamKind::Path,
        group: ParamGroup::Model,
        desc: "模型文件路径",
        detail: "要加载的 GGUF 模型文件路径。可用绝对路径，也可用相对路径（相对路径按脚本所在目录解析）。",
    },
    ParamDef {
        key: "-c",
        flags: &["-c", "--ctx-size"],
        kind: ParamKind::Number,
        group: ParamGroup::Model,
        desc: "上下文长度",
        detail: "上下文窗口大小（token）。0 表示使用模型训练时的默认值。KV 缓存占用随该值近似线性增长。",
    },
    ParamDef {
        key: "-a",
        flags: &["-a", "--alias"],
        kind: ParamKind::String,
        group: ParamGroup::Model,
        desc: "模型别名",
        detail: "替换模型在接口中暴露的名字（如 /v1/models 返回值）。",
    },
    ParamDef {
        key: "--no-mmap",
        flags: &["--no-mmap"],
        kind: ParamKind::Bool,
        group: ParamGroup::Model,
        desc: "禁用内存映射",
        detail: "不使用 mmap 加载模型，而是整体读入内存。内存占用更可控，但加载更慢。",
    },
    ParamDef {
        key: "--mlock",
        flags: &["--mlock"],
        kind: ParamKind::Bool,
        group: ParamGroup::Model,
        desc: "锁定内存",
        detail: "把模型锁定在物理内存中，避免被换出到磁盘。部分系统需要提升权限。",
    },
    // ── 服务 ─────────────────────────────────────────────
    ParamDef {
        key: "--host",
        flags: &["--host"],
        kind: ParamKind::String,
        group: ParamGroup::Server,
        desc: "监听地址",
        detail: "HTTP 服务监听地址，默认 127.0.0.1。改成 0.0.0.0 可被局域网访问（注意安全）。",
    },
    ParamDef {
        key: "--port",
        flags: &["--port"],
        kind: ParamKind::Number,
        group: ParamGroup::Server,
        desc: "服务端口",
        detail: "HTTP 服务监听端口，默认 8080。",
    },
    ParamDef {
        key: "--parallel",
        flags: &["-np", "--parallel"],
        kind: ParamKind::Number,
        group: ParamGroup::Server,
        desc: "并发数",
        detail: "并行处理的请求数。KV 缓存总占用约为 该值 × 单序列上下文。",
    },
    ParamDef {
        key: "--jinja",
        flags: &["--jinja"],
        kind: ParamKind::Bool,
        group: ParamGroup::Server,
        desc: "启用 Jinja 模板",
        detail: "使用模型内置的 Jinja 对话模板处理工具调用等高级模板，推荐开启。",
    },
    ParamDef {
        key: "--no-webui",
        flags: &["--no-webui"],
        kind: ParamKind::Bool,
        group: ParamGroup::Server,
        desc: "禁用网页界面",
        detail: "关闭内置的网页聊天界面，只保留 HTTP API。",
    },
    ParamDef {
        key: "-to",
        flags: &["-to", "--timeout"],
        kind: ParamKind::Number,
        group: ParamGroup::Server,
        desc: "超时（秒）",
        detail: "服务端读写超时时间，单位秒，默认 600。长时间生成大文本时可适当调大。",
    },
    ParamDef {
        key: "--api-key",
        flags: &["--api-key"],
        kind: ParamKind::String,
        group: ParamGroup::Server,
        desc: "API 密钥",
        detail: "设置后所有请求需携带 Authorization: Bearer <key>，用于对外暴露时的简单鉴权。",
    },
    ParamDef {
        key: "--threads-http",
        flags: &["--threads-http"],
        kind: ParamKind::Number,
        group: ParamGroup::Server,
        desc: "HTTP 线程数",
        detail: "处理 HTTP 请求的线程数，默认为 max(1, 物理核心数 - 1)。与推理线程分开。",
    },
    ParamDef {
        key: "--embeddings",
        flags: &["--embeddings"],
        kind: ParamKind::Bool,
        group: ParamGroup::Server,
        desc: "启用向量化",
        detail: "开启 /embedding 接口，只做文本向量化不生成内容。",
    },
    // ── 性能 ─────────────────────────────────────────────
    ParamDef {
        key: "-t",
        flags: &["-t", "--threads"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "CPU 线程数",
        detail: "生成阶段使用的 CPU 线程数，默认取物理核心数。线程过多反而会因调度开销变慢。",
    },
    ParamDef {
        key: "-tb",
        flags: &["-tb", "--threads-batch"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "批处理线程数",
        detail: "预填充（prompt 处理）阶段的 CPU 线程数，默认与 -t 相同。",
    },
    ParamDef {
        key: "-b",
        flags: &["-b", "--batch-size"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "批处理大小",
        detail: "逻辑批大小（一批处理的 token 数），默认 2048。",
    },
    ParamDef {
        key: "-ub",
        flags: &["-ub", "--ubatch-size"],
        kind: ParamKind::Number,
        group: ParamGroup::Perf,
        desc: "微批处理大小",
        detail: "物理批大小（单次计算实际处理的 token 数），默认 512，是显存/内存峰值的主要调节项。",
    },
    // ── GPU / 加速 ───────────────────────────────────────
    ParamDef {
        key: "-ngl",
        flags: &["-ngl", "--n-gpu-layers"],
        kind: ParamKind::Number,
        group: ParamGroup::Gpu,
        desc: "GPU 卸载层数",
        detail: "卸载到 GPU 计算的层数。设很大（如 999）或 auto 表示尽量全部卸载，显存不足时按需下调。",
    },
    ParamDef {
        key: "--flash-attn",
        flags: &["-fa", "--flash-attn", "--no-flash-attn"],
        kind: ParamKind::Bool,
        group: ParamGroup::Gpu,
        desc: "Flash Attention",
        detail: "启用 Flash Attention，可显著降低 KV 缓存显存并提升长上下文速度。取值 on / off / auto。",
    },
    ParamDef {
        key: "-ctk",
        flags: &["-ctk", "--cache-type-k"],
        kind: ParamKind::String,
        group: ParamGroup::Gpu,
        desc: "K 缓存量化",
        detail: "KV 缓存中 K 的量化类型（如 f16、q8_0、q4_0）。量化可省显存，但可能轻微影响质量。",
    },
    ParamDef {
        key: "-ctv",
        flags: &["-ctv", "--cache-type-v"],
        kind: ParamKind::String,
        group: ParamGroup::Gpu,
        desc: "V 缓存量化",
        detail: "KV 缓存中 V 的量化类型。通常 V 比 K 对量化更敏感，建议先只量化 K。",
    },
    ParamDef {
        key: "-ts",
        flags: &["-ts", "--tensor-split"],
        kind: ParamKind::String,
        group: ParamGroup::Gpu,
        desc: "多卡分配比例",
        detail: "多 GPU 时的显存分配比例，如 3,1 表示按 3:1 分配。留空为自动。",
    },
    ParamDef {
        key: "-mg",
        flags: &["-mg", "--main-gpu"],
        kind: ParamKind::Number,
        group: ParamGroup::Gpu,
        desc: "主 GPU 序号",
        detail: "多 GPU 场景下指定主 GPU 的序号（从 0 开始）。",
    },
    ParamDef {
        key: "--no-kv-offload",
        flags: &["--no-kv-offload"],
        kind: ParamKind::Bool,
        group: ParamGroup::Gpu,
        desc: "KV 不卸载",
        detail: "不把 KV 缓存放到 GPU（默认是放 GPU 的）。显存不足时可与 -ngl 搭配权衡。",
    },
    // ── 采样 ─────────────────────────────────────────────
    ParamDef {
        key: "--temp",
        flags: &["--temp"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "温度",
        detail: "采样温度，默认 0.8。越低越确定、越高越发散；代码/工具调用场景常用 0.1~0.3。",
    },
    ParamDef {
        key: "--top-p",
        flags: &["--top-p"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "Top-P",
        detail: "核采样阈值，候选词累计概率达到该比例即截断，默认 0.95。",
    },
    ParamDef {
        key: "--top-k",
        flags: &["--top-k"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "Top-K",
        detail: "仅在概率最高的 K 个候选词中采样，默认 40。",
    },
    ParamDef {
        key: "--min-p",
        flags: &["--min-p"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "Min-P",
        detail: "候选词概率下限：低于「最高概率 × 该值」的词被剔除，默认 0.05。",
    },
    ParamDef {
        key: "--repeat-penalty",
        flags: &["--repeat-penalty"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "重复惩罚",
        detail: "对已出现过的词施加惩罚，1.0 表示不惩罚，默认 1.0。",
    },
    ParamDef {
        key: "--repeat-last-n",
        flags: &["--repeat-last-n"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "重复惩罚范围",
        detail: "参与重复惩罚的最近 token 数，默认 64；设为 0 表示全部，-1 表示关闭。",
    },
    ParamDef {
        key: "--seed",
        flags: &["--seed"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "随机种子",
        detail: "采样随机种子，固定后可复现同样的输出；默认 -1 表示每次随机。",
    },
    ParamDef {
        key: "-n",
        flags: &["-n", "--predict"],
        kind: ParamKind::Number,
        group: ParamGroup::Sampling,
        desc: "最大生成数",
        detail: "最多生成的 token 数，-1 表示不限制（直到上下文用满）。",
    },
    // ── 日志 ─────────────────────────────────────────────
    ParamDef {
        key: "-v",
        flags: &["-v", "--verbose"],
        kind: ParamKind::Bool,
        group: ParamGroup::Log,
        desc: "详细日志",
        detail: "输出更详细的调试日志，排查加载或性能问题时使用。",
    },
];

/// 按任意别名查找参数定义，返回其在注册表中的下标。
pub fn index_of(flag: &str) -> Option<usize> {
    REGISTRY.iter().position(|def| def.flags.contains(&flag))
}
