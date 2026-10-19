//! 定位启动行：只处理「直接调用 llama.cpp 可执行文件」的行（见技术方案 §4.2）。
//!
//! 边界（宁可少覆盖，不猜）：含 `if` / `for` / 多行续行 / `start` 的脚本不做结构化解析。

use super::tokenizer::{self, Token};

#[derive(Debug, Clone)]
pub struct LaunchLine {
    /// 行号（从 0 开始）
    pub line_index: usize,
    pub tokens: Vec<Token>,
}

/// 可执行文件名（不含扩展名）白名单
const EXE_NAMES: [&str; 3] = ["llama-server", "llama-cli", "main"];

fn is_exe(token: &str) -> bool {
    let base = token.rsplit(['/', '\\']).next().unwrap_or(token);
    let stem = base.strip_suffix(".exe").unwrap_or(base);
    EXE_NAMES.contains(&stem) || stem.contains("llama")
}

fn is_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with("::")
        || trimmed.to_lowercase().starts_with("rem ")
}

pub fn locate(text: &str) -> Option<LaunchLine> {
    for (line_index, line) in text.lines().enumerate() {
        if is_comment(line) {
            continue;
        }
        let tokens = tokenizer::tokenize(line);
        let Some(first) = tokens.first() else {
            continue;
        };
        if is_exe(&first.text) {
            return Some(LaunchLine {
                line_index,
                tokens,
            });
        }
    }
    None
}
