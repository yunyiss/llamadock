//! 子进程与日志（平台相关逻辑隔离，见技术方案 §4.4）。

pub mod killer;
pub mod reader;
pub mod spawn;

use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;

/// 运行中的进程。
pub(crate) struct Running {
    pub child: Child,
}

/// 进程管理器：按脚本路径索引，因此可以同时运行多个脚本。
#[derive(Default)]
pub struct ProcessManager {
    pub(crate) running: Mutex<HashMap<String, Running>>,
}
