//! 进程启动 / 停止 / 状态推送（见技术方案 §4.4）。按脚本路径管理，支持同时运行多个脚本。

use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::process::{killer, reader, spawn, ProcessManager, Running};

#[derive(Clone, serde::Serialize)]
pub struct ProcessStatePayload {
    /// 归属的脚本路径
    pub script: String,
    pub state: String,
    pub pid: Option<u32>,
    pub code: Option<i32>,
}

/// 启动指定脚本：不弹控制台窗口，接管输出并推送状态。
#[tauri::command]
pub fn start_process(
    app: AppHandle,
    state: State<ProcessManager>,
    script: String,
) -> Result<u32, String> {
    {
        let mut guard = state.running.lock().map_err(|_| "进程管理器加锁失败")?;
        let alive = match guard.get_mut(&script) {
            Some(running) => matches!(running.child.try_wait(), Ok(None)),
            None => false,
        };
        if alive {
            return Err("该脚本已在运行，请先停止".to_string());
        }
        guard.remove(&script);
    }

    let mut child = spawn::spawn(Path::new(&script)).map_err(|e| format!("启动失败：{e}"))?;
    let pid = child.id();

    if let Some(out) = child.stdout.take() {
        reader::pump(app.clone(), script.clone(), "stdout", out);
    }
    if let Some(err) = child.stderr.take() {
        reader::pump(app.clone(), script.clone(), "stderr", err);
    }

    {
        let mut guard = state.running.lock().map_err(|_| "进程管理器加锁失败")?;
        guard.insert(script.clone(), Running { child });
    }

    emit_state(&app, &script, "running", Some(pid), None);
    watch_exit(app, script);
    Ok(pid)
}

/// 停止指定脚本的运行进程（含子树）。
#[tauri::command]
pub fn stop_process(app: AppHandle, state: State<ProcessManager>, script: String) -> Result<(), String> {
    let running = {
        let mut guard = state.running.lock().map_err(|_| "进程管理器加锁失败")?;
        guard.remove(&script)
    };

    let Some(mut running) = running else {
        return Ok(());
    };

    killer::kill_tree(&mut running.child)?;
    emit_state(&app, &script, "exited", None, None);
    Ok(())
}

/// 后台观察某个脚本的进程退出，推送退出码。
fn watch_exit(app: AppHandle, script: String) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(250));

        let manager = app.state::<ProcessManager>();
        let mut guard = match manager.running.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        let status = match guard.get_mut(&script) {
            Some(running) => running.child.try_wait(),
            None => return,
        };

        match status {
            Ok(Some(exit)) => {
                guard.remove(&script);
                drop(guard);
                emit_state(&app, &script, "exited", None, exit.code());
                return;
            }
            Ok(None) => {}
            Err(_) => return,
        }
    });
}

fn emit_state(app: &AppHandle, script: &str, state: &str, pid: Option<u32>, code: Option<i32>) {
    let _ = app.emit(
        "process://state",
        ProcessStatePayload {
            script: script.to_string(),
            state: state.to_string(),
            pid,
            code,
        },
    );
}
