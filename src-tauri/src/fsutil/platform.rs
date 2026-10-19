//! 平台相关：隐藏控制台窗口的命令构造、在文件管理器中打开目录。

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

/// 用系统默认程序打开 URL。
pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    let mut cmd = {
        let mut c = hidden_command("cmd");
        c.args(["/C", "start", "", url]);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = hidden_command("open");
        c.arg(url);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = hidden_command("xdg-open");
        c.arg(url);
        c
    };

    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("打开链接失败：{e}"))
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
