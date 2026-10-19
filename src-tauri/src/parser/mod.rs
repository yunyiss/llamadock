//! 脚本解析：定位启动行 → 分词 → 映射参数注册表；无法识别的部分原样保留（见技术方案 §4.2）。

pub mod locate;
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

/// 拆分 `--flag=value`；返回 (flag, 内联值)。
fn split_flag(token: &str) -> &str {
    match token.find('=') {
        Some(eq) => &token[..eq],
        None => token,
    }
}

fn inline_value(token: &str) -> Option<String> {
    token.find('=').map(|eq| token[eq + 1..].to_string())
}

pub fn parse(path: &str, text: &str) -> ScriptModel {
    let raw = text.to_string();

    let Some(launch) = locate::locate(text) else {
        return ScriptModel {
            path: path.to_string(),
            found: false,
            launcher: None,
            params: Vec::new(),
            unknown: Vec::new(),
            raw,
        };
    };

    let exe = launch
        .tokens
        .first()
        .map(|t| t.text.clone())
        .unwrap_or_default();

    // 每个注册项 → Option<Option<值>>：外层 Some 表示出现，内层为取值
    let mut matched: Vec<Option<Option<String>>> = vec![None; REGISTRY.len()];
    let mut unknown: Vec<String> = Vec::new();

    let args = &launch.tokens[1..];
    let mut i = 0;
    while i < args.len() {
        let text = args[i].text.as_str();

        if !text.starts_with('-') {
            unknown.push(text.to_string());
            i += 1;
            continue;
        }

        let flag = split_flag(text);
        let mut index = registry::index_of(flag);
        let mut value = inline_value(text);

        // 兼容 `-c4096` 这类短参数紧贴取值的写法
        if index.is_none() {
            'find: for (ri, def) in REGISTRY.iter().enumerate() {
                for &f in def.flags {
                    if f.len() == 2 && text.starts_with(f) && text.len() > 2 {
                        index = Some(ri);
                        value = Some(text[f.len()..].to_string());
                        break 'find;
                    }
                }
            }
        }

        match index {
            Some(ri) => {
                let def = &REGISTRY[ri];
                let resolved = match def.kind {
                    ParamKind::Bool => Some(value.unwrap_or_else(|| {
                        if text == "--no-flash-attn" {
                            "off".to_string()
                        } else {
                            "on".to_string()
                        }
                    })),
                    _ => match value {
                        Some(v) => Some(v),
                        None => {
                            if i + 1 < args.len() && !args[i + 1].text.starts_with('-') {
                                i += 1;
                                Some(args[i].text.clone())
                            } else {
                                None
                            }
                        }
                    },
                };
                matched[ri] = Some(resolved);
            }
            None => {
                // 未知 flag：连同其后紧跟的非 flag token 一起原样保留
                unknown.push(flag.to_string());
                if let Some(v) = inline_value(text) {
                    unknown.push(v);
                } else if i + 1 < args.len() && !args[i + 1].text.starts_with('-') {
                    i += 1;
                    unknown.push(args[i].text.clone());
                }
            }
        }

        i += 1;
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
            line_index: launch.line_index,
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
        let text = "llama-server -c4096 -fa --foo bar --jinja\n";
        let model = parse("t.sh", text);

        assert_eq!(value_of(&model, "-c").unwrap(), "4096");
        assert_eq!(value_of(&model, "--flash-attn").unwrap(), "on");
        assert!(model.unknown.contains(&"--foo".to_string()));
        assert!(model.unknown.contains(&"bar".to_string()));
        assert!(model.unknown.contains(&"--jinja".to_string()));
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
}
