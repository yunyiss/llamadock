//! 定位启动行：把 `^`（cmd）或 `\`（sh）续行拼成一条逻辑命令，再判断它是否直接调用 llama.cpp 可执行文件。
//!
//! 边界（宁可少覆盖，不猜）：含 `if` / `for` / `start` 的脚本不做结构化解析。

use super::tokenizer::{self, Token};

#[derive(Debug, Clone)]
pub struct LaunchLine {
    /// 首行行号（从 0 开始）
    pub line_index: usize,
    pub tokens: Vec<Token>,
    /// 命令在原文中的字节起点
    pub start: usize,
    /// 命令在原文中的字节终点（不含行尾换行）
    pub end: usize,
}

/// 可执行文件名（不含扩展名）白名单
const EXE_NAMES: [&str; 3] = ["llama-server", "llama-cli", "main"];

struct Line {
    no: usize,
    /// 行内容在原文中的起点
    start: usize,
    /// 行内容长度（不含行尾换行）
    len: usize,
}

fn split_lines(text: &str) -> Vec<Line> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for (no, raw) in text.split_inclusive('\n').enumerate() {
        let mut len = raw.len();
        if len > 0 && raw.as_bytes()[len - 1] == b'\n' {
            len -= 1;
            if len > 0 && raw.as_bytes()[len - 1] == b'\r' {
                len -= 1;
            }
        }
        out.push(Line { no, start: offset, len });
        offset += raw.len();
    }
    out
}

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

/// 行尾是否为续行符。
fn ends_with_continuation(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with('^') || trimmed.ends_with('\\')
}

/// 去掉行尾的续行符与多余空白。
fn drop_continuation(line: &str) -> &str {
    let trimmed = line.trim_end();
    trimmed
        .strip_suffix('^')
        .or_else(|| trimmed.strip_suffix('\\'))
        .unwrap_or(trimmed)
        .trim_end()
}

/// 把拼接串里的偏移映射回原文偏移。
fn translate(map: &[(usize, usize, usize)], offset: usize) -> Option<usize> {
    for &(joined_start, orig_start, len) in map {
        if offset >= joined_start && offset <= joined_start + len {
            return Some(orig_start + (offset - joined_start));
        }
    }
    None
}

pub fn locate(text: &str) -> Option<LaunchLine> {
    let lines = split_lines(text);
    let mut i = 0;

    while i < lines.len() {
        if is_comment(&text[lines[i].start..lines[i].start + lines[i].len]) {
            i += 1;
            continue;
        }

        // 收集续行组
        let mut group = vec![i];
        while ends_with_continuation(&text[lines[i].start..lines[i].start + lines[i].len])
            && i + 1 < lines.len()
        {
            i += 1;
            group.push(i);
        }

        // 拼接成一条逻辑命令，并记录每段与原文的对应关系
        let mut joined = String::new();
        let mut map: Vec<(usize, usize, usize)> = Vec::new();
        for (k, &li) in group.iter().enumerate() {
            let line = &lines[li];
            let segment = drop_continuation(&text[line.start..line.start + line.len]);
            if k > 0 {
                joined.push(' ');
            }
            map.push((joined.len(), line.start, segment.len()));
            joined.push_str(segment);
        }

        let mut tokens = tokenizer::tokenize(&joined);
        for token in tokens.iter_mut() {
            if let (Some(s), Some(e)) = (translate(&map, token.start), translate(&map, token.end)) {
                token.start = s;
                token.end = e;
            }
        }

        if tokens.first().is_some_and(|t| is_exe(&t.text)) {
            let last = &lines[*group.last().unwrap()];
            return Some(LaunchLine {
                line_index: lines[group[0]].no,
                tokens,
                start: lines[group[0]].start,
                end: last.start + last.len,
            });
        }

        i += 1;
    }

    None
}
