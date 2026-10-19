//! 引擎路径识别与多版本切换（见技术方案 §4.3）。

use crate::config::{self, EnginePath};
use crate::fsutil::encoding;
use crate::parser::{self, rebuild};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

const SCRIPT_EXTS: [&str; 3] = ["cmd", "bat", "sh"];

/// 已保存的引擎版本列表。
#[tauri::command]
pub fn list_engines(app: AppHandle) -> Vec<EnginePath> {
    config::load(&app).engine_paths
}

/// 新增或更新一个引擎版本。
#[tauri::command]
pub fn save_engine(app: AppHandle, name: String, path: String) -> Result<Vec<EnginePath>, String> {
    let mut cfg = config::load(&app);
    cfg.upsert_engine(&name, &path);
    config::save(&app, &cfg)?;
    Ok(cfg.engine_paths)
}

/// 移除一个引擎版本。
#[tauri::command]
pub fn remove_engine(app: AppHandle, name: String) -> Result<Vec<EnginePath>, String> {
    let mut cfg = config::load(&app);
    cfg.remove_engine(&name);
    config::save(&app, &cfg)?;
    Ok(cfg.engine_paths)
}

/// 记录当前选中的引擎版本。
#[tauri::command]
pub fn set_current_engine(app: AppHandle, name: Option<String>) -> Result<(), String> {
    let mut cfg = config::load(&app);
    cfg.current_engine = name;
    config::save(&app, &cfg)
}

/// 选择引擎可执行文件。
#[tauri::command]
pub async fn pick_engine_file(app: AppHandle) -> Result<Option<String>, String> {
    let Some(file) = app.dialog().file().blocking_pick_file() else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    Ok(Some(path.to_string_lossy().to_string()))
}

/// 替换当前脚本的引擎路径，返回新文本（交由差异确认流程处理）。
#[tauri::command]
pub fn switch_engine_text(path: String, new_exe: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let (text, _enc) = encoding::decode(&bytes);
    rebuild::replace_exe(&text, &new_exe)
        .ok_or_else(|| "未识别到 llama.cpp 启动行，请改用原始脚本编辑器".to_string())
}

/// 批量替换工作目录中引用同一引擎的脚本（直接写盘），返回改动数量。
#[tauri::command]
pub fn switch_engine_all(app: AppHandle, old_exe: String, new_exe: String) -> Result<u32, String> {
    let Some(dir) = config::load(&app).work_dir else {
        return Ok(0);
    };

    let mut changed = 0u32;
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

        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let (text, enc) = encoding::decode(&bytes);
        let model = parser::parse(&path.to_string_lossy(), &text);
        let Some(launcher) = model.launcher else {
            continue;
        };
        if launcher.exe != old_exe {
            continue;
        }

        let Some(updated) = rebuild::replace_exe(&text, &new_exe) else {
            continue;
        };
        std::fs::write(&path, encoding::encode(&updated, enc)).map_err(|e| e.to_string())?;
        changed += 1;
    }

    Ok(changed)
}
