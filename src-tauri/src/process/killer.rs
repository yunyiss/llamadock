//! 进程树终止：Windows 用 taskkill /T /F；Unix 向进程组发 SIGTERM，超时升级 SIGKILL。

use std::process::Child;

/// 结束进程及其子进程。
#[cfg(windows)]
pub fn kill_tree(child: &mut Child) -> Result<(), String> {
    let pid = child.id().to_string();
    let _ = crate::fsutil::platform::hidden_command("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .status();
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

/// 结束进程组。
#[cfg(unix)]
pub fn kill_tree(child: &mut Child) -> Result<(), String> {
    let pgid = child.id() as i32;

    unsafe {
        libc::kill(-pgid, libc::SIGTERM);
    }
    for _ in 0..30 {
        if let Ok(Some(_)) = child.try_wait() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    unsafe {
        libc::kill(-pgid, libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}
