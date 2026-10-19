//! 子进程与日志（平台相关逻辑隔离，见技术方案 §4.4）。

pub mod killer;
pub mod reader;
pub mod spawn;

use std::process::Child;
use std::sync::Mutex;

/// 运行中的进程（MVP 同时只运行一个脚本）。
pub(crate) struct Running {
    pub child: Child,
}

/// 单一进程管理器，作为 Tauri 托管状态使用。
#[derive(Default)]
pub struct ProcessManager {
    pub(crate) running: Mutex<Option<Running>>,
}
