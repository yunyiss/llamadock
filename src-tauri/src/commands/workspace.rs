//! 工作目录与脚本扫描。

use crate::config;
use crate::fsutil::platform;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// 支持的脚本扩展名。
const SCRIPT_EXTS: [&str; 3] = ["cmd", "bat", "sh"];

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
