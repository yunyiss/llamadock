//! 系统级小工具：日志导出。

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// 弹出保存对话框并导出日志；用户取消时返回 None。
#[tauri::command]
pub async fn export_log(app: AppHandle, name: String, content: String) -> Result<Option<String>, String> {
    let Some(target) = app
        .dialog()
        .file()
        .set_file_name(&name)
        .add_filter("文本文件", &["txt", "log"])
        .blocking_save_file()
    else {
        return Ok(None);
    };

    let path = target.into_path().map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| format!("写入失败：{e}"))?;
    Ok(Some(path.to_string_lossy().to_string()))
}
