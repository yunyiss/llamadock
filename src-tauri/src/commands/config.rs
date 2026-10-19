//! 配置相关命令。

use crate::config::{self, Config};
use tauri::AppHandle;

#[tauri::command]
pub fn get_config(app: AppHandle) -> Config {
    config::load(&app)
}
