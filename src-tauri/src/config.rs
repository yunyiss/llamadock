//! 配置读写：便携模式优先（程序目录），不可写时回退系统配置目录；原子写。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// 当前工作目录
    pub work_dir: Option<String>,
    /// 最近使用的工作目录（最新在最前，最多 5 个）
    pub recent_dirs: Vec<String>,
}

impl Config {
    /// 记录一个最近使用的目录：去重、置顶、截断。
    pub fn remember_dir(&mut self, dir: &str) {
        self.recent_dirs.retain(|d| d != dir);
        self.recent_dirs.insert(0, dir.to_string());
        self.recent_dirs.truncate(5);
        self.work_dir = Some(dir.to_string());
    }
}

/// 解析配置文件路径：程序目录可写则用便携模式，否则回退系统配置目录。
fn config_path(app: &AppHandle) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("config.json").exists() || dir_is_writable(dir) {
                return dir.join("config.json");
            }
        }
    }
    app.path()
        .app_config_dir()
        .map(|d| d.join("config.json"))
        .unwrap_or_else(|_| PathBuf::from("config.json"))
}

fn dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(".write-probe");
    match fs::write(&probe, b"") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// 读取配置；文件不存在或损坏时返回默认值。
pub fn load(app: &AppHandle) -> Config {
    fs::read_to_string(config_path(app))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// 原子写回配置（临时文件 + rename）。
pub fn save(app: &AppHandle, cfg: &Config) -> Result<(), String> {
    let path = config_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}
