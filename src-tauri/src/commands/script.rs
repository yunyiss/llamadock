//! 脚本相关命令：解析参数 / 重建 / 保存脚本。

use crate::fsutil::encoding;
use crate::parser::rebuild::{self, Edit};
use crate::parser::{self, ScriptModel};

/// 解析脚本中的 llama.cpp 启动参数。
#[tauri::command]
pub fn parse_script(path: String) -> Result<ScriptModel, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let (text, _enc) = encoding::decode(&bytes);
    Ok(parser::parse(&path, &text))
}

/// 解析内存中的文本（供原始编辑器与参数面板双向同步）。
#[tauri::command]
pub fn parse_text(path: String, text: String) -> ScriptModel {
    parser::parse(&path, &text)
}

/// 按改动重建脚本文本（只返回文本，不写盘）。
#[tauri::command]
pub fn rebuild_script(path: String, edits: Vec<Edit>) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let (text, _enc) = encoding::decode(&bytes);
    rebuild::apply_edits(&text, &edits)
        .ok_or_else(|| "未识别到 llama.cpp 启动行，请改用原始脚本编辑器".to_string())
}

/// 写回脚本：保留原编码、校验文件未被外部改动、原子写。
#[tauri::command]
pub fn save_script(path: String, content: String, base: String) -> Result<(), String> {
    let current_bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let (current, enc) = encoding::decode(&current_bytes);
    if current != base {
        return Err("文件已被外部修改，请重新解析后再保存".to_string());
    }

    let bytes = encoding::encode(&content, enc);
    let tmp = format!("{path}.tmp");
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}
