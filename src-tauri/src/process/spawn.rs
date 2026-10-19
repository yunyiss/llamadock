//! 平台相关启动：Windows 用 cmd.exe 且不弹控制台窗口；Unix 用 sh 并建立独立进程组。

use std::path::Path;
use std::process::{Child, Command, Stdio};

/// 构造启动命令。
#[cfg(windows)]
pub fn build(script: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut cmd = Command::new("cmd.exe");
    cmd.arg("/C").arg(script);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// 构造启动命令：Unix 下把子进程放进独立进程组，便于整组终止。
#[cfg(unix)]
pub fn build(script: &Path) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg(script);
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    cmd
}

/// 启动脚本，接管其 stdout / stderr。
pub fn spawn(script: &Path) -> std::io::Result<Child> {
    let mut cmd = build(script);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
