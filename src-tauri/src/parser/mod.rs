//! 脚本解析：定位启动行 → 分词 → 映射参数注册表；无法识别的部分原样保留（见技术方案 §4.2）。

pub mod locate;
pub mod rebuild;
pub mod registry;
pub mod tokenizer;

use serde::Serialize;

use registry::{ParamGroup, ParamKind, REGISTRY};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Launcher {
    /// 引擎可执行文件路径（启动行首个 token）
    pub exe: String,
    pub line_index: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamView {
    pub key: String,
    pub flags: Vec<String>,
    pub kind: ParamKind,
    pub group: ParamGroup,
    pub desc: String,
    pub value: Option<String>,
    /// 脚本中是否出现该参数
    pub present: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptModel {
    pub path: String,
    /// 是否识别到启动行
    pub found: bool,
    pub launcher: Option<Launcher>,
    pub params: Vec<ParamView>,
    /// 无法识别的 token（原样保留）
    pub unknown: Vec<String>,
    pub raw: String,
}

/// 一个已解析的启动参数项，携带行内字节区间（供最小差异重建使用）。
#[derive(Debug, Clone)]
pub struct ArgItem {
    pub def_index: Option<usize>,
    /// 脚本中实际使用的别名
    pub used_flag: Option<String>,
    pub flag_span: (usize, usize),
    pub value: Option<String>,
    pub value_span: Option<(usize, usize)>,
    /// 原始 token 文本
    pub raw: String,
}

pub struct Analyzed {
    pub launch: locate::LaunchLine,
    pub args: Vec<ArgItem>,
}

/// 拆分 `--flag=value`，只取 flag 部分。
fn split_flag(token: &str) -> &str {
    match token.find('=') {
        Some(eq) => &token[..eq],
        None => token,
    }
}

fn inline_value(token: &str) -> Option<String> {
    token.find('=').map(|eq| token[eq + 1..].to_string())
}

/// 布尔参数的显式取值（如 `-fa on`）。
fn bool_value(token: &str) -> Option<&'static str> {
    match token.to_ascii_lowercase().as_str() {
        "on" | "true" | "1" => Some("on"),
        "off" | "false" | "0" => Some("off"),
        "auto" => Some("auto"),
        _ => None,
    }
}

/// 分析脚本：定位启动行并把参数映射到注册表（保留未知项）。
pub fn analyze(text: &str) -> Option<Analyzed> {
    let launch = locate::locate(text)?;
    let tokens = &launch.tokens;
    let mut args = Vec::new();
    let mut i = 1;

    while i < tokens.len() {
        let tok = &tokens[i];
        let raw_text = tok.text.as_str();

        if !raw_text.starts_with('-') {
            args.push(ArgItem {
                def_index: None,
                used_flag: None,
                flag_span: (tok.start, tok.end),
                value: None,
                value_span: None,
                raw: raw_text.to_string(),
            });
            i += 1;
            continue;
        }

        let flag = split_flag(raw_text);
        let mut index = registry::index_of(flag);
        let mut inline = inline_value(raw_text);

        // 兼容 `-c4096` 这类短参数紧贴取值的写法
        if index.is_none() {
            'find: for (ri, def) in REGISTRY.iter().enumerate() {
                for &f in def.flags {
                    if f.len() == 2 && raw_text.starts_with(f) && raw_text.len() > 2 {
                        let rest = &raw_text[2..];
                        // 仅当紧跟数字时才算紧贴取值（如 `-c4096`），避免把 `-min-p` 误判成 `-m`
                        if rest.starts_with(|c: char| c.is_ascii_digit()) {
                            index = Some(ri);
                            inline = Some(rest.to_string());
                            break 'find;
                        }
                    }
                }
            }
        }

        match index {
            Some(ri) => {
                let def = &REGISTRY[ri];
                let (value, value_span) = match def.kind {
                    ParamKind::Bool => {
                        // 显式取值写法：`-fa on` / `-fa off`
                        if let Some(next) = tokens.get(i + 1).filter(|t| bool_value(&t.text).is_some())
                        {
                            i += 1;
                            let v = bool_value(&next.text).unwrap_or("on").to_string();
                            (Some(v), Some((next.start, next.end)))
                        } else {
                            let v = if raw_text.starts_with("--no-") {
                                "off".to_string()
                            } else {
                                inline.clone().unwrap_or_else(|| "on".to_string())
                            };
                            (Some(v), None)
                        }
                    }
                    _ => match inline {
                        Some(v) => (Some(v), None),
                        None => {
                            if i + 1 < tokens.len() && !tokens[i + 1].text.starts_with('-') {
                                i += 1;
                                let next = &tokens[i];
                                (Some(next.text.clone()), Some((next.start, next.end)))
                            } else {
                                (None, None)
                            }
                        }
                    },
                };
                args.push(ArgItem {
                    def_index: Some(ri),
                    used_flag: Some(raw_text.to_string()),
                    flag_span: (tok.start, tok.end),
                    value,
                    value_span,
                    raw: raw_text.to_string(),
                });
            }
            None => {
                // 未知 flag：连同其后紧跟的非 flag token 一起原样保留
                args.push(ArgItem {
                    def_index: None,
                    used_flag: None,
                    flag_span: (tok.start, tok.end),
                    value: None,
                    value_span: None,
                    raw: raw_text.to_string(),
                });
                if inline.is_none() && i + 1 < tokens.len() && !tokens[i + 1].text.starts_with('-') {
                    i += 1;
                    let next = &tokens[i];
                    args.push(ArgItem {
                        def_index: None,
                        used_flag: None,
                        flag_span: (next.start, next.end),
                        value: None,
                        value_span: None,
                        raw: next.text.clone(),
                    });
                }
            }
        }

        i += 1;
    }

    Some(Analyzed { launch, args })
}

pub fn parse(path: &str, text: &str) -> ScriptModel {
    let raw = text.to_string();

    let Some(analyzed) = analyze(text) else {
        return ScriptModel {
            path: path.to_string(),
            found: false,
            launcher: None,
            params: Vec::new(),
            unknown: Vec::new(),
            raw,
        };
    };

    let exe = analyzed
        .launch
        .tokens
        .first()
        .map(|t| t.text.clone())
        .unwrap_or_default();

    let mut matched: Vec<Option<Option<String>>> = vec![None; REGISTRY.len()];
    let mut unknown = Vec::new();
    for item in &analyzed.args {
        match item.def_index {
            Some(ri) => matched[ri] = Some(item.value.clone()),
            None => unknown.push(item.raw.clone()),
        }
    }

    let params = REGISTRY
        .iter()
        .enumerate()
        .map(|(ri, def)| ParamView {
            key: def.key.to_string(),
            flags: def.flags.iter().map(|f| f.to_string()).collect(),
            kind: def.kind,
            group: def.group,
            desc: def.desc.to_string(),
            value: matched[ri].clone().flatten(),
            present: matched[ri].is_some(),
        })
        .collect();

    ScriptModel {
        path: path.to_string(),
        found: true,
        launcher: Some(Launcher {
            exe,
            line_index: analyzed.launch.line_index,
        }),
        params,
        unknown,
        raw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_of(model: &ScriptModel, key: &str) -> Option<String> {
        model
            .params
            .iter()
            .find(|p| p.key == key)
            .and_then(|p| p.value.clone())
    }

    #[test]
    fn parses_cmd_with_quotes_and_prefix() {
        let text = "@echo off\n\"%~dp0llama-server.exe\" -m \"D:\\models\\a.gguf\" -c 4096 --port 8080 -ngl 99\n";
        let model = parse("t.cmd", text);

        assert!(model.found);
        assert_eq!(
            model.launcher.as_ref().unwrap().exe,
            "%~dp0llama-server.exe"
        );
        assert_eq!(value_of(&model, "-m").unwrap(), "D:\\models\\a.gguf");
        assert_eq!(value_of(&model, "-c").unwrap(), "4096");
        assert_eq!(value_of(&model, "--port").unwrap(), "8080");
        assert_eq!(value_of(&model, "-ngl").unwrap(), "99");
    }

    #[test]
    fn handles_attached_value_bool_and_unknown() {
        let text = "llama-server -c4096 -fa --foo bar --custom-flag\n";
        let model = parse("t.sh", text);

        assert_eq!(value_of(&model, "-c").unwrap(), "4096");
        assert_eq!(value_of(&model, "--flash-attn").unwrap(), "on");
        assert!(model.unknown.contains(&"--foo".to_string()));
        assert!(model.unknown.contains(&"bar".to_string()));
        assert!(model.unknown.contains(&"--custom-flag".to_string()));
    }

    #[test]
    fn no_launch_line_is_safe() {
        let model = parse("t.sh", "#!/bin/sh\necho hello\n");
        assert!(!model.found);
        assert!(model.params.is_empty());
    }

    #[test]
    fn ignores_comments() {
        let text = ":: comment\nrem another\nllama-cli -t 8\n";
        let model = parse("t.cmd", text);
        assert!(model.found);
        assert_eq!(value_of(&model, "-t").unwrap(), "8");
    }

    #[test]
    fn parses_cmd_with_line_continuations() {
        let text = "@echo off\nllama-server.exe ^\n  -m \"a b.gguf\" ^\n  -fa on ^\n  --host 127.0.0.1 ^\n  --port 8089\n";
        let model = parse("t.cmd", text);

        assert!(model.found);
        assert_eq!(value_of(&model, "-m").unwrap(), "a b.gguf");
        assert_eq!(value_of(&model, "--flash-attn").unwrap(), "on");
        assert_eq!(value_of(&model, "--host").unwrap(), "127.0.0.1");
        assert_eq!(value_of(&model, "--port").unwrap(), "8089");
        assert!(model.unknown.is_empty());
    }

    #[test]
    fn parses_script_with_comments_and_checks() {
        let text = "@echo off\nsetlocal\ncd /d \"%~dp0\"\nwhere llama-server.exe >nul 2>nul || (\n    echo [ERROR] 未找到\n    pause\n    exit /b 1\n)\nrem ==========\nllama-server.exe ^\n  -m \"Qwen3.5-9B-UD-Q4_K_XL.gguf\" ^\n  -c 49152 ^\n  --port 8089\n";
        let model = parse("t.cmd", text);

        assert!(model.found);
        assert_eq!(value_of(&model, "-c").unwrap(), "49152");
        assert_eq!(value_of(&model, "--port").unwrap(), "8089");
    }

    #[test]
    fn short_prefix_is_not_treated_as_attached_value() {
        // `-min-p` 不能被误判成 `-m in-p`
        let text = "llama-server -min-p 0.1 -c4096\n";
        let model = parse("t.cmd", text);

        assert_eq!(value_of(&model, "-m"), None);
        assert_eq!(value_of(&model, "-c").unwrap(), "4096");
    }
}
