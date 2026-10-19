//! 最小差异重建：按 token 在原文中的字节区间从后往前替换 / 插入 / 删除，其余文本原样保留（见技术方案 §4.2）。

use serde::Deserialize;

use super::analyze;
use super::registry::{ParamDef, ParamKind, REGISTRY};
use crate::fsutil::quoting::quote;

/// 保留键：修改启动行的可执行文件路径（llama.cpp 引擎路径本身就是一个参数）。
pub const EXE_KEY: &str = "__exe";
/// 保留键：把一段原始文本追加到命令末尾（用于添加任意参数）。
pub const APPEND_KEY: &str = "__append";

/// 前端提交的单条参数改动。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub key: String,
    pub present: bool,
    pub value: Option<String>,
}

/// 删除时的实际起点：吞掉多余空白；若该 token 位于续行行首，则连同上一行的续行符一起删除。
fn deletion_start(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = start;
    while i > 0 && bytes[i - 1] == b' ' {
        i -= 1;
    }
    if i == 0 {
        return i;
    }

    // 是否已退到行首
    let mut j = i;
    while j > 0 && (bytes[j - 1] == b'\n' || bytes[j - 1] == b'\r') {
        j -= 1;
    }
    if j == i {
        return i; // 前面只是空格，不是行首
    }

    // 行首：尝试连同上一行的续行符一起删掉
    let mut k = j;
    while k > 0 && bytes[k - 1] == b' ' {
        k -= 1;
    }
    if k > 0 && (bytes[k - 1] == b'^' || bytes[k - 1] == b'\\') {
        k -= 1;
        while k > 0 && bytes[k - 1] == b' ' {
            k -= 1;
        }
        return k;
    }
    start
}

fn off_alias(def: &ParamDef) -> Option<String> {
    def.flags
        .iter()
        .find(|f| f.starts_with("--no-"))
        .map(|f| f.to_string())
}

fn on_alias(def: &ParamDef, used: Option<&str>) -> String {
    if let Some(u) = used {
        if !u.starts_with("--no-") {
            return u.to_string();
        }
    }
    def.flags
        .iter()
        .find(|f| !f.starts_with("--no-"))
        .map(|f| f.to_string())
        .unwrap_or_else(|| def.flags[0].to_string())
}

/// 应用改动并返回新的脚本文本；未识别到启动行时返回 `None`。
pub fn apply_edits(text: &str, edits: &[Edit]) -> Option<String> {
    let analyzed = analyze(text)?;
    let launch = &analyzed.launch;

    // (字节起点, 字节终点, 新片段)，稍后从后往前应用
    let mut reps: Vec<(usize, usize, String)> = Vec::new();

    for edit in edits {
        // 保留键：改引擎路径
        if edit.key == EXE_KEY {
            if let (Some(token), Some(value)) = (launch.tokens.first(), edit.value.clone()) {
                reps.push((token.start, token.end, quote(&value)));
            }
            continue;
        }
        // 保留键：追加任意参数（自定义 flag）。前端会用 `__append:...` 这样的唯一键，可连续追加多条
        if edit.key.starts_with(APPEND_KEY) {
            if let Some(raw) = edit.value.clone() {
                let raw = raw.trim();
                if !raw.is_empty() {
                    reps.push((launch.end, launch.end, format!(" {raw}")));
                }
            }
            continue;
        }

        let Some(def_index) = REGISTRY.iter().position(|d| d.key == edit.key) else {
            continue;
        };
        let def = &REGISTRY[def_index];
        let existing = analyzed.args.iter().find(|a| a.def_index == Some(def_index));

        match (edit.present, existing) {
            // 原本有：改值 / 改开关
            (true, Some(item)) => {
                if def.kind == ParamKind::Bool {
                    let desired = edit.value.clone().unwrap_or_else(|| "on".to_string());
                    let current = item.value.clone().unwrap_or_else(|| "on".to_string());
                    if desired == current {
                        continue;
                    }
                    if let Some(span) = item.value_span {
                        // 显式取值写法（`-fa on`）：只改取值
                        reps.push((span.0, span.1, desired));
                    } else if desired == "off" {
                        match off_alias(def) {
                            // 无取值写法 + 有否定别名：换成否定别名
                            Some(off) => reps.push((item.flag_span.0, item.flag_span.1, off)),
                            // 没有否定别名：直接移除该开关
                            None => reps.push((
                                deletion_start(text, item.flag_span.0),
                                item.flag_span.1,
                                String::new(),
                            )),
                        }
                    } else {
                        reps.push((
                            item.flag_span.0,
                            item.flag_span.1,
                            on_alias(def, item.used_flag.as_deref()),
                        ));
                    }
                } else {
                    let value = edit.value.clone().unwrap_or_default();
                    match item.value_span {
                        Some(span) => reps.push((span.0, span.1, quote(&value))),
                        None => reps.push((
                            item.flag_span.0,
                            item.flag_span.1,
                            format!("{} {}", def.flags[0], quote(&value)),
                        )),
                    }
                }
            }
            // 原本没有：追加到命令末尾
            (true, None) => {
                let segment = if def.kind == ParamKind::Bool {
                    let desired = edit.value.clone().unwrap_or_else(|| "on".to_string());
                    if desired == "off" {
                        match off_alias(def) {
                            Some(off) => format!(" {off}"),
                            None => continue, // 没有否定写法的开关，「关」等于不加
                        }
                    } else {
                        format!(" {}", def.flags[0])
                    }
                } else {
                    format!(
                        " {} {}",
                        def.flags[0],
                        quote(&edit.value.clone().unwrap_or_default())
                    )
                };
                reps.push((launch.end, launch.end, segment));
            }
            // 原本有：移除
            (false, Some(item)) => {
                let start = deletion_start(text, item.flag_span.0);
                let end = item.value_span.map(|s| s.1).unwrap_or(item.flag_span.1);
                reps.push((start, end, String::new()));
            }
            (false, None) => {}
        }
    }

    if reps.is_empty() {
        return Some(text.to_string());
    }

    reps.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = text.to_string();
    for (start, end, segment) in reps {
        out.replace_range(start..end, &segment);
    }
    Some(out)
}

/// 替换启动行的引擎可执行文件路径，返回新文本。
pub fn replace_exe(text: &str, new_exe: &str) -> Option<String> {
    let launch = super::locate::locate(text)?;
    let token = launch.tokens.first()?;
    Some(format!(
        "{}{}{}",
        &text[..token.start],
        quote(new_exe),
        &text[token.end..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(key: &str, present: bool, value: Option<&str>) -> Edit {
        Edit {
            key: key.to_string(),
            present,
            value: value.map(|v| v.to_string()),
        }
    }

    #[test]
    fn replaces_value_only() {
        let text = "@echo off\nllama-server -c 4096 --port 8080\n";
        let out = apply_edits(text, &[edit("-c", true, Some("8192"))]).unwrap();
        assert_eq!(out, "@echo off\nllama-server -c 8192 --port 8080\n");
    }

    #[test]
    fn removes_and_appends() {
        let text = "llama-server -c 4096 -t 8\n";
        let out = apply_edits(
            text,
            &[edit("-t", false, None), edit("--port", true, Some("9000"))],
        )
        .unwrap();
        assert_eq!(out, "llama-server -c 4096 --port 9000\n");
    }

    #[test]
    fn quotes_when_needed() {
        let text = "llama-server -m old.gguf\n";
        let out = apply_edits(text, &[edit("-m", true, Some("D:\\my models\\a b.gguf"))]).unwrap();
        assert_eq!(out, "llama-server -m \"D:\\my models\\a b.gguf\"\n");
    }

    #[test]
    fn toggles_bool_to_off() {
        let text = "llama-server -fa\n";
        let out = apply_edits(text, &[edit("--flash-attn", true, Some("off"))]).unwrap();
        assert_eq!(out, "llama-server --no-flash-attn\n");
    }

    #[test]
    fn toggles_explicit_bool_value() {
        let text = "llama-server -fa on --port 8089\n";
        let out = apply_edits(text, &[edit("--flash-attn", true, Some("off"))]).unwrap();
        assert_eq!(out, "llama-server -fa off --port 8089\n");
    }

    #[test]
    fn removes_off_alias_making_bool_added() {
        let text = "llama-server --jinja --port 8089\n";
        let out = apply_edits(text, &[edit("--jinja", false, None)]).unwrap();
        assert_eq!(out, "llama-server --port 8089\n");
    }

    #[test]
    fn replaces_engine_and_keeps_rest() {
        let text = "@echo off\n\"%~dp0llama-server.exe\" -c 4096\n";
        let out = replace_exe(text, "D:\\llama\\cuda\\llama-server.exe").unwrap();
        assert_eq!(out, "@echo off\nD:\\llama\\cuda\\llama-server.exe -c 4096\n");
    }

    #[test]
    fn edits_value_inside_continuation_block() {
        let text = "llama-server.exe ^\n  -m a.gguf ^\n  --host 127.0.0.1 ^\n  --port 8089\n";
        let out = apply_edits(text, &[edit("--port", true, Some("9000"))]).unwrap();
        assert_eq!(
            out,
            "llama-server.exe ^\n  -m a.gguf ^\n  --host 127.0.0.1 ^\n  --port 9000\n"
        );
    }

    #[test]
    fn removes_head_of_continuation_line_merging() {
        let text = "llama-server.exe ^\n  -m a.gguf ^\n  --port 8089\n";
        let out = apply_edits(text, &[edit("-m", false, None)]).unwrap();
        assert_eq!(out, "llama-server.exe ^\n  --port 8089\n");
    }

    #[test]
    fn appends_at_end_of_continuation_block() {
        let text = "llama-server.exe ^\n  -m a.gguf ^\n  --port 8089\n";
        let out = apply_edits(text, &[edit("--jinja", true, None)]).unwrap();
        assert_eq!(
            out,
            "llama-server.exe ^\n  -m a.gguf ^\n  --port 8089 --jinja\n"
        );
    }

    #[test]
    fn replaces_exe_via_reserved_key() {
        let text = "llama-server ^\n  -c 4096\n";
        let out = apply_edits(
            text,
            &[Edit {
                key: EXE_KEY.to_string(),
                present: true,
                value: Some("D:\\llama\\llama-server.exe".to_string()),
            }],
        )
        .unwrap();
        assert_eq!(out, "D:\\llama\\llama-server.exe ^\n  -c 4096\n");
    }

    #[test]
    fn appends_custom_param_via_reserved_key() {
        let text = "llama-server -c 4096\n";
        let out = apply_edits(
            text,
            &[Edit {
                key: APPEND_KEY.to_string(),
                present: true,
                value: Some("--cache-reuse 256".to_string()),
            }],
        )
        .unwrap();
        assert_eq!(out, "llama-server -c 4096 --cache-reuse 256\n");
    }
}
