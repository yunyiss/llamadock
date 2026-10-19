//! 工作目录与脚本扫描。

use crate::config;
use crate::fsutil::platform;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// 支持的脚本扩展名。
const SCRIPT_EXTS: [&str; 4] = ["cmd", "bat", "sh", "ps1"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEntry {
    pub path: String,
    pub name: String,
    pub ext: String,
}

/// 选择工作目录：弹出文件夹选择器，记住到配置并返回。
#[tauri::command]
pub async fn pick_work_dir(app: AppHandle) -> Result<Option<String>, String> {
    let Some(folder) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };
    let dir = folder.into_path().map_err(|e| e.to_string())?;
    let dir_str = dir.to_string_lossy().to_string();

    let mut cfg = config::load(&app);
    cfg.remember_dir(&dir_str);
    config::save(&app, &cfg)?;
    Ok(Some(dir_str))
}

/// 扫描当前工作目录下的脚本文件。
#[tauri::command]
pub fn scan_scripts(app: AppHandle) -> Result<Vec<ScriptEntry>, String> {
    let Some(dir) = config::load(&app).work_dir else {
        return Ok(Vec::new());
    };
    let dir = PathBuf::from(dir);
    if !dir.is_dir() {
        return Err("工作目录不存在或不可访问".to_string());
    }

    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !SCRIPT_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        out.push(ScriptEntry {
            path: path.to_string_lossy().to_string(),
            name,
            ext,
        });
    }
    out.sort_by_key(|s| s.name.to_lowercase());
    Ok(out)
}

/// 新建脚本支持的类型。
const CREATE_EXTS: [&str; 4] = ["cmd", "bat", "sh", "ps1"];

/// 规范化脚本文件名：只有名称已经带受支持的扩展名时才替换它，
/// 其余情况直接追加所选类型的扩展名（避免把 `Qwen3.5-9B` 这类含点的名字截断）。
fn normalize_script_name(name: &str, kind: &str) -> Result<String, String> {
    let base = name.trim();
    if base.is_empty() {
        return Err("请填写脚本名称".to_string());
    }
    if base.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("脚本名称不能包含路径分隔符或非法字符".to_string());
    }
    let lower = base.to_lowercase();
    let stem = CREATE_EXTS
        .iter()
        .find_map(|ext| {
            lower
                .ends_with(&format!(".{ext}"))
                .then(|| &base[..base.len() - ext.len() - 1])
        })
        .unwrap_or(base);
    Ok(format!("{stem}.{kind}"))
}

/// 各类型的最小启动脚本模板；返回（内容，行尾）。
///
/// 批处理与 PowerShell 用 CRLF（cmd 对 LF 的行尾续行处理不可靠），shell 用 LF。
/// 文案刻意保持 ASCII：批处理默认代码页不是 UTF-8，中文注释在 cmd 里会乱码。
fn script_template(kind: &str) -> (&'static str, &'static [u8]) {
    match kind {
        "sh" => (
            "#!/bin/sh\n# LlamaDock minimal llama.cpp launch script\ncd \"$(dirname \"$0\")\"\n\nllama-server \\\n  -m model.gguf \\\n  -c 8192 \\\n  -ngl 99 \\\n  --host 127.0.0.1 \\\n  --port 8080",
            b"\n",
        ),
        "ps1" => (
            "# LlamaDock minimal llama.cpp launch script\nSet-Location -Path $PSScriptRoot\n\n& llama-server.exe `\n  -m \"model.gguf\" `\n  -c 8192 `\n  -ngl 99 `\n  --host 127.0.0.1 `\n  --port 8080",
            b"\r\n",
        ),
        _ => (
            "@echo off\nrem LlamaDock minimal llama.cpp launch script\nsetlocal\ncd /d \"%~dp0\"\n\nllama-server.exe ^\n  -m \"model.gguf\" ^\n  -c 8192 ^\n  -ngl 99 ^\n  --host 127.0.0.1 ^\n  --port 8080",
            b"\r\n",
        ),
    }
}

/// 在工作目录里按模板新建一个启动脚本，返回新文件的绝对路径。
#[tauri::command]
pub fn create_script(app: AppHandle, name: String, kind: String) -> Result<String, String> {
    let kind = kind.trim().to_lowercase();
    if !CREATE_EXTS.contains(&kind.as_str()) {
        return Err(format!("不支持的脚本类型：{kind}"));
    }

    let Some(dir) = config::load(&app).work_dir else {
        return Err("请先选择工作目录".to_string());
    };
    let dir = PathBuf::from(dir);
    if !dir.is_dir() {
        return Err("工作目录不存在或不可访问".to_string());
    }

    let file_name = normalize_script_name(&name, &kind)?;
    let path = dir.join(&file_name);
    if path.exists() {
        return Err(format!("已存在同名文件：{file_name}"));
    }

    let (content, newline) = script_template(&kind);
    let mut bytes = Vec::new();
    for line in content.lines() {
        bytes.extend_from_slice(line.as_bytes());
        bytes.extend_from_slice(newline);
    }
    std::fs::write(&path, bytes).map_err(|e| format!("写入失败：{e}"))?;

    // Unix 下给 shell 脚本加可执行位，手动 ./xxx.sh 也能直接跑
    #[cfg(unix)]
    if kind == "sh" {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
    }

    Ok(path.to_string_lossy().to_string())
}

/// 用系统默认程序打开 URL（仅允许 http/https，用于日志里的链接点击）。
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("仅支持 http/https 链接".to_string());
    }
    platform::open_url(&url)
}

/// 判断路径是否存在（用于引擎 / 模型失效提示）。
#[tauri::command]
pub fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}

/// 在系统文件管理器中打开指定目录。
#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    platform::open_folder(Path::new(&path))
}

#[cfg(test)]
mod tests {
    use super::normalize_script_name;

    #[test]
    fn keeps_dotted_names() {
        // 名字里本来就带点（Qwen3.5-9B）不能被当成扩展名截断
        assert_eq!(
            normalize_script_name("Qwen3.5-9B", "cmd").unwrap(),
            "Qwen3.5-9B.cmd"
        );
    }

    #[test]
    fn replaces_known_extension() {
        assert_eq!(normalize_script_name("run.sh", "ps1").unwrap(), "run.ps1");
        assert_eq!(normalize_script_name("run", "sh").unwrap(), "run.sh");
        assert_eq!(
            normalize_script_name("  spaced  ", "bat").unwrap(),
            "spaced.bat"
        );
    }

    #[test]
    fn rejects_invalid_names() {
        assert!(normalize_script_name("", "cmd").is_err());
        assert!(normalize_script_name("   ", "cmd").is_err());
        assert!(normalize_script_name("a/b", "cmd").is_err());
        assert!(normalize_script_name("a\\b", "cmd").is_err());
    }
}
