//! 配置相关命令。

use crate::config::{self, Config};
use tauri::AppHandle;

#[tauri::command]
pub fn get_config(app: AppHandle) -> Config {
    config::load(&app)
}

/// 保存主题偏好（auto / light / dark）；原生窗口外观由前端通过 window API 同步。
#[tauri::command]
pub fn save_theme(app: AppHandle, theme: String) -> Result<(), String> {
    if !matches!(theme.as_str(), "auto" | "light" | "dark") {
        return Err(format!("未知的主题：{theme}"));
    }
    let mut cfg = config::load(&app);
    cfg.theme = Some(theme);
    config::save(&app, &cfg)
}
