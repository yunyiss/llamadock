//! 脚本相关命令：解析参数 / 差异预览 / 保存脚本。

use crate::fsutil::encoding;
use crate::parser::{self, ScriptModel};

/// 解析脚本中的 llama.cpp 启动参数。
#[tauri::command]
pub fn parse_script(path: String) -> Result<ScriptModel, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let (text, _enc) = encoding::decode(&bytes);
    Ok(parser::parse(&path, &text))
}
