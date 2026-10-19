//! Tauri 命令层（IPC 边界）：只做参数校验与编排，业务逻辑下沉到各模块。
//!
//! 计划对应命令（见技术方案 §4.7）：
//! - workspace：扫描 / 新增 / 删除 / 重命名脚本、打开文件夹
//! - script：解析参数 / 差异预览 / 保存脚本
//! - engine：引擎路径识别与切换
//! - process：启动 / 停止 / 查询状态
//! - config：配置读写

pub mod config;
pub mod script;
pub mod workspace;

/// 连通性检查：验证前后端 IPC 通路（P0 阶段用于联调）。
#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}
