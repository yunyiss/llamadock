//! 平台相关启动：Windows 用 cmd.exe / powershell.exe 且不弹控制台窗口；Unix 用 sh / pwsh 并建立独立进程组。

use std::path::Path;
use std::process::{Child, Command, Stdio};

/// 是否为 PowerShell 脚本（按扩展名判断，不区分大小写）。
fn is_powershell(script: &Path) -> bool {
    script
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ps1") || e.eq_ignore_ascii_case("psm1"))
}

/// 构造启动命令。
#[cfg(windows)]
pub fn build(script: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut cmd = if is_powershell(script) {
        let mut c = Command::new("powershell.exe");
        c.arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(script);
        c
    } else {
        let mut c = Command::new("cmd.exe");
        c.arg("/C").arg(script);
        c
    };
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// 构造启动命令：Unix 下把子进程放进独立进程组，便于整组终止。
#[cfg(unix)]
pub fn build(script: &Path) -> Command {
    // pre_exec 由该 trait 提供；缺少导入时只有非 Windows 目标才会编译失败
    use std::os::unix::process::CommandExt;

    let mut cmd = if is_powershell(script) {
        let mut c = Command::new("pwsh");
        c.arg("-NoProfile").arg("-File").arg(script);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg(script);
        c
    };
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    cmd
}

/// 启动脚本，接管其 stdout / stderr。
///
/// 工作目录设为**脚本所在目录**（等价于双击脚本执行），这样脚本里的相对路径
/// （如 `-m xxx.gguf`）才能被 llama.cpp 正确解析。
pub fn spawn(script: &Path) -> std::io::Result<Child> {
    let mut cmd = build(script);
    if let Some(dir) = script.parent() {
        cmd.current_dir(dir);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
