//! 平台相关：隐藏控制台窗口的命令构造、在文件管理器中打开目录。
//!
//! 打开链接统一走 `open` crate（Windows 下 ShellExecute），不要用 `cmd /C start`——那会把 URL 交给 cmd 解析。

use std::path::Path;
use std::process::Command;

/// 构造一个不会弹出控制台窗口的命令（Windows 下设置 `CREATE_NO_WINDOW`）。
pub fn hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// 在系统文件管理器中打开目录。
pub fn open_folder(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";

    hidden_command(program)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("打开文件夹失败：{e}"))
}
