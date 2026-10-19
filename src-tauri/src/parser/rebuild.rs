//! 最小差异重建：按 token 字节区间从后往前替换 / 插入 / 删除，其余文本原样保留（见技术方案 §4.2）。

use serde::Deserialize;

use super::registry::{ParamDef, ParamKind, REGISTRY};
use super::analyze;

/// 前端提交的单条参数改动。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub key: String,
    pub present: bool,
    pub value: Option<String>,
}

/// 取指定行「内容」在原文中的字节区间（不含行尾 CR/LF）。
fn line_bounds(text: &str, index: usize) -> Option<(usize, usize)> {
    let mut pos = 0;
    for (i, raw) in text.split_inclusive('\n').enumerate() {
        let mut content = raw.strip_suffix('\n').unwrap_or(raw);
        content = content.strip_suffix('\r').unwrap_or(content);
        if i == index {
            return Some((pos, pos + content.len()));
        }
        pos += raw.len();
    }
    None
}

/// 值按需加引号（含空格或引号时）。
fn quote(value: &str) -> String {
    if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{value}\"")
    } else {
        value.to_string()
    }
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
    let (line_start, line_end) = line_bounds(text, analyzed.launch.line_index)?;
    let line = &text[line_start..line_end];

    // (字节起点, 字节终点, 新片段)，稍后从后往前应用
    let mut reps: Vec<(usize, usize, String)> = Vec::new();

    for edit in edits {
        let Some(def_index) = REGISTRY.iter().position(|d| d.key == edit.key) else {
            continue;
        };
        let def = &REGISTRY[def_index];
        let existing = analyzed
            .args
            .iter()
            .find(|a| a.def_index == Some(def_index));

        match (edit.present, existing) {
            // 原本有，改值 / 改开关
            (true, Some(item)) => {
                if def.kind == ParamKind::Bool {
                    let desired = edit.value.clone().unwrap_or_else(|| "on".to_string());
                    let current = item.value.clone().unwrap_or_else(|| "on".to_string());
                    if desired != current {
                        let flag = if desired == "off" {
                            off_alias(def).unwrap_or_else(|| def.flags[0].to_string())
                        } else {
                            on_alias(def, item.used_flag.as_deref())
                        };
                        reps.push((item.flag_span.0, item.flag_span.1, flag));
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
            // 原本没有，追加
            (true, None) => {
                let seg = if def.kind == ParamKind::Bool {
                    let desired = edit.value.clone().unwrap_or_else(|| "on".to_string());
                    let flag = if desired == "off" {
                        off_alias(def).unwrap_or_else(|| def.flags[0].to_string())
                    } else {
                        def.flags[0].to_string()
                    };
                    format!(" {flag}")
                } else {
                    format!(
                        " {} {}",
                        def.flags[0],
                        quote(&edit.value.clone().unwrap_or_default())
                    )
                };
                reps.push((line.len(), line.len(), seg));
            }
            // 原本有，移除（连带前导空格）
            (false, Some(item)) => {
                let start = item.flag_span.0;
                let end = item.value_span.map(|s| s.1).unwrap_or(item.flag_span.1);
                let del_start = if start > 0 && line.as_bytes()[start - 1] == b' ' {
                    start - 1
                } else {
                    start
                };
                reps.push((del_start, end, String::new()));
            }
            (false, None) => {}
        }
    }

    if reps.is_empty() {
        return Some(text.to_string());
    }

    reps.sort_by(|a, b| b.0.cmp(&a.0));
    let mut new_line = line.to_string();
    for (start, end, segment) in reps {
        new_line.replace_range(start..end, &segment);
    }

    Some(format!(
        "{}{}{}",
        &text[..line_start],
        new_line,
        &text[line_end..]
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
}
