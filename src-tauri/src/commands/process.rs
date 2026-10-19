//! 进程启动 / 停止 / 状态推送（见技术方案 §4.4）。

use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::process::{killer, reader, spawn, ProcessManager, Running};

#[derive(Clone, serde::Serialize)]
pub struct ProcessStatePayload {
    pub state: String,
    pub pid: Option<u32>,
    pub code: Option<i32>,
}

/// 启动脚本：不弹控制台窗口，接管输出并推送状态。
#[tauri::command]
pub fn start_process(
    app: AppHandle,
    state: State<ProcessManager>,
    script: String,
) -> Result<u32, String> {
    {
        let mut guard = state.running.lock().map_err(|_| "进程管理器加锁失败")?;
        if let Some(running) = guard.as_mut() {
            match running.child.try_wait() {
                Ok(None) => return Err("已有脚本在运行，请先停止".to_string()),
                _ => *guard = None,
            }
        }
    }

    let mut child = spawn::spawn(Path::new(&script)).map_err(|e| format!("启动失败：{e}"))?;
    let pid = child.id();

    if let Some(out) = child.stdout.take() {
        reader::pump(app.clone(), "stdout", out);
    }
    if let Some(err) = child.stderr.take() {
        reader::pump(app.clone(), "stderr", err);
    }

    {
        let mut guard = state.running.lock().map_err(|_| "进程管理器加锁失败")?;
        *guard = Some(Running { child });
    }

    emit_state(&app, "running", Some(pid), None);
    watch_exit(app);
    Ok(pid)
}

/// 停止运行中的进程（含子树）。
#[tauri::command]
pub fn stop_process(app: AppHandle, state: State<ProcessManager>) -> Result<(), String> {
    let mut running = {
        let mut guard = state.running.lock().map_err(|_| "进程管理器加锁失败")?;
        guard.take()
    };

    let Some(running) = running.as_mut() else {
        return Ok(());
    };

    killer::kill_tree(&mut running.child)?;
    emit_state(&app, "exited", None, None);
    Ok(())
}

/// 后台观察进程退出，推送退出码。
fn watch_exit(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(250));

        let manager = app.state::<ProcessManager>();
        let mut guard = match manager.running.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        let status = {
            let Some(running) = guard.as_mut() else {
                return;
            };
            running.child.try_wait()
        };

        match status {
            Ok(Some(exit)) => {
                *guard = None;
                drop(guard);
                emit_state(&app, "exited", None, exit.code());
                return;
            }
            Ok(None) => {}
            Err(_) => return,
        }
    });
}

fn emit_state(app: &AppHandle, state: &str, pid: Option<u32>, code: Option<i32>) {
    let _ = app.emit(
        "process://state",
        ProcessStatePayload {
            state: state.to_string(),
            pid,
            code,
        },
    );
}
