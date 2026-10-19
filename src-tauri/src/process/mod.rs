//! 子进程与日志（平台相关逻辑隔离，见技术方案 §4.4）。
//!
//! 计划子模块：
//! - spawn：平台相关启动（Windows `CREATE_NO_WINDOW` 不弹窗 / Unix 独立进程组）
//! - reader：stdout/stderr 流式读取 + 批量合并推送
//! - killer：进程树终止（Windows Job Object / Unix 进程组）
